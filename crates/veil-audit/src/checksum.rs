//! Checksum calculation and verification.

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::entry::AuditEntry;
use crate::error::AuditError;

const CHECKSUM_SHA256_PREFIX: &str = "sha256:";
const CHECKSUM_HMAC_SHA256_PREFIX: &str = "hmac-sha256:";
const AUDIT_HMAC_KEY_ENV: &str = "VEIL_AUDIT_HMAC_KEY_HEX";

type HmacSha256 = Hmac<Sha256>;

/// Calculate a legacy (unkeyed) SHA-256 checksum for an audit entry.
///
/// This format is retained for backwards compatibility, but does not provide
/// tamper resistance against an attacker who can rewrite log entries.
pub fn calculate_checksum_legacy(entry: &AuditEntry) -> String {
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

/// Calculate unkeyed SHA-256 checksum for an audit entry (prefixed).
pub fn calculate_checksum(entry: &AuditEntry) -> String {
    format!(
        "{}{}",
        CHECKSUM_SHA256_PREFIX,
        calculate_checksum_legacy(entry)
    )
}

/// Calculate HMAC-SHA-256 checksum for an audit entry (prefixed).
///
/// This provides tamper evidence as long as the HMAC key remains secret.
pub fn calculate_checksum_hmac(entry: &AuditEntry, key: &[u8]) -> Result<String, AuditError> {
    if key.len() != 32 {
        return Err(AuditError::InvalidKey {
            expected: 32,
            actual: key.len(),
        });
    }

    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts keys of arbitrary size");

    // MAC the entry fields (excluding checksum)
    mac.update(entry.id.to_string().as_bytes());
    mac.update(entry.timestamp.to_rfc3339().as_bytes());
    mac.update(entry.operation.to_string().as_bytes());

    // MAC parameters
    if let Ok(params_json) = serde_json::to_string(&entry.parameters) {
        mac.update(params_json.as_bytes());
    }

    // MAC outcome
    if let Ok(outcome_json) = serde_json::to_string(&entry.outcome) {
        mac.update(outcome_json.as_bytes());
    }

    // Include previous checksum in the MAC (hash chain)
    if let Some(ref prev) = entry.previous_checksum {
        mac.update(prev.as_bytes());
    }

    let digest = mac.finalize().into_bytes();
    Ok(format!(
        "{}{}",
        CHECKSUM_HMAC_SHA256_PREFIX,
        hex::encode(digest)
    ))
}

fn hmac_key_from_env() -> Result<Vec<u8>, AuditError> {
    let value = std::env::var(AUDIT_HMAC_KEY_ENV).map_err(|_| AuditError::MissingIntegrityKey)?;
    let bytes = hex::decode(value.trim()).map_err(|_| AuditError::MissingIntegrityKey)?;
    if bytes.len() != 32 {
        return Err(AuditError::InvalidKey {
            expected: 32,
            actual: bytes.len(),
        });
    }
    Ok(bytes)
}

/// Verify the hash chain of a sequence of entries.
///
/// If any entry uses HMAC (`hmac-sha256:` prefix), the key is read from `VEIL_AUDIT_HMAC_KEY_HEX`.
pub fn verify_chain(entries: &[AuditEntry]) -> Result<(), AuditError> {
    let key = hmac_key_from_env().ok().map(Zeroizing::new);
    verify_chain_with_key(entries, key.as_ref().map(|k| k.as_slice()))
}

/// Verify the hash chain of a sequence of entries, using an explicit HMAC key when needed.
pub fn verify_chain_with_key(
    entries: &[AuditEntry],
    hmac_key: Option<&[u8]>,
) -> Result<(), AuditError> {
    let mut previous_checksum: Option<String> = None;

    for entry in entries {
        // Verify previous checksum matches
        if entry.previous_checksum != previous_checksum {
            return Err(AuditError::ChainBroken(entry.id.to_string()));
        }

        // Verify entry checksum
        let calculated = if entry.checksum.starts_with(CHECKSUM_HMAC_SHA256_PREFIX) {
            let key = hmac_key.ok_or(AuditError::MissingIntegrityKey)?;
            calculate_checksum_hmac(entry, key)?
        } else if entry.checksum.starts_with(CHECKSUM_SHA256_PREFIX) {
            calculate_checksum(entry)
        } else {
            // Legacy, unprefixed checksum
            calculate_checksum_legacy(entry)
        };

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

        assert!(checksum.starts_with(CHECKSUM_SHA256_PREFIX));
        let digest = checksum.trim_start_matches(CHECKSUM_SHA256_PREFIX);

        // SHA-256 produces 64 hex characters
        assert_eq!(digest.len(), 64);
        // Should be all lowercase hex characters
        assert!(digest
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
    }

    #[test]
    fn test_hmac_checksum_format() {
        let entry = AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());
        let checksum = calculate_checksum_hmac(&entry, &[0u8; 32]).unwrap();

        assert!(checksum.starts_with(CHECKSUM_HMAC_SHA256_PREFIX));
        let digest = checksum.trim_start_matches(CHECKSUM_HMAC_SHA256_PREFIX);
        assert_eq!(digest.len(), 64);
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

    #[test]
    fn test_verify_chain_hmac_requires_key() {
        let mut entry =
            AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());
        entry.checksum = calculate_checksum_hmac(&entry, &[0u8; 32]).unwrap();

        let err = verify_chain_with_key(&[entry], None).unwrap_err();
        assert!(matches!(err, AuditError::MissingIntegrityKey));
    }
}
