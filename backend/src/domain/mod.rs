//! Domain models: the request/response shapes and the receipt data structures.
//!
//! These types are pure data. They carry `Serialize`/`Deserialize` derives so
//! Axum can move them across the wire, but they hold no behaviour beyond a few
//! small constructors. Business logic lives in `crate::reasoning` and
//! `crate::services`.

mod receipt;
mod requests;
mod tokens;

pub(crate) use receipt::{
    DecisionChainStep, DecisionReceipt, NewsStory, PracticePlan, RankedSection,
    ReasoningLayerOutput, ReasoningVerdict, ReceiptListItem, ReceiptSummary, RegionSignal,
    RegionalConvergence, RyoToolEvidence, SourceAvailability, StoryCard, StoryScore, WatchItem,
};
pub(crate) use requests::{
    ChatRequest, ChatResponse, ChatTurn, GuardrailHealth, HealthResponse, PulseRequest,
    WatchRequest,
};
pub(crate) use tokens::{TokenInfo, default_tokens};
