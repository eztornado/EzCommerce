//! Admin sales: orders grid + detail, invoice, ship, cancel, refund hooks.
//! Mirrors `Webkul\Admin\Http\Controllers\Sales\*` (phase 1: orders + invoices +
//! cancel; shipments/refunds fleshed out next).

use std::collections::HashMap;

use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::{HeaderMap, StatusCode};
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Map, Value};

use crate::support::auth;
use crate::support::datagrid;
use crate::support::db;
use crate::support::response::{csv_response, message, resource, validation_error};
use crate::support::sales;

pub struct AdminSalesController;

fn now() -> String {
    db::now_db()
}

pub fn orders_grid() -> datagrid::DataGrid {
    datagrid::DataGrid {
        table: "orders",
        alias: "o",
        primary: "o.id",
        columns: vec![
            datagrid::Column { index: "id", label: "ID", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("o.id"), options: None },
            datagrid::Column { index: "increment_id", label: "Order ID", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("o.increment_id"), options: None },
            datagrid::Column { index: "status", label: "Status", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("o.status"), options: Some(json!([
                {"label": "Pending", "value": "pending"}, {"label": "Processing", "value": "processing"},
                {"label": "Completed", "value": "completed"}, {"label": "Canceled", "value": "canceled"},
                {"label": "Closed", "value": "closed"},
            ])) },
            datagrid::Column { index: "base_grand_total", label: "Grand Total", kind: "decimal", searchable: false, filterable: true, sortable: true, source: Some("o.base_grand_total"), options: None },
            datagrid::Column { index: "customer_email", label: "Customer", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("o.customer_email"), options: None },
            datagrid::Column { index: "created_at", label: "Date", kind: "date", searchable: false, filterable: true, sortable: true, source: Some("o.created_at"), options: None },
        ],
        joins: Vec::new(),
        where_clauses: Vec::new(),
        actions: json!([
            {"title": "View", "method": "GET", "url": "/admin/sales/orders/view/{id}"},
        ]),
        mass_actions: Value::Null,
        formatters: Vec::new(),
    }
}

