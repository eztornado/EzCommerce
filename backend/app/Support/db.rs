//! Generic, table-agnostic data access.
//!
//! The source application drives most of its admin CRUD from the
//! `custom_models` tables and creates the physical tables at runtime, so its
//! controllers never had typed models for `eventos`, `noticias`, `socios`,
//! etc. To keep that same flexibility this module exposes small dynamic
//! helpers over `sqlx::Any` (which Larust's ORM already uses): rows are
//! decoded to `serde_json::Map` values and inputs are bound straight from
//! JSON.

use chrono::{NaiveDateTime, TimeZone, Utc};
use larust_support::orm::{backend, pool, Backend};
use larust_support::serde_json::{Map, Number, Value};
use sqlx::{Column, Row, TypeInfo, ValueRef};

use super::response::{ApiError, ApiResult};

pub type RowMap = Map<String, Value>;

/// Empty row helper for `unwrap_or` on `Option<RowMap>`.
pub fn empty_row() -> RowMap {
    Map::new()
}

/// Converts a JSON object literal into a mutable row map.
pub fn to_map(value: Value) -> RowMap {
    value.as_object().cloned().unwrap_or_else(Map::new)
}

/// Accepts either a `RowMap` or a `Value` object as insert/update payload.
pub trait AsData {
    fn fields(&self) -> Option<&Map<String, Value>>;
}

impl AsData for RowMap {
    fn fields(&self) -> Option<&Map<String, Value>> {
        Some(self)
    }
}

impl AsData for Value {
    fn fields(&self) -> Option<&Map<String, Value>> {
        self.as_object()
    }
}

fn quote_ident(name: &str) -> ApiResult<String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(ApiError::bad_request(format!("Invalid identifier: {name}")));
    }
    Ok(match backend() {
        Backend::Sqlite => format!("\"{name}\""),
        Backend::MySql => format!("`{name}`"),
        Backend::Postgres => format!("\"{name}\""),
    })
}

fn quote_table(name: &str) -> ApiResult<String> {
    quote_ident(name)
}

/// `true` when the table exists in the active database.
pub async fn table_exists(table: &str) -> ApiResult<bool> {
    let db_pool = pool()?;
    let exists: Option<String> = match backend() {
        Backend::Sqlite => {
            let row = sqlx::query("SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?")
                .bind(table)
                .fetch_optional(db_pool)
                .await?;
            row.and_then(|row| row.try_get::<String, _>(0).ok())
        }
        Backend::Postgres => {
            let row = sqlx::query("SELECT tablename FROM pg_tables WHERE schemaname = 'public' AND tablename = $1")
                .bind(table)
                .fetch_optional(db_pool)
                .await?;
            row.and_then(|row| row.try_get::<String, _>(0).ok())
        }
        Backend::MySql => {
            let row = sqlx::query("SELECT table_name FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = ?")
                .bind(table)
                .fetch_optional(db_pool)
                .await?;
            row.and_then(|row| row.try_get::<String, _>(0).ok())
        }
    };
    Ok(exists.is_some())
}

/// Column names of a table, in database order.
pub async fn columns(table: &str) -> ApiResult<Vec<String>> {
    let db_pool = pool()?;
    let names: Vec<String> = match backend() {
        Backend::Sqlite => {
            let sql = format!("PRAGMA table_info({})", quote_table(table)?);
            let rows = sqlx::query(&sql).fetch_all(db_pool).await?;
            rows.iter()
                .filter_map(|row| row.try_get::<String, _>("name").ok())
                .collect()
        }
        Backend::Postgres => {
            let rows = sqlx::query(
                "SELECT column_name FROM information_schema.columns WHERE table_schema = 'public' AND table_name = $1 ORDER BY ordinal_position",
            )
            .bind(table)
            .fetch_all(db_pool)
            .await?;
            rows.iter()
                .filter_map(|row| row.try_get::<String, _>(0).ok())
                .collect()
        }
        Backend::MySql => {
            let rows = sqlx::query(
                "SELECT column_name FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = ? ORDER BY ordinal_position",
            )
            .bind(table)
            .fetch_all(db_pool)
            .await?;
            rows.iter()
                .filter_map(|row| row.try_get::<String, _>(0).ok())
                .collect()
        }
    };
    Ok(names)
}

