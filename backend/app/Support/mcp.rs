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

/// Executes an MCP tool. Unknown names are rejected before reaching this
/// function by the controller (JSON-RPC `-32602`), so a missing match here is
/// also a bad request.
pub async fn call_tool(name: &str, args: &Value) -> ApiResult<Value> {
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
        other => Err(ApiError::bad_request(format!("Unknown MCP tool: {other}"))),
    }
}
