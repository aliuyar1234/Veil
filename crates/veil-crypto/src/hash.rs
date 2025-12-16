//! SHA-256/SHA-512 hashing operations.

use hmac::{Hmac, Mac};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};

use crate::error::CryptoError;
use crate::types::{CryptoMetadata, CryptoResult, OutputFormat, ProtectionMode};

/// Default salt size in bytes.
const DEFAULT_SALT_SIZE: usize = 16;

/// Supported hash algorithms.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    /// SHA-256 (256-bit output).
    #[default]
    Sha256,
    /// SHA-512 (512-bit output).
    Sha512,
}

impl HashAlgorithm {
    /// Get the algorithm name as string.
    pub fn as_str(&self) -> &'static str {
        match self {
            HashAlgorithm::Sha256 => "SHA-256",
            HashAlgorithm::Sha512 => "SHA-512",
        }
    }
}

/// Configuration for hashing operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashConfig {
    /// Hash algorithm to use.
    pub algorithm: HashAlgorithm,
    /// Salt for the hash (optional for deterministic mode).
    pub salt: Option<Vec<u8>>,
    /// Output encoding format.
    pub output_format: OutputFormat,
    /// If true, use HMAC with salt as key.
    pub keyed: bool,
}

impl Default for HashConfig {
    fn default() -> Self {
        Self {
            algorithm: HashAlgorithm::Sha256,
            salt: None,
            output_format: OutputFormat::Hex,
            keyed: false,
        }
    }
}

impl HashConfig {
    /// Create a new hash config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a hash config with SHA-256.
    pub fn sha256() -> Self {
        Self {
            algorithm: HashAlgorithm::Sha256,
            ..Default::default()
        }
    }

    /// Create a hash config with SHA-512.
    pub fn sha512() -> Self {
        Self {
            algorithm: HashAlgorithm::Sha512,
            ..Default::default()
        }
    }

    /// Set the salt.
    pub fn with_salt(mut self, salt: Vec<u8>) -> Self {
        self.salt = Some(salt);
        self
    }

    /// Enable keyed hashing (HMAC).
    pub fn keyed(mut self) -> Self {
        self.keyed = true;
        self
    }

    /// Set output format.
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }
}

/// Hash data using SHA-256 or SHA-512.
///
/// # Arguments
/// * `data` - Data to hash
/// * `config` - Hashing configuration
///
/// # Returns
/// * `Ok(CryptoResult)` - Hash output with metadata
/// * `Err(CryptoError)` - If hashing fails
///
/// # Example
/// ```rust,ignore
/// use veil_crypto::{hash, HashConfig};
///
/// let config = HashConfig::sha256();
/// let result = hash(b"user@example.com", &config)?;
/// ```
pub fn hash(data: &[u8], config: &HashConfig) -> Result<CryptoResult, CryptoError> {
    // Get or generate salt
    let salt = match &config.salt {
        Some(s) => s.clone(),
        None => {
            let mut salt = vec![0u8; DEFAULT_SALT_SIZE];
            rand::rngs::OsRng.fill_bytes(&mut salt);
            salt
        }
    };

    // Compute hash
    let hash_bytes = if config.keyed {
        // HMAC mode
        compute_hmac(data, &salt, config.algorithm)?
    } else {
        // Regular hash with salt prepended
        compute_hash(data, &salt, config.algorithm)
    };

    // Encode output
    let encoded = config.output_format.encode(&hash_bytes);

    // Build metadata
    let metadata = CryptoMetadata {
        salt: Some(config.output_format.encode(&salt)),
        algorithm: Some(config.algorithm.as_str().to_string()),
        ..Default::default()
    };

    Ok(CryptoResult::with_metadata(
        encoded,
        ProtectionMode::Hash,
        metadata,
    ))
}

/// Compute a regular hash with salt.
fn compute_hash(data: &[u8], salt: &[u8], algorithm: HashAlgorithm) -> Vec<u8> {
    match algorithm {
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(salt);
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(salt);
            hasher.update(data);
            hasher.finalize().to_vec()
        }
    }
}