fn decode_cell(row: &sqlx::any::AnyRow, index: usize) -> Value {
    let raw = match row.try_get_raw(index) {
        Ok(raw) => raw,
        Err(_) => return Value::Null,
    };
    if raw.is_null() {
        return Value::Null;
    }
    let type_name = row
        .column(index)
        .type_info()
        .name()
        .to_ascii_uppercase();

    let as_string = || row.try_get::<String, _>(index).ok();
    let as_i64 = || row.try_get::<i64, _>(index).ok();
    let as_f64 = || row.try_get::<f64, _>(index).ok();
    let as_bool = || row.try_get::<bool, _>(index).ok();
    let as_bytes = || row.try_get::<Vec<u8>, _>(index).ok();

    if type_name.contains("BOOL") {
        if let Some(value) = as_bool() {
            return Value::Bool(value);
        }
        if let Some(value) = as_i64() {
            return Value::Bool(value != 0);
        }
    }
    if type_name.contains("INT") {
        if let Some(value) = as_i64() {
            return Value::from(value);
        }
        if let Some(value) = as_bool() {
            return Value::from(if value { 1 } else { 0 });
        }
    }
    if type_name.contains("REAL")
        || type_name.contains("DOUB")
        || type_name.contains("FLOA")
        || type_name.contains("NUMERIC")
        || type_name.contains("DECIMAL")
    {
        if let Some(value) = as_f64() {
            // SQLite stores integral values in REAL columns as `10.0`; PHP/the
            // live MySQL columns serialized them as `10`. Keep the JSON
            // contract identical while preserving actual decimals.
            if value.fract() == 0.0 && value.abs() <= 9_007_199_254_740_992.0 {
                return Value::from(value as i64);
            }
            return Number::from_f64(value)
                .map(Value::Number)
                .unwrap_or(Value::Null);
        }
        if let Some(value) = as_string() {
            return Value::String(value);
        }
    }
    if type_name.contains("BLOB") || type_name.contains("BINARY") {
        if let Some(bytes) = as_bytes() {
            return bytes_to_value(bytes);
        }
    }
    if type_name.contains("DATE")
        || type_name.contains("TIME")
        || type_name.contains("CHAR")
        || type_name.contains("TEXT")
        || type_name.contains("CLOB")
        || type_name.contains("JSON")
        || type_name.contains("ENUM")
    {
        if let Some(value) = as_string() {
            return Value::String(value);
        }
    }

    // Fallback order, most specific first.
    if let Some(value) = as_i64() {
        return Value::from(value);
    }
    if let Some(value) = as_f64() {
        if let Some(number) = Number::from_f64(value) {
            return Value::Number(number);
        }
    }
    if let Some(value) = as_bool() {
        return Value::Bool(value);
    }
    if let Some(value) = as_string() {
        return Value::String(value);
    }
    if let Some(bytes) = as_bytes() {
        return bytes_to_value(bytes);
    }
    Value::Null
}

/// Text columns arrive as `Blob` through `sqlx::Any` (MySQL `TEXT`); decode
/// them as UTF-8 text and fall back to base64 only for real binary payloads.
fn bytes_to_value(bytes: Vec<u8>) -> Value {
    match String::from_utf8(bytes) {
        Ok(text) => Value::String(text),
        Err(error) => {
            use base64::Engine;
            Value::String(
                base64::engine::general_purpose::STANDARD.encode(error.into_bytes()),
            )
        }
    }
}

fn row_to_map(row: &sqlx::any::AnyRow) -> RowMap {
    let mut map = Map::new();
    for (index, column) in row.columns().iter().enumerate() {
        map.insert(column.name().to_string(), decode_cell(row, index));
    }
    map
}

fn bind_value<'q>(
    query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>,
    value: &'q Value,
) -> sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>> {
    match value {
        Value::Null => query.bind(Option::<String>::None),
        Value::Bool(value) => query.bind(if *value { 1_i64 } else { 0_i64 }),
        Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                query.bind(value)
            } else if let Some(value) = number.as_f64() {
                query.bind(value)
            } else {
                query.bind(number.to_string())
            }
        }
        Value::String(value) => query.bind(value.clone()),
        Value::Array(_) | Value::Object(_) => query.bind(value.to_string()),
    }
}

