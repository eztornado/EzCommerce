//! Admin settings: users, roles, channels, locales, currencies, core_config,
//! theme sections. Mirrors `Webkul\Admin\Http\Controllers\Settings\*` (phase 1).

use larust_support::axum::extract::Path;
use larust_support::axum::http::{HeaderMap, StatusCode};
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Map, Value};

use crate::support::auth;
use crate::support::db;
use crate::support::response::{message, resource, validation_error};

pub struct AdminSettingController;

fn now() -> String {
    db::now_db()
}

impl AdminSettingController {
    /// `GET /api/admin/settings/users`.
    pub async fn users(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT `a`.`id`, `a`.`name`, `a`.`email`, `a`.`status`, `a`.`role_id`, `r`.`name` AS role \
             FROM `admins` `a` LEFT JOIN `roles` `r` ON `r`.`id` = `a`.`role_id` ORDER BY `a`.`id`",
            &[],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `POST /api/admin/settings/users` - `{name, email, password, role_id, status}`.
    pub async fn store_user(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let errors: Vec<(String, String)> = ["name", "email", "password"]
            .iter()
            .filter(|field| body[**field].as_str().map(str::trim).unwrap_or_default().is_empty())
            .map(|field| (field.to_string(), format!("The {field} field is required.")))
            .collect();
        if !errors.is_empty() {
            return validation_error(errors);
        }
        let Ok(hash) = crate::support::auth::hash_password(body["password"].as_str().unwrap_or_default())
        else {
            return message(
                larust_support::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to hash the password.",
            );
        };
        match db::insert(
            "admins",
            &json!({
                "name": body["name"].clone(),
                "email": body["email"].as_str().unwrap_or_default().trim().to_lowercase(),
                "password": hash,
                "status": body["status"].as_i64().unwrap_or(1),
                "role_id": body["role_id"].as_i64().unwrap_or(1),
                "created_at": now(),
                "updated_at": now(),
            })
            ,
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The user has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/settings/users/{id}` (self-deletion blocked).
    pub async fn destroy_user(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        if id == admin.id {
            return message(StatusCode::FORBIDDEN, "You cannot delete your own account.");
        }
        match db::delete("admins", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The user has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/settings/roles`.
    pub async fn roles(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select("SELECT * FROM `roles` ORDER BY `id`", &[]).await.unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `POST /api/admin/settings/roles` - `{name, description, permission_type, permissions}`.
    pub async fn store_role(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(name) = body["name"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("name".into(), "The name field is required.".into())]);
        };
        match db::insert(
            "roles",
            &json!({
                "name": name,
                "description": body["description"].clone(),
                "permission_type": body["permission_type"].as_str().unwrap_or("custom"),
                "permissions": body["permissions"].clone(),
                "created_at": now(),
                "updated_at": now(),
            })
            ,
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The role has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/settings/locales`.
    pub async fn locales(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select("SELECT * FROM `locales` ORDER BY `id`", &[]).await.unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `POST /api/admin/settings/locales` - `{code, name, direction}`.
    pub async fn store_locale(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(code) = body["code"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("code".into(), "The code field is required.".into())]);
        };
        match db::insert(
            "locales",
            &json!({
                "code": code,
                "name": body["name"].clone(),
                "direction": body["direction"].as_str().unwrap_or("ltr"),
                "created_at": now(),
                "updated_at": now(),
            })
            ,
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The locale has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/settings/currencies`.
    pub async fn currencies(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select("SELECT * FROM `currencies` ORDER BY `id`", &[]).await.unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `GET /api/admin/settings/channels`.
    pub async fn channels(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT `c`.*, `ct`.`name` FROM `channels` `c` \
             LEFT JOIN `channel_translations` `ct` ON `ct`.`channel_id` = `c`.`id` ORDER BY `c`.`id`",
            &[],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `PUT /api/admin/settings/channels/edit/{id}`.
    pub async fn update_channel(admin: auth::Admin, Path(id): Path<i64>, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let mut row = body.as_object().cloned().unwrap_or_default();
        row.insert("updated_at".into(), json!(now()));
        match db::update("channels", id, &row).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The channel has been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/configuration` - `core_config` rows (optionally by code prefix).
    pub async fn configuration(admin: auth::Admin, Path(code): Path<String>) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT * FROM `core_config` WHERE `code` LIKE ? ORDER BY `code`",
            &[Value::String(format!("{code}%"))],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `POST /api/admin/configuration` - upsert `{values: {code: value}}`.
    pub async fn save_configuration(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(values) = body["values"].as_object() else {
            return validation_error(vec![("values".into(), "The values field is required.".into())]);
        };
        for (code, value) in values {
            let value = match value {
                Value::String(value) => value.clone(),
                other => other.to_string(),
            };
            let existing = db::select_one(
                "SELECT `id` FROM `core_config` WHERE `code` = ? AND `channel_code` IS NULL AND `locale_code` IS NULL LIMIT 1",
                &[Value::String(code.clone())],
            )
            .await
            .ok()
            .flatten();
            match existing {
                Some(row) => {
                    let _ = db::update(
                        "core_config",
                        row["id"].as_i64().unwrap_or(0),
                        &json!({"value": value, "updated_at": now()}),
                    )
                    .await;
                }
                None => {
                    let _ = db::insert(
                        "core_config",
                        &json!({
                            "code": code,
                            "value": value,
                            "created_at": now(),
                            "updated_at": now(),
                        })
                        ,
                    )
                    .await;
                }
            }
        }
        resource(Value::Array(Vec::new()), "The configuration has been saved.")
    }

    /// `GET /api/admin/appearance/sections` - home sections of the channel.
    pub async fn theme_sections(admin: auth::Admin, Path(locale): Path<String>) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT `ts`.*, `tst`.`options` FROM `theme_sections` `ts` \
             LEFT JOIN `theme_section_translations` `tst` ON `tst`.`theme_section_id` = `ts`.`id` AND `tst`.`locale` = ? \
             ORDER BY `ts`.`sort_order`",
            &[Value::String(locale)],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }
}
