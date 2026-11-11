//! SEO: url rewrites, search terms y search synonyms (datagrids + CRUD).

use larust_support::axum::extract::Path;
use larust_support::axum::http::StatusCode;
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::datagrid;
use crate::support::db;
use crate::support::response::{message, resource, validation_error};

pub struct AdminSeoController;

impl AdminSeoController {
    // ------------------------------------------------------------------
    // URL rewrites
    // ------------------------------------------------------------------

    /// `POST /api/admin/seo/url-rewrites` - datagrid.
    pub async fn url_rewrites_grid(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let grid = datagrid::DataGrid {
            table: "url_rewrites",
            alias: "ur",
            primary: "ur.id",
            columns: vec![
                datagrid::Column { index: "id", label: "ID", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("ur.id"), options: None },
                datagrid::Column { index: "entity_type", label: "Entity", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("ur.entity_type"), options: Some(json!([
                    {"label": "Product", "value": "product"}, {"label": "Category", "value": "category"}, {"label": "CMS", "value": "cms"},
                ])) },
                datagrid::Column { index: "request_path", label: "Request Path", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("ur.request_path"), options: None },
                datagrid::Column { index: "target_path", label: "Target Path", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("ur.target_path"), options: None },
            ],
            joins: Vec::new(),
            where_clauses: Vec::new(),
            actions: json!([
                {"title": "Delete", "method": "DELETE", "url": "/api/admin/seo/url-rewrites/{id}"},
            ]),
            mass_actions: Value::Null,
            formatters: Vec::new(),
        };
        let request = datagrid::parse_request(&body);
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/seo/url-rewrites/store`.
    pub async fn store_url_rewrite(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let mut errors = Vec::new();
        for field in ["request_path", "target_path"] {
            if body[field].as_str().map(str::trim).unwrap_or_default().is_empty() {
                errors.push((field.to_string(), format!("The {field} field is required.")));
            }
        }
        if !errors.is_empty() {
            return validation_error(errors);
        }
        match db::insert("url_rewrites", &body).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The URL rewrite has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/admin/seo/url-rewrites/edit/{id}`.
    pub async fn update_url_rewrite(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        match db::update("url_rewrites", id, &body).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The URL rewrite has been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/seo/url-rewrites/{id}`.
    pub async fn destroy_url_rewrite(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match db::delete("url_rewrites", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The URL rewrite has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    // ------------------------------------------------------------------
    // Search terms
    // ------------------------------------------------------------------

    /// `POST /api/admin/seo/search-terms` - datagrid.
    pub async fn search_terms_grid(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let grid = datagrid::DataGrid {
            table: "search_terms",
            alias: "st",
            primary: "st.id",
            columns: vec![
                datagrid::Column { index: "id", label: "ID", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("st.id"), options: None },
                datagrid::Column { index: "term", label: "Search Term", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("st.term"), options: None },
                datagrid::Column { index: "results", label: "Results", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("st.results"), options: None },
                datagrid::Column { index: "uses", label: "Uses", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("st.uses"), options: None },
                datagrid::Column { index: "display_in_suggested_terms", label: "Suggested", kind: "boolean", searchable: false, filterable: true, sortable: true, source: Some("st.display_in_suggested_terms"), options: None },
            ],
            joins: Vec::new(),
            where_clauses: Vec::new(),
            actions: json!([
                {"title": "Delete", "method": "DELETE", "url": "/api/admin/seo/search-terms/{id}"},
            ]),
            mass_actions: Value::Null,
            formatters: Vec::new(),
        };
        let request = datagrid::parse_request(&body);
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/admin/seo/search-terms/edit/{id}` - `{redirect_url?, display_in_suggested_terms?}`.
    pub async fn update_search_term(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        match db::update("search_terms", id, &body).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The search term has been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/seo/search-terms/{id}`.
    pub async fn destroy_search_term(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match db::delete("search_terms", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The search term has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    // ------------------------------------------------------------------
    // Search synonyms
    // ------------------------------------------------------------------

    /// `POST /api/admin/seo/search-synonyms` - datagrid.
    pub async fn search_synonyms_grid(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let grid = datagrid::DataGrid {
            table: "search_synonyms",
            alias: "ss",
            primary: "ss.id",
            columns: vec![
                datagrid::Column { index: "id", label: "ID", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("ss.id"), options: None },
                datagrid::Column { index: "name", label: "Name", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("ss.name"), options: None },
                datagrid::Column { index: "terms", label: "Terms", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("ss.terms"), options: None },
            ],
            joins: Vec::new(),
            where_clauses: Vec::new(),
            actions: json!([
                {"title": "Delete", "method": "DELETE", "url": "/api/admin/seo/search-synonyms/{id}"},
            ]),
            mass_actions: Value::Null,
            formatters: Vec::new(),
        };
        let request = datagrid::parse_request(&body);
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/seo/search-synonyms/store` - `{name, terms: "a,b,c"}`.
    pub async fn store_search_synonym(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(name) = body["name"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("name".into(), "The name field is required.".into())]);
        };
        let Some(terms) = body["terms"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("terms".into(), "The terms field is required.".into())]);
        };
        match db::insert(
            "search_synonyms",
            &json!({"name": name, "terms": terms, "created_at": db::now_db(), "updated_at": db::now_db()}),
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The search synonyms have been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/admin/seo/search-synonyms/edit/{id}`.
    pub async fn update_search_synonym(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        match db::update("search_synonyms", id, &body).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The search synonyms have been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/seo/search-synonyms/{id}`.
    pub async fn destroy_search_synonym(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match db::delete("search_synonyms", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The search synonyms have been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/seo/sitemap.xml` - sitemap público con las rutas del shop.
    pub async fn sitemap() -> Response {
        use larust_support::axum::http::header;
        use larust_support::axum::response::IntoResponse;

        let base = larust_support::config_env::env_or("APP_URL", "http://localhost:34187");
        let mut entries = vec![
            format!("<url><loc>{}/</loc><changefreq>daily</changefreq><priority>1.0</priority></url>", base),
        ];
        // Categorías.
        if let Ok(categories) = db::select(
            "SELECT `c`.`id`, `ct`.`slug` FROM `categories` `c` \
             LEFT JOIN `category_translations` `ct` ON `ct`.`category_id` = `c`.`id` \
             WHERE `c`.`status` = 1 AND `c`.`parent_id` IS NOT NULL",
            &[],
        )
        .await
        {
            for category in categories {
                let slug = category["slug"].as_str().unwrap_or_default();
                if slug.is_empty() {
                    continue;
                }
                entries.push(format!(
                    "<url><loc>{}/category/{}</loc><changefreq>weekly</changefreq><priority>0.8</priority></url>",
                    base, slug
                ));
            }
        }
        // Productos.
        if let Ok(products) = db::select(
            "SELECT `url_key` FROM `product_flat` WHERE `status` = 1 AND `visible_individually` = 1 AND `parent_id` IS NULL",
            &[],
        )
        .await
        {
            for product in products {
                let url_key = product["url_key"].as_str().unwrap_or_default();
                if url_key.is_empty() {
                    continue;
                }
                entries.push(format!(
                    "<url><loc>{}/product/{}</loc><changefreq>weekly</changefreq><priority>0.7</priority></url>",
                    base, url_key
                ));
            }
        }

        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{}\n</urlset>",
            entries.join("\n")
        );
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
            xml,
        )
            .into_response()
    }
}
