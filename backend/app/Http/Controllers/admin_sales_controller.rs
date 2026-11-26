//! Admin sales: orders grid + detail, invoice, ship, cancel, refund hooks.
//! Mirrors `Webkul\Admin\Http\Controllers\Sales\*` (phase 1: orders + invoices +
//! cancel; shipments/refunds fleshed out next).

use std::collections::HashMap;

use larust_support::axum::extract::{Path, Query};
use larust_support::axum::http::{HeaderMap, StatusCode};
use larust_support::axum::response::{Html, IntoResponse, Response};
use larust_support::axum::Json;
use larust_support::serde_json::{json, Map, Value};

use crate::support::auth;
use crate::support::core;
use crate::support::datagrid;
use crate::support::db;
use crate::support::response::{csv_response, message, resource};
use crate::support::sales;
use crate::support::stripe;

pub struct AdminSalesController;

/// Etiqueta capitalizada para las pills de estado del grid.
fn status_label(row: &db::RowMap) -> Value {
    // Orders use `status`; invoices use `state`.
    let status = row
        .get("status")
        .or_else(|| row.get("state"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut chars = status.chars();
    let label = match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    };
    json!(label)
}

/// `partially_refunded` -> `Partially Refunded` para el grid.
fn payment_status_label(row: &db::RowMap) -> Value {
    let raw = row
        .get("payment_status")
        .and_then(Value::as_str)
        .unwrap_or("");
    if raw.is_empty() {
        return json!("Pending");
    }
    let label = raw
        .split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(capitalize)
        .collect::<Vec<_>>()
        .join(" ");
    json!(label)
}

/// Importe con la moneda base (`€88.49`).
fn money_label(row: &db::RowMap) -> Value {
    let amount = row
        .get("base_grand_total")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    json!(format!("€{amount:.2}"))
}

/// `2026-11-14 09:14:11` -> `14 Nov 2026`.
fn date_label(row: &db::RowMap) -> Value {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let raw = row.get("created_at").and_then(Value::as_str).unwrap_or("");
    let date = raw.split_whitespace().next().unwrap_or(raw);
    let parts: Vec<&str> = date.split('-').collect();
    let label = if parts.len() == 3 {
        let month = parts[1].parse::<usize>().unwrap_or(0);
        let day = parts[2].trim_start_matches('0');
        format!(
            "{day} {} {}",
            MONTHS.get(month.saturating_sub(1)).copied().unwrap_or(""),
            parts[0]
        )
        .trim()
        .to_string()
    } else {
        raw.to_string()
    };
    json!(label)
}

/// Hoja de estilos del documento imprimible. Se mantiene fuera de `format!`
/// para no duplicar cada llave del CSS.
const INVOICE_STYLES: &str = r#"
  :root {
    --ink: #0f172a;
    --ink-soft: #334155;
    --muted: #64748b;
    --line: #e2e8f0;
    --accent: #4f46e5;
    --accent-soft: #eef2ff;
    --paper: #ffffff;
  }
  * { box-sizing: border-box; }
  html, body {
    margin: 0;
    padding: 0;
    background: #eef1f6;
    color: var(--ink);
    font-family: 'Inter', 'Segoe UI', Helvetica, Arial, sans-serif;
    -webkit-print-color-adjust: exact;
    print-color-adjust: exact;
  }
  .page {
    width: min(820px, calc(100% - 32px));
    margin: 28px auto;
    background: var(--paper);
    border-radius: 20px;
    overflow: hidden;
    box-shadow: 0 28px 70px rgba(15, 23, 42, .18);
  }
  .toolbar {
    display: flex;
    justify-content: flex-end;
    padding: 14px 22px;
    background: #f8fafc;
    border-bottom: 1px solid var(--line);
  }
  .toolbar button {
    appearance: none;
    border: 0;
    border-radius: 10px;
    background: linear-gradient(135deg, #4f46e5, #7c3aed);
    color: #fff;
    padding: 10px 18px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: .04em;
    cursor: pointer;
    box-shadow: 0 8px 20px rgba(79, 70, 229, .28);
  }
  .toolbar button:hover { filter: brightness(1.06); }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 28px;
    padding: 36px 42px 32px;
    background: linear-gradient(135deg, #0f172a 0%, #1e1b4b 58%, #312e81 100%);
    color: #fff;
  }
  .brand { display: flex; align-items: center; gap: 16px; min-width: 0; }
  .brand-plate {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 64px;
    min-height: 64px;
    padding: 10px 14px;
    border-radius: 16px;
    background: rgba(255, 255, 255, .97);
    box-shadow: inset 0 0 0 1px rgba(15, 23, 42, .06);
  }
  .brand-logo { display: block; max-height: 44px; max-width: 170px; object-fit: contain; }
  .brand-mark {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 64px;
    height: 64px;
    border-radius: 18px;
    border: 1px solid rgba(255, 255, 255, .3);
    background: rgba(255, 255, 255, .14);
    font-size: 22px;
    font-weight: 700;
    letter-spacing: .08em;
  }
  .company-name { font-size: 18px; font-weight: 600; letter-spacing: .01em; }
  .company-meta { display: block; margin-top: 5px; font-size: 11.5px; color: #c7d2fe; }
  .title-block { text-align: right; flex-shrink: 0; }
  .title-block h1 {
    margin: 0;
    font-size: 32px;
    line-height: 1;
    font-weight: 600;
    letter-spacing: .16em;
    text-transform: uppercase;
  }
  .title-block .number {
    margin-top: 10px;
    font-size: 12.5px;
    letter-spacing: .12em;
    color: #c7d2fe;
  }
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    margin-top: 14px;
    padding: 7px 13px;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: .12em;
    text-transform: uppercase;
  }
  .badge::before {
    content: '';
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }
  .status-paid { background: rgba(16, 185, 129, .18); color: #a7f3d0; }
  .status-pending { background: rgba(245, 158, 11, .18); color: #fde68a; }
  .status-canceled { background: rgba(239, 68, 68, .2); color: #fecaca; }
  .status-default { background: rgba(148, 163, 184, .22); color: #e2e8f0; }
  .meta {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    border-bottom: 1px solid var(--line);
  }
  .meta > div { padding: 18px 24px; border-right: 1px solid var(--line); }
  .meta > div:last-child { border-right: 0; }
  .meta span {
    display: block;
    margin-bottom: 6px;
    font-size: 9.5px;
    font-weight: 700;
    letter-spacing: .14em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .meta strong { font-size: 13px; font-weight: 600; }
  .content { padding: 30px 42px 36px; }
  .parties {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 22px;
    margin-bottom: 30px;
  }
  .party {
    min-height: 138px;
    padding: 18px 20px;
    border: 1px solid var(--line);
    border-radius: 14px;
    background: #f8fafc;
  }
  .party h3 {
    margin: 0 0 12px;
    font-size: 9.5px;
    font-weight: 700;
    letter-spacing: .14em;
    text-transform: uppercase;
    color: var(--accent);
  }
  .party p { margin: 0; font-size: 12.5px; line-height: 1.68; color: var(--ink-soft); }
  .party strong { color: var(--ink); }
  .muted { color: var(--muted); }
  table.items { width: 100%; border-collapse: collapse; }
  table.items thead th {
    padding: 0 10px 11px;
    border-bottom: 2px solid var(--ink);
    font-size: 9.5px;
    font-weight: 700;
    letter-spacing: .12em;
    text-transform: uppercase;
    color: var(--muted);
    text-align: left;
  }
  table.items thead th.num { text-align: right; }
  table.items tbody td {
    padding: 14px 10px;
    border-bottom: 1px solid var(--line);
    font-size: 12.5px;
    vertical-align: top;
  }
  table.items tbody tr:nth-child(even) { background: #fbfcfe; }
  .cell-index { color: #94a3b8; font-variant-numeric: tabular-nums; }
  .item-name { font-weight: 600; color: var(--ink); }
  .item-sku { margin-top: 3px; font-size: 10.5px; letter-spacing: .04em; color: var(--muted); }
  .item-note { margin-top: 4px; font-size: 10.5px; color: #b45309; }
  .num { text-align: right; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .summary { display: flex; justify-content: flex-end; margin-top: 22px; }
  .summary-box { width: 290px; }
  .summary-row {
    display: flex;
    justify-content: space-between;
    gap: 18px;
    padding: 7px 0;
    font-size: 12.5px;
    color: #475569;
  }
  .summary-row span:last-child { font-variant-numeric: tabular-nums; }
  .summary-total {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 18px;
    margin-top: 10px;
    padding: 15px 17px;
    border-radius: 13px;
    background: linear-gradient(135deg, #1e1b4b, #312e81);
    color: #fff;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: .1em;
    text-transform: uppercase;
  }
  .summary-total span:last-child {
    font-size: 17px;
    font-weight: 700;
    letter-spacing: .01em;
    text-transform: none;
    font-variant-numeric: tabular-nums;
  }
  .notes {
    margin-top: 30px;
    padding: 18px 20px;
    border-left: 3px solid var(--accent);
    border-radius: 0 12px 12px 0;
    background: var(--accent-soft);
  }
  .notes h4 {
    margin: 0 0 7px;
    font-size: 9.5px;
    font-weight: 700;
    letter-spacing: .14em;
    text-transform: uppercase;
    color: var(--accent);
  }
  .notes p { margin: 0; font-size: 12px; line-height: 1.65; color: #3730a3; }
  .footer {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: 20px;
    padding: 18px 42px 26px;
    border-top: 1px solid var(--line);
    background: #f8fafc;
    font-size: 10.5px;
    color: var(--muted);
  }
  .footer .thanks { font-size: 12px; font-weight: 600; color: var(--ink-soft); }
  @media (max-width: 640px) {
    .header { flex-direction: column; padding: 28px 24px; }
    .title-block { text-align: left; }
    .meta { grid-template-columns: 1fr 1fr; }
    .meta > div:nth-child(2) { border-right: 0; }
    .parties { grid-template-columns: 1fr; }
    .content { padding: 24px; }
    .footer { flex-direction: column; align-items: flex-start; padding: 16px 24px 22px; }
  }
  @media print {
    html, body { background: #fff; }
    .toolbar { display: none !important; }
    .page {
      width: 100%;
      margin: 0;
      border-radius: 0;
      box-shadow: none;
    }
    .header, .summary-total, .party, .notes, table.items tbody tr:nth-child(even) {
      -webkit-print-color-adjust: exact;
      print-color-adjust: exact;
    }
    table.items tr { break-inside: avoid; }
    @page { size: A4; margin: 10mm; }
  }
"#;

/// Escapa texto dinámico antes de incrustarlo en el HTML de la factura.
fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Iniciales de la marca para el monograma cuando no hay logotipo.
fn brand_initials(name: &str) -> String {
    name.split_whitespace()
        .take(2)
        .filter_map(|word| word.chars().next())
        .collect::<String>()
        .to_uppercase()
}

/// `1234.5` -> `1,234.50`.
fn grouped_amount(amount: f64) -> String {
    let formatted = format!("{:.2}", amount.abs());
    let (integer, decimals) = formatted.split_once('.').unwrap_or((formatted.as_str(), "00"));
    let mut reversed = String::new();
    for (index, ch) in integer.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            reversed.push(',');
        }
        reversed.push(ch);
    }
    let grouped: String = reversed.chars().rev().collect();
    format!("{}{grouped}.{decimals}", if amount < 0.0 { "-" } else { "" })
}

/// Cantidad sin decimales innecesarios (`2`, `2.5`).
fn quantity_label(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value}")
    }
}

/// Importe con el símbolo de la moneda (`€1,234.50`).
fn format_money(amount: f64, currency: &str) -> String {
    let symbol = match currency {
        "EUR" => "€",
        "USD" => "$",
        "GBP" => "£",
        "JPY" => "¥",
        _ => "",
    };
    let number = grouped_amount(amount);
    if symbol.is_empty() {
        format!("{number} {currency}")
    } else {
        format!("{symbol}{number}")
    }
}

/// `2026-11-10 02:36:39` -> `10 Nov 2026`.
fn format_invoice_date(raw: &str) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let date = raw.split_whitespace().next().unwrap_or(raw);
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() == 3 {
        if let Ok(month) = parts[1].parse::<usize>() {
            if (1..=12).contains(&month) {
                return format!(
                    "{} {} {}",
                    parts[2].trim_start_matches('0'),
                    MONTHS[month - 1],
                    parts[0]
                );
            }
        }
    }
    raw.to_string()
}

/// Bloque de dirección (facturación o envío) listo para incrustar.
fn address_html(address: Option<&db::RowMap>) -> String {
    let Some(address) = address else {
        return "<span class=\"muted\">Not provided</span>".to_string();
    };
    let mut lines: Vec<String> = Vec::new();
    let name = format!(
        "{} {}",
        address["first_name"].as_str().unwrap_or(""),
        address["last_name"].as_str().unwrap_or("")
    )
    .trim()
    .to_string();
    if !name.is_empty() {
        lines.push(format!("<strong>{}</strong>", escape_html(&name)));
    }
    for key in ["company_name", "address1", "address2"] {
        if let Some(value) = address[key].as_str().map(str::trim).filter(|v| !v.is_empty()) {
            lines.push(escape_html(value));
        }
    }
    let city_line = ["postcode", "city", "state"]
        .iter()
        .filter_map(|key| address[*key].as_str().map(str::trim).filter(|v| !v.is_empty()))
        .collect::<Vec<_>>()
        .join(" ");
    if !city_line.is_empty() {
        lines.push(escape_html(&city_line));
    }
    for key in ["country", "phone", "email"] {
        if let Some(value) = address[key].as_str().map(str::trim).filter(|v| !v.is_empty()) {
            lines.push(escape_html(value));
        }
    }
    if let Some(vat) = address["vat_id"].as_str().map(str::trim).filter(|v| !v.is_empty()) {
        lines.push(format!("<span class=\"muted\">VAT</span> {}", escape_html(vat)));
    }
    if lines.is_empty() {
        "<span class=\"muted\">Not provided</span>".to_string()
    } else {
        lines.join("<br/>")
    }
}

/// Lee el logotipo configurado (`general.design.admin_logo.logo_image`) y lo
/// incrusta como `data:` URI. El documento se abre como blob, así que un
/// `/storage/...` relativo no cargaría y además evitamos depender de sesión.
async fn logo_data_uri(channel_code: &str, fallback: &str) -> Option<String> {
    use base64::Engine as _;

    let configured = core::get_config_data(
        "general.design.admin_logo.logo_image",
        Some(channel_code),
        None,
    )
    .await
    .ok()
    .flatten()
    .and_then(|value| {
        value
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    });
    let path = configured.as_deref().unwrap_or(fallback).trim();
    if path.is_empty() {
        return None;
    }
    if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("data:") {
        return Some(path.to_string());
    }
    let relative = path.trim_start_matches('/');
    let relative = relative.strip_prefix("storage/").unwrap_or(relative);
    let full = crate::paths().join("public/storage").join(relative);
    let bytes = std::fs::read(&full).ok()?;
    let mime = mime_guess::from_path(&full)
        .first_or_octet_stream()
        .to_string();
    Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

pub fn orders_grid() -> datagrid::DataGrid {
    datagrid::DataGrid {
        table: "orders",
        alias: "o",
        primary: "o.id",
        columns: vec![
            datagrid::Column { index: "increment_id", label: "Order ID", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("o.increment_id"), options: None },
            datagrid::Column { index: "status", label: "Status", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("o.status"), options: Some(json!([
                {"label": "Pending", "value": "pending"}, {"label": "Processing", "value": "processing"},
                {"label": "Completed", "value": "completed"}, {"label": "Canceled", "value": "canceled"},
                {"label": "Closed", "value": "closed"},
            ])) },
            datagrid::Column { index: "payment_method", label: "Payment", kind: "string", searchable: false, filterable: false, sortable: false, source: Some("op.method_title"), options: None },
            datagrid::Column { index: "payment_status", label: "Payment status", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("o.payment_status"), options: Some(json!([
                {"label": "Pending", "value": "pending"}, {"label": "Authorized", "value": "authorized"},
                {"label": "Paid", "value": "paid"}, {"label": "Partially refunded", "value": "partially_refunded"},
                {"label": "Refunded", "value": "refunded"}, {"label": "Failed", "value": "failed"},
                {"label": "Disputed", "value": "disputed"},
            ])) },
            datagrid::Column { index: "base_grand_total", label: "Grand Total", kind: "decimal", searchable: false, filterable: true, sortable: true, source: Some("o.base_grand_total"), options: None },
            datagrid::Column { index: "customer_email", label: "Customer", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("o.customer_email"), options: None },
            datagrid::Column { index: "created_at", label: "Date", kind: "date", searchable: false, filterable: true, sortable: true, source: Some("o.created_at"), options: None },
        ],
        joins: vec![
            "LEFT JOIN `order_payment` `op` ON `op`.`id` = \
               (SELECT MAX(`pay`.`id`) FROM `order_payment` `pay` WHERE `pay`.`order_id` = `o`.`id`)"
                .into(),
        ],
        where_clauses: Vec::new(),
        actions: json!([
            {"title": "View", "method": "GET", "url": "/admin/sales/orders/view/{id}"},
        ]),
        mass_actions: Value::Null,
        formatters: vec![
            ("status", status_label as fn(&db::RowMap) -> Value),
            ("payment_status", payment_status_label as fn(&db::RowMap) -> Value),
            ("base_grand_total", money_label as fn(&db::RowMap) -> Value),
            ("created_at", date_label as fn(&db::RowMap) -> Value),
        ],
    }
}

fn invoices_grid() -> datagrid::DataGrid {
    datagrid::DataGrid {
        table: "invoices",
        alias: "inv",
        primary: "inv.id",
        columns: vec![
            datagrid::Column { index: "increment_id", label: "Invoice", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("inv.increment_id"), options: None },
            datagrid::Column { index: "state", label: "State", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("inv.state"), options: Some(json!([
                {"label": "Paid", "value": "paid"}, {"label": "Pending", "value": "pending"},
                {"label": "Pending payment", "value": "pending_payment"},
                {"label": "Refunded", "value": "refunded"},
            ])) },
            datagrid::Column { index: "payment_status", label: "Payment status", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("o.payment_status"), options: Some(json!([
                {"label": "Pending", "value": "pending"}, {"label": "Authorized", "value": "authorized"},
                {"label": "Paid", "value": "paid"}, {"label": "Partially refunded", "value": "partially_refunded"},
                {"label": "Refunded", "value": "refunded"}, {"label": "Failed", "value": "failed"},
                {"label": "Disputed", "value": "disputed"},
            ])) },
            datagrid::Column { index: "transaction_id", label: "Transaction ID", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("inv.transaction_id"), options: None },
            datagrid::Column { index: "base_grand_total", label: "Total", kind: "decimal", searchable: false, filterable: true, sortable: true, source: Some("inv.base_grand_total"), options: None },
            datagrid::Column { index: "customer_email", label: "Customer", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("o.customer_email"), options: None },
            datagrid::Column { index: "order_id", label: "Order", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("o.increment_id"), options: None },
            datagrid::Column { index: "created_at", label: "Date", kind: "date", searchable: false, filterable: true, sortable: true, source: Some("inv.created_at"), options: None },
        ],
        joins: vec![
            "LEFT JOIN `orders` `o` ON `o`.`id` = `inv`.`order_id`".into(),
        ],
        where_clauses: Vec::new(),
        actions: json!([
            {"title": "Print", "method": "GET", "url": "/api/admin/sales/invoices/print/{id}"},
        ]),
        mass_actions: Value::Null,
        formatters: vec![
            ("state", status_label as fn(&db::RowMap) -> Value),
            ("payment_status", payment_status_label as fn(&db::RowMap) -> Value),
            ("base_grand_total", money_label as fn(&db::RowMap) -> Value),
            ("created_at", date_label as fn(&db::RowMap) -> Value),
        ],
    }
}

fn shipments_grid() -> datagrid::DataGrid {
    datagrid::DataGrid {
        table: "shipments",
        alias: "sh",
        primary: "sh.id",
        columns: vec![
            datagrid::Column { index: "carrier_title", label: "Carrier", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("sh.carrier_title"), options: None },
            datagrid::Column { index: "track_number", label: "Track number", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("sh.track_number"), options: None },
            datagrid::Column { index: "inventory_source_name", label: "Source", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("sh.inventory_source_name"), options: None },
            datagrid::Column { index: "customer_email", label: "Customer", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("o.customer_email"), options: None },
            datagrid::Column { index: "order_id", label: "Order", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("o.increment_id"), options: None },
            datagrid::Column { index: "total_qty", label: "Qty", kind: "integer", searchable: false, filterable: true, sortable: true, source: Some("sh.total_qty"), options: None },
            datagrid::Column { index: "created_at", label: "Date", kind: "date", searchable: false, filterable: true, sortable: true, source: Some("sh.created_at"), options: None },
        ],
        joins: vec![
            "LEFT JOIN `orders` `o` ON `o`.`id` = `sh`.`order_id`".into(),
        ],
        where_clauses: Vec::new(),
        actions: Value::Null,
        mass_actions: Value::Null,
        formatters: vec![("created_at", date_label as fn(&db::RowMap) -> Value)],
    }
}

fn refunds_grid() -> datagrid::DataGrid {
    datagrid::DataGrid {
        table: "refunds",
        alias: "rf",
        primary: "rf.id",
        columns: vec![
            datagrid::Column { index: "increment_id", label: "Refund", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("rf.increment_id"), options: None },
            datagrid::Column { index: "state", label: "State", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("rf.state"), options: Some(json!([
                {"label": "Pending", "value": "pending"}, {"label": "Refunded", "value": "refunded"},
                {"label": "Failed", "value": "failed"},
            ])) },
            datagrid::Column { index: "transaction_id", label: "Transaction ID", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("rf.transaction_id"), options: None },
            datagrid::Column { index: "base_grand_total", label: "Total", kind: "decimal", searchable: false, filterable: true, sortable: true, source: Some("rf.base_grand_total"), options: None },
            datagrid::Column { index: "customer_email", label: "Customer", kind: "string", searchable: true, filterable: false, sortable: true, source: Some("o.customer_email"), options: None },
            datagrid::Column { index: "order_id", label: "Order", kind: "string", searchable: false, filterable: true, sortable: true, source: Some("o.increment_id"), options: None },
            datagrid::Column { index: "created_at", label: "Date", kind: "date", searchable: false, filterable: true, sortable: true, source: Some("rf.created_at"), options: None },
        ],
        joins: vec![
            "LEFT JOIN `orders` `o` ON `o`.`id` = `rf`.`order_id`".into(),
        ],
        where_clauses: Vec::new(),
        actions: Value::Null,
        mass_actions: Value::Null,
        formatters: vec![
            ("state", status_label as fn(&db::RowMap) -> Value),
            ("base_grand_total", money_label as fn(&db::RowMap) -> Value),
            ("created_at", date_label as fn(&db::RowMap) -> Value),
        ],
    }
}

impl AdminSalesController {
    /// `POST /api/admin/sales/orders` - datagrid. `?format=csv` exporta.
    pub async fn orders_grid(
        admin: auth::Admin,
        Query(query): Query<HashMap<String, String>>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let grid = orders_grid();
        if query.get("format").map(String::as_str) == Some("csv") {
            let request = datagrid::parse_request(&body);
            return match grid.process_csv(&request).await {
                Ok(csv) => csv_response(&csv, "orders.csv"),
                Err(error) => error.into(),
            };
        }
        let request = datagrid::parse_request(&body);
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/sales/invoices` - datagrid. `?format=csv` exporta.
    pub async fn invoices_grid(
        admin: auth::Admin,
        Query(query): Query<HashMap<String, String>>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let request = datagrid::parse_request(&body);
        let grid = invoices_grid();
        if query.get("format").map(String::as_str) == Some("csv") {
            return match grid.process_csv(&request).await {
                Ok(csv) => csv_response(&csv, "invoices.csv"),
                Err(error) => error.into(),
            };
        }
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/sales/shipments` - datagrid. `?format=csv` exporta.
    pub async fn shipments_grid(
        admin: auth::Admin,
        Query(query): Query<HashMap<String, String>>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let request = datagrid::parse_request(&body);
        let grid = shipments_grid();
        if query.get("format").map(String::as_str) == Some("csv") {
            return match grid.process_csv(&request).await {
                Ok(csv) => csv_response(&csv, "shipments.csv"),
                Err(error) => error.into(),
            };
        }
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/sales/refunds` - datagrid. `?format=csv` exporta.
    pub async fn refunds_grid(
        admin: auth::Admin,
        Query(query): Query<HashMap<String, String>>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let request = datagrid::parse_request(&body);
        let grid = refunds_grid();
        if query.get("format").map(String::as_str) == Some("csv") {
            return match grid.process_csv(&request).await {
                Ok(csv) => csv_response(&csv, "refunds.csv"),
                Err(error) => error.into(),
            };
        }
        match grid.process(&request).await {
            Ok(payload) => resource(payload, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/sales/invoices/print/{id}` - HTML imprimible de la
    /// factura con cabecera de marca (logo), direcciones, items y totales.
    pub async fn invoice_print(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let invoice = db::find_by_id("invoices", id).await.ok().flatten();
        let Some(invoice) = invoice else {
            return message(StatusCode::NOT_FOUND, "Invoice not found.");
        };
        let order_id = invoice["order_id"].as_i64().unwrap_or(0);
        let order = db::find_by_id("orders", order_id)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(db::empty_row);

        let currency = invoice["base_currency_code"]
            .as_str()
            .or_else(|| order["base_currency_code"].as_str())
            .unwrap_or("EUR")
            .to_string();

        let items = db::select(
            "SELECT `name`, `sku`, `qty`, `price`, `total`, `discount_amount`, `tax_amount` \
             FROM `invoice_items` WHERE `invoice_id` = ? ORDER BY `id`",
            &[Value::from(id)],
        )
        .await
        .unwrap_or_default();

        let mut rows = String::new();
        for (index, item) in items.iter().enumerate() {
            let name = escape_html(item["name"].as_str().unwrap_or(""));
            let sku = escape_html(item["sku"].as_str().unwrap_or(""));
            let qty = item["qty"].as_f64().unwrap_or(0.0);
            let price = item["price"].as_f64().unwrap_or(0.0);
            let total = item["total"].as_f64().unwrap_or(0.0);
            let discount = item["discount_amount"].as_f64().unwrap_or(0.0);
            let tax = item["tax_amount"].as_f64().unwrap_or(0.0);
            let mut note = String::new();
            if discount > 0.0 {
                note.push_str(&format!(
                    "<div class=\"item-note\">Discount −{}</div>",
                    format_money(discount, &currency)
                ));
            }
            if tax > 0.0 {
                note.push_str(&format!(
                    "<div class=\"item-note\">Tax {}</div>",
                    format_money(tax, &currency)
                ));
            }
            rows.push_str(&format!(
                "<tr>\
                   <td class=\"cell-index\">{position:02}</td>\
                   <td><div class=\"item-name\">{name}</div><div class=\"item-sku\">{sku}</div>{note}</td>\
                   <td class=\"num\">{qty}</td>\
                   <td class=\"num\">{price}</td>\
                   <td class=\"num\">{total}</td>\
                 </tr>",
                position = index + 1,
                qty = quantity_label(qty),
                price = format_money(price, &currency),
                total = format_money(total, &currency),
            ));
        }

        let state_raw = invoice["state"].as_str().unwrap_or("").trim();
        let state_label = capitalize(state_raw);
        let status_class = match state_raw.to_ascii_lowercase().as_str() {
            "paid" | "complete" | "completed" => "status-paid",
            "pending" | "pending_payment" | "processing" => "status-pending",
            "canceled" | "cancelled" | "refunded" => "status-canceled",
            _ => "status-default",
        };

        let billing = db::select_one(
            "SELECT * FROM `addresses` WHERE `order_id` = ? AND `address_type` = 'order_billing' ORDER BY `id` DESC LIMIT 1",
            &[Value::from(order_id)],
        )
        .await
        .ok()
        .flatten();
        let shipping = db::select_one(
            "SELECT * FROM `addresses` WHERE `order_id` = ? AND `address_type` = 'order_shipping' ORDER BY `id` DESC LIMIT 1",
            &[Value::from(order_id)],
        )
        .await
        .ok()
        .flatten();

        let payment = db::select_one(
            "SELECT `method_title`, `method` FROM `order_payment` WHERE `order_id` = ? ORDER BY `id` DESC LIMIT 1",
            &[Value::from(order_id)],
        )
        .await
        .ok()
        .flatten()
        .and_then(|row| {
            row["method_title"]
                .as_str()
                .or_else(|| row["method"].as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .map(|value| escape_html(&value))
        .unwrap_or_else(|| "—".to_string());

        let channel_id = order["channel_id"].as_i64().unwrap_or(1);
        let channel_code = order["channel_name"]
            .as_str()
            .unwrap_or("default")
            .to_string();
        let channel = db::find_by_id("channels", channel_id)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(db::empty_row);
        let company_name = db::select_one(
            "SELECT `name` FROM `channel_translations` WHERE `channel_id` = ? ORDER BY `id` LIMIT 1",
            &[Value::from(channel_id)],
        )
        .await
        .ok()
        .flatten()
        .and_then(|row| row["name"].as_str().map(str::to_string))
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "EzCommerce".to_string());
        let company_name_html = escape_html(&company_name);
        let logo = logo_data_uri(&channel_code, channel["logo"].as_str().unwrap_or("")).await;
        let brand = match logo {
            Some(src) => format!(
                "<div class=\"brand-plate\"><img class=\"brand-logo\" src=\"{}\" alt=\"{}\" /></div>",
                escape_html(&src),
                company_name_html
            ),
            None => format!(
                "<div class=\"brand-mark\">{}</div>",
                escape_html(&brand_initials(&company_name))
            ),
        };
        let company_meta = channel["hostname"]
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|website| {
                format!(
                    "<span class=\"company-meta\">{}</span>",
                    escape_html(website)
                )
            })
            .unwrap_or_default();

        let sub_total = invoice["base_sub_total"].as_f64().unwrap_or(0.0);
        let shipping_amount = invoice["base_shipping_amount"].as_f64().unwrap_or(0.0);
        let discount = invoice["base_discount_amount"].as_f64().unwrap_or(0.0);
        let tax = invoice["base_tax_amount"].as_f64().unwrap_or(0.0);
        let grand_total = invoice["base_grand_total"].as_f64().unwrap_or(0.0);
        let total_qty = invoice["total_qty"].as_f64().unwrap_or(0.0);

        let mut summary = format!(
            "<div class=\"summary-row\"><span>Subtotal</span><span>{}</span></div>",
            format_money(sub_total, &currency)
        );
        if shipping_amount != 0.0 {
            summary.push_str(&format!(
                "<div class=\"summary-row\"><span>Shipping</span><span>{}</span></div>",
                format_money(shipping_amount, &currency)
            ));
        }
        if discount != 0.0 {
            summary.push_str(&format!(
                "<div class=\"summary-row\"><span>Discount</span><span>−{}</span></div>",
                format_money(discount.abs(), &currency)
            ));
        }
        if tax != 0.0 {
            summary.push_str(&format!(
                "<div class=\"summary-row\"><span>Tax</span><span>{}</span></div>",
                format_money(tax, &currency)
            ));
        }
        summary.push_str(&format!(
            "<div class=\"summary-total\"><span>Grand total</span><span>{}</span></div>",
            format_money(grand_total, &currency)
        ));

        let footer_config = core::get_config_data(
            "general.content.footer.copyright_content",
            Some(&channel_code),
            None,
        )
        .await
        .ok()
        .flatten()
        .and_then(|value| {
            value
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        });
        let footer_text = footer_config
            .map(|value| escape_html(&value))
            .unwrap_or_else(|| {
                format!(
                    "© {} {}. All rights reserved.",
                    chrono::Utc::now().format("%Y"),
                    company_name_html
                )
            });

        let increment = escape_html(invoice["increment_id"].as_str().unwrap_or(""));
        let order_increment = order["increment_id"]
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(escape_html)
            .unwrap_or_else(|| "—".to_string());
        let invoice_date = escape_html(&format_invoice_date(
            invoice["created_at"].as_str().unwrap_or(""),
        ));

        let html = format!(
            r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>Invoice {increment}</title>
<style>{styles}</style>
</head>
<body>
<div class="page">
  <div class="toolbar">
    <button type="button" onclick="window.print()">Print invoice</button>
  </div>
  <header class="header">
    <div class="brand">
      {brand}
      <div>
        <div class="company-name">{company_name}</div>
        {company_meta}
      </div>
    </div>
    <div class="title-block">
      <h1>Invoice</h1>
      <div class="number">{increment}</div>
      <span class="badge {status_class}">{state_label}</span>
    </div>
  </header>
  <section class="meta">
    <div><span>Invoice date</span><strong>{invoice_date}</strong></div>
    <div><span>Order</span><strong>{order_increment}</strong></div>
    <div><span>Payment method</span><strong>{payment}</strong></div>
    <div><span>Items</span><strong>{total_qty}</strong></div>
  </section>
  <section class="content">
    <div class="parties">
      <div class="party">
        <h3>Billed to</h3>
        <p>{billing}</p>
      </div>
      <div class="party">
        <h3>Ship to</h3>
        <p>{shipping}</p>
      </div>
    </div>
    <table class="items">
      <thead>
        <tr>
          <th scope="col">#</th>
          <th scope="col">Description</th>
          <th scope="col" class="num">Qty</th>
          <th scope="col" class="num">Unit price</th>
          <th scope="col" class="num">Amount</th>
        </tr>
      </thead>
      <tbody>
        {rows}
      </tbody>
    </table>
    <div class="summary">
      <div class="summary-box">
        {summary}
      </div>
    </div>
    <div class="notes">
      <h4>Notes</h4>
      <p>Thank you for your business. This invoice was generated electronically and is valid without a signature.</p>
    </div>
  </section>
  <footer class="footer">
    <div>
      <div class="thanks">Thank you, {company_name}.</div>
      <div>{footer_text}</div>
    </div>
    <div>{increment} · Order {order_increment}</div>
  </footer>
</div>
</body>
</html>"#,
            styles = INVOICE_STYLES,
            increment = increment,
            brand = brand,
            company_name = company_name_html,
            company_meta = company_meta,
            status_class = status_class,
            state_label = escape_html(&state_label),
            invoice_date = invoice_date,
            order_increment = order_increment,
            payment = payment,
            total_qty = quantity_label(total_qty),
            billing = address_html(billing.as_ref()),
            shipping = address_html(shipping.as_ref()),
            rows = rows,
            summary = summary,
            footer_text = footer_text,
        );

        larust_support::axum::response::Html(html).into_response()
    }

    /// `GET /api/admin/sales/orders/view/{id}`.
    pub async fn order_view(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match sales::order_resource(id).await {
            Ok(order) => resource(order, Value::Null),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/sales/orders/{id}/cancel`.
    pub async fn order_cancel(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        match sales::cancel_order(id).await {
            Ok(()) => resource(Value::Array(Vec::new()), "The order has been canceled."),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/sales/invoices/create/{order_id}`.
    pub async fn invoice_create(admin: auth::Admin, Path(order_id): Path<i64>) -> Response {
        let _ = admin;
        match sales::create_invoice(order_id).await {
            Ok(invoice) => resource(invoice, "The invoice has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/sales/invoices/view/{id}`.
    pub async fn invoice_view(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let invoice = db::find_by_id("invoices", id).await.ok().flatten();
        match invoice {
            Some(invoice) => resource(invoice, Value::Null),
            None => message(StatusCode::NOT_FOUND, "Invoice not found."),
        }
    }

    /// `POST /api/admin/sales/orders/comment/{order_id}` - `{comment, customer_notified}`.
    pub async fn order_comment(
        admin: auth::Admin,
        Path(order_id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let comment = body["comment"].as_str().unwrap_or_default();
        match crate::support::admin_crud::add_order_comment(
            order_id,
            admin.id,
            comment,
            body["customer_notified"].as_bool().unwrap_or(false),
        )
        .await
        {
            Ok(_) => resource(
                Value::Array(Vec::new()),
                "The comment has been added successfully.",
            ),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/sales/shipments/create/{order_id}` -
    /// `{source_id, items: {order_item_id: qty}}` (o `{"all": qty}`).
    pub async fn shipment_create(
        admin: auth::Admin,
        Path(order_id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        let source_id = body["source_id"].as_i64().unwrap_or(1);
        let requested = body["items"].as_object().cloned().unwrap_or_default();
        match sales::create_shipment(order_id, source_id, &requested).await {
            Ok(shipment) => resource(shipment, "The shipment has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/sales/shipments/view/{id}`.
    pub async fn shipment_view(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let shipment = db::find_by_id("shipments", id).await.ok().flatten();
        match shipment {
            Some(shipment) => resource(shipment, Value::Null),
            None => message(StatusCode::NOT_FOUND, "Shipment not found."),
        }
    }

    /// `POST /api/admin/sales/refunds/create/{order_id}` -
    /// `{items: {order_item_id: qty}, adjustment_refund, adjustment_fee}`.
    pub async fn refund_create(
        admin: auth::Admin,
        Path(order_id): Path<i64>,
        Json(body): Json<Value>,
    ) -> Response {
        let _ = admin;
        match crate::support::admin_crud::create_refund(order_id, &body).await {
            Ok(refund) => resource(refund, "The refund has been created successfully."),
            Err(error) => error.into(),
        }
    }

    /// `POST /api/admin/sales/orders/{id}/capture` - captura un pago Stripe
    /// autorizado con captura manual.
    pub async fn order_capture(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let order = db::find_by_id("orders", id).await.ok().flatten();
        let Some(order) = order else {
            return message(StatusCode::NOT_FOUND, "Order not found.");
        };
        if order["payment_status"].as_str() != Some("authorized") {
            return message(
                StatusCode::BAD_REQUEST,
                "The order has no authorized Stripe payment to capture.",
            );
        }
        let additional = db::select_one(
            "SELECT `additional` FROM `order_payment` WHERE `order_id` = ? ORDER BY `id` DESC LIMIT 1",
            &[Value::from(id)],
        )
        .await
        .ok()
        .flatten()
        .and_then(|row| {
            row["additional"]
                .as_str()
                .and_then(|raw| larust_support::serde_json::from_str::<Value>(raw).ok())
        })
        .unwrap_or_else(|| json!({}));
        let Some(payment_intent) = additional["payment_intent_id"]
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return message(
                StatusCode::BAD_REQUEST,
                "The order has no Stripe PaymentIntent to capture.",
            );
        };
        match stripe::capture_payment_intent(id, payment_intent).await {
            Ok(intent) => {
                let currency = intent["currency"]
                    .as_str()
                    .unwrap_or("eur")
                    .to_ascii_uppercase();
                let amount = stripe::major_units(
                    intent["amount_received"]
                        .as_i64()
                        .filter(|amount| *amount > 0)
                        .or_else(|| intent["amount"].as_i64())
                        .unwrap_or(0),
                    &currency,
                );
                let data = stripe::payment_details(&intent);
                match sales::confirm_payment(id, payment_intent, amount, &currency, data).await {
                    Ok(()) => resource(
                        Value::Array(Vec::new()),
                        "The payment has been captured successfully.",
                    ),
                    Err(error) => error.into(),
                }
            }
            Err(error) => error.into(),
        }
    }

    /// `GET /api/admin/sales/refunds/view/{id}`.
    pub async fn refund_view(admin: auth::Admin, Path(id): Path<i64>) -> Response {
        let _ = admin;
        let refund = db::find_by_id("refunds", id).await.ok().flatten();
        match refund {
            Some(refund) => resource(refund, Value::Null),
            None => message(StatusCode::NOT_FOUND, "Refund not found."),
        }
    }

    /// `GET /api/admin/datagrid/look-up?table=&label=&value=&q=` -
    /// autocompletado genérico para filtros relacionales del datagrid
    /// (devuelve pares {label, value} de cualquier tabla).
    pub async fn datagrid_look_up(Query(query): Query<HashMap<String, String>>) -> Response {
        let table = query.get("table").cloned().unwrap_or_default();
        let label_column = query.get("label").cloned().unwrap_or_else(|| "name".into());
        let value_column = query.get("value").cloned().unwrap_or_else(|| "id".into());
        let term = query.get("q").cloned().unwrap_or_default();

        // Sólo identificadores seguros.
        let valid = |name: &str| {
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        };
        if !valid(&table) || !valid(&label_column) || !valid(&value_column) {
            return message(StatusCode::BAD_REQUEST, "Invalid look-up parameters.");
        }

        let sql = format!(
            "SELECT `{}` AS `label`, `{}` AS `value` FROM `{}`              WHERE CAST(`{}` AS CHAR) LIKE ? ORDER BY `{}` LIMIT 20",
            label_column, value_column, table, label_column, label_column
        );
        let rows = db::select(&sql, &[Value::String(format!("%{term}%"))])
            .await
            .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }

    /// `GET /api/admin/sales/orders/search?q=` - increment/customer search.
    pub async fn order_search(admin: auth::Admin, Query(query): Query<HashMap<String, String>>) -> Response {
        let _ = admin;
        let term = query.get("query").map(String::as_str).unwrap_or_default();
        let rows = db::select(
            "SELECT `id`, `increment_id`, `customer_email`, `base_grand_total`, `status` FROM `orders` \
             WHERE `increment_id` LIKE ? OR `customer_email` LIKE ? ORDER BY `id` DESC LIMIT 10",
            &[
                Value::String(format!("%{term}%")),
                Value::String(format!("%{term}%")),
            ],
        )
        .await
        .unwrap_or_default();
        resource(json!(rows), Value::Null)
    }
}
