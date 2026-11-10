//! Admin cart rules (marketing/promotions): datagrid, CRUD con pivotes de
//! canal/grupos y generación de cupones. Mirrors
//! `Webkul\Admin\Http\Controllers\Marketing\Promotions\CartRule*`.

use larust_support::axum::extract::Path;
use larust_support::axum::http::StatusCode;
use larust_support::axum::response::Response;
use larust_support::axum::Json;
use larust_support::serde_json::{json, Map, Value};

use crate::support::auth;
use crate::support::datagrid;
use crate::support::db;
use crate::support::response::{message, resource, validation_error};

pub struct AdminCartRuleController;

fn now() -> String {
    db::now_db()
}

/// Inserts the channel/customer-group pivots of a rule (default channel 1,
/// groups guest+general+wholesale si no se especifican).
async fn save_pivots(rule_id: i64, body: &Value) {
    let _ = db::execute(
        "DELETE FROM `cart_rule_channels` WHERE `cart_rule_id` = ?",
        &[Value::from(rule_id)],
    )
    .await;
    let channels = body["channels"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(Value::as_i64)
                .collect::<Vec<_>>()
        })
        .filter(|list| !list.is_empty())
        .unwrap_or_else(|| vec![1]);
    for channel_id in channels {
        let _ = db::execute(
            "INSERT OR IGNORE INTO `cart_rule_channels` (`cart_rule_id`, `channel_id`) VALUES (?, ?)",
            &[Value::from(rule_id), Value::from(channel_id)],
        )
        .await;
    }

    let _ = db::execute(
        "DELETE FROM `cart_rule_customer_groups` WHERE `cart_rule_id` = ?",
        &[Value::from(rule_id)],
    )
    .await;
    let groups = body["customer_groups"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(Value::as_i64)
                .collect::<Vec<_>>()
        })
        .filter(|list| !list.is_empty())
        .unwrap_or_else(|| vec![1, 2, 3]);
    for group_id in groups {
        let _ = db::execute(
            "INSERT OR IGNORE INTO `cart_rule_customer_groups` (`cart_rule_id`, `customer_group_id`) VALUES (?, ?)",
            &[Value::from(rule_id), Value::from(group_id)],
        )
        .await;
    }
}

/// Creates the primary coupon when `coupon_type = 2` (auto-generated codes
/// when `coupon_code` is omitted or `use_auto_generation` is set).
async fn save_primary_coupon(rule_id: i64, body: &Value) {
    if body["coupon_type"].as_i64().unwrap_or(1) != 2 {
        return;
    }
    let explicit_code = body["coupon_code"]
        .as_str()
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .map(str::to_string);
    let auto = body["use_auto_generation"].as_i64().unwrap_or(0) == 1;

    let _ = db::execute(
        "DELETE FROM `cart_rule_coupons` WHERE `cart_rule_id` = ? AND `is_primary` = 1",
        &[Value::from(rule_id)],
    )
    .await;

    match explicit_code {
        Some(code) if !auto => {
            let _ = db::insert(
                "cart_rule_coupons",
                &json!({
                    "code": code,
                    "usage_limit": body["uses_per_coupon"].clone(),
                    "usage_per_customer": body["usage_per_customer"].clone(),
                    "times_used": 0,
                    "type": 0,
                    "is_primary": 1,
                    "expired_at": body["ends_till"].as_str().map(|d| d.chars().take(10).collect::<String>()),
                    "cart_rule_id": rule_id,
                    "created_at": now(),
                    "updated_at": now(),
                })
                ,
            )
            .await;
        }
        _ => {
            // Auto: genera 10 códigos con prefijo del nombre de la regla.
            let prefix: String = rule_name_prefix(rule_id).await;
            for _ in 0..10 {
                let code = format!(
                    "{}{}",
                    prefix,
                    &uuid::Uuid::new_v4().simple().to_string()[..8].to_uppercase()
                );
                let _ = db::insert(
                    "cart_rule_coupons",
                    &json!({
                        "code": code,
                        "usage_limit": body["uses_per_coupon"].clone(),
                        "usage_per_customer": body["usage_per_customer"].clone(),
                        "times_used": 0,
                        "type": 1,
                        "is_primary": 0,
                        "expired_at": body["ends_till"].as_str().map(|d| d.chars().take(10).collect::<String>()),
                        "cart_rule_id": rule_id,
                        "created_at": now(),
                        "updated_at": now(),
                    })
                    ,
                )
                .await;
            }
        }
    }
}

