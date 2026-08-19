//! The receipt data model: news, RYO evidence, scores, sections, and the final
//! decision receipt returned to the UI and stored for replay.

use serde::Serialize;
use serde_json::Value;

/// One integration's availability line for a run (Tavily, a RYO tool, etc.).
#[derive(Debug, Serialize, Clone)]
pub(crate) struct SourceAvailability {
    pub(crate) source: String,
    pub(crate) status: String,
    pub(crate) data_mode: String,
    pub(crate) detail: String,
    pub(crate) as_of: String,
}

/// A single news item after normalization from the upstream search.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct NewsStory {
    pub(crate) id: String,
    pub(crate) headline: String,
    pub(crate) source: String,
    pub(crate) url: String,
    pub(crate) content: String,
    pub(crate) published_at: Option<String>,
    pub(crate) region: Option<String>,
    pub(crate) language: Option<String>,
    pub(crate) data_mode: String,
}

/// The captured result of one RYO MCP tool call, kept verbatim for provenance.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct RyoToolEvidence {
    pub(crate) tool: String,
    pub(crate) status: String,
    pub(crate) data_mode: String,
    pub(crate) as_of: String,
    pub(crate) request: Value,
    pub(crate) result: Value,
    pub(crate) summary: Option<String>,
    pub(crate) warnings: Vec<String>,
}

/// Per-signal sub-scores plus the blended impact. `None` means the input was
/// unavailable and was excluded from the weighting, never treated as zero.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct StoryScore {
    pub(crate) relevance: Option<u8>,
    pub(crate) credibility: Option<u8>,
    pub(crate) novelty: Option<u8>,
    pub(crate) urgency: Option<u8>,
    pub(crate) market_confirmation: Option<u8>,
    pub(crate) uncertainty: Option<u8>,
    pub(crate) impact: u8,
    pub(crate) formula: String,
}

/// A scored, categorized story with its reasoning trace and recommendation.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct StoryCard {
    pub(crate) id: String,
    pub(crate) headline: String,
    pub(crate) source: String,
    pub(crate) url: String,
    pub(crate) region: Option<String>,
    pub(crate) language: Option<String>,
    pub(crate) timestamp: Option<String>,
    pub(crate) related_token: String,
    pub(crate) narrative_cluster: String,
    pub(crate) sentiment: String,
    pub(crate) score: StoryScore,
    pub(crate) ryo_alignment: String,
    pub(crate) missing_data: Vec<String>,
    pub(crate) recommendation: String,
    pub(crate) confidence: String,
    pub(crate) reasoning: Vec<String>,
    pub(crate) category: String,
    pub(crate) data_mode: String,
}

/// A named bucket of cards (position-changing, watch, unverified, noise).
#[derive(Debug, Serialize, Clone)]
pub(crate) struct RankedSection {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) cards: Vec<StoryCard>,
}

/// Roll-up counts and headline for a run.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct ReceiptSummary {
    pub(crate) headline: String,
    pub(crate) conclusion: String,
    pub(crate) highest_impact: Option<u8>,
    pub(crate) position_changing_count: usize,
    pub(crate) watch_count: usize,
    pub(crate) noise_count: usize,
    pub(crate) unverified_count: usize,
}

/// The judged verdict, either from OpenAI or the deterministic fallback.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct ReasoningVerdict {
    pub(crate) decision: String,
    pub(crate) confidence: u8,
    pub(crate) token_symbol: String,
    pub(crate) top_global_news: Vec<String>,
    pub(crate) why_it_matters: Vec<String>,
    pub(crate) ryo_market_evidence: Vec<String>,
    pub(crate) missing_data: Vec<String>,
    pub(crate) warnings: Vec<String>,
    pub(crate) recommended_next_action: String,
    pub(crate) generated_by: String,
}

/// The compact reasoning-layer output surfaced at the top level of a receipt.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct ReasoningLayerOutput {
    pub(crate) signal: String,
    pub(crate) symbol: String,
    pub(crate) confidence: f32,
    pub(crate) reasoning: String,
    pub(crate) ryo_tools_used: Vec<String>,
    pub(crate) unavailable_data: Vec<String>,
    pub(crate) next_action: String,
    pub(crate) affected_tokens: Vec<String>,
    pub(crate) sentiment: String,
    pub(crate) market_confirmation: String,
    pub(crate) timestamp: String,
    pub(crate) run_id: String,
}

/// The full, replayable decision receipt returned by a reasoning run.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct DecisionReceipt {
    pub(crate) id: String,
    pub(crate) run_id: String,
    pub(crate) symbol: String,
    pub(crate) timeframe: String,
    pub(crate) regions: Vec<String>,
    pub(crate) sources: Vec<String>,
    pub(crate) user_thesis: Option<String>,
    pub(crate) created_at: String,
    pub(crate) status: String,
    pub(crate) data_mode: String,
    pub(crate) signal: String,
    pub(crate) confidence: f32,
    pub(crate) reasoning: String,
    pub(crate) ryo_tools_used: Vec<String>,
    pub(crate) unavailable_data: Vec<String>,
    pub(crate) next_action: String,
    pub(crate) reasoning_layer: ReasoningLayerOutput,
    pub(crate) summary: ReceiptSummary,
    pub(crate) verdict: ReasoningVerdict,
    pub(crate) sections: Vec<RankedSection>,
    pub(crate) stories: Vec<NewsStory>,
    pub(crate) ryo: Vec<RyoToolEvidence>,
    pub(crate) availability: Vec<SourceAvailability>,
    pub(crate) warnings: Vec<String>,
    pub(crate) scoring_notes: Vec<String>,
}

/// A trimmed receipt entry for the history list.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct ReceiptListItem {
    pub(crate) id: String,
    pub(crate) run_id: String,
    pub(crate) symbol: String,
    pub(crate) created_at: String,
    pub(crate) status: String,
    pub(crate) data_mode: String,
    pub(crate) signal: String,
    pub(crate) confidence: f32,
    pub(crate) headline: String,
    pub(crate) highest_impact: Option<u8>,
}

/// A token registered for the background watch loop.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct WatchItem {
    pub(crate) symbol: String,
    pub(crate) interval_minutes: u64,
    pub(crate) enabled: bool,
    pub(crate) created_at: String,
    pub(crate) last_checked_at: Option<String>,
    pub(crate) next_check_at: Option<String>,
    pub(crate) last_receipt_id: Option<String>,
    pub(crate) last_cluster: Option<String>,
}
