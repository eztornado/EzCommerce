//! Reporting: sales/products/customers con rangos y granularidad
//! (`start`, `end`, `granularity=day|week|month`). Mirrors las fórmulas de
//! `Webkul\Admin\Helpers\Reporting\*`:
//! - total_sales = SUM(base_grand_total_invoiced - base_grand_total_refunded)
//! - tax_collected = SUM(base_tax_amount_invoiced)
//! - shipping_collected = SUM(base_shipping_invoiced)
//! - refunds = SUM(base_grand_total_refunded)
//! - coupon_discount = SUM(base_discount_amount) donde coupon_code no nulo

use std::collections::HashMap;

use chrono::Datelike;

use larust_support::axum::extract::Query;
use larust_support::axum::response::Response;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::db;
use crate::support::response::resource;

pub struct AdminReportingController;

/// Rango por defecto: últimos 30 días. Devuelve (start, end) como fechas.
fn period(query: &HashMap<String, String>) -> (String, String) {
    let end = query
        .get("end")
        .cloned()
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
    let start = query
        .get("start")
        .cloned()
        .unwrap_or_else(|| {
            (chrono::Utc::now() - chrono::Duration::days(30))
                .format("%Y-%m-%d")
                .to_string()
        });
    (format!("{start} 00:00:00"), format!("{end} 23:59:59"))
}

/// Agrupa por día (`substr` es compatible SQLite/MySQL sobre VARCHAR).
fn date_expr(granularity: &str) -> String {
    match granularity {
        "month" => "substr(`created_at`, 1, 7)".to_string(),
        _ => "substr(`created_at`, 1, 10)".to_string(), // day (week se computa en el cliente)
    }
}

fn bucket(granularity: &str, label: &str) -> String {
    match granularity {
        "month" => label.chars().take(7).collect(),
        "week" => {
            // ISO week aproximada desde YYYY-MM-DD.
            if let Ok(date) = chrono::NaiveDate::parse_from_str(label, "%Y-%m-%d") {
                let monday = date - chrono::Duration::days(date.weekday().num_days_from_monday() as i64);
                monday.format("%Y-%m-%d").to_string()
            } else {
                label.chars().take(10).collect()
            }
        }
        _ => label.chars().take(10).collect(),
    }
}

fn money(row: &larust_support::serde_json::Map<String, Value>, field: &str) -> f64 {
    row[field].as_f64().unwrap_or(0.0)
}

impl AdminReportingController {
    /// `GET /api/admin/reporting/sales`.
    pub async fn sales(admin: auth::Admin, Query(query): Query<HashMap<String, String>>) -> Response {
        let _ = admin;
        let (start, end) = period(&query);
        let granularity = query.get("granularity").map(String::as_str).unwrap_or("day");
        let group_expr = date_expr(granularity);

        let totals = db::select_one(
            "SELECT \
                CAST(COALESCE(SUM(`base_grand_total_invoiced` - `base_grand_total_refunded`), 0) * 100 AS INTEGER) AS total_sales, \
                COUNT(*) AS total_orders, \
                CAST(COALESCE(SUM(`base_tax_amount_invoiced`), 0) * 100 AS INTEGER) AS tax_collected, \
                CAST(COALESCE(SUM(`base_shipping_invoiced`), 0) * 100 AS INTEGER) AS shipping_collected, \
                CAST(COALESCE(SUM(`base_grand_total_refunded`), 0) * 100 AS INTEGER) AS refunds, \
                CAST(COALESCE(SUM(CASE WHEN `coupon_code` IS NOT NULL THEN `base_discount_amount` ELSE 0 END), 0) * 100 AS INTEGER) AS coupon_discount \
             FROM `orders` WHERE `created_at` BETWEEN ? AND ?",
            &[Value::String(start.clone()), Value::String(end.clone())],
        )
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| json!({}).as_object().cloned().unwrap_or_default());

        let series = db::select(
            &format!(
                "SELECT {} AS day, \
                        CAST(COALESCE(SUM(`base_grand_total_invoiced` - `base_grand_total_refunded`), 0) * 100 AS INTEGER) AS sales, \
                        COUNT(*) AS orders \
                 FROM `orders` WHERE `created_at` BETWEEN ? AND ? \
                 GROUP BY {} ORDER BY day",
                group_expr, group_expr
            ),
            &[Value::String(start), Value::String(end)],
        )
        .await
        .unwrap_or_default();

        // Agrupación semanal en Rust desde la serie diaria.
        let mut buckets: Vec<(String, f64, i64)> = Vec::new();
        for row in &series {
            let label = row["day"].as_str().unwrap_or_default().to_string();
            let key = bucket(granularity, &label);
            let sales = row["sales"].as_f64().unwrap_or(0.0);
            let orders = row["orders"].as_i64().unwrap_or(0);
            match buckets.last_mut() {
                Some((last_key, last_sales, last_orders)) if *last_key == key => {
                    *last_sales += sales;
                    *last_orders += orders;
                }
                _ => buckets.push((key, sales, orders)),
            }
        }

        resource(
            json!({
                "totals": {
                    "total_sales": money(&totals, "total_sales") / 100.0,
                    "total_orders": totals["total_orders"].as_i64().unwrap_or(0),
                    "tax_collected": money(&totals, "tax_collected") / 100.0,
                    "shipping_collected": money(&totals, "shipping_collected") / 100.0,
                    "refunds": money(&totals, "refunds") / 100.0,
                    "coupon_discount": money(&totals, "coupon_discount") / 100.0,
                },
                "series": buckets.iter().map(|(label, sales, orders)| json!({
                    "label": label,
                    "sales": sales / 100.0,
                    "orders": orders,
                })).collect::<Vec<_>>(),
            }),
            Value::Null,
        )
    }

    /// `GET /api/admin/reporting/products` - top por unidades e importe.
    pub async fn products(admin: auth::Admin, Query(query): Query<HashMap<String, String>>) -> Response {
        let _ = admin;
        let (start, end) = period(&query);
        let rows = db::select(
            "SELECT `oi`.`product_id`, `oi`.`name`, `oi`.`sku`, \
                    SUM(`oi`.`qty_ordered`) AS qty, \
                    CAST(COALESCE(SUM(`oi`.`base_total`), 0) * 100 AS INTEGER) AS total \
             FROM `order_items` `oi` INNER JOIN `orders` `o` ON `o`.`id` = `oi`.`order_id` \
             WHERE `o`.`created_at` BETWEEN ? AND ? \
             GROUP BY `oi`.`product_id`, `oi`.`name`, `oi`.`sku` ORDER BY total DESC LIMIT 10",
            &[Value::String(start), Value::String(end)],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `GET /api/admin/reporting/customers` - top por gasto.
    pub async fn customers(admin: auth::Admin, Query(query): Query<HashMap<String, String>>) -> Response {
        let _ = admin;
        let (start, end) = period(&query);
        let rows = db::select(
            "SELECT `c`.`id`, `c`.`email`, `c`.`first_name`, `c`.`last_name`, \
                    COUNT(`o`.`id`) AS orders, \
                    CAST(COALESCE(SUM(`o`.`base_grand_total`), 0) * 100 AS INTEGER) AS total \
             FROM `customers` `c` INNER JOIN `orders` `o` ON `o`.`customer_id` = `c`.`id` \
             WHERE `o`.`created_at` BETWEEN ? AND ? \
             GROUP BY `c`.`id` ORDER BY total DESC LIMIT 10",
            &[Value::String(start), Value::String(end)],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }
}
