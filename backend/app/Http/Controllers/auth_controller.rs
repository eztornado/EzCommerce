//! Customer + admin authentication over JWT (`Authorization: Bearer`).
//!
//! Bagisto's shop API relies on Laravel cookie sessions; this port issues
//! JWTs instead so Flutter (web + mobile) authenticates statelessly. Status
//! codes and messages keep the original contract (`403` invalid credentials,
//! `401` unauthenticated, verification-aware registration).

use std::collections::HashMap;

use larust_support::axum::extract::Query;
use larust_support::axum::http::HeaderMap;
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::db;
use crate::support::response::{
    ApiError, ApiResult, message, resource, unauthorized, validation_error,
};

pub struct AuthController;

fn now() -> String {
    db::now_db()
}

impl AuthController {
    /// `POST /api/customer/login` - `{email, password}`.
    pub async fn login(Json(body): Json<Value>) -> Response {
        let errors = validate(&body, &["email", "password"]);
        if !errors.is_empty() {
            return validation_error(errors);
        }
        let email = body["email"].as_str().unwrap_or_default().trim().to_lowercase();
        let password = body["password"].as_str().unwrap_or_default();

        let customer = match db::select_one(
            "SELECT * FROM `customers` WHERE `email` = ? LIMIT 1",
            &[Value::String(email.clone())],
        )
        .await
        {
            Ok(row) => row,
            Err(_) => return message(
                larust_support::axum::http::StatusCode::FORBIDDEN,
                "Invalid email or password",
            ),
        };

        let Some(customer) = customer else {
            return message(
                larust_support::axum::http::StatusCode::FORBIDDEN,
                "Invalid email or password",
            );
        };

        let stored = customer["password"].as_str().unwrap_or_default().to_string();
        if !auth::verify_password(password, &stored) {
            return message(
                larust_support::axum::http::StatusCode::FORBIDDEN,
                "Invalid email or password",
            );
        }
        if customer["status"].as_i64().unwrap_or(0) == 0 {
            return message(
                larust_support::axum::http::StatusCode::FORBIDDEN,
                "Your account has been deactivated",
            );
        }

        let (token, expires_at) = match auth::issue_token("customer", customer["id"].as_i64().unwrap_or(0)) {
            Ok(pair) => pair,
            Err(error) => return error.into(),
        };

        resource(
            json!({
                "token": token,
                "token_type": "Bearer",
                "expires_at": expires_at,
                "message": "",
                "user": public_customer(customer.clone().into()),
            }),
            "",
        )
    }

    /// `POST /api/customer/register` - creates the account (verified unless
    /// verification is enabled in `core_config`) and returns a token.
    pub async fn register(Json(body): Json<Value>) -> Response {
        let mut errors = validate(&body, &["first_name", "last_name", "email", "password"]);
        if body["password"].as_str().unwrap_or_default().len() < 6 {
            errors.push(("password".into(), "The password must be at least 6 characters.".into()));
        }
        if body["password"].as_str().unwrap_or_default()
            != body["password_confirmation"].as_str().unwrap_or_default()
        {
            errors.push((
                "password".into(),
                "The password confirmation does not match.".into(),
            ));
        }
        if !errors.is_empty() {
            return validation_error(errors);
        }

        let email = body["email"].as_str().unwrap_or_default().trim().to_lowercase();
        let existing = db::select_one(
            "SELECT `id` FROM `customers` WHERE `email` = ? LIMIT 1",
            &[Value::String(email.clone())],
        )
        .await
        .ok()
        .flatten();
        if existing.is_some() {
            return validation_error(vec![(
                "email".into(),
                "The email has already been taken.".into(),
            )]);
        }

        let hash = match auth::hash_password(body["password"].as_str().unwrap_or_default()) {
            Ok(hash) => hash,
            Err(error) => return error.into(),
        };

        let general_group = db::select_one(
            "SELECT `id` FROM `customer_groups` WHERE `code` = 'general' LIMIT 1",
            &[],
        )
        .await
        .ok()
        .flatten()
        .and_then(|row| row["id"].as_i64())
        .unwrap_or(2);

        let new_id = match db::insert(
            "customers",
            &json!({
                "first_name": body["first_name"].clone(),
                "last_name": body["last_name"].clone(),
                "email": email,
                "password": hash,
                "status": 1,
                "is_verified": 1,
                "customer_group_id": general_group,
                "subscribed_to_news_letter": if body["subscribed_to_news_letter"].as_bool().unwrap_or(false) { 1 } else { 0 },
                "created_at": now(),
                "updated_at": now(),
            }),
        )
        .await
        {
            Ok(id) => id,
            Err(error) => return error.into(),
        };

        let customer = db::find_by_id("customers", new_id).await.ok().flatten();
        let (token, expires_at) = match auth::issue_token("customer", new_id) {
            Ok(pair) => pair,
            Err(error) => return error.into(),
        };

        resource(
            json!({
                "token": token,
                "token_type": "Bearer",
                "expires_at": expires_at,
                "user": customer.as_ref().map(|row| public_customer(row.clone().into())),
            }),
            "Your account has been created successfully.",
        )
    }

