//! Shop core endpoints (`/api/core/...`) and the store context the Flutter
//! shell needs. Mirrors `Webkul\Shop\Http\Controllers\API\CoreController` +
//! the `Locale`/`Currency` middleware behaviour.

use std::collections::HashMap;

use larust_support::axum::extract::Query;
use larust_support::axum::http::HeaderMap;
use larust_support::axum::response::Response;
use larust_support::serde_json::{json, Value};

use crate::support::core;
use crate::support::db;
use crate::support::response::{json_ok, ApiResult};

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
