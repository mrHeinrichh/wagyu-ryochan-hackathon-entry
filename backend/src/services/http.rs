//! Shared HTTP response handling for upstream calls.

use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::error::ApiError;

/// Read an upstream response body as JSON and map non-2xx into an `ApiError`.
///
/// Non-JSON bodies are wrapped as `{ "raw": <text> }` so the detail is never
/// lost. `code` is the stable error code used when the status is not success.
pub(crate) async fn parse_response(
    response: reqwest::Response,
    code: &'static str,
) -> Result<Value, ApiError> {
    let status = response.status();
    let text = response.text().await.map_err(|error| {
        ApiError::new(
            StatusCode::BAD_GATEWAY,
            "response_read_error",
            error.to_string(),
        )
    })?;
    let value = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({ "raw": text }));
    if !status.is_success() {
        return Err(ApiError::new(status, code, "Upstream request failed.").with_detail(value));
    }
    Ok(value)
}
