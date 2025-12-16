//! Error types for WASM bindings.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Error codes for programmatic handling in JavaScript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Invalid input data (empty, corrupted, etc.)
    InvalidInput,
    /// File format not supported
    UnsupportedFormat,
    /// File exceeds maximum size limit (50MB)
    FileTooLarge,
    /// Invalid configuration options
    InvalidConfig,
    /// Internal processing error
    InternalError,
}

impl ErrorCode {
    /// Convert to string representation for JavaScript.
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::InvalidInput => "InvalidInput",
            ErrorCode::UnsupportedFormat => "UnsupportedFormat",
            ErrorCode::FileTooLarge => "FileTooLarge",
            ErrorCode::InvalidConfig => "InvalidConfig",
            ErrorCode::InternalError => "InternalError",
        }
    }
}

/// Error type returned to JavaScript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmError {
    /// Error code for programmatic handling
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
}

impl WasmError {
    /// Create a new error.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Create an InvalidInput error.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidInput, message)
    }

    /// Create an UnsupportedFormat error.
    pub fn unsupported_format(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::UnsupportedFormat, message)
    }

    /// Create a FileTooLarge error.
    pub fn file_too_large(size: usize, max: usize) -> Self {
        Self::new(
            ErrorCode::FileTooLarge,
            format!("File size {} bytes exceeds maximum {} bytes", size, max),
        )
    }

    /// Create an InvalidConfig error.
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidConfig, message)
    }

    /// Create an InternalError.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalError, message)
    }

    /// Convert to JsValue for returning to JavaScript.
    pub fn to_js_value(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self).unwrap_or_else(|_| {
            JsValue::from_str(&format!("{}: {}", self.code.as_str(), self.message))
        })
    }
}

impl From<WasmError> for JsValue {
    fn from(err: WasmError) -> Self {
        err.to_js_value()
    }
}

impl std::fmt::Display for WasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for WasmError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_as_str() {
        assert_eq!(ErrorCode::InvalidInput.as_str(), "InvalidInput");
        assert_eq!(ErrorCode::UnsupportedFormat.as_str(), "UnsupportedFormat");
        assert_eq!(ErrorCode::FileTooLarge.as_str(), "FileTooLarge");
        assert_eq!(ErrorCode::InvalidConfig.as_str(), "InvalidConfig");
        assert_eq!(ErrorCode::InternalError.as_str(), "InternalError");
    }

    #[test]
    fn test_wasm_error_new() {
        let err = WasmError::new(ErrorCode::InvalidInput, "test message");
        assert_eq!(err.code, ErrorCode::InvalidInput);
        assert_eq!(err.message, "test message");
    }

    #[test]
    fn test_wasm_error_invalid_input() {
        let err = WasmError::invalid_input("bad data");
        assert_eq!(err.code, ErrorCode::InvalidInput);
        assert_eq!(err.message, "bad data");
    }

    #[test]
    fn test_wasm_error_unsupported_format() {
        let err = WasmError::unsupported_format("unknown format");
        assert_eq!(err.code, ErrorCode::UnsupportedFormat);
        assert_eq!(err.message, "unknown format");
    }

    #[test]
    fn test_wasm_error_file_too_large() {
        let err = WasmError::file_too_large(100_000_000, 50_000_000);
        assert_eq!(err.code, ErrorCode::FileTooLarge);
        assert!(err.message.contains("100000000"));
        assert!(err.message.contains("50000000"));
    }

    #[test]
    fn test_wasm_error_invalid_config() {
        let err = WasmError::invalid_config("bad config");
        assert_eq!(err.code, ErrorCode::InvalidConfig);
        assert_eq!(err.message, "bad config");
    }

    #[test]
    fn test_wasm_error_internal() {
        let err = WasmError::internal("internal failure");
        assert_eq!(err.code, ErrorCode::InternalError);
        assert_eq!(err.message, "internal failure");
    }

    #[test]
    fn test_wasm_error_display() {
        let err = WasmError::invalid_input("test");
        let display = format!("{}", err);
        assert_eq!(display, "InvalidInput: test");
    }

    #[test]
    fn test_error_code_equality() {
        assert_eq!(ErrorCode::InvalidInput, ErrorCode::InvalidInput);
        assert_ne!(ErrorCode::InvalidInput, ErrorCode::InternalError);
    }
}
