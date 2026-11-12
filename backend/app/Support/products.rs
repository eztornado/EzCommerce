//! Product serialization (`ProductCardResource` / `ProductResource`) shared
//! by listing, detail, related/up-sell/cross-sell endpoints.

use larust_support::serde_json::{json, Value};

use larust_support::serde_json::Map;

fn now() -> String {
    db::now_db()
}

use super::cart;
use super::catalog_rules;
use super::core;
use super::db::{self, RowMap};

/// Unidades disponibles del producto en el canal por defecto. Un
/// configurable no tiene inventario propio: se suma el de sus variantes.
async fn stock_quantity(product_id: i64, product_type: &str) -> i64 {
    if product_type == "configurable" {
        let variants = db::select(
            "SELECT `id` FROM `products` WHERE `parent_id` = ?",
            &[Value::from(product_id)],
        )
        .await
        .unwrap_or_default();
        let mut total = 0;
        for variant in variants {
            if let Some(variant_id) = variant["id"].as_i64() {
                total += cart::available_quantity(variant_id, 1).await;
            }
        }
        total
    } else {
        cart::available_quantity(product_id, 1).await
    }
}

/// `ProductCardResource`: listing card fields.
pub async fn card(product_id: i64) -> Option<Value> {
    card_in(product_id, None, None).await
}

/// Locale/currency-aware card: contenido del idioma pedido (con fallback) y
/// precios convertidos a la moneda pedida con las tasas del canal.
pub async fn card_in(
    product_id: i64,
    locale: Option<&str>,
    currency: Option<&str>,
) -> Option<Value> {
    let product = db::find_by_id("products", product_id).await.ok().flatten()?;
    let attributes = cart::attribute_map_in(product_id, locale).await.ok()?;
    let product_type = product["type"].as_str().unwrap_or("simple").to_string();

    // Effective price = special_price (ventana activa) o price, rebajado
    // además por la regla de catálogo activa si la hay.
    let mut effective = cart::effective_price(&attributes);
    let catalog_discounted =
        catalog_rules::discounted_price(product_id, effective, 1, 1).await; // grupo guest
    let mut on_sale = false;
    if catalog_discounted > 0.0 && catalog_discounted < effective {
        effective = catalog_discounted;
        on_sale = true;
    }

    // Conversión de presentación: base -> moneda pedida (los `base_*` del
    // pedido y del carrito permanecen en la moneda base).
    let currency_row = match currency {
        Some(code) => core::resolve_currency_by_code(code).await,
        None => None,
    };
    let base_code = core::base_currency_code().await;
    let currency_code = currency_row
        .as_ref()
        .and_then(|row| row["code"].as_str())
        .unwrap_or_default()
        .to_string();
    let effective = match (&currency_row, &base_code) {
        (Some(_), Some(base)) if !currency_code.is_empty() => {
            core::convert_price(effective, base, &currency_code).await
        }
        _ => effective,
    };
    let regular_amount = match (&currency_row, &base_code) {
        (Some(_), Some(base)) if !currency_code.is_empty() => {
            core::convert_price(attributes["price"].as_f64().unwrap_or(0.0), base, &currency_code).await
        }
        _ => attributes["price"].as_f64().unwrap_or(0.0),
    };
    let price = match &currency_row {
        Some(row) => core::format_price(effective, row),
        None => core::format_amount(effective, 2),
    };
    let regular = match &currency_row {
        Some(row) => core::format_price(regular_amount, row),
        None => core::format_amount(regular_amount, 2),
    };
    let special = attributes["special_price"].as_f64().unwrap_or(0.0);
    if special > 0.0 && special < attributes["price"].as_f64().unwrap_or(0.0) {
        on_sale = true;
    }

    let images = db::select(
        "SELECT `path` FROM `product_images` WHERE `product_id` = ? ORDER BY `position`, `id`",
        &[Value::from(product_id)],
    )
    .await
    .unwrap_or_default();
    let base_image = images
        .first()
        .and_then(|row| row["path"].as_str())
        .map(str::to_string);

    let ratings = db::select_one(
        "SELECT COALESCE(AVG(`rating`), 0) AS average, COUNT(*) AS total FROM `product_reviews` \
         WHERE `product_id` = ? AND `status` = 'approved'",
        &[Value::from(product_id)],
    )
    .await
    .unwrap_or(None)
    .unwrap_or_else(db::empty_row);

    // El stock real vive en `product_inventories` (fuentes del canal), no en
    // el mapa EAV. Para un configurable se suma el de sus variantes.
    let available = stock_quantity(product_id, &product_type).await;

    let saleable = match product_type.as_str() {
        "configurable" => {
            db::scalar_i64(
                "SELECT COUNT(*) FROM `products` WHERE `parent_id` = ?",
                &[Value::from(product_id)],
            )
            .await
            .unwrap_or(0)
                > 0
                && available > 0
        }
        "downloadable" | "booking" => true,
        "virtual" | "simple" => available > 0,
        _ => true,
    };

    Some(json!({
        "id": product_id,
        "sku": attributes["sku"],
        "type": product_type,
        "name": attributes["name"],
        "url_key": attributes["url_key"],
        "price": effective,
        "formatted_price": price,
        "formatted_regular_price": regular,
        "special_price": if on_sale { Value::from(effective) } else { Value::Null },
        "on_sale": on_sale,
        "is_new": attributes["new"].as_i64().unwrap_or(0) == 1,
        "is_featured": attributes["featured"].as_i64().unwrap_or(0) == 1,
        "is_saleable": saleable,
        "base_image": base_image.map(|path| json!({"url": format!("/storage/{path}")})),
        "images": images.iter().map(|row| json!({"url": format!("/storage/{}", row["path"].as_str().unwrap_or_default())})).collect::<Vec<_>>(),
        "in_stock": available > 0,
        "quantity": available,
        "short_description": attributes["short_description"],
        "ratings": {
            "average": ratings["average"],
            "total": ratings["total"],
        },
        "reviews": {"total": ratings["total"]},
    }))
}

