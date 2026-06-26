use thiserror::Error;

/// Errors returned by the Synapse SDK.
#[derive(Debug, Error)]
pub enum SynapseError {
    /// A structured API error returned by the server (non-2xx response).
    ///
    /// 5xx responses are transient (retryable). 4xx responses are permanent
    /// caller mistakes and are never retried.
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    /// The requested resource was not found (HTTP 404).
    #[error("not found: {0}")]
    NotFound(String),

    /// A pagination cursor was rejected as invalid or expired (HTTP 400).
    #[error("invalid cursor: {0}")]
    InvalidCursor(String),

    /// The response body could not be decoded as the expected JSON type.
    #[error("decode error: {0}")]
    Decode(String),

    /// Raw HTTP error status — used internally by the retry layer; not
    /// produced by resource methods.
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    /// A network-level failure occurred before a response was received.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
}

impl SynapseError {
    /// Returns `true` if this error may resolve on a subsequent attempt.
    ///
    /// Network errors and 5xx HTTP responses are transient. 4xx responses are
    /// permanent (they represent a caller mistake) and must not be retried.
    pub fn is_transient(&self) -> bool {
        match self {
            SynapseError::Network(_) => true,
            SynapseError::Http { status, .. } => *status >= 500,
            SynapseError::Api { status, .. } => *status >= 500,
            _ => false,
        }
    }
}
