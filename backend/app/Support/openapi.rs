//! OpenAPI 3.0 document built from the live router snapshot plus the curated
//! MCP operation. Served at `{api_prefix}/openapi.json`.

use std::sync::OnceLock;

use larust_http::RouteInfo;
use larust_support::serde_json::{json, Map, Value};

static ROUTES: OnceLock<Vec<RouteInfo>> = OnceLock::new();

/// Called once while the router is assembled.
pub fn capture_routes(routes: Vec<RouteInfo>) {
    let _ = ROUTES.set(routes);
}

fn routes_snapshot() -> Vec<RouteInfo> {
    ROUTES.get().cloned().unwrap_or_default()
}

fn api_prefix() -> String {
    crate::config::app::config()["api_prefix"]
        .as_str()
        .unwrap_or("/api")
        .trim_end_matches('/')
        .to_string()
}

fn humanize(value: &str) -> String {
    let cleaned = value.replace(['_', '.', '-'], " ");
    let mut chars = cleaned.trim().chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn verb(method: &str) -> &'static str {
    match method {
        "get" => "Get",
        "post" => "Submit",
        "put" => "Update",
        "patch" => "Update",
        "delete" => "Delete",
        _ => "Call",
    }
}

fn summary(method: &str, path: &str, name: Option<&str>) -> String {
    if let Some(name) = name {
        let humanized = humanize(name);
        if !humanized.is_empty() {
            return humanized;
        }
    }
    let segments: Vec<&str> = path
        .split('/')
        .filter(|segment| !segment.is_empty() && !segment.starts_with('{'))
        .collect();
    let tail = segments
        .iter()
        .rev()
        .take(3)
        .rev()
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    if tail.is_empty() {
        verb(method).to_string()
    } else {
        format!("{} {}", verb(method), tail)
    }
}

fn operation_id(method: &str, path: &str) -> String {
    let cleaned = path
        .replace(['{', '}', '/'], "_")
        .replace(['-', '.'], "_");
    format!("{}_{}", method, cleaned.trim_matches('_'))
}

fn tag_for(path: &str) -> &'static str {
    if path == "/mcp" {
        "MCP"
    } else if path.starts_with("/docs") || path == "/openapi.json" {
        "Documentation"
    } else if path.starts_with("/admin/sales") {
        "Sales"
    } else if path.starts_with("/admin/catalog") {
        "Catalog"
    } else if path.starts_with("/admin/customers") {
        "Customers"
    } else if path.starts_with("/admin/settings") || path.starts_with("/admin/configuration") {
        "Settings"
    } else if path.starts_with("/admin/reporting") {
        "Reporting"
    } else if path.starts_with("/admin/marketing") {
        "Marketing"
    } else if path.starts_with("/admin/cms") {
        "CMS"
    } else if path.starts_with("/admin/communication") {
        "Communications"
    } else if path.starts_with("/admin/seo") {
        "SEO"
    } else if path.starts_with("/admin/appearance") {
        "Appearance"
    } else if path.starts_with("/admin/data-transfer") {
        "Data Transfer"
    } else if path.starts_with("/admin") {
        "Admin"
    } else if path.starts_with("/customer") {
        "Customer Account"
    } else if path.starts_with("/checkout") {
        "Checkout"
    } else if path.starts_with("/products")
        || path.starts_with("/product/")
        || path.starts_with("/categories")
    {
        "Catalog"
    } else if path.starts_with("/webhooks") {
        "Webhooks"
    } else {
        "Storefront"
    }
}

fn requires_bearer(path: &str) -> bool {
    if path == "/admin/login"
        || path == "/customer/login"
        || path == "/customer/register"
        || path == "/customer/forgot-password"
        || path == "/customer/reset-password"
    {
        return false;
    }
    path.starts_with("/admin") || path.starts_with("/customer") || path == "/mcp"
}

fn path_parameters(path: &str) -> Value {
    let mut parameters = Vec::new();
    let mut rest = path;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            break;
        };
        let name = &after[..end];
        let kind = if name == "id" || name.ends_with("_id") {
            "integer"
        } else {
            "string"
        };
        parameters.push(json!({
            "name": name,
            "in": "path",
            "required": true,
            "schema": { "type": kind }
        }));
        rest = &after[end + 1..];
    }
    Value::Array(parameters)
}

