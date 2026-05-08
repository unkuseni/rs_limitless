//! Error types for the Limitless Exchange API client.
//!
//! `LimitlessContentError` captures error details returned by the API itself.
//! `LimitlessError` is the top-level error enum covering API errors, network
//! failures, serialization issues, and validation errors.

use crate::prelude::*;

/// Represents an error returned by the Limitless API in the response body.
///
/// The `message` field contains the human-readable error description.
/// The `code` field (if present) contains a machine-readable error code.
#[derive(Debug, Deserialize, Display)]
#[display("{}", message)]
pub struct LimitlessContentError {
    /// Human-readable error message from the API.
    pub message: String,
    /// Optional machine-readable error code.
    #[serde(default)]
    pub code: Option<String>,
}

/// Top-level error type covering all possible failures when interacting
/// with the Limitless Exchange API.
#[derive(Debug, Error)]
pub enum LimitlessError {
    /// The Limitless API returned an error response (4xx/5xx with a body).
    #[error("Limitless API error: {0}")]
    ApiError(LimitlessContentError),

    /// Failed to send a value on an internal channel (WebSocket event loop).
    #[error("Failed to emit value on channel: {underlying}")]
    ChannelSendError { underlying: String },

    /// Request parameters failed client-side validation before being sent.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Reqwest HTTP client error (network, DNS, TLS, timeout).
    #[error(transparent)]
    ReqError(#[from] reqwest::Error),

    /// Invalid HTTP header value provided.
    #[error(transparent)]
    InvalidHeaderError(#[from] reqwest::header::InvalidHeaderValue),

    /// Standard I/O error.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Failed to parse a string as a floating-point number.
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    /// URL parsing failure.
    #[error(transparent)]
    UrlParserError(#[from] url::ParseError),

    /// JSON serialization/deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// WebSocket protocol / transport error.
    #[error(transparent)]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error),

    /// System time error (clock may be before Unix epoch).
    #[error(transparent)]
    TimestampError(#[from] std::time::SystemTimeError),

    /// Generic serde deserialization error.
    #[error(transparent)]
    SerdeError(#[from] serde::de::value::Error),

    /// The server returned 500 Internal Server Error.
    #[error("Internal Server Error")]
    InternalServerError,

    /// The server returned 503 Service Unavailable.
    #[error("Service Unavailable")]
    ServiceUnavailable,

    /// The server returned 401 Unauthorized — check your API key/token.
    #[error("Unauthorized — check API key or token")]
    Unauthorized,

    /// Rate-limited (429 Too Many Requests). Retry with backoff.
    #[error("Rate limited — retry with exponential backoff")]
    RateLimited,

    /// The server returned an unexpected status code.
    #[error("Unexpected status code: {0}")]
    StatusCode(u16),

    /// A catch-all for errors that do not fit other variants.
    #[error("Limitless error: {0}")]
    Base(String),
}

impl From<String> for LimitlessError {
    fn from(err: String) -> Self {
        LimitlessError::Base(err)
    }
}
