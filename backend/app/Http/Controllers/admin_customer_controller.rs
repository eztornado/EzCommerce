//! Admin customers: grid, 360º view, groups, reviews. Mirrors
//! `Webkul\Admin\Http\Controllers\Customer\*`.

use std::collections::HashMap;

use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::{HeaderMap, StatusCode};
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::datagrid;
use crate::support::db;
use crate::support::response::{csv_response, message, resource};

pub struct AdminCustomerController;

pub fn customers_grid() -> datagrid::DataGrid {
    datagrid::DataGrid {
            table: "customers",
            alias: "c",
            primary: "c.id",
            columns: vec![
                datagrid::Column { index: "id", label: "ID", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("c.id"), options: None },
                datagrid::Column { index: "first_name", label: "First Name", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("c.first_name"), options: None },
                datagrid::Column { index: "last_name", label: "Last Name", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("c.last_name"), options: None },
                datagrid::Column { index: "email", label: "Email", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("c.email"), options: None },
                datagrid::Column { index: "group", label: "Group", kind: "string", searchable: false, filterable: true, sortable: false, source: Some("cg.name"), options: None },
                datagrid::Column { index: "status", label: "Active", kind: "boolean", searchable: false, filterable: true, sortable: true, source: Some("c.status"), options: Some(json!([
                    {"label": "Active", "value": 1}, {"label": "Inactive", "value": 0},
                ])) },
            ],
            joins: vec!["LEFT JOIN `customer_groups` `cg` ON `cg`.`id` = `c`.`customer_group_id`".into()],
            where_clauses: Vec::new(),
            actions: json!([
                {"title": "View", "method": "GET", "url": "/admin/customers/view/{id}"},
                {"title": "Delete", "method": "DELETE", "url": "/api/admin/customers/{id}"},
            ]),
            mass_actions: Value::Null,
            formatters: Vec::new(),
        }
}

impl AdminCustomerController {
    /// `POST /api/admin/customers` - datagrid. `?format=csv` exporta.
    pub async fn grid(
        admin: auth::Admin,
        Query(query): Query<HashMap<String, String>>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let grid = customers_grid();
        if query.get("format").map(String::as_str) == Some("csv") {
            let request = datagrid::parse_request(&body);
            return match grid.process_csv(&request).await {
                Ok(csv) => csv_response(&csv, "customers.csv"),
                Err(error) => error.into(),
            };
        }
        let request = datagrid::parse_request(&body);
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }


    /// `GET /api/admin/customers/view/{id}` - 360º view.
    pub async fn view(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let mut customer = match db::find_by_id("customers", id).await {
            Ok(Some(customer)) => customer,
            _ => return message(StatusCode::NOT_FOUND, "Customer not found."),
        };
        customer["password"] = Value::Null;
        let orders = db::select(
            "SELECT `id`, `increment_id`, `status`, `base_grand_total`, `created_at` FROM `orders` WHERE `customer_id` = ? ORDER BY `id` DESC LIMIT 10",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        let addresses = db::select(
            "SELECT * FROM `addresses` WHERE `customer_id` = ? AND `address_type` = 'customer_address' ORDER BY `id` DESC",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        let reviews = db::select(
            "SELECT * FROM `product_reviews` WHERE `customer_id` = ? ORDER BY `id` DESC LIMIT 10",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        let wishlist_count = db::scalar_i64(
            "SELECT COUNT(*) FROM `wishlist_items` WHERE `customer_id` = ?",
            &[Value::from(id)],
        )
        .await
        .unwrap_or(0);

        resource(
            json!({
                "customer": customer,
                "orders": orders,
                "addresses": addresses,
                "reviews": reviews,
                "wishlist_count": wishlist_count,
            }),
            Value::Null,
        )
    }

    /// `PUT /api/admin/customers/edit/{id}`.
    pub async fn update(admin: auth::Admin, Path(id): Path<i64>, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let mut row = body.as_object().cloned().unwrap_or_default();
        row.remove("password");
        row.insert("updated_at".into(), json!(db::now_db()));
        match db::update("customers", id, &row).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The customer has been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/customers/{id}`.
    pub async fn destroy(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match db::delete("customers", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The customer has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/customers/groups` - datagrid + list.
    pub async fn groups(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT * FROM `customer_groups` ORDER BY `id`",
            &[],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }
}
