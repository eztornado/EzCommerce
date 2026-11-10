use larust_http::{Route, Router};
use larust_support::axum::Json;
use larust_support::serde_json::json;

/// The API has no browser-facing pages; this keeps `/` informative and lets
/// deployment health checks use either `/` or `/up`.
pub fn routes() -> Router {
    Route::get("/", index)
}

async fn index() -> Json<larust_support::serde_json::Value> {
    Json(json!({
        "name": crate::config::app::config()["app_name"],
        "status": "ok",
    }))
}