impl AdminSalesController {
    /// `POST /api/admin/sales/orders` - datagrid. `?format=csv` exporta.
    pub async fn orders_grid(
        admin: auth::Admin,
        Query(query): Query<HashMap<String, String>>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let grid = orders_grid();
        if query.get("format").map(String::as_str) == Some("csv") {
            let request = datagrid::parse_request(&body);
            return match grid.process_csv(&request).await {
                Ok(csv) => csv_response(&csv, "orders.csv"),
                Err(error) => error.into(),
            };
        }
        let request = datagrid::parse_request(&body);
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/sales/orders/view/{id}`.
    pub async fn order_view(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match sales::order_resource(id).await {
            Ok(order) => resource(order, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/sales/orders/{id}/cancel`.
    pub async fn order_cancel(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match sales::cancel_order(id).await {
            Ok(()) => resource(Value::Array(Vec::new()), "The order has been canceled."),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/sales/invoices/create/{order_id}`.
    pub async fn invoice_create(admin: auth::Admin, Path(order_id): Path<i64>) -> Response {
        let _ = admin;
        match sales::create_invoice(order_id).await {
            Ok(invoice) => resource(invoice, "The invoice has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/sales/invoices/view/{id}`.
    pub async fn invoice_view(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let invoice = db::find_by_id("invoices", id).await.ok().flatten();
        match invoice {
            Some(invoice) => resource(invoice, Value::Null),
            None => message(StatusCode::NOT_FOUND, "Invoice not found."),
        }
    }

    /// `POST /api/admin/sales/orders/comment/{order_id}` - `{comment, customer_notified}`.
    pub async fn order_comment(
        admin: auth::Admin,
        Path(order_id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let Some(comment) = body["comment"].as_str().map(str::trim).filter(|c| !c.is_empty()) else {
            return validation_error(vec![("comment".into(), "The comment field is required.".into())]);
        };
        match db::insert(
            "order_comments",
            &json!({
                "order_id": order_id,
                "comment": comment,
                "customer_notified": body["customer_notified"].as_bool().unwrap_or(false) as i64,
                "user_id": admin.id,
                "created_at": now(),
                "updated_at": now(),
            })
            ,
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The comment has been added successfully."),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/sales/shipments/create/{order_id}` -
    /// `{source_id, items: {order_item_id: qty}}` (o `{"all": qty}`).
    pub async fn shipment_create(
        admin: auth::Admin,
        Path(order_id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let source_id = body["source_id"].as_i64().unwrap_or(1);
        let requested = body["items"].as_object().cloned().unwrap_or_default();
        match sales::create_shipment(order_id, source_id, &requested).await {
            Ok(shipment) => resource(shipment, "The shipment has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/sales/shipments/view/{id}`.
    pub async fn shipment_view(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let shipment = db::find_by_id("shipments", id).await.ok().flatten();
        match shipment {
            Some(shipment) => resource(shipment, Value::Null),
            None => message(StatusCode::NOT_FOUND, "Shipment not found."),
        }
    }

    /// `POST /api/admin/sales/refunds/create/{order_id}` -
    /// `{items: {order_item_id: qty}, adjustment_refund, adjustment_fee}`.
    pub async fn refund_create(
        admin: auth::Admin,
        Path(order_id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let requested = body["items"].as_object().cloned().unwrap_or_default();
        let adjustment_refund = body["adjustment_refund"].as_f64().unwrap_or(0.0);
        let adjustment_fee = body["adjustment_fee"].as_f64().unwrap_or(0.0);
        match sales::create_refund(order_id, &requested, adjustment_refund, adjustment_fee).await {
            Ok(refund) => resource(refund, "The refund has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/sales/refunds/view/{id}`.
    pub async fn refund_view(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let refund = db::find_by_id("refunds", id).await.ok().flatten();
        match refund {
            Some(refund) => resource(refund, Value::Null),
            None => message(StatusCode::NOT_FOUND, "Refund not found."),
        }
    }

    /// `GET /api/admin/datagrid/look-up?table=&label=&value=&q=` -
    /// autocompletado genérico para filtros relacionales del datagrid
    /// (devuelve pares {label, value} de cualquier tabla).
    pub async fn datagrid_look_up(Query(query): Query<HashMap<String, String>>) -> Response {
        let table = query.get("table").cloned().unwrap_or_default();
        let label_column = query.get("label").cloned().unwrap_or_else(|| "name".into());
        let value_column = query.get("value").cloned().unwrap_or_else(|| "id".into());
        let term = query.get("q").cloned().unwrap_or_default();

        // Sólo identificadores seguros.
        let valid = |name: &str| {
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        };
        if !valid(&table) || !valid(&label_column) || !valid(&value_column) {
            return message(StatusCode::BAD_REQUEST, "Invalid look-up parameters.");
        }

        let sql = format!(
            "SELECT `{}` AS `label`, `{}` AS `value` FROM `{}`              WHERE CAST(`{}` AS CHAR) LIKE ? ORDER BY `{}` LIMIT 20",
            label_column, value_column, table, label_column, label_column
        );
        let rows = db::select(&sql, &[Value::String(format!("%{term}%"))])
            .await
            .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `GET /api/admin/sales/orders/search?q=` - increment/customer search.
    pub async fn order_search(admin: auth::Admin, Query(query): Query<HashMap<String, String>>) -> Response {
        let _ = admin;
        let term = query.get("query").map(String::as_str).unwrap_or_default();
        let rows = db::select(
            "SELECT `id`, `increment_id`, `customer_email`, `base_grand_total`, `status` FROM `orders` \
             WHERE `increment_id` LIKE ? OR `customer_email` LIKE ? ORDER BY `id` DESC LIMIT 10",
            &[
                Value::String(format!("%{term}%")),
                Value::String(format!("%{term}%")),
            ],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }
}
