//! Cart rules engine (promotions/coupons). Mirrors `Webkul\CartRule\Helpers\CartRule`
//! + `Rule\Validator`:
//!
//! - Candidate rules: `status = 1`, date window valid, channel + customer
//!   group of the cart, ordered by `sort` (end_other_rules cuts the chain).
//! - `can_process`: coupon code (when the rule requires one) must match the
//!   cart coupon, respecting `usage_limit` / `usage_per_customer`.
//! - Conditions: `{attribute, operator, value}` over scopes `cart`,
//!   `cart_item` and `product` (EAV), AND/OR per `condition_type`.
//! - Actions: `by_percent`, `by_fixed`, `cart_fixed` (proportional split),
//!   `buy_x_get_y`; capped at `price * qty`. Shipping discount
//!   (`apply_to_shipping`) and `free_shipping` apply to the selected rate.

use larust_support::serde_json::{json, Map, Value};

use super::cart;
use super::core;
use super::db::{self, RowMap};
use super::response::{ApiError, ApiResult};

fn now() -> String {
    db::now_db()
}

fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

/// Candidate rules for the cart: active, within date window, matching the
/// channel and the customer group, ordered by `sort`.
async fn candidate_rules(channel_id: i64, customer_group_id: i64, coupon_code: Option<&str>) -> ApiResult<Vec<RowMap>> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut sql = String::from(
        "SELECT `r`.* FROM `cart_rules` `r` \
         WHERE `r`.`status` = 1 \
         AND (`r`.`starts_from` IS NULL OR `r`.`starts_from` <= ?) \
         AND (`r`.`ends_till` IS NULL OR `r`.`ends_till` >= ?) \
         AND EXISTS (SELECT 1 FROM `cart_rule_channels` `rc` WHERE `rc`.`cart_rule_id` = `r`.`id` AND `rc`.`channel_id` = ?) \
         AND EXISTS (SELECT 1 FROM `cart_rule_customer_groups` `rg` WHERE `rg`.`cart_rule_id` = `r`.`id` AND `rg`.`customer_group_id` = ?) \
         AND (`r`.`coupon_type` = 1 OR EXISTS (SELECT 1 FROM `cart_rule_coupons` `cp` WHERE `cp`.`cart_rule_id` = `r`.`id` AND `cp`.`code` = ?)) \
         ORDER BY `r`.`sort_order` ASC, `r`.`id` ASC",
    );
    let binds = vec![
        Value::String(format!("{today} 00:00:00")),
        Value::String(format!("{today} 23:59:59")),
        Value::from(channel_id),
        Value::from(customer_group_id),
        Value::String(coupon_code.unwrap_or_default().to_string()),
    ];
    let _ = &mut sql;
    db::select(&sql, &binds).await
}

