//! Admin dashboard (`/api/admin/dashboard/stats?type=...`). Mirrors
//! `Webkul\Admin\Helpers\Dashboard` formulas.

use std::collections::HashMap;

use larust_support::axum::extract::Query;
use larust_support::axum::response::Response;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::db;
use crate::support::response::resource;

pub struct AdminDashboardController;

fn period(query: &HashMap<String, String>) -> (String, String, String, String) {
    // `start`/`end` (YYYY-MM-DD) with the previous symmetric period.
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
    let start_ts = chrono::NaiveDate::parse_from_str(&start, "%Y-%m-%d")
        .map(|date| date.and_hms_opt(0, 0, 0).unwrap())
        .ok();
    let end_ts = chrono::NaiveDate::parse_from_str(&end, "%Y-%m-%d")
        .map(|date| date.and_hms_opt(23, 59, 59).unwrap())
        .ok();
    let (prev_start, prev_end) = match (start_ts, end_ts) {
        (Some(start), Some(end)) => {
            let span = end - start;
            (
                (start - span).format("%Y-%m-%d %H:%M:%S").to_string(),
                (start - chrono::Duration::seconds(1))
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            )
        }
        _ => (String::new(), String::new()),
    };
    (
        format!("{start} 00:00:00"),
        format!("{end} 23:59:59"),
        prev_start,
        prev_end,
    )
}

fn progress(current: f64, previous: f64) -> f64 {
    if previous == 0.0 {
        return if current > 0.0 { 100.0 } else { 0.0 };
    }
    ((current - previous) / previous * 100.0 * 100.0).round() / 100.0
}

