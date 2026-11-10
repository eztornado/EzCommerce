//! Middleware-ish helpers. Axum extractors in `app/Support/auth.rs` play the
//! role of Laravel guards (`admin`, `customer`); channel/locale/currency
//! resolution mirrors Bagisto's `Locale`/`Currency` middleware in
//! `controllers/core_controller.rs`.
