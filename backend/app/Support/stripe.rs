//! Stripe Checkout (redirect) integration: config resolution, Checkout
//! Session creation and webhook signature verification.
//!
//! Secrets are read from the environment first (`STRIPE_SECRET_KEY`,
//! `STRIPE_WEBHOOK_SECRET`, `STRIPE_PUBLISHABLE_KEY`) and fall back to
//! `core_config` for local/demo setups.

use hmac::{Hmac, Mac};
use larust_support::serde_json::{json, Value};
use sha2::Sha256;

use super::core;
use super::db;
use super::response::{ApiError, ApiResult};

const STRIPE_API: &str = "https://api.stripe.com/v1";

#[derive(Clone, Debug)]
pub struct StripeConfig {
    pub active: bool,
    pub mode: String,
    pub secret_key: String,
    pub publishable_key: String,
    pub webhook_secret: String,
    pub title: String,
    pub description: String,
    pub capture_method: String,
    pub statement_descriptor: String,
}

async fn config_value(field: &str, channel_code: &str) -> Option<String> {
    core::get_config_data(field, Some(channel_code), None)
        .await
        .ok()
        .flatten()
        .and_then(|value| {
            value
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn resolve(env_name: &str, field: &str, channel_code: &str) -> String {
    if let Some(value) = env_value(env_name) {
        return value;
    }
    config_value(field, channel_code)
        .await
        .unwrap_or_default()
}

fn is_enabled(value: Option<&str>) -> bool {
    matches!(
        value
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "on" | "yes"
    )
}

/// Loads the gateway settings for a channel (env vars win over `core_config`).
pub async fn load(channel_code: &str) -> StripeConfig {
    let active_raw = core::get_config_data(
        "sales.payment_methods.stripe.active",
        Some(channel_code),
        None,
    )
    .await
    .ok()
    .flatten();
    let active = is_enabled(active_raw.as_ref().and_then(Value::as_str));

    let secret_key = resolve(
        "STRIPE_SECRET_KEY",
        "sales.payment_methods.stripe.secret_key",
        channel_code,
    )
    .await;
    let publishable_key = resolve(
        "STRIPE_PUBLISHABLE_KEY",
        "sales.payment_methods.stripe.publishable_key",
        channel_code,
    )
    .await;
    let webhook_secret = resolve(
        "STRIPE_WEBHOOK_SECRET",
        "sales.payment_methods.stripe.webhook_secret",
        channel_code,
    )
    .await;
    let mode = match config_value("sales.payment_methods.stripe.mode", channel_code).await {
        Some(mode) if !mode.is_empty() => mode,
        _ if secret_key.starts_with("sk_live") => "live".to_string(),
        _ => "test".to_string(),
    };
    let title = config_value("sales.payment_methods.stripe.title", channel_code)
        .await
        .unwrap_or_else(|| "Card payment (Stripe)".to_string());
    let description =
        config_value("sales.payment_methods.stripe.description", channel_code)
            .await
            .unwrap_or_else(|| "Pay securely with your credit or debit card.".to_string());
    let capture_method =
        config_value("sales.payment_methods.stripe.capture_method", channel_code)
            .await
            .unwrap_or_else(|| "automatic".to_string());
    let statement_descriptor =
        config_value("sales.payment_methods.stripe.statement_descriptor", channel_code)
            .await
            .unwrap_or_default();

    StripeConfig {
        active,
        mode,
        secret_key,
        publishable_key,
        webhook_secret,
        title,
        description,
        capture_method,
        statement_descriptor,
    }
}

/// Gateway usable at checkout: enabled and with a secret key available.
pub async fn available(channel_code: &str) -> Option<StripeConfig> {
    let config = load(channel_code).await;
    if config.active && !config.secret_key.is_empty() {
        Some(config)
    } else {
        None
    }
}

/// Webhook signing secret for the channel (environment first).
pub async fn webhook_secret(channel_code: Option<&str>) -> String {
    resolve(
        "STRIPE_WEBHOOK_SECRET",
        "sales.payment_methods.stripe.webhook_secret",
        channel_code.unwrap_or("default"),
    )
    .await
}

fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn encode_form(fields: &[(String, String)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{}={}", encode(key), encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn is_zero_decimal(currency: &str) -> bool {
    matches!(
        currency,
        "BIF" | "CLP"
            | "DJF"
            | "GNF"
            | "JPY"
            | "KMF"
            | "KRW"
            | "MGA"
            | "PYG"
            | "RWF"
            | "UGX"
            | "VND"
            | "VUV"
            | "XAF"
            | "XOF"
            | "XPF"
    )
}

fn is_three_decimal(currency: &str) -> bool {
    matches!(currency, "BHD" | "JOD" | "KWD" | "OMR" | "TND")
}

/// Stripe amounts are integers in the currency's minor unit.
pub fn minor_units(amount: f64, currency: &str) -> i64 {
    let currency = currency.to_ascii_uppercase();
    let factor = if is_zero_decimal(&currency) {
        1.0
    } else if is_three_decimal(&currency) {
        1000.0
    } else {
        100.0
    };
    (amount * factor).round() as i64
}

/// Converts a Stripe minor-unit amount back to a decimal amount.
pub fn major_units(amount: i64, currency: &str) -> f64 {
    let currency = currency.to_ascii_uppercase();
    let factor = if is_zero_decimal(&currency) {
        1.0
    } else if is_three_decimal(&currency) {
        1000.0
    } else {
        100.0
    };
    amount as f64 / factor
}

async fn stripe_request(
    method: reqwest::Method,
    path: &str,
    secret_key: &str,
    body: Option<String>,
) -> ApiResult<Value> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(25))
        .build()
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let mut request = client
        .request(method, format!("{STRIPE_API}{path}"))
        .bearer_auth(secret_key);
    if let Some(body) = body {
        request = request
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body);
    }
    let response = request.send().await?;
    let status = response.status();
    let payload: Value = response.json().await.unwrap_or_else(|_| json!({}));
    if !status.is_success() {
        let message = payload["error"]["message"]
            .as_str()
            .unwrap_or("Stripe rejected the request.");
        return Err(ApiError::unprocessable(format!("Stripe: {message}")));
    }
    Ok(payload)
}

/// Creates a hosted Stripe Checkout Session and stores its id on the order
/// payment. Returns the Stripe session payload (`id`, `url`, ...).
pub async fn create_checkout_session(order: &Value, origin: &str) -> ApiResult<Value> {
    let order_id = order["id"].as_i64().unwrap_or(0);
    let channel_code = order["channel_name"].as_str().unwrap_or("default");
    let config = load(channel_code).await;
    if !config.active || config.secret_key.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Stripe is not configured for this sales channel.",
        ));
    }

    let currency = order["order_currency_code"]
        .as_str()
        .or_else(|| order["base_currency_code"].as_str())
        .unwrap_or("EUR")
        .to_ascii_lowercase();
    let amount = order["grand_total"]
        .as_f64()
        .or_else(|| order["base_grand_total"].as_f64())
        .unwrap_or(0.0);
    let amount_minor = minor_units(amount, &currency);
    if amount_minor <= 0 {
        return Err(ApiError::unprocessable(
            "The order total must be greater than zero to start a Stripe payment.",
        ));
    }

    let origin = origin.trim_end_matches('/');
    let api_prefix = crate::config::app::config()["api_prefix"]
        .as_str()
        .unwrap_or("/api")
        .trim_end_matches('/')
        .to_string();
    let increment_id = order["increment_id"].as_str().unwrap_or_default();
    let success_url = format!(
        "{origin}/checkout/success/{order_id}?session_id={{CHECKOUT_SESSION_ID}}"
    );
    let cancel_url = format!("{origin}{api_prefix}/checkout/onepage/stripe/cancel/{order_id}");
    let description = format!("Order #{increment_id}");

    let line_items = checkout_line_items(
        order,
        order_id,
        &currency,
        &config,
        increment_id,
    )
    .await?;
    let mut fields: Vec<(String, String)> = vec![
        ("mode".into(), "payment".into()),
        ("success_url".into(), success_url),
        ("cancel_url".into(), cancel_url),
        ("client_reference_id".into(), order_id.to_string()),
        ("metadata[order_id]".into(), order_id.to_string()),
        ("metadata[increment_id]".into(), increment_id.to_string()),
        ("metadata[channel_code]".into(), channel_code.to_string()),
        (
            "payment_intent_data[metadata][order_id]".into(),
            order_id.to_string(),
        ),
        (
            "payment_intent_data[metadata][channel_code]".into(),
            channel_code.to_string(),
        ),
        (
            "payment_intent_data[description]".into(),
            description.clone(),
        ),
        // Las sesiones abandonadas expiran a los ~31 minutos y el webhook
        // libera la reserva de stock.
        (
            "expires_at".into(),
            (chrono::Utc::now().timestamp() + 31 * 60).to_string(),
        ),
    ];
    fields.extend(line_items);
    if let Some(email) = order["customer_email"]
        .as_str()
        .map(str::trim)
        .filter(|email| !email.is_empty())
    {
        fields.push(("customer_email".into(), email.to_string()));
    }
    if config.capture_method.eq_ignore_ascii_case("manual") {
        fields.push((
            "payment_intent_data[capture_method]".into(),
            "manual".into(),
        ));
    }
    if !config.statement_descriptor.trim().is_empty() {
        let descriptor: String = config
            .statement_descriptor
            .trim()
            .chars()
            .take(22)
            .collect();
        fields.push((
            "payment_intent_data[statement_descriptor]".into(),
            descriptor,
        ));
    }

    let session = stripe_request(
        reqwest::Method::POST,
        "/checkout/sessions",
        &config.secret_key,
        Some(encode_form(&fields)),
    )
    .await?;

    merge_payment_additional(
        order_id,
        json!({
            "checkout_session_id": session["id"],
            "checkout_origin": origin,
            "mode": config.mode,
            "payment_intent_id": session["payment_intent"],
            "checkout_url": session["url"],
        }),
    )
    .await?;

    Ok(session)
}

