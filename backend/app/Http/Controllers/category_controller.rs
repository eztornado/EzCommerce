//! Shop category endpoints (`/api/categories/...`). Mirrors
//! `Webkul\Shop\Http\Controllers\API\CategoryController`.

use std::collections::HashMap;
use std::future::Future;

use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::{HeaderMap, StatusCode};
use larust_support::axum::response::Response;
use larust_support::serde_json::{json, Value};

use crate::support::core;
use crate::support::db;
use crate::support::response::{message, resource};

pub struct CategoryController;

fn now() -> String {
    db::now_db()
}

/// Builds the category tree of the channel root recursively.
fn tree_from(root_id: i64, locale: &str) -> std::pin::Pin<Box<dyn Future<Output = Vec<Value>> + Send + '_>> {
    Box::pin(async move {
        let children = db::select(
            "SELECT `c`.*, `ct`.`name`, `ct`.`slug`, `ct`.`url_path`, `ct`.`description` \
             FROM `categories` `c` \
             LEFT JOIN `category_translations` `ct` ON `ct`.`category_id` = `c`.`id` AND `ct`.`locale` = ? \
             WHERE `c`.`parent_id` = ? AND `c`.`status` = 1 ORDER BY `c`.`position`, `c`.`id`",
            &[Value::String(locale.to_string()), Value::from(root_id)],
        )
        .await
        .unwrap_or_default();

        let mut payload = Vec::new();
        for category in children {
            let id = category["id"].as_i64().unwrap_or(0);
            payload.push(json!({
                "id": id,
                "name": category["name"],
                "slug": category["slug"],
                "url_path": category["url_path"],
                "description": category["description"],
                "status": category["status"],
                "display_mode": category["display_mode"],
                "banner_url": category["category_banner"].as_str().map(|p| format!("/storage/{p}")),
                "children": tree_from(id, locale).await,
            }));
        }
        payload
    })
}

