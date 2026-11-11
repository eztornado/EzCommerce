//! RMA (devoluciones) y GDPR (solicitudes de datos) — núcleo de fase 3
//! verificable sin servicios externos. Mirrors `Webkul\RMA` y
//! `Webkul\GDPR`:
//! - Cliente: crea solicitud RMA sobre sus pedidos (items + resolución),
//!   lista las suyas; solicita export/borrado de sus datos (GDPR) y
//!   descarga el export.
//! - Admin: lista/cambia estado de RMAs y solicitudes GDPR.

use std::collections::HashMap;

use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::StatusCode;
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Map, Value};

use crate::support::auth::{Admin, Customer};
use crate::support::db;
use crate::support::response::{message, resource, validation_error};

pub struct RmaGdprController;

fn now() -> String {
    db::now_db()
}

impl RmaGdprController {
    // ------------------------------------------------------------------
    // RMA (cliente)
    // ------------------------------------------------------------------

    /// `POST /api/customer/rma/store` -
    /// `{order_id, resolution, package_condition, information, items: [{order_item_id, quantity, reason}]}`
    /// — sólo para pedidos propios y con entregas ya facturadas/enviadas.
    pub async fn store_rma(customer: Customer, Json(body): Json<Value>) -> Response {
        let Some(order_id) = body["order_id"].as_i64() else {
            return validation_error(vec![("order_id".into(), "The order field is required.".into())]);
        };
        let owned = db::select_one(
            "SELECT `id` FROM `orders` WHERE `id` = ? AND `customer_id` = ? LIMIT 1",
            &[Value::from(order_id), Value::from(customer.id)],
        )
        .await
        .ok()
        .flatten();
        if owned.is_none() {
            return message(StatusCode::NOT_FOUND, "Order not found.");
        }
        // Una solicitud abierta por pedido.
        let open = db::select_one(
            "SELECT `id` FROM `rma` WHERE `order_id` = ? LIMIT 1",
            &[Value::from(order_id)],
        )
        .await
        .ok()
        .flatten();
        if open.is_some() {
            return message(
                StatusCode::UNPROCESSABLE_ENTITY,
                "An RMA already exists for this order.",
            );
        }

        let default_status: i64 = db::scalar_i64(
            "SELECT `id` FROM `rma_statuses` ORDER BY `id` LIMIT 1",
            &[],
        )
        .await
        .unwrap_or(0);

        let rma_id = match db::insert(
            "rma",
            &json!({
                "order_id": order_id,
                "rma_status_id": if default_status > 0 { json!(default_status) } else { Value::Null },
                "package_condition": body["package_condition"].clone(),
                "information": body["information"].clone(),
                "created_at": now(),
                "updated_at": now(),
            })
            ,
        )
        .await
        {
            Ok(id) => id,
            Err(error) => return error.into(),
        };

        let items = body["items"].as_array().cloned().unwrap_or_default();
        for item in &items {
            let Some(order_item_id) = item["order_item_id"].as_i64() else { continue };
            let order_item = db::find_by_id("order_items", order_item_id)
                .await
                .ok()
                .flatten();
            let Some(order_item) = order_item else { continue };
            if order_item["order_id"].as_i64() != Some(order_id) {
                continue;
            }
            let quantity = item["quantity"]
                .as_i64()
                .unwrap_or(order_item["qty_ordered"].as_i64().unwrap_or(1))
                .clamp(1, order_item["qty_ordered"].as_i64().unwrap_or(1));
            let _ = db::insert(
                "rma_items",
                &json!({
                    "rma_id": rma_id,
                    "order_item_id": order_item_id,
                    "quantity": quantity,
                    "resolution": item["resolution"].clone(),
                    "created_at": now(),
                    "updated_at": now(),
                })
                ,
            )
            .await;
        }

        resource(json!({"id": rma_id}), "The RMA has been created successfully.")
    }

