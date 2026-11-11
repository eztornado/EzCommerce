//! Appearance: CRUD de theme_sections (home por secciones) con
//! traducciones, reorder y activar/desactivar. Mirrors
//! `Webkul\Admin\Http\Controllers\Appearance\SectionController`.

use std::collections::HashMap;

use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::StatusCode;
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::db;
use crate::support::response::{message, resource, validation_error};

pub struct AdminAppearanceController;

impl AdminAppearanceController {
    /// `GET /api/admin/appearance/sections/list/{locale}` - todas las
    /// secciones del canal con sus opciones traducidas.
    pub async fn sections(admin: auth::Admin, Path(locale): Path<String>) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT `ts`.`id`, `ts`.`type`, `ts`.`name`, `ts`.`sort_order`, `ts`.`status`, `tst`.`options` \
             FROM `theme_sections` `ts` \
             LEFT JOIN `theme_section_translations` `tst` ON `tst`.`theme_section_id` = `ts`.`id` AND `tst`.`locale` = ? \
             ORDER BY `ts`.`sort_order`",
            &[Value::String(locale)],
        )
        .await
        .unwrap_or_default();
        let sections: Vec<Value> = rows
            .into_iter()
            .map(|mut row| {
                if let Some(options) = row.get("options").and_then(Value::as_str) {
                    let parsed = larust_support::serde_json::from_str::<Value>(options)
                        .unwrap_or(Value::Null);
                    row.insert("options".into(), parsed);
                }
                Value::Object(row)
            })
            .collect();
        resource(json!(sections), Value::Null)
    }

    /// `POST /api/admin/appearance/sections/store` -
    /// `{type, name, sort, status, locale, options: {...}}`.
    pub async fn store(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(kind) = body["type"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("type".into(), "The type field is required.".into())]);
        };
        let allowed = [
            "image_carousel",
            "product_carousel",
            "category_carousel",
            "static_content",
            "services_content",
            "footer_links",
        ];
        if !allowed.contains(&kind) {
            return validation_error(vec![("type".into(), "Unsupported section type.".into())]);
        }
        let next_sort: i64 = db::scalar_i64(
            "SELECT COALESCE(MAX(`sort_order`), 0) + 1 FROM `theme_sections`",
            &[],
        )
        .await
        .unwrap_or(1);

        let section_id = match db::insert(
            "theme_sections",
            &json!({
                "type": kind,
                "name": body["name"].clone(),
                "sort_order": body["sort"].as_i64().unwrap_or(next_sort),
                "status": body["status"].as_i64().unwrap_or(1),
                "channel_id": 1,
                "theme_code": "default",
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
        save_translation(section_id, &body).await;
        resource(json!({"id": section_id}), "The section has been created successfully.")
    }

    /// `PUT /api/admin/appearance/sections/edit/{id}`.
    pub async fn update(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let exists = db::find_by_id("theme_sections", id).await.ok().flatten();
        if exists.is_none() {
            return message(StatusCode::NOT_FOUND, "Section not found.");
        }
        let mut row = body
            .as_object()
            .cloned()
            .unwrap_or_default();
        row.remove("options");
        row.remove("id");
        row.remove("created_at");
        row.remove("updated_at");
        // Normaliza `sort` -> `sort_order` (el editor envía sort).
        if let Some(sort) = row.remove("sort") {
            row.insert("sort_order".into(), sort);
        }
        if let Err(error) = db::update("theme_sections", id, &row).await {
            return error.into();
        }
        save_translation(id, &body).await;
        resource(json!({"id": id}), "The section has been updated successfully.")
    }

    /// `DELETE /api/admin/appearance/sections/{id}`.
    pub async fn destroy(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match db::delete("theme_sections", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The section has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/appearance/sections/reorder` - `{order: [ids en orden]}`.
    pub async fn reorder(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        for (position, id) in body["order"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            let Some(id) = id.as_i64() else { continue };
            let _ = db::execute(
                "UPDATE `theme_sections` SET `sort_order` = ?, `updated_at` = ? WHERE `id` = ?",
                &[
                    Value::from(position as i64),
                    Value::String(db::now_db()),
                    Value::from(id),
                ],
            )
            .await;
        }
        resource(Value::Array(Vec::new()), "The sections have been reordered.")
    }

    /// `POST /api/admin/appearance/sections/status/{id}` - `{status: 0|1}`.
    pub async fn toggle_status(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let _ = db::execute(
            "UPDATE `theme_sections` SET `status` = ?, `updated_at` = ? WHERE `id` = ?",
            &[
                Value::from(body["status"].as_i64().unwrap_or(1)),
                Value::String(db::now_db()),
                Value::from(id),
            ],
        )
        .await;
        resource(Value::Array(Vec::new()), "The section status has been updated.")
    }
}

async fn save_translation(section_id: i64, body: &Value) {
    let locale = body["locale"].as_str().unwrap_or("en").to_string();
    let existing = db::select_one(
        "SELECT `id` FROM `theme_section_translations` WHERE `theme_section_id` = ? AND `locale` = ? LIMIT 1",
        &[Value::from(section_id), Value::String(locale.clone())],
    )
    .await
    .ok()
    .flatten();
    let options = match body["options"].as_object() {
        Some(object) => Value::Object(object.clone()).to_string(),
        None => "{}".to_string(),
    };
    let payload = json!({
        "theme_section_id": section_id,
        "locale": locale,
        "options": options,
        "updated_at": db::now_db(),
    });
    match existing {
        Some(row) => {
            let _ = db::update(
                "theme_section_translations",
                row["id"].as_i64().unwrap_or(0),
                &payload,
            )
            .await;
        }
        None => {
            let mut payload = db::to_map(payload);
            payload.insert("created_at".into(), json!(db::now_db()));
            let _ = db::insert("theme_section_translations", &payload).await;
        }
    }
}
