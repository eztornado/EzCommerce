//! Stateless MCP JSON-RPC endpoint (`POST {api_prefix}/mcp`), protected by the
//! regular admin JWT.

use larust_support::axum::body::Bytes;
use larust_support::axum::http::StatusCode;
use larust_support::axum::response::{IntoResponse, Response};
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::mcp;

pub struct McpController;

fn rpc_ok(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}

fn tool_result(result: Value) -> Value {
    let text = larust_support::serde_json::to_string_pretty(&result)
        .unwrap_or_else(|_| result.to_string());
    json!({
        "content": [{ "type": "text", "text": text }],
        "structuredContent": result,
        "isError": false
    })
}

fn tool_error(message: String) -> Value {
    json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true
    })
}

async fn call_tool(params: &Value) -> Result<Value, (i64, String)> {
    let name = params["name"]
        .as_str()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or((-32602, "Missing tool name.".to_string()))?;
    let known = mcp::tools()
        .as_array()
        .map(|tools| {
            tools
                .iter()
                .any(|tool| tool["name"].as_str() == Some(name))
        })
        .unwrap_or(false);
    if !known {
        return Err((-32602, format!("Unknown tool: {name}")));
    }
    let arguments = params
        .get("arguments")
        .cloned()
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}));
    match mcp::call_tool(name, &arguments).await {
        Ok(result) => Ok(tool_result(result)),
        Err(error) => Ok(tool_error(error.message)),
    }
}

async fn process_request(request: Value) -> Option<Value> {
    let id = request.get("id").cloned();
    let notification = id.is_none();
    let Some(object) = request.as_object() else {
        return if notification {
            None
        } else {
            Some(rpc_error(Value::Null, -32600, "Invalid Request"))
        };
    };
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return if notification {
            None
        } else {
            Some(rpc_error(
                id.unwrap_or(Value::Null),
                -32600,
                "Invalid Request",
            ))
        };
    }
    let Some(method) = object.get("method").and_then(Value::as_str) else {
        return if notification {
            None
        } else {
            Some(rpc_error(
                id.unwrap_or(Value::Null),
                -32600,
                "Invalid Request",
            ))
        };
    };
    let params = object.get("params").cloned().unwrap_or_else(|| json!({}));

    if notification {
        // `notifications/initialized` and future notifications need no reply.
        return None;
    }
    let id = id.unwrap_or(Value::Null);

    let result: Result<Value, (i64, String)> = match method {
        "initialize" => {
            let protocol = params["protocolVersion"]
                .as_str()
                .unwrap_or("2025-06-18");
            Ok(json!({
                "protocolVersion": protocol,
                "capabilities": {
                    "tools": { "listChanged": false },
                    "resources": { "listChanged": false },
                    "prompts": { "listChanged": false }
                },
                "serverInfo": {
                    "name": "EzCommerce MCP",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": "EzCommerce administration tools. Authenticate with an admin JWT. Use tools/list to discover operations and tools/call to execute them."
            }))
        }
        "notifications/initialized" => Ok(json!({})),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": mcp::tools() })),
        "tools/call" => call_tool(&params).await,
        "resources/list" | "resources/templates/list" => {
            Ok(json!({ "resources": [], "resourceTemplates": [] }))
        }
        "prompts/list" => Ok(json!({ "prompts": [] })),
        _ => {
            return Some(rpc_error(id, -32601, "Method not found"));
        }
    };

    match result {
        Ok(result) => Some(rpc_ok(id, result)),
        Err((code, message)) => Some(rpc_error(id, code, &message)),
    }
}

impl McpController {
    /// `POST {api_prefix}/mcp` - JSON-RPC 2.0 batch or single request.
    pub async fn handle(admin: auth::Admin, body: Bytes) -> Response {
        let _ = admin;
        let payload: Value = match larust_support::serde_json::from_slice(&body) {
            Ok(payload) => payload,
            Err(_) => {
                return Json(rpc_error(Value::Null, -32700, "Parse error"))
                    .into_response();
            }
        };

        if let Value::Array(requests) = payload {
            let mut responses = Vec::new();
            for request in requests {
                if let Some(response) = process_request(request).await {
                    responses.push(response);
                }
            }
            if responses.is_empty() {
                return StatusCode::ACCEPTED.into_response();
            }
            return Json(Value::Array(responses)).into_response();
        }

        match process_request(payload).await {
            Some(response) => Json(response).into_response(),
            None => StatusCode::ACCEPTED.into_response(),
        }
    }
}
