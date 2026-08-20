//! Runtime configuration loaded from the environment.
//!
//! `Config` is read once at startup and shared behind an `Arc`. Every optional
//! credential stays `Option<String>` so the rest of the app can degrade
//! honestly when a key is missing instead of fabricating data.

use std::env;

/// Application configuration resolved from environment variables.
#[derive(Clone)]
pub(crate) struct Config {
    pub(crate) ryo_mcp_url: String,
    pub(crate) ryo_mcp_key: Option<String>,
    pub(crate) tavily_api_key: Option<String>,
    pub(crate) openai_api_key: Option<String>,
    pub(crate) openai_model: String,
    pub(crate) openai_responses_url: String,
    pub(crate) coingecko_demo_api_key: Option<String>,
    pub(crate) defillama_enabled: bool,
    pub(crate) dexscreener_enabled: bool,
    pub(crate) ryo_mock_enabled: bool,
    pub(crate) database_path: String,
    pub(crate) port: u16,
    pub(crate) watch_loop_enabled: bool,
}

impl Config {
    /// Build a `Config` from the current process environment, applying the
    /// documented defaults for anything that is not set.
    pub(crate) fn from_env() -> Self {
        let ryo_mcp_key = env_optional("RYO_MCP_KEY");
        let tavily_api_key = env_optional("TAVILY_API_KEY");
        let openai_api_key = env_optional("OPENAI_API_KEY");

        Self {
            ryo_mcp_url: env::var("RYO_MCP_URL")
                .unwrap_or_else(|_| "https://app-ryochan.com/api/mcp".to_string())
                .trim_end_matches('/')
                .to_string(),
            ryo_mcp_key,
            tavily_api_key,
            openai_api_key,
            openai_model: env_optional("OPENAI_MODEL")
                .unwrap_or_else(|| "gpt-5.6-terra".to_string()),
            openai_responses_url: env_optional("OPENAI_RESPONSES_URL")
                .unwrap_or_else(|| "https://api.openai.com/v1/responses".to_string()),
            coingecko_demo_api_key: env_optional("COINGECKO_DEMO_API_KEY"),
            defillama_enabled: env_bool("DEFILLAMA_ENABLED", false),
            dexscreener_enabled: env_bool("DEXSCREENER_ENABLED", false),
            ryo_mock_enabled: env_bool("APP_MOCK_RYO", false),
            database_path: env_optional("APP_DB_PATH")
                .unwrap_or_else(|| format!("{}/data/pulse.db", env!("CARGO_MANIFEST_DIR"))),
            port: env::var("APP_PORT")
                .or_else(|_| env::var("PORT"))
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(8788),
            watch_loop_enabled: env_bool("APP_ENABLE_WATCH_LOOP", false),
        }
    }

    /// Names of the required credentials that are still missing.
    /// `RYO_MCP_KEY` can be bypassed only by explicit mock mode, which marks
    /// every RYO result as simulated.
    pub(crate) fn missing_required_keys(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.ryo_mcp_key.is_none() && !self.ryo_mock_enabled {
            missing.push("RYO_MCP_KEY");
        }
        if self.tavily_api_key.is_none() {
            missing.push("TAVILY_API_KEY");
        }
        if self.openai_api_key.is_none() {
            missing.push("OPENAI_API_KEY");
        }
        missing
    }
}

/// Read an environment variable, trimming whitespace and treating empty as unset.
fn env_optional(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Read a boolean-ish environment variable, accepting common truthy spellings.
fn env_bool(name: &str, default: bool) -> bool {
    env::var(name)
        .ok()
        .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}
