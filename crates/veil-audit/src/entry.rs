//! Audit entry types.

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::operation::AuditOperation;
use crate::summary::{FindingsSummary, RedactionsSummary};

/// Parameters for an audit operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditParameters {
    /// Input file path(s).
    #[serde(default)]
    pub input: Vec<PathBuf>,

    /// Output file path (for protect).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<PathBuf>,

    /// Policy used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

/// Outcome of an audit operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditOutcome {
    /// Whether operation succeeded.
    pub success: bool,

    /// Error message if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Findings summary (for scan).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub findings: Option<FindingsSummary>,

    /// Redactions summary (for protect).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redactions: Option<RedactionsSummary>,
}

impl Default for AuditOutcome {
    fn default() -> Self {
        Self {
            success: true,
            error: None,
            findings: None,
            redactions: None,
        }
    }
}

impl AuditOutcome {
    /// Create a successful outcome.
    pub fn success() -> Self {
        Self::default()
    }

    /// Create a failed outcome.
    ///
    /// # Security
    ///
    /// Error messages are automatically sanitized to prevent PII leakage.
    /// Sensitive patterns (emails, paths with usernames, etc.) are redacted.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(Self::sanitize_error(error.into())),
            findings: None,
            redactions: None,
        }
    }

    /// Sanitize error messages to prevent PII leakage in audit logs.
    ///
    /// This function redacts:
    /// - Email addresses
    /// - File paths that might contain usernames
    /// - Credit card-like numbers
    /// - SSN-like patterns
    fn sanitize_error(error: String) -> String {
        use regex::Regex;
        use std::sync::LazyLock;

        // Patterns that might contain PII
        static EMAIL_RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap()
        });
        static PATH_USER_RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"(?i)(/home/|/users/|\\users\\|C:\\Users\\)[^/\\]+").unwrap()
        });
        static CREDIT_CARD_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b").unwrap());
        static SSN_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap());

        let mut result = error;
        result = EMAIL_RE
            .replace_all(&result, "[EMAIL REDACTED]")
            .to_string();
        result = PATH_USER_RE
            .replace_all(&result, "$1[USER REDACTED]")
            .to_string();
        result = CREDIT_CARD_RE
            .replace_all(&result, "[CARD REDACTED]")
            .to_string();
        result = SSN_RE.replace_all(&result, "[SSN REDACTED]").to_string();

        result
    }

    /// Add findings summary.
    pub fn with_findings(mut self, findings: FindingsSummary) -> Self {
        self.findings = Some(findings);
        self
    }

    /// Add redactions summary.
    pub fn with_redactions(mut self, redactions: RedactionsSummary) -> Self {
        self.redactions = Some(redactions);
        self
    }
}

/// A single audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique identifier.
    pub id: Uuid,

    /// When the operation occurred.
    pub timestamp: DateTime<Utc>,

    /// Type of operation.
    pub operation: AuditOperation,

    /// Operation-specific parameters.
    pub parameters: AuditParameters,

    /// Operation outcome.
    pub outcome: AuditOutcome,

    /// Checksum for tamper detection.
    pub checksum: String,

    /// Previous entry's checksum (hash chain).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_checksum: Option<String>,
}

impl AuditEntry {
    /// Create a new audit entry.
    pub fn new(
        operation: AuditOperation,
        parameters: AuditParameters,
        outcome: AuditOutcome,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            operation,
            parameters,
            outcome,
            checksum: String::new(),
            previous_checksum: None,
        }
    }

    /// Set the checksum.
    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = checksum;
        self
    }

    /// Set the previous checksum.
    pub fn with_previous_checksum(mut self, previous: Option<String>) -> Self {
        self.previous_checksum = previous;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::AuditOperation;

    #[test]
    fn test_audit_parameters_default() {
        let params = AuditParameters::default();
        assert!(params.input.is_empty());
        assert!(params.output.is_none());
        assert!(params.policy.is_none());
        assert!(params.metadata.is_empty());
    }

    #[test]
    fn test_audit_parameters_with_data() {
        let mut params = AuditParameters::default();
        params.input.push(PathBuf::from("/test/file.txt"));
        params.output = Some(PathBuf::from("/test/output.txt"));
        params.policy = Some("default".to_string());
        params
            .metadata
            .insert("user".to_string(), "test".to_string());

        assert_eq!(params.input.len(), 1);
        assert!(params.output.is_some());
        assert_eq!(params.policy.as_deref(), Some("default"));
        assert_eq!(params.metadata.get("user"), Some(&"test".to_string()));
    }

    #[test]
    fn test_audit_outcome_default() {
        let outcome = AuditOutcome::default();
        assert!(outcome.success);
        assert!(outcome.error.is_none());
        assert!(outcome.findings.is_none());
        assert!(outcome.redactions.is_none());
    }

    #[test]
    fn test_audit_outcome_success() {
        let outcome = AuditOutcome::success();
        assert!(outcome.success);
        assert!(outcome.error.is_none());
    }

    #[test]
    fn test_audit_outcome_failure() {
        let outcome = AuditOutcome::failure("test error");
        assert!(!outcome.success);
        assert_eq!(outcome.error, Some("test error".to_string()));
    }

    #[test]
    fn test_audit_outcome_with_findings() {
        let mut summary = crate::summary::FindingsSummary::new();
        summary.total = 5;

        let outcome = AuditOutcome::success().with_findings(summary);
        assert!(outcome.success);
        assert!(outcome.findings.is_some());
        assert_eq!(outcome.findings.unwrap().total, 5);
    }

    #[test]
    fn test_audit_outcome_with_redactions() {
        let mut summary = crate::summary::RedactionsSummary::new();
        summary.total = 3;

        let outcome = AuditOutcome::success().with_redactions(summary);
        assert!(outcome.success);
        assert!(outcome.redactions.is_some());
        assert_eq!(outcome.redactions.unwrap().total, 3);
    }

    #[test]
    fn test_audit_entry_new() {
        let entry = AuditEntry::new(
            AuditOperation::Scan,
            AuditParameters::default(),
            AuditOutcome::success(),
        );

        assert_eq!(entry.operation, AuditOperation::Scan);
        assert!(entry.checksum.is_empty());
        assert!(entry.previous_checksum.is_none());
    }

    #[test]
    fn test_audit_entry_with_checksum() {
        let entry = AuditEntry::new(
            AuditOperation::Protect,
            AuditParameters::default(),
            AuditOutcome::success(),
        )
        .with_checksum("abc123".to_string());

        assert_eq!(entry.checksum, "abc123");
    }

    #[test]
    fn test_audit_entry_with_previous_checksum() {
        let entry = AuditEntry::new(
            AuditOperation::Scan,
            AuditParameters::default(),
            AuditOutcome::success(),
        )
        .with_previous_checksum(Some("prev123".to_string()));

        assert_eq!(entry.previous_checksum, Some("prev123".to_string()));
    }

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry::new(
            AuditOperation::Scan,
            AuditParameters::default(),
            AuditOutcome::success(),
        );

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"operation\":\"scan\""));
        assert!(json.contains("\"id\":"));
        assert!(json.contains("\"timestamp\":"));
    }
}
