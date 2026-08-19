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
use crate::domain::{DecisionReceipt, NewsStory, WatchItem};

/// Everything the handlers and background loop need to do their work.
#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) config: Arc<Config>,
    pub(crate) client: Client,
    pub(crate) receipts: Arc<RwLock<Vec<DecisionReceipt>>>,
    pub(crate) watchlist: Arc<RwLock<HashMap<String, WatchItem>>>,
    pub(crate) news_cache: Arc<RwLock<HashMap<String, CacheEntry<Vec<NewsStory>>>>>,
}

impl AppState {
    /// Assemble state from a shared config and a ready HTTP client.
    pub(crate) fn new(config: Arc<Config>, client: Client) -> Self {
        Self {
            config,
            client,
            receipts: Arc::new(RwLock::new(Vec::new())),
            watchlist: Arc::new(RwLock::new(HashMap::new())),
            news_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// A cached value with the data mode it was fetched under and an expiry.
#[derive(Debug, Clone)]
pub(crate) struct CacheEntry<T> {
    pub(crate) value: T,
    pub(crate) data_mode: String,
    pub(crate) expires_at: DateTime<Utc>,
}