    /// `GET /api/customer/get` - the authenticated customer (guard `customer`).
    pub async fn get(customer: auth::Customer) -> Response {
        let row = db::find_by_id("customers", customer.id).await.ok().flatten();
        match row {
            Some(row) => resource(public_customer(row.into()), Value::Null),
            None => unauthorized(),
        }
    }

    /// `PUT /api/customer/profile` - update first/last name, gender, phone,
    /// dob, optional password change.
    pub async fn update_profile(customer: auth::Customer, Json(body): Json<Value>) -> Response {
        let mut data = json!({"updated_at": now()});
        for field in ["first_name", "last_name", "gender", "phone", "image"] {
            if let Some(value) = body.get(field) {
                data[field] = value.clone();
            }
        }
        if let Some(dob) = body.get("date_of_birth") {
            data["date_of_birth"] = dob.clone();
        }
        if let Some(password) = body["new_password"].as_str().filter(|p| !p.is_empty()) {
            let current = db::find_by_id("customers", customer.id)
                .await
                .ok()
                .flatten()
                .and_then(|row| row["password"].as_str().map(str::to_string))
                .unwrap_or_default();
            if !auth::verify_password(body["current_password"].as_str().unwrap_or_default(), &current)
            {
                return validation_error(vec![(
                    "current_password".into(),
                    "The current password is incorrect.".into(),
                )]);
            }
            match auth::hash_password(password) {
                Ok(hash) => data["password"] = json!(hash),
                Err(error) => return error.into(),
            }
        }

        if let Err(error) = db::update("customers", customer.id, &data).await {
            return error.into();
        }
        let row = db::find_by_id("customers", customer.id).await.ok().flatten();
        resource(
            row.map(|row| public_customer(row.into())).unwrap_or(Value::Null),
            "Your profile has been updated successfully.",
        )
    }

    /// `POST /api/customer/logout` - blacklists the current token.
    pub async fn logout(headers: HeaderMap) -> Response {
        if let Some(token) = auth::bearer_token(&headers) {
            if let Ok(claims) = auth::decode_token(&token, false) {
                auth::blacklist(&claims).await;
            }
        }
        resource(Value::Array(Vec::new()), "You have been logged out successfully.")
    }

