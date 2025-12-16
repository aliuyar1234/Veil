//! Shared types for cryptographic operations.

use serde::{Deserialize, Serialize};

/// Protection strategy for PII data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtectionMode {
    /// AES-256-GCM encryption (reversible with key).
    Encrypt,
    /// SHA-256/SHA-512 hashing (irreversible).
    Hash,
    /// Replace with realistic fake data.
    Pseudonymize,
    /// Replace with random token (reversible via vault).
    Tokenize,
}

/// Output encoding format for binary data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Base64 encoding.
    #[default]
    Base64,
    /// Hexadecimal encoding.
    Hex,
}

impl OutputFormat {
    /// Encode bytes to string using this format.
    pub fn encode(&self, bytes: &[u8]) -> String {
        match self {
            OutputFormat::Base64 => {
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
            }
            OutputFormat::Hex => hex_encode(bytes),
        }
    }

    /// Decode string to bytes using this format.
    pub fn decode(&self, s: &str) -> Result<Vec<u8>, String> {
        match self {
            OutputFormat::Base64 => {
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)
                    .map_err(|e| format!("Invalid base64: {}", e))
            }
            OutputFormat::Hex => hex_decode(s),
        }
    }
}

/// Result of a cryptographic protection operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoResult {
    /// The protected value.
    pub value: String,
    /// Protection mode used.
    pub mode: ProtectionMode,
    /// Additional metadata.
    pub metadata: CryptoMetadata,
}

impl CryptoResult {
    /// Create a new crypto result.
    pub fn new(value: String, mode: ProtectionMode) -> Self {
        Self {
            value,
            mode,
            metadata: CryptoMetadata::default(),
        }
    }

    /// Create a new crypto result with metadata.
    pub fn with_metadata(value: String, mode: ProtectionMode, metadata: CryptoMetadata) -> Self {
        Self {
            value,
            mode,
            metadata,
        }
    }
}

/// Metadata about the cryptographic operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CryptoMetadata {
    /// Initialization vector (for encryption).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iv: Option<String>,
    /// Authentication tag (for encryption).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Algorithm used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    /// Salt used (for hashing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
    /// Original data type (for pseudonymization).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<String>,
}

/// Encode bytes to hex string.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Decode hex string to bytes.
fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("Invalid hex: odd length".to_string());
    }

    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("Invalid hex: {}", e)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_base64() {
        let data = b"Hello, World!";
        let encoded = OutputFormat::Base64.encode(data);
        let decoded = OutputFormat::Base64.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_output_format_hex() {
        let data = b"Hello";
        let encoded = OutputFormat::Hex.encode(data);
        assert_eq!(encoded, "48656c6c6f");
        let decoded = OutputFormat::Hex.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_crypto_result_new() {
        let result = CryptoResult::new("encrypted".to_string(), ProtectionMode::Encrypt);
        assert_eq!(result.value, "encrypted");
        assert_eq!(result.mode, ProtectionMode::Encrypt);
    }
}
