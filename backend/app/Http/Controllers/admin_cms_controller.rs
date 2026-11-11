//! CMS: shop page by url_key + admin CRUD (translations + channels).

use std::collections::HashMap;

use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::StatusCode;
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::datagrid;
use crate::support::db;
use crate::support::response::{message, resource, validation_error};

pub struct AdminCmsController;

impl AdminCmsController {
    /// `GET /api/cms/page/{urlKey}` - página pública traducida.
    pub async fn show_page(Path(url_key): Path<String>) -> Response {
        let page = db::select_one(
            "SELECT `cp`.`id`, `ct`.`page_title`, `ct`.`url_key`, `ct`.`html_content`, `ct`.`meta_title`, `ct`.`meta_description` \
             FROM `cms_page_translations` `ct` INNER JOIN `cms_pages` `cp` ON `cp`.`id` = `ct`.`cms_page_id` \
             WHERE `ct`.`url_key` = ? ORDER BY `ct`.`id` LIMIT 1",
            &[Value::String(url_key)],
        )
        .await
        .ok()
        .flatten();
        match page {
            Some(page) => resource(page, Value::Null),
            None => message(StatusCode::NOT_FOUND, "Page not found."),
        }
    }

    /// `POST /api/admin/cms/pages` - datagrid.
    pub async fn pages_grid(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let grid = datagrid::DataGrid {
            table: "cms_pages",
            alias: "cp",
            primary: "cp.id",
            columns: vec![
                datagrid::Column { index: "id", label: "ID", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("cp.id"), options: None },
                datagrid::Column { index: "page_title", label: "Title", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("ct.page_title"), options: None },
                datagrid::Column { index: "url_key", label: "URL Key", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("ct.url_key"), options: None },
            ],
            joins: vec![
                "LEFT JOIN `cms_page_translations` `ct` ON `ct`.`cms_page_id` = `cp`.`id`".into(),
            ],
            where_clauses: Vec::new(),
            actions: json!([
                {"title": "Delete", "method": "DELETE", "url": "/api/admin/cms/pages/{id}"},
            ]),
            mass_actions: Value::Null,
            formatters: Vec::new(),
        };
        let request = datagrid::parse_request(&body);
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/cms/pages/view/{id}`.
    pub async fn view(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let page = db::find_by_id("cms_pages", id).await.ok().flatten();
        let translation = db::select_one(
            "SELECT * FROM `cms_page_translations` WHERE `cms_page_id` = ? ORDER BY `id` LIMIT 1",
            &[Value::from(id)],
        )
        .await
        .ok()
        .flatten();
        resource(
            json!({"page": page.map(Value::Object).unwrap_or(Value::Null), "translation": translation.map(Value::Object).unwrap_or(Value::Null)}),
            Value::Null,
        )
    }

    /// `POST /api/admin/cms/pages/store` - `{page_title, url_key, html_content, meta_*}`.
    pub async fn store(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let errors = validate(&body);
        if !errors.is_empty() {
            return validation_error(errors);
        }
        let page_id = match db::insert("cms_pages", &json!({"created_at": db::now_db(), "updated_at": db::now_db()})).await {
            Ok(id) => id,
            Err(error) => return error.into(),
        };
        save_translation(page_id, &body).await;
        let _ = db::execute(
            "INSERT OR IGNORE INTO `cms_page_channels` (`cms_page_id`, `channel_id`) VALUES (?, 1)",
            &[Value::from(page_id)],
        )
        .await;
        resource(json!({"id": page_id}), "The page has been created successfully.")
    }

    /// `PUT /api/admin/cms/pages/edit/{id}`.
    pub async fn update(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let exists = db::find_by_id("cms_pages", id).await.ok().flatten();
        if exists.is_none() {
            return message(StatusCode::NOT_FOUND, "Page not found.");
        }
        let errors = validate(&body);
        if !errors.is_empty() {
            return validation_error(errors);
        }
        save_translation(id, &body).await;
        resource(json!({"id": id}), "The page has been updated successfully.")
    }

    /// `DELETE /api/admin/cms/pages/{id}`.
    pub async fn destroy(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match db::delete("cms_pages", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The page has been deleted successfully."),
            Err(error) => error.into(),
        }
    }
}

fn validate(body: &Value) -> Vec<(String, String)> {
    let mut errors = Vec::new();
    if body["page_title"].as_str().map(str::trim).unwrap_or_default().is_empty() {
        errors.push(("page_title".into(), "The title field is required.".into()));
    }
    if body["url_key"].as_str().map(str::trim).unwrap_or_default().is_empty() {
        errors.push(("url_key".into(), "The URL key field is required.".into()));
    }
    errors
}

async fn save_translation(page_id: i64, body: &Value) {
    let translation = json!({
        "cms_page_id": page_id,
        "locale": body["locale"].as_str().unwrap_or("en"),
        "page_title": body["page_title"].clone(),
        "url_key": crate::support::slug::slugify(body["url_key"].as_str().unwrap_or_default()),
        "html_content": body["html_content"].clone(),
        "meta_title": body["meta_title"].clone(),
        "meta_description": body["meta_description"].clone(),
        "updated_at": db::now_db(),
    });
    let locale = body["locale"].as_str().unwrap_or("en").to_string();
    let existing = db::select_one(
        "SELECT `id` FROM `cms_page_translations` WHERE `cms_page_id` = ? AND `locale` = ? LIMIT 1",
        &[Value::from(page_id), Value::String(locale)],
    )
    .await
    .ok()
    .flatten();
    match existing {
        Some(row) => {
            let _ = db::update(
                "cms_page_translations",
                row["id"].as_i64().unwrap_or(0),
                &translation,
            )
            .await;
        }
        None => {
            let mut translation = db::to_map(translation);
            translation.insert("created_at".into(), json!(db::now_db()));
            let _ = db::insert("cms_page_translations", &translation).await;
        }
    }
}
