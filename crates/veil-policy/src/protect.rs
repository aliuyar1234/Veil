//! Protection dispatch to veil-crypto and veil-redact.

use std::sync::Arc;

use veil_crypto::{
    encrypt, hash, pseudonymize, tokenize, EncryptionConfig, HashConfig, InMemoryVault,
    PseudonymConfig, PseudonymDataType, TokenConfig, TokenVault,
};
use veil_redact::MaskingRule;

/// Character for black bar redaction.
const BLACK_BAR_CHAR: char = '█';

use crate::error::PolicyError;
use crate::keyref::KeyRef;
use crate::rules::ProtectionAction;

/// Result of protecting a single value.
#[derive(Debug, Clone)]
pub struct ProtectedValue {
    /// The protected string.
    pub value: String,
    /// Action that was applied.
    pub action: ProtectionAction,
    /// Whether this was a cached result (consistent mode).
    pub cached: bool,
}

impl ProtectedValue {
    /// Create a new protected value.
    pub fn new(value: String, action: ProtectionAction) -> Self {
        Self {
            value,
            action,
            cached: false,
        }
    }

    /// Create a cached protected value.
    pub fn cached(value: String, action: ProtectionAction) -> Self {
        Self {
            value,
            action,
            cached: true,
        }
    }
}

/// Context for protection operations.
pub struct ProtectionContext {
    /// Resolved encryption key (if needed).
    pub encryption_key: Option<Vec<u8>>,
    /// Token vault for tokenization.
    pub vault: Arc<dyn TokenVault>,
    /// Locale for pseudonymization.
    pub locale: String,
    /// Whether to use consistent pseudonymization.
    pub consistent: bool,
    /// Seed for consistent pseudonymization.
    pub seed: Option<u64>,
}

impl Default for ProtectionContext {
    fn default() -> Self {
        Self {
            encryption_key: None,
            vault: Arc::new(InMemoryVault::new()),
            locale: "en_US".to_string(),
            consistent: false,
            seed: None,
        }
    }
}

impl ProtectionContext {
    /// Create a new context with encryption key.
    pub fn with_key(mut self, key: Vec<u8>) -> Self {
        self.encryption_key = Some(key);
        self
    }

    /// Set optional encryption key.
    pub fn with_optional_key(mut self, key: Option<Vec<u8>>) -> Self {
        self.encryption_key = key;
        self
    }

    /// Set token vault.
    pub fn with_vault(mut self, vault: Arc<dyn TokenVault>) -> Self {
        self.vault = vault;
        self
    }

    /// Set locale.
    pub fn with_locale(mut self, locale: &str) -> Self {
        self.locale = locale.to_string();
        self
    }

    /// Enable consistent pseudonymization.
    pub fn consistent(mut self, seed: Option<u64>) -> Self {
        self.consistent = true;
        self.seed = seed;
        self
    }
}

/// Protect a value according to the specified action.
pub fn protect_value(
    value: &str,
    action: &ProtectionAction,
    style: Option<&str>,
    key_ref: Option<&KeyRef>,
    ctx: &ProtectionContext,
) -> Result<ProtectedValue, PolicyError> {
    match action {
        ProtectionAction::Redact => protect_redact(value, style),
        ProtectionAction::Mask => protect_mask(value, style),
        ProtectionAction::Hash => protect_hash(value),
        ProtectionAction::Encrypt => protect_encrypt(value, key_ref, ctx),
        ProtectionAction::Pseudonymize => protect_pseudonymize(value, ctx),
        ProtectionAction::Tokenize => protect_tokenize(value, ctx),
    }
}

/// Redact a value with a label or custom style.
fn protect_redact(value: &str, style: Option<&str>) -> Result<ProtectedValue, PolicyError> {
    let redacted = match style {
        Some("bar") | Some("black_bar") => {
            // Create black bar of same length as value
            BLACK_BAR_CHAR.to_string().repeat(value.chars().count())
        }
        Some("mask") => {
            // Use default masking rule for partial visibility
            MaskingRule::default().apply(value)
        }
        None | Some("label") => "[REDACTED]".to_string(),
        Some(custom) => format!("[{}]", custom.to_uppercase()),
    };

    Ok(ProtectedValue::new(redacted, ProtectionAction::Redact))
}

/// Mask a value with partial visibility.
fn protect_mask(value: &str, _style: Option<&str>) -> Result<ProtectedValue, PolicyError> {
    let masked = MaskingRule::default().apply(value);
    Ok(ProtectedValue::new(masked, ProtectionAction::Mask))
}

/// Hash a value using SHA-256.
fn protect_hash(value: &str) -> Result<ProtectedValue, PolicyError> {
    let config = HashConfig::sha256();
    let result =
        hash(value.as_bytes(), &config).map_err(|e| PolicyError::CryptoError(e.to_string()))?;
    Ok(ProtectedValue::new(result.value, ProtectionAction::Hash))
}

