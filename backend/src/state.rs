//! Shared application state.
//!
//! `AppState` is cloned into every handler by Axum. It holds the shared HTTP
//! client, the configuration, and the in-memory stores for receipts, the
//! watchlist, and the short-lived news cache. All mutable state sits behind an
//! `RwLock` so handlers and the watch loop can share it safely.

use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use reqwest::Client;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::domain::{DecisionReceipt, NewsStory, TokenInfo, WatchItem};
use crate::safety::SafetyNet;
use crate::storage::Persistence;

/// Everything the handlers and background loop need to do their work.
#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) config: Arc<Config>,
    pub(crate) client: Client,
    pub(crate) receipts: Arc<RwLock<Vec<DecisionReceipt>>>,
    pub(crate) watchlist: Arc<RwLock<HashMap<String, WatchItem>>>,
    pub(crate) storage: Arc<Persistence>,
    pub(crate) news_cache: Arc<RwLock<HashMap<String, CacheEntry<Vec<NewsStory>>>>>,
    pub(crate) token_cache: Arc<RwLock<Option<CacheEntry<Vec<TokenInfo>>>>>,
    pub(crate) safety: SafetyNet,
}

impl AppState {
    /// Assemble state from a shared config and a ready HTTP client.
    pub(crate) fn new(config: Arc<Config>, client: Client) -> Result<Self, String> {
        let storage = Persistence::open(&config.database_path)?;
        let receipts = storage.load_receipts(100)?;
        let watchlist = storage.load_watchlist()?;
        let safety = SafetyNet::new(&config);
        Ok(Self {
            config,
            client,
            receipts: Arc::new(RwLock::new(receipts)),
            watchlist: Arc::new(RwLock::new(watchlist)),
            storage: Arc::new(storage),
            news_cache: Arc::new(RwLock::new(HashMap::new())),
            token_cache: Arc::new(RwLock::new(None)),
            safety,
        })
    }
}

/// A cached value with its fetch time and expiry.
#[derive(Debug, Clone)]
pub(crate) struct CacheEntry<T> {
    pub(crate) value: T,
    pub(crate) fetched_at: DateTime<Utc>,
    pub(crate) expires_at: DateTime<Utc>,
}
