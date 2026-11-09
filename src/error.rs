use std::fmt;

use crate::config;
use axum::http::StatusCode;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Config(config::ConfigError),
    LockPoisoned,
    ServiceUnavailable { retry_after: std::time::Duration },
    HttpStatus { status: StatusCode },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Config(e) => write!(f, "invalid configuration: {e}"),
            Self::LockPoisoned => {
                write!(f, "ip_records lock poisoned by a panic in another thread")
            }
            Self::ServiceUnavailable { retry_after } => {
                write!(
                    f,
                    "HTTP 503 Service Unavailable, retry after {retry_after:?}"
                )
            }
            Self::HttpStatus { status } => write!(f, "HTTP status error: {status}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Config(e) => Some(e),
            Self::LockPoisoned => None,
            Self::ServiceUnavailable { .. } => None,
            Self::HttpStatus { .. } => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<config::ConfigError> for Error {
    fn from(e: config::ConfigError) -> Self {
        Self::Config(e)
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, ()>>> for Error {
    fn from(_: std::sync::PoisonError<std::sync::MutexGuard<'_, ()>>) -> Self {
        Self::LockPoisoned
    }
}

impl From<StatusCode> for Error {
    fn from(status: StatusCode) -> Self {
        Self::HttpStatus { status }
    }
}
