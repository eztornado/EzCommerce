//! Shop core endpoints (`/api/core/...`) and the store context the Flutter
//! shell needs. Mirrors `Webkul\Shop\Http\Controllers\API\CoreController` +
//! the `Locale`/`Currency` middleware behaviour.

use std::collections::HashMap;

use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::{HeaderMap, StatusCode};
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::core;
use crate::support::db;
use crate::support::response::{json_ok, message, resource, validation_error, ApiResult};

pub struct CoreController;

impl CoreController {
    /// `GET /api/core/context` - channel, locales, currencies, root category.
    /// Extension needed by the stateless Flutter clients (Bagisto embedded
    /// this data in the Blade layout).
    pub async fn context(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        match core::context_payload(&headers, &query).await {
            Ok(payload) => json_ok(payload),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/core/countries` - `{id, code, name}` list.
    pub async fn countries() -> Response {
        let countries = db::select(
            "SELECT `id`, `code`, `name` FROM `countries` ORDER BY `name`",
            &[],
        )
        .await
        .unwrap_or_default();
        json_ok(json!({"data": countries, "message": Value::Null}))
    }

    /// `GET /api/core/states` - states grouped by country code, as Bagisto's
    /// `getAllStates` does.
    pub async fn states() -> Response {
        let states = db::select(
            "SELECT `cs`.`code`, `cs`.`default_name`, `cs`.`country_code` FROM `country_states` `cs` \
             INNER JOIN `countries` `c` ON `c`.`code` = `cs`.`country_code` ORDER BY `cs`.`country_code`, `cs`.`default_name`",
            &[],
        )
        .await
        .unwrap_or_default();

        let mut grouped: serde_json::Map<String, Value> = serde_json::Map::new();
        for state in states {
            let country_code = state["country_code"].as_str().unwrap_or_default().to_string();
            let entry = grouped
                .entry(country_code)
                .or_insert_with(|| Value::Array(Vec::new()));
            if let Value::Array(list) = entry {
                list.push(json!({
                    "code": state["code"],
                    "default_name": state["default_name"],
                }));
            }
        }
        json_ok(json!({"data": grouped, "message": Value::Null}))
    }

    /// `GET /api/core/home` - the enabled theme sections of the channel
    /// (home page composition), mirroring `SectionRepository::findAllOfType`.
    pub async fn home(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let _ = &headers;
        let locale = query.get("locale").cloned().unwrap_or_else(|| "en".to_string());
        let rows = db::select(
            "SELECT `ts`.`id`, `ts`.`type`, `ts`.`name`, `ts`.`sort_order`, `tst`.`options` \
             FROM `theme_sections` `ts` \
             LEFT JOIN `theme_section_translations` `tst` ON `tst`.`theme_section_id` = `ts`.`id` AND `tst`.`locale` = ? \
             WHERE `ts`.`status` = 1 AND `ts`.`channel_id` = 1 ORDER BY `ts`.`sort_order`",
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
        json_ok(json!({"data": sections, "message": Value::Null}))
    }

    /// `GET /storage/{folder}/{file}` - static files uploaded to
    /// `public/storage` (product images, avatars...). Both segments are
    /// sanitised against traversal.
    pub async fn storage(Path((folder, file)): Path<(String, String)>) -> Response {
        use larust_support::axum::response::IntoResponse;

        for segment in [&folder, &file] {
            if segment.contains("..") || segment.starts_with('/') {
                return message(StatusCode::BAD_REQUEST, "Invalid path.");
            }
        }
        let full = crate::paths()
            .join("public/storage")
            .join(&folder)
            .join(&file);
        let Ok(bytes) = std::fs::read(&full) else {
            return message(StatusCode::NOT_FOUND, "File not found.");
        };
        let mime = mime_guess::from_path(&full).first_or_octet_stream().to_string();
        (
            StatusCode::OK,
            [
                (larust_support::axum::http::header::CONTENT_TYPE, mime),
                (
                    larust_support::axum::http::header::CACHE_CONTROL,
                    "public, max-age=86400".to_string(),
                ),
            ],
            bytes,
        )
            .into_response()
    }

    /// `POST /api/newsletter/subscribe` - `{email}`: marca al cliente como
    /// suscrito al newsletter (lo crea si no tiene cuenta).
    pub async fn newsletter_subscribe(Json(body): Json<Value>) -> Response {
        let Some(email) = body["email"]
            .as_str()
            .map(str::trim)
            .map(str::to_lowercase)
            .filter(|email| email.contains('@') && email.contains('.'))
        else {
            return validation_error(vec![("email".into(), "A valid email is required.".into())]);
        };
        let existing = db::select_one(
            "SELECT `id`, `subscribed_to_news_letter` FROM `customers` WHERE `email` = ? LIMIT 1",
            &[Value::String(email.clone())],
        )
        .await
        .ok()
        .flatten();
        match existing {
            Some(customer) => {
                if customer["subscribed_to_news_letter"].as_i64().unwrap_or(0) == 1 {
                    return resource(Value::Array(Vec::new()), "You are already subscribed.");
                }
                let _ = db::update(
                    "customers",
                    customer["id"].as_i64().unwrap_or(0),
                    &json!({"subscribed_to_news_letter": 1, "updated_at": db::now_db()}),
                )
                .await;
            }
            None => {
                let _ = db::insert(
                    "customers",
                    &json!({
                        "email": email,
                        "status": 1,
                        "subscribed_to_news_letter": 1,
                        "customer_group_id": 1,
                        "created_at": db::now_db(),
                        "updated_at": db::now_db(),
                    }),
                )
                .await;
            }
        }
        resource(Value::Array(Vec::new()), "You have been subscribed successfully.")
    }

    /// `POST /api/admin/upload` - stores a base64 file under
    /// `public/storage/{folder}` and returns the public path.
    pub async fn upload(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        use base64::Engine as _;

        let _ = admin;
        let Some(filename) = body["filename"].as_str().map(str::to_string) else {
            return crate::support::response::validation_error(vec![(
                "filename".into(),
                "The filename field is required.".into(),
            )]);
        };
        let Some(data_b64) = body["data"].as_str() else {
            return crate::support::response::validation_error(vec![(
                "data".into(),
                "The data field is required.".into(),
            )]);
        };
        let folder = body["folder"].as_str().unwrap_or("product").to_string();
        if folder.contains("..") || folder.contains('/') {
            return crate::support::response::validation_error(vec![(
                "folder".into(),
                "Invalid folder.".into(),
            )]);
        }

        let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data_b64) else {
            return crate::support::response::validation_error(vec![(
                "data".into(),
                "Invalid base64 payload.".into(),
            )]);
        };
        if bytes.len() > 8 * 1024 * 1024 {
            return message(StatusCode::PAYLOAD_TOO_LARGE, "The file is too large (max 8 MB).");
        }

        let extension = std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .unwrap_or_else(|| "bin".to_string());
        let allowed = ["png", "jpg", "jpeg", "webp", "gif", "svg", "pdf", "mp4"];
        if !allowed.contains(&extension.as_str()) {
            return crate::support::response::validation_error(vec![(
                "filename".into(),
                "Unsupported file type.".into(),
            )]);
        }

        let name = format!("{}.{}", uuid::Uuid::new_v4().simple(), extension);
        let dir = crate::paths().join("public/storage").join(&folder);
        let _ = std::fs::create_dir_all(&dir);
        let relative = format!("{folder}/{name}");
        let full = dir.join(&name);
        if std::fs::write(&full, &bytes).is_err() {
            return message(StatusCode::INTERNAL_SERVER_ERROR, "Unable to store the file.");
        }

        json_ok(json!({
            "path": relative,
            "url": format!("/storage/{relative}"),
        }))
    }

    /// `GET /api/core/currencies` - exchange-rate aware list of the channel.
    pub async fn currencies(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let result: ApiResult<Value> = async {
            let channel = core::current_channel(&headers).await?;
            let currencies = core::channel_currencies(channel["id"].as_i64().unwrap_or(0)).await?;
            let base = core::base_currency(&channel).await?;
            let base_code = base["code"].as_str().unwrap_or_default().to_string();
            let mut payload = Vec::new();
            for currency in currencies {
                let code = currency["code"].as_str().unwrap_or_default().to_string();
                let rate = core::exchange_rate(&base_code, &code).await;
                payload.push(json!({
                    "id": currency["id"],
                    "code": code,
                    "name": currency["name"],
                    "symbol": currency["symbol"],
                    "decimal": currency["decimal"],
                    "exchange_rate": 1.0 / rate,
                }));
            }
            Ok(json!(payload))
        }
        .await;
        json_ok(result.unwrap_or(Value::Null))
    }
}