async fn rule_name_prefix(rule_id: i64) -> String {
    let row = db::find_by_id("cart_rules", rule_id).await.ok().flatten();
    let name = row
        .as_ref()
        .and_then(|rule| rule["name"].as_str())
        .unwrap_or("CODE")
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(6)
        .collect::<String>();
    format!("{}-", if name.is_empty() { "CODE".into() } else { name })
}

fn cart_rules_grid() -> datagrid::DataGrid {
    datagrid::DataGrid {
        table: "cart_rules",
        alias: "r",
        primary: "r.id",
        columns: vec![
            datagrid::Column { index: "id", label: "ID", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("r.id"), options: None },
            datagrid::Column { index: "name", label: "Name", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("r.name"), options: None },
            datagrid::Column { index: "coupon_type", label: "Coupon", kind: "boolean", searchable: false, filterable: true, sortable: true, source: Some("r.coupon_type"), options: Some(json!([
                {"label": "No Coupon", "value": 1}, {"label": "By Coupon", "value": 2},
            ])) },
            datagrid::Column { index: "status", label: "Status", kind: "boolean", searchable: false, filterable: true, sortable: true, source: Some("r.status"), options: Some(json!([
                {"label": "Active", "value": 1}, {"label": "Inactive", "value": 0},
            ])) },
            datagrid::Column { index: "starts_from", label: "From", kind: "date", searchable: false, filterable: false, sortable: true, source: Some("r.starts_from"), options: None },
            datagrid::Column { index: "ends_till", label: "To", kind: "date", searchable: false, filterable: false, sortable: true, source: Some("r.ends_till"), options: None },
        ],
        joins: Vec::new(),
        where_clauses: Vec::new(),
        actions: json!([
            {"title": "Edit", "method": "GET", "url": "/admin/marketing/cart-rules/edit/{id}"},
            {"title": "Delete", "method": "DELETE", "url": "/api/admin/marketing/cart-rules/{id}"},
        ]),
        mass_actions: Value::Null,
        formatters: Vec::new(),
    }
}

