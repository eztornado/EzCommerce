//! Cart engine: guest/registered cart resolution, item management,
//! `collectTotals()` (tax + shipping), coupon application and the
//! cart -> order transformation. Mirrors `Webkul\Checkout\Cart` and the
//! `Sales` repositories.
//!
//! Guest carts are stateless: the API issues a signed "cart token"
//! (JWT, prv=`cart`) that the Flutter client sends back in the
//! `X-Cart-Token` header, replacing Bagisto's session cookie.

use larust_support::axum::http::HeaderMap;
use larust_support::serde_json::{json, Map, Value};
use larust_support::serde_json::Value as Json;

use super::auth;
use super::core;
use super::db::{self, RowMap};
use super::response::{ApiError, ApiResult};

const CART_TTL_DAYS: i64 = 30;

fn now() -> String {
    db::now_db()
}

// ---------------------------------------------------------------------------
// Cart resolution
// ---------------------------------------------------------------------------

/// Signs a cart id into a stateless guest token.
pub fn cart_token_for(cart_id: i64) -> ApiResult<String> {
    let (token, _) = auth::issue_token("cart", cart_id)?;
    // Issue token gives the configured access TTL; carts live longer, so the
    // token stays valid by ignoring exp on decode (see `cart_from_token`).
    Ok(token)
}

fn cart_id_from_token(token: &str) -> Option<i64> {
    let claims = auth::decode_token(token, true).ok()?;
    if claims.prv != "cart" {
        return None;
    }
    claims.sub.parse::<i64>().ok()
}

pub struct CartContext {
    pub customer_id: Option<i64>,
    pub cart_token: Option<String>,
}

pub fn cart_context(headers: &HeaderMap) -> CartContext {
    let customer_id = auth::bearer_token(headers)
        .and_then(|token| auth::decode_token(&token, false).ok())
        .filter(|claims| claims.prv == "customer")
        .and_then(|claims| claims.sub.parse::<i64>().ok());
    let cart_token = headers
        .get("x-cart-token")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    CartContext {
        customer_id,
        cart_token,
    }
}

/// The active cart: by customer, else by guest token. `is_active = 1` only.
pub async fn find_cart(context: &CartContext) -> ApiResult<Option<RowMap>> {
    if let Some(customer_id) = context.customer_id {
        if let Some(cart) = db::select_one(
            "SELECT * FROM `cart` WHERE `customer_id` = ? AND `is_active` = 1 AND `is_buy_now` = 0 ORDER BY `id` DESC LIMIT 1",
            &[Value::from(customer_id)],
        )
        .await?
        {
            return Ok(Some(cart));
        }
    }
    if let Some(token) = &context.cart_token {
        if let Some(cart_id) = cart_id_from_token(token) {
            return db::select_one(
                "SELECT * FROM `cart` WHERE `id` = ? AND `is_active` = 1 LIMIT 1",
                &[Value::from(cart_id)],
            )
            .await
            .map(|cart| cart.filter(|cart| cart["customer_id"].is_null() || context.customer_id.is_some()));
        }
    }
    Ok(None)
}

/// Finds the active cart or creates one for the context.
pub async fn get_or_create_cart(
    context: &CartContext,
    channel_id: i64,
    base_currency: &str,
    currency: &str,
) -> ApiResult<RowMap> {
    if let Some(cart) = find_cart(context).await? {
        return Ok(cart);
    }

    let mut payload = json!({
        "channel_id": channel_id,
        "base_currency_code": base_currency,
        "cart_currency_code": currency,
        "global_currency_code": base_currency,
        "channel_currency_code": currency,
        "exchange_rate": 1,
        "is_guest": if context.customer_id.is_some() { 0 } else { 1 },
        "is_active": 1,
        "created_at": now(),
        "updated_at": now(),
    });
    if let Some(customer_id) = context.customer_id {
        let customer = db::find_by_id("customers", customer_id).await?.unwrap_or_else(db::empty_row);
        payload["customer_id"] = json!(customer_id);
        payload["customer_email"] = customer["email"].clone();
        payload["customer_first_name"] = customer["first_name"].clone();
        payload["customer_last_name"] = customer["last_name"].clone();
    }
    let id = db::insert("cart", &payload).await?;
    Ok(db::find_by_id("cart", id).await?.unwrap_or_else(db::empty_row))
}

