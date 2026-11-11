//! Idempotent seeders mirroring Bagisto's installer order:
//! attributes/family/groups -> customer groups -> root category ->
//! inventory source -> locales/currencies/channel/config -> countries ->
//! admin -> demo catalog (`SEED_DEMO_DATA=true`).

use larust_core::AppError;
use larust_support::orm::{backend, pool, Backend};
use larust_support::serde_json::{json, Value};

use crate::config::app::config;

fn internal(error: impl std::error::Error + Send + Sync + 'static) -> AppError {
    AppError::Internal(Box::new(error))
}

fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Insert-if-missing by `code`/unique key, keeping seeders re-runnable.
async fn insert_ignore(table: &str, columns: &[&str], row: &[Value]) -> Result<(), AppError> {
    let pool = pool()?;
    let placeholders: Vec<String> = vec!["?"; columns.len()].iter().map(|s| s.to_string()).collect();
    let sql = match backend() {
        Backend::Sqlite => format!(
            "INSERT OR IGNORE INTO `{}` ({}) VALUES ({})",
            table,
            columns
                .iter()
                .map(|c| format!("`{c}`"))
                .collect::<Vec<_>>()
                .join(", "),
            placeholders.join(", ")
        ),
        _ => format!(
            "INSERT IGNORE INTO `{}` ({}) VALUES ({})",
            table,
            columns
                .iter()
                .map(|c| format!("`{c}`"))
                .collect::<Vec<_>>()
                .join(", "),
            placeholders.join(", ")
        ),
    };
    let mut query = sqlx::query(&sql);
    for value in row {
        query = bind(value, query);
    }
    query.execute(pool).await.map_err(internal)?;
    Ok(())
}

