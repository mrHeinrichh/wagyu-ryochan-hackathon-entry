//! A single error type shared across handlers and services.
//!
//! `ApiError` carries an HTTP status, a stable machine code, a human message,
//! and an optional structured detail. It implements `IntoResponse` so any
//! handler can return `Result<T, ApiError>` and get consistent JSON errors.

use axum::{
    Json,
    http::{HeaderValue, StatusCode, header::RETRY_AFTER},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

/// Uniform error returned by handlers and upstream service calls.
#[derive(Debug)]
pub(crate) struct ApiError {
    pub(crate) status: StatusCode,
    pub(crate) code: &'static str,
    pub(crate) message: String,
    pub(crate) detail: Option<Value>,
    pub(crate) retry_after_seconds: Option<u64>,
}

impl ApiError {
    /// Create an error with a status, stable code, and message.
    pub(crate) fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            detail: None,
            retry_after_seconds: None,
        }
    }

    /// Attach a structured detail payload (for example an upstream body).
    pub(crate) fn with_detail(mut self, detail: Value) -> Self {
        self.detail = Some(detail);
        self
    }

    pub(crate) fn with_retry_after(mut self, seconds: u64) -> Self {
        self.retry_after_seconds = Some(seconds.max(1));
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = json!({
            "error": {
                "code": self.code,
                "message": self.message,
                "detail": self.detail,
                "retry_after_seconds": self.retry_after_seconds,
            }
        });
        let mut response = (self.status, Json(body)).into_response();
        if let Some(seconds) = self.retry_after_seconds
            && let Ok(value) = HeaderValue::from_str(&seconds.to_string())
        {
            response.headers_mut().insert(RETRY_AFTER, value);
        }
        response
    }
}
