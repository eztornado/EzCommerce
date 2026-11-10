//! Admin ACL + menu. Login/me/logout live in `AuthController` (guard
//! `admin`). Mirrors `Webkul\User\Bouncer` and the ACL config.

use larust_support::axum::response::Response;
use larust_support::serde_json::{json, Value};

use crate::support::auth;
use crate::support::db;
use crate::support::response::resource;

pub struct AdminAuthController;

/// ACL tree (phase 1 keys; mirrors `Webkul\Admin\Config\acl.php`).
const ACL: &[(&str, &str, &str)] = &[
    ("dashboard", "Dashboard", ""),
    ("sales", "Sales", ""),
    ("sales.orders", "Orders", ""),
    ("sales.invoices", "Invoices", ""),
    ("sales.shipments", "Shipments", ""),
    ("sales.refunds", "Refunds", ""),
    ("catalog", "Catalog", ""),
    ("catalog.products", "Products", ""),
    ("catalog.categories", "Categories", ""),
    ("catalog.attributes", "Attributes", ""),
    ("catalog.families", "Attribute Families", ""),
    ("customers", "Customers", ""),
    ("customers.customers", "Customers", ""),
    ("customers.groups", "Customer Groups", ""),
    ("customers.reviews", "Reviews", ""),
    ("settings", "Settings", ""),
    ("settings.channels", "Channels", ""),
    ("settings.locales", "Locales", ""),
    ("settings.currencies", "Currencies", ""),
    ("settings.users", "Users", ""),
    ("settings.roles", "Roles", ""),
    ("settings.taxes", "Taxes", ""),
    ("cms", "CMS", ""),
];

async fn role_permissions(admin: &auth::Admin) -> (String, Value) {
    let role = db::find_by_id("roles", admin.role_id).await.ok().flatten().unwrap_or_else(db::empty_row);
    (
        role["permission_type"].as_str().unwrap_or("all").to_string(),
        role["permissions"].clone(),
    )
}

pub async fn can(admin: &auth::Admin, node: &str) -> bool {
    let (permission_type, permissions) = role_permissions(admin).await;
    if permission_type == "all" {
        return true;
    }
    permissions
        .as_array()
        .map(|list| {
            list.iter().any(|entry| {
                let allowed = entry.as_str().unwrap_or_default();
                allowed == node
                    || node.starts_with(&format!("{allowed}."))
                    || allowed.starts_with(&format!("{node}."))
            })
        })
        .unwrap_or(false)
}

impl AdminAuthController {
    /// `GET /api/admin/acl` - the ACL tree with `allowed` per node.
    pub async fn acl(admin: auth::Admin) -> Response {
        let (permission_type, permissions) = role_permissions(&admin).await;
        let mut items = Vec::new();
        for (key, name, _route) in ACL {
            items.push(json!({
                "key": key,
                "name": name,
                "allowed": can(&admin, key).await,
            }));
        }
        resource(
            json!({
                "permission_type": permission_type,
                "permissions": permissions,
                "items": items,
            }),
            Value::Null,
        )
    }
}
