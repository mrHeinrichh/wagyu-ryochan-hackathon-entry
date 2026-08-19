//! Inbound request bodies and the health payload.

use serde::{Deserialize, Serialize};

/// Body for a reasoning run. `thesis`, `event`, and `news` are accepted as
/// aliases for the same user-supplied narrative so older clients keep working.
#[derive(Debug, Deserialize, Clone)]
pub(crate) struct PulseRequest {
    pub(crate) symbol: String,
    #[serde(default)]
    pub(crate) timeframe: Option<String>,
    #[serde(default)]
    pub(crate) regions: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) sources: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) thesis: Option<String>,
    #[serde(default)]
    pub(crate) event: Option<String>,
    #[serde(default)]
    pub(crate) news: Option<String>,
}

/// Body for adding a token to the background watch loop.
#[derive(Debug, Deserialize)]
pub(crate) struct WatchRequest {
    pub(crate) symbol: String,
    #[serde(default)]
    pub(crate) interval_minutes: Option<u64>,
}

/// Snapshot of which integrations are wired up, surfaced by the health check.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct HealthResponse {
    pub(crate) status: &'static str,
    pub(crate) service: &'static str,
    pub(crate) generated_at: String,
    pub(crate) ryo_configured: bool,
    pub(crate) ryo_mock_enabled: bool,
    pub(crate) tavily_configured: bool,
    pub(crate) openai_configured: bool,
    pub(crate) coingecko_configured: bool,
    pub(crate) defillama_enabled: bool,
    pub(crate) dexscreener_enabled: bool,
    pub(crate) watch_loop_enabled: bool,
}
