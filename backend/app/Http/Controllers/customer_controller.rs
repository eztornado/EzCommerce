//! Customer account endpoints: addresses, wishlist, compare items.
//! Mirrors `API\AddressController`, `API\WishlistController`,
//! `API\CompareController`.

use std::collections::HashMap;

use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::{HeaderMap, StatusCode};
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Map, Value};

use crate::support::auth;
use crate::support::cart;
use crate::support::db;
use crate::support::products;
use crate::support::response::{message, resource, validation_error};

pub struct CustomerController;

fn now() -> String {
    db::now_db()
}

async fn address_fields(body: &Value) -> Vec<(String, String)> {
    let mut errors = Vec::new();
    for field in ["first_name", "last_name", "address", "country", "city", "postcode", "phone"] {
        if body[field].as_str().map(str::trim).unwrap_or_default().is_empty() {
            errors.push((field.to_string(), format!("The {field} field is required.")));
        }
    }
    errors
}

fn address_row(body: &Value, customer_id: i64, kind: &str) -> Map<String, Value> {
    let mut row = body.as_object().cloned().unwrap_or_default();
    row.insert("address_type".into(), json!(kind));
    row.insert("customer_id".into(), json!(customer_id));
    row.insert("address1".into(), body["address"].clone());
    row.insert("created_at".into(), json!(now()));
    row.insert("updated_at".into(), json!(now()));
    row
}

