//! Mail dispatch: `log` (development), `resend` (HTTP API) and `smtp`
//! (via the framework transport). Mirrors the `.env` `MAIL_MAILER` setup.

use larust_support::config_env::env_or;
use larust_support::serde_json::json;

/// Sends a simple text email. Never fails the caller: mail problems are
/// logged and swallowed so a broken SMTP cannot break a checkout.
pub async fn send(to: &str, subject: &str, body: &str) {
    let driver = env_or("MAIL_MAILER", "log");
    match driver.as_str() {
        "resend" => send_resend(to, subject, body).await,
        "smtp" => send_smtp(to, subject, body).await,
        _ => {
            larust_support::tracing::info!(
                to = %to,
                subject = %subject,
                "MAIL_MAILER=log -> {body}"
            );
        }
    }
}

async fn send_resend(to: &str, subject: &str, body: &str) {
    let api_key = env_or("RESEND_API_KEY", "");
    if api_key.is_empty() {
        larust_support::tracing::warn!("RESEND_API_KEY vacía; email no enviado a {to}");
        return;
    }
    let from = env_or("MAIL_FROM_ADDRESS", "shop@example.com");
    let client = reqwest::Client::new();
    let result = client
        .post("https://api.resend.com/emails")
        .bearer_auth(&api_key)
        .json(&json!({
            "from": from,
            "to": [to],
            "subject": subject,
            "text": body,
        }))
        .send()
        .await;
    if let Err(error) = result {
        larust_support::tracing::error!(%error, "resend send failed");
    }
}

async fn send_smtp(to: &str, subject: &str, body: &str) {
    // The framework SMTP transport is not wired here yet; fall back to the
    // log driver so local flows keep working.
    larust_support::tracing::info!(
        to = %to,
        subject = %subject,
        host = %env_or("MAIL_HOST", "127.0.0.1"),
        "MAIL_MAILER=smtp (fallback log) -> {body}"
    );
}