fn generic_request_body() -> Value {
    json!({
        "required": false,
        "content": {
            "application/json": {
                "schema": { "type": "object", "additionalProperties": true }
            }
        }
    })
}

fn responses(method: &str, secured: bool, path: &str) -> Value {
    let mut responses = Map::new();
    responses.insert(
        "200".into(),
        json!({ "description": "Successful response" }),
    );
    if secured {
        responses.insert(
            "401".into(),
            json!({ "description": "Missing or invalid JWT", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Error" } } } }),
        );
    }
    if matches!(method, "post" | "put" | "patch") {
        responses.insert(
            "422".into(),
            json!({ "description": "Validation error", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Error" } } } }),
        );
    }
    if path.contains('{') {
        responses.insert(
            "404".into(),
            json!({ "description": "Resource not found", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Error" } } } }),
        );
    }
    Value::Object(responses)
}

fn operation(method: &str, path: &str, name: Option<&str>) -> Value {
    let secured = requires_bearer(path);
    let mut operation = json!({
        "tags": [tag_for(path)],
        "summary": summary(method, path, name),
        "operationId": operation_id(method, path),
        "parameters": path_parameters(path),
        "responses": responses(method, secured, path)
    });
    if secured {
        operation["security"] = json!([{ "bearerAuth": [] }]);
    }
    if matches!(method, "post" | "put" | "patch") {
        operation["requestBody"] = generic_request_body();
    }
    operation
}

fn mcp_operation() -> Value {
    json!({
        "tags": ["MCP"],
        "summary": "Model Context Protocol (MCP) endpoint",
        "description": "Stateless JSON-RPC 2.0 over HTTP. Authenticate with the same admin JWT used by /api/admin (Authorization: Bearer <token>). Supported methods: initialize, notifications/initialized, ping, tools/list and tools/call. The available tools are listed in x-mcp-tools and in components/schemas.",
        "operationId": "post_mcp",
        "security": [{ "bearerAuth": [] }],
        "requestBody": {
            "required": true,
            "content": {
                "application/json": {
                    "schema": { "$ref": "#/components/schemas/McpJsonRpcRequest" },
                    "examples": {
                        "toolsList": {
                            "summary": "List tools",
                            "value": { "jsonrpc": "2.0", "id": 1, "method": "tools/list" }
                        },
                        "toolsCall": {
                            "summary": "Call a tool",
                            "value": {
                                "jsonrpc": "2.0",
                                "id": 2,
                                "method": "tools/call",
                                "params": {
                                    "name": "list_orders",
                                    "arguments": { "status": "processing", "per_page": 10 }
                                }
                            }
                        }
                    }
                }
            }
        },
        "responses": {
            "200": {
                "description": "JSON-RPC response",
                "content": {
                    "application/json": {
                        "schema": { "$ref": "#/components/schemas/McpJsonRpcResponse" }
                    }
                }
            },
            "202": { "description": "Notification accepted" },
            "401": {
                "description": "Missing or invalid admin JWT",
                "content": {
                    "application/json": {
                        "schema": { "$ref": "#/components/schemas/Error" }
                    }
                }
            }
        },
        "x-mcp-tools": crate::support::mcp::tools()
    })
}

fn docs_operations(paths: &mut Map<String, Value>, prefix: &str) {
    let mcp_path = format!("{prefix}/mcp");
    if let Some(entry) = paths.get_mut(&mcp_path).and_then(Value::as_object_mut) {
        entry.insert("post".into(), mcp_operation());
    }
    let docs_path = format!("{prefix}/docs");
    if let Some(entry) = paths.get_mut(&docs_path).and_then(Value::as_object_mut) {
        entry.insert(
            "get".into(),
            json!({
                "tags": ["Documentation"],
                "summary": "Swagger UI",
                "description": "Interactive Swagger UI for this API. Public route.",
                "operationId": "get_docs",
                "responses": {
                    "200": {
                        "description": "Swagger UI HTML",
                        "content": { "text/html": { "schema": { "type": "string" } } }
                    }
                }
            }),
        );
    }
    let spec_path = format!("{prefix}/openapi.json");
    if let Some(entry) = paths.get_mut(&spec_path).and_then(Value::as_object_mut) {
        entry.insert(
            "get".into(),
            json!({
                "tags": ["Documentation"],
                "summary": "OpenAPI document",
                "description": "OpenAPI 3.0 JSON document. Public route.",
                "operationId": "get_openapi_json",
                "responses": {
                    "200": {
                        "description": "OpenAPI 3.0 document",
                        "content": { "application/json": { "schema": { "type": "object" } } }
                    }
                }
            }),
        );
    }
}

pub fn spec() -> Value {
    let prefix = api_prefix();
    let mut paths: Map<String, Value> = Map::new();

    for route in routes_snapshot() {
        let method = route.method.to_ascii_lowercase();
        if !matches!(method.as_str(), "get" | "post" | "put" | "patch" | "delete") {
            continue;
        }
        let full_path = format!("{prefix}{}", route.path);
        let entry = paths
            .entry(full_path)
            .or_insert_with(|| Value::Object(Map::new()));
        if let Some(object) = entry.as_object_mut() {
            object.insert(
                method.clone(),
                operation(&method, &route.path, route.name.as_deref()),
            );
        }
    }
    docs_operations(&mut paths, &prefix);

    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "EzCommerce API",
            "version": "1.0.0",
            "description": "JSON API for the EzCommerce storefront, admin panel and integrations. Authenticate with `Authorization: Bearer <JWT>`; admin endpoints use admin tokens and customer endpoints use customer tokens. The Stripe webhook is public and verified by its signature. The MCP endpoint uses the same admin JWT."
        },
        "servers": [
            { "url": "/", "description": "Current host" }
        ],
        "tags": [
            { "name": "Documentation", "description": "OpenAPI document and Swagger UI." },
            { "name": "MCP", "description": "Model Context Protocol JSON-RPC endpoint." },
            { "name": "Storefront", "description": "Public storefront endpoints." },
            { "name": "Catalog", "description": "Products, categories and attributes." },
            { "name": "Checkout", "description": "Cart and one-page checkout." },
            { "name": "Customer Account", "description": "Authenticated customer endpoints." },
            { "name": "Admin", "description": "Back-office endpoints." },
            { "name": "Sales", "description": "Orders, invoices, shipments, refunds and payments." },
            { "name": "Customers", "description": "Customers, groups and reviews." },
            { "name": "Marketing", "description": "Cart rules and catalog rules." },
            { "name": "CMS", "description": "Pages and content." },
            { "name": "Communications", "description": "Email templates, events and campaigns." },
            { "name": "SEO", "description": "URL rewrites, search terms and synonyms." },
            { "name": "Reporting", "description": "Sales reporting." },
            { "name": "Settings", "description": "Channels, locales, currencies, taxes and configuration." },
            { "name": "Appearance", "description": "Theme sections." },
            { "name": "Data Transfer", "description": "Import and export." },
            { "name": "Webhooks", "description": "External gateway callbacks." }
        ],
        "paths": Value::Object(paths),
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT",
                    "description": "JWT access token returned by /api/admin/login or /api/customer/login."
                }
            },
            "schemas": {
                "Error": {
                    "type": "object",
                    "properties": {
                        "message": { "type": "string" },
                        "errors": { "type": "object", "additionalProperties": true }
                    }
                },
                "McpJsonRpcRequest": {
                    "type": "object",
                    "required": ["jsonrpc", "method"],
                    "properties": {
                        "jsonrpc": { "type": "string", "enum": ["2.0"] },
                        "id": {
                            "oneOf": [
                                { "type": "string" },
                                { "type": "number" },
                                { "type": "null" }
                            ]
                        },
                        "method": {
                            "type": "string",
                            "enum": [
                                "initialize",
                                "notifications/initialized",
                                "ping",
                                "tools/list",
                                "tools/call",
                                "resources/list",
                                "prompts/list"
                            ]
                        },
                        "params": { "type": "object", "additionalProperties": true }
                    }
                },
                "McpJsonRpcResponse": {
                    "type": "object",
                    "properties": {
                        "jsonrpc": { "type": "string" },
                        "id": {},
                        "result": { "type": "object", "additionalProperties": true },
                        "error": {
                            "type": "object",
                            "properties": {
                                "code": { "type": "integer" },
                                "message": { "type": "string" },
                                "data": {}
                            }
                        }
                    }
                }
            }
        }
    })
}
