//! Data transfer: import/export CSV de productos. El export reutiliza el
//! CSV del datagrid; el import valida y crea productos + inventario por
//! chunks, reportando filas inválidas (mirrors `Webkul\DataTransfer`).

use larust_support::axum::http::StatusCode;
use larust_support::axum::response::IntoResponse;
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::db;
use crate::support::response::{csv_response, resource, validation_error};

pub struct AdminDataTransferController;

/// Parsea un CSV simple con cabecera (sin comillas anidadas; los campos del
/// formato de productos no las requieren).
fn parse_csv(text: &str) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header: Vec<String> = lines
        .next()?
        .split(',')
        .map(|cell| cell.trim().to_lowercase())
        .collect();
    let rows = lines
        .map(|line| {
            line.split(',')
                .map(|cell| cell.trim().to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    Some((header, rows))
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

impl AdminDataTransferController {
    /// `POST /api/admin/data-transfer/products/export` - CSV de productos
    /// visibles: sku,name,price,quantity,status,url_key,type.
    pub async fn export_products(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT `p`.`sku`, `p`.`type`, `pf`.`name`, `pf`.`price`, `pf`.`url_key`, `pf`.`status`, \
                    COALESCE(`pfi`.`qty`, 0) AS quantity \
             FROM `products` `p` \
             LEFT JOIN `product_flat` `pf` ON `pf`.`product_id` = `p`.`id` \
             LEFT JOIN `product_inventory_indices` `pfi` ON `pfi`.`product_id` = `p`.`id` \
             WHERE `p`.`parent_id` IS NULL ORDER BY `p`.`id`",
            &[],
        )
        .await
        .unwrap_or_default();

        let mut csv = String::from("sku,name,type,price,quantity,status,url_key\n");
        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                csv_escape(row["sku"].as_str().unwrap_or_default()),
                csv_escape(row["name"].as_str().unwrap_or_default()),
                csv_escape(row["type"].as_str().unwrap_or_default()),
                row["price"].as_f64().unwrap_or(0.0),
                row["quantity"].as_i64().unwrap_or(0),
                row["status"].as_i64().unwrap_or(0),
                csv_escape(row["url_key"].as_str().unwrap_or_default()),
            ));
        }

        (
            StatusCode::OK,
            [
                (larust_support::axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (
                    larust_support::axum::http::header::CONTENT_DISPOSITION,
                    "attachment; filename=\"products.csv\"",
                ),
            ],
            csv,
        )
            .into_response()
    }

    /// `POST /api/admin/data-transfer/products/import` - `{csv: "sku,name,..."}`
    /// o `{rows: [{sku, name, price, quantity, status}]}`. Validación por
    /// filas: las inválidas no interrumpen el proceso y se reportan.
    pub async fn import_products(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let now = db::now_db();

        let rows: Vec<Value> = if let Some(csv) = body["csv"].as_str() {
            let Some((header, lines)) = parse_csv(csv) else {
                return validation_error(vec![("csv".into(), "The CSV is empty or malformed.".into())]);
            };
            let index = |column: &str| header.iter().position(|entry| entry == column);
            let (sku_i, name_i, price_i, qty_i, status_i) = (
                index("sku"), index("name"), index("price"), index("quantity"), index("status"),
            );
            lines
                .into_iter()
                .map(|cells| {
                    json!({
                        "sku": sku_i.and_then(|i| cells.get(i)).cloned(),
                        "name": name_i.and_then(|i| cells.get(i)).cloned(),
                        "price": price_i.and_then(|i| cells.get(i)).and_then(|cell| cell.parse::<f64>().ok()),
                        "quantity": qty_i.and_then(|i| cells.get(i)).and_then(|cell| cell.parse::<i64>().ok()),
                        "status": status_i.and_then(|i| cells.get(i)).and_then(|cell| cell.parse::<i64>().ok()),
                    })
                })
                .collect()
        } else {
            body["rows"].as_array().cloned().unwrap_or_default()
        };

        let mut processed = 0i64;
        let mut invalid_rows_count = 0i64;
        let mut errors: Vec<Value> = Vec::new();

        for (position, row) in rows.iter().enumerate() {
            let Some(sku) = row["sku"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
                invalid_rows_count += 1;
                errors.push(json!({"row": position + 1, "error": "The sku field is required."}));
                continue;
            };
            let exists = db::select_one(
                "SELECT `id` FROM `products` WHERE `sku` = ? LIMIT 1",
                &[Value::String(sku.to_string())],
            )
            .await
            .ok()
            .flatten();
            if exists.is_some() {
                invalid_rows_count += 1;
                errors.push(json!({"row": position + 1, "error": format!("The sku {sku} already exists.")}));
                continue;
            }

            let price = row["price"].as_f64().unwrap_or(0.0);
            let quantity = row["quantity"].as_i64().unwrap_or(0);
            let status = row["status"].as_i64().unwrap_or(1);
            let name = row["name"].as_str().unwrap_or(sku).to_string();

            match db::insert(
                "products",
                &json!({
                    "sku": sku,
                    "type": "simple",
                    "attribute_family_id": 1,
                    "created_at": now,
                    "updated_at": now,
                }),
            )
            .await
            {
                Ok(product_id) => {
                    // Atributos base + inventario + flat/índices.
                    for (code, value) in [
                        ("sku", Value::String(sku.to_string())),
                        ("name", Value::String(name.clone())),
                        ("price", json!(price)),
                        ("status", json!(status)),
                        ("visible_individually", json!(1)),
                        ("manage_stock", json!(1)),
                    ] {
                        let attribute_id: i64 = db::scalar_i64(
                            "SELECT `id` FROM `attributes` WHERE `code` = ? LIMIT 1",
                            &[Value::String(code.to_string())],
                        )
                        .await
                        .unwrap_or(0);
                        if attribute_id == 0 {
                            continue;
                        }
                        let value_json = if code == "price" || code == "manage_stock" {
                            json!({"float_value": value.as_f64().unwrap_or(0.0)})
                        } else if code == "status" || code == "visible_individually" {
                            json!({"boolean_value": value.as_i64().unwrap_or(0)})
                        } else {
                            json!({"text_value": value.as_str().unwrap_or_default()})
                        };
                        let mut payload = value_json;
                        payload["product_id"] = json!(product_id);
                        payload["attribute_id"] = json!(attribute_id);
                        let _ = db::insert("product_attribute_values", &payload).await;
                    }
                    let _ = db::insert(
                        "product_inventories",
                        &json!({
                            "qty": quantity,
                            "product_id": product_id,
                            "inventory_source_id": 1,
                            "created_at": now,
                            "updated_at": now,
                        })
                        ,
                    )
                    .await;
                    crate::support::products::refresh_flat(product_id).await;
                    processed += 1;
                }
                Err(error) => {
                    invalid_rows_count += 1;
                    errors.push(json!({"row": position + 1, "error": error.to_string()}));
                }
            }
        }

        resource(
            json!({
                "state": if invalid_rows_count > 0 { "completed_with_errors" } else { "completed" },
                "processed_rows_count": processed,
                "invalid_rows_count": invalid_rows_count,
                "errors": errors,
            }),
            format!("{processed} rows imported, {invalid_rows_count} invalid."),
        )
    }

    /// `GET /api/admin/data-transfer/export-customers` - CSV de clientes.
    pub async fn export_customers(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT `id`, `first_name`, `last_name`, `email`, `status`, `customer_group_id` \
             FROM `customers` ORDER BY `id`",
            &[],
        )
        .await
        .unwrap_or_default();
        let mut csv = String::from("id,first_name,last_name,email,status,group_id\n");
        for row in &rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                row["id"].as_i64().unwrap_or(0),
                row["first_name"].as_str().unwrap_or_default().replace(',', " "),
                row["last_name"].as_str().unwrap_or_default().replace(',', " "),
                row["email"].as_str().unwrap_or_default(),
                row["status"].as_i64().unwrap_or(0),
                row["customer_group_id"].as_i64().unwrap_or(0),
            ));
        }
        csv_response(&csv, "customers.csv")
    }

    /// `GET /api/admin/data-transfer/export-orders` - CSV de pedidos.
    pub async fn export_orders(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT `increment_id`, `status`, `customer_email`, `base_grand_total`, \
                    `base_tax_amount`, `base_shipping_amount`, `created_at` \
             FROM `orders` ORDER BY `id`",
            &[],
        )
        .await
        .unwrap_or_default();
        let mut csv = String::from("increment_id,status,customer_email,grand_total,tax,shipping,created_at\n");
        for row in &rows {
            csv.push_str(&format!(
                "{},{},{},{:.2},{:.2},{:.2},{}\n",
                row["increment_id"].as_str().unwrap_or_default(),
                row["status"].as_str().unwrap_or_default(),
                row["customer_email"].as_str().unwrap_or_default().replace(',', " "),
                row["base_grand_total"].as_f64().unwrap_or(0.0),
                row["base_tax_amount"].as_f64().unwrap_or(0.0),
                row["base_shipping_amount"].as_f64().unwrap_or(0.0),
                row["created_at"].as_str().unwrap_or_default(),
            ));
        }
        csv_response(&csv, "orders.csv")
    }
}
