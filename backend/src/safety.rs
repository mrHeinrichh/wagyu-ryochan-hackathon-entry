//! In-process safety controls for expensive and AI-backed operations.

use std::{
    collections::{HashMap, VecDeque},
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{http::StatusCode, http::header::HeaderMap};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::config::Config;
use crate::error::ApiError;

const MAX_TRACKED_CLIENTS: usize = 4_096;

#[derive(Debug, Clone, Copy)]
pub(crate) enum GuardAction {
    Analysis,
    Demo,
    Chat,
    Watchlist,
}

impl GuardAction {
    fn label(self) -> &'static str {
        match self {
            Self::Analysis => "analysis",
            Self::Demo => "sample",
            Self::Chat => "assistant",
            Self::Watchlist => "watchlist",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RatePolicy {
    per_client: usize,
    global: usize,
    window: Duration,
    cooldown: Duration,
}

#[derive(Default)]
struct Bucket {
    accepted: VecDeque<Instant>,
    last_accepted: Option<Instant>,
}

#[derive(Default)]
struct RateStore {
    buckets: HashMap<String, Bucket>,
}

/// Shared quotas and AI concurrency controls. Limits are deliberately held in
/// process so no user content or IP address is persisted.
#[derive(Clone)]
pub(crate) struct SafetyNet {
    requests: Arc<Mutex<RateStore>>,
    ai_requests: Arc<Mutex<Bucket>>,
    ai_slots: Arc<Semaphore>,
    analysis: RatePolicy,
    demo: RatePolicy,
    chat: RatePolicy,
    watchlist: RatePolicy,
    ai_limit_per_minute: usize,
}

impl SafetyNet {
    pub(crate) fn new(config: &Config) -> Self {
        Self {
            requests: Arc::new(Mutex::new(RateStore::default())),
            ai_requests: Arc::new(Mutex::new(Bucket::default())),
            ai_slots: Arc::new(Semaphore::new(config.ai_max_concurrency)),
            analysis: RatePolicy {
                per_client: config.analysis_rate_limit_per_minute,
                global: config.analysis_global_rate_limit_per_minute,
                window: Duration::from_secs(60),
                cooldown: Duration::from_secs(config.analysis_cooldown_seconds),
            },
            demo: RatePolicy {
                per_client: 20,
                global: 100,
                window: Duration::from_secs(60),
                cooldown: Duration::from_secs(2),
            },
            chat: RatePolicy {
                per_client: config.chat_rate_limit_per_minute,
                global: config.chat_global_rate_limit_per_minute,
                window: Duration::from_secs(60),
                cooldown: Duration::from_secs(config.chat_cooldown_seconds),
            },
            watchlist: RatePolicy {
                per_client: 20,
                global: 120,
                window: Duration::from_secs(60),
                cooldown: Duration::from_secs(1),
            },
            ai_limit_per_minute: config.ai_rate_limit_per_minute,
        }
    }

    pub(crate) fn check_request(
        &self,
        action: GuardAction,
        headers: &HeaderMap,
    ) -> Result<(), ApiError> {
        let policy = match action {
            GuardAction::Analysis => self.analysis,
            GuardAction::Demo => self.demo,
            GuardAction::Chat => self.chat,
            GuardAction::Watchlist => self.watchlist,
        };
        let now = Instant::now();
        let client = client_key(headers);
        let requested_client_bucket = format!("{}:{client}", action.label());
        let global_bucket = format!("{}:global", action.label());
        let mut store = self
            .requests
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        compact_store(&mut store, now);
        let client_bucket = if store.buckets.len() >= MAX_TRACKED_CLIENTS
            && !store.buckets.contains_key(&requested_client_bucket)
        {
            format!("{}:overflow", action.label())
        } else {
            requested_client_bucket
        };
        let client_retry = retry_after(
            store.buckets.entry(client_bucket.clone()).or_default(),
            now,
            policy.per_client,
            policy.window,
            policy.cooldown,
        );
        let global_retry = retry_after(
            store.buckets.entry(global_bucket.clone()).or_default(),
            now,
            policy.global,
            policy.window,
            Duration::ZERO,
        );
        if let Some(seconds) = client_retry.or(global_retry) {
            return Err(rate_limit_error(action.label(), seconds));
        }

        record_acceptance(store.buckets.entry(client_bucket).or_default(), now);
        record_acceptance(store.buckets.entry(global_bucket).or_default(), now);
        Ok(())
    }

    /// Reserve one OpenAI slot and one request from the global model budget.
    /// Dropping the returned permit releases concurrency automatically.
    pub(crate) fn acquire_ai(&self) -> Result<OwnedSemaphorePermit, ApiError> {
        let permit = self.ai_slots.clone().try_acquire_owned().map_err(|_| {
            ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "ai_capacity_reached",
                "AI capacity is busy. The deterministic safety fallback will be used.",
            )
            .with_retry_after(2)
        })?;
        let now = Instant::now();
        let mut bucket = self
            .ai_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(seconds) = retry_after(
            &mut bucket,
            now,
            self.ai_limit_per_minute,
            Duration::from_secs(60),
            Duration::from_millis(250),
        ) {
            return Err(ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "ai_rate_limit_reached",
                "AI request budget reached. The deterministic safety fallback will be used.",
            )
            .with_retry_after(seconds));
        }
        record_acceptance(&mut bucket, now);
        Ok(permit)
    }
}

fn compact_store(store: &mut RateStore, now: Instant) {
    if store.buckets.len() < MAX_TRACKED_CLIENTS {
        return;
    }
    store.buckets.retain(|_, bucket| {
        bucket
            .last_accepted
            .is_some_and(|last| now.saturating_duration_since(last) < Duration::from_secs(120))
    });
}

fn retry_after(
    bucket: &mut Bucket,
    now: Instant,
    limit: usize,
    window: Duration,
    cooldown: Duration,
) -> Option<u64> {
    while bucket
        .accepted
        .front()
        .is_some_and(|time| now.saturating_duration_since(*time) >= window)
    {
        bucket.accepted.pop_front();
    }
    if let Some(last) = bucket.last_accepted {
        let elapsed = now.saturating_duration_since(last);
        if elapsed < cooldown {
            return Some(seconds_ceil(cooldown - elapsed));
        }
    }
    if bucket.accepted.len() >= limit.max(1) {
        let oldest = *bucket.accepted.front().expect("non-empty limited bucket");
        return Some(seconds_ceil(window - now.saturating_duration_since(oldest)));
    }
    None
}

fn record_acceptance(bucket: &mut Bucket, now: Instant) {
    bucket.accepted.push_back(now);
    bucket.last_accepted = Some(now);
}

fn seconds_ceil(duration: Duration) -> u64 {
    duration.as_secs().max(1) + u64::from(duration.subsec_nanos() > 0 && duration.as_secs() > 0)
}

fn rate_limit_error(action: &'static str, retry_after: u64) -> ApiError {
    ApiError::new(
        StatusCode::TOO_MANY_REQUESTS,
        "rate_limit_reached",
        format!("Please wait {retry_after}s before the next {action} request."),
    )
    .with_retry_after(retry_after)
}

/// Prefer the first proxy-provided address and heavily restrict its shape so
/// an untrusted header cannot create unbounded or log-hostile keys.
fn client_key(headers: &HeaderMap) -> String {
    ["x-forwarded-for", "x-real-ip"]
        .iter()
        .find_map(|name| headers.get(*name))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| value.len() <= 64)
        .and_then(|value| value.parse::<IpAddr>().ok())
        .map(|address| address.to_string())
        .unwrap_or_else(|| "anonymous".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooldown_returns_retry_time() {
        let now = Instant::now();
        let mut bucket = Bucket::default();
        record_acceptance(&mut bucket, now);

        assert_eq!(
            retry_after(
                &mut bucket,
                now + Duration::from_secs(1),
                10,
                Duration::from_secs(60),
                Duration::from_secs(3),
            ),
            Some(2)
        );
    }

    #[test]
    fn rejects_untrusted_client_header_shape() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "not an ip / injection".parse().unwrap());
        assert_eq!(client_key(&headers), "anonymous");
    }
}