impl CustomerController {
    /// `GET /api/customer/addresses`.
    pub async fn addresses(customer: auth::Customer) -> Response {
        let rows = db::select(
            "SELECT * FROM `addresses` WHERE `customer_id` = ? AND `address_type` = 'customer_address' ORDER BY `id` DESC",
            &[Value::from(customer.id)],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `POST /api/customer/addresses`.
    pub async fn store_address(customer: auth::Customer, Json(body): Json<Value>) -> Response {
        let errors = address_fields(&body).await;
        if !errors.is_empty() {
            return validation_error(errors);
        }
        match db::insert("addresses", &address_row(&body, customer.id, "customer_address")).await {
            Ok(id) => {
                let address = db::find_by_id("addresses", id).await.ok().flatten();
                resource(address.map(Value::Object).unwrap_or(Value::Null), "The address has been created successfully.")
            }
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/customer/addresses/edit/{id}`.
    pub async fn update_address(
        customer: auth::Customer,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let existing = db::select_one(
            "SELECT `id` FROM `addresses` WHERE `id` = ? AND `customer_id` = ? LIMIT 1",
            &[Value::from(id), Value::from(customer.id)],
        )
        .await
        .ok()
        .flatten();
        if existing.is_none() {
            return message(StatusCode::NOT_FOUND, "Address not found.");
        }
        let errors = address_fields(&body).await;
        if !errors.is_empty() {
            return validation_error(errors);
        }
        let mut row = body.as_object().cloned().unwrap_or_default();
        row.insert("address1".into(), body["address"].clone());
        row.insert("updated_at".into(), json!(now()));
        match db::update("addresses", id, &row).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The address has been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `PATCH /api/customer/addresses/default/{id}` (extension).
    pub async fn make_default(customer: auth::Customer, Path(id): Path<i64>) -> Response {
        let _ = db::execute(
            "UPDATE `addresses` SET `default_address` = 0 WHERE `customer_id` = ? AND `address_type` = 'customer_address'",
            &[Value::from(customer.id)],
        )
        .await;
        let _ = db::execute(
            "UPDATE `addresses` SET `default_address` = 1 WHERE `id` = ? AND `customer_id` = ?",
            &[Value::from(id), Value::from(customer.id)],
        )
        .await;
        resource(Value::Array(Vec::new()), "The default address has been updated.")
    }

    /// `DELETE /api/customer/addresses/{id}`.
    pub async fn delete_address(customer: auth::Customer, Path(id): Path<i64>) -> Response {
        match db::execute(
            "DELETE FROM `addresses` WHERE `id` = ? AND `customer_id` = ?",
            &[Value::from(id), Value::from(customer.id)],
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The address has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/customer/wishlist` - paginated product cards.
    pub async fn wishlist(customer: auth::Customer, Query(query): Query<HashMap<String, String>>) -> Response {
        let page = query.get("page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(1).max(1);
        let per_page = query.get("per_page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(12).clamp(1, 100);
        let total = db::scalar_i64(
            "SELECT COUNT(*) FROM `wishlist_items` WHERE `customer_id` = ?",
            &[Value::from(customer.id)],
        )
        .await
        .unwrap_or(0);
        let offset = (page - 1) * per_page;
        let rows = db::select(
            "SELECT `product_id` FROM `wishlist_items` WHERE `customer_id` = ? ORDER BY `id` DESC LIMIT ? OFFSET ?",
            &[Value::from(customer.id), Value::from(per_page), Value::from(offset)],
        )
        .await
        .unwrap_or_default();
        let mut cards = Vec::new();
        for row in rows {
            if let Some(product_id) = row["product_id"].as_i64() {
                if let Some(card) = products::card(product_id).await {
                    cards.push(card);
                }
            }
        }
        resource(
            json!({
                "data": cards,
                "meta": {
                    "current_page": page,
                    "last_page": ((total as f64) / (per_page as f64)).ceil() as i64,
                    "per_page": per_page,
                    "total": total,
                }
            }),
            Value::Null,
        )
    }

    /// `POST /api/customer/wishlist` - `{product_id}` (toggles off if present).
    pub async fn add_to_wishlist(customer: auth::Customer, Json(body): Json<Value>) -> Response {
        let Some(product_id) = body["product_id"].as_i64() else {
            return validation_error(vec![("product_id".into(), "The product field is required.".into())]);
        };
        let existing = db::select_one(
            "SELECT `id` FROM `wishlist_items` WHERE `customer_id` = ? AND `product_id` = ? LIMIT 1",
            &[Value::from(customer.id), Value::from(product_id)],
        )
        .await
        .ok()
        .flatten();
        match existing {
            Some(row) => {
                let _ = db::delete("wishlist_items", row["id"].as_i64().unwrap_or(0)).await;
                resource(json!({"in_wishlist": false}), "The item has been removed from your wishlist.")
            }
            None => {
                let _ = db::insert(
                    "wishlist_items",
                    &json!({
                        "product_id": product_id,
                        "customer_id": customer.id,
                        "created_at": now(),
                        "updated_at": now(),
                    })
                    ,
                )
                .await;
                resource(json!({"in_wishlist": true}), "The item has been added to your wishlist.")
            }
        }
    }

    /// `POST /api/customer/wishlist/{id}/move-to-cart` - wishlist item id.
    pub async fn move_to_cart(customer: auth::Customer, Path(item_id): Path<i64>) -> Response {
        let item = db::select_one(
            "SELECT * FROM `wishlist_items` WHERE `id` = ? AND `customer_id` = ? LIMIT 1",
            &[Value::from(item_id), Value::from(customer.id)],
        )
        .await
        .ok()
        .flatten();
        let Some(item) = item else {
            return message(StatusCode::NOT_FOUND, "Wishlist item not found.");
        };
        let product_id = item["product_id"].as_i64().unwrap_or(0);
        let product_type = cart::product_type(product_id).await.unwrap_or_else(|_| "simple".into());
        let attributes = cart::attribute_map(product_id).await.unwrap_or_default();
    let _ = &attributes;
        let selected = Map::new();

        // The cart of the customer must exist; reuse `cart::add_product`.
        let channel_id = item["channel_id"].as_i64().unwrap_or(1);
        let existing_cart = db::select_one(
            "SELECT * FROM `cart` WHERE `customer_id` = ? AND `is_active` = 1 ORDER BY `id` DESC LIMIT 1",
            &[Value::from(customer.id)],
        )
        .await
        .ok()
        .flatten();
        let cart_row = match existing_cart {
            Some(cart_row) => cart_row,
            None => {
                let id = db::insert(
                    "cart",
                    &json!({
                        "customer_id": customer.id,
                        "customer_email": customer.email,
                        "customer_first_name": customer.first_name,
                        "customer_last_name": customer.last_name,
                        "channel_id": channel_id,
                        "is_guest": 0,
                        "is_active": 1,
                        "created_at": now(),
                        "updated_at": now(),
                    })
                    ,
                )
                .await
                .ok();
                let created = match id {
                    Some(new_id) => db::find_by_id("cart", new_id).await.ok().flatten(),
                    None => None,
                };
                match created {
                    Some(cart_row) => cart_row,
                    None => return message(StatusCode::INTERNAL_SERVER_ERROR, "Unable to create the cart."),
                }
            }
        };

        let base_currency = db::find_by_id("channels", channel_id)
            .await
            .ok()
            .flatten()
            .map(|channel| channel["base_currency_id"].clone())
            .unwrap_or(Value::Null);
        let _ = base_currency;

        match cart::add_product(&cart_row, product_id, 1.0, &selected).await {
            Ok(_) => {
                let _ = db::delete("wishlist_items", item_id).await;
                let _ = attributes;
                let _ = product_type;
                let updated = db::find_by_id("cart", cart_row["id"].as_i64().unwrap_or(0))
                    .await
                    .ok()
                    .flatten();
                let cart_json = match updated {
                    Some(updated) => cart::cart_resource(&updated).await,
                    None => Value::Null,
                };
                resource(
                    json!({
                        "cart": cart_json,
                        "message": "The item has been moved to your cart.",
                    }),
                    Value::Null,
                )
            }
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/customer/wishlist/{id}` and `DELETE /api/customer/wishlist/all`.
    pub async fn remove_wishlist(customer: auth::Customer, Path(item_id): Path<i64>) -> Response {
        let _ = db::execute(
            "DELETE FROM `wishlist_items` WHERE (`id` = ? OR ? = 0) AND `customer_id` = ?",
            &[Value::from(item_id), Value::from(item_id), Value::from(customer.id)],
        )
        .await;
        resource(Value::Array(Vec::new()), "The wishlist has been updated.")
    }

    /// `GET /api/customer/compare-items`.
    pub async fn compare_items(customer: auth::Customer) -> Response {
        let rows = db::select(
            "SELECT `product_id` FROM `compare_items` WHERE `customer_id` = ? ORDER BY `id` DESC",
            &[Value::from(customer.id)],
        )
        .await
        .unwrap_or_default();
        let mut cards = Vec::new();
        for row in rows {
            if let Some(product_id) = row["product_id"].as_i64() {
                if let Some(card) = products::card(product_id).await {
                    cards.push(card);
                }
            }
        }
        resource(json!(cards), Value::Null)
    }

    /// `POST /api/customer/compare-items` - `{product_id}` (toggles off if present).
    pub async fn add_to_compare(customer: auth::Customer, Json(body): Json<Value>) -> Response {
        let Some(product_id) = body["product_id"].as_i64() else {
            return validation_error(vec![("product_id".into(), "The product field is required.".into())]);
        };
        let existing = db::select_one(
            "SELECT `id` FROM `compare_items` WHERE `customer_id` = ? AND `product_id` = ? LIMIT 1",
            &[Value::from(customer.id), Value::from(product_id)],
        )
        .await
        .ok()
        .flatten();
        match existing {
            Some(row) => {
                let _ = db::delete("compare_items", row["id"].as_i64().unwrap_or(0)).await;
                resource(json!({"in_compare": false}), "The item has been removed from the compare list.")
            }
            None => {
                let _ = db::insert(
                    "compare_items",
                    &json!({
                        "product_id": product_id,
                        "customer_id": customer.id,
                        "created_at": now(),
                        "updated_at": now(),
                    })
                    ,
                )
                .await;
                resource(json!({"in_compare": true}), "The item has been added to the compare list.")
            }
        }
    }

    /// `DELETE /api/customer/compare-items` (`?product_id`), `/all` clears.
    pub async fn remove_compare(customer: auth::Customer, Query(query): Query<HashMap<String, String>>) -> Response {
        match query.get("product_id").and_then(|p| p.parse::<i64>().ok()) {
            Some(product_id) => {
                let _ = db::execute(
                    "DELETE FROM `compare_items` WHERE `customer_id` = ? AND `product_id` = ?",
                    &[Value::from(customer.id), Value::from(product_id)],
                )
                .await;
            }
            None => {
                let _ = db::execute(
                    "DELETE FROM `compare_items` WHERE `customer_id` = ?",
                    &[Value::from(customer.id)],
                )
                .await;
            }
        }
        resource(Value::Array(Vec::new()), "The compare list has been updated.")
    }

    /// `GET /api/customer/orders` - the customer's orders.
    pub async fn orders(customer: auth::Customer, Query(query): Query<HashMap<String, String>>) -> Response {
        let page = query.get("page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(1).max(1);
        let per_page = query.get("per_page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(10).clamp(1, 100);
        let total = db::scalar_i64(
            "SELECT COUNT(*) FROM `orders` WHERE `customer_id` = ?",
            &[Value::from(customer.id)],
        )
        .await
        .unwrap_or(0);
        let offset = (page - 1) * per_page;
        let rows = db::select(
            "SELECT * FROM `orders` WHERE `customer_id` = ? ORDER BY `id` DESC LIMIT ? OFFSET ?",
            &[Value::from(customer.id), Value::from(per_page), Value::from(offset)],
        )
        .await
        .unwrap_or_default();
        resource(
            json!({
                "data": rows,
                "meta": {
                    "current_page": page,
                    "last_page": ((total as f64) / (per_page as f64)).ceil() as i64,
                    "per_page": per_page,
                    "total": total,
                }
            }),
            Value::Null,
        )
    }

    /// `GET /api/customer/orders/{id}` - order detail owned by the customer.
    pub async fn order_detail(customer: auth::Customer, Path(id): Path<i64>) -> Response {
        let row = db::select_one(
            "SELECT `id` FROM `orders` WHERE `id` = ? AND `customer_id` = ? LIMIT 1",
            &[Value::from(id), Value::from(customer.id)],
        )
        .await
        .ok()
        .flatten();
        if row.is_none() {
            return message(StatusCode::NOT_FOUND, "Order not found.");
        }
        match crate::support::sales::order_resource(id).await {
            Ok(order) => resource(order, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/customer/orders/{id}/cancel`.
    pub async fn cancel_order(customer: auth::Customer, Path(id): Path<i64>) -> Response {
        let row = db::select_one(
            "SELECT `id` FROM `orders` WHERE `id` = ? AND `customer_id` = ? LIMIT 1",
            &[Value::from(id), Value::from(customer.id)],
        )
        .await
        .ok()
        .flatten();
        if row.is_none() {
            return message(StatusCode::NOT_FOUND, "Order not found.");
        }
        match crate::support::sales::cancel_order(id).await {
            Ok(()) => resource(Value::Array(Vec::new()), "The order has been canceled."),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/customer/orders/{id}/reorder` - re-adds the items to the cart.
    pub async fn reorder(customer: auth::Customer, Path(id): Path<i64>) -> Response {
        let order = db::select_one(
            "SELECT * FROM `orders` WHERE `id` = ? AND `customer_id` = ? LIMIT 1",
            &[Value::from(id), Value::from(customer.id)],
        )
        .await
        .ok()
        .flatten();
        let Some(order) = order else {
            return message(StatusCode::NOT_FOUND, "Order not found.");
        };
        let items = db::select(
            "SELECT * FROM `order_items` WHERE `order_id` = ? AND `parent_id` IS NULL",
            &[order["id"].clone()],
        )
        .await
        .unwrap_or_default();

        // Ensure an active cart.
        let existing = db::select_one(
            "SELECT * FROM `cart` WHERE `customer_id` = ? AND `is_active` = 1 ORDER BY `id` DESC LIMIT 1",
            &[Value::from(customer.id)],
        )
        .await
        .ok()
        .flatten();
        let cart_id = match existing {
            Some(cart) => cart["id"].as_i64().unwrap_or(0),
            None => db::insert(
                "cart",
                &json!({
                    "customer_id": customer.id,
                    "customer_email": customer.email,
                    "customer_first_name": customer.first_name,
                    "customer_last_name": customer.last_name,
                    "is_guest": 0,
                    "is_active": 1,
                    "created_at": now(),
                    "updated_at": now(),
                })
                ,
            )
            .await
            .unwrap_or(0),
        };
        let cart = db::find_by_id("cart", cart_id).await.ok().flatten().unwrap_or_else(db::empty_row);

        let mut added = 0;
        for item in items {
            let product_id = item["product_id"].as_i64().unwrap_or(0);
            let quantity = item["qty_ordered"].as_i64().unwrap_or(1) as f64;
            if product_id > 0
                && cart::add_product(&cart, product_id, quantity, &Map::new())
                    .await
                    .is_ok()
            {
                added += 1;
            }
        }
        if added == 0 {
            return message(StatusCode::UNPROCESSABLE_ENTITY, "No items could be added to the cart.");
        }
        resource(Value::Array(Vec::new()), "The items have been added to your cart.")
    }
}
