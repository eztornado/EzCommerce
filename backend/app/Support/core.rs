//! Shop context: channel / locale / currency resolution and formatting.
//! Mirrors `Webkul\Core\Core` + the `Locale`/`Currency` middlewares.

use larust_support::axum::http::HeaderMap;
use larust_support::serde_json::{json, Map, Value};

use super::db;
use super::response::{ApiError, ApiResult};

pub type RowMap = Map<String, Value>;

/// The active channel: by `Host` header hostname, else the first channel.
pub async fn current_channel(headers: &HeaderMap) -> ApiResult<RowMap> {
    let host = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(':').next().unwrap_or(value).to_string());

    let row = if let Some(host) = host.as_deref() {
        db::select_one(
            "SELECT * FROM `channels` WHERE `hostname` = ? LIMIT 1",
            &[Value::String(host.to_string())],
        )
        .await?
        .or(db::select_one("SELECT * FROM `channels` ORDER BY `id` LIMIT 1", &[]).await?)
    } else {
        db::select_one("SELECT * FROM `channels` ORDER BY `id` LIMIT 1", &[]).await?
    };

    row.ok_or_else(|| ApiError::internal("No channel configured"))
}

/// Locales enabled on the channel.
pub async fn channel_locales(channel_id: i64) -> ApiResult<Vec<RowMap>> {
    db::select(
        "SELECT `l`.* FROM `locales` `l` \
         INNER JOIN `channel_locales` `cl` ON `cl`.`locale_id` = `l`.`id` \
         WHERE `cl`.`channel_id` = ? ORDER BY `l`.`id`",
        &[Value::from(channel_id)],
    )
    .await
}

/// Currencies enabled on the channel.
pub async fn channel_currencies(channel_id: i64) -> ApiResult<Vec<RowMap>> {
    db::select(
        "SELECT `c`.* FROM `currencies` `c` \
         INNER JOIN `channel_currencies` `cc` ON `cc`.`currency_id` = `c`.`id` \
         WHERE `cc`.`channel_id` = ? ORDER BY `c`.`id`",
        &[Value::from(channel_id)],
    )
    .await
}

pub async fn default_locale(channel: &RowMap) -> ApiResult<RowMap> {
    let id = channel["default_locale_id"].as_i64().unwrap_or(1);
    db::find_by_id("locales", id)
        .await?
        .ok_or_else(|| ApiError::internal("No default locale"))
}

pub async fn base_currency(channel: &RowMap) -> ApiResult<RowMap> {
    let id = channel["base_currency_id"].as_i64().unwrap_or(1);
    db::find_by_id("currencies", id)
        .await?
        .ok_or_else(|| ApiError::internal("No base currency"))
}

/// Resolves the requested locale: `?locale=` validated against the channel,
/// else the channel default (mirrors the `Locale` middleware).
pub async fn resolve_locale(channel: &RowMap, query_locale: Option<&str>) -> ApiResult<RowMap> {
    if let Some(code) = query_locale {
        let available = channel_locales(channel["id"].as_i64().unwrap_or(0)).await?;
        if let Some(matched) = available
            .iter()
            .find(|locale| locale["code"].as_str() == Some(code))
        {
            return Ok(matched.clone());
        }
    }
    default_locale(channel).await
}

/// Resolves the requested currency: `?currency=` validated against the
/// channel, else the channel base currency (mirrors the `Currency` middleware).
pub async fn resolve_currency(channel: &RowMap, query_currency: Option<&str>) -> ApiResult<RowMap> {
    if let Some(code) = query_currency {
        let available = channel_currencies(channel["id"].as_i64().unwrap_or(0)).await?;
        if let Some(matched) = available
            .iter()
            .find(|currency| currency["code"].as_str() == Some(code))
        {
            return Ok(matched.clone());
        }
    }
    base_currency(channel).await
}

/// `core()->formatPrice()`: symbol + grouped number with the currency's
/// decimals (defaults to 2).
pub fn format_price(amount: f64, currency: &RowMap) -> String {
    let symbol = currency["symbol"].as_str().unwrap_or("");
    let decimals = currency["decimal"].as_i64().unwrap_or(2) as usize;
    format!("{}{}", symbol, format_amount(amount, decimals))
}

