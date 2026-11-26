use larust_http::{Route, Router};
use larust_support::axum::response::Redirect;
use larust_support::axum::Json;
use larust_support::serde_json::json;

/// The API has no browser-facing pages; this keeps `/` informative and lets
/// deployment health checks use either `/` or `/up`.
pub fn routes() -> Router {
    Route::get("/", index).get("/docs", docs_redirect)
}

async fn index() -> Json<larust_support::serde_json::Value> {
    Json(json!({
        "name": crate::config::app::config()["app_name"],
        "status": "ok",
    }))
}

/// Convenience alias: `/docs` -> `/api/docs`.
async fn docs_redirect() -> Redirect {
    Redirect::permanent("/api/docs")
}
