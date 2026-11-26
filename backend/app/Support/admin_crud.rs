//! Reusable admin mutations shared by the admin HTTP controllers and the MCP
//! tools. Keeping the write path here means an MCP tool and the panel always
//! validate and persist the same way.

use larust_support::serde_json::{json, Map, Value};

use super::db;
use super::products;
use super::response::{ApiError, ApiResult};

fn now() -> String {
    db::now_db()
}

fn required<'a>(body: &'a Value, key: &str) -> ApiResult<&'a str> {
    body[key]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError::unprocessable(format!("The {key} field is required.")))
}

// ---------------------------------------------------------------------------
// Products
// ---------------------------------------------------------------------------

/// Creates the base product row plus super attributes and refreshes its flat
/// index. Content, categories, images and stock are applied through
/// [`update_product`], exactly like the two-step admin editor flow.
pub async fn create_product(body: &Value) -> ApiResult<i64> {
    let sku = required(body, "sku")?.to_string();
    let product_type = body["type"].as_str().unwrap_or("simple").to_string();
    let family = body["attribute_family_id"].as_i64().unwrap_or(1);

    let taken = db::select_one(
        "SELECT `id` FROM `products` WHERE `sku` = ? LIMIT 1",
        &[Value::String(sku.clone())],
    )
    .await?;
    if taken.is_some() {
        return Err(ApiError::unprocessable("The sku has already been taken."));
    }

    let product_id = db::insert(
        "products",
        &json!({
            "sku": sku,
            "type": product_type,
            "attribute_family_id": family,
            "created_at": now(),
            "updated_at": now(),
        }),
    )
    .await?;

    for attribute_id in body["super_attributes"].as_array().cloned().unwrap_or_default() {
        if let Some(attribute_id) = attribute_id.as_i64() {
            let _ = db::execute(
                "INSERT OR IGNORE INTO `product_super_attributes` (`product_id`, `attribute_id`) VALUES (?, ?)",
                &[Value::from(product_id), Value::from(attribute_id)],
            )
            .await;
        }
    }

    if body["attributes"].is_object()
        || body["categories"].is_array()
        || body["images"].is_array()
        || body["inventories"].is_object()
    {
        update_product(product_id, body).await?;
    } else {
        products::refresh_flat(product_id).await;
    }

    Ok(product_id)
}

pub async fn update_product(id: i64, body: &Value) -> ApiResult<()> {
    let exists = db::find_by_id("products", id).await?;
    if exists.is_none() {
        return Err(ApiError::not_found("Product not found."));
    }

    if let Some(sku) = body["sku"].as_str().map(str::trim).filter(|sku| !sku.is_empty()) {
        let taken = db::select_one(
            "SELECT `id` FROM `products` WHERE `sku` = ? AND `id` <> ? LIMIT 1",
            &[Value::String(sku.to_string()), Value::from(id)],
        )
        .await?;
        if taken.is_some() {
            return Err(ApiError::unprocessable("The sku has already been taken."));
        }
        db::execute(
            "UPDATE `products` SET `sku` = ?, `updated_at` = ? WHERE `id` = ?",
            &[Value::String(sku.to_string()), Value::String(now()), Value::from(id)],
        )
        .await?;
    }

    let locale = body["locale"].as_str();
    if let Some(attributes) = body["attributes"].as_object() {
        for (code, value) in attributes {
            products::set_attribute_in(id, code, value, locale).await;
        }
    }

    if let Some(categories) = body["categories"].as_array() {
        db::execute(
            "DELETE FROM `product_categories` WHERE `product_id` = ?",
            &[Value::from(id)],
        )
        .await?;
        for category in categories {
            if let Some(category_id) = category.as_i64().or_else(|| category["id"].as_i64()) {
                let _ = db::execute(
                    "INSERT OR IGNORE INTO `product_categories` (`product_id`, `category_id`) VALUES (?, ?)",
                    &[Value::from(id), Value::from(category_id)],
                )
                .await;
            }
        }
    }

    if let Some(images) = body["images"].as_array() {
        db::execute(
            "DELETE FROM `product_images` WHERE `product_id` = ?",
            &[Value::from(id)],
        )
        .await?;
        for (position, image) in images.iter().enumerate() {
            let path = image
                .as_str()
                .map(str::to_string)
                .or_else(|| image["path"].as_str().map(str::to_string));
            let Some(path) = path else { continue };
            let _ = db::insert(
                "product_images",
                &json!({
                    "product_id": id,
                    "type": "image",
                    "path": path,
                    "position": position as i64,
                    "created_at": now(),
                    "updated_at": now(),
                }),
            )
            .await;
        }
    }

    if let Some(links) = body["links"].as_object() {
        for (kind, ids) in links {
            let table = match kind.as_str() {
                "related" => "product_relations",
                "up_sells" => "product_up_sells",
                "cross_sells" => "product_cross_sells",
                _ => continue,
            };
            db::execute(
                &format!("DELETE FROM `{table}` WHERE `parent_id` = ?"),
                &[Value::from(id)],
            )
            .await?;
            for child in ids.as_array().cloned().unwrap_or_default() {
                if let Some(child_id) = child.as_i64() {
                    if child_id == id {
                        continue;
                    }
                    let _ = db::execute(
                        &format!(
                            "INSERT OR IGNORE INTO `{table}` (`parent_id`, `child_id`) VALUES (?, ?)"
                        ),
                        &[Value::from(id), Value::from(child_id)],
                    )
                    .await;
                }
            }
        }
    }

    if let Some(inventories) = body["inventories"].as_object() {
        for (source, qty) in inventories {
            let Some(source_id) = source.parse::<i64>().ok() else { continue };
            let quantity = qty.as_i64().unwrap_or(0);
            let existing = db::select_one(
                "SELECT `id` FROM `product_inventories` WHERE `product_id` = ? AND `inventory_source_id` = ? LIMIT 1",
                &[Value::from(id), Value::from(source_id)],
            )
            .await?;
            match existing {
                Some(row) => {
                    db::update(
                        "product_inventories",
                        row["id"].as_i64().unwrap_or(0),
                        &json!({"qty": quantity, "updated_at": now()}),
                    )
                    .await?;
                }
                None => {
                    db::insert(
                        "product_inventories",
                        &json!({
                            "product_id": id,
                            "inventory_source_id": source_id,
                            "qty": quantity,
                            "created_at": now(),
                            "updated_at": now(),
                        }),
                    )
                    .await?;
                }
            }
        }
    }

    products::refresh_flat(id).await;
    Ok(())
}

