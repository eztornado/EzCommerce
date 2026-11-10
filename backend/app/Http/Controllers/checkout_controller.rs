//! One-page checkout (`/api/checkout/onepage/...`). Mirrors
//! `Webkul\Shop\Http\Controllers\API\OnepageController`.

use std::collections::HashMap;

use larust_support::axum::extract::Query;
use larust_support::axum::http::{HeaderMap, StatusCode};
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::cart;
use crate::support::core;
use crate::support::db;
use crate::support::response::{message, resource, validation_error};
use crate::support::sales;

pub struct CheckoutController;

async fn active_cart(
    headers: &HeaderMap,
    query: &HashMap<String, String>,
) -> Result<larust_support::serde_json::Map<String, Value>, Response> {
    let context = cart::cart_context(headers);
    cart::find_cart(&context)
        .await
        .map(|found| found.unwrap_or_else(db::empty_row))
        .map_err(|error| error.into())
}

impl CheckoutController {
    /// `GET /api/checkout/onepage/summary` - the cart (`CartResource`).
    pub async fn summary(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let cart = match active_cart(&headers, &query).await {
            Ok(cart) => cart,
            Err(response) => return response,
        };
        if cart.is_empty() {
            return resource(Value::Null, Value::Null);
        }
        resource(cart::cart_resource(&cart).await, Value::Null)
    }