fn bind<'q>(
    value: &'q Value,
    query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>,
) -> sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>> {
    match value {
        Value::Null => query.bind(None::<String>),
        Value::Bool(b) => query.bind(*b as i64),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                query.bind(i)
            } else {
                query.bind(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => query.bind(s.as_str()),
        other => query.bind(other.to_string()),
    }
}

/// SELECT id by unique code, creating nothing.
async fn id_of(table: &str, code: &str) -> Option<i64> {
    let Ok(pool) = pool() else { return None };
    let column = match table {
        "currencies" | "locales" => "code",
        "attribute_families" | "attributes" | "tax_categories" | "inventory_sources" => "code",
        "customer_groups" => "code",
        "channels" => "code",
        _ => "code",
    };
    let sql = format!("SELECT `id` FROM `{table}` WHERE `{column}` = ? LIMIT 1");
    sqlx::query_scalar(&sql)
        .bind(code)
        .fetch_one(pool)
        .await
        .ok()
}

pub async fn run() -> Result<(), AppError> {
    let config = config();
    seed_locales_and_currencies().await?;
    seed_customer_groups().await?;
    seed_attribute_family().await?;
    seed_system_attributes().await?;
    seed_root_category().await?;
    seed_inventory_source().await?;
    seed_channel().await?;
    seed_countries().await?;
    seed_admin(&config).await?;
    if config["seed_demo_data"].as_bool().unwrap_or(true) {
        seed_demo_catalog().await?;
        seed_demo_home().await?;
        seed_demo_cart_rules().await?;
        seed_demo_catalog_rule().await?;
    }
    Ok(())
}

async fn seed_locales_and_currencies() -> Result<(), AppError> {
    let stamp = now();
    for (code, name, direction) in [
        ("en", "English", "ltr"),
        ("es", "Spanish", "ltr"),
        ("fr", "French", "ltr"),
        ("de", "German", "ltr"),
        ("ar", "Arabic", "rtl"),
    ] {
        insert_ignore(
            "locales",
            &["code", "name", "direction", "created_at", "updated_at"],
            &[
                json!(code),
                json!(name),
                json!(direction),
                json!(stamp),
                json!(stamp),
            ],
        )
        .await?;
    }
    for (code, name, symbol) in [
        ("USD", "US Dollar", "$"),
        ("EUR", "Euro", "€"),
        ("GBP", "Pound Sterling", "£"),
    ] {
        insert_ignore(
            "currencies",
            &["code", "name", "symbol", "decimal", "created_at", "updated_at"],
            &[
                json!(code),
                json!(name),
                json!(symbol),
                json!(2),
                json!(stamp),
                json!(stamp),
            ],
        )
        .await?;
    }
    // Exchange rates relative to EUR (base of the default channel).
    let eur = id_of("currencies", "EUR").await.unwrap_or(2);
    for (code, rate) in [("USD", 1.08), ("GBP", 0.85)] {
        insert_ignore(
            "currency_exchange_rates",
            &["rate", "target_currency", "created_at", "updated_at"],
            &[
                json!(rate),
                json!(id_of("currencies", code).await.unwrap_or(1)),
                json!(stamp),
                json!(stamp),
            ],
        )
        .await?;
    }
    let _ = eur;
    Ok(())
}

async fn seed_customer_groups() -> Result<(), AppError> {
    let stamp = now();
    for (code, name) in [
        ("guest", "Guest"),
        ("general", "General"),
        ("wholesale", "Wholesale"),
    ] {
        insert_ignore(
            "customer_groups",
            &["code", "name", "is_user_defined", "created_at", "updated_at"],
            &[json!(code), json!(name), json!(0), json!(stamp), json!(stamp)],
        )
        .await?;
    }
    Ok(())
}

async fn seed_attribute_family() -> Result<(), AppError> {
    let stamp = now();
    insert_ignore(
        "attribute_families",
        &["code", "name", "status", "is_user_defined", "created_at", "updated_at"],
        &[
            json!("default"),
            json!("Default"),
            json!(1),
            json!(0),
            json!(stamp),
            json!(stamp),
        ],
    )
    .await?;
    let family_id = id_of("attribute_families", "default").await.unwrap_or(1);
    for (position, group) in [
        "General",
        "Description",
        "Meta Description",
        "Price",
        "Shipping",
        "Settings",
        "Inventories",
    ]
    .iter()
    .enumerate()
    {
        insert_ignore(
            "attribute_groups",
            &[
                "attribute_family_id",
                "name",
                "position",
                "is_user_defined",
                "created_at",
                "updated_at",
            ],
            &[
                json!(family_id),
                json!(group),
                json!(position),
                json!(0),
                json!(stamp),
                json!(stamp),
            ],
        )
        .await?;
    }
    Ok(())
}

/// The 29 system attributes of Bagisto (`AttributeTableSeeder`).
async fn seed_system_attributes() -> Result<(), AppError> {
    let stamp = now();
    // (code, admin_name, type, position, is_filterable, is_configurable, visible_on_front, value_per_locale)
    let attributes: &[(&str, &str, &str, i64, bool, bool, bool, bool)] = &[
        ("sku", "SKU", "text", 1, false, false, false, false),
        ("name", "Name", "text", 2, false, false, true, false),
        ("url_key", "URL Key", "text", 3, false, false, false, false),
        ("tax_category_id", "Tax Category", "select", 4, false, false, false, false),
        ("new", "New", "boolean", 5, true, false, false, false),
        ("featured", "Featured", "boolean", 6, true, false, false, false),
        ("visible_individually", "Visible Individually", "boolean", 7, false, false, false, false),
        ("status", "Status", "boolean", 8, false, false, false, false),
        ("short_description", "Short Description", "textarea", 9, false, false, true, false),
        ("description", "Description", "textarea", 10, false, false, true, false),
        ("price", "Price", "price", 11, true, false, true, false),
        ("cost", "Cost", "price", 12, false, false, false, false),
        ("special_price", "Special Price", "price", 13, true, false, true, false),
        ("special_price_from", "Special Price From", "date", 14, false, false, false, false),
        ("special_price_to", "Special Price To", "date", 15, false, false, false, false),
        ("meta_title", "Meta Title", "text", 16, false, false, false, false),
        ("meta_keywords", "Meta Keywords", "textarea", 17, false, false, false, false),
        ("meta_description", "Meta Description", "textarea", 18, false, false, false, false),
        ("length", "Length", "text", 19, false, false, true, false),
        ("width", "Width", "text", 20, false, false, true, false),
        ("height", "Height", "text", 21, false, false, true, false),
        ("weight", "Weight", "text", 22, false, false, true, false),
        ("color", "Color", "select", 23, true, true, true, false),
        ("size", "Size", "select", 24, true, true, true, false),
        ("brand", "Brand", "select", 25, true, false, true, false),
        ("manage_stock", "Manage Stock", "boolean", 26, false, false, false, false),
        ("product_number", "Product Number", "text", 27, false, false, false, false),
        ("guest_checkout", "Guest Checkout", "boolean", 28, false, false, false, false),
        ("allow_rma", "Allow RMA", "boolean", 29, false, false, false, false),
    ];

    for (code, name, kind, position, filterable, configurable, visible, _per_locale) in attributes {
        insert_ignore(
            "attributes",
            &[
                "code",
                "admin_name",
                "type",
                "position",
                "is_required",
                "is_unique",
                "is_filterable",
                "is_comparable",
                "is_configurable",
                "is_user_defined",
                "is_visible_on_front",
                "value_per_locale",
                "value_per_channel",
                "enable_wysiwyg",
                "created_at",
                "updated_at",
            ],
            &[
                json!(code),
                json!(name),
                json!(kind),
                json!(position),
                json!(if *code == "sku" { 1 } else { 0 }),
                json!(if *code == "sku" { 1 } else { 0 }),
                json!(*filterable as i64),
                json!(0),
                json!(*configurable as i64),
                json!(0),
                json!(*visible as i64),
                json!(0),
                json!(0),
                json!(0),
                json!(stamp),
                json!(stamp),
            ],
        )
        .await?;
    }

    // Swatch options for color/size/brand.
    let options: &[(&str, &str, &str)] = &[
        ("color", "Red", "#DC2626"),
        ("color", "Green", "#059669"),
        ("color", "Blue", "#1D4ED8"),
        ("color", "Black", "#111827"),
        ("color", "White", "#FFFFFF"),
        ("size", "S", ""),
        ("size", "M", ""),
        ("size", "L", ""),
        ("size", "XL", ""),
        ("brand", "EzCommerce", ""),
        ("brand", "Acme", ""),
    ];
    for (attribute_code, name, swatch) in options {
        let attribute_id = id_of("attributes", attribute_code).await.unwrap_or(0);
        if attribute_id == 0 {
            continue;
        }
        let pool = pool()?;
        let existing: Option<(i64,)> = sqlx::query_as(
            "SELECT `id` FROM `attribute_options` WHERE `attribute_id` = ? AND `admin_name` = ?",
        )
        .bind(attribute_id)
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(internal)?;
        if existing.is_none() {
            let _ = sqlx::query(
                "INSERT INTO `attribute_options` (`attribute_id`, `admin_name`, `sort_order`, `swatch_value`) VALUES (?, ?, 0, ?)",
            )
            .bind(attribute_id)
            .bind(name)
            .bind(swatch)
            .execute(pool)
            .await;
        }
    }
    Ok(())
}

async fn seed_root_category() -> Result<(), AppError> {
    let stamp = now();
    insert_ignore(
        "categories",
        &[
            "id",
            "position",
            "status",
            "_lft",
            "_rgt",
            "parent_id",
            "display_mode",
            "created_at",
            "updated_at",
        ],
        &[
            json!(1),
            json!(1),
            json!(1),
            json!(1),
            json!(2),
            Value::Null,
            json!("products_and_description"),
            json!(stamp),
            json!(stamp),
        ],
    )
    .await?;
    insert_ignore(
        "category_translations",
        &["category_id", "locale", "name", "slug", "url_path", "created_at", "updated_at"],
        &[
            json!(1),
            json!("en"),
            json!("Root"),
            json!("root"),
            json!(""),
            json!(stamp),
            json!(stamp),
        ],
    )
    .await?;
    Ok(())
}

async fn seed_inventory_source() -> Result<(), AppError> {
    let stamp = now();
    insert_ignore(
        "inventory_sources",
        &[
            "id",
            "code",
            "name",
            "priority",
            "status",
            "created_at",
            "updated_at",
        ],
        &[
            json!(1),
            json!("default"),
            json!("Default"),
            json!(0),
            json!(1),
            json!(stamp),
            json!(stamp),
        ],
    )
    .await?;
    Ok(())
}

async fn seed_channel() -> Result<(), AppError> {
    let stamp = now();
    insert_ignore(
        "channels",
        &[
            "id",
            "code",
            "timezone",
            "theme",
            "hostname",
            "root_category_id",
            "default_locale_id",
            "base_currency_id",
            "created_at",
            "updated_at",
        ],
        &[
            json!(1),
            json!("default"),
            json!("UTC"),
            json!("default"),
            Value::Null,
            json!(1),
            json!(id_of("locales", "en").await.unwrap_or(1)),
            json!(id_of("currencies", "EUR").await.unwrap_or(2)),
            json!(stamp),
            json!(stamp),
        ],
    )
    .await?;
    insert_ignore(
        "channel_translations",
        &["channel_id", "locale", "name", "description", "created_at", "updated_at"],
        &[
            json!(1),
            json!("en"),
            json!("EzCommerce"),
            json!("EzCommerce store"),
            json!(stamp),
            json!(stamp),
        ],
    )
    .await?;
    for code in ["en", "es"] {
        if let Some(locale_id) = id_of("locales", code).await {
            insert_ignore(
                "channel_locales",
                &["channel_id", "locale_id"],
                &[json!(1), json!(locale_id)],
            )
            .await?;
        }
    }
    for code in ["USD", "EUR", "GBP"] {
        if let Some(currency_id) = id_of("currencies", code).await {
            insert_ignore(
                "channel_currencies",
                &["channel_id", "currency_id"],
                &[json!(1), json!(currency_id)],
            )
            .await?;
        }
    }
    insert_ignore(
        "channel_inventory_sources",
        &["channel_id", "inventory_source_id", "created_at", "updated_at"],
        &[json!(1), json!(1), json!(stamp), json!(stamp)],
    )
    .await?;
    Ok(())
}

/// ISO-3166 alpha-2 selection (the full 250-country list lands with the
/// complete countries seeder; checkout needs the common ones first).
async fn seed_countries() -> Result<(), AppError> {
    let countries: &[(&str, &str)] = &[
        ("AD", "Andorra"), ("AR", "Argentina"), ("AT", "Austria"), ("AU", "Australia"),
        ("BE", "Belgium"), ("BO", "Bolivia"), ("BR", "Brazil"), ("CA", "Canada"),
        ("CH", "Switzerland"), ("CL", "Chile"), ("CN", "China"), ("CO", "Colombia"),
        ("CR", "Costa Rica"), ("CU", "Cuba"), ("CZ", "Czechia"), ("DE", "Germany"),
        ("DK", "Denmark"), ("DO", "Dominican Republic"), ("EC", "Ecuador"),
        ("EE", "Estonia"), ("EG", "Egypt"), ("ES", "Spain"), ("FI", "Finland"),
        ("FR", "France"), ("GB", "United Kingdom"), ("GR", "Greece"), ("GT", "Guatemala"),
        ("HN", "Honduras"), ("HR", "Croatia"), ("HU", "Hungary"), ("ID", "Indonesia"),
        ("IE", "Ireland"), ("IL", "Israel"), ("IN", "India"), ("IS", "Iceland"),
        ("IT", "Italy"), ("JP", "Japan"), ("KR", "South Korea"), ("LT", "Lithuania"),
        ("LU", "Luxembourg"), ("LV", "Latvia"), ("MA", "Morocco"), ("MX", "Mexico"),
        ("NI", "Nicaragua"), ("NL", "Netherlands"), ("NO", "Norway"), ("NZ", "New Zealand"),
        ("PA", "Panama"), ("PE", "Peru"), ("PL", "Poland"), ("PT", "Portugal"),
        ("PY", "Paraguay"), ("RO", "Romania"), ("SE", "Sweden"), ("SG", "Singapore"),
        ("SI", "Slovenia"), ("SK", "Slovakia"), ("SV", "El Salvador"), ("TR", "Turkey"),
        ("UA", "Ukraine"), ("US", "United States"), ("UY", "Uruguay"), ("VE", "Venezuela"),
        ("ZA", "South Africa"),
    ];
    for (code, name) in countries {
        insert_ignore("countries", &["code", "name"], &[json!(code), json!(name)]).await?;
    }
    // Spanish states (provinces abbreviated set) for checkout testing.
    let states: &[(&str, &str)] = &[
        ("AN", "Andalusia"), ("AR", "Aragon"), ("AS", "Asturias"), ("CB", "Cantabria"),
        ("CM", "Castile-La Mancha"), ("CL", "Castile and Leon"), ("CT", "Catalonia"),
        ("EX", "Extremadura"), ("GA", "Galicia"), ("IB", "Balearic Islands"),
        ("MC", "Murcia"), ("MD", "Madrid"), ("NC", "Navarre"), ("PV", "Basque Country"),
        ("RI", "La Rioja"), ("VC", "Valencian Community"),
    ];
    for (code, name) in states {
        insert_ignore(
            "country_states",
            &["country_code", "code", "default_name", "country_id"],
            &[json!("ES"), json!(code), json!(name), json!(22)],
        )
        .await?;
    }
    Ok(())
}

async fn seed_admin(config: &Value) -> Result<(), AppError> {
    let stamp = now();
    let email = config["admin_email"].as_str().unwrap_or("admin@example.com");
    let password = config["admin_password"].as_str().unwrap_or("admin123");
    let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(internal)?;

    insert_ignore(
        "roles",
        &["name", "description", "permission_type", "created_at", "updated_at"],
        &[
            json!("Administrator"),
            json!("Admin role"),
            json!("all"),
            json!(stamp),
            json!(stamp),
        ],
    )
    .await?;

    let pool = pool()?;
    let role_id: i64 =
        sqlx::query_scalar("SELECT `id` FROM `roles` WHERE `permission_type` = 'all' LIMIT 1")
            .fetch_one(pool)
            .await
            .map_err(internal)?;

    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT `id` FROM `admins` WHERE `email` = ?")
            .bind(email)
            .fetch_optional(pool)
            .await
            .map_err(internal)?;
    if existing.is_none() {
        sqlx::query(
            "INSERT INTO `admins` (`name`, `email`, `password`, `status`, `role_id`, `created_at`, `updated_at`) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("Admin")
        .bind(email)
        .bind(&hash)
        .bind(1)
        .bind(role_id)
        .bind(&stamp)
        .bind(&stamp)
        .execute(pool)
        .await
        .map_err(internal)?;
    }

    // Default core_config for carriers + payment methods.
    let defaults: &[(&str, &str)] = &[
        ("sales.carriers.flatrate.active", "1"),
        ("sales.carriers.flatrate.default_rate", "10"),
        ("sales.carriers.flatrate.type", "per_unit"),
        ("sales.carriers.free.active", "1"),
        ("sales.payment_methods.cashondelivery.active", "1"),
        ("sales.payment_methods.moneytransfer.active", "1"),
        ("sales.taxes.calculation.based_on", "billing_address"),
        ("catalog.products.home_page.out_of_stock_items", "0"),
    ];
    for (code, value) in defaults {
        let existing: Option<(i64,)> =
            sqlx::query_as("SELECT `id` FROM `core_config` WHERE `code` = ? AND `channel_code` IS NULL AND `locale_code` IS NULL")
                .bind(code)
                .fetch_optional(pool)
                .await
                .map_err(internal)?;
        if existing.is_none() {
            sqlx::query("INSERT INTO `core_config` (`code`, `value`, `created_at`, `updated_at`) VALUES (?, ?, ?, ?)")
                .bind(code)
                .bind(value)
                .bind(&stamp)
                .bind(&stamp)
                .execute(pool)
                .await
                .map_err(internal)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Demo catalog (`SEED_DEMO_DATA=true`)
// ---------------------------------------------------------------------------

async fn attr_id(code: &str) -> i64 {
    id_of("attributes", code).await.unwrap_or(0)
}

async fn set_attr(product_id: i64, code: &str, value: Value) {
    let attribute_id = attr_id(code).await;
    if attribute_id == 0 {
        return;
    }
    let Ok(pool) = pool() else { return };
    let attribute_type = sqlx::query_scalar::<_, String>(
        "SELECT `type` FROM `attributes` WHERE `id` = ?",
    )
    .bind(attribute_id)
    .fetch_one(pool)
    .await
    .unwrap_or_else(|_| "text".to_string());

    let mut row = json!({"product_id": product_id, "attribute_id": attribute_id});
    match attribute_type.as_str() {
        "boolean" => row["boolean_value"] = json!(value.as_i64().unwrap_or(0)),
        "integer" | "select" => row["integer_value"] = value,
        "price" => row["float_value"] = value,
        "date" => row["date_value"] = value,
        "datetime" => row["datetime_value"] = value,
        "multiselect" | "checkbox" => row["json_value"] = value,
        _ => row["text_value"] = value,
    }

    let existing: Option<(i64,)> = sqlx::query_as(
        "SELECT `id` FROM `product_attribute_values` WHERE `product_id` = ? AND `attribute_id` = ?",
    )
    .bind(product_id)
    .bind(attribute_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    match existing {
        Some((id,)) => {
            let _ = sqlx::query("UPDATE `product_attribute_values` SET `text_value` = ?, `boolean_value` = ?, `integer_value` = ?, `float_value` = ?, `date_value` = ?, `datetime_value` = ?, `json_value` = ? WHERE `id` = ?")
                .bind(row["text_value"].as_str())
                .bind(row["boolean_value"].as_i64())
                .bind(row["integer_value"].as_i64())
                .bind(row["float_value"].as_f64())
                .bind(row["date_value"].as_str())
                .bind(row["datetime_value"].as_str())
                .bind(row["json_value"].as_str().map(|s| s.to_string()))
                .bind(id)
                .execute(pool)
                .await;
        }
        None => {
            let _ = sqlx::query("INSERT INTO `product_attribute_values` (`product_id`, `attribute_id`, `text_value`, `boolean_value`, `integer_value`, `float_value`, `date_value`, `datetime_value`, `json_value`) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(row["product_id"].as_i64())
                .bind(row["attribute_id"].as_i64())
                .bind(row["text_value"].as_str())
                .bind(row["boolean_value"].as_i64())
                .bind(row["integer_value"].as_i64())
                .bind(row["float_value"].as_f64())
                .bind(row["date_value"].as_str())
                .bind(row["datetime_value"].as_str())
                .bind(row["json_value"].as_str().map(|s| s.to_string()))
                .execute(pool)
                .await;
        }
    }
}

async fn seed_demo_catalog() -> Result<(), AppError> {
    let pool = pool()?;
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM `products`")
        .fetch_one(pool)
        .await
        .map_err(internal)?;
    if existing > 0 {
        return Ok(());
    }
    larust_support::tracing::info!("SEED_DEMO_DATA=true; creando catálogo demo");

    let stamp = now();
    // Categories under root.
    let demo_categories: &[(&str, &str)] = &[
        ("electronics", "Electronics"),
        ("men", "Men"),
        ("women", "Women"),
        ("home", "Home & Living"),
    ];
    let mut lft = 2;
    for (slug, name) in demo_categories {
        sqlx::query(
            "INSERT INTO `categories` (`position`, `status`, `_lft`, `_rgt`, `parent_id`, `display_mode`, `created_at`, `updated_at`) VALUES (?, 1, ?, ?, 1, 'products_and_description', ?, ?)",
        )
        .bind(lft - 1)
        .bind(lft)
        .bind(lft + 1)
        .bind(&stamp)
        .bind(&stamp)
        .execute(pool)
        .await
        .map_err(internal)?;
        lft += 2;
        let category_id: i64 = sqlx::query_scalar("SELECT MAX(`id`) FROM `categories`")
            .fetch_one(pool)
            .await
            .map_err(internal)?;
        sqlx::query(
            "INSERT INTO `category_translations` (`category_id`, `locale`, `name`, `slug`, `url_path`, `created_at`, `updated_at`) VALUES (?, 'en', ?, ?, ?, ?, ?)",
        )
        .bind(category_id)
        .bind(name)
        .bind(slug)
        .bind(slug)
        .bind(&stamp)
        .bind(&stamp)
        .execute(pool)
        .await
        .map_err(internal)?;
    }

    // Filterable attributes on root + demo categories.
    for code in ["price", "color", "size", "brand", "new"] {
        let attribute_id = attr_id(code).await;
        if attribute_id > 0 {
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO `category_filterable_attributes` (`category_id`, `attribute_id`) VALUES (1, ?)",
            )
            .bind(attribute_id)
            .execute(pool)
            .await;
            for offset in 2..6 {
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO `category_filterable_attributes` (`category_id`, `attribute_id`) VALUES (?, ?)",
                )
                .bind(offset)
                .bind(attribute_id)
                .execute(pool)
                .await;
            }
        }
    }

    // Simple products.
    let products: &[(&str, &str, &str, f64, i64, &str, &str)] = &[
        ("EZ-WATCH-01", "Classic Analog Watch", "electronics", 89.99, 3, "A timeless analog watch with a stainless steel case.", "The Classic Analog Watch pairs a minimalist dial with a durable stainless steel bracelet. Water resistant up to 5 ATM."),
        ("EZ-HEAD-01", "Wireless Headphones", "electronics", 149.0, 3, "Over-ear wireless headphones with active noise cancelling.", "Studio-quality sound with 30 hours of battery life, plush ear cushions and active noise cancelling."),
        ("EZ-TEE-01", "Cotton Crew T-Shirt", "men", 24.99, 1, "100% organic cotton t-shirt.", "Soft, breathable and pre-shrunk. A wardrobe staple in organic cotton with reinforced stitching."),
        ("EZ-JEAN-01", "Slim Fit Jeans", "men", 59.99, 2, "Slim fit jeans in stretch denim.", "Five-pocket jeans in comfortable stretch denim with a modern slim cut."),
        ("EZ-DRESS-01", "Summer Wrap Dress", "women", 69.99, 1, "Flowy wrap dress in viscose.", "A flattering wrap silhouette in breezy viscose, perfect for warm days and layering."),
        ("EZ-BAG-01", "Leather Tote Bag", "women", 119.99, 2, "Handcrafted leather tote.", "Handcrafted from full-grain leather with an interior zip pocket and magnetic closure."),
        ("EZ-MUG-01", "Ceramic Coffee Mug", "home", 14.99, 3, "350 ml stoneware mug.", "A generous 350 ml stoneware mug with a matte glaze. Dishwasher and microwave safe."),
        ("EZ-LAMP-01", "Minimal Desk Lamp", "home", 49.99, 3, "LED desk lamp with touch dimming.", "Three light temperatures, touch dimming and a slim aluminium body."),
    ];
    let category_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT `id` FROM `categories` WHERE `parent_id` = 1 ORDER BY `id`",
    )
    .fetch_all(pool)
    .await
    .map_err(internal)?
    .into_iter()
    .filter_map(Some)
    .collect();

    let mut index = 0usize;
    for (sku, name, slug, price, color_option, short_description, description) in products {
        let category_id = category_ids
            .iter()
            .find(|_| true)
            .copied()
            .unwrap_or(1);
        let _ = slug;
        // Find the category id by slug translation.
        let category_id: i64 = sqlx::query_scalar(
            "SELECT `c`.`id` FROM `categories` `c` INNER JOIN `category_translations` `ct` ON `ct`.`category_id` = `c`.`id` WHERE `ct`.`slug` = ? LIMIT 1",
        )
        .bind(slug)
        .fetch_one(pool)
        .await
        .unwrap_or(category_id);

        sqlx::query(
            "INSERT INTO `products` (`sku`, `type`, `attribute_family_id`, `created_at`, `updated_at`) VALUES (?, 'simple', 1, ?, ?)",
        )
        .bind(sku)
        .bind(&stamp)
        .bind(&stamp)
        .execute(pool)
        .await
        .map_err(internal)?;
        let product_id: i64 = sqlx::query_scalar("SELECT MAX(`id`) FROM `products`")
            .fetch_one(pool)
            .await
            .map_err(internal)?;

        set_attr(product_id, "sku", json!(sku)).await;
        set_attr(product_id, "name", json!(name)).await;
        set_attr(product_id, "url_key", json!(slugify_sku(sku))).await;
        set_attr(product_id, "status", json!(1)).await;
        set_attr(product_id, "new", json!(1)).await;
        set_attr(product_id, "visible_individually", json!(1)).await;
        set_attr(product_id, "manage_stock", json!(1)).await;
        set_attr(product_id, "price", json!(price)).await;
        set_attr(product_id, "short_description", json!(short_description)).await;
        set_attr(product_id, "description", json!(description)).await;
        set_attr(product_id, "weight", json!(1)).await;
        if *color_option > 0 {
            set_attr(product_id, "color", json!(color_option)).await;
        }

        let _ = sqlx::query(
            "INSERT OR IGNORE INTO `product_categories` (`product_id`, `category_id`) VALUES (?, ?)",
        )
        .bind(product_id)
        .bind(category_id)
        .execute(pool)
        .await;
        let _ = sqlx::query(
            "INSERT INTO `product_inventories` (`qty`, `product_id`, `inventory_source_id`, `created_at`, `updated_at`) VALUES (100, ?, 1, ?, ?)",
        )
        .bind(product_id)
        .bind(&stamp)
        .bind(&stamp)
        .execute(pool)
        .await;

        index += 1;
        let _ = index;
    }

    // One configurable product (t-shirt with color variants).
    sqlx::query(
        "INSERT INTO `products` (`sku`, `type`, `attribute_family_id`, `created_at`, `updated_at`) VALUES ('EZ-SNEAKER-01', 'configurable', 1, ?, ?)",
    )
    .bind(&stamp)
    .bind(&stamp)
    .execute(pool)
    .await
    .map_err(internal)?;
    let parent_id: i64 = sqlx::query_scalar("SELECT MAX(`id`) FROM `products`")
        .fetch_one(pool)
        .await
        .map_err(internal)?;
    set_attr(parent_id, "sku", json!("EZ-SNEAKER-01")).await;
    set_attr(parent_id, "name", json!("Running Sneaker")).await;
    set_attr(parent_id, "url_key", json!("running-sneaker")).await;
    set_attr(parent_id, "status", json!(1)).await;
    set_attr(parent_id, "new", json!(1)).await;
    set_attr(parent_id, "visible_individually", json!(1)).await;
    set_attr(parent_id, "price", json!(99.99)).await;
    set_attr(parent_id, "short_description", json!("Lightweight running sneaker.")).await;
    set_attr(parent_id, "description", json!("Cushioned midsole, breathable mesh upper and a grippy outsole. Available in three colors.")).await;

    let _ = sqlx::query(
        "INSERT OR IGNORE INTO `product_super_attributes` (`product_id`, `attribute_id`) VALUES (?, ?)",
    )
    .bind(parent_id)
    .bind(attr_id("color").await)
    .execute(pool)
    .await;

    let colors: &[(i64, &str, f64)] = &[(2, "Green", 99.99), (3, "Blue", 99.99), (1, "Red", 94.99)];
    let sneaker_category: i64 = sqlx::query_scalar(
        "SELECT `c`.`id` FROM `categories` `c` INNER JOIN `category_translations` `ct` ON `ct`.`category_id` = `c`.`id` WHERE `ct`.`slug` = 'men' LIMIT 1",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(2);
    for (color_option_id, color_name, price) in colors {
        let variant_sku = format!("EZ-SNEAKER-01-{}", color_name.to_uppercase());
        sqlx::query(
            "INSERT INTO `products` (`sku`, `type`, `parent_id`, `attribute_family_id`, `created_at`, `updated_at`) VALUES (?, 'simple', ?, 1, ?, ?)",
        )
        .bind(&variant_sku)
        .bind(parent_id)
        .bind(&stamp)
        .bind(&stamp)
        .execute(pool)
        .await
        .map_err(internal)?;
        let variant_id: i64 = sqlx::query_scalar("SELECT MAX(`id`) FROM `products`")
            .fetch_one(pool)
            .await
            .map_err(internal)?;
        set_attr(variant_id, "sku", json!(variant_sku)).await;
        set_attr(variant_id, "name", json!(format!("Running Sneaker - {color_name}"))).await;
        set_attr(variant_id, "status", json!(1)).await;
        set_attr(variant_id, "price", json!(price)).await;
        set_attr(variant_id, "color", json!(color_option_id)).await;
        set_attr(variant_id, "manage_stock", json!(1)).await;
        set_attr(variant_id, "weight", json!(1)).await;
        let _ = sqlx::query(
            "INSERT INTO `product_inventories` (`qty`, `product_id`, `inventory_source_id`, `created_at`, `updated_at`) VALUES (50, ?, 1, ?, ?)",
        )
        .bind(variant_id)
        .bind(&stamp)
        .bind(&stamp)
        .execute(pool)
        .await;
    }
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO `product_categories` (`product_id`, `category_id`) VALUES (?, ?)",
    )
    .bind(parent_id)
    .bind(sneaker_category)
    .execute(pool)
    .await;

    // Materialize product_flat + price/inventory indices for the shop listing.
    let product_ids: Vec<i64> =
        sqlx::query_scalar("SELECT `id` FROM `products` ORDER BY `id`")
            .fetch_all(pool)
            .await
            .map_err(internal)?
            .into_iter()
            .filter_map(Some)
            .collect();
    for product_id in product_ids {
        crate::support::products::refresh_flat(product_id).await;
    }

    Ok(())
}

fn slugify_sku(sku: &str) -> String {
    sku.to_lowercase().replace('_', "-")
}

/// Home sections del tema por defecto (carrusel + categorías + productos).
async fn seed_demo_home() -> Result<(), AppError> {
    let pool = pool()?;
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM `theme_sections`")
        .fetch_one(pool)
        .await
        .map_err(internal)?;
    if existing > 0 {
        return Ok(());
    }
    let stamp = now();

    let slides = json!({
        "slides": [
            {"image": "", "title": "New Season", "link": "/products"},
            {"image": "", "title": "Up to 30% off electronics", "link": "/products?category=electronics"},
            {"image": "", "title": "Free shipping over 50 EUR", "link": "/products"},
        ]
    });
    let _ = sqlx::query(
        "INSERT INTO `theme_sections` (`type`, `name`, `sort_order`, `status`, `channel_id`, `theme_code`, `created_at`, `updated_at`) VALUES ('image_carousel', 'Hero Slider', 1, 1, 1, 'default', ?, ?)",
    )
    .bind(&stamp)
    .bind(&stamp)
    .execute(pool)
    .await
    .map_err(internal)?;
    let section_id: i64 = sqlx::query_scalar("SELECT MAX(`id`) FROM `theme_sections`")
        .fetch_one(pool)
        .await
        .map_err(internal)?;
    let _ = sqlx::query(
        "INSERT INTO `theme_section_translations` (`theme_section_id`, `locale`, `options`) VALUES (?, 'en', ?)",
    )
    .bind(section_id)
    .bind(slides.to_string())
    .execute(pool)
    .await
    .map_err(internal)?;

    let _ = sqlx::query(
        "INSERT INTO `theme_sections` (`type`, `name`, `sort_order`, `status`, `channel_id`, `theme_code`, `created_at`, `updated_at`) VALUES ('category_carousel', 'Shop by Category', 2, 1, 1, 'default', ?, ?)",
    )
    .bind(&stamp)
    .bind(&stamp)
    .execute(pool)
    .await
    .map_err(internal)?;
    let section_id: i64 = sqlx::query_scalar("SELECT MAX(`id`) FROM `theme_sections`")
        .fetch_one(pool)
        .await
        .map_err(internal)?;
    let _ = sqlx::query(
        "INSERT INTO `theme_section_translations` (`theme_section_id`, `locale`, `options`) VALUES (?, 'en', ?)",
    )
    .bind(section_id)
    .bind(json!({"title": "Shop by Category", "filters": {"parent_id": 1}, "limit": 8}).to_string())
    .execute(pool)
    .await
    .map_err(internal)?;

    let _ = sqlx::query(
        "INSERT INTO `theme_sections` (`type`, `name`, `sort_order`, `status`, `channel_id`, `theme_code`, `created_at`, `updated_at`) VALUES ('product_carousel', 'New Products', 3, 1, 1, 'default', ?, ?)",
    )
    .bind(&stamp)
    .bind(&stamp)
    .execute(pool)
    .await
    .map_err(internal)?;
    let section_id: i64 = sqlx::query_scalar("SELECT MAX(`id`) FROM `theme_sections`")
        .fetch_one(pool)
        .await
        .map_err(internal)?;
    let _ = sqlx::query(
        "INSERT INTO `theme_section_translations` (`theme_section_id`, `locale`, `options`) VALUES (?, 'en', ?)",
    )
    .bind(section_id)
    .bind(json!({"title": "New Products", "filters": {"sort": "newest"}, "limit": 8}).to_string())
    .execute(pool)
    .await
    .map_err(internal)?;

    let _ = sqlx::query(
        "INSERT INTO `theme_sections` (`type`, `name`, `sort_order`, `status`, `channel_id`, `theme_code`, `created_at`, `updated_at`) VALUES ('static_content', 'Offer Banner', 4, 1, 1, 'default', ?, ?)",
    )
    .bind(&stamp)
    .bind(&stamp)
    .execute(pool)
    .await
    .map_err(internal)?;
    let section_id: i64 = sqlx::query_scalar("SELECT MAX(`id`) FROM `theme_sections`")
        .fetch_one(pool)
        .await
        .map_err(internal)?;
    let _ = sqlx::query(
        "INSERT INTO `theme_section_translations` (`theme_section_id`, `locale`, `options`) VALUES (?, 'en', ?)",
    )
    .bind(section_id)
    .bind(
        json!({"title": "Get ready for the new season", "subtitle": "Up to 30% off selected items", "button": {"title": "Shop now", "link": "/products"}})
            .to_string(),
    )
    .execute(pool)
    .await
    .map_err(internal)?;

    Ok(())
}

/// Reglas demo: 10% sin cupón para todos + cupón WELCOME10 (10 EUR de
/// descuento fijo al carrito, límite 100 usos).
async fn seed_demo_cart_rules() -> Result<(), AppError> {
    let pool = pool()?;
    let existing: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM `cart_rules`").fetch_one(pool).await.map_err(internal)?;
    if existing > 0 {
        return Ok(());
    }
    let stamp = now();

    // 10% sin cupón.
    sqlx::query(
        "INSERT INTO `cart_rules` (`name`, `description`, `status`, `coupon_type`, `condition_type`, `action_type`, `discount_amount`, `apply_to_shipping`, `free_shipping`, `sort_order`, `created_at`, `updated_at`) \
         VALUES ('10% off everything', 'Automatic 10% discount on the whole cart.', 1, 1, 1, 'by_percent', 10, 0, 0, 0, ?, ?)",
    )
    .bind(&stamp)
    .bind(&stamp)
    .execute(pool)
    .await
    .map_err(internal)?;
    let rule_id: i64 = sqlx::query_scalar("SELECT MAX(`id`) FROM `cart_rules`").fetch_one(pool).await.map_err(internal)?;
    for channel_id in [1] {
        let _ = sqlx::query("INSERT OR IGNORE INTO `cart_rule_channels` (`cart_rule_id`, `channel_id`) VALUES (?, ?)")
            .bind(rule_id).bind(channel_id).execute(pool).await;
    }
    for group_id in [1, 2] {
        let _ = sqlx::query("INSERT OR IGNORE INTO `cart_rule_customer_groups` (`cart_rule_id`, `customer_group_id`) VALUES (?, ?)")
            .bind(rule_id).bind(group_id).execute(pool).await;
    }

    // Cupón WELCOME10: 10 EUR fijos.
    sqlx::query(
        "INSERT INTO `cart_rules` (`name`, `description`, `status`, `coupon_type`, `condition_type`, `action_type`, `discount_amount`, `sort_order`, `created_at`, `updated_at`) \
         VALUES ('Welcome coupon', '10 EUR off your first order with the coupon WELCOME10.', 1, 2, 1, 'by_fixed', 10, 1, ?, ?)",
    )
    .bind(&stamp)
    .bind(&stamp)
    .execute(pool)
    .await
    .map_err(internal)?;
    let rule_id: i64 = sqlx::query_scalar("SELECT MAX(`id`) FROM `cart_rules`").fetch_one(pool).await.map_err(internal)?;
    for channel_id in [1] {
        let _ = sqlx::query("INSERT OR IGNORE INTO `cart_rule_channels` (`cart_rule_id`, `channel_id`) VALUES (?, ?)")
            .bind(rule_id).bind(channel_id).execute(pool).await;
    }
    for group_id in [1, 2] {
        let _ = sqlx::query("INSERT OR IGNORE INTO `cart_rule_customer_groups` (`cart_rule_id`, `customer_group_id`) VALUES (?, ?)")
            .bind(rule_id).bind(group_id).execute(pool).await;
    }
    let _ = sqlx::query(
        "INSERT INTO `cart_rule_coupons` (`code`, `usage_limit`, `usage_per_customer`, `times_used`, `type`, `is_primary`, `cart_rule_id`, `created_at`, `updated_at`) \
         VALUES ('WELCOME10', 100, 1, 0, 0, 1, ?, ?, ?)",
    )
    .bind(rule_id)
    .bind(&stamp)
    .bind(&stamp)
    .execute(pool)
    .await
    .map_err(internal)?;

    Ok(())
}

/// Regla de catálogo demo: -15% a todos los productos (para el grupo guest).
async fn seed_demo_catalog_rule() -> Result<(), AppError> {
    let pool = pool()?;
    let existing: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM `catalog_rules`").fetch_one(pool).await.map_err(internal)?;
    if existing > 0 {
        return Ok(());
    }
    let stamp = now();
    sqlx::query(
        "INSERT INTO `catalog_rules` (`name`, `description`, `status`, `condition_type`, `action_type`, `discount_amount`, `sort_order`, `created_at`, `updated_at`) \
         VALUES ('15% catalog sale', 'Automatic 15% discount on every product price.', 1, 1, 'by_percent', 15, 0, ?, ?)",
    )
    .bind(&stamp)
    .bind(&stamp)
    .execute(pool)
    .await
    .map_err(internal)?;
    let rule_id: i64 = sqlx::query_scalar("SELECT MAX(`id`) FROM `catalog_rules`").fetch_one(pool).await.map_err(internal)?;
    let _ = sqlx::query("INSERT OR IGNORE INTO `catalog_rule_channels` (`catalog_rule_id`, `channel_id`) VALUES (?, 1)")
        .bind(rule_id).execute(pool).await;
    for group_id in [1, 2] {
        let _ = sqlx::query("INSERT OR IGNORE INTO `catalog_rule_customer_groups` (`catalog_rule_id`, `customer_group_id`) VALUES (?, ?)")
            .bind(rule_id).bind(group_id).execute(pool).await;
    }
    // Reindexa los precios de todo el catálogo con la regla aplicada.
    if let Err(error) = crate::support::catalog_rules::reindex_all().await {
        larust_support::tracing::error!(%error, "catalog rules reindex failed");
    }
    Ok(())
}
