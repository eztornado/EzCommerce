//! MCP (Model Context Protocol) tools exposed over `/api/mcp`.
//!
//! The transport is stateless JSON-RPC 2.0 over HTTP. Authentication reuses
//! the regular admin JWT (`Authorization: Bearer <token>`).

use larust_support::serde_json::{json, Value};

use super::db;
use super::products;
use super::response::{ApiError, ApiResult};
use super::sales;

fn arg_str(args: &Value, key: &str) -> String {
    args[key]
        .as_str()
        .map(str::trim)
        .unwrap_or("")
        .to_string()
}

fn arg_i64(args: &Value, key: &str, default: i64) -> i64 {
    args[key].as_i64().unwrap_or(default)
}

fn parse_json_field(row: &mut db::RowMap, key: &str) {
    let Some(raw) = row.get(key).and_then(Value::as_str) else {
        return;
    };
    if let Ok(parsed) = larust_support::serde_json::from_str::<Value>(raw) {
        row.insert(key.to_string(), parsed);
    }
}

/// Tool catalogue advertised through `tools/list` and the OpenAPI document.
pub fn tools() -> Value {
    json!([
        {
            "name": "list_products",
            "title": "List products",
            "description": "List catalog products with optional text search and pagination.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search by name or SKU." },
                    "page": { "type": "integer", "minimum": 1, "default": 1 },
                    "per_page": { "type": "integer", "minimum": 1, "maximum": 100, "default": 20 }
                },
                "additionalProperties": false
            }
        },
        {
            "name": "get_product",
            "title": "Get product",
            "description": "Return the full product resource, including attributes, variants and images.",
            "inputSchema": {
                "type": "object",
                "properties": { "product_id": { "type": "integer" } },
                "required": ["product_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_orders",
            "title": "List orders",
            "description": "List orders filtered by status, order number or customer email.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "status": { "type": "string" },
                    "query": { "type": "string", "description": "Order increment id or customer email." },
                    "page": { "type": "integer", "minimum": 1, "default": 1 },
                    "per_page": { "type": "integer", "minimum": 1, "maximum": 100, "default": 20 }
                },
                "additionalProperties": false
            }
        },
        {
            "name": "get_order",
            "title": "Get order",
            "description": "Return the full order resource with items, addresses, payment, invoices, shipments, refunds and transactions.",
            "inputSchema": {
                "type": "object",
                "properties": { "order_id": { "type": "integer" } },
                "required": ["order_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "create_invoice",
            "title": "Create invoice",
            "description": "Create an invoice for every invoiceable item of an order.",
            "inputSchema": {
                "type": "object",
                "properties": { "order_id": { "type": "integer" } },
                "required": ["order_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "cancel_order",
            "title": "Cancel order",
            "description": "Cancel the order, release reserved stock and recompute its state.",
            "inputSchema": {
                "type": "object",
                "properties": { "order_id": { "type": "integer" } },
                "required": ["order_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_customers",
            "title": "List customers",
            "description": "List customers with optional search by name or email.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "page": { "type": "integer", "minimum": 1, "default": 1 },
                    "per_page": { "type": "integer", "minimum": 1, "maximum": 100, "default": 20 }
                },
                "additionalProperties": false
            }
        },
        {
            "name": "list_transactions",
            "title": "List order transactions",
            "description": "List payment, authorization, refund and dispute transactions for an order.",
            "inputSchema": {
                "type": "object",
                "properties": { "order_id": { "type": "integer" } },
                "required": ["order_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_refunds",
            "title": "List refunds",
            "description": "List refunds, optionally filtered by order.",
            "inputSchema": {
                "type": "object",
                "properties": { "order_id": { "type": "integer" } },
                "additionalProperties": false
            }
        },
        {
            "name": "sales_summary",
            "title": "Sales summary",
            "description": "Aggregate order count, sales, invoiced and refunded totals for an optional date range.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "from": { "type": "string", "description": "Start date (YYYY-MM-DD)." },
                    "to": { "type": "string", "description": "End date (YYYY-MM-DD)." }
                },
                "additionalProperties": false
            }
        },
        {
            "name": "list_categories",
            "title": "List categories",
            "description": "List the category tree (flat, ordered by nested-set position) with translated names.",
            "inputSchema": {
                "type": "object",
                "properties": { "locale": { "type": "string", "description": "Locale code, defaults to en." } },
                "additionalProperties": false
            }
        },
        {
            "name": "get_category",
            "title": "Get category",
            "description": "Return one category with all of its translations.",
            "inputSchema": {
                "type": "object",
                "properties": { "category_id": { "type": "integer" } },
                "required": ["category_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_attribute_families",
            "title": "List attribute families",
            "description": "List the attribute families available when creating products.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "list_attributes",
            "title": "List attributes",
            "description": "List product attributes and their types/flags, used to fill the attributes object of create_product.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "get_customer",
            "title": "Get customer",
            "description": "Return a customer profile with recent orders, addresses and review/wishlist counts.",
            "inputSchema": {
                "type": "object",
                "properties": { "customer_id": { "type": "integer" } },
                "required": ["customer_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_invoices",
            "title": "List invoices",
            "description": "List invoices, optionally filtered by order.",
            "inputSchema": {
                "type": "object",
                "properties": { "order_id": { "type": "integer" } },
                "additionalProperties": false
            }
        },
        {
            "name": "get_invoice",
            "title": "Get invoice",
            "description": "Return one invoice with its line items.",
            "inputSchema": {
                "type": "object",
                "properties": { "invoice_id": { "type": "integer" } },
                "required": ["invoice_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_shipments",
            "title": "List shipments",
            "description": "List shipments, optionally filtered by order.",
            "inputSchema": {
                "type": "object",
                "properties": { "order_id": { "type": "integer" } },
                "additionalProperties": false
            }
        },
        {
            "name": "get_shipment",
            "title": "Get shipment",
            "description": "Return one shipment with its line items.",
            "inputSchema": {
                "type": "object",
                "properties": { "shipment_id": { "type": "integer" } },
                "required": ["shipment_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_cart_rules",
            "title": "List cart price rules",
            "description": "List cart price rules (coupons/promotions) with status, discount and usage counters.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "get_cart_rule",
            "title": "Get cart price rule",
            "description": "Return one cart rule with its translations, channels and coupon codes.",
            "inputSchema": {
                "type": "object",
                "properties": { "rule_id": { "type": "integer" } },
                "required": ["rule_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_catalog_rules",
            "title": "List catalog price rules",
            "description": "List catalog price rules with status, discount and active window.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "get_catalog_rule",
            "title": "Get catalog price rule",
            "description": "Return one catalog rule with its channels and the products it has been applied to.",
            "inputSchema": {
                "type": "object",
                "properties": { "rule_id": { "type": "integer" } },
                "required": ["rule_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_cms_pages",
            "title": "List CMS pages",
            "description": "List CMS pages with their translated title and URL key.",
            "inputSchema": {
                "type": "object",
                "properties": { "locale": { "type": "string", "description": "Locale code, defaults to en." } },
                "additionalProperties": false
            }
        },
        {
            "name": "get_cms_page",
            "title": "Get CMS page",
            "description": "Return one CMS page with all translations and channel assignments.",
            "inputSchema": {
                "type": "object",
                "properties": { "page_id": { "type": "integer" } },
                "required": ["page_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "list_channels",
            "title": "List channels",
            "description": "List store channels with their translated names and currency/locale defaults.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "list_locales",
            "title": "List locales",
            "description": "List configured locales.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "list_currencies",
            "title": "List currencies",
            "description": "List configured currencies.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "list_tax_rates",
            "title": "List tax rates",
            "description": "List tax rates with their country/state/ZIP scope.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "list_inventory_sources",
            "title": "List inventory sources",
            "description": "List inventory sources (warehouses) usable in stock and shipment tools.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "list_reviews",
            "title": "List product reviews",
            "description": "List product reviews, optionally filtered by status or product.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "status": { "type": "string", "description": "pending, approved or rejected." },
                    "product_id": { "type": "integer" },
                    "page": { "type": "integer", "minimum": 1, "default": 1 },
                    "per_page": { "type": "integer", "minimum": 1, "maximum": 100, "default": 20 }
                },
                "additionalProperties": false
            }
        },
        {
            "name": "low_stock_products",
            "title": "Low stock products",
            "description": "List visible products at or below a stock threshold.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "threshold": { "type": "integer", "minimum": 0, "default": 5 },
                    "per_page": { "type": "integer", "minimum": 1, "maximum": 200, "default": 50 }
                },
                "additionalProperties": false
            }
        },
        {
            "name": "get_product_stock",
            "title": "Get product stock",
            "description": "Return per-source stock levels for a product.",
            "inputSchema": {
                "type": "object",
                "properties": { "product_id": { "type": "integer" } },
                "required": ["product_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "create_product",
            "title": "Create product",
            "description": "Create a product and optionally apply its attributes, categories, images, links and stock in the same call.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sku": { "type": "string" },
                    "type": { "type": "string", "description": "simple, configurable, virtual, downloadable, bundle or booking. Defaults to simple." },
                    "attribute_family_id": { "type": "integer", "default": 1 },
                    "super_attributes": { "type": "array", "items": { "type": "integer" } },
                    "locale": { "type": "string" },
                    "attributes": { "type": "object", "description": "{code: value}." },
                    "categories": { "type": "array", "items": { "type": "integer" } },
                    "images": { "type": "array", "items": { "type": "string" }, "description": "Storage paths, e.g. products/lamp.jpg." },
                    "links": { "type": "object" },
                    "inventories": { "type": "object", "description": "{inventory_source_id: qty}." }
                },
                "required": ["sku"],
                "additionalProperties": false
            }
        },
        {
            "name": "update_product",
            "title": "Update product",
            "description": "Update a product SKU, attributes (per locale), categories, images, related links and stock.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "product_id": { "type": "integer" },
                    "sku": { "type": "string" },
                    "locale": { "type": "string" },
                    "attributes": { "type": "object", "description": "{code: value}." },
                    "categories": { "type": "array", "items": { "type": "integer" } },
                    "images": { "type": "array", "items": { "type": "string" } },
                    "links": { "type": "object", "description": "{related|up_sells|cross_sells: [product_id]}." },
                    "inventories": { "type": "object", "description": "{inventory_source_id: qty}." }
                },
                "required": ["product_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "delete_product",
            "title": "Delete product",
            "description": "Delete a product by id. Destructive: prefer disabling it through attributes/status when possible.",
            "inputSchema": {
                "type": "object",
                "properties": { "product_id": { "type": "integer" } },
                "required": ["product_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "set_product_stock",
            "title": "Set product stock",
            "description": "Set a product's stock for one inventory source.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "product_id": { "type": "integer" },
                    "source_id": { "type": "integer", "default": 1 },
                    "quantity": { "type": "integer", "minimum": 0 }
                },
                "required": ["product_id", "quantity"],
                "additionalProperties": false
            }
        },
        {
            "name": "create_category",
            "title": "Create category",
            "description": "Create a category under a parent, with its translated name and SEO fields.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "parent_id": { "type": "integer", "default": 1 },
                    "locale": { "type": "string", "default": "en" },
                    "position": { "type": "integer" },
                    "status": { "type": "integer", "enum": [0, 1], "default": 1 },
                    "description": { "type": "string" },
                    "meta_title": { "type": "string" },
                    "meta_description": { "type": "string" }
                },
                "required": ["name"],
                "additionalProperties": false
            }
        },
        {
            "name": "update_category",
            "title": "Update category",
            "description": "Update a category's position/status and translated content.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "category_id": { "type": "integer" },
                    "locale": { "type": "string", "default": "en" },
                    "name": { "type": "string" },
                    "position": { "type": "integer" },
                    "status": { "type": "integer", "enum": [0, 1] },
                    "description": { "type": "string" },
                    "meta_title": { "type": "string" },
                    "meta_description": { "type": "string" }
                },
                "required": ["category_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "delete_category",
            "title": "Delete category",
            "description": "Delete a non-root category by id. Destructive.",
            "inputSchema": {
                "type": "object",
                "properties": { "category_id": { "type": "integer" } },
                "required": ["category_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "create_cms_page",
            "title": "Create CMS page",
            "description": "Create a CMS page (title, url key, HTML content and SEO metadata) for a locale.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "page_title": { "type": "string" },
                    "url_key": { "type": "string" },
                    "html_content": { "type": "string" },
                    "locale": { "type": "string", "default": "en" },
                    "meta_title": { "type": "string" },
                    "meta_description": { "type": "string" }
                },
                "required": ["page_title", "url_key"],
                "additionalProperties": false
            }
        },
        {
            "name": "update_cms_page",
            "title": "Update CMS page",
            "description": "Update a CMS page's translated content and metadata.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "page_id": { "type": "integer" },
                    "page_title": { "type": "string" },
                    "url_key": { "type": "string" },
                    "html_content": { "type": "string" },
                    "locale": { "type": "string", "default": "en" },
                    "meta_title": { "type": "string" },
                    "meta_description": { "type": "string" }
                },
                "required": ["page_id", "page_title", "url_key"],
                "additionalProperties": false
            }
        },
        {
            "name": "delete_cms_page",
            "title": "Delete CMS page",
            "description": "Delete a CMS page by id. Destructive.",
            "inputSchema": {
                "type": "object",
                "properties": { "page_id": { "type": "integer" } },
                "required": ["page_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "create_customer",
            "title": "Create customer",
            "description": "Create a customer account from the admin panel (password hashed, email unique).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "first_name": { "type": "string" },
                    "last_name": { "type": "string" },
                    "email": { "type": "string" },
                    "password": { "type": "string", "description": "At least 6 characters." },
                    "phone": { "type": "string" },
                    "customer_group_id": { "type": "integer", "default": 2 },
                    "status": { "type": "integer", "enum": [0, 1], "default": 1 },
                    "subscribed_to_news_letter": { "type": "integer", "enum": [0, 1], "default": 0 }
                },
                "required": ["first_name", "last_name", "email", "password"],
                "additionalProperties": false
            }
        },
        {
            "name": "update_customer",
            "title": "Update customer",
            "description": "Update basic customer profile fields (name, email, phone, group, status). Passwords are never changed here.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "customer_id": { "type": "integer" },
                    "first_name": { "type": "string" },
                    "last_name": { "type": "string" },
                    "email": { "type": "string" },
                    "phone": { "type": "string" },
                    "gender": { "type": "string" },
                    "date_of_birth": { "type": "string" },
                    "customer_group_id": { "type": "integer" },
                    "status": { "type": "integer", "enum": [0, 1] },
                    "subscribed_to_news_letter": { "type": "integer", "enum": [0, 1] }
                },
                "required": ["customer_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "add_order_comment",
            "title": "Add order comment",
            "description": "Append an internal comment to an order, optionally notifying the customer.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "order_id": { "type": "integer" },
                    "comment": { "type": "string" },
                    "customer_notified": { "type": "boolean", "default": false }
                },
                "required": ["order_id", "comment"],
                "additionalProperties": false
            }
        },
        {
            "name": "create_shipment",
            "title": "Create shipment",
            "description": "Create a shipment for an order, shipping all remaining items or the given quantities.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "order_id": { "type": "integer" },
                    "source_id": { "type": "integer", "default": 1 },
                    "items": { "type": "object", "description": "{order_item_id: qty} or {all: qty}." }
                },
                "required": ["order_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "create_refund",
            "title": "Create refund",
            "description": "Refund all refundable items or the given quantities. Stripe-backed payments are refunded through Stripe first.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "order_id": { "type": "integer" },
                    "items": { "type": "object", "description": "{order_item_id: qty} or {all: qty}." },
                    "adjustment_refund": { "type": "number", "default": 0 },
                    "adjustment_fee": { "type": "number", "default": 0 }
                },
                "required": ["order_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "save_configuration",
            "title": "Save configuration",
            "description": "Upsert global core configuration values by code, e.g. general.content.footer.copyright_content.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "values": { "type": "object", "description": "{config_code: value}.", "additionalProperties": true }
                },
                "required": ["values"],
                "additionalProperties": false
            }
        }
    ])
}

async fn list_products(args: &Value) -> ApiResult<Value> {
    let page = arg_i64(args, "page", 1).max(1);
    let per_page = arg_i64(args, "per_page", 20).clamp(1, 100);
    let search = arg_str(args, "query");
    let mut clauses = vec![
        "`pf`.`status` = 1".to_string(),
        "`pf`.`visible_individually` = 1".to_string(),
        "`pf`.`parent_id` IS NULL".to_string(),
        "`pf`.`name` IS NOT NULL".to_string(),
        "`pf`.`product_id` IN (SELECT `id` FROM `products`)".to_string(),
    ];
    let mut binds: Vec<Value> = Vec::new();
    if !search.is_empty() {
        clauses.push("(`pf`.`name` LIKE ? OR `pf`.`sku` LIKE ?)".to_string());
        binds.push(Value::String(format!("%{search}%")));
        binds.push(Value::String(format!("%{search}%")));
    }
    let where_sql = clauses.join(" AND ");
    let total = db::scalar_i64(
        &format!(
            "SELECT COUNT(DISTINCT `pf`.`product_id`) FROM `product_flat` `pf` WHERE {where_sql}"
        ),
        &binds,
    )
    .await?;
    let mut list_binds = binds;
    list_binds.push(Value::from(per_page));
    list_binds.push(Value::from((page - 1) * per_page));
    let rows = db::select(
        &format!(
            "SELECT DISTINCT `pf`.`product_id` AS `id`, `pf`.`name`, `pf`.`sku`, \
                    `pf`.`type`, `pf`.`price`, `pf`.`url_key`, `pf`.`base_image` \
             FROM `product_flat` `pf` WHERE {where_sql} \
             ORDER BY `pf`.`product_id` DESC LIMIT ? OFFSET ?"
        ),
        &list_binds,
    )
    .await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            let image = row["base_image"]
                .as_str()
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .map(|path| json!({ "url": format!("/storage/{path}") }));
            json!({
                "id": row["id"],
                "name": row["name"],
                "sku": row["sku"],
                "type": row["type"],
                "price": row["price"],
                "url_key": row["url_key"],
                "base_image": image
            })
        })
        .collect();
    Ok(json!({
        "data": data,
        "meta": { "page": page, "per_page": per_page, "total": total }
    }))
}

async fn list_orders(args: &Value) -> ApiResult<Value> {
    let page = arg_i64(args, "page", 1).max(1);
    let per_page = arg_i64(args, "per_page", 20).clamp(1, 100);
    let status = arg_str(args, "status");
    let search = arg_str(args, "query");
    let mut clauses: Vec<String> = Vec::new();
    let mut binds: Vec<Value> = Vec::new();
    if !status.is_empty() {
        clauses.push("`status` = ?".to_string());
        binds.push(Value::String(status));
    }
    if !search.is_empty() {
        clauses.push("(`increment_id` LIKE ? OR `customer_email` LIKE ?)".to_string());
        binds.push(Value::String(format!("%{search}%")));
        binds.push(Value::String(format!("%{search}%")));
    }
    let where_sql = if clauses.is_empty() {
        "1 = 1".to_string()
    } else {
        clauses.join(" AND ")
    };
    let total = db::scalar_i64(
        &format!("SELECT COUNT(*) FROM `orders` WHERE {where_sql}"),
        &binds,
    )
    .await?;
    let mut list_binds = binds;
    list_binds.push(Value::from(per_page));
    list_binds.push(Value::from((page - 1) * per_page));
    let rows = db::select(
        &format!(
            "SELECT `id`, `increment_id`, `status`, `payment_status`, `customer_email`, \
                    `customer_first_name`, `customer_last_name`, `base_grand_total`, \
                    `created_at` FROM `orders` WHERE {where_sql} \
             ORDER BY `id` DESC LIMIT ? OFFSET ?"
        ),
        &list_binds,
    )
    .await?;
    Ok(json!({
        "data": rows,
        "meta": { "page": page, "per_page": per_page, "total": total }
    }))
}

async fn list_customers(args: &Value) -> ApiResult<Value> {
    let page = arg_i64(args, "page", 1).max(1);
    let per_page = arg_i64(args, "per_page", 20).clamp(1, 100);
    let search = arg_str(args, "query");
    let mut clauses: Vec<String> = Vec::new();
    let mut binds: Vec<Value> = Vec::new();
    if !search.is_empty() {
        clauses.push(
            "(`first_name` LIKE ? OR `last_name` LIKE ? OR `email` LIKE ?)".to_string(),
        );
        binds.push(Value::String(format!("%{search}%")));
        binds.push(Value::String(format!("%{search}%")));
        binds.push(Value::String(format!("%{search}%")));
    }
    let where_sql = if clauses.is_empty() {
        "1 = 1".to_string()
    } else {
        clauses.join(" AND ")
    };
    let total = db::scalar_i64(
        &format!("SELECT COUNT(*) FROM `customers` WHERE {where_sql}"),
        &binds,
    )
    .await?;
    let mut list_binds = binds;
    list_binds.push(Value::from(per_page));
    list_binds.push(Value::from((page - 1) * per_page));
    let rows = db::select(
        &format!(
            "SELECT `id`, `first_name`, `last_name`, `email`, `status`, \
                    `customer_group_id`, `created_at` FROM `customers` \
             WHERE {where_sql} ORDER BY `id` DESC LIMIT ? OFFSET ?"
        ),
        &list_binds,
    )
    .await?;
    Ok(json!({
        "data": rows,
        "meta": { "page": page, "per_page": per_page, "total": total }
    }))
}

async fn list_transactions(order_id: i64) -> ApiResult<Value> {
    if order_id <= 0 {
        return Err(ApiError::bad_request("order_id is required."));
    }
    let rows = db::select(
        "SELECT * FROM `order_transactions` WHERE `order_id` = ? ORDER BY `id` DESC",
        &[Value::from(order_id)],
    )
    .await?;
    let data: Vec<Value> = rows
        .into_iter()
        .map(|mut row| {
            parse_json_field(&mut row, "data");
            Value::Object(row)
        })
        .collect();
    Ok(json!({ "order_id": order_id, "data": data }))
}

async fn list_refunds(order_id: i64) -> ApiResult<Value> {
    let rows = if order_id > 0 {
        db::select(
            "SELECT * FROM `refunds` WHERE `order_id` = ? ORDER BY `id` DESC",
            &[Value::from(order_id)],
        )
        .await?
    } else {
        db::select("SELECT * FROM `refunds` ORDER BY `id` DESC LIMIT 100", &[]).await?
    };
    Ok(json!({ "data": rows }))
}

async fn sales_summary(args: &Value) -> ApiResult<Value> {
    let from = arg_str(args, "from");
    let to = arg_str(args, "to");
    let mut clauses: Vec<String> = Vec::new();
    let mut binds: Vec<Value> = Vec::new();
    if !from.is_empty() {
        clauses.push("`created_at` >= ?".to_string());
        binds.push(Value::String(format!("{from} 00:00:00")));
    }
    if !to.is_empty() {
        clauses.push("`created_at` <= ?".to_string());
        binds.push(Value::String(format!("{to} 23:59:59")));
    }
    let where_sql = if clauses.is_empty() {
        "1 = 1".to_string()
    } else {
        clauses.join(" AND ")
    };
    let row = db::select_one(
        &format!(
            "SELECT COUNT(*) AS `orders`, \
                    COALESCE(SUM(`base_grand_total`), 0) AS `grand_total`, \
                    COALESCE(SUM(`base_grand_total_invoiced`), 0) AS `invoiced`, \
                    COALESCE(SUM(`base_grand_total_refunded`), 0) AS `refunded` \
             FROM `orders` WHERE {where_sql}"
        ),
        &binds,
    )
    .await?
    .unwrap_or_else(db::empty_row);
    Ok(json!({
        "from": from,
        "to": to,
        "orders": row["orders"],
        "grand_total": row["grand_total"],
        "invoiced": row["invoiced"],
        "refunded": row["refunded"]
    }))
}

// ---------------------------------------------------------------------------
// Catalog reads
// ---------------------------------------------------------------------------

async fn list_categories(args: &Value) -> ApiResult<Value> {
    let locale = args["locale"].as_str().unwrap_or("en").to_string();
    let rows = db::select(
        "SELECT `c`.`id`, `c`.`parent_id`, `c`.`position`, `c`.`status`, `c`.`display_mode`, \
                `ct`.`name`, `ct`.`slug`, `ct`.`url_path`, `ct`.`description` \
         FROM `categories` `c` \
         LEFT JOIN `category_translations` `ct` ON `ct`.`category_id` = `c`.`id` AND `ct`.`locale` = ? \
         ORDER BY `c`.`_lft`, `c`.`id`",
        &[Value::String(locale.clone())],
    )
    .await?;
    Ok(json!({ "locale": locale, "data": rows }))
}

async fn get_category(category_id: i64) -> ApiResult<Value> {
    let category = db::find_by_id("categories", category_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Category not found."))?;
    let translations = db::select(
        "SELECT * FROM `category_translations` WHERE `category_id` = ? ORDER BY `locale`",
        &[Value::from(category_id)],
    )
    .await?;
    Ok(json!({
        "category": category,
        "translations": translations
    }))
}

async fn list_attribute_families() -> ApiResult<Value> {
    let rows = db::select(
        "SELECT `id`, `code`, `name`, `status`, `is_user_defined` FROM `attribute_families` ORDER BY `id`",
        &[],
    )
    .await?;
    Ok(json!({ "data": rows }))
}

async fn list_attributes() -> ApiResult<Value> {
    let rows = db::select(
        "SELECT `id`, `code`, `admin_name`, `type`, `swatch_type`, `is_required`, \
                `is_unique`, `is_filterable`, `is_configurable`, `value_per_locale`, \
                `value_per_channel`, `default_value` \
         FROM `attributes` ORDER BY `id`",
        &[],
    )
    .await?;
    Ok(json!({ "data": rows }))
}

async fn get_customer(customer_id: i64) -> ApiResult<Value> {
    let mut customer = db::find_by_id("customers", customer_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Customer not found."))?;
    for key in ["password", "api_token", "token", "remember_token"] {
        customer.remove(key);
    }
    let orders = db::select(
        "SELECT `id`, `increment_id`, `status`, `base_grand_total`, `created_at` \
         FROM `orders` WHERE `customer_id` = ? ORDER BY `id` DESC LIMIT 10",
        &[Value::from(customer_id)],
    )
    .await?;
    let addresses = db::select(
        "SELECT * FROM `addresses` WHERE `customer_id` = ? AND `address_type` = 'customer_address' ORDER BY `id` DESC",
        &[Value::from(customer_id)],
    )
    .await?;
    let reviews = db::select(
        "SELECT `id`, `title`, `rating`, `status`, `product_id`, `created_at` \
         FROM `product_reviews` WHERE `customer_id` = ? ORDER BY `id` DESC LIMIT 10",
        &[Value::from(customer_id)],
    )
    .await?;
    let wishlist_count = db::scalar_i64(
        "SELECT COUNT(*) FROM `wishlist_items` WHERE `customer_id` = ?",
        &[Value::from(customer_id)],
    )
    .await
    .unwrap_or(0);
    Ok(json!({
        "customer": customer,
        "orders": orders,
        "addresses": addresses,
        "reviews": reviews,
        "wishlist_count": wishlist_count
    }))
}

// ---------------------------------------------------------------------------
// Sales reads
// ---------------------------------------------------------------------------

async fn list_invoices(args: &Value) -> ApiResult<Value> {
    let order_id = arg_i64(args, "order_id", 0);
    let rows = if order_id > 0 {
        db::select(
            "SELECT `invoices`.*, `orders`.`increment_id` AS `order_increment_id`, \
                    `orders`.`customer_email` FROM `invoices` \
             LEFT JOIN `orders` ON `orders`.`id` = `invoices`.`order_id` \
             WHERE `invoices`.`order_id` = ? ORDER BY `invoices`.`id` DESC",
            &[Value::from(order_id)],
        )
        .await?
    } else {
        db::select(
            "SELECT `invoices`.*, `orders`.`increment_id` AS `order_increment_id`, \
                    `orders`.`customer_email` FROM `invoices` \
             LEFT JOIN `orders` ON `orders`.`id` = `invoices`.`order_id` \
             ORDER BY `invoices`.`id` DESC LIMIT 100",
            &[],
        )
        .await?
    };
    Ok(json!({ "data": rows }))
}

async fn get_invoice(invoice_id: i64) -> ApiResult<Value> {
    let invoice = db::find_by_id("invoices", invoice_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Invoice not found."))?;
    let items = db::select(
        "SELECT * FROM `invoice_items` WHERE `invoice_id` = ? ORDER BY `id`",
        &[Value::from(invoice_id)],
    )
    .await?;
    Ok(json!({ "invoice": invoice, "items": items }))
}

async fn list_shipments(args: &Value) -> ApiResult<Value> {
    let order_id = arg_i64(args, "order_id", 0);
    let rows = if order_id > 0 {
        db::select(
            "SELECT `shipments`.*, `orders`.`increment_id` AS `order_increment_id` \
             FROM `shipments` LEFT JOIN `orders` ON `orders`.`id` = `shipments`.`order_id` \
             WHERE `shipments`.`order_id` = ? ORDER BY `shipments`.`id` DESC",
            &[Value::from(order_id)],
        )
        .await?
    } else {
        db::select(
            "SELECT `shipments`.*, `orders`.`increment_id` AS `order_increment_id` \
             FROM `shipments` LEFT JOIN `orders` ON `orders`.`id` = `shipments`.`order_id` \
             ORDER BY `shipments`.`id` DESC LIMIT 100",
            &[],
        )
        .await?
    };
    Ok(json!({ "data": rows }))
}

async fn get_shipment(shipment_id: i64) -> ApiResult<Value> {
    let shipment = db::find_by_id("shipments", shipment_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Shipment not found."))?;
    let items = db::select(
        "SELECT * FROM `shipment_items` WHERE `shipment_id` = ? ORDER BY `id`",
        &[Value::from(shipment_id)],
    )
    .await?;
    Ok(json!({ "shipment": shipment, "items": items }))
}

// ---------------------------------------------------------------------------
// Marketing/content reads
// ---------------------------------------------------------------------------

async fn list_cart_rules() -> ApiResult<Value> {
    let rows = db::select(
        "SELECT `id`, `name`, `description`, `status`, `coupon_type`, `action_type`, \
                `discount_amount`, `starts_from`, `ends_till`, `times_used`, `sort_order` \
         FROM `cart_rules` ORDER BY `id` DESC",
        &[],
    )
    .await?;
    Ok(json!({ "data": rows }))
}

async fn get_cart_rule(rule_id: i64) -> ApiResult<Value> {
    let mut rule = db::find_by_id("cart_rules", rule_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Cart rule not found."))?;
    parse_json_field(&mut rule, "conditions");
    let translations = db::select(
        "SELECT * FROM `cart_rule_translations` WHERE `cart_rule_id` = ? ORDER BY `locale`",
        &[Value::from(rule_id)],
    )
    .await?;
    let coupons = db::select(
        "SELECT `id`, `code`, `usage_limit`, `usage_per_customer`, `times_used`, `expired_at` \
         FROM `cart_rule_coupons` WHERE `cart_rule_id` = ? ORDER BY `id`",
        &[Value::from(rule_id)],
    )
    .await?;
    let channels = db::select(
        "SELECT `channel_id` FROM `cart_rule_channels` WHERE `cart_rule_id` = ?",
        &[Value::from(rule_id)],
    )
    .await?;
    Ok(json!({
        "rule": rule,
        "translations": translations,
        "coupons": coupons,
        "channels": channels
    }))
}

async fn list_catalog_rules() -> ApiResult<Value> {
    let rows = db::select(
        "SELECT `id`, `name`, `description`, `status`, `action_type`, `discount_amount`, \
                `starts_from`, `ends_till`, `sort_order` FROM `catalog_rules` ORDER BY `id` DESC",
        &[],
    )
    .await?;
    Ok(json!({ "data": rows }))
}

async fn get_catalog_rule(rule_id: i64) -> ApiResult<Value> {
    let mut rule = db::find_by_id("catalog_rules", rule_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Catalog rule not found."))?;
    parse_json_field(&mut rule, "conditions");
    let channels = db::select(
        "SELECT `channel_id` FROM `catalog_rule_channels` WHERE `catalog_rule_id` = ?",
        &[Value::from(rule_id)],
    )
    .await?;
    let products = db::select(
        "SELECT `product_id`, `action_type`, `discount_amount`, `starts_from`, `ends_till` \
         FROM `catalog_rule_products` WHERE `catalog_rule_id` = ? ORDER BY `id` LIMIT 200",
        &[Value::from(rule_id)],
    )
    .await?;
    Ok(json!({ "rule": rule, "channels": channels, "products": products }))
}

async fn list_cms_pages(args: &Value) -> ApiResult<Value> {
    let locale = args["locale"].as_str().unwrap_or("en").to_string();
    let rows = db::select(
        "SELECT `cp`.`id`, `cp`.`layout`, `cp`.`created_at`, `cp`.`updated_at`, \
                `ct`.`locale`, `ct`.`page_title`, `ct`.`url_key` \
         FROM `cms_pages` `cp` \
         LEFT JOIN `cms_page_translations` `ct` ON `ct`.`cms_page_id` = `cp`.`id` AND `ct`.`locale` = ? \
         ORDER BY `cp`.`id` DESC",
        &[Value::String(locale.clone())],
    )
    .await?;
    Ok(json!({ "locale": locale, "data": rows }))
}

async fn get_cms_page(page_id: i64) -> ApiResult<Value> {
    let page = db::find_by_id("cms_pages", page_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Page not found."))?;
    let translations = db::select(
        "SELECT * FROM `cms_page_translations` WHERE `cms_page_id` = ? ORDER BY `locale`",
        &[Value::from(page_id)],
    )
    .await?;
    let channels = db::select(
        "SELECT `channel_id` FROM `cms_page_channels` WHERE `cms_page_id` = ?",
        &[Value::from(page_id)],
    )
    .await?;
    Ok(json!({ "page": page, "translations": translations, "channels": channels }))
}

// ---------------------------------------------------------------------------
// Store configuration reads
// ---------------------------------------------------------------------------

async fn list_channels(args: &Value) -> ApiResult<Value> {
    let locale = args["locale"].as_str().unwrap_or("en").to_string();
    let rows = db::select(
        "SELECT `c`.`id`, `c`.`code`, `c`.`timezone`, `c`.`theme`, `c`.`hostname`, \
                `c`.`logo`, `c`.`favicon`, `c`.`root_category_id`, `c`.`default_locale_id`, \
                `c`.`base_currency_id`, `ct`.`name`, `ct`.`description` \
         FROM `channels` `c` \
         LEFT JOIN `channel_translations` `ct` ON `ct`.`channel_id` = `c`.`id` AND `ct`.`locale` = ? \
         ORDER BY `c`.`id`",
        &[Value::String(locale.clone())],
    )
    .await?;
    Ok(json!({ "locale": locale, "data": rows }))
}

async fn list_locales() -> ApiResult<Value> {
    let rows = db::select(
        "SELECT `id`, `code`, `name`, `direction` FROM `locales` ORDER BY `id`",
        &[],
    )
    .await?;
    Ok(json!({ "data": rows }))
}

async fn list_currencies() -> ApiResult<Value> {
    let rows = db::select(
        "SELECT `id`, `code`, `name`, `symbol`, `decimal` FROM `currencies` ORDER BY `id`",
        &[],
    )
    .await?;
    Ok(json!({ "data": rows }))
}

async fn list_tax_rates() -> ApiResult<Value> {
    let rows = db::select("SELECT * FROM `tax_rates` ORDER BY `id`", &[]).await?;
    Ok(json!({ "data": rows }))
}

async fn list_inventory_sources() -> ApiResult<Value> {
    let rows = db::select(
        "SELECT `id`, `code`, `name`, `description`, `country`, `state`, `city`, \
                `priority`, `status` FROM `inventory_sources` ORDER BY `id`",
        &[],
    )
    .await?;
    Ok(json!({ "data": rows }))
}

async fn list_reviews(args: &Value) -> ApiResult<Value> {
    let page = arg_i64(args, "page", 1).max(1);
    let per_page = arg_i64(args, "per_page", 20).clamp(1, 100);
    let status = arg_str(args, "status");
    let product_id = arg_i64(args, "product_id", 0);
    let mut clauses: Vec<String> = Vec::new();
    let mut binds: Vec<Value> = Vec::new();
    if !status.is_empty() {
        clauses.push("`status` = ?".to_string());
        binds.push(Value::String(status));
    }
    if product_id > 0 {
        clauses.push("`product_id` = ?".to_string());
        binds.push(Value::from(product_id));
    }
    let where_sql = if clauses.is_empty() {
        "1 = 1".to_string()
    } else {
        clauses.join(" AND ")
    };
    let total = db::scalar_i64(
        &format!("SELECT COUNT(*) FROM `product_reviews` WHERE {where_sql}"),
        &binds,
    )
    .await?;
    let mut list_binds = binds;
    list_binds.push(Value::from(per_page));
    list_binds.push(Value::from((page - 1) * per_page));
    let rows = db::select(
        &format!(
            "SELECT `id`, `name`, `title`, `rating`, `comment`, `status`, `product_id`, \
                    `customer_id`, `created_at` FROM `product_reviews` WHERE {where_sql} \
             ORDER BY `id` DESC LIMIT ? OFFSET ?"
        ),
        &list_binds,
    )
    .await?;
    Ok(json!({
        "data": rows,
        "meta": { "page": page, "per_page": per_page, "total": total }
    }))
}

async fn low_stock_products(args: &Value) -> ApiResult<Value> {
    let threshold = arg_i64(args, "threshold", 5).max(0);
    let per_page = arg_i64(args, "per_page", 50).clamp(1, 200);
    let rows = db::select(
        "SELECT `pf`.`product_id` AS `id`, MIN(`pf`.`name`) AS `name`, MIN(`pf`.`sku`) AS `sku`, \
                MIN(`pf`.`type`) AS `type`, MIN(`pf`.`quantity`) AS `quantity`, \
                MIN(`pf`.`base_image`) AS `base_image` \
         FROM `product_flat` `pf` \
         WHERE `pf`.`parent_id` IS NULL AND `pf`.`status` = 1 AND `pf`.`quantity` <= ? \
         GROUP BY `pf`.`product_id` ORDER BY `quantity` ASC, `pf`.`product_id` DESC LIMIT ?",
        &[Value::from(threshold), Value::from(per_page)],
    )
    .await?;
    Ok(json!({ "threshold": threshold, "data": rows }))
}

async fn get_product_stock(product_id: i64) -> ApiResult<Value> {
    if product_id <= 0 {
        return Err(ApiError::bad_request("product_id is required."));
    }
    let sources = db::select(
        "SELECT `pi`.`id`, `pi`.`inventory_source_id`, `pi`.`qty`, \
                `isrc`.`code` AS `source_code`, `isrc`.`name` AS `source_name` \
         FROM `product_inventories` `pi` \
         LEFT JOIN `inventory_sources` `isrc` ON `isrc`.`id` = `pi`.`inventory_source_id` \
         WHERE `pi`.`product_id` = ? ORDER BY `pi`.`inventory_source_id`",
        &[Value::from(product_id)],
    )
    .await?;
    let total: i64 = db::scalar_i64(
        "SELECT COALESCE(SUM(`qty`), 0) FROM `product_inventories` WHERE `product_id` = ?",
        &[Value::from(product_id)],
    )
    .await
    .unwrap_or(0);
    Ok(json!({ "product_id": product_id, "total": total, "sources": sources }))
}

// ---------------------------------------------------------------------------
// Admin writes
// ---------------------------------------------------------------------------

async fn create_product(args: &Value) -> ApiResult<Value> {
    let product_id = crate::support::admin_crud::create_product(args).await?;
    let product = products::detail(product_id).await;
    Ok(json!({ "product_id": product_id, "product": product }))
}

async fn update_product(args: &Value) -> ApiResult<Value> {
    let product_id = arg_i64(args, "product_id", 0);
    if product_id <= 0 {
        return Err(ApiError::bad_request("product_id is required."));
    }
    crate::support::admin_crud::update_product(product_id, args).await?;
    Ok(json!({
        "product_id": product_id,
        "product": products::detail(product_id).await
    }))
}

async fn delete_product(args: &Value) -> ApiResult<Value> {
    let product_id = arg_i64(args, "product_id", 0);
    if product_id <= 0 {
        return Err(ApiError::bad_request("product_id is required."));
    }
    crate::support::admin_crud::delete_product(product_id).await?;
    Ok(json!({ "product_id": product_id, "deleted": true }))
}

async fn set_product_stock(args: &Value) -> ApiResult<Value> {
    let product_id = arg_i64(args, "product_id", 0);
    if product_id <= 0 {
        return Err(ApiError::bad_request("product_id is required."));
    }
    if args["quantity"].as_i64().is_none() {
        return Err(ApiError::bad_request("quantity is required."));
    }
    let source_id = arg_i64(args, "source_id", 1);
    let quantity = arg_i64(args, "quantity", 0).max(0);
    crate::support::admin_crud::update_product(
        product_id,
        &json!({ "inventories": { source_id.to_string(): quantity } }),
    )
    .await?;
    get_product_stock(product_id).await
}

async fn create_category(args: &Value) -> ApiResult<Value> {
    let category_id = crate::support::admin_crud::create_category(args).await?;
    Ok(json!({ "category_id": category_id }))
}

async fn update_category(args: &Value) -> ApiResult<Value> {
    let category_id = arg_i64(args, "category_id", 0);
    if category_id <= 0 {
        return Err(ApiError::bad_request("category_id is required."));
    }
    let locale = args["locale"].as_str().unwrap_or("en");
    crate::support::admin_crud::update_category(category_id, locale, args).await?;
    Ok(json!({ "category_id": category_id }))
}

async fn delete_category(args: &Value) -> ApiResult<Value> {
    let category_id = arg_i64(args, "category_id", 0);
    if category_id <= 0 {
        return Err(ApiError::bad_request("category_id is required."));
    }
    crate::support::admin_crud::delete_category(category_id).await?;
    Ok(json!({ "category_id": category_id, "deleted": true }))
}

async fn create_cms_page(args: &Value) -> ApiResult<Value> {
    let page_id = crate::support::admin_crud::create_cms_page(args).await?;
    Ok(json!({ "page_id": page_id }))
}

async fn update_cms_page(args: &Value) -> ApiResult<Value> {
    let page_id = arg_i64(args, "page_id", 0);
    if page_id <= 0 {
        return Err(ApiError::bad_request("page_id is required."));
    }
    crate::support::admin_crud::update_cms_page(page_id, args).await?;
    Ok(json!({ "page_id": page_id }))
}

async fn delete_cms_page(args: &Value) -> ApiResult<Value> {
    let page_id = arg_i64(args, "page_id", 0);
    if page_id <= 0 {
        return Err(ApiError::bad_request("page_id is required."));
    }
    crate::support::admin_crud::delete_cms_page(page_id).await?;
    Ok(json!({ "page_id": page_id, "deleted": true }))
}

async fn create_customer(args: &Value) -> ApiResult<Value> {
    let customer_id = crate::support::admin_crud::create_customer(args).await?;
    Ok(json!({ "customer_id": customer_id }))
}

async fn update_customer(args: &Value) -> ApiResult<Value> {
    let customer_id = arg_i64(args, "customer_id", 0);
    if customer_id <= 0 {
        return Err(ApiError::bad_request("customer_id is required."));
    }
    let allowed = [
        "first_name",
        "last_name",
        "email",
        "phone",
        "gender",
        "date_of_birth",
        "customer_group_id",
        "status",
        "subscribed_to_news_letter",
    ];
    let mut payload = larust_support::serde_json::Map::new();
    for key in allowed {
        if !args[key].is_null() {
            payload.insert(key.to_string(), args[key].clone());
        }
    }
    crate::support::admin_crud::update_customer(customer_id, &Value::Object(payload)).await?;
    Ok(json!({ "customer_id": customer_id }))
}

async fn add_order_comment(args: &Value, admin_id: i64) -> ApiResult<Value> {
    let order_id = arg_i64(args, "order_id", 0);
    if order_id <= 0 {
        return Err(ApiError::bad_request("order_id is required."));
    }
    let comment = arg_str(args, "comment");
    let comment_id = crate::support::admin_crud::add_order_comment(
        order_id,
        admin_id,
        &comment,
        args["customer_notified"].as_bool().unwrap_or(false),
    )
    .await?;
    Ok(json!({ "order_id": order_id, "comment_id": comment_id }))
}

async fn create_shipment_tool(args: &Value) -> ApiResult<Value> {
    let order_id = arg_i64(args, "order_id", 0);
    if order_id <= 0 {
        return Err(ApiError::bad_request("order_id is required."));
    }
    let source_id = arg_i64(args, "source_id", 1);
    let requested = args["items"].as_object().cloned().unwrap_or_default();
    let shipment = sales::create_shipment(order_id, source_id, &requested).await?;
    Ok(json!({ "shipment": shipment }))
}

async fn create_refund_tool(args: &Value) -> ApiResult<Value> {
    let order_id = arg_i64(args, "order_id", 0);
    if order_id <= 0 {
        return Err(ApiError::bad_request("order_id is required."));
    }
    let refund = crate::support::admin_crud::create_refund(order_id, args).await?;
    Ok(json!({ "refund": refund }))
}

async fn save_configuration(args: &Value) -> ApiResult<Value> {
    let Some(values) = args["values"].as_object() else {
        return Err(ApiError::bad_request("values is required."));
    };
    crate::support::admin_crud::save_configuration(values).await?;
    Ok(json!({ "saved": values.len() }))
}

/// Executes an MCP tool. Unknown names are rejected before reaching this
/// function by the controller (JSON-RPC `-32602`), so a missing match here is
/// also a bad request.
pub async fn call_tool(name: &str, args: &Value, admin_id: i64) -> ApiResult<Value> {
    match name {
        "list_products" => list_products(args).await,
        "get_product" => {
            let product_id = arg_i64(args, "product_id", 0);
            products::detail(product_id)
                .await
                .ok_or_else(|| ApiError::not_found("Product not found."))
        }
        "list_orders" => list_orders(args).await,
        "get_order" => sales::order_resource(arg_i64(args, "order_id", 0)).await,
        "create_invoice" => {
            let invoice = sales::create_invoice(arg_i64(args, "order_id", 0)).await?;
            Ok(json!({ "invoice": invoice }))
        }
        "cancel_order" => {
            let order_id = arg_i64(args, "order_id", 0);
            sales::cancel_order(order_id).await?;
            Ok(json!({ "order_id": order_id, "status": "canceled" }))
        }
        "list_customers" => list_customers(args).await,
        "list_transactions" => list_transactions(arg_i64(args, "order_id", 0)).await,
        "list_refunds" => list_refunds(arg_i64(args, "order_id", 0)).await,
        "sales_summary" => sales_summary(args).await,
        "list_categories" => list_categories(args).await,
        "get_category" => get_category(arg_i64(args, "category_id", 0)).await,
        "list_attribute_families" => list_attribute_families().await,
        "list_attributes" => list_attributes().await,
        "get_customer" => get_customer(arg_i64(args, "customer_id", 0)).await,
        "list_invoices" => list_invoices(args).await,
        "get_invoice" => get_invoice(arg_i64(args, "invoice_id", 0)).await,
        "list_shipments" => list_shipments(args).await,
        "get_shipment" => get_shipment(arg_i64(args, "shipment_id", 0)).await,
        "list_cart_rules" => list_cart_rules().await,
        "get_cart_rule" => get_cart_rule(arg_i64(args, "rule_id", 0)).await,
        "list_catalog_rules" => list_catalog_rules().await,
        "get_catalog_rule" => get_catalog_rule(arg_i64(args, "rule_id", 0)).await,
        "list_cms_pages" => list_cms_pages(args).await,
        "get_cms_page" => get_cms_page(arg_i64(args, "page_id", 0)).await,
        "list_channels" => list_channels(args).await,
        "list_locales" => list_locales().await,
        "list_currencies" => list_currencies().await,
        "list_tax_rates" => list_tax_rates().await,
        "list_inventory_sources" => list_inventory_sources().await,
        "list_reviews" => list_reviews(args).await,
        "low_stock_products" => low_stock_products(args).await,
        "get_product_stock" => get_product_stock(arg_i64(args, "product_id", 0)).await,
        "create_product" => create_product(args).await,
        "update_product" => update_product(args).await,
        "delete_product" => delete_product(args).await,
        "set_product_stock" => set_product_stock(args).await,
        "create_category" => create_category(args).await,
        "update_category" => update_category(args).await,
        "delete_category" => delete_category(args).await,
        "create_cms_page" => create_cms_page(args).await,
        "update_cms_page" => update_cms_page(args).await,
        "delete_cms_page" => delete_cms_page(args).await,
        "create_customer" => create_customer(args).await,
        "update_customer" => update_customer(args).await,
        "add_order_comment" => add_order_comment(args, admin_id).await,
        "create_shipment" => create_shipment_tool(args).await,
        "create_refund" => create_refund_tool(args).await,
        "save_configuration" => save_configuration(args).await,
        other => Err(ApiError::bad_request(format!("Unknown MCP tool: {other}"))),
    }
}
