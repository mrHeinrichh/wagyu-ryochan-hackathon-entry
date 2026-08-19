//! RYO Global Token News Reasoning Layer - backend entry point.
//!
//! This file only bootstraps the process: load env, build shared state, wire
//! the router, optionally start the watch loop, and serve. All behaviour lives
//! in the modules below.
//!
//! Module map:
//! - `config`   environment-driven configuration
//! - `state`    shared `AppState` and the news cache entry
//! - `error`    the shared `ApiError` type
//! - `domain`   request/response and receipt data models
//! - `services` outbound integrations (news, RYO, OpenAI, shared HTTP)
//! - `reasoning` the scoring/verdict pipeline (`build_pulse`)
//! - `handlers` HTTP handlers and the router
//! - `watch`    the optional background watch loop
//! - `util`     small shared helpers

mod config;
mod domain;
mod error;
mod handlers;
mod reasoning;
mod services;
mod state;
mod util;
mod watch;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use reqwest::Client;
use tracing::info;

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "global_token_news_pulse=info,tower_http=warn".into()),
        )
        .init();

    let config = Arc::new(Config::from_env());
    let client = Client::builder()
        .timeout(Duration::from_secs(45))
        .build()
        .expect("reqwest client");
    let state = AppState::new(config.clone(), client);

    if config.watch_loop_enabled {
        tokio::spawn(watch::watch_loop(state.clone()));
    }

    // Serve the Next.js static export produced by npm run build:static.
    let static_dir = format!("{}/../frontend/out", env!("CARGO_MANIFEST_DIR"));
    let app = handlers::router(state, static_dir);

    let addr: SocketAddr = ([127, 0, 0, 1], config.port).into();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind local server");

    info!(
        "RYO Global Token Reasoning Layer listening on http://{}",
        addr
    );
    axum::serve(listener, app).await.expect("server");
}