    /// `GET /api/customer/refresh` - issues a new token from a valid-or-
    /// recently-expired one.
    pub async fn refresh(headers: HeaderMap) -> Response {
        let Some(token) = auth::bearer_token(&headers) else {
            return unauthorized();
        };
        let claims = match auth::decode_token(&token, true) {
            Ok(claims) if claims.prv == "customer" => claims,
            _ => return unauthorized(),
        };
        let user_id = claims.sub.parse::<i64>().unwrap_or(0);
        auth::blacklist(&claims).await;
        match auth::issue_token("customer", user_id) {
            Ok((token, expires_at)) => resource(
                json!({"token": token, "token_type": "Bearer", "expires_at": expires_at}),
                Value::Null,
            ),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/customer/forgot-password` - stores a reset token and mails it.
    pub async fn forgot_password(Json(body): Json<Value>) -> Response {
        let Some(email) = body["email"].as_str().map(|e| e.trim().to_lowercase()) else {
            return validation_error(vec![("email".into(), "The email field is required.".into())]);
        };
        let exists = db::select_one(
            "SELECT `id` FROM `customers` WHERE `email` = ? LIMIT 1",
            &[Value::String(email.clone())],
        )
        .await
        .ok()
        .flatten();
        if exists.is_none() {
            // Same response either way, like Laravel's `sendResetLink`.
            return resource(Value::Array(Vec::new()), "If your account exists, you will receive an email shortly.");
        }
        let token = uuid::Uuid::new_v4().simple().to_string();
        let _ = db::execute(
            "DELETE FROM `customer_password_resets` WHERE `email` = ?",
            &[Value::String(email.clone())],
        )
        .await;
        let _ = db::execute(
            "INSERT INTO `customer_password_resets` (`email`, `token`, `created_at`) VALUES (?, ?, ?)",
            &[
                Value::String(email.clone()),
                Value::String(token.clone()),
                Value::String(now()),
            ],
        )
        .await;
        crate::support::mailer::send(
            &email,
            "Reset Password",
            &format!("Your password reset link: /customer/reset-password/{token}"),
        )
        .await;
        resource(Value::Array(Vec::new()), "If your account exists, you will receive an email shortly.")
    }

    /// `POST /api/customer/reset-password` - `{token, password, password_confirmation}`.
    pub async fn reset_password(Json(body): Json<Value>) -> Response {
        let token = body["token"].as_str().unwrap_or_default().to_string();
        let password = body["password"].as_str().unwrap_or_default().to_string();
        if body["password"].as_str().unwrap_or_default()
            != body["password_confirmation"].as_str().unwrap_or_default()
            || password.len() < 6
        {
            return validation_error(vec![(
                "password".into(),
                "The password confirmation does not match or is too short.".into(),
            )]);
        }
        let row = db::select_one(
            "SELECT * FROM `customer_password_resets` WHERE `token` = ? LIMIT 1",
            &[Value::String(token.clone())],
        )
        .await
        .ok()
        .flatten();
        let Some(row) = row else {
            return message(
                larust_support::axum::http::StatusCode::BAD_REQUEST,
                "Invalid or expired token",
            );
        };
        let hash = match auth::hash_password(&password) {
            Ok(hash) => hash,
            Err(error) => return error.into(),
        };
        if let Some(email) = row["email"].as_str() {
            let _ = db::execute(
                "UPDATE `customers` SET `password` = ?, `updated_at` = ? WHERE `email` = ?",
                &[
                    Value::String(hash),
                    Value::String(now()),
                    Value::String(email.to_string()),
                ],
            )
            .await;
            let _ = db::execute(
                "DELETE FROM `customer_password_resets` WHERE `token` = ?",
                &[Value::String(token)],
            )
            .await;
        }
        resource(Value::Array(Vec::new()), "Your password has been reset successfully.")
    }

    // ------------------------------------------------------------------
    // Admin guard
    // ------------------------------------------------------------------

    /// `POST /api/admin/login` - `{email, password}` (guard `admin`).
    pub async fn admin_login(Json(body): Json<Value>) -> Response {
        let errors = validate(&body, &["email", "password"]);
        if !errors.is_empty() {
            return validation_error(errors);
        }
        let email = body["email"].as_str().unwrap_or_default().trim().to_lowercase();
        let admin = db::select_one(
            "SELECT * FROM `admins` WHERE `email` = ? LIMIT 1",
            &[Value::String(email)],
        )
        .await
        .ok()
        .flatten();

        let Some(admin) = admin else {
            return message(
                larust_support::axum::http::StatusCode::UNAUTHORIZED,
                "Please check your credentials and try again.",
            );
        };
        let stored = admin["password"].as_str().unwrap_or_default().to_string();
        if !auth::verify_password(body["password"].as_str().unwrap_or_default(), &stored) {
            return message(
                larust_support::axum::http::StatusCode::UNAUTHORIZED,
                "Please check your credentials and try again.",
            );
        }
        if admin["status"].as_i64().unwrap_or(0) == 0 {
            return message(
                larust_support::axum::http::StatusCode::FORBIDDEN,
                "Your account has been deactivated.",
            );
        }

        let (token, expires_at) = match auth::issue_token("admin", admin["id"].as_i64().unwrap_or(0)) {
            Ok(pair) => pair,
            Err(error) => return error.into(),
        };
        resource(
            json!({"token": token, "token_type": "Bearer", "expires_at": expires_at}),
            "Logged in successfully.",
        )
    }

    /// `GET /api/admin/me` - admin profile + role/permissions.
    pub async fn admin_me(admin: auth::Admin) -> Response {
        let row = db::find_by_id("admins", admin.id).await.ok().flatten();
        let role = db::find_by_id("roles", admin.role_id).await.ok().flatten();
        let mut payload = row.unwrap_or_else(db::empty_row);
        payload["password"] = Value::Null;
        payload["role"] = role.map(Value::Object).unwrap_or(Value::Null);
        resource(payload, Value::Null)
    }

    /// `POST /api/admin/logout`.
    pub async fn admin_logout(headers: HeaderMap) -> Response {
        if let Some(token) = auth::bearer_token(&headers) {
            if let Ok(claims) = auth::decode_token(&token, false) {
                auth::blacklist(&claims).await;
            }
        }
        resource(Value::Array(Vec::new()), "Logged out successfully.")
    }
}

/// Result-ish helper used by login validation.
fn validate(body: &Value, fields: &[&str]) -> Vec<(String, String)> {
    let mut errors = Vec::new();
    for field in fields {
        if body[field].as_str().map(str::trim).unwrap_or_default().is_empty() {
            errors.push((field.to_string(), format!("The {field} field is required.")));
        }
    }
    errors
}

fn public_customer(customer: Value) -> Value {
    let mut payload = customer.clone();
    payload["password"] = Value::Null;
    payload["remember_token"] = Value::Null;
    payload["api_token"] = Value::Null;
    payload["token"] = Value::Null;
    payload["name"] = json!(format!(
        "{} {}",
        customer["first_name"].as_str().unwrap_or_default(),
        customer["last_name"].as_str().unwrap_or_default()
    )
    .trim());
    payload
}