impl AdminDashboardController {
    /// `GET /api/admin/dashboard/stats?type=over-all|today|total-sales|top-selling-products|top-customers|stock-threshold-products`
    pub async fn stats(admin: auth::Admin, Query(query): Query<HashMap<String, String>>) -> Response {
        let _ = admin;
        let (start, end, prev_start, prev_end) = period(&query);
        let kind = query.get("type").map(String::as_str).unwrap_or("over-all");

        let response = match kind {
            "over-all" => {
                let sales_current: f64 = db::scalar_i64(
                    "SELECT CAST(COALESCE(SUM(`base_grand_total_invoiced` - `base_grand_total_refunded`), 0) * 100 AS INTEGER) \
                     FROM `orders` WHERE `created_at` BETWEEN ? AND ?",
                    &[Value::String(start.clone()), Value::String(end.clone())],
                )
                .await
                .unwrap_or(0) as f64
                    / 100.0;
                let sales_previous: f64 = db::scalar_i64(
                    "SELECT CAST(COALESCE(SUM(`base_grand_total_invoiced` - `base_grand_total_refunded`), 0) * 100 AS INTEGER) \
                     FROM `orders` WHERE `created_at` BETWEEN ? AND ?",
                    &[Value::String(prev_start.clone()), Value::String(prev_end.clone())],
                )
                .await
                .unwrap_or(0) as f64
                    / 100.0;
                let orders_current = db::scalar_i64(
                    "SELECT COUNT(*) FROM `orders` WHERE `created_at` BETWEEN ? AND ?",
                    &[Value::String(start.clone()), Value::String(end.clone())],
                )
                .await
                .unwrap_or(0);
                let orders_previous = db::scalar_i64(
                    "SELECT COUNT(*) FROM `orders` WHERE `created_at` BETWEEN ? AND ?",
                    &[Value::String(prev_start.clone()), Value::String(prev_end.clone())],
                )
                .await
                .unwrap_or(0);
                let customers_current = db::scalar_i64(
                    "SELECT COUNT(*) FROM `customers` WHERE `created_at` BETWEEN ? AND ?",
                    &[Value::String(start), Value::String(end)],
                )
                .await
                .unwrap_or(0);
                let customers_previous = db::scalar_i64(
                    "SELECT COUNT(*) FROM `customers` WHERE `created_at` BETWEEN ? AND ?",
                    &[Value::String(prev_start.clone()), Value::String(prev_end.clone())],
                )
                .await
                .unwrap_or(0);
                let avg = if orders_current > 0 { sales_current / orders_current as f64 } else { 0.0 };

                json!({
                    "total_sales": {"current": sales_current, "previous": sales_previous, "progress": progress(sales_current, sales_previous)},
                    "total_orders": {"current": orders_current, "previous": orders_previous, "progress": progress(orders_current as f64, orders_previous as f64)},
                    "total_customers": {"current": customers_current, "previous": customers_previous, "progress": progress(customers_current as f64, customers_previous as f64)},
                    "avg_sales": {"current": avg, "progress": progress(avg, if orders_previous > 0 { sales_previous / orders_previous as f64 } else { 0.0 })},
                })
            }
            "today" => {
                let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                let sales: f64 = db::scalar_i64(
                    "SELECT CAST(COALESCE(SUM(`base_grand_total`), 0) * 100 AS INTEGER) FROM `orders` WHERE DATE(`created_at`) = ?",
                    &[Value::String(today.clone())],
                )
                .await
                .unwrap_or(0) as f64
                    / 100.0;
                let orders = db::scalar_i64(
                    "SELECT COUNT(*) FROM `orders` WHERE DATE(`created_at`) = ?",
                    &[Value::String(today.clone())],
                )
                .await
                .unwrap_or(0);
                let customers = db::scalar_i64(
                    "SELECT COUNT(*) FROM `customers` WHERE DATE(`created_at`) = ?",
                    &[Value::String(today.clone())],
                )
                .await
                .unwrap_or(0);
                let recent = db::select(
                    "SELECT `id`, `increment_id`, `status`, `customer_email`, `customer_first_name`, `customer_last_name`, `base_grand_total`, `created_at` \
                     FROM `orders` WHERE DATE(`created_at`) = ? ORDER BY `id` DESC LIMIT 5",
                    &[Value::String(today)],
                )
                .await
                .unwrap_or_default();
                json!({
                    "sales_today": sales,
                    "orders_today": orders,
                    "customers_today": customers,
                    "recent_orders": recent,
                })
            }
            "total-sales" => {
                // Series over time (daily).
                let rows = db::select(
                    "SELECT DATE(`created_at`) AS day, \
                            CAST(COALESCE(SUM(`base_grand_total`), 0) * 100 AS INTEGER) AS total, \
                            COUNT(*) AS orders \
                     FROM `orders` WHERE `created_at` BETWEEN ? AND ? GROUP BY DATE(`created_at`) ORDER BY day",
                    &[Value::String(start), Value::String(end)],
                )
                .await
                .unwrap_or_default();
                json!({
                    "sales": rows.iter().map(|row| json!({
                        "label": row["day"],
                        "total": row["total"].as_i64().unwrap_or(0) as f64 / 100.0,
                        "orders": row["orders"],
                    })).collect::<Vec<_>>(),
                })
            }
            "top-selling-products" => {
                let rows = db::select(
                    "SELECT `oi`.`product_id`, `oi`.`name`, SUM(`oi`.`qty_ordered`) AS qty, \
                            CAST(COALESCE(SUM(`oi`.`base_total`), 0) * 100 AS INTEGER) AS total \
                     FROM `order_items` `oi` INNER JOIN `orders` `o` ON `o`.`id` = `oi`.`order_id` \
                     WHERE `o`.`created_at` BETWEEN ? AND ? \
                     GROUP BY `oi`.`product_id`, `oi`.`name` ORDER BY qty DESC LIMIT 5",
                    &[Value::String(start), Value::String(end)],
                )
                .await
                .unwrap_or_default();
                json!(rows)
            }
            "top-customers" => {
                let rows = db::select(
                    "SELECT `c`.`id`, `c`.`first_name`, `c`.`last_name`, `c`.`email`, \
                            COUNT(`o`.`id`) AS orders, \
                            CAST(COALESCE(SUM(`o`.`base_grand_total`), 0) * 100 AS INTEGER) AS total \
                     FROM `customers` `c` INNER JOIN `orders` `o` ON `o`.`customer_id` = `c`.`id` \
                     WHERE `o`.`created_at` BETWEEN ? AND ? \
                     GROUP BY `c`.`id` ORDER BY total DESC LIMIT 5",
                    &[Value::String(start), Value::String(end)],
                )
                .await
                .unwrap_or_default();
                json!(rows)
            }
            "stock-threshold-products" => {
                let rows = db::select(
                    "SELECT `p`.`id`, `p`.`sku`, COALESCE(`pf`.`name`, `p`.`sku`) AS name, \
                            COALESCE(SUM(`pi`.`qty`), 0) - COALESCE((SELECT SUM(`qty`) FROM `product_ordered_inventories` `poi` WHERE `poi`.`product_id` = `p`.`id`), 0) AS qty \
                     FROM `products` `p` \
                     LEFT JOIN `product_flat` `pf` ON `pf`.`product_id` = `p`.`id` \
                     LEFT JOIN `product_inventories` `pi` ON `pi`.`product_id` = `p`.`id` \
                     WHERE `p`.`parent_id` IS NULL \
                     GROUP BY `p`.`id`, `p`.`sku`, `pf`.`name` \
                     HAVING qty <= 5 ORDER BY qty ASC LIMIT 5",
                    &[],
                )
                .await
                .unwrap_or_default();
                json!(rows)
            }
            _ => json!({}),
        };

        resource(response, Value::Null)
    }
}
