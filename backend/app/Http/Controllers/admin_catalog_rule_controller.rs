//! Admin catalog rules: datagrid + CRUD con pivotes y reindexado de precios.

use larust_support::axum::extract::Path;
use larust_support::axum::http::StatusCode;
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::catalog_rules;
use crate::support::datagrid;
use crate::support::db;
use crate::support::response::{message, resource, validation_error};

pub struct AdminCatalogRuleController;

fn catalog_rules_grid() -> datagrid::DataGrid {
    datagrid::DataGrid {
        table: "catalog_rules",
        alias: "r",
        primary: "r.id",
        columns: vec![
            datagrid::Column { index: "id", label: "ID", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("r.id"), options: None },
            datagrid::Column { index: "name", label: "Name", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("r.name"), options: None },
            datagrid::Column { index: "action_type", label: "Action", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("r.action_type"), options: Some(json!([
                {"label": "To percent", "value": "to_percent"}, {"label": "By percent", "value": "by_percent"},
                {"label": "To fixed", "value": "to_fixed"}, {"label": "By fixed", "value": "by_fixed"},
            ])) },
            datagrid::Column { index: "status", label: "Status", kind: "boolean", searchable: false, filterable: true, sortable: true, source: Some("r.status"), options: Some(json!([
                {"label": "Active", "value": 1}, {"label": "Inactive", "value": 0},
            ])) },
            datagrid::Column { index: "starts_from", label: "From", kind: "date", searchable: false, filterable: false, sortable: true, source: Some("r.starts_from"), options: None },
            datagrid::Column { index: "ends_till", label: "To", kind: "date", searchable: false, filterable: false, sortable: true, source: Some("r.ends_till"), options: None },
        ],
        joins: Vec::new(),
        where_clauses: Vec::new(),
        actions: json!([
            {"title": "Delete", "method": "DELETE", "url": "/api/admin/marketing/catalog-rules/{id}"},
        ]),
        mass_actions: Value::Null,
        formatters: Vec::new(),
    }
}

impl AdminCatalogRuleController {
    /// `POST /api/admin/marketing/catalog-rules` - datagrid.
    pub async fn grid(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let request = datagrid::parse_request(&body);
        match catalog_rules_grid().process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/marketing/catalog-rules/store`.
    pub async fn store(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(name) = body["name"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("name".into(), "The name field is required.".into())]);
        };
        let rule_id = match db::insert(
            "catalog_rules",
            &json!({
                "name": name,
                "description": body["description"].clone(),
                "starts_from": body["starts_from"].clone(),
                "ends_till": body["ends_till"].clone(),
                "status": body["status"].as_i64().unwrap_or(1),
                "condition_type": body["condition_type"].as_i64().unwrap_or(1),
                "conditions": body["conditions"].clone(),
                "action_type": body["action_type"].as_str().unwrap_or("by_percent"),
                "discount_amount": body["discount_amount"].as_f64().unwrap_or(0.0),
                "sort_order": body["sort"].as_i64().unwrap_or(0),
                "created_at": db::now_db(),
                "updated_at": db::now_db(),
            })
            ,
        )
        .await
        {
            Ok(id) => id,
            Err(error) => return error.into(),
        };
        catalog_rules::save_pivots(rule_id, &body).await;
        let _ = catalog_rules::reindex_all().await;
        resource(json!({"id": rule_id}), "The catalog rule has been created successfully.")
    }

    /// `PUT /api/admin/marketing/catalog-rules/edit/{id}`.
    pub async fn update(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let exists = db::find_by_id("catalog_rules", id).await.ok().flatten();
        if exists.is_none() {
            return message(StatusCode::NOT_FOUND, "Catalog rule not found.");
        }
        let mut row = body.as_object().cloned().unwrap_or_default();
        row.remove("created_at");
        row.remove("updated_at");
        if let Err(error) = db::update("catalog_rules", id, &row).await {
            return error.into();
        }
        catalog_rules::save_pivots(id, &body).await;
        let _ = catalog_rules::reindex_all().await;
        resource(json!({"id": id}), "The catalog rule has been updated successfully.")
    }

    /// `DELETE /api/admin/marketing/catalog-rules/{id}` + reindexado.
    pub async fn destroy(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match db::delete("catalog_rules", id).await {
            Ok(_) => {
                let _ = catalog_rules::reindex_all().await;
                resource(Value::Array(Vec::new()), "The catalog rule has been deleted successfully.")
            }
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/marketing/catalog-rules/reindex` - reindex completo.
    pub async fn reindex(admin: auth::Admin) -> Response {
        let _ = admin;
        match catalog_rules::reindex_all().await {
            Ok(()) => resource(Value::Array(Vec::new()), "The price indices have been recalculated."),
            Err(error) => error.into(),
        }
    }
}