/// Runs a `SELECT` and decodes every row into a JSON object.
pub async fn select(sql: &str, binds: &[Value]) -> ApiResult<Vec<RowMap>> {
    let db_pool = pool()?;
    let mut query = sqlx::query(sql);
    for value in binds {
        query = bind_value(query, value);
    }
    let rows = query.fetch_all(db_pool).await?;
    Ok(rows.iter().map(row_to_map).collect())
}

pub async fn select_one(sql: &str, binds: &[Value]) -> ApiResult<Option<RowMap>> {
    let db_pool = pool()?;
    let mut query = sqlx::query(sql);
    for value in binds {
        query = bind_value(query, value);
    }
    let row = query.fetch_optional(db_pool).await?;
    Ok(row.as_ref().map(row_to_map))
}

pub async fn scalar_i64(sql: &str, binds: &[Value]) -> ApiResult<i64> {
    let db_pool = pool()?;
    let mut query = sqlx::query(sql);
    for value in binds {
        query = bind_value(query, value);
    }
    let row = query.fetch_one(db_pool).await?;
    Ok(row.try_get::<i64, _>(0).unwrap_or_default())
}

pub async fn execute(sql: &str, binds: &[Value]) -> ApiResult<u64> {
    let db_pool = pool()?;
    let mut query = sqlx::query(sql);
    for value in binds {
        query = bind_value(query, value);
    }
    let result = query.execute(db_pool).await?;
    Ok(result.rows_affected())
}

/// `SELECT * FROM table WHERE <conditions>` without ordering. Conditions use
/// simple `column = ?` fragments built by the caller from whitelisted
/// identifiers (never raw user input).
pub async fn all(table: &str, where_sql: Option<&str>, binds: &[Value]) -> ApiResult<Vec<RowMap>> {
    let table = quote_table(table)?;
    let sql = match where_sql {
        Some(clause) if !clause.is_empty() => format!("SELECT * FROM {table} WHERE {clause}"),
        _ => format!("SELECT * FROM {table}"),
    };
    select(&sql, binds).await
}

pub async fn find_by_id(table: &str, id: i64) -> ApiResult<Option<RowMap>> {
    let table = quote_table(table)?;
    select_one(&format!("SELECT * FROM {table} WHERE id = ? LIMIT 1"), &[Value::from(id)]).await
}

/// Inserts `data` after dropping unknown/key columns, and returns the new id.
pub async fn insert<D: AsData>(table: &str, data: &D) -> ApiResult<i64> {
    let table_name = quote_table(table)?;
    let known = columns(table).await?;
    let Some(fields) = data.fields() else {
        return Err(ApiError::bad_request("insert expects an object"));
    };
    let mut usable: Vec<(&String, &Value)> = fields
        .iter()
        .filter(|(key, _)| known.iter().any(|column| column == *key))
        .filter(|(key, _)| key.as_str() != "id")
        .collect();
    usable.sort_by(|a, b| a.0.cmp(b.0));

    if usable.is_empty() {
        let sql = format!("INSERT INTO {table_name} DEFAULT VALUES");
        return last_insert_id(&sql, &[]).await;
    }

    let column_sql = usable
        .iter()
        .map(|(key, _)| quote_ident(key))
        .collect::<ApiResult<Vec<_>>>()?
        .join(", ");
    let placeholders = vec!["?"; usable.len()].join(", ");
    let binds: Vec<Value> = usable.iter().map(|(_, value)| (*value).clone()).collect();
    let sql = format!("INSERT INTO {table_name} ({column_sql}) VALUES ({placeholders})");
    last_insert_id(&sql, &binds).await
}

