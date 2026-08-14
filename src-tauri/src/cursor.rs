use crate::usage::UsageSnapshot;
use serde::Deserialize;
use std::path::PathBuf;

const USAGE_URL: &str = "https://cursor.com/api/usage-summary";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsageSummary {
    billing_cycle_end: Option<String>,
    individual_usage: Option<IndividualUsage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IndividualUsage {
    plan: Option<PlanUsage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanUsage {
    total_percent_used: f64,
}

fn state_db_path() -> Option<PathBuf> {
    // %APPDATA%/Cursor/User/globalStorage/state.vscdb on Windows.
    Some(
        dirs::config_dir()?
            .join("Cursor")
            .join("User")
            .join("globalStorage")
            .join("state.vscdb"),
    )
}

fn read_access_token() -> Result<String, String> {
    let path = state_db_path().ok_or("could not resolve Cursor config directory")?;
    if !path.exists() {
        return Err(format!("Cursor state db not found at {}", path.display()));
    }
    // Open read-only: Cursor itself may hold the file open while running.
    let conn = rusqlite::Connection::open_with_flags(
        &path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| format!("could not open Cursor state db: {e}"))?;

    conn.query_row(
        "SELECT value FROM ItemTable WHERE key = ?1",
        ["cursorAuth/accessToken"],
        |row| row.get::<_, String>(0),
    )
    .map_err(|e| format!("no Cursor session token found: {e}"))
}

#[derive(Debug, Deserialize)]
struct JwtClaims {
    sub: String,
}

/// The session cookie Cursor expects is `{userId}::{token}`, where userId is
/// the JWT's own `sub` claim — the raw access token alone is rejected (401).
fn session_cookie_value(token: &str) -> Result<String, String> {
    let payload_segment = token
        .split('.')
        .nth(1)
        .ok_or("access token is not a JWT (no payload segment)")?;

    use base64::Engine;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_segment)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(payload_segment))
        .map_err(|e| format!("could not base64-decode JWT payload: {e}"))?;

    let claims: JwtClaims =
        serde_json::from_slice(&decoded).map_err(|e| format!("could not parse JWT claims: {e}"))?;

    Ok(format!("{}%3A%3A{}", claims.sub, token))
}

pub async fn fetch(client: &reqwest::Client) -> Result<UsageSnapshot, String> {
    let token = read_access_token()?;
    let cookie_value = session_cookie_value(&token)?;

    let resp = client
        .get(USAGE_URL)
        .header("Cookie", format!("WorkosCursorSessionToken={cookie_value}"))
        .header("Origin", "https://cursor.com")
        .header("Referer", "https://cursor.com/dashboard")
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    let parsed: UsageSummary = resp
        .json()
        .await
        .map_err(|e| format!("could not parse response: {e}"))?;

    let percent = parsed
        .individual_usage
        .and_then(|u| u.plan)
        .map(|p| p.total_percent_used)
        .ok_or_else(|| "response had no individual plan usage".to_string())?;

    Ok(UsageSnapshot::fresh(percent, parsed.billing_cycle_end))
}
