//! In-memory token vault implementation.

use std::collections::HashMap;
use std::sync::RwLock;

use veil_core::SensitiveString;

use crate::error::VaultError;
use crate::vault::TokenVault;

/// In-memory token vault for testing and simple use cases.
///
/// Thread-safe implementation using RwLock.
/// Original values are stored using SensitiveString for secure memory handling.
#[derive(Debug, Default)]
pub struct InMemoryVault {
    /// Token to original mapping (original values use SensitiveString for security).
    tokens: RwLock<HashMap<String, SensitiveString>>,
    /// Original to token mapping (for consistency lookups).
    /// Keys use SensitiveString to securely store original values.
    reverse: RwLock<HashMap<SensitiveString, String>>,
}

impl InMemoryVault {
    /// Create a new empty in-memory vault.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of stored tokens.
    pub fn len(&self) -> usize {
        self.tokens.read().map(|t| t.len()).unwrap_or(0)
    }

    /// Check if the vault is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all stored tokens.
    pub fn clear(&self) {
        if let Ok(mut tokens) = self.tokens.write() {
            tokens.clear();
        }
        if let Ok(mut reverse) = self.reverse.write() {
            reverse.clear();
        }
    }
}

impl TokenVault for InMemoryVault {
    fn store(&self, token: &str, original: &str) -> Result<(), VaultError> {
        // Acquire both locks
        let mut tokens = self
            .tokens
            .write()
            .map_err(|_| VaultError::lock_poisoned("acquire tokens write lock"))?;

        let mut reverse = self
            .reverse
            .write()
            .map_err(|_| VaultError::lock_poisoned("acquire reverse write lock"))?;

        // Check for duplicate token
        if tokens.contains_key(token) {
            return Err(VaultError::Duplicate(token.to_string()));
        }

        // Store both mappings using SensitiveString for secure memory handling
        let sensitive_original = SensitiveString::new(original);
        tokens.insert(token.to_string(), sensitive_original.clone());
        reverse.insert(sensitive_original, token.to_string());

        Ok(())
    }

    fn lookup(&self, token: &str) -> Result<Option<String>, VaultError> {
        let tokens = self
            .tokens
            .read()
            .map_err(|_| VaultError::lock_poisoned("acquire tokens read lock"))?;

        // Convert SensitiveString back to String for return
        Ok(tokens.get(token).map(|s| s.as_str().to_string()))
    }

    fn delete(&self, token: &str) -> Result<bool, VaultError> {
        let mut tokens = self
            .tokens
            .write()
            .map_err(|_| VaultError::lock_poisoned("acquire tokens write lock"))?;

        let mut reverse = self
            .reverse
            .write()
            .map_err(|_| VaultError::lock_poisoned("acquire reverse write lock"))?;

        // Find original value to remove from reverse map
        if let Some(original) = tokens.remove(token) {
            reverse.remove(&original);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn find_by_original(&self, original: &str) -> Result<Option<String>, VaultError> {
        let reverse = self
            .reverse
            .read()
            .map_err(|_| VaultError::lock_poisoned("acquire reverse read lock"))?;

        // Create a SensitiveString key for lookup
        let key = SensitiveString::new(original);
        Ok(reverse.get(&key).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_lookup() {
        let vault = InMemoryVault::new();

        vault.store("tok_123", "original_value").unwrap();
        let result = vault.lookup("tok_123").unwrap();

        assert_eq!(result, Some("original_value".to_string()));
    }

    #[test]
    fn test_lookup_nonexistent() {
        let vault = InMemoryVault::new();

        let result = vault.lookup("nonexistent").unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn test_duplicate_token_error() {
        let vault = InMemoryVault::new();

        vault.store("tok_dup", "value1").unwrap();
        let result = vault.store("tok_dup", "value2");

        assert!(matches!(result, Err(VaultError::Duplicate(_))));
    }

    #[test]
    fn test_delete_existing() {
        let vault = InMemoryVault::new();

        vault.store("tok_del", "value").unwrap();
        let deleted = vault.delete("tok_del").unwrap();
        let lookup = vault.lookup("tok_del").unwrap();

        assert!(deleted);
        assert_eq!(lookup, None);
    }

    #[test]
    fn test_delete_nonexistent() {
        let vault = InMemoryVault::new();

        let deleted = vault.delete("nonexistent").unwrap();

        assert!(!deleted);
    }

    #[test]
    fn test_find_by_original() {
        let vault = InMemoryVault::new();

        vault.store("tok_abc", "original").unwrap();
        let result = vault.find_by_original("original").unwrap();

        assert_eq!(result, Some("tok_abc".to_string()));
    }

    #[test]
    fn test_find_by_original_not_found() {
        let vault = InMemoryVault::new();

        let result = vault.find_by_original("unknown").unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn test_len_and_is_empty() {
        let vault = InMemoryVault::new();

        assert!(vault.is_empty());
        assert_eq!(vault.len(), 0);

        vault.store("tok_1", "val_1").unwrap();
        vault.store("tok_2", "val_2").unwrap();

        assert!(!vault.is_empty());
        assert_eq!(vault.len(), 2);
    }

    #[test]
    fn test_clear() {
        let vault = InMemoryVault::new();

        vault.store("tok_1", "val_1").unwrap();
        vault.store("tok_2", "val_2").unwrap();
        vault.clear();

        assert!(vault.is_empty());
        assert_eq!(vault.lookup("tok_1").unwrap(), None);
    }

    #[test]
    fn test_delete_removes_from_reverse_map() {
        let vault = InMemoryVault::new();

        vault.store("tok_rev", "value").unwrap();
        assert_eq!(
            vault.find_by_original("value").unwrap(),
            Some("tok_rev".to_string())
        );

        vault.delete("tok_rev").unwrap();
        assert_eq!(vault.find_by_original("value").unwrap(), None);
    }
}
