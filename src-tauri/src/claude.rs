use crate::usage::UsageSnapshot;
use serde::Deserialize;
use std::path::PathBuf;

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
// Load-bearing: the endpoint 429s aggressively without a recognized Claude Code UA.
const USER_AGENT: &str = "claude-code/2.1.183";
const ANTHROPIC_BETA: &str = "oauth-2025-04-20";

#[derive(Debug, Deserialize)]
struct CredentialsFile {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: OauthBlock,
}

#[derive(Debug, Deserialize)]
struct OauthBlock {
    #[serde(rename = "accessToken")]
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct UsageResponse {
    five_hour: Option<Window>,
}

#[derive(Debug, Deserialize)]
struct Window {
    utilization: f64,
    resets_at: Option<String>,
}

fn credentials_path() -> Option<PathBuf> {
    Some(dirs::home_dir()?.join(".claude").join(".credentials.json"))
}

fn read_access_token() -> Result<String, String> {
    let path = credentials_path().ok_or("could not resolve home directory")?;
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("could not read {}: {e}", path.display()))?;
    let parsed: CredentialsFile =
        serde_json::from_str(&raw).map_err(|e| format!("could not parse credentials: {e}"))?;
    Ok(parsed.claude_ai_oauth.access_token)
}

pub async fn fetch(client: &reqwest::Client) -> Result<UsageSnapshot, String> {
    let token = read_access_token()?;

    let resp = client
        .get(USAGE_URL)
        .bearer_auth(token)
        .header("anthropic-beta", ANTHROPIC_BETA)
        .header("User-Agent", USER_AGENT)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

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
        .five_hour
        .ok_or_else(|| "response had no five_hour window".to_string())?;

    Ok(UsageSnapshot::fresh(window.utilization, window.resets_at))
}
