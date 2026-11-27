use larust_http::{Route, Router};
use larust_support::axum::response::Redirect;
use larust_support::axum::Json;
use larust_support::serde_json::json;

use crate::controllers::{AgenticController, CoreController};

/// The API has no browser-facing pages; this keeps `/` informative and lets
/// deployment health checks use either `/` or `/up`.
pub fn routes() -> Router {
    Route::get("/", index)
        .get("/docs", docs_redirect)
        // `/storage/...` must go through the controller (instead of the
        // ServeDir fallback) to set long-lived cache headers for optimized
        // product variants.
        .get("/storage/{folder}/{file}", CoreController::storage)
        // Navegación agéntica (#40): llms.txt y manifiesto ARD en la raíz del
        // dominio, tal y como los piden los agentes y las auditorías de
        // PageSpeed. Bajo `/api/` existen rutas equivalentes.
        .get("/llms.txt", AgenticController::llms_txt)
        .get("/ai-catalog.json", AgenticController::ai_catalog)
        .get("/.well-known/ard.json", AgenticController::ai_catalog)
        .get("/.well-known/ai-catalog.json", AgenticController::ai_catalog)
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
