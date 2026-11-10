use larust_support::orm::{ConnectionConfig, DatabaseConnections, Driver};
use std::collections::HashMap;

/// SQLite locally, MySQL/MariaDB for production (set `DB_CONNECTION` in
/// `.env`) - the same arrangement as the rest of the Larust projects.
pub fn config() -> DatabaseConnections {
    let mut connections = HashMap::new();

    connections.insert(
        "sqlite".to_string(),
        ConnectionConfig {
            driver: Driver::Sqlite,
            host: String::new(),
            port: 0,
            database: larust_support::config_env::env_or("DB_DATABASE", "database/database.sqlite"),
            username: String::new(),
            password: String::new(),
            charset: String::new(),
        },
    );

    connections.insert(
        "mysql".to_string(),
        ConnectionConfig {
            driver: Driver::MySql,
            host: larust_support::config_env::env_or("DB_HOST", "127.0.0.1"),
            port: larust_support::config_env::env_or("DB_PORT", "3306")
                .parse::<u16>()
                .unwrap_or(3306),
            database: larust_support::config_env::env_or("DB_DATABASE", "ezcommerce"),
            username: larust_support::config_env::env_or("DB_USERNAME", "root"),
            password: larust_support::config_env::env_or("DB_PASSWORD", ""),
            charset: larust_support::config_env::env_or("DB_CHARSET", "utf8mb4"),
        },
    );

    connections.insert(
        "mariadb".to_string(),
        ConnectionConfig {
            driver: Driver::MySql,
            host: larust_support::config_env::env_or("DB_HOST", "127.0.0.1"),
            port: larust_support::config_env::env_or("DB_PORT", "3306")
                .parse::<u16>()
                .unwrap_or(3306),
            database: larust_support::config_env::env_or("DB_DATABASE", "ezcommerce"),
            username: larust_support::config_env::env_or("DB_USERNAME", "root"),
            password: larust_support::config_env::env_or("DB_PASSWORD", ""),
            charset: larust_support::config_env::env_or("DB_CHARSET", "utf8mb4"),
        },
    );

    DatabaseConnections {
        default: larust_support::config_env::env_or("DB_CONNECTION", "sqlite"),
        connections,
    }
}