/// `1234.5` -> `1,234.50`.
pub fn format_amount(amount: f64, decimals: usize) -> String {
    let value = format!("{:.*}", decimals, amount);
    let (integer, fraction) = value.split_once('.').unwrap_or((&value, ""));
    let mut grouped = String::new();
    let chars: Vec<char> = integer.chars().collect();
    for (index, character) in chars.iter().enumerate() {
        if index > 0 && (chars.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(*character);
    }
    if fraction.is_empty() {
        grouped
    } else {
        format!("{grouped}.{fraction}")
    }
}

/// Exchange rate of `code` relative to the channel base currency
/// (`currency_exchange_rates.target_currency`).
pub async fn exchange_rate(base_code: &str, code: &str) -> f64 {
    if base_code == code {
        return 1.0;
    }
    let row = db::select_one(
        "SELECT `er`.`rate` FROM `currency_exchange_rates` `er` \
         INNER JOIN `currencies` `c` ON `c`.`id` = `er`.`target_currency` \
         WHERE `c`.`code` = ? LIMIT 1",
        &[Value::String(code.to_string())],
    )
    .await
    .ok()
    .flatten();
    row.and_then(|row| row["rate"].as_f64())
        .filter(|rate| *rate > 0.0)
        .unwrap_or(1.0)
}

/// `core()->convertPrice()`: base amount -> currency amount.
pub async fn convert_price(amount: f64, base_code: &str, code: &str) -> f64 {
    amount * exchange_rate(base_code, code).await
}

/// `core()->convertToBasePrice()`: currency amount -> base amount.
pub async fn convert_to_base(amount: f64, base_code: &str, code: &str) -> f64 {
    amount / exchange_rate(base_code, code).await
}

/// `core()->getConfigData()`: `core_config` lookup with channel+locale
/// precedence.
pub async fn get_config_data(
    field: &str,
    channel_code: Option<&str>,
    locale_code: Option<&str>,
) -> ApiResult<Option<Value>> {
    let mut sql = String::from("SELECT `value` FROM `core_config` WHERE `code` = ?");
    let mut binds = vec![Value::String(field.to_string())];
    if let Some(channel) = channel_code {
        sql.push_str(" AND (`channel_code` = ? OR `channel_code` IS NULL)");
        binds.push(Value::String(channel.to_string()));
    } else {
        sql.push_str(" AND `channel_code` IS NULL");
    }
    if let Some(locale) = locale_code {
        sql.push_str(" AND (`locale_code` = ? OR `locale_code` IS NULL)");
        binds.push(Value::String(locale.to_string()));
    } else {
        sql.push_str(" AND `locale_code` IS NULL");
    }
    sql.push_str(" ORDER BY `channel_code` IS NULL, `locale_code` IS NULL LIMIT 1");

    let row = db::select_one(&sql, &binds).await?;
    Ok(row.map(|row| row["value"].clone()))
}

/// Product attribute value read (EAV): the value column by attribute type.
pub fn attribute_value(attribute_type: &str, row: &RowMap) -> Value {
    match attribute_type {
        "boolean" => row["boolean_value"].clone(),
        "integer" | "select" => row["integer_value"].clone(),
        "price" | "float" | "decimal" => row["float_value"].clone(),
        "date" => row["date_value"].clone(),
        "datetime" => row["datetime_value"].clone(),
        "multiselect" | "checkbox" => row["json_value"].clone(),
        _ => row["text_value"].clone(),
    }
}

/// Resource metadata shared by shop endpoints: active channel, locale,
/// currency and their catalogues.
pub async fn context_payload(
    headers: &HeaderMap,
    query: &std::collections::HashMap<String, String>,
) -> ApiResult<Value> {
    let channel = current_channel(headers).await?;
    let locale = resolve_locale(&channel, query.get("locale").map(String::as_str)).await?;
    let currency = resolve_currency(&channel, query.get("currency").map(String::as_str)).await?;
    let locales = channel_locales(channel["id"].as_i64().unwrap_or(0)).await?;
    let currencies = channel_currencies(channel["id"].as_i64().unwrap_or(0)).await?;

    Ok(json!({
        "channel": {
            "id": channel["id"],
            "code": channel["code"],
            "name": channel_name(&channel).await,
            "theme": channel["theme"],
            "logo": channel["logo"],
            "favicon": channel["favicon"],
            "base_currency_code": base_currency(&channel).await.map(|c| c["code"].clone()).unwrap_or(Value::Null),
            "default_locale_code": default_locale(&channel).await.map(|l| l["code"].clone()).unwrap_or(Value::Null),
        },
        "locale": locale["code"].clone(),
        "currency": currency["code"].clone(),
        "locales": locales.iter().map(|l| json!({
            "id": l["id"], "code": l["code"], "name": l["name"], "direction": l["direction"],
        })).collect::<Vec<_>>(),
        "currencies": currencies.iter().map(|c| json!({
            "id": c["id"], "code": c["code"], "name": c["name"], "symbol": c["symbol"],
        })).collect::<Vec<_>>(),
    }))
}

async fn channel_name(channel: &RowMap) -> Value {
    db::select_one(
        "SELECT `name` FROM `channel_translations` WHERE `channel_id` = ? ORDER BY `id` LIMIT 1",
        &[channel["id"].clone()],
    )
    .await
    .ok()
    .flatten()
    .map(|row| row["name"].clone())
    .unwrap_or(Value::Null)
}
