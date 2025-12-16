//! Detector trait and match types.

use crate::category::PiiCategory;
use crate::finding::ValidationStatus;

/// A match found by a detector.
#[derive(Debug, Clone)]
pub struct Match {
    /// Start position in the text (byte offset).
    pub start: usize,
    /// End position (exclusive) in the text (byte offset).
    pub end: usize,
    /// The matched text.
    pub text: String,
}

/// Trait for PII detectors.
pub trait Detector: Send + Sync {
    /// Unique name for this detector.
    fn name(&self) -> &str;

    /// PII category this detector finds.
    fn category(&self) -> PiiCategory;

    /// Find all matches in the given text.
    fn detect(&self, text: &str) -> Vec<Match>;

    /// Validate a potential match (checksum, format, etc.).
    fn validate(&self, matched: &str) -> ValidationStatus;

    /// Base confidence score for this detector.
    fn base_confidence(&self) -> f32;
}
