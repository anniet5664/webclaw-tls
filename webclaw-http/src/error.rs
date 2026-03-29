//! Error types for webclaw-http.

use std::fmt;

/// All errors from HTTP operations.
#[derive(Debug)]
pub enum Error {
    /// Failed to build the HTTP client.
    Build(String),
    /// Network or protocol error during request.
    Request(reqwest::Error),
    /// Response body decode failure.
    BodyDecode(String),
    /// URL is invalid or missing scheme.
    InvalidUrl(url::ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(msg) => write!(f, "client build error: {msg}"),
            Self::Request(err) => write!(f, "request error: {err}"),
            Self::BodyDecode(msg) => write!(f, "body decode error: {msg}"),
            Self::InvalidUrl(err) => write!(f, "invalid URL: {err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Request(err) => Some(err),
            Self::InvalidUrl(err) => Some(err),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Request(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Self::InvalidUrl(err)
    }
}
