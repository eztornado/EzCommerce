//! Forward-only SQL migrations with a small dialect shim.
//!
//! Schema files are written SQLite-flavoured (what `sqlx::Any` can decode
//! natively) and the handful of dialect differences are rewritten when
//! `DB_CONNECTION=mysql`/`mariadb`. The bookkeeping table follows the
//! framework's `_migrations` shape.

use std::path::PathBuf;

use larust_core::AppError;
use larust_support::orm::{backend, pool, Backend};

fn internal(error: impl std::error::Error + Send + Sync + 'static) -> AppError {
    AppError::Internal(Box::new(error))
}

fn migrations_dir() -> PathBuf {
    crate::paths().migrations()
}

pub async fn migrate() -> Result<(), AppError> {
    run(false).await
}

pub async fn migrate_fresh() -> Result<(), AppError> {
    run(true).await
}

async fn run(fresh: bool) -> Result<(), AppError> {
    let pool = pool()?;

    if fresh {
        drop_all_tables().await?;
    }

    let create_bookkeeping = match backend() {
        Backend::Sqlite => {
            "CREATE TABLE IF NOT EXISTS `_migrations` (\
                `name` TEXT PRIMARY KEY, \
                `applied_at` TEXT NOT NULL DEFAULT (datetime('now'))\
             )"
        }
        Backend::MySql => {
            "CREATE TABLE IF NOT EXISTS `_migrations` (\
                `name` VARCHAR(255) PRIMARY KEY, \
                `applied_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP\
             )"
        }
        Backend::Postgres => {
            "CREATE TABLE IF NOT EXISTS \"_migrations\" (\
                \"name\" TEXT PRIMARY KEY, \
                \"applied_at\" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP\
             )"
        }
    };
    sqlx::query(create_bookkeeping)
        .execute(pool)
        .await
        .map_err(internal)?;

    let mut files: Vec<PathBuf> = std::fs::read_dir(migrations_dir())
        .map_err(internal)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "sql"))
        .collect();
    files.sort();

    for path in files {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();

        let applied: Option<(String,)> = sqlx::query_as(
            "SELECT `name` FROM `_migrations` WHERE `name` = ?",
        )
        .bind(&name)
        .fetch_optional(pool)
        .await
        .map_err(internal)?;
        if applied.is_some() {
            continue;
        }

        let sql = std::fs::read_to_string(&path).map_err(internal)?;
        let sql = match backend() {
            Backend::MySql => mysql_dialect(&sql),
            Backend::Sqlite => sqlite_dialect(&sql),
            Backend::Postgres => sql,
        };

        for statement in split_statements(&sql) {
            sqlx::query(&statement)
                .execute(pool)
                .await
                .map_err(|error| {
                    internal(std::io::Error::other(format!(
                        "migration {name} failed: {error}"
                    )))
                })?;
        }

        sqlx::query("INSERT INTO `_migrations` (`name`) VALUES (?)")
            .bind(&name)
            .execute(pool)
            .await
            .map_err(internal)?;

        larust_support::tracing::info!(migration = %name, "migrated");
    }

    Ok(())
}

async fn drop_all_tables() -> Result<(), AppError> {
    let pool = pool()?;
    let tables: Vec<String> = match backend() {
        Backend::Sqlite => {
            let rows = sqlx::query(
                "SELECT `name` FROM sqlite_master \
                 WHERE `type` = 'table' AND `name` NOT LIKE 'sqlite_%' ORDER BY `name`",
            )
            .fetch_all(pool)
            .await
            .map_err(internal)?;
            rows.iter()
                .filter_map(|row| {
                    use sqlx::Row;
                    row.try_get::<String, _>(0).ok()
                })
                .collect()
        }
        Backend::MySql => {
            let rows = sqlx::query(
                "SELECT `table_name` FROM information_schema.tables \
                 WHERE `table_schema` = DATABASE()",
            )
            .fetch_all(pool)
            .await
            .map_err(internal)?;
            rows.iter()
                .filter_map(|row| {
                    use sqlx::Row;
                    row.try_get::<String, _>(0).ok()
                })
                .collect()
        }
        Backend::Postgres => {
            let rows = sqlx::query(
                "SELECT tablename FROM pg_tables WHERE schemaname = 'public'",
            )
            .fetch_all(pool)
            .await
            .map_err(internal)?;
            rows.iter()
                .filter_map(|row| {
                    use sqlx::Row;
                    row.try_get::<String, _>(0).ok()
                })
                .collect()
        }
    };

    for table in tables {
        if !table
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            continue;
        }
        let statement = match backend() {
            Backend::Postgres => format!("DROP TABLE IF EXISTS \"{table}\" CASCADE"),
            _ => format!("DROP TABLE IF EXISTS `{table}`"),
        };
        sqlx::query(&statement)
            .execute(pool)
            .await
            .map_err(internal)?;
    }

    Ok(())
}

