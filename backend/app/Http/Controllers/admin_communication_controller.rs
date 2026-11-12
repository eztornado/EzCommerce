//! Marketing communications: plantillas de email, eventos y campañas
//! (mirrors `Webkul\Admin\Http\Controllers\Marketing\Communications\*`).
//! El envío renderiza la plantilla por destinatario y la escribe al log del
//! servicio (MAIL_MAILER=log); campañas reales con SMTP quedan pendientes de
//! credenciales.

use std::collections::HashMap;

use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::StatusCode;
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::db;
use crate::support::response::{message, resource, validation_error};

pub struct AdminCommunicationController;

fn now() -> String {
    db::now_db()
}

fn render_template(content: &str, customer: &Value) -> String {
    let replace = |text: String, placeholder: &str, value: &Value| -> String {
        let value = match value {
            Value::Null => String::new(),
            other => other.to_string().trim_matches('"').to_string(),
        };
        text.replace(placeholder, &value)
    };
    let mut output = content.to_string();
    output = replace(output, "{{first_name}}", &customer["first_name"]);
    output = replace(output, "{{last_name}}", &customer["last_name"]);
    output = replace(output, "{{email}}", &customer["email"]);
    output = replace(output, "{{name}}", &customer["first_name"]);
    output
}

impl AdminCommunicationController {
    // ------------------------------------------------------------------
    // Plantillas
    // ------------------------------------------------------------------