/// `ProductResource`: full detail with attribute pairs, super attributes,
/// variant options and associations.
pub async fn detail(product_id: i64) -> Option<Value> {
    detail_in(product_id, None, None).await
}

pub async fn detail_in(
    product_id: i64,
    locale: Option<&str>,
    currency: Option<&str>,
) -> Option<Value> {
    let mut detail_value = card_in(product_id, locale, currency).await?;
    let product = db::find_by_id("products", product_id).await.ok().flatten()?;
    let attributes = cart::attribute_map_in(product_id, locale).await.ok()?;
    let product_type = product["type"].as_str().unwrap_or("simple").to_string();

    // Attribute pairs shown in "Additional Information".
    let visible = db::select(
        "SELECT `a`.`code`, `a`.`admin_name`, `a`.`type` FROM `attributes` `a` \
         WHERE `a`.`is_visible_on_front` = 1 AND `a`.`code` NOT IN ('sku', 'name', 'status', 'visible_individually') \
         ORDER BY `a`.`position`",
        &[],
    )
    .await
    .unwrap_or_default();
    let mut additional = Vec::new();
    for attribute in visible {
        let code = attribute["code"].as_str().unwrap_or_default();
        let value = &attributes[code];
        if value.is_null() {
            continue;
        }
        let value = if attribute["type"].as_str() == Some("select") {
            let option = db::select_one(
                "SELECT `admin_name` FROM `attribute_options` WHERE `id` = ? LIMIT 1",
                &[value.clone()],
            )
            .await
            .ok()
            .flatten();
            option
                .and_then(|row| row["admin_name"].clone().as_str().map(str::to_string))
                .map(Value::String)
                .unwrap_or_else(|| value.clone())
        } else {
            value.clone()
        };
        additional.push(json!({
            "code": code,
            "label": attribute["admin_name"],
            "value": value,
        }));
    }
    detail_value["additional_information"] = Value::Array(additional);
    detail_value["description"] = attributes["description"].clone();

    if product_type == "configurable" {
        let super_attributes = db::select(
            "SELECT `a`.`id`, `a`.`code`, `a`.`admin_name`, `a`.`swatch_type` FROM `product_super_attributes` `ps` \
             INNER JOIN `attributes` `a` ON `a`.`id` = `ps`.`attribute_id` WHERE `ps`.`product_id` = ? \
             ORDER BY `a`.`position`",
            &[Value::from(product_id)],
        )
        .await
        .unwrap_or_default();
        let mut payload = Vec::new();
        for attribute in super_attributes {
            let attribute_id = attribute["id"].as_i64().unwrap_or(0);
            let options = db::select(
                "SELECT DISTINCT `ao`.`id`, `ao`.`admin_name`, `ao`.`swatch_value`, `ao`.`sort_order` \
                 FROM `attribute_options` `ao` \
                 INNER JOIN `product_attribute_values` `pav` ON `pav`.`integer_value` = `ao`.`id` \
                 WHERE `pav`.`attribute_id` = ? AND `pav`.`product_id` IN \
                   (SELECT `id` FROM `products` WHERE `parent_id` = ?) \
                 ORDER BY `ao`.`sort_order`",
                &[Value::from(attribute_id), Value::from(product_id)],
            )
            .await
            .unwrap_or_default();
            payload.push(json!({
                "id": attribute_id,
                "code": attribute["code"],
                "label": attribute["admin_name"],
                "swatch_type": attribute["swatch_type"],
                "options": options,
            }));
        }
        detail_value["super_attributes"] = Value::Array(payload);

        let variants = db::select(
            "SELECT `id`, `sku` FROM `products` WHERE `parent_id` = ? ORDER BY `id`",
            &[Value::from(product_id)],
        )
        .await
        .unwrap_or_default();
        let mut variant_payload = Vec::new();
        for variant in variants {
            let variant_id = variant["id"].as_i64().unwrap_or(0);
            let variant_attributes = db::select(
                "SELECT `a`.`code`, `pav`.`integer_value` FROM `product_attribute_values` `pav` \
                 INNER JOIN `attributes` `a` ON `a`.`id` = `pav`.`attribute_id` \
                 WHERE `pav`.`product_id` = ? AND `a`.`is_configurable` = 1",
                &[Value::from(variant_id)],
            )
            .await
            .unwrap_or_default();
            let mut selection = json!({});
            for value in &variant_attributes {
                selection[value["code"].as_str().unwrap_or_default()] = value["integer_value"].clone();
            }
            let variant_price = cart::attribute_map(variant_id)
                .await
                .ok()
                .map(|attrs| cart::effective_price(&attrs))
                .unwrap_or(0.0);
            let variant_price = match (currency, core::base_currency_code().await) {
                (Some(code), Some(base)) if code != base => {
                    core::convert_price(variant_price, &base, code).await
                }
                _ => variant_price,
            };
            variant_payload.push(json!({
                "id": variant_id,
                "sku": variant["sku"],
                "attributes": selection,
                "price": variant_price,
            }));
        }
        detail_value["variants"] = Value::Array(variant_payload);
    }

    if product_type == "grouped" {
        let grouped = db::select(
            "SELECT `gp`.`qty`, `gp`.`associated_product_id` FROM `product_grouped_products` `gp` \
             WHERE `gp`.`product_id` = ? ORDER BY `gp`.`sort_order`",
            &[Value::from(product_id)],
        )
        .await
        .unwrap_or_default();
        let mut payload = Vec::new();
        for row in grouped {
            if let Some(associated) = row["associated_product_id"].as_i64() {
                if let Some(associated_card) = card(associated).await {
                    payload.push(json!({"qty": row["qty"], "product": associated_card}));
                }
            }
        }
        detail_value["grouped_products"] = Value::Array(payload);
    }

    if product_type == "downloadable" {
        let links = db::select(
            "SELECT `id`, `title`, `price`, `file_name` FROM `product_downloadable_links` \
             WHERE `product_id` = ? ORDER BY `sort_order`",
            &[Value::from(product_id)],
        )
        .await
        .unwrap_or_default();
        detail_value["downloadable_links"] = Value::Array(links.iter().cloned().map(Value::Object).collect());
    }

    if product_type == "bundle" {
        let options = db::select(
            "SELECT `id`, `type`, `is_required`, `sort_order` FROM `product_bundle_options` \
             WHERE `product_id` = ? ORDER BY `sort_order`",
            &[Value::from(product_id)],
        )
        .await
        .unwrap_or_default();
        let mut payload = Vec::new();
        for option in options {
            let products = db::select(
                "SELECT `pbop`.`qty`, `pbop`.`is_default`, `pbop`.`product_id` FROM `product_bundle_option_products` `pbop` \
                 WHERE `pbop`.`product_bundle_option_id` = ? ORDER BY `pbop`.`sort_order`",
                &[option["id"].clone()],
            )
            .await
            .unwrap_or_default();
            let mut product_payload = Vec::new();
            for row in products {
                if let Some(bundle_product) = row["product_id"].as_i64() {
                    if let Some(bundle_card) = card(bundle_product).await {
                        product_payload.push(json!({"qty": row["qty"], "is_default": row["is_default"], "product": bundle_card}));
                    }
                }
            }
            payload.push(json!({
                "id": option["id"],
                "type": option["type"],
                "is_required": option["is_required"],
                "label": option["id"],
                "products": product_payload,
            }));
        }
        detail_value["bundle_options"] = Value::Array(payload);
    }

    Some(detail_value)
}

