//! JSON response helpers following Bagisto's shop API contract:
//!
//! - Simple resources answer `{data: ..., message: ...}` (Laravel `JsonResource`).
//! - Paginated collections answer `{data: [...], links: {...}, meta: {...}}`.
//! - Errors answer `{message: "..."}` with the matching HTTP status.
//! - Validation failures answer Laravel's `{message, errors: {field: [...]}}` (422).

use larust_support::axum::http::StatusCode;
use larust_support::axum::response::{IntoResponse, Response};
use larust_support::axum::Json;
use larust_support::serde_json::{json, Map, Value};

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn unprocessable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, message)
    }

    pub fn too_many(message: impl Into<String>) -> Self {
        Self::new(StatusCode::TOO_MANY_REQUESTS, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        message(self.status, self.message).into_response()
    }
}

impl From<ApiError> for Response {
    fn from(error: ApiError) -> Self {
        error.into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(source: sqlx::Error) -> Self {
        eprintln!("sqlx error: {source}");
        larust_support::tracing::error!(%source, "database error");
        Self::internal("Internal Server Error")
    }
}

impl From<larust_core::AppError> for ApiError {
    fn from(source: larust_core::AppError) -> Self {
        larust_support::tracing::error!(%source, "application error");
        Self::internal("Internal Server Error")
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(source: serde_json::Error) -> Self {
        Self::unprocessable(source.to_string())
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(source: reqwest::Error) -> Self {
        larust_support::tracing::error!(%source, "http client error");
        Self::internal("External service error")
    }
}

/// `{data: ..., message: ...}` - the `JsonResource(['data' => ..., 'message' => ...])` shape.
pub fn resource(data: impl Into<Value>, message: impl Into<Value>) -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "data": data.into(),
            "message": message.into(),
        })),
    )
        .into_response()
}

/// Plain JSON body (used where Bagisto returns `response()->json($data)`).
pub fn json_ok(data: impl Into<Value>) -> Response {
    (StatusCode::OK, Json(data.into())).into_response()
}

/// Laravel paginator JSON: `{data, links, meta}`.
pub fn paginated(records: Vec<Value>, meta: Value) -> Response {
    let meta = meta.as_object().cloned().unwrap_or_else(Map::new);
    let current_page = meta
        .get("current_page")
        .and_then(Value::as_i64)
        .unwrap_or(1);
    let last_page = meta.get("last_page").and_then(Value::as_i64).unwrap_or(1);
    let path = meta
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let page_url = |page: i64| {
        if page < 1 || page > last_page {
            Value::Null
        } else {
            Value::String(format!("{}?page={}", path, page))
        }
    };

    (
        StatusCode::OK,
        Json(json!({
            "data": records,
            "links": {
                "first": page_url(1),
                "last": page_url(last_page),
                "prev": page_url(current_page - 1),
                "next": page_url(current_page + 1),
            },
            "meta": meta,
        })),
    )
        .into_response()
}

/// `{message: "..."}` error body, matching Bagisto's error responses.
pub fn message(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(json!({ "message": message.into() }))).into_response()
}

/// 401 like `AuthenticateCustomer`: `{"message": ""}` (empty message).
pub fn unauthorized() -> Response {
    (StatusCode::UNAUTHORIZED, Json(json!({ "message": "" }))).into_response()
}

/// Laravel validator failure: `{"message": "<first>", "errors": {...}}` (422).
pub fn validation_error(errors: Vec<(String, String)>) -> Response {
    let first = errors
        .first()
        .map(|(_, message)| message.clone())
        .unwrap_or_default();
    let mut map = Map::new();
    for (field, message) in errors {
        match map.get_mut(&field).and_then(Value::as_array_mut) {
            Some(messages) => messages.push(Value::String(message)),
            None => {
                map.insert(field, json!([message]));
            }
        }
    }
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({
            "message": first,
            "errors": map,
        })),
    )
        .into_response()
}

/// Respuesta de descarga CSV con `Content-Disposition: attachment`.
pub fn csv_response(csv: &str, filename: &str) -> Response {
    use larust_support::axum::http::header;
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                Box::leak(format!("attachment; filename=\"{filename}\"").into_boxed_str()),
            ),
        ],
        csv.to_string(),
    )
        .into_response()
}