fn single_line_item(amount_minor: i64, currency: &str, description: &str) -> Vec<(String, String)> {
    vec![
        ("line_items[0][price_data][currency]".into(), currency.to_string()),
        (
            "line_items[0][price_data][unit_amount]".into(),
            amount_minor.max(0).to_string(),
        ),
        (
            "line_items[0][price_data][product_data][name]".into(),
            description.to_string(),
        ),
        ("line_items[0][quantity]".into(), "1".into()),
    ]
}

async fn create_discount_coupon(
    config: &StripeConfig,
    order_id: i64,
    increment_id: &str,
    discount_minor: i64,
    currency: &str,
) -> ApiResult<String> {
    let fields = vec![
        ("amount_off".to_string(), discount_minor.to_string()),
        ("currency".to_string(), currency.to_string()),
        ("duration".to_string(), "once".to_string()),
        (
            "name".to_string(),
            format!("Order #{increment_id} discount"),
        ),
        ("metadata[order_id]".to_string(), order_id.to_string()),
    ];
    let coupon = stripe_request(
        reqwest::Method::POST,
        "/coupons",
        &config.secret_key,
        Some(encode_form(&fields)),
    )
    .await?;
    Ok(coupon["id"].as_str().unwrap_or_default().to_string())
}

/// Itemized Checkout line items. If the sum of products, tax and shipping
/// minus discounts does not match the order grand total, it falls back to a
/// single line with the exact total so the charge is always correct.
async fn checkout_line_items(
    order: &Value,
    order_id: i64,
    currency: &str,
    config: &StripeConfig,
    increment_id: &str,
) -> ApiResult<Vec<(String, String)>> {
    let description = format!("Order #{increment_id}");
    let grand_minor = minor_units(
        order["grand_total"]
            .as_f64()
            .or_else(|| order["base_grand_total"].as_f64())
            .unwrap_or(0.0),
        currency,
    );
    let fallback = || single_line_item(grand_minor, currency, &description);

    let items = db::select(
        "SELECT `name`, `sku`, `qty_ordered`, `qty_canceled`, `price` \
         FROM `order_items` WHERE `order_id` = ? AND `parent_id` IS NULL ORDER BY `id`",
        &[Value::from(order_id)],
    )
    .await
    .unwrap_or_default();
    if items.is_empty() {
        return Ok(fallback());
    }

    let mut fields: Vec<(String, String)> = Vec::new();
    let mut items_minor = 0i64;
    let mut index = 0usize;
    for item in &items {
        let qty = item["qty_ordered"].as_i64().unwrap_or(0)
            - item["qty_canceled"].as_i64().unwrap_or(0);
        let unit = minor_units(item["price"].as_f64().unwrap_or(0.0), currency);
        if qty <= 0 || unit <= 0 {
            continue;
        }
        let name = item["name"].as_str().unwrap_or("Item");
        fields.push((
            format!("line_items[{index}][price_data][currency]"),
            currency.to_string(),
        ));
        fields.push((
            format!("line_items[{index}][price_data][unit_amount]"),
            unit.to_string(),
        ));
        fields.push((
            format!("line_items[{index}][price_data][product_data][name]"),
            name.to_string(),
        ));
        if let Some(sku) = item["sku"].as_str().map(str::trim).filter(|sku| !sku.is_empty()) {
            fields.push((
                format!("line_items[{index}][price_data][product_data][description]"),
                sku.to_string(),
            ));
        }
        fields.push((
            format!("line_items[{index}][quantity]"),
            qty.to_string(),
        ));
        items_minor += unit * qty;
        index += 1;
    }

    let mut append_line = |name: &str, amount_minor: i64| {
        if amount_minor <= 0 {
            return;
        }
        fields.push((
            format!("line_items[{index}][price_data][currency]"),
            currency.to_string(),
        ));
        fields.push((
            format!("line_items[{index}][price_data][unit_amount]"),
            amount_minor.to_string(),
        ));
        fields.push((
            format!("line_items[{index}][price_data][product_data][name]"),
            name.to_string(),
        ));
        fields.push((format!("line_items[{index}][quantity]"), "1".into()));
        index += 1;
    };
    let tax_minor = minor_units(order["tax_amount"].as_f64().unwrap_or(0.0), currency);
    let shipping_minor = minor_units(
        order["shipping_amount"].as_f64().unwrap_or(0.0),
        currency,
    );
    append_line("Tax", tax_minor);
    append_line("Shipping", shipping_minor);
    drop(append_line);

    let discount_minor = minor_units(
        order["discount_amount"].as_f64().unwrap_or(0.0),
        currency,
    );
    let expected = items_minor + tax_minor + shipping_minor - discount_minor;
    if index == 0 || expected != grand_minor {
        return Ok(fallback());
    }
    if discount_minor > 0 {
        match create_discount_coupon(
            config,
            order_id,
            increment_id,
            discount_minor,
            currency,
        )
        .await
        {
            Ok(coupon) if !coupon.is_empty() => {
                fields.push(("discounts[0][coupon]".into(), coupon));
            }
            _ => return Ok(fallback()),
        }
    }
    Ok(fields)
}