impl AdminCartRuleController {
    /// `POST /api/admin/marketing/cart-rules` - datagrid. `?format=csv` exporta.
    pub async fn grid(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let request = datagrid::parse_request(&body);
        let grid = cart_rules_grid();
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/marketing/cart-rules/view/{id}` - regla + pivotes + cupones.
    pub async fn view(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let rule = match db::find_by_id("cart_rules", id).await {
            Ok(Some(rule)) => rule,
            _ => return message(StatusCode::NOT_FOUND, "Cart rule not found."),
        };
        let channels = db::select(
            "SELECT `channel_id` FROM `cart_rule_channels` WHERE `cart_rule_id` = ?",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        let groups = db::select(
            "SELECT `customer_group_id` FROM `cart_rule_customer_groups` WHERE `cart_rule_id` = ?",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        let coupons = db::select(
            "SELECT `id`, `code`, `usage_limit`, `usage_per_customer`, `times_used`, `expired_at`, `is_primary` \
             FROM `cart_rule_coupons` WHERE `cart_rule_id` = ? ORDER BY `id` DESC",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();

        resource(
            json!({
                "rule": rule,
                "channels": channels,
                "customer_groups": groups,
                "coupons": coupons,
            }),
            Value::Null,
        )
    }

    /// `POST /api/admin/marketing/cart-rules/store`.
    pub async fn store(admin: auth::Admin, Json(body): Json<Value>) -> Response {
        let _ = admin;
        let Some(name) = body["name"].as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            return validation_error(vec![("name".into(), "The name field is required.".into())]);
        };

        let rule_id = match db::insert(
            "cart_rules",
            &json!({
                "name": name,
                "description": body["description"].clone(),
                "starts_from": body["starts_from"].clone(),
                "ends_till": body["ends_till"].clone(),
                "status": body["status"].as_i64().unwrap_or(1),
                "coupon_type": body["coupon_type"].as_i64().unwrap_or(1),
                "use_auto_generation": body["use_auto_generation"].as_i64().unwrap_or(0),
                "usage_per_customer": body["usage_per_customer"].clone(),
                "uses_per_coupon": body["uses_per_coupon"].clone(),
                "times_used": 0,
                "condition_type": body["condition_type"].as_i64().unwrap_or(1),
                "conditions": body["conditions"].clone(),
                "end_other_rules": body["end_other_rules"].as_i64().unwrap_or(0),
                "action_type": body["action_type"].as_str().unwrap_or("by_percent"),
                "discount_amount": body["discount_amount"].as_f64().unwrap_or(0.0),
                "discount_quantity": body["discount_quantity"].as_i64().unwrap_or(1),
                "discount_step": body["discount_step"].as_i64().unwrap_or(1),
                "apply_to_shipping": body["apply_to_shipping"].as_i64().unwrap_or(0),
                "free_shipping": body["free_shipping"].as_i64().unwrap_or(0),
                "sort": body["sort"].as_i64().unwrap_or(0),
                "created_at": now(),
                "updated_at": now(),
            })
            ,
        )
        .await
        {
            Ok(id) => id,
            Err(error) => return error.into(),
        };

        save_pivots(rule_id, &body).await;
        save_primary_coupon(rule_id, &body).await;

        resource(json!({"id": rule_id}), "The cart rule has been created successfully.")
    }

    /// `PUT /api/admin/marketing/cart-rules/edit/{id}`.
    pub async fn update(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let exists = db::find_by_id("cart_rules", id).await.ok().flatten();
        if exists.is_none() {
            return message(StatusCode::NOT_FOUND, "Cart rule not found.");
        }
        let mut row = body.as_object().cloned().unwrap_or_default();
        row.remove("times_used");
        row.remove("created_at");
        row.remove("updated_at");
        if let Err(error) = db::update("cart_rules", id, &row).await {
            return error.into();
        }
        save_pivots(id, &body).await;
        // Sustituye el cupón primario si llega un código nuevo.
        if body.get("coupon_code").is_some() {
            save_primary_coupon(id, &body).await;
        }
        resource(json!({"id": id}), "The cart rule has been updated successfully.")
    }

    /// `DELETE /api/admin/marketing/cart-rules/{id}` (cascade limpia pivotes/cupones).
    pub async fn destroy(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match db::delete("cart_rules", id).await {
            Ok(_) => resource(Value::Array(Vec::new()), "The cart rule has been deleted successfully."),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/marketing/cart-rules/coupons/{id}` - genera cupones
    /// adicionales `{count, prefix?}`.
    pub async fn generate_coupons(
        admin: auth::Admin,
        Path(id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let count = body["count"].as_i64().unwrap_or(10).clamp(1, 100) as usize;
        let prefix = body["prefix"]
            .as_str()
            .map(|p| format!("{}-", p.to_uppercase()))
            .unwrap_or_else(|| String::new());
        let mut created = 0;
        for _ in 0..count {
            let code = format!(
                "{prefix}{}",
                &uuid::Uuid::new_v4().simple().to_string()[..8].to_uppercase()
            );
            if db::insert(
                "cart_rule_coupons",
                &json!({
                    "code": code,
                    "times_used": 0,
                    "type": 1,
                    "is_primary": 0,
                    "cart_rule_id": id,
                    "created_at": now(),
                    "updated_at": now(),
                })
                ,
            )
            .await
            .is_ok()
            {
                created += 1;
            }
        }
        resource(json!({"created": created}), format!("{created} coupons generated."))
    }

    /// `GET /api/admin/marketing/cart-rules/coupons/{id}` - cupones de la regla.
    pub async fn coupons(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let coupons = db::select(
            "SELECT * FROM `cart_rule_coupons` WHERE `cart_rule_id` = ? ORDER BY `id` DESC",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();
        resource(json!(coupons), Value::Null)
    }
}
