//! Cloudflare Turnstile verification for paid or compute-heavy endpoints.

use std::time::Duration;

use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::safety::client_ip;
use crate::state::AppState;

const TOKEN_HEADER: &str = "x-turnstile-token";
const MAX_TOKEN_LENGTH: usize = 2_048;
const ALWAYS_PASS_TEST_SECRET: &str = "1x0000000000000000000000000000000AA";

#[derive(Debug, Clone, Copy)]
pub(crate) enum TurnstileAction {
    MarketAnalysis,
    AssistantChat,
}

impl TurnstileAction {
    fn label(self) -> &'static str {
        match self {
            Self::MarketAnalysis => "market_analysis",
            Self::AssistantChat => "assistant_chat",
        }
    }
}

#[derive(Serialize)]
struct SiteverifyRequest<'a> {
    secret: &'a str,
    response: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    remoteip: Option<&'a str>,
    idempotency_key: String,
}

#[derive(Debug, Deserialize)]
struct SiteverifyResponse {
    success: bool,
    #[serde(default)]
    hostname: Option<String>,
    #[serde(default)]
    action: Option<String>,
}

/// Verify one single-use browser token before an expensive request proceeds.
/// When Turnstile is optional and no secret is configured, local development
/// continues normally. Required mode always fails closed.
pub(crate) async fn verify(
    state: &AppState,
    headers: &HeaderMap,
    expected_action: TurnstileAction,
) -> Result<(), ApiError> {
    let Some(secret) = state.config.turnstile_secret_key.as_deref() else {
        return if state.config.turnstile_required {
            Err(unavailable_error())
        } else {
            Ok(())
        };
    };
    let test_mode = secret == ALWAYS_PASS_TEST_SECRET;
    if test_mode
        && !state
            .config
            .turnstile_expected_hostname
            .as_deref()
            .is_some_and(is_local_hostname)
    {
        return Err(unavailable_error());
    }

    let token = match headers
        .get(TOKEN_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(token) => token,
        None if !state.config.turnstile_required => return Ok(()),
        None => {
            return Err(ApiError::new(
                StatusCode::FORBIDDEN,
                "turnstile_token_missing",
                "Complete the security check before continuing.",
            ));
        }
    };

    if token.len() > MAX_TOKEN_LENGTH {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "turnstile_token_invalid",
            "The security check token is invalid. Please try again.",
        ));
    }

    let remote_ip = client_ip(headers);
    let request = SiteverifyRequest {
        secret,
        response: token,
        remoteip: remote_ip.as_deref(),
        idempotency_key: Uuid::new_v4().to_string(),
    };
    let response = state
        .client
        .post(&state.config.turnstile_siteverify_url)
        .timeout(Duration::from_secs(8))
        .json(&request)
        .send()
        .await
        .map_err(|_| unavailable_error())?;

    if !response.status().is_success() {
        return Err(unavailable_error());
    }

    let verification = response
        .json::<SiteverifyResponse>()
        .await
        .map_err(|_| unavailable_error())?;
    validate_response(
        verification,
        (!test_mode).then_some(expected_action.label()),
        (!test_mode)
            .then(|| state.config.turnstile_expected_hostname.as_deref())
            .flatten(),
    )
}

fn validate_response(
    response: SiteverifyResponse,
    expected_action: Option<&str>,
    expected_hostname: Option<&str>,
) -> Result<(), ApiError> {
    if !response.success {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "turnstile_verification_failed",
            "The security check could not be verified. Please try again.",
        ));
    }

    if expected_action.is_some_and(|expected| response.action.as_deref() != Some(expected)) {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "turnstile_action_mismatch",
            "The security check was issued for a different action. Please try again.",
        ));
    }

    if let Some(expected) = expected_hostname
        && !response
            .hostname
            .as_deref()
            .is_some_and(|hostname| hostname.eq_ignore_ascii_case(expected))
    {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "turnstile_hostname_mismatch",
            "The security check was issued for a different site.",
        ));
    }

    Ok(())
}

fn is_local_hostname(hostname: &str) -> bool {
    matches!(hostname, "localhost" | "127.0.0.1" | "::1")
}

fn unavailable_error() -> ApiError {
    ApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "turnstile_unavailable",
        "The security check is temporarily unavailable. Please try again shortly.",
    )
    .with_retry_after(5)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(success: bool, hostname: Option<&str>, action: Option<&str>) -> SiteverifyResponse {
        SiteverifyResponse {
            success,
            hostname: hostname.map(str::to_string),
            action: action.map(str::to_string),
        }
    }

    #[test]
    fn accepts_matching_action_and_hostname() {
        assert!(
            validate_response(
                response(
                    true,
                    Some("ryo-marketpulse.vercel.app"),
                    Some("market_analysis")
                ),
                Some("market_analysis"),
                Some("ryo-marketpulse.vercel.app"),
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_action_reuse_across_routes() {
        let error = validate_response(
            response(
                true,
                Some("ryo-marketpulse.vercel.app"),
                Some("assistant_chat"),
            ),
            Some("market_analysis"),
            Some("ryo-marketpulse.vercel.app"),
        )
        .expect_err("action mismatch");
        assert_eq!(error.code, "turnstile_action_mismatch");
    }

    #[test]
    fn rejects_tokens_from_another_hostname() {
        let error = validate_response(
            response(true, Some("example.com"), Some("market_analysis")),
            Some("market_analysis"),
            Some("ryo-marketpulse.vercel.app"),
        )
        .expect_err("hostname mismatch");
        assert_eq!(error.code, "turnstile_hostname_mismatch");
    }

    #[test]
    fn rejects_failed_verification_without_exposing_provider_detail() {
        let error = validate_response(response(false, None, None), Some("assistant_chat"), None)
            .expect_err("failure");
        assert_eq!(error.code, "turnstile_verification_failed");
        assert!(error.detail.is_none());
    }

    #[test]
    fn published_test_response_is_accepted_only_without_production_claims() {
        assert!(validate_response(response(true, Some("example.com"), None), None, None).is_ok());
        assert!(is_local_hostname("localhost"));
        assert!(!is_local_hostname("ryo-marketpulse.vercel.app"));
    }
}
