//! Summary types for audit entries.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use veil_detect::PiiCategory;

/// Summary of findings from a scan operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FindingsSummary {
    /// Total number of findings.
    pub total: usize,
    /// Count by category.
    pub by_category: HashMap<String, usize>,
}

impl FindingsSummary {
    /// Create a new findings summary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a finding to the summary.
    pub fn add(&mut self, category: &PiiCategory) {
        self.total += 1;
        *self
            .by_category
            .entry(category.as_str().to_string())
            .or_insert(0) += 1;
    }

    /// Create from a list of findings.
    pub fn from_findings(findings: &[veil_detect::Finding]) -> Self {
        let mut summary = Self::new();
        for finding in findings {
            summary.add(&finding.category);
        }
        summary
    }
}

/// Summary of redactions from a protect operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RedactionsSummary {
    /// Total number of redactions.
    pub total: usize,
    /// Count by category.
    pub by_category: HashMap<String, usize>,
}

impl RedactionsSummary {
    /// Create a new redactions summary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a redaction to the summary.
    pub fn add(&mut self, category: &PiiCategory) {
        self.total += 1;
        *self
            .by_category
            .entry(category.as_str().to_string())
            .or_insert(0) += 1;
    }

    /// Create from applied redactions.
    pub fn from_redactions(redactions: &[veil_redact::AppliedRedaction]) -> Self {
        let mut summary = Self::new();
        for redaction in redactions {
            summary.add(&redaction.category);
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veil_detect::{Finding, ValidationStatus};
    use veil_redact::AppliedRedaction;

    #[test]
    fn test_findings_summary_add_increments_counts() {
        let mut summary = FindingsSummary::new();

        summary.add(&PiiCategory::Email);
        summary.add(&PiiCategory::Email);
        summary.add(&PiiCategory::Ssn);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.by_category.get("email"), Some(&2));
        assert_eq!(summary.by_category.get("ssn"), Some(&1));
    }

    #[test]
    fn test_findings_summary_from_findings_counts_categories() {
        let findings = vec![
            Finding::new(
                "alice@example.com",
                PiiCategory::Email,
                0,
                17,
                0.9,
                ValidationStatus::Unvalidated,
                0,
            ),
            Finding::new(
                "bob@example.com",
                PiiCategory::Email,
                0,
                15,
                0.9,
                ValidationStatus::Unvalidated,
                0,
            ),
            Finding::new(
                "123-45-6789",
                PiiCategory::Ssn,
                0,
                11,
                0.9,
                ValidationStatus::Valid,
                0,
            ),
        ];

        let summary = FindingsSummary::from_findings(&findings);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.by_category.get("email"), Some(&2));
        assert_eq!(summary.by_category.get("ssn"), Some(&1));
    }

    #[test]
    fn test_redactions_summary_from_redactions_counts_categories() {
        let redactions = vec![
            AppliedRedaction::new(
                "alice@example.com",
                "[EMAIL]",
                (0, 17),
                (0, 7),
                PiiCategory::Email,
            ),
            AppliedRedaction::new(
                "bob@example.com",
                "[EMAIL]",
                (0, 15),
                (0, 7),
                PiiCategory::Email,
            ),
            AppliedRedaction::new("123-45-6789", "[SSN]", (0, 11), (0, 5), PiiCategory::Ssn),
        ];

        let summary = RedactionsSummary::from_redactions(&redactions);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.by_category.get("email"), Some(&2));
        assert_eq!(summary.by_category.get("ssn"), Some(&1));
    }
}
