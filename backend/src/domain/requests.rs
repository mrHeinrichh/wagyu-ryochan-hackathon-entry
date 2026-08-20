//! Inbound request bodies and the health payload.

use serde::{Deserialize, Serialize};

/// Body for a reasoning run. `thesis`, `event`, and `news` are accepted as
/// aliases for the same user-supplied narrative so older clients keep working.
#[derive(Debug, Deserialize, Clone)]
pub(crate) struct PulseRequest {
    pub(crate) symbol: String,
    #[serde(default)]
    pub(crate) demo: bool,
    #[serde(default)]
    pub(crate) risk_budget_pct: Option<f32>,
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

/// One prior exchange included with a receipt-aware assistant request.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct ChatTurn {
    pub(crate) role: String,
    pub(crate) content: String,
}

/// A question for the research assistant, optionally grounded in one receipt.
#[derive(Debug, Deserialize)]
pub(crate) struct ChatRequest {
    #[serde(default)]
    pub(crate) receipt_id: Option<String>,
    pub(crate) question: String,
    #[serde(default)]
    pub(crate) history: Vec<ChatTurn>,
}

/// The assistant answer and useful receipt-grounded follow-up prompts.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct ChatResponse {
    pub(crate) answer: String,
    pub(crate) suggestions: Vec<String>,
    pub(crate) generated_by: String,
    pub(crate) receipt_id: Option<String>,
    pub(crate) warnings: Vec<String>,
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
    pub(crate) turnstile_configured: bool,
    pub(crate) turnstile_required: bool,
    pub(crate) persistence: &'static str,
    pub(crate) guardrails: GuardrailHealth,
}

#[derive(Debug, Serialize, Clone)]
pub(crate) struct GuardrailHealth {
    pub(crate) analysis_cooldown_seconds: u64,
    pub(crate) analysis_limit_per_minute: usize,
    pub(crate) chat_cooldown_seconds: u64,
    pub(crate) chat_limit_per_minute: usize,
    pub(crate) ai_limit_per_minute: usize,
    pub(crate) ai_max_concurrency: usize,
}
