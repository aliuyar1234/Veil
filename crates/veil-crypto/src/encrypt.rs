//! AES-256-GCM encryption and decryption.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use zeroize::Zeroizing;

use crate::error::CryptoError;
use crate::types::{CryptoMetadata, CryptoResult, OutputFormat, ProtectionMode};

/// AES-256 key size in bytes.
const AES_256_KEY_SIZE: usize = 32;

/// AES-GCM nonce size in bytes.
const AES_GCM_NONCE_SIZE: usize = 12;

/// Configuration for encryption operations.
#[derive(Clone)]
pub struct EncryptionConfig {
    /// Encryption key (must be 32 bytes for AES-256).
    key: Zeroizing<Vec<u8>>,
    /// Output encoding format.
    pub output_format: OutputFormat,
}

impl EncryptionConfig {
    /// Create a new encryption config with the given key.
    ///
    /// # Arguments
    /// * `key` - 32-byte encryption key for AES-256
    pub fn new(key: Vec<u8>) -> Self {
        Self {
            key: Zeroizing::new(key),
            output_format: OutputFormat::Base64,
        }
    }

    /// Create a new encryption config with custom output format.
    pub fn with_format(key: Vec<u8>, output_format: OutputFormat) -> Self {
        Self {
            key: Zeroizing::new(key),
            output_format,
        }
    }

    /// Get the key reference.
    pub fn key(&self) -> &[u8] {
        self.key.as_slice()
    }
}

