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
