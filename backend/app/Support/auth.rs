//! JWT bearer authentication with two principals:
//!
//! - `admins` (back office, roles/permissions ACL) - guard "admin".
//! - `customers` (shop) - guard "customer".
//!
//! Bagisto drives both guards with Laravel sessions; this port issues JWT
//! access tokens instead so the Flutter clients (web + mobile) can
//! authenticate with `Authorization: Bearer <token>`. Claims keep the
//! Laravel jwt-auth shape (`iss/iat/exp/nbf/jti/sub`).

use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use larust_support::axum::extract::FromRequestParts;
use larust_support::axum::http::request::Parts;
use larust_support::axum::http::StatusCode;
use larust_support::axum::response::{IntoResponse, Response};
use larust_support::axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use super::db;
use super::response::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub iat: i64,
    pub exp: i64,
    pub nbf: i64,
    pub jti: String,
    pub sub: String,
    /// Which principal the token belongs to: `admin` or `customer`.
    #[serde(default)]
    pub prv: String,
}

fn configuration() -> (String, i64, i64, bool) {
    let config = crate::config::app::config();
    let secret = config["jwt_secret"].as_str().unwrap_or("dev").to_string();
    let ttl = config["jwt_ttl_minutes"].as_i64().unwrap_or(60);
    let refresh_ttl = config["jwt_refresh_ttl_minutes"].as_i64().unwrap_or(20160);
    let blacklist = config["jwt_blacklist_enabled"].as_bool().unwrap_or(true);
    (secret, ttl, refresh_ttl, blacklist)
}

pub fn jwt_ttl_seconds() -> i64 {
    configuration().1 * 60
}

/// Issues a new access token for the principal (`admin`/`customer`) and id,
/// returning `(token, expires_at)`.
pub fn issue_token(principal: &str, user_id: i64) -> ApiResult<(String, String)> {
    let (secret, ttl, _refresh_ttl, _) = configuration();
    let now = Utc::now().timestamp();
    let exp = now + ttl * 60;
    let claims = Claims {
        iss: larust_support::config_env::env_or("APP_URL", "http://localhost"),
        iat: now,
        exp,
        nbf: now,
        jti: Uuid::new_v4().simple().to_string(),
        sub: user_id.to_string(),
        prv: principal.to_string(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).map_err(|error| super::response::ApiError::internal(error.to_string()))?;
    let expires_at = chrono::DateTime::from_timestamp(exp, 0)
        .unwrap_or_else(Utc::now)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    Ok((token, expires_at))
}

pub enum TokenError {
    Invalid,
    Expired,
}

pub fn decode_token(token: &str, allow_expired: bool) -> Result<Claims, TokenError> {
    let (secret, _ttl, refresh_ttl, _blacklist) = configuration();
    let mut validation = Validation::default();
    validation.validate_exp = !allow_expired;
    validation.set_issuer(&[larust_support::config_env::env_or("APP_URL", "http://localhost")]);

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|error| match error.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
        _ => TokenError::Invalid,
    })?;

    if allow_expired {
        let now = Utc::now().timestamp();
        // An access token may only be refreshed inside the refresh window.
        if data.claims.exp + refresh_ttl * 60 < now {
            return Err(TokenError::Expired);
        }
    }

    Ok(data.claims)
}

pub async fn is_blacklisted(jti: &str) -> bool {
    let (_, _, _, enabled) = configuration();
    if !enabled {
        return false;
    }
    match db::select_one(
        "SELECT jti FROM jwt_blacklist WHERE jti = ? LIMIT 1",
        &[Value::String(jti.to_string())],
    )
    .await
    {
        Ok(row) => row.is_some(),
        Err(_) => false,
    }
}

pub async fn blacklist(claims: &Claims) {
    let (_, _, _, enabled) = configuration();
    if !enabled {
        return;
    }
    let expires_at = chrono::DateTime::from_timestamp(claims.exp, 0)
        .unwrap_or_else(Utc::now)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let insert = match larust_support::orm::backend() {
        larust_support::orm::Backend::Sqlite => {
            "INSERT OR IGNORE INTO jwt_blacklist (jti, expires_at) VALUES (?, ?)"
        }
        _ => "INSERT IGNORE INTO jwt_blacklist (jti, expires_at) VALUES (?, ?)",
    };
    let _ = db::execute(
        insert,
        &[Value::String(claims.jti.clone()), Value::String(expires_at)],
    )
    .await;
}

pub fn bearer_token(headers: &larust_support::axum::http::HeaderMap) -> Option<String> {
    let value = headers.get("authorization")?.to_str().ok()?;
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))?;
    Some(token.trim().to_string())
}

/// Extracts and validates the token, returning its claims.
pub fn claims_from_parts(parts: &mut Parts, principal: &str) -> Result<Claims, Unauthenticated> {
    let token = bearer_token(&parts.headers).ok_or_else(Unauthenticated::default)?;
    let claims = decode_token(&token, false).map_err(|_| Unauthenticated::default())?;
    if claims.prv != principal {
        return Err(Unauthenticated::default());
    }
    Ok(claims)
}

/// The rejection body matches Bagisto's `AuthenticateCustomer`/admin guard:
/// `{"message": ""}` with 401 (admin guard answers `{"message": "Unauthenticated."}`).
pub struct Unauthenticated {
    message: &'static str,
}

impl Default for Unauthenticated {
    fn default() -> Self {
        Self { message: "" }
    }
}