pub async fn retrieve_checkout_session(
    config: &StripeConfig,
    session_id: &str,
) -> ApiResult<Value> {
    stripe_request(
        reqwest::Method::GET,
        &format!("/checkout/sessions/{}", encode(session_id)),
        &config.secret_key,
        None,
    )
    .await
}

/// Expires a hosted Checkout Session so it cannot be paid after the customer
/// canceled and the order was released.
pub async fn expire_checkout_session(
    config: &StripeConfig,
    session_id: &str,
) -> ApiResult<Value> {
    stripe_request(
        reqwest::Method::POST,
        &format!("/checkout/sessions/{}/expire", encode(session_id)),
        &config.secret_key,
        Some(String::new()),
    )
    .await
}

/// Retrieves the PaymentIntent with its latest charge expanded so the admin
/// can display card brand, last four digits and the receipt link.
pub async fn retrieve_payment_intent(
    config: &StripeConfig,
    payment_intent_id: &str,
) -> ApiResult<Value> {
    stripe_request(
        reqwest::Method::GET,
        &format!(
            "/payment_intents/{}?expand[]=latest_charge",
            encode(payment_intent_id)
        ),
        &config.secret_key,
        None,
    )
    .await
}

/// Creates a Stripe refund for the order's PaymentIntent.
pub async fn refund_payment(
    order_id: i64,
    amount: f64,
    currency: &str,
    reason: &str,
) -> ApiResult<Value> {
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;
    let config = load(order["channel_name"].as_str().unwrap_or("default")).await;
    if config.secret_key.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Stripe is not configured for this sales channel.",
        ));
    }
    let additional = db::select_one(
        "SELECT `additional` FROM `order_payment` WHERE `order_id` = ? ORDER BY `id` DESC LIMIT 1",
        &[Value::from(order_id)],
    )
    .await?
    .and_then(|row| {
        row["additional"]
            .as_str()
            .and_then(|raw| larust_support::serde_json::from_str::<Value>(raw).ok())
    })
    .unwrap_or_else(|| json!({}));
    let payment_intent = additional["payment_intent_id"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ApiError::unprocessable("This order has no Stripe PaymentIntent to refund.")
        })?;
    let amount_minor = minor_units(amount, currency);
    if amount_minor <= 0 {
        return Err(ApiError::unprocessable(
            "The refund amount must be greater than zero.",
        ));
    }
    let reason = match reason {
        "duplicate" | "fraudulent" | "requested_by_customer" => reason,
        _ => "requested_by_customer",
    };
    let fields = vec![
        ("payment_intent".to_string(), payment_intent.to_string()),
        ("amount".to_string(), amount_minor.to_string()),
        ("reason".to_string(), reason.to_string()),
        ("metadata[order_id]".to_string(), order_id.to_string()),
    ];
    stripe_request(
        reqwest::Method::POST,
        "/refunds",
        &config.secret_key,
        Some(encode_form(&fields)),
    )
    .await
}