/// Products associated: related / up-sell.
pub async fn associations(product_id: i64, kind: &str) -> Vec<Value> {
    associations_in(product_id, kind, None, None).await
}

pub async fn associations_in(
    product_id: i64,
    kind: &str,
    locale: Option<&str>,
    currency: Option<&str>,
) -> Vec<Value> {
    let table = match kind {
        "related" => "product_relations",
        _ => "product_up_sells",
    };
    let rows = db::select(
        &format!(
            "SELECT `child_id` FROM `{table}` WHERE `parent_id` = ? LIMIT 12"
        ),
        &[Value::from(product_id)],
    )
    .await
    .unwrap_or_default();
    let mut cards = Vec::new();
    for row in rows {
        if let Some(child) = row["child_id"].as_i64() {
            if let Some(child_card) = card_in(child, locale, currency).await {
                cards.push(child_card);
            }
        }
    }
    cards
}

/// Reviews of a product.
pub async fn reviews(product_id: i64, page: i64, per_page: i64) -> Value {
    let total = db::scalar_i64(
        "SELECT COUNT(*) FROM `product_reviews` WHERE `product_id` = ? AND `status` = 'approved'",
        &[Value::from(product_id)],
    )
    .await
    .unwrap_or(0);
    let offset = (page.max(1) - 1) * per_page;
    let rows = db::select(
        "SELECT `id`, `name`, `title`, `rating`, `comment`, `created_at` FROM `product_reviews` \
         WHERE `product_id` = ? AND `status` = 'approved' ORDER BY `id` DESC LIMIT ? OFFSET ?",
        &[
            Value::from(product_id),
            Value::from(per_page),
            Value::from(offset),
        ],
    )
    .await
    .unwrap_or_default();
    let last_page = ((total as f64) / (per_page as f64)).ceil() as i64;
    json!({
        "data": rows,
        "meta": {
            "current_page": page,
            "last_page": last_page,
            "per_page": per_page,
            "total": total,
        }
    })
}