impl Unauthenticated {
    pub fn admin() -> Self {
        Self {
            message: "Unauthenticated.",
        }
    }
}

impl IntoResponse for Unauthenticated {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": self.message })),
        )
            .into_response()
    }
}

/// Authenticated shop customer (guard `customer`).
#[derive(Debug, Clone)]
pub struct Customer {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub name: String,
    pub email: String,
    pub image: Option<String>,
    pub customer_group_id: i64,
    pub status: i64,
}

#[larust_support::axum::async_trait]
impl<S> FromRequestParts<S> for Customer
where
    S: Send + Sync,
{
    type Rejection = Unauthenticated;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = claims_from_parts(parts, "customer")?;
        let user_id = claims.sub.parse::<i64>().map_err(|_| Unauthenticated::default())?;
        let row = db::find_by_id("customers", user_id)
            .await
            .map_err(|_| Unauthenticated::default())?
            .ok_or(Unauthenticated::default())?;

        if row.get("status").and_then(Value::as_i64).unwrap_or(0) == 0 {
            return Err(Unauthenticated::default());
        }

        let first_name = row
            .get("first_name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let last_name = row
            .get("last_name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        Ok(Customer {
            id: user_id,
            name: format!("{} {}", first_name, last_name).trim().to_string(),
            first_name,
            last_name,
            email: row
                .get("email")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            image: row.get("image").and_then(Value::as_str).map(str::to_string),
            customer_group_id: row
                .get("customer_group_id")
                .and_then(Value::as_i64)
                .unwrap_or(1),
            status: row.get("status").and_then(Value::as_i64).unwrap_or(1),
        })
    }
}

/// Authenticated admin user (guard `admin`).
#[derive(Debug, Clone)]
pub struct Admin {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub image: Option<String>,
    pub role_id: i64,
}

#[larust_support::axum::async_trait]
impl<S> FromRequestParts<S> for Admin
where
    S: Send + Sync,
{
    type Rejection = Unauthenticated;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = claims_from_parts(parts, "admin")?;
        let user_id = claims.sub.parse::<i64>().map_err(|_| Unauthenticated::admin())?;
        let row = db::find_by_id("admins", user_id)
            .await
            .map_err(|_| Unauthenticated::admin())?
            .ok_or_else(Unauthenticated::admin)?;

        Ok(Admin {
            id: user_id,
            name: row
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            email: row
                .get("email")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            image: row.get("image").and_then(Value::as_str).map(str::to_string),
            role_id: row.get("role_id").and_then(Value::as_i64).unwrap_or(1),
        })
    }
}

/// Either principal (admin token takes precedence). Useful for endpoints the
/// shop exposes to both, like cart access for logged-in customers.
#[derive(Debug, Clone)]
pub enum Principal {
    Admin(Admin),
    Customer(Customer),
}

#[larust_support::axum::async_trait]
impl<S> FromRequestParts<S> for Principal
where
    S: Send + Sync,
{
    type Rejection = Unauthenticated;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = bearer_token(&parts.headers).ok_or(Unauthenticated::default())?;
        let claims = decode_token(&token, false).map_err(|_| Unauthenticated::default())?;
        if is_blacklisted(&claims.jti).await {
            return Err(Unauthenticated::default());
        }
        let user_id = claims
            .sub
            .parse::<i64>()
            .map_err(|_| Unauthenticated::default())?;

        match claims.prv.as_str() {
            "admin" => {
                let admin = db::find_by_id("admins", user_id)
                    .await
                    .map_err(|_| Unauthenticated::admin())?
                    .ok_or_else(Unauthenticated::admin)?;
                Ok(Principal::Admin(Admin {
                    id: user_id,
                    name: admin
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    email: admin
                        .get("email")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    image: admin
                        .get("image")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    role_id: admin
                        .get("role_id")
                        .and_then(Value::as_i64)
                        .unwrap_or(1),
                }))
            }
            _ => {
                let row = db::find_by_id("customers", user_id)
                    .await
                    .map_err(|_| Unauthenticated::default())?
                    .ok_or(Unauthenticated::default())?;
                if row.get("status").and_then(Value::as_i64).unwrap_or(0) == 0 {
                    return Err(Unauthenticated::default());
                }
                let first_name = row
                    .get("first_name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let last_name = row
                    .get("last_name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                Ok(Principal::Customer(Customer {
                    id: user_id,
                    name: format!("{} {}", first_name, last_name).trim().to_string(),
                    first_name,
                    last_name,
                    email: row
                        .get("email")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    image: row.get("image").and_then(Value::as_str).map(str::to_string),
                    customer_group_id: row
                        .get("customer_group_id")
                        .and_then(Value::as_i64)
                        .unwrap_or(1),
                    status: row.get("status").and_then(Value::as_i64).unwrap_or(1),
                }))
            }
        }
    }
}

pub fn hash_password(plain: &str) -> ApiResult<String> {
    bcrypt::hash(plain, bcrypt::DEFAULT_COST)
        .map_err(|error| super::response::ApiError::internal(error.to_string()))
}

/// Verifies a bcrypt hash. PHP bcrypt hashes (`$2y$`) are accepted by
/// rewriting them to `$2b$`.
pub fn verify_password(plain: &str, hash: &str) -> bool {
    let hash = if let Some(rest) = hash.strip_prefix("$2y$") {
        format!("$2b${rest}")
    } else {
        hash.to_string()
    };
    bcrypt::verify(plain, &hash).unwrap_or(false)
}