/// Encrypt data using AES-256-GCM.
///
/// # Arguments
/// * `plaintext` - Data to encrypt
/// * `config` - Encryption configuration
///
/// # Returns
/// * `Ok(CryptoResult)` - Encrypted data with metadata
/// * `Err(CryptoError)` - If encryption fails
///
/// # Example
/// ```rust,ignore
/// use veil_crypto::{encrypt, EncryptionConfig};
///
/// let key = [0u8; 32];
/// let config = EncryptionConfig::new(key.to_vec());
/// let result = encrypt(b"secret", &config)?;
/// ```
pub fn encrypt(plaintext: &[u8], config: &EncryptionConfig) -> Result<CryptoResult, CryptoError> {
    // Validate key length
    if config.key().len() != AES_256_KEY_SIZE {
        return Err(CryptoError::InvalidKeyLength {
            expected: AES_256_KEY_SIZE,
            actual: config.key().len(),
        });
    }

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(config.key())
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // Generate random nonce
    let mut nonce_bytes = [0u8; AES_GCM_NONCE_SIZE];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // Combine nonce + ciphertext for output
    let mut combined = Vec::with_capacity(AES_GCM_NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    // Encode output
    let encoded = config.output_format.encode(&combined);

    // Build metadata
    let metadata = CryptoMetadata {
        iv: Some(config.output_format.encode(&nonce_bytes)),
        algorithm: Some("AES-256-GCM".to_string()),
        ..Default::default()
    };

    Ok(CryptoResult::with_metadata(
        encoded,
        ProtectionMode::Encrypt,
        metadata,
    ))
}

/// Decrypt data using AES-256-GCM.
///
/// # Arguments
/// * `result` - Previously encrypted data
/// * `config` - Encryption configuration (must use same key)
///
/// # Returns
/// * `Ok(Vec<u8>)` - Decrypted plaintext
/// * `Err(CryptoError)` - If decryption fails
///
/// # Example
/// ```rust,ignore
/// use veil_crypto::{encrypt, decrypt, EncryptionConfig};
///
/// let key = [0u8; 32];
/// let config = EncryptionConfig::new(key.to_vec());
/// let result = encrypt(b"secret", &config)?;
/// let plaintext = decrypt(&result, &config)?;
/// ```
pub fn decrypt(result: &CryptoResult, config: &EncryptionConfig) -> Result<Vec<u8>, CryptoError> {
    // Validate key length
    if config.key().len() != AES_256_KEY_SIZE {
        return Err(CryptoError::InvalidKeyLength {
            expected: AES_256_KEY_SIZE,
            actual: config.key().len(),
        });
    }

    // Decode ciphertext
    let combined = config
        .output_format
        .decode(&result.value)
        .map_err(CryptoError::InvalidCiphertext)?;

    // Extract nonce and ciphertext
    if combined.len() < AES_GCM_NONCE_SIZE {
        return Err(CryptoError::InvalidCiphertext(
            "Ciphertext too short".to_string(),
        ));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(AES_GCM_NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(config.key())
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // Decrypt
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptionFailed)
}

/// Decrypt raw encoded ciphertext.
pub fn decrypt_raw(encoded: &str, config: &EncryptionConfig) -> Result<Vec<u8>, CryptoError> {
    let result = CryptoResult::new(encoded.to_string(), ProtectionMode::Encrypt);
    decrypt(&result, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Vec<u8> {
        vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f,
        ]
    }

    #[test]
    fn test_encrypt_produces_base64_output() {
        let config = EncryptionConfig::new(test_key());
        let plaintext = b"Hello, World!";

        let result = encrypt(plaintext, &config).unwrap();

        assert_eq!(result.mode, ProtectionMode::Encrypt);
        // Base64 should only contain valid characters
        assert!(result
            .value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
        // Ciphertext should be different from plaintext
        assert_ne!(result.value.as_bytes(), plaintext);
    }

    #[test]
    fn test_decrypt_recovers_original_plaintext() {
        let config = EncryptionConfig::new(test_key());
        let plaintext = b"DE89370400440532013000"; // IBAN

        let encrypted = encrypt(plaintext, &config).unwrap();
        let decrypted = decrypt(&encrypted, &config).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        let config1 = EncryptionConfig::new(test_key());
        let mut wrong_key = test_key();
        wrong_key[0] = 0xff; // Change one byte
        let config2 = EncryptionConfig::new(wrong_key);

        let plaintext = b"secret data";
        let encrypted = encrypt(plaintext, &config1).unwrap();
        let result = decrypt(&encrypted, &config2);

        assert!(matches!(result, Err(CryptoError::DecryptionFailed)));
    }

    #[test]
    fn test_invalid_key_length_returns_error() {
        let short_key = vec![0u8; 16]; // AES-128 key, not AES-256
        let config = EncryptionConfig::new(short_key);

        let result = encrypt(b"test", &config);

        assert!(matches!(
            result,
            Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: 16
            })
        ));
    }

    #[test]
    fn test_tampered_ciphertext_fails_authentication() {
        let config = EncryptionConfig::new(test_key());
        let plaintext = b"authenticated data";

        let mut encrypted = encrypt(plaintext, &config).unwrap();
        // Tamper with the ciphertext
        let mut decoded = config.output_format.decode(&encrypted.value).unwrap();
        if let Some(byte) = decoded.last_mut() {
            *byte ^= 0xff; // Flip bits in last byte
        }
        encrypted.value = config.output_format.encode(&decoded);

        let result = decrypt(&encrypted, &config);
        assert!(matches!(result, Err(CryptoError::DecryptionFailed)));
    }

    #[test]
    fn test_encrypt_with_hex_format() {
        let config = EncryptionConfig::with_format(test_key(), OutputFormat::Hex);
        let plaintext = b"hex test";

        let result = encrypt(plaintext, &config).unwrap();

        // Hex should only contain 0-9a-f
        assert!(result.value.chars().all(|c| c.is_ascii_hexdigit()));

        // Should decrypt correctly
        let decrypted = decrypt(&result, &config).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_metadata_contains_iv_and_algorithm() {
        let config = EncryptionConfig::new(test_key());
        let result = encrypt(b"test", &config).unwrap();

        assert!(result.metadata.iv.is_some());
        assert_eq!(result.metadata.algorithm, Some("AES-256-GCM".to_string()));
    }

    #[test]
    fn test_empty_plaintext() {
        let config = EncryptionConfig::new(test_key());
        let plaintext = b"";

        let encrypted = encrypt(plaintext, &config).unwrap();
        let decrypted = decrypt(&encrypted, &config).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_large_plaintext() {
        let config = EncryptionConfig::new(test_key());
        let plaintext = vec![0x42u8; 1024 * 1024]; // 1MB

        let encrypted = encrypt(&plaintext, &config).unwrap();
        let decrypted = decrypt(&encrypted, &config).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_raw_round_trip() {
        let config = EncryptionConfig::new(test_key());
        let plaintext = b"decrypt_raw";

        let encrypted = encrypt(plaintext, &config).unwrap();
        let decrypted = decrypt_raw(&encrypted.value, &config).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_nonce_only_ciphertext_returns_decryption_failed() {
        let config = EncryptionConfig::new(test_key());

        // Decode yields exactly the nonce size; AES-GCM requires a tag, so decryption should fail.
        let encoded = config.output_format.encode(&[0u8; AES_GCM_NONCE_SIZE]);
        let result = CryptoResult::new(encoded, ProtectionMode::Encrypt);

        let err = decrypt(&result, &config).unwrap_err();
        assert!(matches!(err, CryptoError::DecryptionFailed));
    }
}