async fn attr_id_by_code(code: &str) -> i64 {
    db::select_one(
        "SELECT `id` FROM `attributes` WHERE `code` = ? LIMIT 1",
        &[Value::String(code.to_string())],
    )
    .await
    .ok()
    .flatten()
    .and_then(|row| row["id"].as_i64())
    .unwrap_or(0)
}

/// Writes a product attribute value (EAV upsert).
/// Writes a product attribute value (EAV upsert on
/// `(channel, locale, attribute_id, product_id)`).
pub async fn set_attribute(product_id: i64, code: &str, value: &Value) {
    set_attribute_in(product_id, code, value, None).await;
}

/// Escribe un valor de atributo. Los atributos `value_per_locale = 1` se
/// guardan bajo el locale indicado (o el por defecto si no se indica);
/// el resto son globales (`locale`/`channel` NULL).
pub async fn set_attribute_in(product_id: i64, code: &str, value: &Value, locale: Option<&str>) {
    let attribute_id = attr_id_by_code(code).await;
    if attribute_id == 0 {
        return;
    }
    let attribute = db::find_by_id("attributes", attribute_id).await.ok().flatten();
    let attribute_type = attribute
        .as_ref()
        .and_then(|row| row["type"].as_str())
        .unwrap_or("text")
        .to_string();
    let per_locale = attribute
        .as_ref()
        .and_then(|row| row["value_per_locale"].as_i64())
        .unwrap_or(0)
        == 1;
    let attribute_locale = if per_locale {
        Some(locale.unwrap_or("en").to_string())
    } else {
        None
    };

    let mut row = db::to_map(json!({"product_id": product_id, "attribute_id": attribute_id, "updated_at": now()}));
    // Índice directo de Map panica si la clave no existe: usar insert.
    match attribute_type.as_str() {
        "boolean" => row.insert("boolean_value".into(), json!(value.as_bool().unwrap_or(false) as i64)),
        "integer" | "select" => row.insert("integer_value".into(), value.clone()),
        "price" => row.insert("float_value".into(), value.clone()),
        "date" => row.insert("date_value".into(), value.clone()),
        "datetime" => row.insert("datetime_value".into(), value.clone()),
        "multiselect" | "checkbox" => row.insert("json_value".into(), value.clone()),
        _ => row.insert("text_value".into(), value.clone()),
    };

    let existing = match &attribute_locale {
        // `IS ?` con placeholder no es portable a MySQL: ramificar.
        Some(locale_code) => {
            db::select_one(
                "SELECT `id` FROM `product_attribute_values` WHERE `product_id` = ? AND `attribute_id` = ? \
                 AND `channel` IS NULL AND `locale` = ? LIMIT 1",
                &[
                    Value::from(product_id),
                    Value::from(attribute_id),
                    Value::String(locale_code.clone()),
                ],
            )
            .await
            .ok()
            .flatten()
        }
        None => {
            db::select_one(
                "SELECT `id` FROM `product_attribute_values` WHERE `product_id` = ? AND `attribute_id` = ? \
                 AND `channel` IS NULL AND `locale` IS NULL LIMIT 1",
                &[Value::from(product_id), Value::from(attribute_id)],
            )
            .await
            .ok()
            .flatten()
        }
    };
    if let Some(locale_code) = attribute_locale {
        row.insert("locale".into(), json!(locale_code));
    }
    match existing {
        Some(existing) => {
            let _ = db::update(
                "product_attribute_values",
                existing["id"].as_i64().unwrap_or(0),
                &row,
            )
            .await;
        }
        None => {
            row.insert("created_at".into(), json!(now()));
            let _ = db::insert("product_attribute_values", &row).await;
        }
    }
}

