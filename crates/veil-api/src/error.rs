//! API error types and HTTP status code mapping.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// API error type with HTTP status code mapping.
#[derive(Debug, Error)]
pub enum ApiError {
    /// Bad request - invalid input.
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Unauthorized - missing or invalid authentication.
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden - insufficient permissions.
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Not found - resource doesn't exist.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Unsupported media type.
    #[error("Unsupported media type: {0}")]
    UnsupportedMediaType(String),

    /// Payload too large.
    #[error("Payload too large: {0}")]
    PayloadTooLarge(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Internal server error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Service unavailable.
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Parse error from veil-parsers.
    #[error("Parse error: {0}")]
    ParseError(#[from] veil_parsers::ParseError),

    /// Policy error.
    #[error("Policy error: {0}")]
    PolicyError(String),
}

/// Error response body.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error code for programmatic handling.
    pub error: String,
    /// Human-readable error message.
    pub message: String,
    /// Request ID for tracing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ApiError {
    /// Get the error code string.
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::BadRequest(_) => "bad_request",
            ApiError::Unauthorized(_) => "unauthorized",
            ApiError::Forbidden(_) => "forbidden",
            ApiError::NotFound(_) => "not_found",
            ApiError::UnsupportedMediaType(_) => "unsupported_media_type",
            ApiError::PayloadTooLarge(_) => "payload_too_large",
            ApiError::RateLimitExceeded => "rate_limit_exceeded",
            ApiError::Internal(_) => "internal_error",
            ApiError::ServiceUnavailable(_) => "service_unavailable",
            ApiError::ParseError(_) => "parse_error",
            ApiError::PolicyError(_) => "policy_error",
        }
    }

    /// Get the HTTP status code.
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ApiError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::ParseError(_) => StatusCode::BAD_REQUEST,
            ApiError::PolicyError(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ErrorResponse {
            error: self.error_code().to_string(),
            message: self.to_string(),
            request_id: None,
        };

        (status, Json(body)).into_response()
    }
}

/// Result type for API operations.
pub type ApiResult<T> = Result<T, ApiError>;