/// Coupon checks: usage limit per coupon and per customer.
async fn can_process_rule(rule: &RowMap, coupon_code: Option<&str>, customer_id: Option<i64>) -> ApiResult<bool> {
    // No coupon required -> always processable.
    if rule["coupon_type"].as_i64().unwrap_or(1) == 1 {
        return Ok(true);
    }
    let Some(code) = coupon_code.filter(|c| !c.is_empty()) else {
        return Ok(false);
    };
    let coupon = db::select_one(
        "SELECT * FROM `cart_rule_coupons` WHERE `code` = ? AND `cart_rule_id` = ? LIMIT 1",
        &[Value::String(code.to_string()), rule["id"].clone()],
    )
    .await?
    .ok_or_else(|| ApiError::unprocessable("The coupon code is invalid."))?;

    // Expiry.
    if let Some(expired_at) = coupon["expired_at"].as_str().filter(|d| !d.is_empty()) {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        if expired_at < today.as_str() {
            return Ok(false);
        }
    }
    // Global usage limit.
    if let Some(limit) = coupon["usage_limit"].as_i64().filter(|l| *l > 0) {
        if coupon["times_used"].as_i64().unwrap_or(0) >= limit {
            return Ok(false);
        }
    }
    // Per-customer usage.
    if let Some(customer_id) = customer_id {
        if let Some(limit) = coupon["usage_per_customer"].as_i64().filter(|l| *l > 0) {
            let used = db::scalar_i64(
                "SELECT COALESCE(SUM(`times_used`), 0) FROM `cart_rule_coupon_usage` \
                 WHERE `cart_rule_coupon_id` = ? AND `customer_id` = ?",
                &[coupon["id"].clone(), Value::from(customer_id)],
            )
            .await
            .unwrap_or(0);
            if used >= limit {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// Resolves an attribute value for the validator: cart columns, cart item
/// columns or product EAV attributes.
async fn attribute_value(scope: &str, attribute: &str, cart: &RowMap, item: &RowMap) -> Option<Value> {
    match scope {
        "cart" => cart.get(attribute).cloned(),
        "cart_item" => item
            .get(attribute)
            .cloned()
            .or_else(|| item.get("quantity").map(|_| item["quantity"].clone())),
        "product" => {
            let product_id = item["product_id"].as_i64()?;
            let attributes = cart::attribute_map(product_id).await.ok()?;
            attributes.get(attribute).cloned()
        }
        _ => None,
    }
}

/// Evaluates one condition `{attribute, operator, value}`. Operators mirror
/// the Bagisto validator: == != >= <= > < {} (contains) !{} (not contains).
fn evaluate_condition(condition: &Value, actual: Option<&Value>) -> bool {
    let Some(actual) = actual else {
        return false;
    };
    let expected = &condition["value"];
    let operator = condition["operator"].as_str().unwrap_or("==");

    let actual_num = actual.as_f64().or_else(|| actual.as_str().and_then(|s| s.parse().ok()));
    let expected_num = expected.as_f64().or_else(|| expected.as_str().and_then(|s| s.parse().ok()));

    let contains = |haystack: &Value, needle: &Value| -> bool {
        match haystack {
            Value::Array(list) => list.contains(needle)
                || list
                    .iter()
                    .any(|entry| entry.as_str() == needle.as_str()),
            Value::String(text) => needle
                .as_str()
                .map(|needle| text.contains(needle))
                .unwrap_or(false),
            _ => false,
        }
    };

    match operator {
        "==" => actual == expected || actual_num == expected_num,
        "!=" => !(actual == expected || actual_num == expected_num),
        ">=" => matches!((actual_num, expected_num), (Some(a), Some(e)) if a >= e),
        "<=" => matches!((actual_num, expected_num), (Some(a), Some(e)) if a <= e),
        ">" => matches!((actual_num, expected_num), (Some(a), Some(e)) if a > e),
        "<" => matches!((actual_num, expected_num), (Some(a), Some(e)) if a < e),
        "{}" => contains(actual, expected),
        "!{}" => !contains(actual, expected),
        _ => false,
    }
}

/// Validates the rule conditions against the cart and one item.
/// `condition_type` 1 = ALL (AND), 2 = ANY (OR).
async fn validate_conditions(
    rule: &RowMap,
    cart: &RowMap,
    item: &RowMap,
    customer_id: Option<i64>,
) -> bool {
    let Some(conditions) = rule["conditions"].as_array() else {
        return true; // Sin condiciones -> aplica siempre.
    };
    if conditions.is_empty() {
        return true;
    }
    let any = rule["condition_type"].as_i64().unwrap_or(1) == 2;

    let mut results = Vec::new();
    for condition in conditions {
        let attribute = condition["attribute"].as_str().unwrap_or_default().to_string();
        let (scope, code) = match attribute.split_once("::") {
            Some((scope, code)) => (scope.to_string(), code.to_string()),
            None => ("cart".to_string(), attribute.clone()),
        };
        let value = attribute_value(&scope, &code, cart, item).await;
        // `cart::customer_id` especiales: invitado tratado como grupo guest.
        let mut passed = evaluate_condition(condition, value.as_ref());
        if code == "customer_id" && customer_id.is_none() && condition["value"] == json!(0) {
            passed = true; // Regla sólo invitados.
        }
        results.push(passed);
    }

    if any {
        results.iter().any(|passed| *passed)
    } else {
        results.iter().all(|passed| *passed)
    }
}

/// Per-item discount for an action (`Webkul\CartRule\Helpers\CartRule::process`).
fn item_discount(action: &str, rule: &RowMap, item: &RowMap) -> f64 {
    let quantity = item["quantity"].as_i64().unwrap_or(0) as f64;
    let price = item["base_price"].as_f64().unwrap_or(0.0);
    let already = item["discount_amount"].as_f64().unwrap_or(0.0);
    let cap = price * quantity - already;
    let amount = rule["discount_amount"].as_f64().unwrap_or(0.0);

    let discount = match action {
        "by_percent" => {
            let percent = amount.min(100.0);
            cap * percent / 100.0
        }
        "by_fixed" => amount * quantity,
        "buy_x_get_y" => {
            let step = rule["discount_step"].as_i64().unwrap_or(1).max(1) as f64;
            let get_free = amount.max(1.0);
            let periods = (quantity / (step + get_free)).floor();
            let mut free = periods * get_free;
            let remainder = quantity - periods * (step + get_free);
            if remainder > step {
                free += remainder - step;
            }
            free * price
        }
        _ => 0.0, // `cart_fixed` se reparte a nivel de carrito.
    };

    round4(discount.min(cap).max(0.0))
}

/// Applies the active cart rules to the cart: resets previous discounts,
/// processes items, splits `cart_fixed` proportionally, applies shipping
/// discount/free shipping and refreshes totals.
pub async fn apply_cart_rules(cart_id: i64) -> ApiResult<RowMap> {
    let cart = db::find_by_id("cart", cart_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Cart not found"))?;
    let coupon_code = cart["coupon_code"]
        .as_str()
        .filter(|c| !c.is_empty())
        .map(str::to_string);
    let customer_id = cart["customer_id"].as_i64();
    let customer_group_id = match customer_id {
        Some(customer_id) => db::select_one(
            "SELECT `customer_group_id` FROM `customers` WHERE `id` = ? LIMIT 1",
            &[Value::from(customer_id)],
        )
        .await?
        .and_then(|row| row["customer_group_id"].as_i64())
        .unwrap_or(2), // general
        None => 1, // guest
    };
    let channel_id = cart["channel_id"].as_i64().unwrap_or(1);

    // Reset previous per-item discounts.
    let _ = db::execute(
        "UPDATE `cart_items` SET `discount_percent` = 0, `discount_amount` = 0, `base_discount_amount` = 0, `applied_cart_rule_ids` = NULL, `updated_at` = ? WHERE `cart_id` = ?",
        &[Value::String(now()), Value::from(cart_id)],
    )
    .await;
    let _ = db::execute(
        "UPDATE `cart` SET `discount_amount` = 0, `base_discount_amount` = 0, `applied_cart_rule_ids` = NULL, `updated_at` = ? WHERE `id` = ?",
        &[Value::String(now()), Value::from(cart_id)],
    )
    .await;

    let rules = candidate_rules(channel_id, customer_group_id, coupon_code.as_deref()).await?;
    let mut applied: Vec<Value> = Vec::new();
    let mut cart_fixed_pool = 0.0;

    for rule in &rules {
        if !can_process_rule(rule, coupon_code.as_deref(), customer_id).await? {
            continue;
        }
        let action = rule["action_type"].as_str().unwrap_or_default().to_string();
        let mut rule_applied = false;

        let items = db::select(
            "SELECT * FROM `cart_items` WHERE `cart_id` = ? AND `parent_id` IS NULL ORDER BY `id`",
            &[Value::from(cart_id)],
        )
        .await?;
        for item in &items {
            if !validate_conditions(rule, &cart, item, customer_id).await {
                continue;
            }
            let discount = item_discount(&action, rule, item);
            if discount > 0.0 {
                let price = item["base_price"].as_f64().unwrap_or(0.0);
                let quantity = item["quantity"].as_i64().unwrap_or(0) as f64;
                let percent = if price * quantity > 0.0 {
                    discount / (price * quantity) * 100.0
                } else {
                    0.0
                };
                db::update(
                    "cart_items",
                    item["id"].as_i64().unwrap_or(0),
                    &json!({
                        "discount_amount": discount,
                        "base_discount_amount": discount,
                        "discount_percent": round4(percent),
                        "applied_cart_rule_ids": rule["id"].as_i64().unwrap_or(0).to_string(),
                        "updated_at": now(),
                    })
                    ,
                )
                .await?;
                rule_applied = true;
            }
        }

        // `cart_fixed`: acumula el pool para el reparto proporcional.
        if action == "cart_fixed" {
            cart_fixed_pool += rule["discount_amount"].as_f64().unwrap_or(0.0);
            rule_applied = true;
        }

        // Free shipping / shipping discount sobre la tarifa seleccionada.
        if rule["free_shipping"].as_i64().unwrap_or(0) == 1
            || rule["apply_to_shipping"].as_i64().unwrap_or(0) == 1
        {
            let rate = db::select_one(
                "SELECT `id`, `price`, `base_price` FROM `cart_shipping_rates` WHERE `cart_id` = ? AND `is_selected` = 1 LIMIT 1",
                &[Value::from(cart_id)],
            )
            .await?
            .unwrap_or_else(db::empty_row);
            if !rate.is_empty() {
                let shipping_discount = if rule["free_shipping"].as_i64().unwrap_or(0) == 1 {
                    rate["base_price"].as_f64().unwrap_or(0.0)
                } else {
                    match action.as_str() {
                        "by_percent" => {
                            rate["base_price"].as_f64().unwrap_or(0.0)
                                * rule["discount_amount"].as_f64().unwrap_or(0.0).min(100.0)
                                / 100.0
                        }
                        "by_fixed" => rule["discount_amount"].as_f64().unwrap_or(0.0),
                        _ => 0.0,
                    }
                };
                db::update(
                    "cart_shipping_rates",
                    rate["id"].as_i64().unwrap_or(0),
                    &json!({"discount_amount": round4(shipping_discount), "base_discount_amount": round4(shipping_discount), "updated_at": now()})
                    ,
                )
                .await?;
                rule_applied = true;
            }
        }

        if rule_applied {
            applied.push(rule["id"].clone());
            // `end_other_rules` corta la cadena.
            if rule["end_other_rules"].as_i64().unwrap_or(0) == 1 {
                break;
            }
        }
    }

    // Reparto proporcional de `cart_fixed` entre ítems válidos (peso base).
    if cart_fixed_pool > 0.0 {
        let items = db::select(
            "SELECT `id`, `base_price`, `quantity` FROM `cart_items` WHERE `cart_id` = ? AND `parent_id` IS NULL",
            &[Value::from(cart_id)],
        )
        .await?;
        let total_base: f64 = items
            .iter()
            .map(|item| item["base_price"].as_f64().unwrap_or(0.0) * item["quantity"].as_i64().unwrap_or(0) as f64)
            .sum();
        if total_base > 0.0 {
            for item in &items {
                let share = item["base_price"].as_f64().unwrap_or(0.0)
                    * item["quantity"].as_i64().unwrap_or(0) as f64
                    / total_base;
                let discount = round4(cart_fixed_pool * share);
                let existing = item["discount_amount"].as_f64().unwrap_or(0.0);
                db::update(
                    "cart_items",
                    item["id"].as_i64().unwrap_or(0),
                    &json!({"discount_amount": existing + discount, "base_discount_amount": existing + discount, "updated_at": now()})
                    ,
                )
                .await?;
            }
        }
    }

    // Guardar reglas aplicadas y refrescar totales.
    let applied_ids = applied
        .iter()
        .map(|id| id.as_i64().unwrap_or(0).to_string())
        .collect::<Vec<_>>()
        .join(",");
    db::update(
        "cart",
        cart_id,
        &json!({"applied_cart_rule_ids": applied_ids, "updated_at": now()})
        ,
    )
    .await?;

    cart::collect_totals(cart_id).await?;
    db::find_by_id("cart", cart_id)
        .await
        .map(|cart| cart.unwrap_or_else(db::empty_row))
}

/// Incrementa los contadores de uso del cupón (al crear la factura/pedido).
pub async fn consume_coupon_usage(cart_id: i64) {
    let Ok(Some(cart)) = db::find_by_id("cart", cart_id).await else {
        return;
    };
    let Some(code) = cart["coupon_code"].as_str().filter(|c| !c.is_empty()) else {
        return;
    };
    let _ = db::execute(
        "UPDATE `cart_rule_coupons` SET `times_used` = `times_used` + 1 WHERE `code` = ?",
        &[Value::String(code.to_string())],
    )
    .await;
    let _ = db::execute(
        "UPDATE `cart_rules` SET `times_used` = `times_used` + 1 WHERE `id` IN (SELECT `cart_rule_id` FROM `cart_rule_coupons` WHERE `code` = ?)",
        &[Value::String(code.to_string())],
    )
    .await;
    if let Some(customer_id) = cart["customer_id"].as_i64() {
        if let Ok(Some(coupon)) = db::select_one(
            "SELECT `id` FROM `cart_rule_coupons` WHERE `code` = ? LIMIT 1",
            &[Value::String(code.to_string())],
        )
        .await
        {
            let _ = db::insert(
                "cart_rule_coupon_usage",
                &json!({
                    "cart_rule_coupon_id": coupon["id"].clone(),
                    "customer_id": customer_id,
                    "times_used": 1,
                    "created_at": now(),
                    "updated_at": now(),
                })
                ,
            )
            .await;
        }
    }
}
