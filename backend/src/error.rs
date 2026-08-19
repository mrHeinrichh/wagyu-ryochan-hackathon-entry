//! A single error type shared across handlers and services.
//!
//! `ApiError` carries an HTTP status, a stable machine code, a human message,
//! and an optional structured detail. It implements `IntoResponse` so any
//! handler can return `Result<T, ApiError>` and get consistent JSON errors.

use axum::{
    Json,
    http::StatusCode,
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
}

impl ApiError {
    /// Create an error with a status, stable code, and message.
    pub(crate) fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            detail: None,
        }
    }

    /// Attach a structured detail payload (for example an upstream body).
    pub(crate) fn with_detail(mut self, detail: Value) -> Self {
        self.detail = Some(detail);
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
            }
        });
        (self.status, Json(body)).into_response()
    }
}
