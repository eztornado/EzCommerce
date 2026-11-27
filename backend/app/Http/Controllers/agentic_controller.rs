//! Recursos de navegación agéntica (#40): `llms.txt` y manifiesto ARD
//! (`ai-catalog.json` / `.well-known/ard.json`). Se generan dinámicamente
//! desde el canal y las categorías reales de la BD.
//!
//! Especificaciones:
//! - llms.txt: https://llmstxt.org (Markdown con H1 al inicio y enlaces
//!   `[texto](url)`; Lighthouse exige ambos).
//! - ARD (Agentic Resource Discovery): https://agenticresourcediscovery.org —
//!   manifiesto con array raíz `entries`; cada entrada requiere `identifier`
//!   (URN `urn:air:<fqdn>:[ns:]<nombre>`), `displayName`, `type` (media type)
//!   y exactamente una de `url`/`data`; `representativeQueries` (2-5) es
//!   recomendado (su ausencia baja la auditoría a 0.9).

use std::collections::HashMap;

use larust_support::axum::extract::Query;
use larust_support::axum::http::{header, HeaderMap, StatusCode};
use larust_support::axum::response::{IntoResponse, Response};
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::core;
use crate::support::db;

pub struct AgenticController;

/// Contexto común de los dos recursos: canal, locale y URL base absoluta.
struct SiteContext {
    name: String,
    locale: String,
    base_url: String,
    domain: String,
}

async fn site_context(query: &HashMap<String, String>, headers: &HeaderMap) -> SiteContext {
    let channel = core::current_channel(headers).await
        .map(|channel| channel)
        .unwrap_or_default();

    let name = db::select_one(
        "SELECT `name` FROM `channel_translations` WHERE `channel_id` = ? ORDER BY `id` LIMIT 1",
        &[channel["id"].clone()],
    )
    .await
    .ok()
    .flatten()
    .and_then(|row| row["name"].as_str().map(String::from))
    .unwrap_or_else(|| "EzCommerce".to_string());

    let mut locale = query.get("locale").cloned().unwrap_or_default();
    if locale.is_empty() {
        locale = db::select_one(
            "SELECT `code` FROM `locales` WHERE `id` = ? LIMIT 1",
            &[channel["default_locale_id"].clone()],
        )
        .await
        .ok()
        .flatten()
        .and_then(|row| row["code"].as_str().map(String::from))
        .unwrap_or_else(|| "es".to_string());
    }

    // Host completo (con puerto si es no estándar, p. ej. dev) y dominio
    // sin puerto (el FQDN del publicador para el URN).
    let host_full = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|| "localhost".to_string());
    let domain = host_full.split(':').next().unwrap_or(&host_full).to_string();
    let forwarded_proto = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(|value| value.trim().to_string());
    let scheme = forwarded_proto.unwrap_or_else(|| {
        // Detrás de nginx/production siempre llega X-Forwarded-Proto; en
        // desarrollo local (vite) no y el host es localhost/127.0.0.1.
        let local = domain == "localhost" || domain == "127.0.0.1" || domain.ends_with(".local");
        if local { "http".into() } else { "https".into() }
    });

    SiteContext {
        name,
        locale,
        base_url: format!("{scheme}://{host_full}"),
        domain,
    }
}

/// Categorías visibles del canal (id + nombre traducido) para los enlaces.
async fn channel_categories(channel_id: i64, locale: &str) -> Vec<(String, i64)> {
    let root_id = db::select_one(
        "SELECT `root_category_id` FROM `channels` WHERE `id` = ? LIMIT 1",
        &[Value::from(channel_id)],
    )
    .await
    .ok()
    .flatten()
    .and_then(|row| row["root_category_id"].as_i64())
    .unwrap_or(1);

    let rows = db::select(
        "SELECT `c`.`id`, `ct`.`name` FROM `categories` `c` \
         LEFT JOIN `category_translations` `ct` ON `ct`.`category_id` = `c`.`id` AND `ct`.`locale` = ? \
         WHERE `c`.`parent_id` = ? AND `c`.`status` = 1 ORDER BY `c`.`position`, `c`.`id`",
        &[Value::String(locale.to_string()), Value::from(root_id)],
    )
    .await
    .unwrap_or_default();

    rows.into_iter()
        .filter_map(|row| {
            let name = row["name"].as_str()?.trim().to_string();
            let id = row["id"].as_i64()?;
            if name.is_empty() { None } else { Some((name, id)) }
        })
        .collect()
}

fn markdown_response(body: String) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/markdown; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        body,
    )
        .into_response()
}

