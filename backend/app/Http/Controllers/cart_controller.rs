//! Shop cart endpoints (`/api/checkout/cart/...`). Mirrors
//! `Webkul\Shop\Http\Controllers\API\CartController`.

use std::collections::HashMap;

use larust_support::axum::extract::Query;
use larust_support::axum::http::{HeaderMap, StatusCode};
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::cart;
use crate::support::cart_rules;
use crate::support::core;
use crate::support::db;
use crate::support::products;
use crate::support::response::{message, resource, unauthorized, validation_error};

pub struct CartController;

/// Resolves the cart context, creating the cart on demand. Returns
/// `(cart, cart_token)`; the token is only present for guests. On failure the
/// cart comes back empty and the caller answers 404/500.
async fn with_cart(
    headers: &HeaderMap,
    query: &HashMap<String, String>,
    create: bool,
) -> (larust_support::serde_json::Map<String, Value>, Option<String>) {
    let context = cart::cart_context(headers);
    let Ok(channel) = core::current_channel(headers).await else {
        return (db::empty_row(), None);
    };
    let Ok(currency) = core::resolve_currency(&channel, query.get("currency").map(String::as_str)).await else {
        return (db::empty_row(), None);
    };
    let Ok(base) = core::base_currency(&channel).await else {
        return (db::empty_row(), None);
    };

    if create {
        let created = cart::get_or_create_cart(
            &context,
            channel["id"].as_i64().unwrap_or(1),
            base["code"].as_str().unwrap_or("EUR"),
            currency["code"].as_str().unwrap_or("EUR"),
        )
        .await;
        let Ok(created) = created else {
            return (db::empty_row(), None);
        };
        let token = if context.customer_id.is_none() {
            cart::cart_token_for(created["id"].as_i64().unwrap_or(0)).ok()
        } else {
            None
        };
        (created, token)
    } else {
        let existing = cart::find_cart(&context).await.ok().flatten();
        (existing.unwrap_or_else(db::empty_row), None)
    }
}

impl CartController {
    /// `GET /api/checkout/cart` - current cart or `{data: null}`.
    pub async fn get(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let (existing, _) = with_cart(&headers, &query, false).await;
        if existing.is_empty() {
            return resource(Value::Null, Value::Null);
        }
        resource(cart::cart_resource(&existing).await, Value::Null)
    }

