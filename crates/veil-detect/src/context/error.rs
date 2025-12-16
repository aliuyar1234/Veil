//! Error types for context detection.

use thiserror::Error;

/// Errors that can occur during context analysis.
#[derive(Error, Debug)]
pub enum ContextError {
    /// Invalid regex pattern in context rule.
    #[error("Invalid context pattern: {0}")]
    InvalidPattern(#[from] regex::Error),

    /// Invalid context rule configuration.
    #[error("Invalid rule configuration: {0}")]
    InvalidRule(String),

    /// YAML parsing error.
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// Context analysis failed.
    #[error("Context analysis failed: {0}")]
    AnalysisFailed(String),

    /// Configuration file loading error.
    #[error("Config load error: {0}")]
    ConfigLoadError(String),

    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