/// Inserts `data` including an explicit `id` (used by seeders that need
/// deterministic menu/role ids).
pub async fn insert_with_id<D: AsData>(table: &str, data: &D) -> ApiResult<u64> {
    let table_name = quote_table(table)?;
    let known = columns(table).await?;
    let Some(fields) = data.fields() else {
        return Err(ApiError::bad_request("insert_with_id expects an object"));
    };
    let mut usable: Vec<(&String, &Value)> = fields
        .iter()
        .filter(|(key, _)| known.iter().any(|column| column == *key))
        .collect();
    usable.sort_by(|a, b| a.0.cmp(b.0));

    if usable.is_empty() {
        return Ok(0);
    }

    let column_sql = usable
        .iter()
        .map(|(key, _)| quote_ident(key))
        .collect::<ApiResult<Vec<_>>>()?
        .join(", ");
    let placeholders = vec!["?"; usable.len()].join(", ");
    let binds: Vec<Value> = usable.iter().map(|(_, value)| (*value).clone()).collect();
    let sql = format!("INSERT INTO {table_name} ({column_sql}) VALUES ({placeholders})");
    execute(&sql, &binds).await
}

async fn last_insert_id(sql: &str, binds: &[Value]) -> ApiResult<i64> {
    let db_pool = pool()?;
    // `last_insert_rowid()` / `LAST_INSERT_ID()` are per-connection, so the
    // insert and the id lookup must share one pooled connection. Relying on
    // `AnyQueryResult::last_insert_id()` alone returns 0 on sqlx's SQLite
    // driver, which silently broke every `db::insert` caller.
    let mut connection = db_pool.acquire().await?;
    let mut query = sqlx::query(sql);
    for value in binds {
        query = bind_value(query, value);
    }
    let result = query.execute(&mut *connection).await?;
    if let Some(id) = result.last_insert_id() {
        if id != 0 {
            return Ok(id as i64);
        }
    }

    let id_sql = match backend() {
        Backend::MySql => "SELECT LAST_INSERT_ID() AS id",
        _ => "SELECT last_insert_rowid() AS id",
    };
    let row = sqlx::query(id_sql).fetch_one(&mut *connection).await?;
    let id: i64 = row.try_get("id")?;
    Ok(id)
}

/// Updates the row with `id`, touching `updated_at` when the table has one.
pub async fn update<D: AsData>(table: &str, id: i64, data: &D) -> ApiResult<u64> {
    let table_name = quote_table(table)?;
    let Some(fields) = data.fields() else {
        return Err(ApiError::bad_request("update expects an object"));
    };
    let known = columns(table).await?;
    let mut entries: Vec<(&String, &Value)> = fields
        .iter()
        .filter(|(key, _)| known.iter().any(|column| column == *key))
        .filter(|(key, _)| {
            key.as_str() != "id" && key.as_str() != "created_at" && key.as_str() != "updated_at"
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    let mut assignments: Vec<String> = entries
        .iter()
        .map(|(key, _)| Ok(format!("{} = ?", quote_ident(key)?)))
        .collect::<ApiResult<Vec<_>>>()?;

    let mut binds: Vec<Value> = entries.iter().map(|(_, value)| (*value).clone()).collect();
    if known.iter().any(|column| column == "updated_at") {
        assignments.push(format!("{} = ?", quote_ident("updated_at")?));
        binds.push(Value::String(now_db()));
    }
    if assignments.is_empty() {
        return Ok(0);
    }
    binds.push(Value::from(id));
    let sql = format!(
        "UPDATE {table_name} SET {} WHERE id = ?",
        assignments.join(", ")
    );
    execute(&sql, &binds).await
}

/// Hard-deletes one row (the source app calls `forceDelete()` everywhere).
pub async fn delete(table: &str, id: i64) -> ApiResult<u64> {
    let table = quote_table(table)?;
    execute(&format!("DELETE FROM {table} WHERE id = ?"), &[Value::from(id)]).await
}

pub async fn count(table: &str, where_sql: Option<&str>, binds: &[Value]) -> ApiResult<i64> {
    let table = quote_table(table)?;
    let sql = match where_sql {
        Some(clause) if !clause.is_empty() => {
            format!("SELECT COUNT(*) AS total FROM {table} WHERE {clause}")
        }
        _ => format!("SELECT COUNT(*) AS total FROM {table}"),
    };
    scalar_i64(&sql, binds).await
}

/// Laravel's `now()` in the DB's own `Y-m-d H:i:s` format.
pub fn now_db() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string()
}

/// Converts a `Y-m-d H:i:s` (or already ISO) DB timestamp into the ISO-8601
/// shape Eloquent emits: `2025-06-12T10:30:58.000000Z`.
pub fn iso_timestamp(value: &str) -> String {
    if value.contains('T') {
        return value.to_string();
    }
    if let Ok(parsed) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Utc
            .from_utc_datetime(&parsed)
            .format("%Y-%m-%dT%H:%M:%S%.6fZ")
            .to_string();
    }
    value.to_string()
}

