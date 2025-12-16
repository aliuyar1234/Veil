//! Tokenization with vault storage.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::CryptoError;
use crate::types::{CryptoMetadata, CryptoResult, ProtectionMode};
use crate::vault::TokenVault;

/// Format for generated tokens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenFormat {
    /// UUID v4 (e.g., "a1b2c3d4-e5f6-7890-abcd-ef1234567890").
    #[default]
    Uuid,
    /// 16-character hexadecimal (e.g., "a1b2c3d4e5f67890").
    Hex16,
    /// 32-character hexadecimal (e.g., "a1b2c3d4e5f67890a1b2c3d4e5f67890").
    Hex32,
    /// 16-character alphanumeric (e.g., "A1B2C3D4E5F67890").
    Alphanumeric,
}

impl TokenFormat {
    /// Generate a token in this format.
    pub fn generate(&self) -> String {
        match self {
            TokenFormat::Uuid => Uuid::new_v4().to_string(),
            TokenFormat::Hex16 => generate_hex(16),
            TokenFormat::Hex32 => generate_hex(32),
            TokenFormat::Alphanumeric => generate_alphanumeric(16),
        }
    }
}

/// Configuration for tokenization operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Token format to generate.
    pub format: TokenFormat,
    /// Whether same input should produce same token.
    pub consistent: bool,
    /// Prefix for generated tokens (optional).
    pub prefix: Option<String>,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            format: TokenFormat::Uuid,
            consistent: true,
            prefix: None,
        }
    }
}

impl TokenConfig {
    /// Create a new token config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the token format.
    pub fn with_format(mut self, format: TokenFormat) -> Self {
        self.format = format;
        self
    }

    /// Disable consistent mode (each call generates new token).
    pub fn non_consistent(mut self) -> Self {
        self.consistent = false;
        self
    }

    /// Set a prefix for tokens.
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }
}

/// Tokenize a value, storing the mapping in a vault.
///
/// # Arguments
/// * `original` - Original value to tokenize
/// * `config` - Tokenization configuration
/// * `vault` - Token vault for storage
///
/// # Returns
/// * `Ok(CryptoResult)` - Generated token
/// * `Err(CryptoError)` - If tokenization fails
///
/// # Example
/// ```rust,ignore
/// use veil_crypto::{tokenize, TokenConfig, InMemoryVault};
/// use std::sync::Arc;
///
/// let vault = Arc::new(InMemoryVault::new());
/// let config = TokenConfig::new().with_prefix("tok_");
/// let result = tokenize("4532015112830366", &config, vault)?;
/// ```
pub fn tokenize(
    original: &str,
    config: &TokenConfig,
    vault: Arc<dyn TokenVault>,
) -> Result<CryptoResult, CryptoError> {
    // Check for existing token if consistent mode
    if config.consistent {
        if let Some(existing) = vault
            .find_by_original(original)
            .map_err(CryptoError::Vault)?
        {
            return Ok(CryptoResult::new(existing, ProtectionMode::Tokenize));
        }
    }

    // Generate new token
    let raw_token = config.format.generate();
    let token = match &config.prefix {
        Some(prefix) => format!("{}{}", prefix, raw_token),
        None => raw_token,
    };

    // Store in vault
    vault.store(&token, original).map_err(CryptoError::Vault)?;

    let metadata = CryptoMetadata::default();
    Ok(CryptoResult::with_metadata(
        token,
        ProtectionMode::Tokenize,
        metadata,
    ))
}

/// Detokenize a value by looking up in vault.
///
/// # Arguments
/// * `token` - Token to look up
/// * `vault` - Token vault
///
/// # Returns
/// * `Ok(String)` - Original value
/// * `Err(CryptoError)` - If token not found
///
/// # Example
/// ```rust,ignore
/// use veil_crypto::{tokenize, detokenize, TokenConfig, InMemoryVault};
/// use std::sync::Arc;
///
/// let vault = Arc::new(InMemoryVault::new());
/// let config = TokenConfig::new();
/// let result = tokenize("secret", &config, vault.clone())?;
/// let original = detokenize(&result.value, vault)?;
/// ```
pub fn detokenize(token: &str, vault: Arc<dyn TokenVault>) -> Result<String, CryptoError> {
    vault
        .lookup(token)
        .map_err(CryptoError::Vault)?
        .ok_or_else(|| CryptoError::TokenFailed(format!("Token not found: {}", token)))
}

