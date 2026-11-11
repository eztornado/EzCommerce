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
        if locale_code_taken(code, 0).await {
            return validation_error(vec![("code".into(), "The code has already been taken.".into())]);
        }
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

    /// `POST /api/admin/settings/currencies` - `{code, name, symbol, decimal}`.
    pub async fn store_currency(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(code) = body["code"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("code".into(), "The code field is required.".into())]);
        };
        if currency_code_taken(code, 0).await {
            return validation_error(vec![("code".into(), "The code has already been taken.".into())]);
        }
        match db::insert(
            "currencies",
            &json!({
                "code": code,
                "name": body["name"].clone(),
                "symbol": body["symbol"].clone(),
                "decimal": body["decimal"].as_i64().unwrap_or(2),
                "created_at": now(),
                "updated_at": now(),
            }),
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The currency has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/admin/settings/currencies/edit/{id}`.
    pub async fn update_currency(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let mut row = Map::new();
        if let Some(code) = body["code"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
            if currency_code_taken(code, id).await {
                return validation_error(vec![("code".into(), "The code has already been taken.".into())]);
            }
            row.insert("code".into(), json!(code));
        }
        for key in ["name", "symbol"] {
            if !body[key].is_null() {
                row.insert(key.into(), body[key].clone());
            }
        }
        if let Some(decimal) = body["decimal"].as_i64() {
            row.insert("decimal".into(), json!(decimal));
        }
        row.insert("updated_at".into(), json!(now()));
        match db::update("currencies", id, &row).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The currency has been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/settings/currencies/{id}` - bloqueada mientras un
    /// canal la use como base o la tenga asociada.
    pub async fn destroy_currency(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let in_use = db::scalar_i64(
            "SELECT COUNT(*) FROM `channels` WHERE `base_currency_id` = ?",
            &[Value::from(id)],
        )
        .await
        .unwrap_or(0)
            + db::scalar_i64(
                "SELECT COUNT(*) FROM `channel_currencies` WHERE `currency_id` = ?",
                &[Value::from(id)],
            )
            .await
            .unwrap_or(0);
        if in_use > 0 {
            return message(
                StatusCode::UNPROCESSABLE_ENTITY,
                "The currency is used by a channel and cannot be deleted.",
            );
        }
        match db::delete("currencies", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The currency has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/admin/settings/locales/edit/{id}` - `{name?, direction?, code?}`.
    pub async fn update_locale(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let mut row = Map::new();
        if let Some(code) = body["code"].as_str().map(str::trim).filter(|s| !s.is_empty()) {
            if locale_code_taken(code, id).await {
                return validation_error(vec![("code".into(), "The code has already been taken.".into())]);
            }
            row.insert("code".into(), json!(code));
        }
        if !body["name"].is_null() {
            row.insert("name".into(), body["name"].clone());
        }
        if let Some(direction) = body["direction"].as_str() {
            row.insert("direction".into(), json!(direction));
        }
        row.insert("updated_at".into(), json!(now()));
        match db::update("locales", id, &row).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The locale has been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/settings/locales/{id}` - bloqueado mientras un canal
    /// lo use por defecto o lo tenga asociado.
    pub async fn destroy_locale(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let in_use = db::scalar_i64(
            "SELECT COUNT(*) FROM `channels` WHERE `default_locale_id` = ?",
            &[Value::from(id)],
        )
        .await
        .unwrap_or(0)
            + db::scalar_i64(
                "SELECT COUNT(*) FROM `channel_locales` WHERE `locale_id` = ?",
                &[Value::from(id)],
            )
            .await
            .unwrap_or(0);
        if in_use > 0 {
            return message(
                StatusCode::UNPROCESSABLE_ENTITY,
                "The locale is used by a channel and cannot be deleted.",
            );
        }
        match db::delete("locales", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The locale has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/settings/channels` - con pivotes de locales y monedas.
    pub async fn channels(admin: auth::Admin) -> Response {
        let _ = admin;
        let mut rows = db::select(
            // Una sola translation por canal: la primera creada (los seeds
            // antiguos duplicaban filas del mismo locale).
            "SELECT `c`.*, `ct`.`name` FROM `channels` `c` \
             LEFT JOIN `channel_translations` `ct` ON `ct`.`id` = \
               (SELECT MIN(`t`.`id`) FROM `channel_translations` `t` WHERE `t`.`channel_id` = `c`.`id`) \
             ORDER BY `c`.`id`",
            &[],
        )
        .await
        .unwrap_or_default();
        for row in rows.iter_mut() {
            let channel_id = row["id"].as_i64().unwrap_or(0);
            let locale_ids: Vec<Value> = db::select(
                "SELECT `locale_id` FROM `channel_locales` WHERE `channel_id` = ?",
                &[Value::from(channel_id)],
            )
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|r| r["locale_id"].clone())
            .collect();
            let currency_ids: Vec<Value> = db::select(
                "SELECT `currency_id` FROM `channel_currencies` WHERE `channel_id` = ?",
                &[Value::from(channel_id)],
            )
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|r| r["currency_id"].clone())
            .collect();
            row.insert("locales".into(), Value::Array(locale_ids));
            row.insert("currencies".into(), Value::Array(currency_ids));
        }
        resource(json!(rows), Value::Null)
    }

    /// `PUT /api/admin/settings/channels/edit/{id}` - campos directos + name
    /// (translation) + pivotes `{locales: [id], currencies: [id]}`.
    pub async fn update_channel(admin: auth::Admin, Path(id): Path<i64>, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let mut row = Map::new();
        for key in ["code", "timezone", "theme", "hostname", "logo", "favicon"] {
            if !body[key].is_null() {
                row.insert(key.into(), body[key].clone());
            }
        }
        for key in ["default_locale_id", "base_currency_id", "root_category_id", "is_maintenance_on"] {
            if !body[key].is_null() {
                row.insert(key.into(), body[key].clone());
            }
        }
        row.insert("updated_at".into(), json!(now()));
        match db::update("channels", id, &row).await {
            Ok(_) => {}
            Err(error) => return error.into(),
        }

        if let Some(name) = body["name"].as_str() {
            upsert_channel_name(id, name).await;
        }
        if let Some(locales) = body["locales"].as_array() {
            replace_pivots("channel_locales", "channel_id", "locale_id", id, locales).await;
            // El locale por defecto debe estar asociado al canal.
            if let Some(default_locale_id) = body["default_locale_id"].as_i64() {
                let _ = db::execute(
                    "INSERT OR IGNORE INTO `channel_locales` (`channel_id`, `locale_id`) VALUES (?, ?)",
                    &[Value::from(id), Value::from(default_locale_id)],
                )
                .await;
            }
        }
        if let Some(currencies) = body["currencies"].as_array() {
            replace_pivots("channel_currencies", "channel_id", "currency_id", id, currencies).await;
            if let Some(base_currency_id) = body["base_currency_id"].as_i64() {
                let _ = db::execute(
                    "INSERT OR IGNORE INTO `channel_currencies` (`channel_id`, `currency_id`) VALUES (?, ?)",
                    &[Value::from(id), Value::from(base_currency_id)],
                )
                .await;
            }
        }
        resource(Value::Array(Vec::new()), "The channel has been updated successfully.")
    }

    /// `POST /api/admin/settings/channels` - `{code, name, locales: [id],
    /// currencies: [id], default_locale_id, base_currency_id, root_category_id,
    /// timezone, hostname, theme}`.
    pub async fn store_channel(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(code) = body["code"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("code".into(), "The code field is required.".into())]);
        };
        let taken = db::select_one(
            "SELECT `id` FROM `channels` WHERE `code` = ? LIMIT 1",
            &[Value::String(code.to_string())],
        )
        .await
        .ok()
        .flatten();
        if taken.is_some() {
            return validation_error(vec![("code".into(), "The code has already been taken.".into())]);
        }
        let channel_id = match db::insert(
            "channels",
            &json!({
                "code": code,
                "timezone": body["timezone"].clone(),
                "theme": body["theme"].as_str().unwrap_or("default"),
                "hostname": body["hostname"].clone(),
                "default_locale_id": body["default_locale_id"].clone(),
                "base_currency_id": body["base_currency_id"].clone(),
                "root_category_id": body["root_category_id"].clone(),
                "created_at": now(),
                "updated_at": now(),
            }),
        )
        .await
        {
            Ok(channel_id) => channel_id,
            Err(error) => return error.into(),
        };
        if let Some(name) = body["name"].as_str() {
            upsert_channel_name(channel_id, name).await;
        }
        let locales = body["locales"].as_array().cloned().unwrap_or_default();
        let currencies = body["currencies"].as_array().cloned().unwrap_or_default();
        let mut locale_values: Vec<Value> = locales;
        if !locale_values.is_empty() && body["default_locale_id"].as_i64().is_none() {
            if let Some(first) = locale_values.first().and_then(|v| v.as_i64()) {
                let _ = db::execute(
                    "UPDATE `channels` SET `default_locale_id` = ? WHERE `id` = ?",
                    &[Value::from(first), Value::from(channel_id)],
                )
                .await;
            }
        }
        replace_pivots("channel_locales", "channel_id", "locale_id", channel_id, &locale_values).await;
        replace_pivots("channel_currencies", "channel_id", "currency_id", channel_id, &currencies).await;
        // Asegurar base currency asociada si solo vino base_currency_id.
        if currencies.is_empty() {
            if let Some(base) = body["base_currency_id"].as_i64() {
                let _ = db::execute(
                    "INSERT OR IGNORE INTO `channel_currencies` (`channel_id`, `currency_id`) VALUES (?, ?)",
                    &[Value::from(channel_id), Value::from(base)],
                )
                .await;
            }
        }
        resource(json!({"id": channel_id}), "The channel has been created successfully.")
    }

    /// `DELETE /api/admin/settings/channels/{id}` - el canal por defecto (id 1)
    /// no se puede borrar.
    pub async fn destroy_channel(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        if id == 1 {
            return message(
                StatusCode::UNPROCESSABLE_ENTITY,
                "The default channel cannot be deleted.",
            );
        }
        match db::delete("channels", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The channel has been deleted successfully."),
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

/// `true` si el código de moneda ya existe (excluyendo `except_id`).
async fn currency_code_taken(code: &str, except_id: i64) -> bool {
    db::scalar_i64(
        "SELECT COUNT(*) FROM `currencies` WHERE `code` = ? AND `id` != ?",
        &[Value::String(code.to_string()), Value::from(except_id)],
    )
    .await
    .unwrap_or(0)
    > 0
}

/// `true` si el código de locale ya existe (excluyendo `except_id`).
async fn locale_code_taken(code: &str, except_id: i64) -> bool {
    db::scalar_i64(
        "SELECT COUNT(*) FROM `locales` WHERE `code` = ? AND `id` != ?",
        &[Value::String(code.to_string()), Value::from(except_id)],
    )
    .await
    .unwrap_or(0)
    > 0
}

/// Upsert de la translation del canal en el locale por defecto.
async fn upsert_channel_name(channel_id: i64, name: &str) {
    let existing = db::select_one(
        "SELECT `id` FROM `channel_translations` WHERE `channel_id` = ? AND `locale` = 'en' LIMIT 1",
        &[Value::from(channel_id)],
    )
    .await
    .ok()
    .flatten();
    match existing {
        Some(row) => {
            let _ = db::update(
                "channel_translations",
                row["id"].as_i64().unwrap_or(0),
                &json!({"name": name, "updated_at": now()}),
            )
            .await;
        }
        None => {
            let _ = db::insert(
                "channel_translations",
                &json!({
                    "channel_id": channel_id,
                    "locale": "en",
                    "name": name,
                    "created_at": now(),
                    "updated_at": now(),
                }),
            )
            .await;
        }
    }
}

/// Reemplaza los pivotes `{table} ({channel_col}, {pivot_col})` de un canal.
async fn replace_pivots(
    table: &str,
    channel_col: &str,
    pivot_col: &str,
    channel_id: i64,
    values: &[Value],
) {
    // Nombres construidos solo con literales internas: no hay entrada de usuario.
    let _ = db::execute(
        &format!("DELETE FROM `{table}` WHERE `{channel_col}` = ?"),
        &[Value::from(channel_id)],
    )
    .await;
    for value in values {
        if let Some(id) = value.as_i64().or_else(|| value["id"].as_i64()) {
            let _ = db::execute(
                &format!(
                    "INSERT OR IGNORE INTO `{table}` (`{channel_col}`, `{pivot_col}`) VALUES (?, ?)"
                ),
                &[Value::from(channel_id), Value::from(id)],
            )
            .await;
        }
    }
}
