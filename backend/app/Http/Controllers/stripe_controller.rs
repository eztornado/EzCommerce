//! Stripe webhook + redirect return endpoints.
//!
//! - `POST /api/webhooks/stripe` receives signed events (no admin auth).
//! - `GET /api/checkout/onepage/stripe/verify/{order_id}` confirms the
//!   Checkout Session when the browser comes back (fallback for local setups
//!   without a public webhook).
//! - `GET /api/checkout/onepage/stripe/cancel/{order_id}` releases the order
//!   and restores the cart when the customer cancels on Stripe.

use std::collections::HashMap;

use larust_support::axum::body::Bytes;
use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::{HeaderMap, StatusCode};
use larust_support::axum::response::{IntoResponse, Redirect, Response};
use larust_support::serde_json::{json, Value};

use crate::support::db;
use crate::support::response::{json_ok, message, resource, ApiError, ApiResult};
use crate::support::sales;
use crate::support::stripe;

pub struct StripeController;

fn now() -> String {
    db::now_db()
}

fn app_url() -> String {
    if let Ok(frontend) = std::env::var("FRONTEND_URL") {
        let frontend = frontend.trim().trim_end_matches('/');
        if !frontend.is_empty() {
            return frontend.to_string();
        }
    }
    crate::config::app::config()["app_url"]
        .as_str()
        .unwrap_or("http://localhost:8000")
        .trim_end_matches('/')
        .to_string()
}

async fn payment_data(order_id: i64) -> Value {
    db::select_one(
        "SELECT `additional` FROM `order_payment` WHERE `order_id` = ? ORDER BY `id` DESC LIMIT 1",
        &[Value::from(order_id)],
    )
    .await
    .ok()
    .flatten()
    .and_then(|row| {
        row["additional"]
            .as_str()
            .and_then(|raw| larust_support::serde_json::from_str::<Value>(raw).ok())
    })
    .unwrap_or_else(|| json!({}))
}

fn order_id_from_object(object: &Value) -> Option<i64> {
    object["metadata"]["order_id"]
        .as_str()
        .and_then(|value| value.parse::<i64>().ok())
        .or_else(|| {
            object["client_reference_id"]
                .as_str()
                .and_then(|value| value.parse::<i64>().ok())
        })
}

async fn reactivate_cart(order: &db::RowMap) {
    if let Some(cart_id) = order["cart_id"].as_i64() {
        if cart_id > 0 {
            let _ = db::update(
                "cart",
                cart_id,
                &json!({"is_active": 1, "updated_at": now()}),
            )
            .await;
        }
    }
}

async fn confirm_from_session(session: &Value, channel_code: &str) -> ApiResult<()> {
    let order_id = order_id_from_object(session)
        .ok_or_else(|| ApiError::bad_request("Stripe session without order reference."))?;
    let config = stripe::load(channel_code).await;
    let payment_intent = session["payment_intent"].as_str().unwrap_or_default();
    let currency = session["currency"]
        .as_str()
        .unwrap_or("eur")
        .to_ascii_uppercase();
    let mut amount = stripe::major_units(
        session["amount_total"].as_i64().unwrap_or(0),
        &currency,
    );
    if amount <= 0.0 {
        if let Ok(Some(order)) = db::find_by_id("orders", order_id).await {
            amount = order["grand_total"]
                .as_f64()
                .or_else(|| order["base_grand_total"].as_f64())
                .unwrap_or(0.0);
        }
    }

    let mut data = json!({
        "checkout_session_id": session["id"],
        "payment_status": session["payment_status"],
    });
    if !payment_intent.is_empty() {
        data["payment_intent_id"] = json!(payment_intent);
    }
    if let Some(origin) = payment_data(order_id).await["checkout_origin"].as_str() {
        data["checkout_origin"] = json!(origin);
    }
    if !payment_intent.is_empty() {
        if let Ok(intent) = stripe::retrieve_payment_intent(&config, payment_intent).await {
            if let (Some(target), Some(details)) =
                (data.as_object_mut(), stripe::payment_details(&intent).as_object())
            {
                for (key, value) in details {
                    target.insert(key.clone(), value.clone());
                }
            }
        }
    }

    let transaction_id = if payment_intent.is_empty() {
        session["id"].as_str().unwrap_or_default().to_string()
    } else {
        payment_intent.to_string()
    };
    sales::confirm_payment(order_id, &transaction_id, amount, &currency, data).await
}

