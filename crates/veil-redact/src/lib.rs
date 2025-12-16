//! # veil-redact
//!
//! PII redaction engine with multiple styles (labels, black bars, masking).

mod applied;
mod config;
mod engine;
mod error;
mod mask;
mod position;
mod result;
mod style;

pub use applied::AppliedRedaction;
pub use config::RedactionConfig;
pub use engine::RedactionEngine;
pub use error::RedactError;
pub use mask::MaskingRule;
pub use position::PositionMap;
pub use result::RedactionResult;
pub use style::RedactionStyle;

use veil_detect::Finding;

/// Redact findings in text using the default configuration.
pub fn redact(text: &str, findings: &[Finding]) -> RedactionResult {
    let config = RedactionConfig::default();
    RedactionEngine::new(config).redact(text, findings)
}

/// Redact findings in text using a specific style.
pub fn redact_with_style(
    text: &str,
    findings: &[Finding],
    style: RedactionStyle,
) -> RedactionResult {
    let config = RedactionConfig::with_style(style);
    RedactionEngine::new(config).redact(text, findings)
}
