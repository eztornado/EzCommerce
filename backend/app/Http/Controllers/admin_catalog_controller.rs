//! Admin catalog: products, categories, attributes, families (JSON CRUD +
//! DataGrid). Mirrors `Webkul\Admin\Http\Controllers\Catalog\*`.

use std::collections::HashMap;

use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::{HeaderMap, StatusCode};
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Map, Value};

use crate::support::auth;
use crate::support::cart;
use crate::support::datagrid;
use crate::support::db;
use crate::support::response::{csv_response, message, resource, validation_error};

pub struct AdminCatalogController;

fn now() -> String {
    db::now_db()
}

async fn attr_id(code: &str) -> i64 {
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





pub fn products_grid() -> datagrid::DataGrid {
    datagrid::DataGrid {
        table: "products",
        alias: "p",
        primary: "p.id",
        columns: vec![
            datagrid::Column { index: "id", label: "ID", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("p.id"), options: None },
            datagrid::Column { index: "sku", label: "SKU", kind: "string", searchable: true, filterable: true, sortable: true, source: Some("p.sku"), options: None },
            datagrid::Column { index: "name", label: "Name", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("pf.name"), options: None },
            datagrid::Column { index: "type", label: "Type", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("p.type"), options: Some(json!([
                {"label": "Simple", "value": "simple"}, {"label": "Virtual", "value": "virtual"},
                {"label": "Configurable", "value": "configurable"}, {"label": "Grouped", "value": "grouped"},
                {"label": "Bundle", "value": "bundle"}, {"label": "Downloadable", "value": "downloadable"},
            ])) },
            datagrid::Column { index: "quantity", label: "Quantity", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("pfi.qty"), options: None },
            datagrid::Column { index: "price", label: "Price", kind: "decimal", searchable: false, filterable: true, sortable: true, source: Some("pf.price"), options: None },
            datagrid::Column { index: "status", label: "Active", kind: "boolean", searchable: false, filterable: true, sortable: true, source: Some("pf.status"), options: Some(json!([
                {"label": "Active", "value": 1}, {"label": "Inactive", "value": 0},
            ])) },
        ],
        joins: vec![
            "LEFT JOIN `product_flat` `pf` ON `pf`.`product_id` = `p`.`id`".into(),
            "LEFT JOIN `product_inventory_indices` `pfi` ON `pfi`.`product_id` = `p`.`id`".into(),
        ],
        where_clauses: vec![("`p`.`parent_id` IS NULL".to_string(), Vec::new())],
        actions: json!([
            {"title": "Edit", "method": "GET", "url": "/admin/catalog/products/edit/{id}"},
            {"title": "Delete", "method": "DELETE", "url": "/api/admin/catalog/products/{id}"},
        ]),
        mass_actions: json!([
            {"title": "Delete", "method": "POST", "url": "/api/admin/catalog/products/mass-delete", "action": "delete"},
        ]),
        formatters: Vec::new(),
    }
}

impl AdminCatalogController {
    /// `POST /api/admin/catalog/products` - datagrid (Bagisto uses POST with
    /// filters for grids). `?format=csv` exporta el grid completo.
    pub async fn products_grid(
        admin: auth::Admin,
        Query(query): Query<HashMap<String, String>>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let request = datagrid::parse_request(&body);
        let grid = products_grid();
        if query.get("format").map(String::as_str) == Some("csv") {
            return match grid.process_csv(&request).await {
                Ok(csv) => csv_response(&csv, "products.csv"),
                Err(error) => error.into(),
            };
        }
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/catalog/products/create` - `{sku, type, attribute_family_id?}`.
    pub async fn store_product(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(sku) = body["sku"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("sku".into(), "The sku field is required.".into())]);
        };
        let product_type = body["type"].as_str().unwrap_or("simple").to_string();
        let family = body["attribute_family_id"].as_i64().unwrap_or_else(|| 1);

        let exists = db::select_one(
            "SELECT `id` FROM `products` WHERE `sku` = ? LIMIT 1",
            &[Value::String(sku.to_string())],
        )
        .await
        .ok()
        .flatten();
        if exists.is_some() {
            return validation_error(vec![("sku".into(), "The sku has already been taken.".into())]);
        }

        let product_id = match db::insert(
            "products",
            &json!({
                "sku": sku,
                "type": product_type,
                "attribute_family_id": family,
                "created_at": now(),
                "updated_at": now(),
            })
            ,
        )
        .await
        {
            Ok(id) => id,
            Err(error) => return error.into(),
        };

        // Super attributes de los configurables ({super_attributes: [id...]}).
        for attribute_id in body["super_attributes"].as_array().cloned().unwrap_or_default() {
            if let Some(attribute_id) = attribute_id.as_i64() {
                let _ = db::execute(
                    "INSERT OR IGNORE INTO `product_super_attributes` (`product_id`, `attribute_id`) VALUES (?, ?)",
                    &[Value::from(product_id), Value::from(attribute_id)],
                )
                .await;
            }
        }

        crate::support::products::refresh_flat(product_id).await;
        resource(json!({"id": product_id}), "The product has been created successfully.")
    }

    /// `GET /api/admin/catalog/attributes/configurable` - atributos que pueden
    /// actuar como super attributes de un configurable.
    pub async fn configurable_attributes(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT `id`, `code`, `admin_name` FROM `attributes` WHERE `is_configurable` = 1 ORDER BY `id`",
            &[],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `GET /api/admin/catalog/attributes/all` - todos los atributos con su
    /// tipo (para el editor de condiciones de reglas).
    pub async fn all_attributes(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT `id`, `code`, `admin_name`, `type` FROM `attributes` \
             WHERE `type` IN ('select', 'multiselect', 'text', 'price', 'number') \
             ORDER BY `id`",
            &[],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `GET /api/admin/catalog/products/edit/{id}` - product with all data for
    /// the editor (attributes, categories, inventories, images, variants).
    pub async fn show_product(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let product = match db::find_by_id("products", id).await {
            Ok(Some(product)) => product,
            _ => return message(StatusCode::NOT_FOUND, "Product not found."),
        };
        let attributes = cart::attribute_map(id).await.unwrap_or_default();
        let categories = db::select(
            "SELECT `category_id` FROM `product_categories` WHERE `product_id` = ?",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        let inventories = db::select(
            "SELECT `pi`.`inventory_source_id`, `pi`.`qty`, `isrc`.`name` FROM `product_inventories` `pi` \
             LEFT JOIN `inventory_sources` `isrc` ON `isrc`.`id` = `pi`.`inventory_source_id` \
             WHERE `pi`.`product_id` = ?",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        let images = db::select(
            "SELECT `id`, `path`, `position` FROM `product_images` WHERE `product_id` = ? ORDER BY `position`, `id`",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        // Super attributes (configurables) con sus opciones para el editor.
        let mut super_attributes = db::select(
            "SELECT `a`.`id`, `a`.`code`, `a`.`admin_name`, `a`.`type` FROM `product_super_attributes` `psa` \
             INNER JOIN `attributes` `a` ON `a`.`id` = `psa`.`attribute_id` \
             WHERE `psa`.`product_id` = ? ORDER BY `a`.`id`",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        for attribute in super_attributes.iter_mut() {
            let options = db::select(
                "SELECT `id`, `admin_name` FROM `attribute_options` WHERE `attribute_id` = ? ORDER BY `sort_order`, `id`",
                &[attribute["id"].clone()],
            )
            .await
            .unwrap_or_default();
            attribute.insert(
                "options".into(),
                Value::Array(options.into_iter().map(Value::Object).collect()),
            );
        }

        // Variantes con sus valores de super attributes y stock total.
        let variant_rows = db::select(
            "SELECT `id`, `sku` FROM `products` WHERE `parent_id` = ? ORDER BY `id`",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        let mut variants = Vec::new();
        for row in variant_rows {
            let variant_id = row["id"].as_i64().unwrap_or(0);
            let mut variant = Map::new();
            variant.insert("id".into(), row["id"].clone());
            variant.insert("sku".into(), row["sku"].clone());
            variant.insert(
                "attributes".into(),
                cart::attribute_map(variant_id).await.unwrap_or_default(),
            );
            variant.insert(
                "qty".into(),
                Value::from(
                    db::scalar_i64(
                        "SELECT COALESCE(SUM(`qty`), 0) FROM `product_inventories` WHERE `product_id` = ?",
                        &[Value::from(variant_id)],
                    )
                    .await
                    .unwrap_or(0),
                ),
            );
            variants.push(Value::Object(variant));
        }

        resource(
            json!({
                "product": product,
                "attributes": attributes,
                "categories": categories,
                "inventories": inventories,
                "images": images,
                "super_attributes": super_attributes,
                "variants": variants,
            }),
            Value::Null,
        )
    }

    /// `PUT /api/admin/catalog/products/edit/{id}` - update attributes +
    /// categories + inventories in one call.
    pub async fn update_product(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let exists = db::find_by_id("products", id).await.ok().flatten();
        if exists.is_none() {
            return message(StatusCode::NOT_FOUND, "Product not found.");
        }

        if let Some(sku) = body["sku"].as_str() {
            let _ = db::execute(
                "UPDATE `products` SET `sku` = ?, `updated_at` = ? WHERE `id` = ?",
                &[Value::String(sku.to_string()), Value::String(now()), Value::from(id)],
            )
            .await;
        }

        // Attributes payload: {attributes: {code: value}, locale?}. Con
        // `locale` los atributos traducibles se escriben para ese idioma.
        let locale = body["locale"].as_str();
        if let Some(attributes) = body["attributes"].as_object() {
            for (code, value) in attributes {
                crate::support::products::set_attribute_in(id, code, value, locale).await;
            }
        }

        // Categories: replace pivots.
        if let Some(categories) = body["categories"].as_array() {
            let _ = db::execute(
                "DELETE FROM `product_categories` WHERE `product_id` = ?",
                &[Value::from(id)],
            )
            .await;
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

        // Images: {images: [{path: "product/uuid.png"}]} -> replace all.
        if let Some(images) = body["images"].as_array() {
            let _ = db::execute(
                "DELETE FROM `product_images` WHERE `product_id` = ?",
                &[Value::from(id)],
            )
            .await;
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
                    })
                    ,
                )
                .await;
            }
        }

        // Inventories: {inventories: {source_id: qty}}.
        if let Some(inventories) = body["inventories"].as_object() {
            for (source, qty) in inventories {
                let Some(source_id) = source.parse::<i64>().ok() else { continue };
                let quantity = qty.as_i64().unwrap_or(0);
                let existing = db::select_one(
                    "SELECT `id` FROM `product_inventories` WHERE `product_id` = ? AND `inventory_source_id` = ? LIMIT 1",
                    &[Value::from(id), Value::from(source_id)],
                )
                .await
                .ok()
                .flatten();
                match existing {
                    Some(row) => {
                        let _ = db::update(
                            "product_inventories",
                            row["id"].as_i64().unwrap_or(0),
                            &json!({"qty": quantity, "updated_at": now()}),
                        )
                        .await;
                    }
                    None => {
                        let _ = db::insert(
                            "product_inventories",
                            &json!({
                                "product_id": id,
                                "inventory_source_id": source_id,
                                "qty": quantity,
                                "created_at": now(),
                                "updated_at": now(),
                            })
                            ,
                        )
                        .await;
                    }
                }
            }
        }

        crate::support::products::refresh_flat(id).await;
        resource(json!({"id": id}), "The product has been updated successfully.")
    }

    /// `DELETE /api/admin/catalog/products/{id}`.
    pub async fn delete_product(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match db::delete("products", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The product has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/catalog/products/mass-delete` - `{indices: []}`.
    pub async fn mass_delete_products(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        for id in body["indices"].as_array().cloned().unwrap_or_default() {
            if let Some(id) = id.as_i64() {
                let _ = db::delete("products", id).await;
            }
        }
        resource(Value::Array(Vec::new()), "The selected products have been deleted.")
    }

    /// `POST /api/admin/catalog/products/edit/{id}/variants` - crea una
    /// variante del configurable: `{sku, attributes: {<super code>: value,
    /// name?, price?, weight?}, qty?}`. Los códigos nuevos se registran como
    /// super attributes del padre.
    pub async fn store_variant(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let parent = db::find_by_id("products", id).await.ok().flatten();
        let Some(parent) = parent else {
            return message(StatusCode::NOT_FOUND, "Product not found.");
        };

        let Some(sku) = body["sku"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("sku".into(), "The sku field is required.".into())]);
        };
        let exists = db::select_one(
            "SELECT `id` FROM `products` WHERE `sku` = ? LIMIT 1",
            &[Value::String(sku.to_string())],
        )
        .await
        .ok()
        .flatten();
        if exists.is_some() {
            return validation_error(vec![("sku".into(), "The sku has already been taken.".into())]);
        }

        let variant_id = match db::insert(
            "products",
            &json!({
                "sku": sku,
                "type": "simple",
                "parent_id": id,
                "attribute_family_id": parent["attribute_family_id"].clone(),
                "created_at": now(),
                "updated_at": now(),
            }),
        )
        .await
        {
            Ok(variant_id) => variant_id,
            Err(error) => return error.into(),
        };

        let mut attributes = body["attributes"].as_object().cloned().unwrap_or_default();
        attributes.insert("sku".into(), Value::String(sku.to_string()));
        attributes.insert("status".into(), Value::from(1));
        attributes.insert("manage_stock".into(), Value::from(1));
        for (code, value) in &attributes {
            crate::support::products::set_attribute(variant_id, code, value).await;
        }
        // Los super attributes del configurable se gestionan aparte: aquí solo
        // se rellenan los valores de los ya registrados (la UI ofrece sus
        // opciones).

        if let Some(qty) = body["qty"].as_i64() {
            let _ = db::insert(
                "product_inventories",
                &json!({
                    "product_id": variant_id,
                    "inventory_source_id": 1,
                    "qty": qty,
                    "created_at": now(),
                    "updated_at": now(),
                }),
            )
            .await;
        }

        crate::support::products::refresh_flat(variant_id).await;
        crate::support::products::refresh_flat(id).await;
        resource(json!({"id": variant_id}), "The variant has been created successfully.")
    }

    /// `DELETE /api/admin/catalog/products/variants/{id}` - borra la variante
    /// y refresca el flat del padre.
    pub async fn delete_variant(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let variant = db::find_by_id("products", id).await.ok().flatten();
        let Some(variant) = variant else {
            return message(StatusCode::NOT_FOUND, "Variant not found.");
        };
        let parent_id = variant["parent_id"].as_i64().unwrap_or(0);
        match db::delete("products", id).await {
            Ok(_) => {
                if parent_id > 0 {
                    crate::support::products::refresh_flat(parent_id).await;
                }
                resource(Value::Array(Vec::new()), "The variant has been deleted successfully.")
            }
            Err(error) => error.into(),
        }
    }

    // ------------------------------------------------------------------
    // Categories
    // ------------------------------------------------------------------

    /// `POST /api/admin/catalog/categories` - datagrid.
    pub async fn categories_grid(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let grid = datagrid::DataGrid {
            table: "categories",
            alias: "c",
            primary: "c.id",
            columns: vec![
                datagrid::Column { index: "id", label: "ID", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("c.id"), options: None },
                datagrid::Column { index: "name", label: "Name", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("ct.name"), options: None },
                datagrid::Column { index: "position", label: "Position", kind: "integer", searchable: false, filterable: false, sortable: true, source: Some("c.position"), options: None },
                datagrid::Column { index: "status", label: "Active", kind: "boolean", searchable: false, filterable: true, sortable: true, source: Some("c.status"), options: Some(json!([
                    {"label": "Active", "value": 1}, {"label": "Inactive", "value": 0},
                ])) },
            ],
            joins: vec!["LEFT JOIN `category_translations` `ct` ON `ct`.`category_id` = `c`.`id`".into()],
            where_clauses: vec![("`c`.`parent_id` IS NOT NULL".to_string(), Vec::new())],
            actions: json!([
                {"title": "Edit", "method": "GET", "url": "/admin/catalog/categories/edit/{id}"},
                {"title": "Delete", "method": "DELETE", "url": "/api/admin/catalog/categories/{id}"},
            ]),
            mass_actions: Value::Null,
            formatters: Vec::new(),
        };
        let request = datagrid::parse_request(&body);
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/catalog/categories/tree`.
    pub async fn categories_tree(admin: auth::Admin, Query(query): Query<HashMap<String, String>>) -> Response {
        let _ = admin;
        let locale = query.get("locale").cloned().unwrap_or_else(|| "en".to_string());
        let tree = categories_tree_from(1, &locale).await;
        resource(json!(tree), Value::Null)
    }

    /// `POST /api/admin/catalog/categories/store` - create with translation.
    pub async fn store_category(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(name) = body["name"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("name".into(), "The name field is required.".into())]);
        };
        let parent_id = body["parent_id"].as_i64().unwrap_or(1);
        let parent = db::find_by_id("categories", parent_id).await.ok().flatten();
        let parent_rgt = parent
            .as_ref()
            .and_then(|row| row["_rgt"].as_i64())
            .unwrap_or(1);

        // Nested-set insert: shift right boundary.
        let _ = db::execute(
            "UPDATE `categories` SET `_lft` = `_lft` + 2 WHERE `_lft` >= ?",
            &[Value::from(parent_rgt)],
        )
        .await;
        let _ = db::execute(
            "UPDATE `categories` SET `_rgt` = `_rgt` + 2 WHERE `_rgt` >= ?",
            &[Value::from(parent_rgt)],
        )
        .await;

        let slug = crate::support::slug::slugify(name);
        let category_id = match db::insert(
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
            })
            ,
        )
        .await
        {
            Ok(id) => id,
            Err(error) => return error.into(),
        };

        let _ = db::insert(
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
            })
            ,
        )
        .await;

        resource(json!({"id": category_id}), "The category has been created successfully.")
    }

    /// `PUT /api/admin/catalog/categories/edit/{id}`.
    pub async fn update_category(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Query(query): Query<HashMap<String, String>>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let locale = query.get("locale").cloned().unwrap_or_else(|| "en".to_string());
        let _ = db::execute(
            "UPDATE `categories` SET `position` = ?, `status` = ?, `updated_at` = ? WHERE `id` = ?",
            &[
                Value::from(body["position"].as_i64().unwrap_or(0)),
                Value::from(body["status"].as_i64().unwrap_or(1)),
                Value::String(now()),
                Value::from(id),
            ],
        )
        .await;
        let translation = json!({
            "name": body["name"].clone(),
            "slug": crate::support::slug::slugify(body["name"].as_str().unwrap_or_default()),
            "description": body["description"].clone(),
            "meta_title": body["meta_title"].clone(),
            "meta_description": body["meta_description"].clone(),
            "updated_at": now(),
        });
        let existing = db::select_one(
            "SELECT `id` FROM `category_translations` WHERE `category_id` = ? AND `locale` = ? LIMIT 1",
            &[Value::from(id), Value::String(locale.clone())],
        )
        .await
        .ok()
        .flatten();
        match existing {
            Some(row) => {
                let _ = db::update(
                    "category_translations",
                    row["id"].as_i64().unwrap_or(0),
                    &translation,
                )
                .await;
            }
            None => {
                let mut translation = db::to_map(translation);
                translation.insert("category_id".into(), json!(id));
                translation.insert("locale".into(), json!(locale));
                translation.insert("created_at".into(), json!(now()));
                let _ = db::insert("category_translations", &translation).await;
            }
        }
        resource(json!({"id": id}), "The category has been updated successfully.")
    }

    /// `DELETE /api/admin/catalog/categories/{id}` (root protected).
    pub async fn delete_category(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        if id == 1 {
            return message(StatusCode::FORBIDDEN, "The root category cannot be deleted.");
        }
        match db::delete("categories", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The category has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    // ------------------------------------------------------------------
    // Attributes
    // ------------------------------------------------------------------

    /// `POST /api/admin/catalog/attributes` - datagrid.
    pub async fn attributes_grid(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let grid = datagrid::DataGrid {
            table: "attributes",
            alias: "a",
            primary: "a.id",
            columns: vec![
                datagrid::Column { index: "id", label: "ID", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("a.id"), options: None },
                datagrid::Column { index: "code", label: "Code", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("a.code"), options: None },
                datagrid::Column { index: "admin_name", label: "Name", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("a.admin_name"), options: None },
                datagrid::Column { index: "type", label: "Type", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("a.type"), options: None },
                datagrid::Column { index: "is_required", label: "Required", kind: "boolean", searchable: false, filterable: true, sortable: true, source: Some("a.is_required"), options: None },
            ],
            joins: Vec::new(),
            where_clauses: Vec::new(),
            actions: Value::Null,
            mass_actions: Value::Null,
            formatters: Vec::new(),
        };
        let request = datagrid::parse_request(&body);
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/catalog/attributes/edit/{id}` - atributo + opciones.
    pub async fn show_attribute(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let attribute = match db::find_by_id("attributes", id).await {
            Ok(Some(attribute)) => attribute,
            _ => return message(StatusCode::NOT_FOUND, "Attribute not found."),
        };
        let options = db::select(
            "SELECT `id`, `admin_name`, `swatch_value`, `sort_order` FROM `attribute_options` \
             WHERE `attribute_id` = ? ORDER BY `sort_order`, `id`",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        resource(
            json!({
                "attribute": attribute,
                "options": options,
            }),
            Value::Null,
        )
    }

    fn attribute_row(body: &Value) -> Value {
        json!({
            "admin_name": body["admin_name"].clone(),
            "type": body["type"].as_str().unwrap_or("text"),
            "position": body["position"].as_i64().unwrap_or(0),
            "is_required": body["is_required"].as_i64().unwrap_or(0),
            "is_unique": body["is_unique"].as_i64().unwrap_or(0),
            "is_filterable": body["is_filterable"].as_i64().unwrap_or(0),
            "is_comparable": body["is_comparable"].as_i64().unwrap_or(0),
            "is_configurable": body["is_configurable"].as_i64().unwrap_or(0),
            "is_visible_on_front": body["is_visible_on_front"].as_i64().unwrap_or(0),
            "value_per_locale": body["value_per_locale"].as_i64().unwrap_or(0),
            "value_per_channel": body["value_per_channel"].as_i64().unwrap_or(0),
            "updated_at": now(),
        })
    }

    /// Reemplaza las opciones de un atributo select (`options: [{admin_name}]`).
    async fn replace_attribute_options(attribute_id: i64, body: &Value) {
        let Some(options) = body["options"].as_array() else { return };
        let _ = db::execute(
            "DELETE FROM `attribute_options` WHERE `attribute_id` = ?",
            &[Value::from(attribute_id)],
        )
        .await;
        for (position, option) in options.iter().enumerate() {
            let Some(name) = option["admin_name"].as_str().map(str::trim).filter(|s| !s.is_empty()) else { continue };
            let _ = db::insert(
                "attribute_options",
                &json!({
                    "attribute_id": attribute_id,
                    "admin_name": name,
                    "swatch_value": option["swatch_value"].clone(),
                    "sort_order": position as i64,
                    "created_at": now(),
                    "updated_at": now(),
                }),
            )
            .await;
        }
    }

    /// `POST /api/admin/catalog/attributes/store` - `{code, admin_name, type,
    /// flags..., options?}`.
    pub async fn store_attribute(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(code) = body["code"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("code".into(), "The code field is required.".into())]);
        };
        let taken = db::select_one(
            "SELECT `id` FROM `attributes` WHERE `code` = ? LIMIT 1",
            &[Value::String(code.to_string())],
        )
        .await
        .ok()
        .flatten();
        if taken.is_some() {
            return validation_error(vec![("code".into(), "The code has already been taken.".into())]);
        }
        let mut row = db::to_map(Self::attribute_row(&body));
        row.insert("code".into(), json!(code));
        row.insert("is_user_defined".into(), json!(1));
        row.insert("created_at".into(), json!(now()));
        let attribute_id = match db::insert("attributes", &Value::Object(row)).await {
            Ok(attribute_id) => attribute_id,
            Err(error) => return error.into(),
        };
        Self::replace_attribute_options(attribute_id, &body).await;
        resource(json!({"id": attribute_id}), "The attribute has been created successfully.")
    }

    /// `PUT /api/admin/catalog/attributes/edit/{id}` - solo user-defined.
    pub async fn update_attribute(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let attribute = db::find_by_id("attributes", id).await.ok().flatten();
        let Some(attribute) = attribute else {
            return message(StatusCode::NOT_FOUND, "Attribute not found.");
        };
        if attribute["is_user_defined"].as_i64().unwrap_or(1) == 0 {
            return message(
                StatusCode::UNPROCESSABLE_ENTITY,
                "System attributes cannot be modified.",
            );
        }
        match db::update("attributes", id, &Self::attribute_row(&body)).await {
            Ok(_) => {}
            Err(error) => return error.into(),
        }
        Self::replace_attribute_options(id, &body).await;
        resource(Value::Array(Vec::new()), "The attribute has been updated successfully.")
    }

    /// `DELETE /api/admin/catalog/attributes/{id}` - solo user-defined.
    pub async fn destroy_attribute(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let attribute = db::find_by_id("attributes", id).await.ok().flatten();
        let Some(attribute) = attribute else {
            return message(StatusCode::NOT_FOUND, "Attribute not found.");
        };
        if attribute["is_user_defined"].as_i64().unwrap_or(1) == 0 {
            return message(
                StatusCode::UNPROCESSABLE_ENTITY,
                "System attributes cannot be deleted.",
            );
        }
        match db::delete("attributes", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The attribute has been deleted successfully."),
            Err(error) => error.into(),
        }
    }
}

fn categories_tree_from(
    parent_id: i64,
    locale: &str,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<Value>> + Send + '_>> {
    Box::pin(async move {
        let children = db::select(
            "SELECT `c`.`id`, `c`.`position`, `c`.`status`, `ct`.`name`, `ct`.`slug` \
             FROM `categories` `c` \
             LEFT JOIN `category_translations` `ct` ON `ct`.`category_id` = `c`.`id` AND `ct`.`locale` = ? \
             WHERE `c`.`parent_id` = ? ORDER BY `c`.`position`, `c`.`id`",
            &[Value::String(locale.to_string()), Value::from(parent_id)],
        )
        .await
        .unwrap_or_default();
        let mut payload = Vec::new();
        for child in children {
            let id = child["id"].as_i64().unwrap_or(0);
            payload.push(json!({
                "id": id,
                "name": child["name"],
                "slug": child["slug"],
                "children": categories_tree_from(id, locale).await,
            }));
        }
        payload
    })
}
