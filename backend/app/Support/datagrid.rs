//! Generic admin DataGrid engine, replicating `Webkul\DataGrid\DataGrid`:
//! request `{filters, sort, pagination, export}` -> response
//! `{columns, actions, mass_actions, records, meta}` with `all` (global
//! search), per-column filters (text/dropdown/range/date-range), sorting and
//! pagination computed in SQL.

use larust_support::serde_json::{json, Map, Value};

use super::db::{self, RowMap};
use super::response::{ApiError, ApiResult};

fn valid_ident(name: &str) -> ApiResult<String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return Err(ApiError::bad_request(format!("Invalid identifier: {name}")));
    }
    Ok(name.to_string())
}

/// Column definition.
pub struct Column {
    pub index: &'static str,
    pub label: &'static str,
    pub kind: &'static str, // string|integer|decimal|boolean|date|datetime
    pub searchable: bool,
    pub filterable: bool,
    pub sortable: bool,
    /// SQL expression (defaults to the index, supports `join.alias`).
    pub source: Option<&'static str>,
    /// Static dropdown options `[{label, value}]`.
    pub options: Option<Value>,
}

impl Column {
    fn sql(&self) -> String {
        self.source.unwrap_or(self.index).to_string()
    }
}

pub struct DataGrid {
    pub table: &'static str,
    /// SQL alias used by `primary`/columns/joins (e.g. `p`).
    pub alias: &'static str,
    pub primary: &'static str,
    pub columns: Vec<Column>,
    /// Extra FROM/JOIN clauses (e.g. `INNER JOIN products ...`).
    pub joins: Vec<String>,
    /// Static WHERE with its binds.
    pub where_clauses: Vec<(String, Vec<Value>)>,
    /// Row actions: `[{title, method, url}]` resolved per row by `url_key`.
    pub actions: Value,
    pub mass_actions: Value,
    /// Per-row label overrides `(column index, function)`.
    pub formatters: Vec<(&'static str, fn(&RowMap) -> Value)>,
}

pub struct GridRequest {
    pub filters: Map<String, Value>,
    pub sort_column: Option<String>,
    pub sort_order: Option<String>,
    pub page: i64,
    pub per_page: i64,
}

pub fn parse_request(body: &Value) -> GridRequest {
    let filters = body["filters"]
        .as_object()
        .cloned()
        .unwrap_or_else(Map::new);
    let sort = &body["sort"];
    let pagination = &body["pagination"];
    GridRequest {
        filters,
        sort_column: sort["column"].as_str().map(str::to_string),
        sort_order: sort["order"].as_str().map(str::to_string),
        page: pagination["page"].as_i64().unwrap_or(1).max(1),
        per_page: pagination["per_page"].as_i64().unwrap_or(10).clamp(1, 200),
    }
}

impl DataGrid {
    fn column(&self, index: &str) -> Option<&Column> {
        self.columns.iter().find(|column| column.index == index)
    }

    fn from_sql(&self) -> String {
        let mut from = format!("`{}` `{}`", self.table, self.alias);
        for join in &self.joins {
            from.push(' ');
            from.push_str(join);
        }
        from
    }

    /// Builds WHERE with filters. Key `all` = global OR LIKE over searchable
    /// text columns; anything else delegates to the matching column.
    fn build_where(&self, request: &GridRequest) -> ApiResult<(String, Vec<Value>)> {
        let mut clauses: Vec<String> = self
            .where_clauses
            .iter()
            .map(|(sql, _)| sql.clone())
            .collect();
        let mut binds: Vec<Value> = self
            .where_clauses
            .iter()
            .flat_map(|(_, values)| values.clone())
            .collect();

        for (key, value) in &request.filters {
            if key == "all" {
                let term = value.as_str().unwrap_or_default().trim().to_string();
                if term.is_empty() {
                    continue;
                }
                let mut parts = Vec::new();
                for column in self.columns.iter().filter(|column| column.searchable) {
                    parts.push(format!("CAST({} AS CHAR) LIKE ?", column.sql()));
                    binds.push(Value::String(format!("%{term}%")));
                }
                if !parts.is_empty() {
                    clauses.push(format!("({})", parts.join(" OR ")));
                }
                continue;
            }

            let Some(column) = self.column(key) else {
                continue;
            };
            if !column.filterable {
                continue;
            }
            let sql = column.sql();

            match column.kind {
                "boolean" => {
                    if let Some(flag) = value.as_bool().or_else(|| value.as_i64().map(|n| n != 0)) {
                        clauses.push(format!("{sql} = ?"));
                        binds.push(Value::from(flag as i64));
                    }
                }
                "integer" | "decimal" => {
                    if let Some(number) = value.as_f64().or_else(|| value.as_str().and_then(|s| s.parse().ok())) {
                        clauses.push(format!("{sql} = ?"));
                        binds.push(Value::from(number));
                    }
                }
                "date" | "datetime" => {
                    // `value` may be `[min, max]` (range) or a scalar.
                    match value {
                        Value::Array(range) if range.len() == 2 => {
                            clauses.push(format!("{sql} >= ? AND {sql} <= ?"));
                            binds.push(Value::String(range[0].as_str().unwrap_or_default().to_string()));
                            binds.push(Value::String(range[1].as_str().unwrap_or_default().to_string()));
                        }
                        scalar => {
                            clauses.push(format!("{sql} LIKE ?"));
                            binds.push(Value::String(format!("%{}%", scalar.as_str().unwrap_or_default())));
                        }
                    }
                }
                _ => match value {
                    Value::Array(list) => {
                        let placeholders = vec!["?"; list.len()].join(",");
                        clauses.push(format!("{sql} IN ({placeholders})"));
                        for entry in list {
                            binds.push(entry.clone());
                        }
                    }
                    scalar if column.options.is_some() => {
                        clauses.push(format!("{sql} = ?"));
                        binds.push(scalar.clone());
                    }
                    scalar => {
                        clauses.push(format!("{sql} LIKE ?"));
                        binds.push(Value::String(format!("%{}%", scalar.as_str().unwrap_or_default())));
                    }
                },
            }
        }

        let where_sql = if clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", clauses.join(" AND "))
        };
        Ok((where_sql, binds))
    }

