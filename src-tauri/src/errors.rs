//! Application-level error types with structured error codes
//!
//! This module provides a unified error type (`AppError`) that all services
//! and commands should use for error handling.
//!
//! Error codes are serialized alongside the message, allowing frontend
//! to handle errors programmatically rather than by string pattern matching.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Application-level error codes for programmatic error handling
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Database operation failed
    Database,
    /// File system operation failed
    FileIo,
    /// Network request failed
    Network,
    /// Authentication or authorization failed
    Auth,
    /// API quota or rate limit exceeded
    Quota,
    /// Invalid input or validation failed
    Validation,
    /// Screenshot or capture operation failed
    Screenshot,
    /// Operation timed out
    Timeout,
    /// Internal error that doesn't fit other categories
    Internal,
    /// Unknown error
    #[default]
    Unknown,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::Database => write!(f, "database"),
            ErrorCode::FileIo => write!(f, "file_io"),
            ErrorCode::Network => write!(f, "network"),
            ErrorCode::Auth => write!(f, "auth"),
            ErrorCode::Quota => write!(f, "quota"),
            ErrorCode::Validation => write!(f, "validation"),
            ErrorCode::Screenshot => write!(f, "screenshot"),
            ErrorCode::Timeout => write!(f, "timeout"),
            ErrorCode::Internal => write!(f, "internal"),
            ErrorCode::Unknown => write!(f, "unknown"),
        }
    }
}

/// Unified application error type with code and message
///
/// This error type should be used by all services and commands instead of
/// returning plain `String` errors. It provides:
/// - A machine-readable error code for programmatic handling
/// - A human-readable message for debugging and logging
/// - Optional context for additional details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    /// Error code for programmatic error handling
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Additional context about the error (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl AppError {
    /// Create a new AppError with the given code and message
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: None,
        }
    }

    /// Create a new AppError with code, message, and context
    pub fn with_context(
        code: ErrorCode,
        message: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            context: Some(context.into()),
        }
    }

    /// Create an internal error with a message
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }

    /// Create a database error with a message
    pub fn database(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Database, message)
    }

    /// Create a network error with a message
    pub fn network(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Network, message)
    }

    /// Create a validation error with a message
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Validation, message)
    }

    /// Create an auth error with a message
    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Auth, message)
    }

    /// Create a quota error with a message
    pub fn quota(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Quota, message)
    }

    /// Create a file I/O error with a message
    pub fn file_io(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::FileIo, message)
    }

    /// Create a screenshot error with a message
    pub fn screenshot(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Screenshot, message)
    }

    /// Create a timeout error with a message
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Timeout, message)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;
        if let Some(ref ctx) = self.context {
            write!(f, " ({})", ctx)?;
        }
        Ok(())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self::new(ErrorCode::Unknown, s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        Self::new(ErrorCode::Unknown, s)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        Self::database(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::timeout(err.to_string())
        } else {
            // All other reqwest errors (connect, redirect, etc.) are network errors
            Self::network(err.to_string())
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::file_io(format!("file not found: {}", err)),
            std::io::ErrorKind::PermissionDenied => {
                Self::file_io(format!("permission denied: {}", err))
            }
            _ => Self::file_io(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::validation(format!("JSON error: {}", err))
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        Self::validation(format!("Chrono parse error: {}", err))
    }
}

impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::internal(format!("Lock poisoned: {}", err))
    }
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::Database.to_string(), "database");
        assert_eq!(ErrorCode::Network.to_string(), "network");
    }

    #[test]
    fn test_app_error_display() {
        let err = AppError::new(ErrorCode::Database, "connection failed");
        assert_eq!(err.to_string(), "database: connection failed");
    }

    #[test]
    fn test_app_error_with_context() {
        let err = AppError::with_context(
            ErrorCode::Network,
            "connection refused",
            "trying to reach api.example.com",
        );
        assert_eq!(
            err.to_string(),
            "network: connection refused (trying to reach api.example.com)"
        );
    }

    #[test]
    fn test_from_string() {
        let err: AppError = "some error".into();
        assert_eq!(err.code, ErrorCode::Unknown);
        assert_eq!(err.message, "some error");
    }

    #[test]
    fn test_app_error_serialization() {
        let err = AppError::new(ErrorCode::Database, "test error");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("database"));
        assert!(json.contains("test error"));
    }

    #[test]
    fn test_from_poison_error() {
        use std::sync::{Arc, Mutex};
        let lock = Arc::new(Mutex::new(42));
        // Poison the lock by panicking while holding it
        let lock_clone = lock.clone();
        let _ = std::panic::catch_unwind(|| {
            let _guard = lock_clone.lock().unwrap();
            panic!("test panic");
        });
        // Now trying to lock should give a PoisonError
        let err: AppError = lock.lock().unwrap_err().into();
        assert_eq!(err.code, ErrorCode::Internal);
        assert!(err.message.contains("Lock poisoned"));
    }

    #[test]
    fn test_app_error_to_string_conversion() {
        let err = AppError::database("connection failed");
        let s: String = err.into();
        assert_eq!(s, "database: connection failed");
    }
}