/// Rewrites the only constructs our schema files use that differ between
/// SQLite and MySQL. Identifiers are backtick-quoted in every file (SQLite
/// accepts that too), so no other translation is needed.
fn mysql_dialect(sql: &str) -> String {
    // Runtime upserts written for SQLite are rewritten for MySQL as well.
    let sql = sql.replace("INSERT OR IGNORE INTO", "INSERT IGNORE INTO");
    let decimal = regex::Regex::new(r"(?i)DECIMAL\(\d+\s*,\s*\d+\)").unwrap();
    let sql = decimal.replace_all(&sql, "DOUBLE");
    let sql = sql.replace(
        "INTEGER PRIMARY KEY AUTOINCREMENT",
        // Signed, not `UNSIGNED`: `sqlx::Any` decodes MySQL unsigned bigints
        // poorly when mapped to guaranteed i64 values.
        "BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY",
    );
    let sql = sql.replace(
        "BIGINT PRIMARY KEY AUTOINCREMENT",
        "BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY",
    );
    // `sqlx::Any` has no MySQL mapping for DATE/DATETIME/TIMESTAMP, DECIMAL or
    // BOOLEAN (TINYINT). Dates are stored as `YYYY-MM-DD HH:MM:SS` strings in
    // VARCHAR columns - exactly what the controllers read/write - decimals as
    // DOUBLE and booleans as INTEGER.
    let datetime = regex::Regex::new(r"(?i)\b(DATETIME|TIMESTAMP)\b").unwrap();
    let sql = datetime.replace_all(&sql, "VARCHAR(255)");
    let date = regex::Regex::new(r"(?i)\bDATE\b").unwrap();
    let sql = date.replace_all(&sql, "VARCHAR(10)");
    let boolean = regex::Regex::new(r"(?i)\bBOOLEAN\b").unwrap();
    boolean.replace_all(&sql, "INTEGER").into_owned()
}

/// `sqlx::Any` only understands SQLite's native storage classes. Declared
/// `DATETIME`/`BOOLEAN`/`DECIMAL`/`VARCHAR` columns map to type names the Any
/// driver refuses to decode, so the SQLite schema uses `TEXT`/`INTEGER`/`REAL`
/// instead. Values are unchanged: dates stay `YYYY-MM-DD HH:MM:SS` strings and
/// booleans stay `0`/`1`, exactly what the MySQL schema stores too.
fn sqlite_dialect(sql: &str) -> String {
    let decimal = regex::Regex::new(r"(?i)DECIMAL\(\d+\s*,\s*\d+\)").unwrap();
    let varchar = regex::Regex::new(r"(?i)VARCHAR\(\d+\)").unwrap();
    // SQLite's FK enforcement is off in sqlx::Any and type equality rules
    // differ from MySQL, so `FOREIGN KEY ... REFERENCES ...` clauses are
    // stripped; production MySQL keeps cascades.
    let foreign_keys =
        regex::Regex::new(r"(?i),?\s*FOREIGN KEY\s*\([^)]*\)\s*REFERENCES\s+[^\n]*").unwrap();
    let sql = foreign_keys.replace_all(sql, "");
    let json = regex::Regex::new(r"(?i)\bJSON\b").unwrap();
    let longtext = regex::Regex::new(r"(?i)\bLONGTEXT\b").unwrap();
    let sql = decimal.replace_all(&sql, "REAL");
    let sql = varchar.replace_all(&sql, "TEXT");
    let sql = json.replace_all(&sql, "TEXT");
    let sql = longtext.replace_all(&sql, "TEXT");
    // SQLite only allows AUTOINCREMENT on INTEGER PRIMARY KEY; BIGINT maps to
    // the same INTEGER affinity (64-bit values are stored losslessly).
    sql.replace("DATETIME", "TEXT")
        .replace("DATE", "TEXT")
        .replace("BOOLEAN", "INTEGER")
        .replace("BIGINT PRIMARY KEY AUTOINCREMENT", "INTEGER PRIMARY KEY AUTOINCREMENT")
}

fn split_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for line in sql.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }
        for character in line.chars() {
            if escaped {
                current.push(character);
                escaped = false;
                continue;
            }
            match character {
                '\\' if in_single || in_double => {
                    current.push(character);
                    escaped = true;
                }
                '\'' if !in_double => {
                    in_single = !in_single;
                    current.push(character);
                }
                '"' if !in_single => {
                    in_double = !in_double;
                    current.push(character);
                }
                ';' if !in_single && !in_double => {
                    let statement = current.trim().to_string();
                    if !statement.is_empty() {
                        statements.push(statement);
                    }
                    current.clear();
                }
                _ => current.push(character),
            }
        }
        current.push('\n');
    }

    let statement = current.trim().to_string();
    if !statement.is_empty() {
        statements.push(statement);
    }
    statements
}