/// Captures a PaymentIntent authorized with manual capture.
pub async fn capture_payment_intent(
    order_id: i64,
    payment_intent_id: &str,
) -> ApiResult<Value> {
    let order = db::find_by_id("orders", order_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Order not found"))?;
    let config = load(order["channel_name"].as_str().unwrap_or("default")).await;
    if config.secret_key.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Stripe is not configured for this sales channel.",
        ));
    }
    stripe_request(
        reqwest::Method::POST,
        &format!(
            "/payment_intents/{}/capture",
            encode(payment_intent_id)
        ),
        &config.secret_key,
        Some(String::new()),
    )
    .await
}

/// Extracts the payment details we persist on `order_payment.additional`.
pub fn payment_details(intent: &Value) -> Value {
    let charge = &intent["latest_charge"];
    json!({
        "payment_intent_id": intent["id"],
        "status": intent["status"],
        "brand": charge["payment_method_details"]["card"]["brand"],
        "last4": charge["payment_method_details"]["card"]["last4"],
        "wallet": charge["payment_method_details"]["card"]["wallet"]["type"],
        "receipt_url": charge["receipt_url"],
    })
}

async fn merge_payment_additional(order_id: i64, extra: Value) -> ApiResult<()> {
    let row = db::select_one(
        "SELECT `id`, `additional` FROM `order_payment` WHERE `order_id` = ? ORDER BY `id` DESC LIMIT 1",
        &[Value::from(order_id)],
    )
    .await?;
    let Some(row) = row else {
        return Ok(());
    };
    let mut data = row["additional"]
        .as_str()
        .and_then(|raw| larust_support::serde_json::from_str::<Value>(raw).ok())
        .unwrap_or_else(|| json!({}));
    if !data.is_object() {
        data = json!({});
    }
    if let (Some(target), Some(source)) = (data.as_object_mut(), extra.as_object()) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    db::update(
        "order_payment",
        row["id"].as_i64().unwrap_or(0),
        &json!({"additional": data, "updated_at": db::now_db()}),
    )
    .await?;
    Ok(())
}

/// Verifies the `Stripe-Signature` header (HMAC-SHA256, 5-minute tolerance).
pub fn verify_signature(payload: &[u8], signature_header: &str, secret: &str) -> bool {
    if secret.trim().is_empty() || signature_header.trim().is_empty() {
        return false;
    }
    let mut timestamp: Option<i64> = None;
    let mut signatures: Vec<String> = Vec::new();
    for part in signature_header.split(',') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("t=") {
            timestamp = value.parse::<i64>().ok();
        } else if let Some(value) = part.strip_prefix("v1=") {
            signatures.push(value.to_string());
        }
    }
    let Some(timestamp) = timestamp else {
        return false;
    };
    let now = chrono::Utc::now().timestamp();
    if (now - timestamp).abs() > 300 {
        return false;
    }
    let signed = format!("{timestamp}.{}", String::from_utf8_lossy(payload));
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(signed.as_bytes());
    let expected = mac.finalize().into_bytes();
    signatures.iter().any(|signature| {
        hex::decode(signature)
            .map(|bytes| bytes.as_slice() == expected.as_slice())
            .unwrap_or(false)
    })
}
