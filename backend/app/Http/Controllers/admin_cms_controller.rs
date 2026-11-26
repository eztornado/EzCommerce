//! CMS: shop page by url_key + admin CRUD (translations + channels).

use larust_support::axum::extract::Path;
use larust_support::axum::http::StatusCode;
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::datagrid;
use crate::support::db;
use crate::support::response::{message, resource};

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
        match crate::support::admin_crud::create_cms_page(&body).await {
            Ok(page_id) => resource(
                json!({"id": page_id}),
                "The page has been created successfully.",
            ),
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/admin/cms/pages/edit/{id}`.
    pub async fn update(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        match crate::support::admin_crud::update_cms_page(id, &body).await {
            Ok(_) => resource(
                json!({"id": id}),
                "The page has been updated successfully.",
            ),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/cms/pages/{id}`.
    pub async fn destroy(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match crate::support::admin_crud::delete_cms_page(id).await {
            Ok(_) => resource(
                Value::Array(Vec::new()),
                "The page has been deleted successfully.",
            ),
            Err(error) => error.into(),
        }
    }
}
