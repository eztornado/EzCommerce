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
            "LEFT JOIN `product_flat` `pf` ON `pf`.`product_id` = `p`.`id` AND `pf`.`locale` = 'en'".into(),
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
        match crate::support::admin_crud::create_product(&body).await {
            Ok(product_id) => resource(
                json!({"id": product_id}),
                "The product has been created successfully.",
            ),
            Err(error) => error.into(),
        }
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

    /// `GET /api/admin/catalog/families` - familias de atributos.
    pub async fn families(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT * FROM `attribute_families` ORDER BY `id`",
            &[],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `POST /api/admin/catalog/families/store` - `{code, name}`.
    pub async fn store_family(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(code) = body["code"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("code".into(), "The code field is required.".into())]);
        };
        let taken = db::select_one(
            "SELECT `id` FROM `attribute_families` WHERE `code` = ? LIMIT 1",
            &[Value::String(code.to_string())],
        )
        .await
        .ok()
        .flatten();
        if taken.is_some() {
            return validation_error(vec![("code".into(), "The code has already been taken.".into())]);
        }
        match db::insert(
            "attribute_families",
            &json!({
                "code": code,
                "name": body["name"].as_str().unwrap_or_default().trim(),
                "status": 1,
                "is_user_defined": 1,
                "created_at": now(),
                "updated_at": now(),
            }),
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The attribute family has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/admin/catalog/families/edit/{id}` - solo user-defined.
    pub async fn update_family(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let family = db::find_by_id("attribute_families", id).await.ok().flatten();
        let Some(family) = family else {
            return message(StatusCode::NOT_FOUND, "Family not found.");
        };
        if family["is_user_defined"].as_i64().unwrap_or(1) == 0 {
            return message(StatusCode::UNPROCESSABLE_ENTITY, "The default family cannot be modified.");
        }
        match db::update(
            "attribute_families",
            id,
            &json!({"name": body["name"].as_str().unwrap_or_default().trim(), "updated_at": now()}),
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The attribute family has been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/catalog/families/{id}` - solo user-defined y sin
    /// productos asignados.
    pub async fn destroy_family(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let family = db::find_by_id("attribute_families", id).await.ok().flatten();
        let Some(family) = family else {
            return message(StatusCode::NOT_FOUND, "Family not found.");
        };
        if family["is_user_defined"].as_i64().unwrap_or(1) == 0 {
            return message(StatusCode::UNPROCESSABLE_ENTITY, "The default family cannot be deleted.");
        }
        let in_use = db::scalar_i64(
            "SELECT COUNT(*) FROM `products` WHERE `attribute_family_id` = ?",
            &[Value::from(id)],
        )
        .await
        .unwrap_or(0);
        if in_use > 0 {
            return validation_error(vec![(
                "id".into(),
                "The family has products assigned and cannot be deleted.".into(),
            )]);
        }
        match db::delete("attribute_families", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The attribute family has been deleted successfully."),
            Err(error) => error.into(),
        }
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
    pub async fn show_product(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Query(query): Query<HashMap<String, String>>,
    ) -> Response {
        let _ = admin;
        let product = match db::find_by_id("products", id).await {
            Ok(Some(product)) => product,
            _ => return message(StatusCode::NOT_FOUND, "Product not found."),
        };
        // Contenido del idioma pedido (fallback al global/por defecto).
        let locale = query
            .get("locale")
            .map(String::as_str)
            .filter(|code| !code.is_empty() && *code != "en");
        let attributes = cart::attribute_map_in(id, locale).await.unwrap_or_default();
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
                        "SELECT COALESCE(CAST(SUM(`qty`) AS SIGNED), 0) FROM `product_inventories` WHERE `product_id` = ?",
                        &[Value::from(variant_id)],
                    )
                    .await
                    .unwrap_or(0),
                ),
            );
            variants.push(Value::Object(variant));
        }

        // Linked products (related/up-sell/cross-sell) para el editor.
        let links = json!({
            "related": db::select(
                "SELECT `child_id` FROM `product_relations` WHERE `parent_id` = ?",
                &[Value::from(id)],
            )
            .await
            .unwrap_or_default()
            .iter()
            .filter_map(|row| row["child_id"].as_i64())
            .collect::<Vec<_>>(),
            "up_sells": db::select(
                "SELECT `child_id` FROM `product_up_sells` WHERE `parent_id` = ?",
                &[Value::from(id)],
            )
            .await
            .unwrap_or_default()
            .iter()
            .filter_map(|row| row["child_id"].as_i64())
            .collect::<Vec<_>>(),
            "cross_sells": db::select(
                "SELECT `child_id` FROM `product_cross_sells` WHERE `parent_id` = ?",
                &[Value::from(id)],
            )
            .await
            .unwrap_or_default()
            .iter()
            .filter_map(|row| row["child_id"].as_i64())
            .collect::<Vec<_>>(),
        });

        resource(
            json!({
                "product": product,
                "attributes": attributes,
                "categories": categories,
                "inventories": inventories,
                "images": images,
                "super_attributes": super_attributes,
                "variants": variants,
                "links": links,
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
        match crate::support::admin_crud::update_product(id, &body).await {
            Ok(_) => resource(
                json!({"id": id}),
                "The product has been updated successfully.",
            ),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/catalog/products/{id}`.
    pub async fn delete_product(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match crate::support::admin_crud::delete_product(id).await {
            Ok(_) => resource(
                Value::Array(Vec::new()),
                "The product has been deleted successfully.",
            ),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/catalog/products/copy/{id}` - clona el producto con
    /// SKU nuevo (`SKU-COPY`): atributos (todos los idiomas), categorías,
    /// imágenes, super attributes y variantes.
    pub async fn copy_product(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let product = db::find_by_id("products", id).await.ok().flatten();
        let Some(product) = product else {
            return message(StatusCode::NOT_FOUND, "Product not found.");
        };

        let copy_sku = format!("{}-COPY", product["sku"].as_str().unwrap_or("SKU"));
        let taken = db::select_one(
            "SELECT `id` FROM `products` WHERE `sku` = ? LIMIT 1",
            &[Value::String(copy_sku.clone())],
        )
        .await
        .ok()
        .flatten();
        if taken.is_some() {
            return validation_error(vec![(
                "sku".into(),
                "The copy SKU already exists: rename it and try again.".into(),
            )]);
        }

        let copy_id = match db::insert(
            "products",
            &json!({
                "sku": copy_sku,
                "type": product["type"].clone(),
                "parent_id": product["parent_id"].clone(),
                "attribute_family_id": product["attribute_family_id"].clone(),
                "created_at": now(),
                "updated_at": now(),
            }),
        )
        .await
        {
            Ok(copy_id) => copy_id,
            Err(error) => return error.into(),
        };

        // Atributos: copiar todas las filas EAV (globales y por idioma),
        // excepto url_key (colisión de slug en la tienda). El sello temporal
        // identifica las filas copiadas para reasignar el producto.
        let stamp = now();
        let _ = db::execute(
            "INSERT INTO `product_attribute_values` \
             (`product_id`, `attribute_id`, `channel`, `locale`, `text_value`, `boolean_value`, `integer_value`, \
              `float_value`, `date_value`, `datetime_value`, `json_value`, `created_at`, `updated_at`) \
             SELECT ?, `attribute_id`, `channel`, `locale`, `text_value`, `boolean_value`, `integer_value`, \
              `float_value`, `date_value`, `datetime_value`, `json_value`, ?, ? \
             FROM `product_attribute_values` \
             WHERE `product_id` = ? AND `attribute_id` != (SELECT `id` FROM `attributes` WHERE `code` = 'url_key')",
            &[Value::from(copy_id), Value::String(stamp.clone()), Value::String(stamp.clone()), Value::from(id)],
        )
        .await;

        // Categorías e imágenes.
        let _ = db::execute(
            "INSERT OR IGNORE INTO `product_categories` (`product_id`, `category_id`) \
             SELECT ?, `category_id` FROM `product_categories` WHERE `product_id` = ?",
            &[Value::from(copy_id), Value::from(id)],
        )
        .await;
        let _ = db::execute(
            "INSERT INTO `product_images` (`product_id`, `type`, `path`, `position`, `created_at`, `updated_at`) \
             SELECT ?, `type`, `path`, `position`, ?, ? FROM `product_images` WHERE `product_id` = ?",
            &[Value::from(copy_id), Value::String(now()), Value::String(now()), Value::from(id)],
        )
        .await;
        let _ = db::execute(
            "INSERT OR IGNORE INTO `product_super_attributes` (`product_id`, `attribute_id`) \
             SELECT ?, `attribute_id` FROM `product_super_attributes` WHERE `product_id` = ?",
            &[Value::from(copy_id), Value::from(id)],
        )
        .await;

        // Variantes con SKUs derivados.
        let variants = db::select(
            "SELECT `id`, `sku` FROM `products` WHERE `parent_id` = ? ORDER BY `id`",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        for variant in &variants {
            let variant_sku = format!("{}-COPY", variant["sku"].as_str().unwrap_or("VARIANT"));
            let taken = db::select_one(
                "SELECT `id` FROM `products` WHERE `sku` = ? LIMIT 1",
                &[Value::String(variant_sku.clone())],
            )
            .await
            .ok()
            .flatten();
            if taken.is_some() {
                continue;
            }
            let variant_copy_id = match db::insert(
                "products",
                &json!({
                    "sku": variant_sku,
                    "type": "simple",
                    "parent_id": copy_id,
                    "attribute_family_id": product["attribute_family_id"].clone(),
                    "created_at": now(),
                    "updated_at": now(),
                }),
            )
            .await
            {
                Ok(variant_copy_id) => variant_copy_id,
                Err(_) => continue,
            };
            let _ = db::execute(
                "INSERT INTO `product_attribute_values` \
                 (`product_id`, `attribute_id`, `channel`, `locale`, `text_value`, `boolean_value`, \
                  `integer_value`, `float_value`, `date_value`, `datetime_value`, `json_value`, \
                  `created_at`, `updated_at`) \
                 SELECT ?, `attribute_id`, `channel`, `locale`, `text_value`, `boolean_value`, \
                  `integer_value`, `float_value`, `date_value`, `datetime_value`, `json_value`, ?, ? \
                 FROM `product_attribute_values` WHERE `product_id` = ?",
                &[
                    Value::from(variant_copy_id),
                    Value::String(stamp.clone()),
                    Value::String(stamp.clone()),
                    Value::from(variant["id"].as_i64().unwrap_or(0)),
                ],
            )
            .await;
            let _ = db::execute(
                "INSERT OR IGNORE INTO `product_inventories` (`product_id`, `inventory_source_id`, `qty`, `created_at`, `updated_at`) \
                 SELECT ?, `inventory_source_id`, `qty`, ?, ? FROM `product_inventories` WHERE `product_id` = ?",
                &[
                    Value::from(variant_copy_id),
                    Value::String(now()),
                    Value::String(now()),
                    Value::from(variant["id"].as_i64().unwrap_or(0)),
                ],
            )
            .await;
        }

        crate::support::products::refresh_flat(copy_id).await;
        resource(
            json!({"id": copy_id, "sku": copy_sku}),
            "The product has been copied successfully.",
        )
    }

    /// `POST /api/admin/catalog/products/reindex-flats` - regenera las filas
    /// de product_flat (por locale) y los índices de precio de todo el
    /// catálogo. Útil tras activar locales nuevos o importar productos.
    pub async fn reindex_flats(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select("SELECT `id` FROM `products` ORDER BY `id`", &[])
            .await
            .unwrap_or_default();
        for row in &rows {
            if let Some(id) = row["id"].as_i64() {
                crate::support::products::refresh_flat(id).await;
            }
        }
        resource(
            json!({"products": rows.len()}),
            format!("{} products have been reindexed.", rows.len()),
        )
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
        match crate::support::admin_crud::create_category(&body).await {
            Ok(category_id) => resource(
                json!({"id": category_id}),
                "The category has been created successfully.",
            ),
            Err(error) => error.into(),
        }
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
        match crate::support::admin_crud::update_category(id, &locale, &body).await {
            Ok(_) => resource(
                json!({"id": id}),
                "The category has been updated successfully.",
            ),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/catalog/categories/{id}` (root protected).
    pub async fn delete_category(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match crate::support::admin_crud::delete_category(id).await {
            Ok(_) => resource(
                Value::Array(Vec::new()),
                "The category has been deleted successfully.",
            ),
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
                "status": child["status"],
                "position": child["position"],
                "children": categories_tree_from(id, locale).await,
            }));
        }
        payload
    })
}

// Booking: configuración de reservas del producto (ubicado tras families).

impl AdminCatalogController {
    /// `GET /api/admin/catalog/products/booking/{id}` - config de reserva.
    pub async fn booking_config(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let booking = db::select_one(
            "SELECT * FROM `booking_products` WHERE `product_id` = ? LIMIT 1",
            &[Value::from(id)],
        )
        .await
        .ok()
        .flatten();
        let slots = match &booking {
            Some(row) => db::select_one(
                "SELECT * FROM `booking_product_default_slots` WHERE `booking_product_id` = ? LIMIT 1",
                &[row["id"].clone()],
            )
            .await
            .ok()
            .flatten(),
            None => None,
        };
        resource(
            json!({
                "booking": booking,
                "slots": slots,
            }),
            Value::Null,
        )
    }

    /// `POST /api/admin/catalog/products/booking/{id}` - upsert de la config:
    /// `{qty, location, available_every_week, duration, break_time, slots}`.
    pub async fn store_booking_config(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let product = db::find_by_id("products", id).await.ok().flatten();
        if product.is_none() {
            return message(StatusCode::NOT_FOUND, "Product not found.");
        }
        let existing = db::select_one(
            "SELECT `id` FROM `booking_products` WHERE `product_id` = ? LIMIT 1",
            &[Value::from(id)],
        )
        .await
        .ok()
        .flatten();
        let booking_id = match existing {
            Some(row) => {
                let _ = db::update(
                    "booking_products",
                    row["id"].as_i64().unwrap_or(0),
                    &json!({
                        "qty": body["qty"].as_i64().unwrap_or(1),
                        "location": body["location"].clone(),
                        "available_every_week": body["available_every_week"].as_i64().unwrap_or(1),
                        "updated_at": now(),
                    }),
                )
                .await;
                row["id"].as_i64().unwrap_or(0)
            }
            None => match db::insert(
                "booking_products",
                &json!({
                    "type": "default",
                    "qty": body["qty"].as_i64().unwrap_or(1),
                    "location": body["location"].clone(),
                    "available_every_week": body["available_every_week"].as_i64().unwrap_or(1),
                    "product_id": id,
                    "created_at": now(),
                    "updated_at": now(),
                }),
            )
            .await
            {
                Ok(booking_id) => booking_id,
                Err(error) => return error.into(),
            },
        };

        // Upsert de los slots semanales (duration, break y JSON de franjas).
        let slots_existing = db::select_one(
            "SELECT `id` FROM `booking_product_default_slots` WHERE `booking_product_id` = ? LIMIT 1",
            &[Value::from(booking_id)],
        )
        .await
        .ok()
        .flatten();
        let slots_row = json!({
            "duration": body["duration"].as_f64().unwrap_or(30.0),
            "break_time": body["break_time"].as_f64().unwrap_or(0.0),
            "slots": body["slots"].clone(),
            "booking_type": "default",
            "booking_product_id": booking_id,
            "updated_at": now(),
        });
        match slots_existing {
            Some(row) => {
                let _ = db::update("booking_product_default_slots", row["id"].as_i64().unwrap_or(0), &slots_row).await;
            }
            None => {
                let mut slots_row = db::to_map(slots_row);
                slots_row.insert("created_at".into(), json!(now()));
                let _ = db::insert("booking_product_default_slots", &Value::Object(slots_row)).await;
            }
        }

        resource(json!({"id": booking_id}), "The booking configuration has been saved successfully.")
    }
}