async fn confirm_from_intent(intent: &Value, channel_code: &str) -> ApiResult<()> {
    let order_id = order_id_from_object(intent)
        .ok_or_else(|| ApiError::bad_request("Stripe payment intent without order reference."))?;
    let config = stripe::load(channel_code).await;
    let payment_intent_id = intent["id"].as_str().unwrap_or_default();
    let expanded = match stripe::retrieve_payment_intent(&config, payment_intent_id).await {
        Ok(value) => value,
        Err(_) => intent.clone(),
    };
    let currency = expanded["currency"]
        .as_str()
        .unwrap_or("eur")
        .to_ascii_uppercase();
    let amount_minor = expanded["amount_received"]
        .as_i64()
        .filter(|amount| *amount > 0)
        .or_else(|| expanded["amount"].as_i64())
        .unwrap_or(0);
    let mut amount = stripe::major_units(amount_minor, &currency);
    if amount <= 0.0 {
        if let Ok(Some(order)) = db::find_by_id("orders", order_id).await {
            amount = order["grand_total"]
                .as_f64()
                .or_else(|| order["base_grand_total"].as_f64())
                .unwrap_or(0.0);
        }
    }
    sales::confirm_payment(
        order_id,
        payment_intent_id,
        amount,
        &currency,
        stripe::payment_details(&expanded),
    )
    .await
}

async fn authorize_from_intent(intent: &Value, channel_code: &str) -> ApiResult<()> {
    let order_id = order_id_from_object(intent)
        .ok_or_else(|| ApiError::bad_request("Stripe payment intent without order reference."))?;
    let config = stripe::load(channel_code).await;
    let payment_intent_id = intent["id"].as_str().unwrap_or_default();
    let expanded = match stripe::retrieve_payment_intent(&config, payment_intent_id).await {
        Ok(value) => value,
        Err(_) => intent.clone(),
    };
    let currency = expanded["currency"]
        .as_str()
        .unwrap_or("eur")
        .to_ascii_uppercase();
    let amount_minor = expanded["amount_capturable"]
        .as_i64()
        .filter(|amount| *amount > 0)
        .or_else(|| expanded["amount"].as_i64())
        .unwrap_or(0);
    sales::mark_payment_authorized(
        order_id,
        payment_intent_id,
        stripe::major_units(amount_minor, &currency),
        &currency,
        stripe::payment_details(&expanded),
    )
    .await
}

