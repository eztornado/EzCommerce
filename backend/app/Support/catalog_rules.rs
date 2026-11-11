//! Catalog rules engine: discounts over the displayed product price.
//! Mirrors `Webkul\CatalogRule\Helpers\*`:
//!
//! - Candidate rules: `status = 1`, within the date window, matching the
//!   channel + customer group via pivots, ordered by `sort_order`; the
//!   first applicable rule wins (Bagisto indexes one price per rule/day).
//! - Actions: `to_percent` (final % of price), `by_percent` (-% of price),
//!   `to_fixed` (final price), `by_fixed` (-fixed amount).
//! - The discounted price lands in `product_price_indices` (min/regular)
//!   via `apply_to_indices`, which the shop listing/detail read.

use larust_support::serde_json::{json, Value};

use super::db::{self, RowMap};
use super::response::ApiResult;

fn now() -> String {
    db::now_db()
}

fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

/// The first active catalog rule for product/channel/group, or None.
pub async fn active_rule(
    product_id: i64,
    channel_id: i64,
    customer_group_id: i64,
) -> ApiResult<Option<RowMap>> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let rules = db::select(
        "SELECT `r`.* FROM `catalog_rules` `r` \
         WHERE `r`.`status` = 1 \
         AND (`r`.`starts_from` IS NULL OR `r`.`starts_from` <= ?) \
         AND (`r`.`ends_till` IS NULL OR `r`.`ends_till` >= ?) \
         AND EXISTS (SELECT 1 FROM `catalog_rule_channels` `rc` WHERE `rc`.`catalog_rule_id` = `r`.`id` AND `rc`.`channel_id` = ?) \
         AND EXISTS (SELECT 1 FROM `catalog_rule_customer_groups` `rg` WHERE `rg`.`catalog_rule_id` = `r`.`id` AND `rg`.`customer_group_id` = ?) \
         ORDER BY `r`.`sort_order` ASC, `r`.`id` ASC",
        &[
            Value::String(today.clone()),
            Value::String(today),
            Value::from(channel_id),
            Value::from(customer_group_id),
        ],
    )
    .await?;

    for rule in rules {
        // Product scope: conditions are phase-2 extras; an empty `conditions`
        // (NULL/[]) applies to every product. A non-empty condition attribute
        // `product::categories` with a category id list narrows the scope.
        let conditions_empty = rule["conditions"]
            .as_array()
            .map(|list| list.is_empty())
            .unwrap_or(true);
        if !conditions_empty {
            if let Some(category_ids) = product_category_ids(product_id).await {
                let mut applies = false;
                for condition in rule["conditions"].as_array().unwrap() {
                    if condition["attribute"].as_str() == Some("product::categories") {
                        if let Some(list) = condition["value"].as_array() {
                            if list
                                .iter()
                                .filter_map(Value::as_i64)
                                .any(|id| category_ids.contains(&id))
                            {
                                applies = true;
                            }
                        }
                    }
                }
                if !applies {
                    continue;
                }
            }
        }
        return Ok(Some(rule));
    }
    Ok(None)
}

async fn product_category_ids(product_id: i64) -> Option<Vec<i64>> {
    let rows = db::select(
        "SELECT `category_id` FROM `product_categories` WHERE `product_id` = ?",
        &[Value::from(product_id)],
    )
    .await
    .ok()?;
    Some(rows.iter().filter_map(|row| row["category_id"].as_i64()).collect())
}

/// Applies the rule action over a price.
pub fn apply_action(action: &str, discount_amount: f64, price: f64) -> f64 {
    let discounted = match action {
        "to_percent" => price * (1.0 - discount_amount.min(100.0) / 100.0),
        "by_percent" => price - price * discount_amount.min(100.0) / 100.0,
        "to_fixed" => discount_amount,
        "by_fixed" => price - discount_amount,
        _ => price,
    };
    round4(discounted.max(0.0))
}

/// The discounted price of a product for a customer group (0.0 if no rule).
pub async fn discounted_price(
    product_id: i64,
    price: f64,
    channel_id: i64,
    customer_group_id: i64,
) -> f64 {
    let Ok(Some(rule)) = active_rule(product_id, channel_id, customer_group_id).await else {
        return 0.0;
    };
    let action = rule["action_type"].as_str().unwrap_or_default();
    let amount = rule["discount_amount"].as_f64().unwrap_or(0.0);
    apply_action(action, amount, price)
}

