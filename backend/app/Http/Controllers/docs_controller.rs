//! Public API documentation: Swagger UI and the OpenAPI JSON document.

use larust_support::axum::response::{Html, IntoResponse, Response};

use crate::support::openapi;
use crate::support::response::json_ok;

pub struct DocsController;

impl DocsController {
    /// `GET {api_prefix}/docs` - public Swagger UI.
    pub async fn swagger_ui() -> Response {
        let prefix = crate::config::app::config()["api_prefix"]
            .as_str()
            .unwrap_or("/api")
            .trim_end_matches('/')
            .to_string();
        let spec_url = format!("{prefix}/openapi.json");
        let html = format!(
            r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>EzCommerce API · Swagger UI</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
  <style>
    html {{ box-sizing: border-box; overflow-y: scroll; }}
    *, *:before, *:after {{ box-sizing: inherit; }}
    body {{ margin: 0; background: #fafafa; }}
    .swagger-ui .topbar {{ display: none; }}
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js" crossorigin></script>
  <script>
    window.ui = SwaggerUIBundle({{
      url: '{spec_url}',
      dom_id: '#swagger-ui',
      deepLinking: true,
      displayRequestDuration: true,
      filter: true,
      persistAuthorization: true,
      tryItOutEnabled: true,
      tagsSorter: 'alpha',
      operationsSorter: 'alpha'
    }});
  </script>
</body>
</html>"#
        );
        Html(html).into_response()
    }

    /// `GET {api_prefix}/openapi.json` - public OpenAPI 3.0 document.
    pub async fn openapi() -> Response {
        json_ok(openapi::spec())
    }
}