/// Refreshes `product_flat` for a product (per channel/locale rows exist only
/// for the default channel/locale in phase 1).
pub async fn refresh_flat(product_id: i64) {
    let product = match db::find_by_id("products", product_id).await {
        Ok(Some(product)) => product,
        _ => return,
    };
    let quantity = cart::available_quantity(product_id, 1).await;
    let images_count = db::scalar_i64(
        "SELECT COUNT(*) FROM `product_images` WHERE `product_id` = ?",
        &[Value::from(product_id)],
    )
    .await
    .unwrap_or(0);

    // Una fila de product_flat por locale habilitado del canal; el contenido
    // traducido del locale con fallback al global (attribute_map_in).
    let channel_id: i64 = db::scalar_i64(
        "SELECT `id` FROM `channels` ORDER BY `id` LIMIT 1",
        &[],
    )
    .await
    .unwrap_or(1);
    let locale_codes: Vec<String> = match core::channel_locales(channel_id).await {
        Ok(locales) if !locales.is_empty() => locales
            .iter()
            .map(|locale| locale["code"].as_str().unwrap_or("en").to_string())
            .collect(),
        _ => vec!["en".to_string()],
    };

    for locale_code in &locale_codes {
        let attributes = cart::attribute_map_in(product_id, Some(locale_code))
            .await
            .unwrap_or_default();

        let existing = db::select_one(
            "SELECT `id` FROM `product_flat` WHERE `product_id` = ? AND `locale` = ? AND `channel` = 'default' LIMIT 1",
            &[Value::from(product_id), Value::String(locale_code.clone())],
        )
        .await
        .ok()
        .flatten();

        // Booleans/numerics get Bagisto defaults; NULLs would violate the NOT
        // NULL constraints of product_flat.
        let flag = |code: &str, default: i64| attributes[code].as_i64().unwrap_or(default);
        let flat = json!({
            "sku": product["sku"].clone(),
            "type": product["type"].clone(),
            "name": attributes["name"].clone(),
            "short_description": attributes["short_description"].clone(),
            "description": attributes["description"].clone(),
            "url_key": attributes["url_key"].clone(),
            "new": flag("new", 0),
            "featured": flag("featured", 0),
            "status": flag("status", 1),
            "meta_title": attributes["meta_title"].clone(),
            "meta_keywords": attributes["meta_keywords"].clone(),
            "meta_description": attributes["meta_description"].clone(),
            "price": attributes["price"].as_f64().unwrap_or(0.0),
            "special_price": attributes["special_price"].clone(),
            "special_price_from": attributes["special_price_from"].clone(),
            "special_price_to": attributes["special_price_to"].clone(),
            "weight": attributes["weight"].as_f64().unwrap_or(0.0),
            "quantity": quantity,
            "images_count": images_count,
            "manage_stock": flag("manage_stock", 1),
            "locale": locale_code,
            "channel": "default",
            "product_id": product_id,
            "parent_id": product["parent_id"].clone(),
            // Variants are never individually visible.
            "visible_individually": if product["parent_id"].is_null() {
                flag("visible_individually", 1)
            } else {
                0
            },
            "updated_at": now(),
        });

        match existing {
            Some(row) => {
                if let Err(error) = db::update("product_flat", row["id"].as_i64().unwrap_or(0), &flat).await {
                    eprintln!("refresh_flat update failed: {error}");
                }
            }
            None => {
                let mut flat = db::to_map(flat);
                flat.insert("created_at".into(), json!(now()));
                if let Err(error) = db::insert("product_flat", &flat).await {
                    eprintln!("refresh_flat insert failed: {error}");
                }
            }
        }
    }

    // Price index (independiente del locale): valores globales.
    let attributes = cart::attribute_map(product_id).await.unwrap_or_default();

    // Price index for the guest + general groups.
    let price = cart::effective_price(&attributes);
    let regular = attributes["price"].as_f64().unwrap_or(0.0);
    for group in ["guest", "general"] {
        let group_id = db::select_one(
            "SELECT `id` FROM `customer_groups` WHERE `code` = ? LIMIT 1",
            &[Value::String(group.to_string())],
        )
        .await
        .ok()
        .flatten()
        .and_then(|row| row["id"].as_i64());
        let Some(group_id) = group_id else { continue };
        let existing = db::select_one(
            "SELECT `id` FROM `product_price_indices` WHERE `product_id` = ? AND `customer_group_id` = ? LIMIT 1",
            &[Value::from(product_id), Value::from(group_id)],
        )
        .await
        .ok()
        .flatten();
        let index = json!({
            "product_id": product_id,
            "customer_group_id": group_id,
            "min_price": price,
            "regular_min_price": regular,
            "max_price": price,
            "regular_max_price": regular,
            "updated_at": now(),
        });
        match existing {
            Some(row) => {
                let _ = db::update("product_price_indices", row["id"].as_i64().unwrap_or(0), &index).await;
            }
            None => {
                let mut index = db::to_map(index);
                index.insert("created_at".into(), json!(now()));
                let _ = db::insert("product_price_indices", &index).await;
            }
        }
    }

    // Catalog rules rebajan los índices de precio por grupo.
    super::catalog_rules::apply_to_indices(product_id, price).await;

    // Inventory index.
    let existing = db::select_one(
        "SELECT `id` FROM `product_inventory_indices` WHERE `product_id` = ? LIMIT 1",
        &[Value::from(product_id)],
    )
    .await
    .ok()
    .flatten();
    let index = json!({"product_id": product_id, "channel_id": 1, "qty": quantity, "updated_at": now()});
    match existing {
        Some(row) => {
            let _ = db::update("product_inventory_indices", row["id"].as_i64().unwrap_or(0), &index).await;
        }
        None => {
            let mut index = db::to_map(index);
            index.insert("created_at".into(), json!(now()));
            let _ = db::insert("product_inventory_indices", &index).await;
        }
    }
}