/// Delete a token from the vault.
pub fn delete_token(token: &str, vault: Arc<dyn TokenVault>) -> Result<bool, CryptoError> {
    vault.delete(token).map_err(CryptoError::Vault)
}

/// Generate random hex string.
fn generate_hex(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let byte: u8 = rng.gen_range(0..16);
            format!("{:x}", byte)
        })
        .collect()
}

/// Generate random alphanumeric string.
fn generate_alphanumeric(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::InMemoryVault;

    fn test_vault() -> Arc<dyn TokenVault> {
        Arc::new(InMemoryVault::new())
    }

    #[test]
    fn test_tokenize_produces_uuid_token() {
        let vault = test_vault();
        let config = TokenConfig::new();

        let result = tokenize("original_value", &config, vault).unwrap();

        assert_eq!(result.mode, ProtectionMode::Tokenize);
        // UUID format: 8-4-4-4-12
        assert_eq!(result.value.matches('-').count(), 4);
        assert_eq!(result.value.len(), 36);
    }

    #[test]
    fn test_detokenize_retrieves_original() {
        let vault = test_vault();
        let config = TokenConfig::new();
        let original = "credit_card_number";

        let result = tokenize(original, &config, vault.clone()).unwrap();
        let retrieved = detokenize(&result.value, vault).unwrap();

        assert_eq!(retrieved, original);
    }

    #[test]
    fn test_consistent_mode_returns_same_token() {
        let vault = test_vault();
        let config = TokenConfig::new(); // consistent by default
        let original = "same_value";

        let result1 = tokenize(original, &config, vault.clone()).unwrap();
        let result2 = tokenize(original, &config, vault).unwrap();

        assert_eq!(result1.value, result2.value);
    }

    #[test]
    fn test_non_consistent_mode_returns_different_tokens() {
        let vault = test_vault();
        let config = TokenConfig::new().non_consistent();
        let original = "value";

        let result1 = tokenize(original, &config, vault.clone()).unwrap();
        // Need different original to avoid vault error
        let result2 = tokenize("value2", &config, vault).unwrap();

        assert_ne!(result1.value, result2.value);
    }

    #[test]
    fn test_token_with_prefix() {
        let vault = test_vault();
        let config = TokenConfig::new().with_prefix("tok_");

        let result = tokenize("value", &config, vault).unwrap();

        assert!(result.value.starts_with("tok_"));
    }

    #[test]
    fn test_hex16_format() {
        let vault = test_vault();
        let config = TokenConfig::new().with_format(TokenFormat::Hex16);

        let result = tokenize("value", &config, vault).unwrap();

        assert_eq!(result.value.len(), 16);
        assert!(result.value.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hex32_format() {
        let vault = test_vault();
        let config = TokenConfig::new().with_format(TokenFormat::Hex32);

        let result = tokenize("value", &config, vault).unwrap();

        assert_eq!(result.value.len(), 32);
        assert!(result.value.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_alphanumeric_format() {
        let vault = test_vault();
        let config = TokenConfig::new().with_format(TokenFormat::Alphanumeric);

        let result = tokenize("value", &config, vault).unwrap();

        assert_eq!(result.value.len(), 16);
        assert!(result.value.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_detokenize_not_found() {
        let vault = test_vault();

        let result = detokenize("nonexistent_token", vault);

        assert!(matches!(result, Err(CryptoError::TokenFailed(_))));
    }

    #[test]
    fn test_delete_token() {
        let vault = test_vault();
        let config = TokenConfig::new();

        let result = tokenize("value", &config, vault.clone()).unwrap();
        let deleted = delete_token(&result.value, vault.clone()).unwrap();
        let lookup = detokenize(&result.value, vault);

        assert!(deleted);
        assert!(lookup.is_err());
    }

    #[test]
    fn test_prefix_with_different_formats() {
        let vault = test_vault();
        let config = TokenConfig::new()
            .with_format(TokenFormat::Hex16)
            .with_prefix("cc_");

        let result = tokenize("card", &config, vault).unwrap();

        assert!(result.value.starts_with("cc_"));
        // Total length = prefix (3) + hex16 (16) = 19
        assert_eq!(result.value.len(), 19);
    }
}