    /// Runs the grid and returns the Bagisto datagrid contract.
    pub async fn process(&self, request: &GridRequest) -> ApiResult<Value> {
        let _ = valid_ident(self.table)?;
        let (where_sql, binds) = self.build_where(request)?;

        // Sorting (default: primary desc, only sortable columns).
        let mut sort_sql = format!("ORDER BY {} DESC", self.primary);
        if let Some(column_index) = &request.sort_column {
            if let Some(column) = self.column(column_index) {
                if column.sortable {
                    let direction = match request.sort_order.as_deref() {
                        Some("asc") => "ASC",
                        _ => "DESC",
                    };
                    sort_sql = format!("ORDER BY {} {}", column.sql(), direction);
                }
            }
        }

        let total = db::scalar_i64(
            &format!("SELECT COUNT(*) FROM {} {}", self.from_sql(), where_sql),
            &binds,
        )
        .await?;
        let offset = (request.page - 1) * request.per_page;

        let columns_sql = self
            .columns
            .iter()
            .map(|column| format!("{} AS `{}`", column.sql(), column.index))
            .collect::<Vec<_>>()
            .join(", ");
        let rows = db::select(
            &format!(
                "SELECT {}, {} AS `__primary` FROM {} {} {} LIMIT ? OFFSET ?",
                columns_sql,
                self.primary,
                self.from_sql(),
                where_sql,
                sort_sql
            ),
            &{
                let mut all = binds.clone();
                all.push(Value::from(request.per_page));
                all.push(Value::from(offset));
                all
            },
        )
        .await?;

        // Apply formatters + attach actions per row.
        let mut records: Vec<Value> = Vec::new();
        for mut row in rows {
            for (index, formatter) in &self.formatters {
                let formatted = formatter(&row);
                row.insert((*index).to_string(), formatted);
            }
            records.push(Value::Object(row));
        }

        let last_page = ((total as f64) / (request.per_page as f64)).ceil() as i64;
        let columns_json: Vec<Value> = self
            .columns
            .iter()
            .map(|column| {
                json!({
                    "index": column.index,
                    "label": column.label,
                    "type": column.kind,
                    "searchable": column.searchable,
                    "filterable": column.filterable,
                    "sortable": column.sortable,
                    "filterable_options": column.options.clone().unwrap_or(Value::Null),
                })
            })
            .collect();

        Ok(json!({
            "columns": columns_json,
            "actions": self.actions,
            "mass_actions": self.mass_actions,
            "records": records,
            "meta": {
                "primary_column": self.primary,
                "total": total,
                "from": if total == 0 { 0 } else { offset + 1 },
                "to": (offset + records.len() as i64),
                "current_page": request.page,
                "last_page": last_page,
                "per_page": request.per_page,
                "per_page_options": [10, 20, 30, 40, 50, 100],
            },
        }))
    }
}

/// Builds the column list from a compact spec
/// `[(index, label, kind, searchable, filterable, sortable)]`.
pub fn columns(spec: &[(&'static str, &'static str, &'static str, bool, bool, bool)]) -> Vec<Column> {
    spec.iter()
        .map(|(index, label, kind, searchable, filterable, sortable)| Column {
            index,
            label,
            kind,
            searchable: *searchable,
            filterable: *filterable,
            sortable: *sortable,
            source: None,
            options: None,
        })
        .collect()
}