/// Encrypt a value using AES-256-GCM.
fn protect_encrypt(
    value: &str,
    key_ref: Option<&KeyRef>,
    ctx: &ProtectionContext,
) -> Result<ProtectedValue, PolicyError> {
    // Get key from context or resolve from reference
    let key = match (&ctx.encryption_key, key_ref) {
        (Some(key), _) => key.clone(),
        (None, Some(kref)) => kref.resolve().map_err(PolicyError::KeyRefError)?,
        (None, None) => {
            return Err(PolicyError::MissingKey(
                "Encryption requires a key reference".to_string(),
            ))
        }
    };

    // Validate key length
    if key.len() != 32 {
        return Err(PolicyError::InvalidKey(format!(
            "AES-256 requires 32-byte key, got {}",
            key.len()
        )));
    }

    let config = EncryptionConfig::new(key);
    let result =
        encrypt(value.as_bytes(), &config).map_err(|e| PolicyError::CryptoError(e.to_string()))?;
    Ok(ProtectedValue::new(result.value, ProtectionAction::Encrypt))
}

/// Pseudonymize a value with fake data.
fn protect_pseudonymize(
    value: &str,
    ctx: &ProtectionContext,
) -> Result<ProtectedValue, PolicyError> {
    let mut config = PseudonymConfig::new()
        .with_locale(&ctx.locale)
        .with_type(PseudonymDataType::Name);

    if ctx.consistent {
        config = if let Some(seed) = ctx.seed {
            config.consistent_with_seed(seed)
        } else {
            config.consistent()
        };
    }

    let result =
        pseudonymize(value, &config).map_err(|e| PolicyError::CryptoError(e.to_string()))?;
    Ok(ProtectedValue::new(
        result.value,
        ProtectionAction::Pseudonymize,
    ))
}

/// Tokenize a value with vault storage.
fn protect_tokenize(value: &str, ctx: &ProtectionContext) -> Result<ProtectedValue, PolicyError> {
    let config = TokenConfig::new().with_prefix("tok_");
    let result = tokenize(value, &config, ctx.vault.clone())
        .map_err(|e| PolicyError::CryptoError(e.to_string()))?;
    Ok(ProtectedValue::new(
        result.value,
        ProtectionAction::Tokenize,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_ctx() -> ProtectionContext {
        ProtectionContext::default()
    }

    #[test]
    fn test_protect_redact_label() {
        let result = protect_value(
            "secret",
            &ProtectionAction::Redact,
            Some("label"),
            None,
            &default_ctx(),
        )
        .unwrap();
        assert_eq!(result.value, "[REDACTED]");
        assert_eq!(result.action, ProtectionAction::Redact);
    }

    #[test]
    fn test_protect_redact_custom() {
        let result = protect_value(
            "email@example.com",
            &ProtectionAction::Redact,
            Some("email"),
            None,
            &default_ctx(),
        )
        .unwrap();
        assert_eq!(result.value, "[EMAIL]");
    }

    #[test]
    fn test_protect_mask() {
        let result = protect_value(
            "test@example.com",
            &ProtectionAction::Mask,
            None,
            None,
            &default_ctx(),
        )
        .unwrap();
        assert_ne!(result.value, "test@example.com");
        assert!(result.value.contains('*'));
        assert_eq!(result.action, ProtectionAction::Mask);
    }

    #[test]
    fn test_protect_hash() {
        let result = protect_value(
            "to-hash",
            &ProtectionAction::Hash,
            None,
            None,
            &default_ctx(),
        )
        .unwrap();
        // SHA-256 hex = 64 chars
        assert_eq!(result.value.len(), 64);
        assert!(result.value.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(result.action, ProtectionAction::Hash);
    }

    #[test]
    fn test_protect_encrypt() {
        let key = vec![0u8; 32]; // Test key
        let ctx = ProtectionContext::default().with_key(key);
        let result =
            protect_value("sensitive", &ProtectionAction::Encrypt, None, None, &ctx).unwrap();
        assert_ne!(result.value, "sensitive");
        assert_eq!(result.action, ProtectionAction::Encrypt);
    }

    #[test]
    fn test_protect_encrypt_missing_key() {
        let result = protect_value(
            "sensitive",
            &ProtectionAction::Encrypt,
            None,
            None,
            &default_ctx(),
        );
        assert!(matches!(result, Err(PolicyError::MissingKey(_))));
    }

    #[test]
    fn test_protect_pseudonymize() {
        let ctx = ProtectionContext::default().with_locale("en_US");
        let result = protect_value(
            "John Doe",
            &ProtectionAction::Pseudonymize,
            None,
            None,
            &ctx,
        )
        .unwrap();
        assert_ne!(result.value, "John Doe");
        assert_eq!(result.action, ProtectionAction::Pseudonymize);
    }

    #[test]
    fn test_protect_tokenize() {
        let result = protect_value(
            "credit-card-123",
            &ProtectionAction::Tokenize,
            None,
            None,
            &default_ctx(),
        )
        .unwrap();
        assert!(result.value.starts_with("tok_"));
        assert_eq!(result.action, ProtectionAction::Tokenize);
    }

    #[test]
    fn test_consistent_pseudonymize() {
        let ctx = ProtectionContext::default()
            .with_locale("en_US")
            .consistent(Some(12345));

        let result1 = protect_value(
            "Same Name",
            &ProtectionAction::Pseudonymize,
            None,
            None,
            &ctx,
        )
        .unwrap();
        let result2 = protect_value(
            "Same Name",
            &ProtectionAction::Pseudonymize,
            None,
            None,
            &ctx,
        )
        .unwrap();

        assert_eq!(result1.value, result2.value);
    }
}