async fn authorize_from_session(session: &Value, channel_code: &str) -> ApiResult<()> {
    let Some(payment_intent) = session["payment_intent"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    let config = stripe::load(channel_code).await;
    let intent = stripe::retrieve_payment_intent(&config, payment_intent).await?;
    if intent["status"].as_str() != Some("requires_capture") {
        return Ok(());
    }
    authorize_from_intent(&intent, channel_code).await
}

async fn resolve_dispute_order_id(object: &Value, channel_code: &str) -> Option<i64> {
    if let Some(order_id) = order_id_from_object(object) {
        return Some(order_id);
    }
    let payment_intent = &object["payment_intent"];
    if let Some(order_id) = order_id_from_object(payment_intent) {
        return Some(order_id);
    }
    let payment_intent_id = payment_intent
        .as_str()
        .or_else(|| payment_intent["id"].as_str())?;
    let config = stripe::load(channel_code).await;
    let intent = stripe::retrieve_payment_intent(&config, payment_intent_id)
        .await
        .ok()?;
    order_id_from_object(&intent)
}

async fn process_event(event: &Value, channel_code: &str) -> ApiResult<()> {
    let event_type = event["type"].as_str().unwrap_or_default();
    let object = &event["data"]["object"];
    match event_type {
        "checkout.session.completed" | "checkout.session.async_payment_succeeded" => {
            if object["payment_status"].as_str() == Some("paid") {
                confirm_from_session(object, channel_code).await?;
            } else {
                // Captura manual: el pago queda autorizado hasta que el
                // administrador pulse "Capture payment".
                authorize_from_session(object, channel_code).await?;
            }
        }
        "payment_intent.amount_capturable_updated" => {
            if object["status"].as_str() == Some("requires_capture") {
                authorize_from_intent(object, channel_code).await?;
            }
        }
        "checkout.session.async_payment_failed" => {
            if let Some(order_id) = order_id_from_object(object) {
                sales::mark_payment_failed(
                    order_id,
                    "Stripe reported a failed asynchronous payment.",
                )
                .await?;
            }
        }
        "checkout.session.expired" => {
            if let Some(order_id) = order_id_from_object(object) {
                if let Ok(Some(order)) = db::find_by_id("orders", order_id).await {
                    let _ = sales::cancel_order(order_id).await;
                    let _ =
                        sales::mark_payment_failed(order_id, "Stripe Checkout Session expired.")
                            .await;
                    reactivate_cart(&order).await;
                }
            }
        }
        "payment_intent.succeeded" => {
            if order_id_from_object(object).is_some() {
                confirm_from_intent(object, channel_code).await?;
            }
        }
        "payment_intent.payment_failed" => {
            if let Some(order_id) = order_id_from_object(object) {
                let reason = object["last_payment_error"]["message"]
                    .as_str()
                    .unwrap_or("Stripe payment failed.");
                sales::mark_payment_failed(order_id, reason).await?;
            }
        }
        "charge.dispute.created" => {
            if let Some(order_id) = resolve_dispute_order_id(object, channel_code).await {
                let currency = object["currency"]
                    .as_str()
                    .unwrap_or("eur")
                    .to_ascii_uppercase();
                sales::mark_payment_disputed(
                    order_id,
                    object["id"].as_str().unwrap_or_default(),
                    object["status"].as_str().unwrap_or("needs_response"),
                    object["reason"].as_str().unwrap_or("chargeback"),
                    stripe::major_units(object["amount"].as_i64().unwrap_or(0), &currency),
                    &currency,
                )
                .await?;
            }
        }
        "charge.dispute.closed" => {
            if let Some(order_id) = resolve_dispute_order_id(object, channel_code).await {
                let currency = object["currency"]
                    .as_str()
                    .unwrap_or("eur")
                    .to_ascii_uppercase();
                sales::mark_dispute_closed(
                    order_id,
                    object["id"].as_str().unwrap_or_default(),
                    object["status"].as_str().unwrap_or("closed"),
                    object["reason"].as_str().unwrap_or("chargeback"),
                    stripe::major_units(object["amount"].as_i64().unwrap_or(0), &currency),
                    &currency,
                )
                .await?;
            }
        }
        "payment_intent.canceled" => {
            if let Some(order_id) = order_id_from_object(object) {
                sales::mark_payment_failed(order_id, "Stripe cancelled the payment.").await?;
            }
        }
        "charge.refund.updated" | "refund.updated" => {
            if let Some(refund_id) = object["id"].as_str().filter(|id| !id.is_empty()) {
                sales::update_refund_state(
                    refund_id,
                    object["status"].as_str().unwrap_or("pending"),
                )
                .await?;
            }
        }
        _ => {}
    }
    Ok(())
}

impl StripeController {
    /// `GET /api/checkout/onepage/stripe/cancel/{order_id}` - returns the
    /// customer to the checkout with the cart restored.
    pub async fn cancel(Path(order_id): Path<i64>) -> Response {
        let Some(order) = db::find_by_id("orders", order_id).await.ok().flatten() else {
            return message(StatusCode::NOT_FOUND, "Order not found.");
        };
        let payment = payment_data(order_id).await;
        let origin = payment["checkout_origin"]
            .as_str()
            .map(|value| value.trim_end_matches('/'))
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(app_url);
        let status = order["payment_status"].as_str().unwrap_or("");

        if matches!(status, "paid" | "partially_refunded" | "refunded") {
            return Redirect::to(&format!("{origin}/checkout/success/{order_id}")).into_response();
        }

        if status.is_empty() || matches!(status, "pending" | "failed") {
            // Cierra la sesión de Stripe para que no pueda cobrarse después.
            if let Some(session_id) = payment["checkout_session_id"].as_str() {
                let config =
                    stripe::load(order["channel_name"].as_str().unwrap_or("default")).await;
                if !config.secret_key.is_empty() {
                    let _ = stripe::expire_checkout_session(&config, session_id).await;
                }
            }
            let _ = sales::cancel_order(order_id).await;
            let _ =
                sales::mark_payment_failed(order_id, "Payment canceled by the customer.").await;
            reactivate_cart(&order).await;
        }
        Redirect::to(&format!("{origin}/checkout?canceled=1")).into_response()
    }

    /// `GET /api/checkout/onepage/stripe/verify/{order_id}?session_id=...`
    pub async fn verify(
        Path(order_id): Path<i64>,
        Query(query): Query<HashMap<String, String>>,
    ) -> Response {
        let Some(session_id) = query
            .get("session_id")
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return message(StatusCode::BAD_REQUEST, "Missing Stripe session id.");
        };
        let Some(order) = db::find_by_id("orders", order_id).await.ok().flatten() else {
            return message(StatusCode::NOT_FOUND, "Order not found.");
        };
        let config = stripe::load(order["channel_name"].as_str().unwrap_or("default")).await;
        if config.secret_key.is_empty() {
            return message(StatusCode::INTERNAL_SERVER_ERROR, "Stripe is not configured.");
        }
        let session = match stripe::retrieve_checkout_session(&config, session_id).await {
            Ok(session) => session,
            Err(error) => return error.into(),
        };
        if order_id_from_object(&session) != Some(order_id) {
            return message(
                StatusCode::FORBIDDEN,
                "The Stripe session does not belong to this order.",
            );
        }
        if session["payment_status"].as_str() == Some("paid") {
            if let Err(error) = confirm_from_session(
                &session,
                order["channel_name"].as_str().unwrap_or("default"),
            )
            .await
            {
                return error.into();
            }
        }
        match sales::order_resource(order_id).await {
            Ok(order) => resource(order, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/webhooks/stripe` - signature checked before processing.
    pub async fn webhook(headers: HeaderMap, body: Bytes) -> Response {
        let signature = headers
            .get("stripe-signature")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        let payload: Value = match larust_support::serde_json::from_slice(&body) {
            Ok(payload) => payload,
            Err(_) => return message(StatusCode::BAD_REQUEST, "Invalid Stripe payload."),
        };
        let event_id = payload["id"].as_str().unwrap_or_default();
        let event_type = payload["type"].as_str().unwrap_or_default();
        if event_id.is_empty() || event_type.is_empty() {
            return message(StatusCode::BAD_REQUEST, "Invalid Stripe event.");
        }

        let object = &payload["data"]["object"];
        let channel_code = object["metadata"]["channel_code"]
            .as_str()
            .unwrap_or("default");
        let secret = stripe::webhook_secret(Some(channel_code)).await;
        if secret.is_empty() {
            return message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Stripe webhook secret is not configured.",
            );
        }
        if !stripe::verify_signature(&body, signature, &secret) {
            return message(StatusCode::BAD_REQUEST, "Invalid Stripe signature.");
        }

        let duplicate = db::select_one(
            "SELECT `id` FROM `payment_webhook_events` WHERE `event_id` = ? LIMIT 1",
            &[Value::String(event_id.to_string())],
        )
        .await
        .ok()
        .flatten()
        .is_some();
        if duplicate {
            return json_ok(json!({"received": true, "duplicate": true}));
        }

        let event_row = match db::insert(
            "payment_webhook_events",
            &json!({
                "gateway": "stripe",
                "event_id": event_id,
                "type": event_type,
                "status": "processing",
                "payload": payload.clone(),
                "created_at": now(),
                "updated_at": now(),
            }),
        )
        .await
        {
            Ok(id) => id,
            Err(error) => return error.into(),
        };

        match process_event(&payload, channel_code).await {
            Ok(()) => {
                let _ = db::update(
                    "payment_webhook_events",
                    event_row,
                    &json!({
                        "status": "processed",
                        "processed_at": now(),
                        "updated_at": now(),
                    }),
                )
                .await;
                json_ok(json!({"received": true}))
            }
            Err(error) => {
                let _ = db::update(
                    "payment_webhook_events",
                    event_row,
                    &json!({
                        "status": "failed",
                        "error_message": error.message,
                        "updated_at": now(),
                    }),
                )
                .await;
                message(StatusCode::INTERNAL_SERVER_ERROR, error.message)
            }
        }
    }
}