// ---------------------------------------------------------------------------
// Product resolution (EAV)
// ---------------------------------------------------------------------------

/// Attribute values of a single product (channel/locale-less rows).
async fn load_attributes(product_id: i64) -> ApiResult<Vec<RowMap>> {
    db::select(
        "SELECT `a`.`code`, `a`.`type`, `v`.* FROM `product_attribute_values` `v` \
         INNER JOIN `attributes` `a` ON `a`.`id` = `v`.`attribute_id` \
         WHERE `v`.`product_id` = ? AND (`v`.`channel` IS NULL) AND (`v`.`locale` IS NULL)",
        &[Value::from(product_id)],
    )
    .await
}

/// Flattened attribute map for a product, falling back to its parent for
/// non-super attributes (configurable variants inherit price/description).
pub async fn attribute_map(product_id: i64) -> ApiResult<Value> {
    let product = db::find_by_id("products", product_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Product not found"))?;
    let parent_id = product["parent_id"].as_i64();

    let mut map = Map::new();
    for row in load_attributes(product_id).await? {
        let code = row["code"].as_str().unwrap_or_default().to_string();
        map.insert(code, core::attribute_value(row["type"].as_str().unwrap_or("text"), &row));
    }
    if let Some(parent_id) = parent_id {
        for row in load_attributes(parent_id).await? {
            let code = row["code"].as_str().unwrap_or_default().to_string();
            map.entry(code).or_insert_with(|| {
                core::attribute_value(row["type"].as_str().unwrap_or("text"), &row)
            });
        }
    }
    Ok(Value::Object(map))
}

/// Effective unit price: `special_price` when the window is active, else
/// `price` (in the channel base currency).
pub fn effective_price(attributes: &Value) -> f64 {
    let price = attributes["price"].as_f64().unwrap_or(0.0);
    let special = attributes["special_price"].as_f64();
    let Some(special) = special else {
        return price;
    };
    let from = attributes["special_price_from"].as_str().unwrap_or_default();
    let to = attributes["special_price_to"].as_str().unwrap_or_default();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let in_window = (from.is_empty() || from <= today.as_str())
        && (to.is_empty() || today.as_str() <= to);
    if in_window {
        special.min(price)
    } else {
        price
    }
}

/// Product type (`simple|virtual|configurable|grouped|bundle|downloadable|booking`).
pub async fn product_type(product_id: i64) -> ApiResult<String> {
    let product = db::find_by_id("products", product_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Product not found"))?;
    Ok(product["type"].as_str().unwrap_or("simple").to_string())
}

/// Available stock: inventory sources (status=1) minus ordered (reserved).
pub async fn available_quantity(product_id: i64, channel_id: i64) -> i64 {
    let available: i64 = db::scalar_i64(
        "SELECT COALESCE(SUM(`pi`.`qty`), 0) FROM `product_inventories` `pi` \
         INNER JOIN `inventory_sources` `isrc` ON `isrc`.`id` = `pi`.`inventory_source_id` \
         WHERE `pi`.`product_id` = ? AND `isrc`.`status` = 1 \
         AND `isrc`.`id` IN (SELECT `inventory_source_id` FROM `channel_inventory_sources` WHERE `channel_id` = ?)",
        &[Value::from(product_id), Value::from(channel_id)],
    )
    .await
    .unwrap_or(0);
    let ordered: i64 = db::scalar_i64(
        "SELECT COALESCE(SUM(`qty`), 0) FROM `product_ordered_inventories` WHERE `product_id` = ?",
        &[Value::from(product_id)],
    )
    .await
    .unwrap_or(0);
    (available - ordered).max(0)
}

fn is_stockable(product_type: &str) -> bool {
    !matches!(product_type, "downloadable" | "booking")
}

// ---------------------------------------------------------------------------
// Items
// ---------------------------------------------------------------------------

/// Adds a product to the cart. For configurable products the selected super
/// attributes must resolve to exactly one variant.
pub async fn add_product(
    cart: &RowMap,
    product_id: i64,
    quantity: f64,
    selected: &Map<String, Value>,
) -> ApiResult<RowMap> {
    let channel_id = cart["channel_id"].as_i64().unwrap_or(1);
    let parent_type = product_type(product_id).await?;

    let target_id = match parent_type.as_str() {
        "configurable" => {
            let super_attrs = db::select(
                "SELECT `a`.`id`, `a`.`code` FROM `product_super_attributes` `ps` \
                 INNER JOIN `attributes` `a` ON `a`.`id` = `ps`.`attribute_id` WHERE `ps`.`product_id` = ?",
                &[Value::from(product_id)],
            )
            .await?;
            if super_attrs.is_empty() {
                return Err(ApiError::unprocessable("This product has no options."));
            }
            // Variant = child whose EAV matches every selected option *value*
            // (one IN-subquery per super attribute).
            let mut conditions = Vec::new();
            let mut binds: Vec<Value> = vec![Value::from(product_id)];
            for attribute in &super_attrs {
                let id = attribute["id"].as_i64().unwrap_or(0);
                let code = attribute["code"].as_str().unwrap_or_default();
                let Some(value) = selected.get(code).and_then(Value::as_i64) else {
                    continue;
                };
                conditions.push(
                    "`id` IN (SELECT `product_id` FROM `product_attribute_values` \
                     WHERE `attribute_id` = ? AND `integer_value` = ?)".to_string(),
                );
                binds.push(Value::from(id));
                binds.push(Value::from(value));
            }
            if conditions.is_empty() {
                return Err(ApiError::unprocessable("Please select all the options."));
            }
            let variant = db::select_one(
                &format!(
                    "SELECT `id` FROM `products` WHERE `parent_id` = ? AND {} LIMIT 1",
                    conditions.join(" AND ")
                ),
                &binds,
            )
            .await?;
            variant
                .and_then(|row| row["id"].as_i64())
                .ok_or_else(|| ApiError::unprocessable("The selected combination is unavailable."))?
        }
        _ => product_id,
    };

    let attributes = attribute_map(target_id).await?;
    let variant_type = product_type(target_id).await?;
    let price = effective_price(&attributes);
    let name = attributes["name"]
        .as_str()
        .unwrap_or_else(|| attributes["sku"].as_str().unwrap_or_default())
        .to_string();
    let sku = attributes["sku"].as_str().unwrap_or_default().to_string();

    // Stock check for stockable products.
    if is_stockable(&variant_type) {
        let manage_stock = attributes["manage_stock"].as_bool().unwrap_or(true) || attributes["manage_stock"].as_i64().unwrap_or(1) == 1;
        if manage_stock {
            let available = available_quantity(target_id, channel_id).await;
            let already: i64 = db::scalar_i64(
                "SELECT COALESCE(SUM(`quantity`), 0) FROM `cart_items` WHERE `cart_id` = ? AND `product_id` = ?",
                &[cart["id"].clone(), Value::from(target_id)],
            )
            .await
            .unwrap_or(0);
            if (available as f64) < already as f64 + quantity {
                return Err(ApiError::unprocessable(
                    "The requested quantity is not available.",
                ));
            }
        }
    }

    // Merge with an existing line for simple products without options.
    let existing = db::select_one(
        "SELECT * FROM `cart_items` WHERE `cart_id` = ? AND `product_id` = ? AND `parent_id` IS NULL LIMIT 1",
        &[cart["id"].clone(), Value::from(target_id)],
    )
    .await?;

    let base_price = price; // cart prices are stored in base currency
    let additional = if parent_type == "configurable" {
        json!({"selected_configurable_option": selected, "product_id": product_id})
    } else if !selected.is_empty() {
        json!({"options": selected})
    } else {
        Value::Null
    };

    let item_id = match existing {
        Some(item) if additional.is_null() => {
            let quantity = item["quantity"].as_i64().unwrap_or(0) + quantity as i64;
            db::update(
                "cart_items",
                item["id"].as_i64().unwrap_or(0),
                &json!({"quantity": quantity, "updated_at": now()}),
            )
            .await?;
            item["id"].as_i64().unwrap_or(0)
        }
        _ => {
            db::insert(
                "cart_items",
                &json!({
                    "cart_id": cart["id"].clone(),
                    "product_id": target_id,
                    "sku": sku,
                    "type": variant_type,
                    "name": name,
                    "quantity": quantity as i64,
                    "price": base_price,
                    "base_price": base_price,
                    "total": base_price * quantity,
                    "base_total": base_price * quantity,
                    "weight": attributes["weight"].as_f64().unwrap_or(0.0),
                    "total_weight": attributes["weight"].as_f64().unwrap_or(0.0) * quantity,
                    "base_total_weight": attributes["weight"].as_f64().unwrap_or(0.0) * quantity,
                    "additional": additional,
                    "created_at": now(),
                    "updated_at": now(),
                })
                ,
            )
            .await?
        }
    };

    collect_totals(cart["id"].as_i64().unwrap_or(0)).await?;
    db::find_by_id("cart_items", item_id).await.map(|item| item.unwrap_or_else(db::empty_row))
}

/// Updates quantities: `items = [{cart_item_id, quantity}]`.
pub async fn update_items(cart: &RowMap, items: &[Value]) -> ApiResult<()> {
    for entry in items {
        let Some(item_id) = entry["cart_item_id"].as_i64().or(entry["item_id"].as_i64()) else {
            continue;
        };
        let item = db::find_by_id("cart_items", item_id).await?;
        let Some(item) = item else { continue };
        if item["cart_id"].as_i64() != cart["id"].as_i64() {
            continue;
        }
        let quantity = entry["quantity"].as_i64().unwrap_or(1).max(0);
        if quantity == 0 {
            db::delete("cart_items", item_id).await?;
            continue;
        }
        let price = item["base_price"].as_f64().unwrap_or(0.0);
        db::update(
            "cart_items",
            item_id,
            &json!({
                "quantity": quantity,
                "total": price * quantity as f64,
                "base_total": price * quantity as f64,
                "updated_at": now(),
            }),
        )
        .await?;
    }
    collect_totals(cart["id"].as_i64().unwrap_or(0)).await
    .map(|_| ())
}

pub async fn remove_item(cart: &RowMap, item_id: i64) -> ApiResult<()> {
    db::execute(
        "DELETE FROM `cart_items` WHERE `id` = ? AND `cart_id` = ?",
        &[Value::from(item_id), cart["id"].clone()],
    )
    .await?;
    collect_totals(cart["id"].as_i64().unwrap_or(0)).await
    .map(|_| ())
}

// ---------------------------------------------------------------------------
// Totals (collectTotals)
// ---------------------------------------------------------------------------

/// The tax rate matching an address for a tax category, mirroring
/// `Tax::isTaxApplicableInCurrentAddress` (country match, state wildcard,
/// zip exact or range, first by rate DESC).
async fn tax_rate_for(category_id: i64, address: Option<&RowMap>) -> (f64, String) {
    if category_id <= 0 {
        return (0.0, String::new());
    }
    let rates = db::select(
        "SELECT `tr`.* FROM `tax_categories_tax_rates` `ttr` \
         INNER JOIN `tax_rates` `tr` ON `tr`.`id` = `ttr`.`tax_rate_id` \
         WHERE `ttr`.`tax_category_id` = ? ORDER BY `tr`.`tax_rate` DESC",
        &[Value::from(category_id)],
    )
    .await
    .unwrap_or_default();
    let country = address
        .and_then(|address| address["country"].as_str())
        .unwrap_or_default();
    let state = address
        .and_then(|address| address["state"].as_str())
        .unwrap_or_default();
    let postcode = address
        .and_then(|address| address["postcode"].as_str())
        .unwrap_or_default();

    for rate in rates {
        if rate["country"].as_str().unwrap_or_default() != country {
            continue;
        }
        let rate_state = rate["state"].as_str().unwrap_or_default();
        if !rate_state.is_empty() && rate_state != "*" && rate_state != state {
            continue;
        }
        let matches_zip = if rate["is_zip"].as_i64().unwrap_or(0) == 1 {
            let from = rate["zip_from"].as_str().unwrap_or_default();
            let to = rate["zip_to"].as_str().unwrap_or_default();
            !postcode.is_empty()
                && postcode >= from
                && postcode <= to
        } else {
            let zip = rate["zip_code"].as_str().unwrap_or_default();
            zip.is_empty() || zip == "*" || zip == postcode
        };
        if matches_zip {
            return (
                rate["tax_rate"].as_f64().unwrap_or(0.0),
                rate["identifier"].as_str().unwrap_or_default().to_string(),
            );
        }
    }
    (0.0, String::new())
}

/// The shipping/billing address of the cart used for tax resolution.
async fn cart_tax_address(cart: &RowMap) -> Option<RowMap> {
    db::select_one(
        "SELECT * FROM `addresses` WHERE `cart_id` = ? ORDER BY `id` ASC LIMIT 1",
        &[cart["id"].clone()],
    )
    .await
    .ok()
    .flatten()
}

/// `Cart::collectTotals()`: items -> tax -> shipping -> grand total -> round.
pub async fn collect_totals(cart_id: i64) -> ApiResult<RowMap> {
    let cart = db::find_by_id("cart", cart_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Cart not found"))?;
    let items = db::select(
        "SELECT * FROM `cart_items` WHERE `cart_id` = ? AND `parent_id` IS NULL ORDER BY `id`",
        &[Value::from(cart_id)],
    )
    .await?;
    let address = cart_tax_address(&cart).await;

    let mut sub_total = 0.0;
    let mut tax_total = 0.0;
    let mut items_qty = 0.0;
    let mut items_count = 0;

    for item in &items {
        let price = item["base_price"].as_f64().unwrap_or(0.0);
        let quantity = item["quantity"].as_i64().unwrap_or(0) as f64;
        let mut total = price * quantity;

        // Tax category: the item column, else resolved from the product.
        let tax_category_id = match item["tax_category_id"].as_i64() {
            Some(id) if id > 0 => id,
            _ => {
                item_tax_category(item["product_id"].as_i64().unwrap_or(0))
                    .await
            }
        };
        let (rate_percent, _identifier) = tax_rate_for(tax_category_id, address.as_ref()).await;
        let mut tax_amount = 0.0;
        if rate_percent > 0.0 {
            // Product prices are stored exclusive of tax (Bagisto default).
            tax_amount = (total * rate_percent / 100.0 * 10000.0).round() / 10000.0;
        }

        db::update(
            "cart_items",
            item["id"].as_i64().unwrap_or(0),
            &json!({
                "total": total,
                "base_total": total,
                "tax_percent": rate_percent,
                "tax_amount": tax_amount,
                "base_tax_amount": tax_amount,
                "updated_at": now(),
            }),
        )
        .await?;

        total += tax_amount;
        sub_total += price * quantity;
        tax_total += tax_amount;
        items_qty += quantity;
        items_count += 1;
    }

    // Selected shipping rate.
    let shipping = db::select_one(
        "SELECT * FROM `cart_shipping_rates` WHERE `cart_id` = ? AND `is_selected` = 1 LIMIT 1",
        &[Value::from(cart_id)],
    )
    .await
    .unwrap_or(None);
    let mut shipping_amount = 0.0;
    let mut shipping_tax = 0.0;
    if let Some(rate) = &shipping {
        shipping_amount = rate["price"].as_f64().unwrap_or(0.0);
        shipping_tax = rate["tax_amount"].as_f64().unwrap_or(0.0);
    }

    // Cart rules (coupons/discounts) land in phase 2; the column already
    // carries the amount so totals stay consistent.
    let discount_amount = cart["discount_amount"].as_f64().unwrap_or(0.0);
    let tax_total = tax_total + shipping_tax;
    let grand_total = sub_total + tax_total + shipping_amount - discount_amount;

    let round2 = |value: f64| (value * 100.0).round() / 100.0;

    db::update(
        "cart",
        cart_id,
        &json!({
            "sub_total": round2(sub_total),
            "base_sub_total": round2(sub_total),
            "tax_total": round2(tax_total),
            "base_tax_total": round2(tax_total),
            "discount_amount": round2(discount_amount),
            "base_discount_amount": round2(discount_amount),
            "shipping_amount": round2(shipping_amount),
            "base_shipping_amount": round2(shipping_amount),
            "grand_total": round2(grand_total),
            "base_grand_total": round2(grand_total),
            "items_qty": round2(items_qty),
            "items_count": items_count,
            "updated_at": now(),
        })
        ,
    )
    .await?;

    db::find_by_id("cart", cart_id)
        .await
        .map(|cart| cart.unwrap_or_else(db::empty_row))
}

/// Tax category of a product via the `tax_category_id` attribute.
async fn item_tax_category(product_id: i64) -> i64 {
    let row = db::select_one(
        "SELECT `v`.`integer_value` FROM `product_attribute_values` `v` \
         INNER JOIN `attributes` `a` ON `a`.`id` = `v`.`attribute_id` \
         WHERE `v`.`product_id` = ? AND `a`.`code` = 'tax_category_id' LIMIT 1",
        &[Value::from(product_id)],
    )
    .await
    .ok()
    .flatten();
    row.and_then(|row| row["integer_value"].as_i64()).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Shipping + payment methods
// ---------------------------------------------------------------------------

fn config_f64(config: Option<Value>) -> f64 {
    config.and_then(|value| value.as_str().map(str::to_string)).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0)
}

fn config_bool(config: Option<Value>) -> bool {
    match config {
        Some(Value::String(text)) => matches!(text.as_str(), "1" | "true" | "on" | "yes"),
        Some(Value::Bool(flag)) => flag,
        _ => false,
    }
}

/// Carrier rates for the cart, persisted and grouped by carrier
/// (`Shipping::collectRates`). Carriers: `flatrate` (per_unit|per_order) and
/// `free`, configured via `core_config sales.carriers.*`.
pub async fn collect_shipping_rates(
    cart_id: i64,
    address_country: &str,
) -> ApiResult<Vec<RowMap>> {
    let cart = db::find_by_id("cart", cart_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Cart not found"))?;
    let channel = db::find_by_id("channels", cart["channel_id"].as_i64().unwrap_or(1))
        .await
        .unwrap_or(None);
    let channel_code = channel
        .as_ref()
        .and_then(|channel| channel["code"].as_str())
        .unwrap_or("default")
        .to_string();

    let items = db::select(
        "SELECT `ci`.* FROM `cart_items` `ci` INNER JOIN `products` `p` ON `p`.`id` = `ci`.`product_id` \
         WHERE `ci`.`cart_id` = ? AND `ci`.`parent_id` IS NULL AND `p`.`type` IN ('simple', 'configurable', 'grouped', 'bundle')",
        &[Value::from(cart_id)],
    )
    .await?;
    let total_qty: i64 = items
        .iter()
        .map(|item| item["quantity"].as_i64().unwrap_or(0))
        .sum();
    if items.is_empty() {
        return Ok(Vec::new());
    }

    let flatrate_active = core::get_config_data("sales.carriers.flatrate.active", Some(&channel_code), None).await;
    let free_active = core::get_config_data("sales.carriers.free.active", Some(&channel_code), None).await;
    let mut rates: Vec<RowMap> = Vec::new();

    if config_bool(flatrate_active.ok().flatten()) {
        let default_rate = config_f64(
            core::get_config_data("sales.carriers.flatrate.default_rate", Some(&channel_code), None)
                .await
                .ok()
                .flatten(),
        );
        let kind = core::get_config_data("sales.carriers.flatrate.type", Some(&channel_code), None)
            .await
            .ok()
            .flatten()
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_else(|| "per_unit".to_string());
        let price = match kind.as_str() {
            "per_unit" => default_rate * total_qty as f64,
            _ => default_rate,
        };
        rates.push(db::to_map(json!({
            "carrier": "flatrate",
            "carrier_title": "Flat Rate",
            "method": "flatrate_flatrate",
            "method_title": "Flat Rate",
            "method_description": "Flat Rate Shipping",
            "price": price,
            "base_price": price,
        })));
    }

    let free_country = core::get_config_data("sales.carriers.free.country", Some(&channel_code), None)
        .await
        .ok()
        .flatten()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default();
    if config_bool(free_active.ok().flatten()) && (free_country.is_empty() || free_country == address_country) {
        rates.push(db::to_map(json!({
            "carrier": "free",
            "carrier_title": "Free Shipping",
            "method": "free_free",
            "method_title": "Free Shipping",
            "method_description": "Free Shipping",
            "price": 0.0,
            "base_price": 0.0,
        })));
    }

    // Replace previous rates for this cart.
    db::execute(
        "DELETE FROM `cart_shipping_rates` WHERE `cart_id` = ? AND `is_selected` = 0",
        &[Value::from(cart_id)],
    )
    .await?;
    for rate in &rates {
        let mut row = rate.clone();
        row.insert("cart_id".into(), json!(cart_id));
        row.insert("is_selected".into(), json!(0));
        row.insert("created_at".into(), json!(now()));
        row.insert("updated_at".into(), json!(now()));
        db::insert("cart_shipping_rates", &row).await?;
    }
    let stored = db::select(
        "SELECT * FROM `cart_shipping_rates` WHERE `cart_id` = ? ORDER BY `id`",
        &[Value::from(cart_id)],
    )
    .await?;
    Ok(stored)
}

/// Saves the chosen shipping method (marks `is_selected`) and refreshes totals.
pub async fn save_shipping_method(cart_id: i64, method: &str) -> ApiResult<()> {
    let updated = db::execute(
        "UPDATE `cart_shipping_rates` SET `is_selected` = (CASE WHEN `method` = ? THEN 1 ELSE 0 END), `updated_at` = ? WHERE `cart_id` = ?",
        &[Value::String(method.to_string()), Value::String(now()), Value::from(cart_id)],
    )
    .await?;
    if updated == 0 {
        return Err(ApiError::unprocessable("Shipping method not found."));
    }
    let cart = db::find_by_id("cart", cart_id).await?.unwrap_or_else(db::empty_row);
    let rate = db::select_one(
        "SELECT * FROM `cart_shipping_rates` WHERE `cart_id` = ? AND `is_selected` = 1 LIMIT 1",
        &[Value::from(cart_id)],
    )
    .await?
    .unwrap_or_else(db::empty_row);
    db::update(
        "cart",
        cart_id,
        &json!({
            "shipping_method": rate["method"].clone(),
            "updated_at": now(),
        })
        ,
    )
    .await?;
    let _ = cart;
    collect_totals(cart_id).await?;
    Ok(())
}

/// Supported offline payment methods (`cashondelivery`, `moneytransfer`).
pub async fn supported_payment_methods(channel_code: &str) -> Vec<RowMap> {
    let mut methods = Vec::new();
    for (code, title, description, sort) in [
        (
            "cashondelivery",
            "Cash On Delivery",
            "Cash On Delivery (COD). Payment on delivery of goods.",
            1,
        ),
        (
            "moneytransfer",
            "Money Transfer",
            "Money transfer — send the payment to the bank account.",
            2,
        ),
    ] {
        let active = config_bool(
            core::get_config_data(&format!("sales.payment_methods.{code}.active"), Some(channel_code), None)
                .await
                .ok()
                .flatten(),
        ) || {
            // Default to available when not configured at all.
            core::get_config_data(&format!("sales.payment_methods.{code}.active"), Some(channel_code), None)
                .await
                .ok()
                .flatten()
                .is_none()
        };
        if active {
            methods.push(db::to_map(json!({
                "method": code,
                "method_title": title,
                "description": description,
                "sort": sort,
            })));
        }
    }
    methods
}

pub async fn save_payment_method(cart_id: i64, method: &str) -> ApiResult<()> {
    let titles: &[(&str, &str)] = &[
        ("cashondelivery", "Cash On Delivery"),
        ("moneytransfer", "Money Transfer"),
    ];
    let title = titles
        .iter()
        .find(|(code, _)| *code == method)
        .map(|(_, title)| *title)
        .unwrap_or(method);
    let existing = db::select_one(
        "SELECT `id` FROM `cart_payment` WHERE `cart_id` = ? LIMIT 1",
        &[Value::from(cart_id)],
    )
    .await?;
    match existing {
        Some(row) => {
            db::update(
                "cart_payment",
                row["id"].as_i64().unwrap_or(0),
                &json!({"method": method, "method_title": title, "updated_at": now()}),
            )
            .await?;
        }
        None => {
            db::insert(
                "cart_payment",
                &json!({
                    "cart_id": cart_id,
                    "method": method,
                    "method_title": title,
                    "created_at": now(),
                    "updated_at": now(),
                })
                ,
            )
            .await?;
        }
    }
    Ok(())
}

/// Serialises a cart for the API (`CartResource`): items, addresses, totals.
/// Los `formatted_*` llevan el símbolo de la moneda del carrito, como el
/// `core()->formatPrice()` de Bagisto.
pub async fn cart_resource(cart: &RowMap) -> Json {
    let currency_row = db::select_one(
        "SELECT * FROM `currencies` WHERE `code` = ? LIMIT 1",
        &[cart["cart_currency_code"].clone()],
    )
    .await
    .ok()
    .flatten()
    .unwrap_or_else(|| db::to_map(json!({"symbol": "", "decimal": 2})));
    let fmt = |amount: f64| core::format_price(amount, &currency_row);
    let cart_id = cart["id"].as_i64().unwrap_or(0);
    let items = db::select(
        "SELECT * FROM `cart_items` WHERE `cart_id` = ? AND `parent_id` IS NULL ORDER BY `id`",
        &[Value::from(cart_id)],
    )
    .await
    .unwrap_or_default();
    let mut items_json: Vec<Json> = Vec::new();
    for item in items {
        let options = item["additional"].clone();
        items_json.push(json!({
            "id": item["id"],
            "quantity": item["quantity"],
            "sku": item["sku"],
            "type": item["type"],
            "name": item["name"],
            "price": item["price"],
            "formatted_price": fmt(item["price"].as_f64().unwrap_or(0.0)),
            "total": item["total"],
            "formatted_total": fmt(item["total"].as_f64().unwrap_or(0.0)),
            "discount_amount": item["discount_amount"],
            "tax_amount": item["tax_amount"],
            "product_id": item["product_id"],
            "additional": options,
        }));
    }
    let payment = db::select_one(
        "SELECT * FROM `cart_payment` WHERE `cart_id` = ? LIMIT 1",
        &[Value::from(cart_id)],
    )
    .await
    .ok()
    .flatten();

    json!({
        "id": cart["id"],
        "customer_email": cart["customer_email"],
        "is_guest": cart["is_guest"],
        "items_count": cart["items_count"],
        "items_qty": cart["items_qty"],
        "shipping_method": cart["shipping_method"],
        "coupon_code": cart["coupon_code"],
        "sub_total": cart["sub_total"],
        "formatted_sub_total": fmt(cart["sub_total"].as_f64().unwrap_or(0.0)),
        "tax_total": cart["tax_total"],
        "formatted_tax_total": fmt(cart["tax_total"].as_f64().unwrap_or(0.0)),
        "discount_amount": cart["discount_amount"],
        "formatted_discount_amount": fmt(cart["discount_amount"].as_f64().unwrap_or(0.0)),
        "shipping_amount": cart["shipping_amount"],
        "formatted_shipping_amount": fmt(cart["shipping_amount"].as_f64().unwrap_or(0.0)),
        "grand_total": cart["grand_total"],
        "formatted_grand_total": fmt(cart["grand_total"].as_f64().unwrap_or(0.0)),
        "cart_currency_code": cart["cart_currency_code"],
        "payment_method": payment.as_ref().and_then(|p| p["method"].as_str()),
        "payment_method_title": payment.as_ref().and_then(|p| p["method_title"].as_str()),
        "items": items_json,
    })
}