const BOOLEAN_COLUMNS: &[(&str, &[&str])] = &[
    ("users", &["active"]),
    ("patrocinadores", &["active"]),
    ("menus_items", &["active"]),
    ("custom_models_fields", &["unique", "nullable", "shortable", "visible", "hidden", "disabled"]),
    ("roles", &[]),
];

/// Applies the casts the Laravel Eloquent models declared (`$casts`,
/// `$appends`) to a decoded row.
pub fn apply_default_casts(table: &str, row: &mut RowMap) {
    for (table_name, columns) in BOOLEAN_COLUMNS {
        if *table_name != table {
            continue;
        }
        for column in *columns {
            if let Some(Value::Number(number)) = row.get(*column) {
                if let Some(value) = number.as_i64() {
                    row.insert((*column).to_string(), Value::Bool(value != 0));
                }
            }
        }
    }

    for column in ["created_at", "updated_at", "email_verified_at", "deleted_at"] {
        if let Some(Value::String(value)) = row.get(column) {
            let iso = iso_timestamp(value);
            row.insert(column.to_string(), Value::String(iso));
        }
    }

    // Eloquent's `$hidden` on `User`.
    if table == "users" {
        row.remove("password");
        row.remove("remember_token");
    }
}

/// Laravel paginator JSON shape (`LengthAwarePaginator`).
pub fn paginate(
    rows: Vec<RowMap>,
    page: i64,
    per_page: i64,
    total: i64,
    path: &str,
) -> Value {
    let last_page = if per_page <= 0 {
        1
    } else {
        ((total as f64) / (per_page as f64)).ceil() as i64
    }
    .max(1);
    let mut links = Vec::new();
    let url = |page: Option<i64>, label: String| {
        Value::Object(
            [
                (
                    "url".to_string(),
                    page.map(|p| Value::String(format!("{path}?page={p}")))
                        .unwrap_or(Value::Null),
                ),
                ("label".to_string(), Value::String(label)),
                ("active".to_string(), Value::Bool(false)),
            ]
            .into_iter()
            .collect(),
        )
    };
    links.push(url(
        if page > 1 { Some(page - 1) } else { None },
        "&laquo; Previous".to_string(),
    ));
    for p in 1..=last_page {
        let mut link = url(Some(p), p.to_string());
        if let Value::Object(map) = &mut link {
            map.insert("active".to_string(), Value::Bool(p == page));
        }
        links.push(link);
    }
    links.push(url(
        if page < last_page { Some(page + 1) } else { None },
        "Next &raquo;".to_string(),
    ));

    let first_page_url = format!("{path}?page=1");
    let last_page_url = format!("{path}?page={last_page}");
    let next_page_url = if page < last_page {
        Value::String(format!("{path}?page={}", page + 1))
    } else {
        Value::Null
    };
    let prev_page_url = if page > 1 {
        Value::String(format!("{path}?page={}", page - 1))
    } else {
        Value::Null
    };

    serde_json::json!({
        "current_page": page,
        "data": rows,
        "first_page_url": first_page_url,
        "from": if rows.is_empty() { Value::Null } else { Value::from((page - 1) * per_page + 1) },
        "last_page": last_page,
        "last_page_url": last_page_url,
        "links": links,
        "next_page_url": next_page_url,
        "path": path,
        "per_page": per_page,
        "prev_page_url": prev_page_url,
        "to": if rows.is_empty() { Value::Null } else { Value::from((page - 1) * per_page + rows.len() as i64) },
        "total": total,
    })
}