pub async fn delete_product(id: i64) -> ApiResult<()> {
    if db::find_by_id("products", id).await?.is_none() {
        return Err(ApiError::not_found("Product not found."));
    }
    db::delete("products", id).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Categories
// ---------------------------------------------------------------------------

pub async fn create_category(body: &Value) -> ApiResult<i64> {
    let name = required(body, "name")?.to_string();
    let parent_id = body["parent_id"].as_i64().unwrap_or(1);
    let parent = db::find_by_id("categories", parent_id).await?;
    let parent_rgt = parent
        .as_ref()
        .and_then(|row| row["_rgt"].as_i64())
        .unwrap_or(1);

    db::execute(
        "UPDATE `categories` SET `_lft` = `_lft` + 2 WHERE `_lft` >= ?",
        &[Value::from(parent_rgt)],
    )
    .await?;
    db::execute(
        "UPDATE `categories` SET `_rgt` = `_rgt` + 2 WHERE `_rgt` >= ?",
        &[Value::from(parent_rgt)],
    )
    .await?;

    let slug = crate::support::slug::slugify(&name);
    let category_id = db::insert(
        "categories",
        &json!({
            "parent_id": parent_id,
            "position": body["position"].as_i64().unwrap_or(0),
            "status": body["status"].as_i64().unwrap_or(1),
            "_lft": parent_rgt,
            "_rgt": parent_rgt + 1,
            "display_mode": body["display_mode"].as_str().unwrap_or("products_and_description"),
            "created_at": now(),
            "updated_at": now(),
        }),
    )
    .await?;

    db::insert(
        "category_translations",
        &json!({
            "category_id": category_id,
            "locale": body["locale"].as_str().unwrap_or("en"),
            "name": name,
            "slug": slug,
            "url_path": slug,
            "description": body["description"].clone(),
            "meta_title": body["meta_title"].clone(),
            "meta_description": body["meta_description"].clone(),
            "created_at": now(),
            "updated_at": now(),
        }),
    )
    .await?;

    Ok(category_id)
}

pub async fn update_category(id: i64, locale: &str, body: &Value) -> ApiResult<()> {
    if db::find_by_id("categories", id).await?.is_none() {
        return Err(ApiError::not_found("Category not found."));
    }
    db::execute(
        "UPDATE `categories` SET `position` = ?, `status` = ?, `updated_at` = ? WHERE `id` = ?",
        &[
            Value::from(body["position"].as_i64().unwrap_or(0)),
            Value::from(body["status"].as_i64().unwrap_or(1)),
            Value::String(now()),
            Value::from(id),
        ],
    )
    .await?;

    let name = body["name"].as_str().unwrap_or_default();
    let translation = json!({
        "name": body["name"].clone(),
        "slug": crate::support::slug::slugify(name),
        "description": body["description"].clone(),
        "meta_title": body["meta_title"].clone(),
        "meta_description": body["meta_description"].clone(),
        "updated_at": now(),
    });
    let existing = db::select_one(
        "SELECT `id` FROM `category_translations` WHERE `category_id` = ? AND `locale` = ? LIMIT 1",
        &[Value::from(id), Value::String(locale.to_string())],
    )
    .await?;
    match existing {
        Some(row) => {
            db::update(
                "category_translations",
                row["id"].as_i64().unwrap_or(0),
                &translation,
            )
            .await?;
        }
        None => {
            let mut translation = db::to_map(translation);
            translation.insert("category_id".into(), json!(id));
            translation.insert("locale".into(), json!(locale));
            translation.insert("created_at".into(), json!(now()));
            db::insert("category_translations", &translation).await?;
        }
    }
    Ok(())
}

pub async fn delete_category(id: i64) -> ApiResult<()> {
    if id == 1 {
        return Err(ApiError::forbidden("The root category cannot be deleted."));
    }
    if db::find_by_id("categories", id).await?.is_none() {
        return Err(ApiError::not_found("Category not found."));
    }
    db::delete("categories", id).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// CMS pages
// ---------------------------------------------------------------------------

fn validate_cms(body: &Value) -> ApiResult<()> {
    required(body, "page_title")?;
    required(body, "url_key")?;
    Ok(())
}

async fn save_cms_translation(page_id: i64, body: &Value) -> ApiResult<()> {
    let locale = body["locale"].as_str().unwrap_or("en").to_string();
    let translation = json!({
        "cms_page_id": page_id,
        "locale": locale,
        "page_title": body["page_title"].clone(),
        "url_key": crate::support::slug::slugify(body["url_key"].as_str().unwrap_or_default()),
        "html_content": body["html_content"].clone(),
        "meta_title": body["meta_title"].clone(),
        "meta_description": body["meta_description"].clone(),
        "updated_at": now(),
    });
    let existing = db::select_one(
        "SELECT `id` FROM `cms_page_translations` WHERE `cms_page_id` = ? AND `locale` = ? LIMIT 1",
        &[Value::from(page_id), Value::String(locale)],
    )
    .await?;
    match existing {
        Some(row) => {
            db::update(
                "cms_page_translations",
                row["id"].as_i64().unwrap_or(0),
                &translation,
            )
            .await?;
        }
        None => {
            let mut translation = db::to_map(translation);
            translation.insert("created_at".into(), json!(now()));
            db::insert("cms_page_translations", &translation).await?;
        }
    }
    Ok(())
}

pub async fn create_cms_page(body: &Value) -> ApiResult<i64> {
    validate_cms(body)?;
    let page_id = db::insert(
        "cms_pages",
        &json!({"created_at": now(), "updated_at": now()}),
    )
    .await?;
    save_cms_translation(page_id, body).await?;
    let _ = db::execute(
        "INSERT OR IGNORE INTO `cms_page_channels` (`cms_page_id`, `channel_id`) VALUES (?, 1)",
        &[Value::from(page_id)],
    )
    .await;
    Ok(page_id)
}

pub async fn update_cms_page(id: i64, body: &Value) -> ApiResult<()> {
    if db::find_by_id("cms_pages", id).await?.is_none() {
        return Err(ApiError::not_found("Page not found."));
    }
    validate_cms(body)?;
    save_cms_translation(id, body).await
}

pub async fn delete_cms_page(id: i64) -> ApiResult<()> {
    if db::find_by_id("cms_pages", id).await?.is_none() {
        return Err(ApiError::not_found("Page not found."));
    }
    db::delete("cms_pages", id).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Customers and configuration
// ---------------------------------------------------------------------------

pub async fn create_customer(body: &Value) -> ApiResult<i64> {
    let first_name = required(body, "first_name")?.to_string();
    let last_name = required(body, "last_name")?.to_string();
    let email = required(body, "email")?.to_lowercase();
    let password = required(body, "password")?.to_string();
    if password.len() < 6 {
        return Err(ApiError::unprocessable(
            "The password must be at least 6 characters.",
        ));
    }
    let taken = db::select_one(
        "SELECT `id` FROM `customers` WHERE `email` = ? LIMIT 1",
        &[Value::String(email.clone())],
    )
    .await?;
    if taken.is_some() {
        return Err(ApiError::unprocessable("The email has already been taken."));
    }
    let hash = crate::support::auth::hash_password(&password)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let group = match body["customer_group_id"].as_i64() {
        Some(group) => group,
        None => db::select_one(
            "SELECT `id` FROM `customer_groups` WHERE `code` = 'general' LIMIT 1",
            &[],
        )
        .await?
        .and_then(|row| row["id"].as_i64())
        .unwrap_or(2),
    };
    let id = db::insert(
        "customers",
        &json!({
            "first_name": first_name,
            "last_name": last_name,
            "email": email,
            "phone": body["phone"].clone(),
            "password": hash,
            "status": body["status"].as_i64().unwrap_or(1),
            "is_verified": 1,
            "customer_group_id": group,
            "subscribed_to_news_letter": body["subscribed_to_news_letter"].as_i64().unwrap_or(0),
            "created_at": now(),
            "updated_at": now(),
        }),
    )
    .await?;
    Ok(id)
}

pub async fn update_customer(id: i64, body: &Value) -> ApiResult<()> {
    if db::find_by_id("customers", id).await?.is_none() {
        return Err(ApiError::not_found("Customer not found."));
    }
    let mut row = body.as_object().cloned().unwrap_or_default();
    row.remove("password");
    row.insert("updated_at".into(), json!(now()));
    db::update("customers", id, &row).await?;
    Ok(())
}

/// Upserts `{code: value}` pairs into `core_config` for the global scope.
pub async fn save_configuration(values: &Map<String, Value>) -> ApiResult<()> {    for (code, value) in values {
        let value = match value {
            Value::String(value) => value.clone(),
            other => other.to_string(),
        };
        let existing = db::select_one(
            "SELECT `id` FROM `core_config` WHERE `code` = ? AND `channel_code` IS NULL AND `locale_code` IS NULL LIMIT 1",
            &[Value::String(code.clone())],
        )
        .await?;
        match existing {
            Some(row) => {
                db::update(
                    "core_config",
                    row["id"].as_i64().unwrap_or(0),
                    &json!({"value": value, "updated_at": now()}),
                )
                .await?;
            }
            None => {
                db::insert(
                    "core_config",
                    &json!({
                        "code": code,
                        "value": value,
                        "created_at": now(),
                        "updated_at": now(),
                    }),
                )
                .await?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Refunds
// ---------------------------------------------------------------------------

/// Creates a refund, routing through Stripe first when the order's captured
/// payment used it. Shared by the admin controller and MCP so both paths take
/// exactly the same side effects.
pub async fn create_refund(order_id: i64, body: &Value) -> ApiResult<Map<String, Value>> {
    let requested = body["items"].as_object().cloned().unwrap_or_default();
    let adjustment_refund = body["adjustment_refund"].as_f64().unwrap_or(0.0);
    let adjustment_fee = body["adjustment_fee"].as_f64().unwrap_or(0.0);

    let preview =
        super::sales::refund_preview(order_id, &requested, adjustment_refund, adjustment_fee)
            .await?;
    if preview.amount <= 0.0 {
        return Err(ApiError::bad_request("Nothing to refund."));
    }

    // Stripe: return the money first and register the local refund with the
    // gateway's own `re_...` id and real status.
    if preview.method == "stripe" {
        let stripe_refund = super::stripe::refund_payment(
            order_id,
            preview.amount,
            &preview.currency,
            "requested_by_customer",
        )
        .await?;
        let transaction_id = stripe_refund["id"].as_str().unwrap_or_default().to_string();
        let state = match stripe_refund["status"].as_str().unwrap_or("succeeded") {
            "succeeded" => "refunded",
            "pending" => "pending",
            "failed" | "canceled" => {
                return Err(ApiError::bad_request("Stripe rejected the refund."))
            }
            _ => "pending",
        };
        return match super::sales::create_refund(
            order_id,
            &requested,
            adjustment_refund,
            adjustment_fee,
            Some(&transaction_id),
            state,
        )
        .await
        {
            Ok(refund) => Ok(refund),
            Err(error) => {
                // The money already left: at least leave a transaction-log
                // trace if the local record fails.
                let _ = db::insert(
                    "order_transactions",
                    &json!({
                        "transaction_id": transaction_id,
                        "status": stripe_refund["status"],
                        "type": "refund",
                        "amount": preview.amount,
                        "base_amount": preview.amount,
                        "payment_method": "stripe",
                        "currency_code": preview.currency,
                        "data": stripe_refund,
                        "order_id": order_id,
                        "created_at": now(),
                        "updated_at": now(),
                    }),
                )
                .await;
                Err(error)
            }
        };
    }

    super::sales::create_refund(
        order_id,
        &requested,
        adjustment_refund,
        adjustment_fee,
        None,
        "refunded",
    )
    .await
}

/// Adds an admin note to an order.
pub async fn add_order_comment(
    order_id: i64,
    admin_id: i64,
    comment: &str,
    customer_notified: bool,
) -> ApiResult<i64> {
    if db::find_by_id("orders", order_id).await?.is_none() {
        return Err(ApiError::not_found("Order not found."));
    }
    if comment.trim().is_empty() {
        return Err(ApiError::unprocessable("The comment field is required."));
    }
    let id = db::insert(
        "order_comments",
        &json!({
            "order_id": order_id,
            "comment": comment,
            "customer_notified": customer_notified as i64,
            "user_id": admin_id,
            "created_at": now(),
            "updated_at": now(),
        }),
    )
    .await?;
    Ok(id)
}
