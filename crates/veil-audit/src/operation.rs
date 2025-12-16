//! Audit operation types.

use serde::{Deserialize, Serialize};

/// Types of auditable operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOperation {
    /// File scan operation.
    Scan,
    /// File protection (redaction) operation.
    Protect,
    /// Policy validation.
    PolicyValidate,
    /// Report generation.
    ReportGenerate,
}

impl std::fmt::Display for AuditOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditOperation::Scan => write!(f, "scan"),
            AuditOperation::Protect => write!(f, "protect"),
            AuditOperation::PolicyValidate => write!(f, "policy_validate"),
            AuditOperation::ReportGenerate => write!(f, "report_generate"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_display() {
        assert_eq!(AuditOperation::Scan.to_string(), "scan");
        assert_eq!(AuditOperation::Protect.to_string(), "protect");
        assert_eq!(AuditOperation::PolicyValidate.to_string(), "policy_validate");
        assert_eq!(AuditOperation::ReportGenerate.to_string(), "report_generate");
    }

    #[test]
    fn test_operation_equality() {
        assert_eq!(AuditOperation::Scan, AuditOperation::Scan);
        assert_ne!(AuditOperation::Scan, AuditOperation::Protect);
    }

    #[test]
    fn test_operation_serialization() {
        let scan = AuditOperation::Scan;
        let json = serde_json::to_string(&scan).unwrap();
        assert_eq!(json, "\"scan\"");

        let protect = AuditOperation::Protect;
        let json = serde_json::to_string(&protect).unwrap();
        assert_eq!(json, "\"protect\"");
    }

    #[test]
    fn test_operation_deserialization() {
        let scan: AuditOperation = serde_json::from_str("\"scan\"").unwrap();
        assert_eq!(scan, AuditOperation::Scan);

        let protect: AuditOperation = serde_json::from_str("\"protect\"").unwrap();
        assert_eq!(protect, AuditOperation::Protect);
    }
}
