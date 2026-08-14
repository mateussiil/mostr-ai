use crate::usage::UsageSnapshot;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::PathBuf;

const USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";
const REFRESH_URL: &str = "https://auth.openai.com/oauth/token";
const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const USER_AGENT: &str = "codex-cli";

#[derive(Debug, Deserialize)]
struct AuthFile {
    tokens: Tokens,
}

#[derive(Debug, Deserialize)]
struct Tokens {
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    account_id: Option<String>,
    expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RefreshResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct UsageResponse {
    rate_limit: Option<RateLimit>,
}

#[derive(Debug, Deserialize)]
struct RateLimit {
    primary_window: Option<Window>,
}

#[derive(Debug, Deserialize)]
struct Window {
    used_percent: f64,
    reset_at: Option<i64>,
    reset_after_seconds: Option<i64>,
}

fn auth_path() -> Option<PathBuf> {
    Some(dirs::home_dir()?.join(".codex").join("auth.json"))
}

fn read_tokens() -> Result<Tokens, String> {
    let path = auth_path().ok_or("could not resolve home directory")?;
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("could not read {}: {e}", path.display()))?;
    let parsed: AuthFile =
        serde_json::from_str(&raw).map_err(|e| format!("could not parse auth.json: {e}"))?;
    Ok(parsed.tokens)
}

fn jwt_exp(token: &str) -> Option<DateTime<Utc>> {
    let payload_segment = token.split('.').nth(1)?;
    use base64::Engine;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_segment)
        .ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    let exp = claims.get("exp")?.as_i64()?;
    DateTime::from_timestamp(exp, 0)
}

fn is_expired(tokens: &Tokens) -> bool {
    let expiry = tokens
        .expires_at
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| jwt_exp(&tokens.access_token))
        .or_else(|| tokens.id_token.as_deref().and_then(jwt_exp));

    match expiry {
        Some(exp) => exp <= Utc::now(),
        // Unknown expiry: assume it might be stale rather than risk a guaranteed 401.
        None => true,
    }
}

/// Refreshes the access token in memory only — never writes back to
/// auth.json, since that file is owned by the real Codex CLI's login state.
async fn refresh_access_token(client: &reqwest::Client, refresh_token: &str) -> Result<String, String> {
    let resp = client
        .post(REFRESH_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", "openid profile email"),
            ("client_id", CLIENT_ID),
        ])
        .send()
        .await
        .map_err(|e| format!("refresh request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("refresh HTTP {status}: {body}"));
    }

    let parsed: RefreshResponse = resp
        .json()
        .await
        .map_err(|e| format!("could not parse refresh response: {e}"))?;
    Ok(parsed.access_token)
}

pub async fn fetch(client: &reqwest::Client) -> Result<UsageSnapshot, String> {
    let tokens = read_tokens()?;

    let access_token = if is_expired(&tokens) {
        let refresh_token = tokens
            .refresh_token
            .as_ref()
            .ok_or("access token expired and no refresh_token available")?;
        refresh_access_token(client, refresh_token).await?
    } else {
        tokens.access_token.clone()
    };

    let mut req = client
        .get(USAGE_URL)
        .bearer_auth(&access_token)
        .header("User-Agent", USER_AGENT);
    if let Some(account_id) = &tokens.account_id {
        req = req.header("ChatGPT-Account-Id", account_id);
    }

    let resp = req.send().await.map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    let parsed: UsageResponse = resp
        .json()
        .await
        .map_err(|e| format!("could not parse response: {e}"))?;

    let window = parsed
        .rate_limit
        .and_then(|r| r.primary_window)
        .ok_or_else(|| "response had no primary rate-limit window".to_string())?;

    let resets_at = window
        .reset_at
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
        .or_else(|| {
            window
                .reset_after_seconds
                .map(|secs| Utc::now() + chrono::Duration::seconds(secs))
        })
        .map(|dt| dt.to_rfc3339());

    Ok(UsageSnapshot::fresh(window.used_percent, resets_at))
}