    /// `GET /api/admin/marketing/communications/templates`.
    pub async fn templates(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT * FROM `marketing_templates` ORDER BY `id` DESC",
            &[],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `POST /api/admin/marketing/communications/templates/store` - `{name, status, content}`.
    pub async fn store_template(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(name) = body["name"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("name".into(), "The name field is required.".into())]);
        };
        match db::insert(
            "marketing_templates",
            &json!({
                "name": name,
                "status": body["status"].as_i64().unwrap_or(1),
                "content": body["content"].as_str().unwrap_or_default(),
                "created_at": now(),
                "updated_at": now(),
            }),
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The template has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/admin/marketing/communications/templates/edit/{id}`.
    pub async fn update_template(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let mut row = json!({"updated_at": now()});
        if let Some(name) = body["name"].as_str() {
            row["name"] = json!(name);
        }
        if let Some(status) = body["status"].as_i64() {
            row["status"] = json!(status);
        }
        if let Some(content) = body["content"].as_str() {
            row["content"] = json!(content);
        }
        match db::update("marketing_templates", id, &row).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The template has been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/marketing/communications/templates/{id}`.
    pub async fn destroy_template(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match db::delete("marketing_templates", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The template has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    // ------------------------------------------------------------------
    // Eventos
    // ------------------------------------------------------------------

    /// `GET /api/admin/marketing/communications/events`.
    pub async fn events(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT * FROM `marketing_events` ORDER BY `id` DESC",
            &[],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `POST /api/admin/marketing/communications/events/store` - `{name, description?, date?}`.
    pub async fn store_event(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(name) = body["name"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("name".into(), "The name field is required.".into())]);
        };
        match db::insert(
            "marketing_events",
            &json!({
                "name": name,
                "description": body["description"].clone(),
                "date": body["date"].clone(),
                "created_at": now(),
                "updated_at": now(),
            }),
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The event has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/marketing/communications/events/{id}`.
    pub async fn destroy_event(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match db::delete("marketing_events", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The event has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    // ------------------------------------------------------------------
    // Campañas
    // ------------------------------------------------------------------

    /// `GET /api/admin/marketing/communications/campaigns` - con plantilla y evento.
    pub async fn campaigns(admin: auth::Admin) -> Response {
        let _ = admin;
        let rows = db::select(
            "SELECT `mc`.*, `mt`.`name` AS template_name, `me`.`name` AS event_name, `cg`.`name` AS group_name \
             FROM `marketing_campaigns` `mc` \
             LEFT JOIN `marketing_templates` `mt` ON `mt`.`id` = `mc`.`marketing_template_id` \
             LEFT JOIN `marketing_events` `me` ON `me`.`id` = `mc`.`marketing_event_id` \
             LEFT JOIN `customer_groups` `cg` ON `cg`.`id` = `mc`.`customer_group_id` \
             ORDER BY `mc`.`id` DESC",
            &[],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `POST /api/admin/marketing/communications/campaigns/store` -
    /// `{name, subject, marketing_template_id, customer_group_id?, marketing_event_id?}`.
    pub async fn store_campaign(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let errors: Vec<(String, String)> = ["name", "subject", "marketing_template_id"]
            .iter()
            .filter(|field| body[**field].is_null())
            .map(|field| (field.to_string(), format!("The {field} field is required.")))
            .collect();
        if !errors.is_empty() {
            return validation_error(errors);
        }
        match db::insert(
            "marketing_campaigns",
            &json!({
                "name": body["name"].as_str().unwrap_or_default().trim(),
                "subject": body["subject"].as_str().unwrap_or_default(),
                "status": 0,
                "spooling": 0,
                "channel_id": 1,
                "customer_group_id": body["customer_group_id"].clone(),
                "marketing_template_id": body["marketing_template_id"],
                "marketing_event_id": body["marketing_event_id"].clone(),
                "created_at": now(),
                "updated_at": now(),
            }),
        )
        .await
        {
            Ok(_) => resource(Value::Array(Vec::new()), "The campaign has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `PUT /api/admin/marketing/communications/campaigns/edit/{id}`.
    pub async fn update_campaign(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let mut row = json!({"updated_at": now()});
        for key in ["name", "subject"] {
            if let Some(value) = body[key].as_str() {
                row[key] = json!(value);
            }
        }
        for key in ["customer_group_id", "marketing_template_id", "marketing_event_id"] {
            if !body[key].is_null() {
                row[key] = body[key].clone();
            }
        }
        match db::update("marketing_campaigns", id, &row).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The campaign has been updated successfully."),
            Err(error) => error.into(),
        }
    }

    /// `DELETE /api/admin/marketing/communications/campaigns/{id}`.
    pub async fn destroy_campaign(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match db::delete("marketing_campaigns", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The campaign has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/marketing/communications/campaigns/send/{id}` -
    /// renderiza la plantilla para los clientes del grupo suscritos al
    /// newsletter y la escribe al log (MAIL_MAILER=log). Marca la campaña
    /// enviada (`spooling = 1`).
    pub async fn send_campaign(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let campaign = db::find_by_id("marketing_campaigns", id).await.ok().flatten();
        let Some(campaign) = campaign else {
            return message(StatusCode::NOT_FOUND, "Campaign not found.");
        };
        let template_id = campaign["marketing_template_id"].as_i64().unwrap_or(0);
        let template = if template_id > 0 {
            db::find_by_id("marketing_templates", template_id).await.ok().flatten()
        } else {
            None
        };
        let Some(template) = template else {
            return message(StatusCode::UNPROCESSABLE_ENTITY, "The campaign has no valid template.");
        };
        let content = template["content"].as_str().unwrap_or_default().to_string();
        let subject = campaign["subject"].as_str().unwrap_or_default().to_string();

        let group_id = campaign["customer_group_id"].as_i64();
        let mut sql = "SELECT `id`, `email`, `first_name`, `last_name` FROM `customers` \
             WHERE `status` = 1 AND `subscribed_to_news_letter` = 1".to_string();
        let mut binds: Vec<Value> = Vec::new();
        if let Some(group_id) = group_id {
            sql.push_str(" AND `customer_group_id` = ?");
            binds.push(Value::from(group_id));
        }
        let recipients = db::select(&sql, &binds).await.unwrap_or_default();
        if recipients.is_empty() {
            return message(
                StatusCode::UNPROCESSABLE_ENTITY,
                "No subscribed customers match this campaign.",
            );
        }

        for customer in &recipients {
            let rendered = render_template(&content, &Value::Object(customer.clone()));
            larust_support::tracing::info!(
                "MAIL to={} subject={} body={}",
                customer["email"].as_str().unwrap_or_default(),
                subject,
                rendered
            );
        }

        let _ = db::update(
            "marketing_campaigns",
            id,
            &json!({"spooling": 1, "status": 1, "updated_at": now()}),
        )
        .await;
        resource(
            json!({"recipients": recipients.len()}),
            format!(
                "The campaign has been queued for {} recipients (logged mailer).",
                recipients.len()
            ),
        )
    }
}
