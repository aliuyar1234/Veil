//! Checksum calculation and verification.

use sha2::{Digest, Sha256};

use crate::entry::AuditEntry;
use crate::error::AuditError;

/// Calculate SHA-256 checksum for an audit entry.
pub fn calculate_checksum(entry: &AuditEntry) -> String {
    let mut hasher = Sha256::new();

    // Hash the entry fields (excluding checksum)
    hasher.update(entry.id.to_string().as_bytes());
    hasher.update(entry.timestamp.to_rfc3339().as_bytes());
    hasher.update(entry.operation.to_string().as_bytes());

    // Hash parameters
    if let Ok(params_json) = serde_json::to_string(&entry.parameters) {
        hasher.update(params_json.as_bytes());
    }

    // Hash outcome
    if let Ok(outcome_json) = serde_json::to_string(&entry.outcome) {
        hasher.update(outcome_json.as_bytes());
    }

    // Include previous checksum in the hash (hash chain)
    if let Some(ref prev) = entry.previous_checksum {
        hasher.update(prev.as_bytes());
    }

    format!("{:x}", hasher.finalize())
}

/// Verify the hash chain of a sequence of entries.
pub fn verify_chain(entries: &[AuditEntry]) -> Result<(), AuditError> {
    let mut previous_checksum: Option<String> = None;

    for entry in entries {
        // Verify previous checksum matches
        if entry.previous_checksum != previous_checksum {
            return Err(AuditError::ChainBroken(entry.id.to_string()));
        }

        // Verify entry checksum
        let calculated = calculate_checksum(entry);
        if calculated != entry.checksum {
            return Err(AuditError::ChecksumMismatch(entry.id.to_string()));
        }

        previous_checksum = Some(entry.checksum.clone());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::AuditOperation;

    #[test]
    fn test_checksum_deterministic() {
        let entry = AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());

        let checksum1 = calculate_checksum(&entry);
        let checksum2 = calculate_checksum(&entry);

        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_checksum_changes_with_content() {
        let entry1 = AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());

        let entry2 = AuditEntry::new(
            AuditOperation::Protect,
            Default::default(),
            Default::default(),
        );

        assert_ne!(calculate_checksum(&entry1), calculate_checksum(&entry2));
    }

    #[test]
    fn test_checksum_format() {
        let entry = AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());
        let checksum = calculate_checksum(&entry);

        // SHA-256 produces 64 hex characters
        assert_eq!(checksum.len(), 64);
        // Should be all lowercase hex characters
        assert!(checksum
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
    }

    #[test]
    fn test_verify_chain_empty() {
        let entries: Vec<AuditEntry> = vec![];
        assert!(verify_chain(&entries).is_ok());
    }

    #[test]
    fn test_verify_chain_single_entry() {
        let mut entry =
            AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());
        entry.checksum = calculate_checksum(&entry);

        assert!(verify_chain(&[entry]).is_ok());
    }

    #[test]
    fn test_verify_chain_multiple_entries() {
        let mut entry1 =
            AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());
        entry1.checksum = calculate_checksum(&entry1);

        let mut entry2 = AuditEntry::new(
            AuditOperation::Protect,
            Default::default(),
            Default::default(),
        );
        entry2.previous_checksum = Some(entry1.checksum.clone());
        entry2.checksum = calculate_checksum(&entry2);

        assert!(verify_chain(&[entry1, entry2]).is_ok());
    }

    #[test]
    fn test_verify_chain_broken_chain() {
        let mut entry1 =
            AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());
        entry1.checksum = calculate_checksum(&entry1);

        let mut entry2 = AuditEntry::new(
            AuditOperation::Protect,
            Default::default(),
            Default::default(),
        );
        // Wrong previous checksum
        entry2.previous_checksum = Some("wrong_checksum".to_string());
        entry2.checksum = calculate_checksum(&entry2);

        let result = verify_chain(&[entry1, entry2]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuditError::ChainBroken(_)));
    }

    #[test]
    fn test_verify_chain_tampered_entry() {
        let mut entry =
            AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());
        entry.checksum = calculate_checksum(&entry);

        // Tamper with the entry after checksum
        entry.outcome.success = false;

        let result = verify_chain(&[entry]);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuditError::ChecksumMismatch(_)
        ));
    }

    #[test]
    fn test_checksum_includes_previous() {
        let mut entry1 =
            AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());
        entry1.checksum = calculate_checksum(&entry1);

        let mut entry2a = AuditEntry::new(
            AuditOperation::Protect,
            Default::default(),
            Default::default(),
        );
        entry2a.previous_checksum = Some(entry1.checksum.clone());

        let mut entry2b = AuditEntry::new(
            AuditOperation::Protect,
            Default::default(),
            Default::default(),
        );
        entry2b.previous_checksum = Some("different_previous".to_string());

        // Same entry content but different previous checksums should yield different checksums
        assert_ne!(calculate_checksum(&entry2a), calculate_checksum(&entry2b));
    }
}