    /// `GET /api/customer/rma` - las solicitudes del cliente.
    pub async fn my_rmas(customer: Customer, Query(query): Query<HashMap<String, String>>) -> Response {
        let page = query.get("page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(1).max(1);
        let per_page = query.get("per_page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(10).clamp(1, 100);
        let total = db::scalar_i64(
            "SELECT COUNT(*) FROM `rma` `r` INNER JOIN `orders` `o` ON `o`.`id` = `r`.`order_id` WHERE `o`.`customer_id` = ?",
            &[Value::from(customer.id)],
        )
        .await
        .unwrap_or(0);
        let offset = (page - 1) * per_page;
        let rows = db::select(
            "SELECT `r`.*, `rs`.`title` AS status_title, `rs`.`color` AS status_color, `o`.`increment_id` \
             FROM `rma` `r` \
             INNER JOIN `orders` `o` ON `o`.`id` = `r`.`order_id` \
             LEFT JOIN `rma_statuses` `rs` ON `rs`.`id` = `r`.`rma_status_id` \
             WHERE `o`.`customer_id` = ? ORDER BY `r`.`id` DESC LIMIT ? OFFSET ?",
            &[Value::from(customer.id), Value::from(per_page), Value::from(offset)],
        )
        .await
        .unwrap_or_default();
        resource(
            json!({
                "data": rows,
                "meta": {
                    "current_page": page,
                    "last_page": ((total as f64) / (per_page as f64)).ceil() as i64,
                    "per_page": per_page,
                    "total": total,
                }
            }),
            Value::Null,
        )
    }

    /// `GET /api/customer/rma/view/{id}` - detalle con items.
    pub async fn view_rma(customer: Customer, Path(id): Path<i64>) -> Response {
        let rma = db::select_one(
            "SELECT `r`.*, `rs`.`title` AS status_title FROM `rma` `r` \
             INNER JOIN `orders` `o` ON `o`.`id` = `r`.`order_id` \
             LEFT JOIN `rma_statuses` `rs` ON `rs`.`id` = `r`.`rma_status_id` \
             WHERE `r`.`id` = ? AND `o`.`customer_id` = ? LIMIT 1",
            &[Value::from(id), Value::from(customer.id)],
        )
        .await
        .ok()
        .flatten();
        let Some(rma) = rma else {
            return message(StatusCode::NOT_FOUND, "RMA not found.");
        };
        let items = db::select(
            "SELECT `ri`.*, `oi`.`name`, `oi`.`sku` FROM `rma_items` `ri` \
             LEFT JOIN `order_items` `oi` ON `oi`.`id` = `ri`.`order_item_id` \
             WHERE `ri`.`rma_id` = ? ORDER BY `ri`.`id`",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        let mut payload = rma;
        payload.insert("items".into(), Value::Array(items.into_iter().map(Value::Object).collect()));
        resource(payload, Value::Null)
    }

    // ------------------------------------------------------------------
    // RMA (admin)
    // ------------------------------------------------------------------

    /// `GET /api/admin/rma` - listado paginado.
    pub async fn admin_rmas(admin: Admin, Query(query): Query<HashMap<String, String>>) -> Response {
        let _ = admin;
        let page = query.get("page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(1).max(1);
        let per_page = query.get("per_page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(10).clamp(1, 100);
        let total = db::scalar_i64("SELECT COUNT(*) FROM `rma`", &[]).await.unwrap_or(0);
        let offset = (page - 1) * per_page;
        let rows = db::select(
            "SELECT `r`.*, `rs`.`title` AS status_title, `o`.`increment_id` FROM `rma` `r` \
             INNER JOIN `orders` `o` ON `o`.`id` = `r`.`order_id` \
             LEFT JOIN `rma_statuses` `rs` ON `rs`.`id` = `r`.`rma_status_id` \
             ORDER BY `r`.`id` DESC LIMIT ? OFFSET ?",
            &[Value::from(per_page), Value::from(offset)],
        )
        .await
        .unwrap_or_default();
        resource(
            json!({
                "data": rows,
                "meta": {"current_page": page, "last_page": ((total as f64) / (per_page as f64)).ceil() as i64, "per_page": per_page, "total": total}
            }),
            Value::Null,
        )
    }

    /// `POST /api/admin/rma/status/{id}` - `{rma_status_id}`.
    pub async fn admin_update_status(admin: Admin, Path(id): Path<i64>, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(status_id) = body["rma_status_id"].as_i64() else {
            return validation_error(vec![("rma_status_id".into(), "The status field is required.".into())]);
        };
        match db::update(
            "rma",
            id,
            &json!({"rma_status_id": status_id, "updated_at": now()})
            ,
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The RMA status has been updated."),
            Err(error) => error.into(),
        }
    }

    // ------------------------------------------------------------------
    // GDPR (cliente)
    // ------------------------------------------------------------------

    /// `POST /api/customer/gdpr/request` - `{type: export|delete, message?}`.
    pub async fn store_gdpr_request(customer: Customer, Json(body): Json<Value>) -> Response {
        let request_type = body["type"].as_str().unwrap_or("export").to_string();
        if !["export", "delete"].contains(&request_type.as_str()) {
            return validation_error(vec![("type".into(), "The type must be export or delete.".into())]);
        }
        match db::insert(
            "gdpr_data_request",
            &json!({
                "customer_id": customer.id,
                "email": customer.email,
                "status": "pending",
                "type": request_type,
                "message": body["message"].clone(),
                "created_at": now(),
                "updated_at": now(),
            })
            ,
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "Your request has been submitted successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/customer/gdpr` - mis solicitudes.
    pub async fn my_gdpr_requests(customer: Customer) -> Response {
        let rows = db::select(
            "SELECT * FROM `gdpr_data_request` WHERE `customer_id` = ? ORDER BY `id` DESC",
            &[Value::from(customer.id)],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `GET /api/customer/gdpr/view/{id}` - export de los datos propios
    /// (sólo si la solicitud `export` fue aprobada).
    pub async fn view_gdpr_export(customer: Customer, Path(id): Path<i64>) -> Response {
        let request = db::select_one(
            "SELECT * FROM `gdpr_data_request` WHERE `id` = ? AND `customer_id` = ? LIMIT 1",
            &[Value::from(id), Value::from(customer.id)],
        )
        .await
        .ok()
        .flatten();
        let Some(request) = request else {
            return message(StatusCode::NOT_FOUND, "Request not found.");
        };
        if request["type"].as_str() != Some("export") || request["status"].as_str() != Some("approved") {
            return message(
                StatusCode::UNPROCESSABLE_ENTITY,
                "The request must be an approved export.",
            );
        }
        let profile = db::find_by_id("customers", customer.id)
            .await
            .ok()
            .flatten()
            .map(|mut row| {
                row.remove("password");
                row.remove("remember_token");
                row.remove("api_token");
                row.remove("token");
                row
            })
            .unwrap_or_else(|| json!({}).as_object().cloned().unwrap_or_default());
        let orders = db::select(
            "SELECT * FROM `orders` WHERE `customer_id` = ?",
            &[Value::from(customer.id)],
        )
        .await
        .unwrap_or_default();
        let addresses = db::select(
            "SELECT * FROM `addresses` WHERE `customer_id` = ?",
            &[Value::from(customer.id)],
        )
        .await
        .unwrap_or_default();
        resource(
            json!({
                "customer": profile,
                "orders": orders,
                "addresses": addresses,
            }),
            Value::Null,
        )
    }

    // ------------------------------------------------------------------
    // GDPR (admin)
    // ------------------------------------------------------------------

    /// `GET /api/admin/gdpr` - solicitudes.
    pub async fn admin_gdpr_requests(admin: Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT `g`.*, `c`.`email` FROM `gdpr_data_request` `g` \
             LEFT JOIN `customers` `c` ON `c`.`id` = `g`.`customer_id` ORDER BY `g`.`id` DESC",
            &[],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `POST /api/admin/gdpr/status/{id}` - `{status: pending|approved|declined}`.
    /// Un `delete` aprobado anonimiza los datos del cliente.
    pub async fn admin_update_gdpr_status(
        admin: Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let status = body["status"].as_str().unwrap_or("pending").to_string();
        if !["pending", "approved", "declined"].contains(&status.as_str()) {
            return validation_error(vec![("status".into(), "Invalid status.".into())]);
        }
        let request = db::find_by_id("gdpr_data_request", id).await.ok().flatten();
        let Some(request) = request else {
            return message(StatusCode::NOT_FOUND, "Request not found.");
        };

        if status == "approved" && request["type"].as_str() == Some("delete") {
            let Some(customer_id) = request["customer_id"].as_i64() else {
                return message(StatusCode::UNPROCESSABLE_ENTITY, "The request has no customer.");
            };
            // Anonimización (borrado suave de identidad, conserva pedidos).
            let _ = db::execute(
                "UPDATE `customers` SET `first_name` = 'Deleted', `last_name` = 'User', \
                 `email` = CONCAT('deleted-', `id`, '@anonymous.local'), `phone` = NULL, `image` = NULL, \
                 `date_of_birth` = NULL, `gender` = NULL, `updated_at` = ? WHERE `id` = ?",
                &[Value::String(now()), Value::from(customer_id)],
            )
            .await;
            let _ = db::execute(
                "DELETE FROM `addresses` WHERE `customer_id` = ? AND `address_type` = 'customer_address'",
                &[Value::from(customer_id)],
            )
            .await;
        }

        match db::update(
            "gdpr_data_request",
            id,
            &json!({
                "status": status,
                "revoked_at": if status == "declined" { json!(now()) } else { Value::Null },
                "updated_at": now(),
            })
            ,
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The request status has been updated."),
            Err(error) => error.into(),
        }
    }
}
