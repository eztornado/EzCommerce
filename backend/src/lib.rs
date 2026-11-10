#[path = "../config/mod.rs"]
pub mod config;
#[path = "../database/mod.rs"]
pub mod database;
#[path = "../app/Console/Commands/mod.rs"]
pub mod commands;
#[path = "../app/Http/Controllers/mod.rs"]
pub mod controllers;
#[path = "../app/Http/Middleware/mod.rs"]
pub mod middleware;
#[path = "../app/Mail/mod.rs"]
pub mod mail;
#[path = "../app/Models/mod.rs"]
pub mod models;
#[path = "../app/Support/mod.rs"]
pub mod support;
#[path = "../database/seeders/mod.rs"]
pub mod seeders;
#[path = "../routes/mod.rs"]
pub mod routes;

use larust_support::axum::http::{HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn paths() -> larust_core::AppPaths {
    larust_core::AppPaths::new(larust_core::resolve_app_root(env!("CARGO_MANIFEST_DIR")))
}

pub async fn connect_database() -> Result<(), larust_core::AppError> {
    larust_support::orm::connect(&database_url()?).await
}

fn database_url() -> Result<String, larust_core::AppError> {
    let url = config::database::config().default_connection_url()?;
    let relative_sqlite_path = url
        .strip_prefix("sqlite://")
        .filter(|path| !path.starts_with('/') && !path.starts_with(":memory:"))
        .map(str::to_string);

    Ok(match relative_sqlite_path {
        Some(relative) => {
            let path = paths().join(relative);
            format!("sqlite:///{}", path.to_string_lossy().replace('\\', "/"))
        }
        None => url,
    })
}

pub fn port() -> u16 {
    config::app::config()["app_port"]
        .as_u64()
        .and_then(|value| u16::try_from(value).ok())
        .unwrap_or(34187)
}

pub fn application() -> Result<larust_core::Application, larust_core::AppError> {
    let app = larust_core::Application::at_root(
        larust_core::resolve_app_root(env!("CARGO_MANIFEST_DIR")),
        config::app::config,
    )?;
    Ok(app.with_error_pages(larust_core::ErrorPages {
        not_found: larust_support::error_view!("404"),
        internal: larust_support::error_view!("500"),
    }))
}

pub fn router(app: &larust_core::Application) -> larust_http::Router {
    let api = routes::api::routes();
    let cors = cors_layer();
    let api = api.middleware(cors);
    routes::web::routes().merge(&app.config().api_prefix, api)
}

fn cors_layer() -> CorsLayer {
    // The Flutter web build and any localhost dev server must be able to call
    // the API; production origins come from `.env`.
    let mut origins: Vec<HeaderValue> = Vec::new();
    let front = larust_support::config_env::env("FRONTEND_URL");
    let front = if front.is_empty() {
        "http://localhost:8080".to_string()
    } else {
        front
    };
    let www = larust_support::config_env::env("FRONTEND_WWW_URL");
    let www = if www.is_empty() {
        "http://localhost:3000".to_string()
    } else {
        www
    };
    for origin in [front, www] {
        if let Ok(value) = origin.parse::<HeaderValue>() {
            origins.push(value);
        }
    }

    let allow_origin = AllowOrigin::predicate(move |origin: &HeaderValue, _| {
        let Ok(origin) = origin.to_str() else {
            return false;
        };
        let origin = origin.to_ascii_lowercase();
        if origins
            .iter()
            .any(|allowed| allowed.as_bytes() == origin.as_bytes())
        {
            return true;
        }
        let localhost =
            regex::Regex::new(r"^https?://(localhost|127\.0\.0\.1)(:\d+)?$").unwrap();
        localhost.is_match(&origin)
    });

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            larust_support::axum::http::header::CONTENT_TYPE,
            larust_support::axum::http::header::AUTHORIZATION,
            larust_support::axum::http::header::ACCEPT,
            larust_support::axum::http::HeaderName::from_static("x-requested-with"),
        ])
        .max_age(std::time::Duration::from_secs(86400))
}

/// Connects the database and serves the API. The API is stateless (JWT
/// bearer tokens), so Larust's cookie session layer is not attached.
pub async fn serve() -> Result<(), larust_core::AppError> {
    let app = application()?;
    let route = router(&app);

    connect_database().await?;

    app.with_health_route("/up")
        .router(route.into_axum_router())
        .serve()
        .await
}

/// Converts a `larust_http::Router` into a raw axum router (used by tests).
pub fn into_axum(router: larust_http::Router) -> larust_support::axum::Router {
    router.into_axum_router()
}