impl AgenticController {
    /// `GET /api/llms.txt` — resumen del sitio para LLMs, generado desde el
    /// canal (nombre) y las categorías visibles (enlaces a páginas propias).
    pub async fn llms_txt(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let site = site_context(&query, &headers).await;
        let channel = core::current_channel(&headers).await
            .map(|channel| channel)
            .unwrap_or_default();
        let categories = channel_categories(channel["id"].as_i64().unwrap_or(1), &site.locale).await;

        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", site.name));
        md.push_str(&format!(
            "> {} es una tienda online con envío rápido y pago seguro. Esta guía resume el sitio y sus enlaces para agentes de IA y modelos de lenguaje.\n\n",
            site.name
        ));
        md.push_str(&format!(
            "{} vende productos online organizados en categorías. Los usuarios pueden navegar el catálogo, buscar productos, añadirlos al carrito y completar la compra como invitado o con cuenta de cliente.\n\n",
            site.name
        ));

        if !categories.is_empty() {
            md.push_str("## Categorías\n\n");
            for (name, id) in &categories {
                md.push_str(&format!("- [{}]({}/category/{})\n", name, site.base_url, id));
            }
            md.push('\n');
        }

        md.push_str("## Enlaces del sitio\n\n");
        md.push_str(&format!("- [Inicio]({}/)\n", site.base_url));
        md.push_str(&format!("- [Todos los productos]({}/products)\n", site.base_url));
        md.push_str(&format!("- [Buscar productos]({}/search)\n", site.base_url));
        md.push_str(&format!("- [Carrito]({}/cart)\n", site.base_url));
        md.push_str(&format!("- [Mi cuenta]({}/account)\n", site.base_url));
        md.push('\n');

        md.push_str("## API pública (JSON)\n\n");
        md.push_str(&format!("- [Listado de productos]({}/api/products)\n", site.base_url));
        md.push_str(&format!("- [Listado de categorías]({}/api/categories)\n", site.base_url));
        md.push_str(&format!("- [Documentación OpenAPI]({}/api/openapi.json)\n", site.base_url));

        markdown_response(md)
    }

    /// `GET /api/ai-catalog.json` — manifiesto ARD (Agentic Resource
    /// Discovery). Ruta legacy compatible con `.well-known/ai-catalog.json`;
    /// la estándar es `.well-known/ard.json` (mismo documento).
    pub async fn ai_catalog(Query(query): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
        let site = site_context(&query, &headers).await;
        let name = &site.name;
        let base = &site.base_url;
        let domain = &site.domain;

        let manifest = json!({
            "@context": "https://agenticresourcediscovery.org/context/v1",
            "entries": [
                {
                    "identifier": format!("urn:air:{domain}:web:storefront"),
                    "displayName": format!("{name} — tienda online"),
                    "type": "text/html",
                    "url": format!("{base}/"),
                    "description": format!(
                        "Tienda online de {name}: catálogo por categorías, búsqueda de productos, carrito y checkout como invitado o con cuenta de cliente."
                    ),
                    "representativeQueries": [
                        format!("¿Qué tipo de productos vende {name}?"),
                        "¿Cuánto cuesta el envío y en cuántos días llega?",
                        "¿Cómo puedo crear una cuenta de cliente?"
                    ],
                },
                {
                    "identifier": format!("urn:air:{domain}:api:products"),
                    "displayName": format!("{name} — catálogo de productos (API pública)"),
                    "type": "application/json",
                    "url": format!("{base}/api/products"),
                    "description": "Listado y búsqueda de productos en JSON: filtros, ordenación, paginación y ficha por url_key o SKU.",
                    "capabilities": ["search", "pagination", "filters", "locale", "currency"],
                    "representativeQueries": [
                        "Busca zapatillas running",
                        "Muéstrame los productos de electrónica disponibles",
                        "¿Qué productos están en oferta?",
                        "Dame los detalles del producto EZ-SNEAKER-01"
                    ],
                },
                {
                    "identifier": format!("urn:air:{domain}:api:categories"),
                    "displayName": format!("{name} — categorías de la tienda (API pública)"),
                    "type": "application/json",
                    "url": format!("{base}/api/categories"),
                    "description": "Categorías visibles del canal en JSON, con árbol jerárquico y atributos filtrables.",
                    "capabilities": ["tree", "attributes"],
                    "representativeQueries": [
                        "¿Qué categorías tiene la tienda?",
                        "Dame el árbol de categorías en JSON"
                    ],
                },
                {
                    "identifier": format!("urn:air:{domain}:docs:llms-txt"),
                    "displayName": format!("{name} — guía del sitio para LLMs (llms.txt)"),
                    // El tipo estándar de discovery para Markdown lleva el
                    // parámetro profile (sin él Lighthouse emite advertencia).
                    "type": "text/markdown; profile=\"urn:air:agent-skills\"",
                    "url": format!("{base}/llms.txt"),
                    "description": "Resumen en Markdown del sitio: qué vende la tienda, categorías y enlaces principales para navegación agéntica.",
                    "representativeQueries": [
                        "Resume qué es este sitio web",
                        "¿Qué enlaces principales tiene la tienda?"
                    ],
                },
                {
                    "identifier": format!("urn:air:{domain}:docs:openapi"),
                    "displayName": format!("{name} — especificación OpenAPI de la API"),
                    "type": "application/json",
                    "url": format!("{base}/api/openapi.json"),
                    "description": "Documentación OpenAPI 3 de la API pública de la tienda (productos, categorías, carrito y checkout).",
                    "representativeQueries": [
                        "¿Qué endpoints públicos tiene la API de la tienda?",
                        "¿Cómo se añade un producto al carrito vía API?"
                    ],
                },
            ],
        });

        (
            StatusCode::OK,
            [
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            Json(manifest),
        )
            .into_response()
    }
}