impl CategoryController {
    /// `GET /api/categories` - paginated list (root children).
    pub async fn index(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let page = query.get("page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(1).max(1);
        let per_page = query
            .get("per_page")
            .and_then(|p| p.parse::<i64>().ok())
            .unwrap_or(12)
            .clamp(1, 100);
        let locale = query.get("locale").cloned().unwrap_or_else(|| "en".to_string());

        let channel = match core::current_channel(&headers).await {
            Ok(channel) => channel,
            Err(error) => return error.into(),
        };
        let root_id = channel["root_category_id"].as_i64().unwrap_or(1);

        let total = db::scalar_i64(
            "SELECT COUNT(*) FROM `categories` WHERE `parent_id` = ? AND `status` = 1",
            &[Value::from(root_id)],
        )
        .await
        .unwrap_or(0);
        let offset = (page - 1) * per_page;
        let rows = db::select(
            "SELECT `c`.`id`, `c`.`position`, `c`.`status`, `c`.`display_mode`, `c`.`category_banner`, \
                    `ct`.`name`, `ct`.`slug`, `ct`.`url_path`, `ct`.`description` \
             FROM `categories` `c` \
             LEFT JOIN `category_translations` `ct` ON `ct`.`category_id` = `c`.`id` AND `ct`.`locale` = ? \
             WHERE `c`.`parent_id` = ? AND `c`.`status` = 1 ORDER BY `c`.`position`, `c`.`id` LIMIT ? OFFSET ?",
            &[
                Value::String(locale),
                Value::from(root_id),
                Value::from(per_page),
                Value::from(offset),
            ],
        )
        .await
        .unwrap_or_default();

        let records: Vec<Value> = rows
            .into_iter()
            .map(|row| {
                json!({
                    "id": row["id"],
                    "name": row["name"],
                    "slug": row["slug"],
                    "url_path": row["url_path"],
                    "description": row["description"],
                    "banner_url": row["category_banner"].as_str().map(|p| format!("/storage/{p}")),
                })
            })
            .collect();

        resource(
            json!({
                "data": records,
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

    /// `GET /api/categories/tree` - the visible tree of the channel.
    pub async fn tree(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let channel = match core::current_channel(&headers).await {
            Ok(channel) => channel,
            Err(error) => return error.into(),
        };
        let locale = query.get("locale").cloned().unwrap_or_else(|| "en".to_string());
        let root_id = channel["root_category_id"].as_i64().unwrap_or(1);
        let tree = tree_from(root_id, &locale).await;
        resource(json!(tree), Value::Null)
    }

    /// `GET /api/categories/{id}` - category detail (extension for slug-based
    /// navigation in Flutter).
    pub async fn show(Query(query): Query<HashMap<String, String>>, Path(id): Path<i64>) -> Response {
        let locale = query.get("locale").cloned().unwrap_or_else(|| "en".to_string());
        let category = db::select_one(
            "SELECT `c`.*, `ct`.`name`, `ct`.`slug`, `ct`.`url_path`, `ct`.`description`, \
                    `ct`.`meta_title`, `ct`.`meta_description` \
             FROM `categories` `c` \
             LEFT JOIN `category_translations` `ct` ON `ct`.`category_id` = `c`.`id` AND `ct`.`locale` = ? \
             WHERE `c`.`id` = ? LIMIT 1",
            &[Value::String(locale), Value::from(id)],
        )
        .await
        .ok()
        .flatten();
        match category {
            Some(category) => resource(category, Value::Null),
            None => message(StatusCode::NOT_FOUND, "Category not found."),
        }
    }

    /// `GET /api/categories/attributes` - filterable attributes of the
    /// category (falls back to the channel root).
    pub async fn attributes(Query(query): Query<HashMap<String, String>>) -> Response {
        let category_id = query
            .get("category_id")
            .and_then(|id| id.parse::<i64>().ok())
            .unwrap_or(1);
        let rows = db::select(
            "SELECT DISTINCT `a`.`id`, `a`.`code`, `a`.`admin_name`, `a`.`type`, `a`.`swatch_type` \
             FROM `category_filterable_attributes` `cfa` \
             INNER JOIN `attributes` `a` ON `a`.`id` = `cfa`.`attribute_id` \
             WHERE `cfa`.`category_id` IN ( \
                SELECT `id` FROM `categories` WHERE `id` = ? OR `parent_id` = ? \
             ) \
             ORDER BY `a`.`position`",
            &[Value::from(category_id), Value::from(category_id)],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `GET /api/categories/attributes/{id}/options` - options of an attribute.
    pub async fn attribute_options(Path(id): Path<i64>) -> Response {
        let options = db::select(
            "SELECT `id`, `admin_name`, `swatch_value`, `sort_order` FROM `attribute_options` \
             WHERE `attribute_id` = ? ORDER BY `sort_order`, `id`",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        resource(json!(options), Value::Null)
    }

    /// `GET /api/categories/max-price/{id?}` - max price for the filter slider.
    pub async fn max_price(Query(query): Query<HashMap<String, String>>, headers: HeaderMap, Path(id): Path<i64>) -> Response {
        let _ = headers;
        let channel = match core::current_channel(&headers).await {
            Ok(channel) => channel,
            Err(error) => return error.into(),
        };
        let channel_id = channel["id"].as_i64().unwrap_or(1);
        let max = db::scalar_i64(
            "SELECT COALESCE(CEIL(MAX(`ppi`.`min_price`)), 0) FROM `product_price_indices` `ppi` \
             INNER JOIN `product_categories` `pc` ON `pc`.`product_id` = `ppi`.`product_id` \
             WHERE `pc`.`category_id` IN (SELECT `id` FROM `categories` WHERE `id` = ? OR `parent_id` = ?) \
             AND `ppi`.`customer_group_id` = (SELECT `id` FROM `customer_groups` WHERE `code` = 'guest' LIMIT 1)",
            &[Value::from(id), Value::from(id)],
        )
        .await
        .unwrap_or(0);
        let _ = channel_id;
        resource(json!({"max_price": max}), Value::Null)
    }
}