    /// `POST /api/checkout/onepage/addresses` - saves billing (+shipping).
    pub async fn save_addresses(
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Response {
        let cart = match active_cart(&headers, &query).await {
            Ok(cart) => cart,
            Err(response) => return response,
        };
        if cart.is_empty() {
            return message(StatusCode::NOT_FOUND, "Cart not found.");
        }
        let cart_id = cart["id"].as_i64().unwrap_or(0);

        let mut errors = Vec::new();
        for field in ["first_name", "last_name", "address", "country", "city", "postcode"] {
            if body["billing"][field]
                .as_str()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
            {
                errors.push((format!("billing.{field}"), format!("The {field} field is required.")));
            }
        }
        if body["billing"]["email"].as_str().map(str::trim).unwrap_or_default().is_empty() {
            errors.push(("billing.email".into(), "The email field is required.".into()));
        }
        if !errors.is_empty() {
            return validation_error(errors);
        }

        let shipping = if body.get("shipping").map(Value::is_object).unwrap_or(false) {
            body["shipping"].clone()
        } else {
            body["billing"].clone()
        };

        // Replace previous cart addresses.
        let _ = db::execute(
            "DELETE FROM `addresses` WHERE `cart_id` = ?",
            &[Value::from(cart_id)],
        )
        .await;

        for (kind, payload) in [("cart_billing", &body["billing"]), ("cart_shipping", &shipping)] {
            let address = payload.as_object().cloned().unwrap_or_default();
            if address.is_empty() {
                continue;
            }
            let mut row = address.clone();
            row.insert("address_type".into(), json!(kind));
            row.insert("cart_id".into(), json!(cart_id));
            if let Some(customer_id) = cart["customer_id"].as_i64() {
                row.insert("customer_id".into(), json!(customer_id));
            }
            row.insert("address1".into(), json!(address.get("address").cloned().unwrap_or(Value::Null)));
            row.insert("created_at".into(), json!(db::now_db()));
            row.insert("updated_at".into(), json!(db::now_db()));
            if let Err(error) = db::insert("addresses", &row).await {
                return error.into();
            }
        }

        // Guest email on the cart.
        if cart["customer_id"].is_null() {
            let _ = db::execute(
                "UPDATE `cart` SET `customer_email` = ?, `customer_first_name` = ?, `customer_last_name` = ?, `updated_at` = ? WHERE `id` = ?",
                &[
                    body["billing"]["email"].clone(),
                    body["billing"]["first_name"].clone(),
                    body["billing"]["last_name"].clone(),
                    Value::String(db::now_db()),
                    Value::from(cart_id),
                ],
            )
            .await;
        }

        let updated = db::find_by_id("cart", cart_id).await.ok().flatten();
        resource(
            updated
                .map(|cart| json!({"cart": cart}))
                .unwrap_or(Value::Null),
            "The addresses have been saved.",
        )
    }

    /// `POST /api/checkout/onepage/shipping-methods` - persists the chosen
    /// method and returns the available payment methods.
    pub async fn save_shipping_method(
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Response {
        let cart = match active_cart(&headers, &query).await {
            Ok(cart) => cart,
            Err(response) => return response,
        };
        if cart.is_empty() {
            return message(StatusCode::NOT_FOUND, "Cart not found.");
        }
        let Some(method) = body["shipping_method"]
            .as_str()
            .or_else(|| body["method"].as_str())
            .map(str::to_string)
        else {
            return validation_error(vec![(
                "shipping_method".into(),
                "The shipping method field is required.".into(),
            )]);
        };

        let channel = core::current_channel(&headers).await.ok();
        let channel_code = channel
            .as_ref()
            .and_then(|channel| channel["code"].as_str())
            .unwrap_or("default")
            .to_string();

        match cart::save_shipping_method(cart["id"].as_i64().unwrap_or(0), &method).await {
            Ok(()) => {
                let methods = cart::supported_payment_methods(&channel_code).await;
                resource(json!({"payment_methods": methods}), Value::Null)
            }
            Err(error) => error.into(),
        }
    }

    /// `GET /api/checkout/onepage/shipping-methods` - available rates for the
    /// saved billing address (extension used by the Flutter checkout).
    pub async fn shipping_methods(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let cart = match active_cart(&headers, &query).await {
            Ok(cart) => cart,
            Err(response) => return response,
        };
        if cart.is_empty() {
            return resource(json!({"shippingMethods": []}), Value::Null);
        }
        let address = db::select_one(
            "SELECT * FROM `addresses` WHERE `cart_id` = ? AND `address_type` = 'cart_shipping' LIMIT 1",
            &[cart["id"].clone()],
        )
        .await
        .ok()
        .flatten();
        let country = address
            .as_ref()
            .and_then(|address| address["country"].as_str())
            .unwrap_or_default()
            .to_string();
        match cart::collect_shipping_rates(cart["id"].as_i64().unwrap_or(0), &country).await {
            Ok(rates) => resource(json!({"shippingMethods": rates}), Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/checkout/onepage/payment-methods` - supported methods.
    pub async fn payment_methods(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let channel = match core::current_channel(&headers).await {
            Ok(channel) => channel,
            Err(error) => return error.into(),
        };
        let channel_code = channel["code"].as_str().unwrap_or("default").to_string();
        let methods = cart::supported_payment_methods(&channel_code).await;
        resource(json!({"payment_methods": methods}), Value::Null)
    }

    /// `POST /api/checkout/onepage/payment-methods` - persists the method.
    pub async fn save_payment_method(
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Response {
        let cart = match active_cart(&headers, &query).await {
            Ok(cart) => cart,
            Err(response) => return response,
        };
        if cart.is_empty() {
            return message(StatusCode::NOT_FOUND, "Cart not found.");
        }
        let Some(method) = body["payment_method"]
            .as_str()
            .or_else(|| body["method"].as_str())
            .map(str::to_string)
        else {
            return validation_error(vec![(
                "payment_method".into(),
                "The payment method field is required.".into(),
            )]);
        };
        match cart::save_payment_method(cart["id"].as_i64().unwrap_or(0), &method).await {
            Ok(()) => resource(Value::Array(Vec::new()), "The payment method has been saved."),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/checkout/onepage/orders` - creates the order.
    /// Offline methods complete immediately (`{redirect: false, order: ...}`);
    /// redirect-based gateways land in phase 3.
    pub async fn store_order(
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = body;
        let cart = match active_cart(&headers, &query).await {
            Ok(cart) => cart,
            Err(response) => return response,
        };
        if cart.is_empty() {
            return message(StatusCode::NOT_FOUND, "Cart not found.");
        }
        match sales::create_order_from_cart(cart["id"].as_i64().unwrap_or(0)).await {
            Ok(order) => resource(
                json!({
                    "redirect": false,
                    "order": order,
                    "order_id": order["id"],
                    "increment_id": order["increment_id"],
                }),
                "The order has been placed successfully.",
            ),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/checkout/onepage/success/{orderId}` - order payload for the
    /// success screen (owner-checked for customers).
    pub async fn success(
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
        Path(order_id): Path<i64>,
    ) -> Response {
        let _ = query;
        let order = match db::find_by_id("orders", order_id).await {
            Ok(Some(order)) => order,
            _ => return message(StatusCode::NOT_FOUND, "Order not found."),
        };
        // Guests may read their own order for the success screen only if they
        // still hold the cart context; keep it simple: customers restricted
        // to their own orders.
        if let Some(claims) = auth::bearer_token(&headers)
            .and_then(|token| auth::decode_token(&token, false).ok())
        {
            if claims.prv == "customer" {
                let customer_id = claims.sub.parse::<i64>().unwrap_or(0);
                if order["customer_id"].as_i64() != Some(customer_id) {
                    return message(StatusCode::FORBIDDEN, "Not allowed.");
                }
            }
        }
        resource(order, Value::Null)
    }
}

use larust_support::axum::extract::Path;