    /// `POST /api/checkout/cart` - add a product
    /// `{product_id, quantity, [options...]}`.
    pub async fn add(
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Response {
        let Some(product_id) = body["product_id"].as_i64() else {
            return validation_error(vec![(
                "product_id".into(),
                "The product field is required.".into(),
            )]);
        };
        let quantity = body["quantity"].as_f64().unwrap_or(1.0).max(1.0);

        let (cart, token) = with_cart(&headers, &query, true).await;
        if cart.is_empty() {
            return message(StatusCode::INTERNAL_SERVER_ERROR, "Unable to create the cart.");
        }

        let options = body.as_object().cloned().unwrap_or_default();
        match cart::add_product(&cart, product_id, quantity, &options).await {
            Ok(_) => {
                // Automatic rules (coupon-less) reapply after cart changes.
                let _ = cart_rules::apply_cart_rules(cart["id"].as_i64().unwrap_or(0)).await;
                let refreshed = cart::find_cart(&cart::cart_context(&headers)).await.ok().flatten();
                let item = refreshed
                    .as_ref()
                    .and_then(|cart| cart["id"].as_i64())
                    .map(|cart_id| json!({"cart_id": cart_id}))
                    .unwrap_or_else(|| json!({}));
                let mut payload = json!({
                    "message": "The item has been added to your cart successfully.",
                    "item": item,
                });
                if let Some(token) = token {
                    payload["cart_token"] = json!(token);
                }
                resource(payload, Value::Null)
            }
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/checkout/cart` - update quantities `{qty: [{cart_item_id, quantity}]}`.
    pub async fn update(
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Response {
        let (cart, _) = with_cart(&headers, &query, false).await;
        if cart.is_empty() {
            return message(StatusCode::NOT_FOUND, "Cart not found.");
        }
        let items = body["qty"]
            .as_array()
            .cloned()
            .or_else(|| body["items"].as_array().cloned())
            .unwrap_or_default();
        match cart::update_items(&cart, &items).await {
            Ok(()) => resource(cart::cart_resource(&cart).await, "The cart has been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/checkout/cart` - remove all items.
    pub async fn destroy(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let (cart, _) = with_cart(&headers, &query, false).await;
        if cart.is_empty() {
            return message(StatusCode::NOT_FOUND, "Cart not found.");
        }
        let cart_id = cart["id"].as_i64().unwrap_or(0);
        let _ = db::execute(
            "DELETE FROM `cart_items` WHERE `cart_id` = ?",
            &[Value::from(cart_id)],
        )
        .await;
        match cart::collect_totals(cart_id).await {
            Ok(_) => resource(cart::cart_resource(&cart).await, "The cart has been cleared."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/checkout/cart/selected` - remove `{item_ids: []}`.
    pub async fn remove_selected(
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Response {
        let (cart, _) = with_cart(&headers, &query, false).await;
        if cart.is_empty() {
            return message(StatusCode::NOT_FOUND, "Cart not found.");
        }
        let cart_id = cart["id"].as_i64().unwrap_or(0);
        for item_id in body["item_ids"].as_array().cloned().unwrap_or_default() {
            let _ = db::execute(
                "DELETE FROM `cart_items` WHERE `id` = ? AND `cart_id` = ?",
                &[item_id, Value::from(cart_id)],
            )
            .await;
        }
        let _ = cart::collect_totals(cart_id).await;
        resource(cart::cart_resource(&cart).await, "The selected items have been removed.")
    }

    /// `POST /api/checkout/cart/coupon` - `{code}` (coupon validation lands
    /// with cart rules in phase 2; the code is stored for now).
    pub async fn apply_coupon(
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Response {
        let (cart, _) = with_cart(&headers, &query, false).await;
        if cart.is_empty() {
            return message(StatusCode::NOT_FOUND, "Cart not found.");
        }
        let Some(code) = body["code"].as_str().map(str::trim).filter(|c| !c.is_empty()) else {
            return validation_error(vec![("code".into(), "The coupon code field is required.".into())]);
        };
        let exists = db::select_one(
            "SELECT `crc`.`id` FROM `cart_rule_coupons` `crc` \
             INNER JOIN `cart_rules` `r` ON `r`.`id` = `crc`.`cart_rule_id` \
             WHERE `crc`.`code` = ? AND `r`.`status` = 1 LIMIT 1",
            &[Value::String(code.to_string())],
        )
        .await
        .ok()
        .flatten();
        if exists.is_none() {
            return message(StatusCode::BAD_REQUEST, "The coupon code is invalid.");
        }
        let _ = db::execute(
            "UPDATE `cart` SET `coupon_code` = ?, `updated_at` = ? WHERE `id` = ?",
            &[
                Value::String(code.to_string()),
                Value::String(db::now_db()),
                cart["id"].clone(),
            ],
        )
        .await;
        let updated = crate::support::cart_rules::apply_cart_rules(cart["id"].as_i64().unwrap_or(0)).await;
        if let Ok(updated) = updated {
            let _ = updated;
        }
        resource(Value::Array(Vec::new()), "The coupon code has been applied successfully.")
    }

    /// `DELETE /api/checkout/cart/coupon`.
    pub async fn remove_coupon(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let (cart, _) = with_cart(&headers, &query, false).await;
        if cart.is_empty() {
            return message(StatusCode::NOT_FOUND, "Cart not found.");
        }
        let _ = db::execute(
            "UPDATE `cart` SET `coupon_code` = NULL, `updated_at` = ? WHERE `id` = ?",
            &[Value::String(db::now_db()), cart["id"].clone()],
        )
        .await;
        let _ = crate::support::cart_rules::apply_cart_rules(cart["id"].as_i64().unwrap_or(0)).await;
        resource(Value::Array(Vec::new()), "The coupon code has been removed successfully.")
    }

    /// `POST /api/checkout/cart/estimate-shipping-methods` - `{country, state, postcode}`.
    pub async fn estimate_shipping(
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Response {
        let (cart, _) = with_cart(&headers, &query, false).await;
        if cart.is_empty() {
            return message(StatusCode::NOT_FOUND, "Cart not found.");
        }
        let country = body["country"].as_str().unwrap_or_default().to_string();
        match cart::collect_shipping_rates(cart["id"].as_i64().unwrap_or(0), &country).await {
            Ok(rates) => resource(json!({"shippingMethods": rates}), Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/checkout/cart/cross-sell` - cross-sell products of the cart items.
    pub async fn cross_sell(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let (cart, _) = with_cart(&headers, &query, false).await;
        if cart.is_empty() {
            return resource(json!([]), Value::Null);
        }
        let products = db::select(
            "SELECT DISTINCT `p`.`id` FROM `product_cross_sells` `pc` \
             INNER JOIN `cart_items` `ci` ON `ci`.`product_id` = `pc`.`parent_id` \
             INNER JOIN `products` `p` ON `p`.`id` = `pc`.`child_id` \
             WHERE `ci`.`cart_id` = ? LIMIT 12",
            &[cart["id"].clone()],
        )
        .await
        .unwrap_or_default();
        let mut cards = Vec::new();
        for row in products {
            if let Some(id) = row["id"].as_i64() {
                if let Some(card) = products::card(id).await {
                    cards.push(card);
                }
            }
        }
        resource(json!(cards), Value::Null)
    }

    /// `POST /api/checkout/cart/move-to-wishlist` - `{item_ids: []}` (logged in).
    pub async fn move_to_wishlist(
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Response {
        let claims = match auth::bearer_token(&headers)
            .and_then(|token| auth::decode_token(&token, false).ok())
        {
            Some(claims) if claims.prv == "customer" => claims,
            _ => return unauthorized(),
        };
        let customer_id: i64 = match claims.sub.parse() {
            Ok(id) => id,
            Err(_) => return unauthorized(),
        };

        let (cart, _) = with_cart(&headers, &query, false).await;
        if cart.is_empty() {
            return message(StatusCode::NOT_FOUND, "Cart not found.");
        }
        let channel_id = cart["channel_id"].as_i64().unwrap_or(1);
        let item_ids = body["item_ids"].as_array().cloned().unwrap_or_default();
        let mut moved = 0;
        for item_id in item_ids {
            let item = db::find_by_id("cart_items", item_id.as_i64().unwrap_or(0))
                .await
                .ok()
                .flatten();
            let Some(item) = item else { continue };
            if item["cart_id"].as_i64() != cart["id"].as_i64() {
                continue;
            }
            let _ = db::insert(
                "wishlist_items",
                &json!({
                    "channel_id": channel_id,
                    "product_id": item["product_id"].clone(),
                    "customer_id": customer_id,
                    "created_at": db::now_db(),
                    "updated_at": db::now_db(),
                })
                ,
            )
            .await;
            let _ = db::delete("cart_items", item["id"].as_i64().unwrap_or(0)).await;
            moved += 1;
        }
        let _ = cart::collect_totals(cart["id"].as_i64().unwrap_or(0)).await;
        if moved == 0 {
            return message(StatusCode::UNPROCESSABLE_ENTITY, "Nothing to move.");
        }
        resource(Value::Array(Vec::new()), "The selected items have been moved to the wishlist.")
    }
}
