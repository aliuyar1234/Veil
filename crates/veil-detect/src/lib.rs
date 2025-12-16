//! # veil-detect
//!
//! PII detection library with regex patterns and validation.
//!
//! ## Example
//!
//! ```rust,ignore
//! use veil_detect::{DetectorRegistry, detect_pii};
//!
//! let registry = DetectorRegistry::default();
//! let findings = detect_pii("Contact: john@example.com", &registry);
//! ```

mod category;
pub mod context;
mod detector;
pub mod dictionary;
mod error;
mod finding;
mod patterns;
mod registry;
mod validators;

pub use category::PiiCategory;
pub use context::{ContextAnalysis, ContextAnalyzer, ContextError};
pub use detector::{Detector, Match};
pub use error::DetectError;
pub use finding::{Finding, ValidationStatus};
pub use registry::DetectorRegistry;

use veil_parsers::TextSegment;

/// Detect PII in text segments using the given registry.
pub fn detect_pii(segments: &[TextSegment], registry: &DetectorRegistry) -> Vec<Finding> {
    registry.detect_all(segments)
}

/// Detect PII in a single string using the default registry.
pub fn detect_in_text(text: &str) -> Vec<Finding> {
    let segment = TextSegment {
        content: text.to_string(),
        position: veil_parsers::Position::Text {
            line: 1,
            column: 1,
            byte_offset: 0,
            byte_length: text.len(),
        },
    };
    let registry = DetectorRegistry::default();
    registry.detect_all(&[segment])
}
