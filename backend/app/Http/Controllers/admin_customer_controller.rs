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
use crate::support::response::{csv_response, message, resource, validation_error};

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

    /// `POST /api/admin/customers/groups` - list.
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

    /// `POST /api/admin/customers/groups/store` - `{code, name}`.
    pub async fn store_group(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let errors: Vec<(String, String)> = ["code", "name"]
            .iter()
            .filter(|field| body[**field].as_str().map(str::trim).unwrap_or_default().is_empty())
            .map(|field| (field.to_string(), format!("The {field} field is required.")))
            .collect();
        if !errors.is_empty() {
            return validation_error(errors);
        }
        let taken = db::select_one(
            "SELECT `id` FROM `customer_groups` WHERE `code` = ? LIMIT 1",
            &[Value::String(body["code"].as_str().unwrap_or_default().trim().to_string())],
        )
        .await
        .ok()
        .flatten();
        if taken.is_some() {
            return validation_error(vec![("code".into(), "The code has already been taken.".into())]);
        }
        match db::insert(
            "customer_groups",
            &json!({
                "code": body["code"].as_str().unwrap_or_default().trim(),
                "name": body["name"].as_str().unwrap_or_default().trim(),
                "is_user_defined": 1,
                "created_at": db::now_db(),
                "updated_at": db::now_db(),
            }),
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The customer group has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/admin/customers/groups/edit/{id}` - `{name?}`.
    pub async fn update_group(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let group = db::find_by_id("customer_groups", id).await.ok().flatten();
        let Some(group) = group else {
            return message(StatusCode::NOT_FOUND, "Customer group not found.");
        };
        if group["is_user_defined"].as_i64().unwrap_or(1) == 0 {
            return message(StatusCode::UNPROCESSABLE_ENTITY, "System groups cannot be modified.");
        }
        let mut row = json!({"updated_at": db::now_db()});
        if let Some(name) = body["name"].as_str() {
            row["name"] = json!(name.trim());
        }
        match db::update("customer_groups", id, &row).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The customer group has been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/customers/groups/{id}` - solo user-defined.
    pub async fn destroy_group(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let group = db::find_by_id("customer_groups", id).await.ok().flatten();
        let Some(group) = group else {
            return message(StatusCode::NOT_FOUND, "Customer group not found.");
        };
        if group["is_user_defined"].as_i64().unwrap_or(1) == 0 {
            return message(StatusCode::UNPROCESSABLE_ENTITY, "System groups cannot be deleted.");
        }
        match db::delete("customer_groups", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The customer group has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/seo/subscribers` - clientes suscritos al newsletter.
    pub async fn subscribers(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT `id`, `email`, `first_name`, `last_name`, `subscribed_to_news_letter`, `created_at` \
             FROM `customers` WHERE `subscribed_to_news_letter` = 1 ORDER BY `id` DESC",
            &[],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `PUT /api/admin/seo/subscribers/{id}` - `{subscribed: false}` da de baja.
    pub async fn update_subscriber(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        match db::update(
            "customers",
            id,
            &json!({
                "subscribed_to_news_letter": body["subscribed"].as_i64().unwrap_or(0),
                "updated_at": db::now_db(),
            }),
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The subscription has been updated."),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/customers/reviews` - grid de moderación de reseñas.
    pub async fn reviews(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let request = datagrid::parse_request(&body);
        let grid = datagrid::DataGrid {
            table: "product_reviews",
            alias: "pr",
            primary: "pr.id",
            columns: vec![
                datagrid::Column { index: "id", label: "ID", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("pr.id"), options: None },
                datagrid::Column { index: "name", label: "Author", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("pr.name"), options: None },
                datagrid::Column { index: "title", label: "Title", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("pr.title"), options: None },
                datagrid::Column { index: "rating", label: "Rating", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("pr.rating"), options: None },
                datagrid::Column { index: "status", label: "Status", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("pr.status"), options: Some(json!([
                    {"label": "Approved", "value": "approved"}, {"label": "Pending", "value": "pending"},
                    {"label": "Rejected", "value": "rejected"},
                ])) },
                datagrid::Column { index: "created_at", label: "Date", kind: "date", searchable: false, filterable: true, sortable: true, source: Some("pr.created_at"), options: None },
            ],
            joins: Vec::new(),
            where_clauses: Vec::new(),
            actions: Value::Null,
            mass_actions: Value::Null,
            formatters: Vec::new(),
        };
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/admin/customers/reviews/{id}/status` - `{status}`.
    pub async fn review_status(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let status = body["status"].as_str().unwrap_or_default();
        if !matches!(status, "approved" | "pending" | "rejected") {
            return validation_error(vec![("status".into(), "Invalid review status.".into())]);
        }
        match db::update(
            "product_reviews",
            id,
            &json!({"status": status, "updated_at": db::now_db()}),
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The review status has been updated."),
            Err(error) => error.into(),
        }
    }
}