/// Re-applies catalog rules to `product_price_indices` for all groups
/// (called by `refresh_flat`).
pub async fn apply_to_indices(product_id: i64, regular_price: f64) {
    let groups = db::select("SELECT `id` FROM `customer_groups` ORDER BY `id`", &[])
        .await
        .unwrap_or_default();
    let channel_id = db::select_one(
        "SELECT `id` FROM `channels` ORDER BY `id` LIMIT 1",
        &[],
    )
    .await
    .ok()
    .flatten()
    .and_then(|row| row["id"].as_i64())
    .unwrap_or(1);

    for group in groups {
        let Some(group_id) = group["id"].as_i64() else { continue };
        let discounted = discounted_price(product_id, regular_price, channel_id, group_id).await;
        if discounted <= 0.0 {
            continue;
        }
        let _ = db::execute(
            "UPDATE `product_price_indices` SET `min_price` = ?, `regular_min_price` = ?, \
             `max_price` = ?, `regular_max_price` = ?, `updated_at` = ? \
             WHERE `product_id` = ? AND `customer_group_id` = ?",
            &[
                Value::from(round4(discounted)),
                Value::from(round4(regular_price)),
                Value::from(round4(discounted)),
                Value::from(round4(regular_price)),
                Value::String(now()),
                Value::from(product_id),
                Value::from(group_id),
            ],
        )
        .await;
    }
}

/// Reindex helper used after saving a rule: refreshes the price indices of
/// every visible product (the Bagisto `reIndexComplete` equivalent).
pub async fn reindex_all() -> ApiResult<()> {
    let products = db::select(
        "SELECT `id` FROM `products` WHERE `parent_id` IS NULL ORDER BY `id`",
        &[],
    )
    .await?;
    for product in products {
        let Some(product_id) = product["id"].as_i64() else { continue };
        let price = db::select_one(
            "SELECT `price` FROM `product_flat` WHERE `product_id` = ? ORDER BY `id` LIMIT 1",
            &[Value::from(product_id)],
        )
        .await?
        .and_then(|row| row["price"].as_f64())
        .unwrap_or(0.0);
        if price > 0.0 {
            apply_to_indices(product_id, price).await;
        }
    }
    Ok(())
}

/// Saves channel + customer-group pivots of a catalog rule.
pub async fn save_pivots(rule_id: i64, body: &Value) {
    let _ = db::execute(
        "DELETE FROM `catalog_rule_channels` WHERE `catalog_rule_id` = ?",
        &[Value::from(rule_id)],
    )
    .await;
    let channels = body["channels"]
        .as_array()
        .map(|list| list.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
        .filter(|list| !list.is_empty())
        .unwrap_or_else(|| vec![1]);
    for channel_id in channels {
        let _ = db::execute(
            "INSERT OR IGNORE INTO `catalog_rule_channels` (`catalog_rule_id`, `channel_id`) VALUES (?, ?)",
            &[Value::from(rule_id), Value::from(channel_id)],
        )
        .await;
    }

    let _ = db::execute(
        "DELETE FROM `catalog_rule_customer_groups` WHERE `catalog_rule_id` = ?",
        &[Value::from(rule_id)],
    )
    .await;
    let groups = body["customer_groups"]
        .as_array()
        .map(|list| list.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
        .filter(|list| !list.is_empty())
        .unwrap_or_else(|| vec![1, 2, 3]);
    for group_id in groups {
        let _ = db::execute(
            "INSERT OR IGNORE INTO `catalog_rule_customer_groups` (`catalog_rule_id`, `customer_group_id`) VALUES (?, ?)",
            &[Value::from(rule_id), Value::from(group_id)],
        )
        .await;
    }
}

/// Helper for seeders/tests.
pub async fn insert_rule(
    name: &str,
    action_type: &str,
    discount_amount: f64,
    status: i64,
) -> ApiResult<i64> {
    let rule_id = db::insert(
        "catalog_rules",
        &json!({
            "name": name,
            "description": Value::Null,
            "starts_from": Value::Null,
            "ends_till": Value::Null,
            "status": status,
            "condition_type": 1,
            "conditions": Value::Null,
            "action_type": action_type,
            "discount_amount": discount_amount,
            "sort_order": 0,
            "created_at": now(),
            "updated_at": now(),
        })
        ,
    )
    .await?;
    save_pivots(rule_id, &json!({})).await;
    Ok(rule_id)
}
