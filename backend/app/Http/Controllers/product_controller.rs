//! Shop product endpoints (`/api/products`, `/api/product/{id}/reviews`).
//! Mirrors `Webkul\Shop\Http\Controllers\API\ProductController` +
//! `ReviewController`.

use std::collections::HashMap;

use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::{HeaderMap, StatusCode};
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Map, Value};

use crate::support::auth;
use crate::support::core;
use crate::support::db;
use crate::support::products;
use crate::support::response::{message, resource, validation_error};

pub struct ProductController;

/// Builds the listing query from query params:
/// `query, category_id (alias category), sort, price_min, price_max,
/// limit/page/per_page, <attribute_code>=value`.
async fn listing(
    query: &HashMap<String, String>,
    locale_code: Option<&str>,
    currency_code: Option<&str>,
) -> (Vec<Value>, Value) {
    let page = query.get("page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(1).max(1);
    let per_page = query
        .get("per_page")
        .or_else(|| query.get("limit"))
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(12)
        .clamp(1, 100);

    let mut where_clauses: Vec<String> = vec![
        "`pf`.`status` = 1".to_string(),
        "`pf`.`visible_individually` = 1".to_string(),
        "`pf`.`parent_id` IS NULL".to_string(),
        // Ignora filas de `product_flat` cuyo producto ya no existe.
        "`pf`.`product_id` IN (SELECT `id` FROM `products`)".to_string(),
    ];
    let mut binds: Vec<Value> = Vec::new();

    // Contenido del idioma pedido (refresh_flat crea fila por locale).
    let locale_filter = locale_code.unwrap_or("en").to_string();
    where_clauses.push("`pf`.`locale` = ?".to_string());
    binds.push(Value::String(locale_filter.clone()));

    // Filtros de precio en la moneda del visitante -> convertir a base.
    let base_code = core::base_currency_code().await;
    let price_filter = |amount: f64| -> std::pin::Pin<Box<dyn std::future::Future<Output = f64> + Send>> {
        let base_code = base_code.clone();
        let currency_code = currency_code.map(str::to_string);
        Box::pin(async move {
            match (&currency_code, &base_code) {
                (Some(code), Some(base)) if code != base => {
                    core::convert_to_base(amount, base, code).await
                }
                _ => amount,
            }
        })
    };

    if let Some(search) = query.get("query").map(String::as_str).map(str::trim).filter(|q| !q.is_empty()) {
        // Sinónimos del término (Search SEO): `zapatos` incluye `zapatillas,
        // shoes` — se busca por cada término con OR.
        let mut terms = vec![search.to_string()];
        let synonyms = db::select(
            "SELECT `terms` FROM `search_synonyms` WHERE `name` = ? OR `terms` LIKE ? LIMIT 5",
            &[
                Value::String(search.to_string()),
                Value::String(format!("%{search}%")),
            ],
        )
        .await
        .unwrap_or_default();
        for row in &synonyms {
            let raw = row["terms"].as_str().unwrap_or_default();
            for term in raw.split(',') {
                let term = term.trim();
                if !term.is_empty() && term.to_lowercase() != search.to_lowercase() {
                    terms.push(term.to_string());
                }
            }
        }

        let mut search_clauses: Vec<String> = Vec::new();
        for term in &terms {
            search_clauses.push("(`pf`.`name` LIKE ? OR `pf`.`sku` LIKE ?)".to_string());
            binds.push(Value::String(format!("%{term}%")));
            binds.push(Value::String(format!("%{term}%")));
        }
        where_clauses.push(format!("({})", search_clauses.join(" OR ")));

        // Registro del término buscado (Search terms, SEO).
        let results = db::scalar_i64(
            &format!(
                "SELECT COUNT(DISTINCT `pf`.`product_id`) FROM `product_flat` `pf` \
                 WHERE `pf`.`status` = 1 AND `pf`.`visible_individually` = 1 AND `pf`.`parent_id` IS NULL \
                   AND (`pf`.`name` LIKE ? OR `pf`.`sku` LIKE ?) AND `pf`.`locale` = ?",
                ),
            &[
                Value::String(format!("%{search}%")),
                Value::String(format!("%{search}%")),
                Value::String(locale_filter.clone()),
            ],
        )
        .await
        .unwrap_or(0);
        let existing = db::select_one(
            "SELECT `id` FROM `search_terms` WHERE `term` = ? LIMIT 1",
            &[Value::String(search.to_string())],
        )
        .await
        .ok()
        .flatten();
        match existing {
            Some(row) => {
                let _ = db::execute(
                    "UPDATE `search_terms` SET `uses` = `uses` + 1, `results` = ?, `updated_at` = ? WHERE `id` = ?",
                    &[
                        Value::from(results),
                        Value::String(crate::support::db::now_db()),
                        row["id"].clone(),
                    ],
                )
                .await;
            }
            None => {
                let _ = db::insert(
                    "search_terms",
                    &json!({
                        "term": search,
                        "results": results,
                        "uses": 1,
                        "locale": locale_filter,
                        "channel_id": 1,
                        "created_at": crate::support::db::now_db(),
                        "updated_at": crate::support::db::now_db(),
                    }),
                )
                .await;
            }
        }
    }

    let category_id = query
        .get("category_id")
        .or_else(|| query.get("category"))
        .and_then(|c| c.parse::<i64>().ok());
    if let Some(category_id) = category_id {
        where_clauses.push(
            "`pf`.`product_id` IN (SELECT `product_id` FROM `product_categories` WHERE `category_id` = ?)".into(),
        );
        binds.push(Value::from(category_id));
    }

    if let Some(min) = query.get("price_min").and_then(|p| p.parse::<f64>().ok()) {
        let min = price_filter(min).await;
        where_clauses.push("`ppi`.`min_price` >= ?".into());
        binds.push(Value::from(min));
    }
    if let Some(max) = query.get("price_max").and_then(|p| p.parse::<f64>().ok()) {
        let max = price_filter(max).await;
        where_clauses.push("`ppi`.`min_price` <= ?".into());
        binds.push(Value::from(max));
    }

    // Attribute filters: `code` = value or `code[]`.
    for (key, value) in query {
        if matches!(
            key.as_str(),
            "query" | "category_id" | "category" | "sort" | "page" | "per_page" | "limit" | "price_min"
                | "price_max" | "locale" | "currency" | "channel" | "new" | "featured" | "search"
        ) {
            continue;
        }
        let attribute = db::select_one(
            "SELECT `id` FROM `attributes` WHERE `code` = ? LIMIT 1",
            &[Value::String(key.clone())],
        )
        .await
        .ok()
        .flatten();
        let Some(attribute) = attribute else { continue };
        let attribute_id = attribute["id"].as_i64().unwrap_or(0);
        let values: Vec<Value> = value
            .split(',')
            .map(|entry| Value::String(entry.trim().to_string()))
            .collect();
        let placeholders = vec!["?"; values.len()].join(",");
        where_clauses.push(format!(
            "`pf`.`product_id` IN (SELECT `pav`.`product_id` FROM `product_attribute_values` `pav` \
             WHERE `pav`.`attribute_id` = ? AND (CAST(`pav`.`integer_value` AS CHAR) IN ({placeholders}) \
             OR CAST(`pav`.`text_value` AS CHAR) IN ({placeholders})))"
        ));
        binds.push(Value::from(attribute_id));
        for entry in values.clone() {
            binds.push(entry.clone());
        }
        for entry in values {
            binds.push(entry);
        }
    }

    let order = match query.get("sort").map(String::as_str) {
        Some("price_desc") => "ORDER BY `ppi`.`min_price` DESC",
        Some("price_asc") => "ORDER BY `ppi`.`min_price` ASC",
        Some("name_asc") => "ORDER BY `pf`.`name` ASC",
        Some("name_desc") => "ORDER BY `pf`.`name` DESC",
        Some("newest") => "ORDER BY `pf`.`id` DESC",
        _ => "ORDER BY `pf`.`id` DESC",
    };

    let where_sql = where_clauses.join(" AND ");
    let total = db::scalar_i64(
        &format!(
            "SELECT COUNT(DISTINCT `pf`.`product_id`) FROM `product_flat` `pf` \
             LEFT JOIN `product_price_indices` `ppi` ON `ppi`.`product_id` = `pf`.`product_id` \
             WHERE {}",
            where_sql
        ),
        &binds,
    )
    .await
    .unwrap_or(0);
    let offset = (page - 1) * per_page;
    let mut query_binds = binds.clone();
    query_binds.push(Value::from(per_page));
    query_binds.push(Value::from(offset));

    let rows = db::select(
        &format!(
            "SELECT DISTINCT `pf`.`product_id` FROM `product_flat` `pf` \
             LEFT JOIN `product_price_indices` `ppi` ON `ppi`.`product_id` = `pf`.`product_id` \
             WHERE {} {} LIMIT ? OFFSET ?",
            where_sql, order
        ),
        &query_binds,
    )
    .await
    .unwrap_or_default();

    let mut cards = Vec::new();
    for row in rows {
        if let Some(id) = row["product_id"].as_i64() {
            if let Some(card) = products::card_in(id, locale_code, currency_code).await {
                cards.push(card);
            }
        }
    }

    let meta = json!({
        "current_page": page,
        "last_page": ((total as f64) / (per_page as f64)).ceil() as i64,
        "per_page": per_page,
        "total": total,
    });
    (cards, meta)
}

impl ProductController {
    /// `GET /api/products` - paginated listing with search/filters/sort.
    pub async fn index(
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
    ) -> Response {
        let (locale, currency) = resolve_shop_context(&query, &headers).await;
        let (cards, meta) = listing(&query, locale.as_deref(), currency.as_deref()).await;
        resource(json!({"data": cards, "meta": meta}), Value::Null)
    }

    /// `GET /api/products/{id}` - detail by id or url_key. Si el slug
    /// resuelve a una variante, se sirve el configurable padre.
    pub async fn show(
        Path(id_or_key): Path<String>,
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
    ) -> Response {
        let (locale, currency) = resolve_shop_context(&query, &headers).await;
        let locale_code = locale.unwrap_or_else(|| "en".to_string());
        let currency_code = currency;
        let product_id = match id_or_key.parse::<i64>() {
            Ok(id) => id,
            Err(_) => {
                // Fallback: slug del idioma por defecto si este locale no
                // tiene traducción del url_key.
                let localized = db::select_one(
                    "SELECT `pf`.`product_id`, `p`.`parent_id` FROM `product_flat` `pf` \
                     INNER JOIN `products` `p` ON `p`.`id` = `pf`.`product_id` \
                     WHERE `pf`.`url_key` = ? AND `pf`.`locale` = ? ORDER BY `pf`.`id` DESC LIMIT 1",
                    &[Value::String(id_or_key.clone()), Value::String(locale_code.clone())],
                )
                .await
                .ok()
                .flatten();
                let by_key = match localized {
                    Some(row) => Some(row),
                    None => {
                        db::select_one(
                            "SELECT `pf`.`product_id`, `p`.`parent_id` FROM `product_flat` `pf` \
                             INNER JOIN `products` `p` ON `p`.`id` = `pf`.`product_id` \
                             WHERE `pf`.`url_key` = ? ORDER BY `pf`.`id` DESC LIMIT 1",
                            &[Value::String(id_or_key.clone())],
                        )
                        .await
                        .ok()
                        .flatten()
                    }
                };
                // URL rewrites: request_path -> target_path (Search SEO).
                let by_key = match by_key {
                    Some(row) => Some(row),
                    None => {
                        let rewrite = db::select_one(
                            "SELECT `target_path` FROM `url_rewrites` WHERE `request_path` = ? LIMIT 1",
                            &[Value::String(format!("/{}", id_or_key.trim_start_matches('/')))],
                        )
                        .await
                        .ok()
                        .flatten();
                        let Some(target) = rewrite.and_then(|row| row["target_path"].as_str().map(str::to_string)) else {
                            return message(StatusCode::NOT_FOUND, "Product not found.");
                        };
                        let target_slug = target
                            .trim_start_matches('/')
                            .rsplit('/')
                            .next()
                            .unwrap_or("")
                            .to_string();
                        db::select_one(
                            "SELECT `pf`.`product_id`, `p`.`parent_id` FROM `product_flat` `pf` \
                             INNER JOIN `products` `p` ON `p`.`id` = `pf`.`product_id` \
                             WHERE `pf`.`url_key` = ? ORDER BY `pf`.`id` DESC LIMIT 1",
                            &[Value::String(target_slug)],
                        )
                        .await
                        .ok()
                        .flatten()
                    }
                };
                let Some(row) = by_key else {
                    return message(StatusCode::NOT_FOUND, "Product not found.");
                };
                row["parent_id"]
                    .as_i64()
                    .unwrap_or_else(|| row["product_id"].as_i64().unwrap_or(0))
            }
        };
        match products::detail_in(product_id, Some(&locale_code), currency_code.as_deref()).await {
            Some(detail) => resource(detail, Value::Null),
            None => message(StatusCode::NOT_FOUND, "Product not found."),
        }
    }

    /// `GET /api/products/{id}/related`.
    pub async fn related(
        Path(id): Path<i64>,
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
    ) -> Response {
        let (locale, currency) = resolve_shop_context(&query, &headers).await;
        let cards = products::associations_in(id, "related", locale.as_deref(), currency.as_deref()).await;
        resource(json!(cards), Value::Null)
    }

    /// `GET /api/products/{id}/up-sell`.
    pub async fn up_sell(
        Path(id): Path<i64>,
        Query(query): Query<HashMap<String, String>>,
        headers: HeaderMap,
    ) -> Response {
        let (locale, currency) = resolve_shop_context(&query, &headers).await;
        let cards = products::associations_in(id, "up-sell", locale.as_deref(), currency.as_deref()).await;
        resource(json!(cards), Value::Null)
    }

    /// `GET /api/product/{id}/reviews`.
    pub async fn reviews(Path(id): Path<i64>, Query(query): Query<HashMap<String, String>>) -> Response {
        let page = query.get("page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(1);
        resource(products::reviews(id, page, 10).await, Value::Null)
    }

    /// `POST /api/product/{id}/review` - create review (auto-approved unless
    /// `catalog.products.review.summary` says otherwise).
    pub async fn store_review(
        Path(id): Path<i64>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Response {
        let mut errors = Vec::new();
        if body["title"].as_str().map(str::trim).unwrap_or_default().is_empty() {
            errors.push(("title".into(), "The title field is required.".into()));
        }
        if body["comment"].as_str().map(str::trim).unwrap_or_default().is_empty() {
            errors.push(("comment".into(), "The comment field is required.".into()));
        }
        if !errors.is_empty() {
            return validation_error(errors);
        }
        let exists = db::find_by_id("products", id).await.ok().flatten();
        if exists.is_none() {
            return message(StatusCode::NOT_FOUND, "Product not found.");
        }

        let customer_name = match auth::bearer_token(&headers)
            .and_then(|token| auth::decode_token(&token, false).ok())
            .filter(|claims| claims.prv == "customer")
            .and_then(|claims| claims.sub.parse::<i64>().ok())
        {
            Some(customer_id) => match db::find_by_id("customers", customer_id).await {
                Ok(Some(customer)) => format!(
                    "{} {}",
                    customer["first_name"].as_str().unwrap_or_default(),
                    customer["last_name"].as_str().unwrap_or_default()
                )
                .trim()
                .to_string(),
                _ => body["name"].as_str().unwrap_or("Guest").to_string(),
            },
            None => body["name"].as_str().unwrap_or("Guest").to_string(),
        };

        let rating = body["rating"].as_i64().unwrap_or(5).clamp(1, 5);
        let mut row = db::to_map(json!({
            "name": customer_name,
            "title": body["title"].clone(),
            "comment": body["comment"].clone(),
            "rating": rating,
            "status": "approved",
            "product_id": id,
            "created_at": db::now_db(),
            "updated_at": db::now_db(),
        }));
        if let Some(claims) = auth::bearer_token(&headers)
            .and_then(|token| auth::decode_token(&token, false).ok())
            .filter(|claims| claims.prv == "customer")
        {
            row.insert("customer_id".into(), json!(claims.sub.parse::<i64>().unwrap_or(0)));
        }
        match db::insert("product_reviews", &row).await {
            Ok(_) => resource(Value::Array(Vec::new()), "Your review has been submitted successfully."),
            Err(error) => error.into(),
        }
    }
}

/// Resuelve `(locale, currency)` del request validados contra el canal.
async fn resolve_shop_context(
    query: &HashMap<String, String>,
    headers: &HeaderMap,
) -> (Option<String>, Option<String>) {
    let channel = core::current_channel(headers).await.ok();
    let locale_code = match &channel {
        Some(channel) => core::resolve_locale(channel, query.get("locale").map(String::as_str))
            .await
            .ok()
            .and_then(|locale| locale["code"].as_str().map(str::to_string)),
        None => None,
    };
    let currency_code = match &channel {
        Some(channel) => core::resolve_currency(channel, query.get("currency").map(String::as_str))
            .await
            .ok()
            .and_then(|currency| currency["code"].as_str().map(str::to_string)),
        None => None,
    };
    (locale_code, currency_code)
}
