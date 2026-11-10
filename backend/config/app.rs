use larust_support::serde_json::{json, Value};

/// Application configuration, mirroring Bagisto's `config/*.php` values that
/// are relevant to the API.
pub fn config() -> Value {
    let mut config = json!({});

    config["app_name"] = json!(larust_support::config_env::env_or("APP_NAME", "EzCommerce"));
    config["app_env"] = json!(larust_support::config_env::env_or("APP_ENV", "local"));
    config["app_port"] = json!(larust_support::config_env::app_port_or(34187));
    config["app_debug"] = json!(larust_support::config_env::env_bool("APP_DEBUG", false));
    config["app_url"] =
        json!(larust_support::config_env::env_or("APP_URL", "http://localhost:8000"));
    config["api_prefix"] = json!(larust_support::config_env::env_or("API_PREFIX", "/api"));
    config["app_locale"] = json!(larust_support::config_env::env_or("APP_LOCALE", "es"));
    config["app_fallback_locale"] =
        json!(larust_support::config_env::env_or("APP_FALLBACK_LOCALE", "es"));
    config["app_currency"] = json!(larust_support::config_env::env_or("APP_CURRENCY", "EUR"));

    // Mail - the framework's own SMTP/log transport.
    config["mail_mailer"] = json!(larust_support::config_env::env_or("MAIL_MAILER", "log"));
    config["mail_host"] = json!(larust_support::config_env::env_or("MAIL_HOST", "127.0.0.1"));
    config["mail_port"] = json!(
        larust_support::config_env::env_or("MAIL_PORT", "2525")
            .parse::<u16>()
            .unwrap_or(2525)
    );
    config["mail_username"] = json!(larust_support::config_env::env_or("MAIL_USERNAME", ""));
    config["mail_password"] = json!(larust_support::config_env::env_or("MAIL_PASSWORD", ""));
    config["mail_encryption"] = json!(larust_support::config_env::env_or("MAIL_ENCRYPTION", "tls"));
    config["mail_from_address"] =
        json!(larust_support::config_env::env_or("MAIL_FROM_ADDRESS", "shop@example.com"));
    config["mail_from_name"] = json!(larust_support::config_env::env_or("MAIL_FROM_NAME", "EzCommerce"));
    config["resend_api_key"] = json!(larust_support::config_env::env("RESEND_API_KEY"));

    // JWT - shared with Bagisto's token contract (iss/iat/exp/nbf/jti/sub).
    config["jwt_secret"] =
        json!(larust_support::config_env::env_or("JWT_SECRET", "ezcommerce-dev-secret"));
    config["jwt_ttl_minutes"] = json!(
        larust_support::config_env::env_or("JWT_TTL", "60")
            .parse::<i64>()
            .unwrap_or(60)
    );
    config["jwt_refresh_ttl_minutes"] = json!(
        larust_support::config_env::env_or("JWT_REFRESH_TTL", "20160")
            .parse::<i64>()
            .unwrap_or(20160)
    );
    config["jwt_blacklist_enabled"] =
        json!(larust_support::config_env::env_bool("JWT_BLACKLIST_ENABLED", true));
    config["password_reset_expire"] = json!(
        larust_support::config_env::env_or("PASSWORD_RESET_EXPIRE", "60")
            .parse::<i64>()
            .unwrap_or(60)
    );

    // Seeds.
    config["admin_email"] =
        json!(larust_support::config_env::env_or("ADMIN_EMAIL", "admin@example.com"));
    config["admin_password"] =
        json!(larust_support::config_env::env_or("ADMIN_PASSWORD", "admin123"));
    config["seed_demo_data"] = json!(larust_support::config_env::env_bool("SEED_DEMO_DATA", true));

    config
}
