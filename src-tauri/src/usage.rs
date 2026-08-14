use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSnapshot {
    /// 0.0..=100.0
    pub percent: f64,
    /// RFC3339 timestamp of the next reset, when known.
    pub resets_at: Option<String>,
    /// True when this snapshot is a cached value served after a failed refresh.
    pub stale: bool,
    /// Present when the last refresh attempt failed (used for stale tooltips/logs).
    pub last_error: Option<String>,
}

impl UsageSnapshot {
    pub fn fresh(percent: f64, resets_at: Option<String>) -> Self {
        Self {
            percent: percent.clamp(0.0, 100.0),
            resets_at,
            stale: false,
            last_error: None,
        }
    }

    pub fn as_stale(&self, error: String) -> Self {
        Self {
            percent: self.percent,
            resets_at: self.resets_at.clone(),
            stale: true,
            last_error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsagePayload {
    pub claude: Option<UsageSnapshot>,
    pub cursor: Option<UsageSnapshot>,
    pub codex: Option<UsageSnapshot>,
}