/// Compute HMAC.
fn compute_hmac(data: &[u8], key: &[u8], algorithm: HashAlgorithm) -> Result<Vec<u8>, CryptoError> {
    match algorithm {
        HashAlgorithm::Sha256 => {
            type HmacSha256 = Hmac<Sha256>;
            let mut mac = HmacSha256::new_from_slice(key)
                .map_err(|e| CryptoError::HashingFailed(e.to_string()))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        HashAlgorithm::Sha512 => {
            type HmacSha512 = Hmac<Sha512>;
            let mut mac = HmacSha512::new_from_slice(key)
                .map_err(|e| CryptoError::HashingFailed(e.to_string()))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
    }
}

/// Verify if data matches a hash.
pub fn verify_hash(
    data: &[u8],
    result: &CryptoResult,
    config: &HashConfig,
) -> Result<bool, CryptoError> {
    // Need the salt from metadata
    let salt = result
        .metadata
        .salt
        .as_ref()
        .ok_or_else(|| CryptoError::HashingFailed("No salt in metadata".to_string()))?;

    let salt_bytes = config
        .output_format
        .decode(salt)
        .map_err(CryptoError::HashingFailed)?;

    // Recompute hash with same salt
    let verify_config = HashConfig {
        salt: Some(salt_bytes),
        ..config.clone()
    };

    let new_hash = hash(data, &verify_config)?;

    // Constant-time comparison
    Ok(subtle::ConstantTimeEq::ct_eq(new_hash.value.as_bytes(), result.value.as_bytes()).into())
}

/// Generate a random salt.
pub fn generate_salt(size: usize) -> Vec<u8> {
    let mut salt = vec![0u8; size];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash_produces_hex_output() {
        let config = HashConfig::sha256().with_salt(b"test-salt".to_vec());
        let data = b"user@example.com";

        let result = hash(data, &config).unwrap();

        assert_eq!(result.mode, ProtectionMode::Hash);
        // SHA-256 produces 64 hex characters
        assert_eq!(result.value.len(), 64);
        // Should only contain hex characters
        assert!(result.value.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha512_hash_produces_hex_output() {
        let config = HashConfig::sha512().with_salt(b"test-salt".to_vec());
        let data = b"user@example.com";

        let result = hash(data, &config).unwrap();

        // SHA-512 produces 128 hex characters
        assert_eq!(result.value.len(), 128);
        assert!(result.value.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_same_input_salt_produces_same_hash() {
        let config = HashConfig::sha256().with_salt(b"fixed-salt".to_vec());
        let data = b"consistent-data";

        let hash1 = hash(data, &config).unwrap();
        let hash2 = hash(data, &config).unwrap();

        assert_eq!(hash1.value, hash2.value);
    }

    #[test]
    fn test_different_salt_produces_different_hash() {
        let config1 = HashConfig::sha256().with_salt(b"salt-one".to_vec());
        let config2 = HashConfig::sha256().with_salt(b"salt-two".to_vec());
        let data = b"same-data";

        let hash1 = hash(data, &config1).unwrap();
        let hash2 = hash(data, &config2).unwrap();

        assert_ne!(hash1.value, hash2.value);
    }

    #[test]
    fn test_keyed_hash_produces_consistent_output() {
        let config = HashConfig::sha256().with_salt(b"hmac-key".to_vec()).keyed();
        let data = b"authenticated-data";

        let hmac1 = hash(data, &config).unwrap();
        let hmac2 = hash(data, &config).unwrap();

        assert_eq!(hmac1.value, hmac2.value);
    }

    #[test]
    fn test_keyed_hash_different_from_regular_hash() {
        let salt = b"shared-key".to_vec();
        let config_regular = HashConfig::sha256().with_salt(salt.clone());
        let config_keyed = HashConfig::sha256().with_salt(salt).keyed();
        let data = b"test-data";

        let regular = hash(data, &config_regular).unwrap();
        let keyed = hash(data, &config_keyed).unwrap();

        assert_ne!(regular.value, keyed.value);
    }

    #[test]
    fn test_verify_hash_succeeds_for_correct_data() {
        let config = HashConfig::sha256().with_salt(b"verify-salt".to_vec());
        let data = b"verify-me";

        let result = hash(data, &config).unwrap();
        let verified = verify_hash(data, &result, &config).unwrap();

        assert!(verified);
    }

    #[test]
    fn test_verify_hash_fails_for_wrong_data() {
        let config = HashConfig::sha256().with_salt(b"verify-salt".to_vec());
        let data = b"original-data";
        let wrong_data = b"tampered-data";

        let result = hash(data, &config).unwrap();
        let verified = verify_hash(wrong_data, &result, &config).unwrap();

        assert!(!verified);
    }

    #[test]
    fn test_random_salt_when_none_provided() {
        let config = HashConfig::sha256();
        let data = b"random-salt-test";

        let hash1 = hash(data, &config).unwrap();
        let hash2 = hash(data, &config).unwrap();

        // Different random salts = different hashes
        assert_ne!(hash1.value, hash2.value);
        // But metadata should contain the salt used
        assert!(hash1.metadata.salt.is_some());
        assert!(hash2.metadata.salt.is_some());
    }

    #[test]
    fn test_metadata_contains_algorithm() {
        let config = HashConfig::sha256();
        let result = hash(b"test", &config).unwrap();

        assert_eq!(result.metadata.algorithm, Some("SHA-256".to_string()));
    }

    #[test]
    fn test_base64_output_format() {
        let config = HashConfig::sha256()
            .with_salt(b"salt".to_vec())
            .with_format(OutputFormat::Base64);
        let result = hash(b"test", &config).unwrap();

        // Base64 characters
        assert!(result
            .value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn test_empty_data() {
        let config = HashConfig::sha256().with_salt(b"salt".to_vec());
        let result = hash(b"", &config).unwrap();

        // Should still produce a valid hash
        assert_eq!(result.value.len(), 64);
    }
}
