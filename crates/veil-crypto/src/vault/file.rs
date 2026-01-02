//! Persistent file-based token vault implementation.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

use crate::error::VaultError;
use crate::vault::TokenVault;
use crate::PlaintextStoragePolicy;

fn open_secure_append(path: &Path) -> Result<File, VaultError> {
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .map_err(|e| VaultError::io("open vault file", path, e))
}

fn create_secure_file(path: &Path) -> Result<File, VaultError> {
    let mut options = OpenOptions::new();
    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .map_err(|e| VaultError::io("create vault file", path, e))
}

/// A token entry stored in the vault file.
#[derive(Clone, Serialize, Deserialize)]
struct TokenEntry {
    token: String,
    original: String,
}

/// File-based persistent token vault.
///
/// Stores token mappings in a JSONL file, one entry per line.
/// Thread-safe with in-memory caching for performance.
///
/// # Security
/// Plaintext storage is disabled by default. Pass
/// `PlaintextStoragePolicy::allow_insecure()` (or use `new_from_env`).
pub struct FileVault {
    /// Path to the vault file.
    path: PathBuf,
    /// In-memory token to original mapping.
    tokens: RwLock<HashMap<String, String>>,
    /// In-memory original to token mapping (for consistency lookups).
    reverse: RwLock<HashMap<String, String>>,
}

impl std::fmt::Debug for FileVault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token_count = self.tokens.read().map(|t| t.len()).unwrap_or(0);
        let reverse_count = self.reverse.read().map(|r| r.len()).unwrap_or(0);

        f.debug_struct("FileVault")
            .field("path", &self.path)
            .field("token_count", &token_count)
            .field("reverse_count", &reverse_count)
            .finish()
    }
}

impl FileVault {
    /// Create a new file vault at the given path.
    ///
    /// If the file exists, loads existing tokens. Otherwise creates a new file.
    pub fn new(
        path: impl Into<PathBuf>,
        storage_policy: PlaintextStoragePolicy,
    ) -> Result<Self, VaultError> {
        let path = path.into();

        if !storage_policy.is_allowed() {
            return Err(VaultError::PlaintextStorageDisabled);
        }

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| VaultError::io("create vault directory", parent, e))?;
            }
        }

        let mut vault = Self {
            path,
            tokens: RwLock::new(HashMap::new()),
            reverse: RwLock::new(HashMap::new()),
        };

        // Load existing tokens if file exists
        vault.load()?;

        Ok(vault)
    }

    /// Create a new file vault using `VEIL_ALLOW_PLAINTEXT_STORAGE`.
    pub fn new_from_env(path: impl Into<PathBuf>) -> Result<Self, VaultError> {
        Self::new(path, PlaintextStoragePolicy::from_env())
    }

    /// Load tokens from the vault file.
    fn load(&mut self) -> Result<(), VaultError> {
        if !self.path.exists() {
            return Ok(());
        }

        let file =
            File::open(&self.path).map_err(|e| VaultError::io("open vault file", &self.path, e))?;

        let reader = BufReader::new(file);

        let mut tokens = self
            .tokens
            .write()
            .map_err(|_| VaultError::lock_poisoned("acquire tokens write lock"))?;
        let mut reverse = self
            .reverse
            .write()
            .map_err(|_| VaultError::lock_poisoned("acquire reverse write lock"))?;

        for line in reader.lines() {
            let line = line.map_err(|e| VaultError::io("read vault line", &self.path, e))?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: TokenEntry = serde_json::from_str(&line)
                .map_err(|e| VaultError::json("parse vault entry", e))?;

            tokens.insert(entry.token.clone(), entry.original.clone());
            reverse.insert(entry.original, entry.token);
        }

        Ok(())
    }

    /// Append a token entry to the vault file.
    fn append(&self, token: &str, original: &str) -> Result<(), VaultError> {
        let entry = TokenEntry {
            token: token.to_string(),
            original: original.to_string(),
        };

        let json = serde_json::to_string(&entry)
            .map_err(|e| VaultError::json("serialize vault entry", e))?;

        let mut file = open_secure_append(&self.path)?;

        writeln!(file, "{}", json)
            .map_err(|e| VaultError::io("append vault entry", &self.path, e))?;

        Ok(())
    }

    /// Rewrite the entire vault file (used after deletions).
    fn rewrite(&self) -> Result<(), VaultError> {
        let tokens = self
            .tokens
            .read()
            .map_err(|_| VaultError::lock_poisoned("acquire tokens read lock"))?;

        // Write to a temporary file first for atomicity
        let temp_path = self.path.with_extension("tmp");
        let mut file = create_secure_file(&temp_path)?;

        for (token, original) in tokens.iter() {
            let entry = TokenEntry {
                token: token.clone(),
                original: original.clone(),
            };
            let json = serde_json::to_string(&entry)
                .map_err(|e| VaultError::json("serialize vault entry", e))?;
            writeln!(file, "{}", json)
                .map_err(|e| VaultError::io("write vault file", &temp_path, e))?;
        }

        // Atomically replace the original file
        fs::rename(&temp_path, &self.path)
            .map_err(|e| VaultError::io("replace vault file", &self.path, e))?;

        Ok(())
    }

    /// Get the number of stored tokens.
    pub fn len(&self) -> usize {
        self.tokens.read().map(|t| t.len()).unwrap_or(0)
    }

    /// Check if the vault is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the vault file path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl TokenVault for FileVault {
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

        // Append to file first
        self.append(token, original)?;

        // Then update in-memory maps
        tokens.insert(token.to_string(), original.to_string());
        reverse.insert(original.to_string(), token.to_string());

        Ok(())
    }

    fn lookup(&self, token: &str) -> Result<Option<String>, VaultError> {
        let tokens = self
            .tokens
            .read()
            .map_err(|_| VaultError::lock_poisoned("acquire tokens read lock"))?;

        Ok(tokens.get(token).cloned())
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

        // Remove from in-memory maps
        if let Some(original) = tokens.remove(token) {
            reverse.remove(&original);
            // Rewrite the file without the deleted entry
            drop(tokens);
            drop(reverse);
            self.rewrite()?;
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

        Ok(reverse.get(original).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_vault() -> (TempDir, FileVault) {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("tokens.jsonl");
        let vault = FileVault::new(&vault_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
        (temp_dir, vault)
    }

    #[test]
    fn test_store_and_lookup() {
        let (_dir, vault) = temp_vault();

        vault.store("tok_123", "original_value").unwrap();
        let result = vault.lookup("tok_123").unwrap();

        assert_eq!(result, Some("original_value".to_string()));
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("tokens.jsonl");

        // Create vault and store tokens
        {
            let vault =
                FileVault::new(&vault_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            vault.store("tok_persist", "persisted_value").unwrap();
        }

        // Reopen vault and verify
        {
            let vault =
                FileVault::new(&vault_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            let result = vault.lookup("tok_persist").unwrap();
            assert_eq!(result, Some("persisted_value".to_string()));
        }
    }

    #[test]
    fn test_lookup_nonexistent() {
        let (_dir, vault) = temp_vault();

        let result = vault.lookup("nonexistent").unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn test_duplicate_token_error() {
        let (_dir, vault) = temp_vault();

        vault.store("tok_dup", "value1").unwrap();
        let result = vault.store("tok_dup", "value2");

        assert!(matches!(result, Err(VaultError::Duplicate(_))));
    }

    #[test]
    fn test_delete_existing() {
        let (_dir, vault) = temp_vault();

        vault.store("tok_del", "value").unwrap();
        let deleted = vault.delete("tok_del").unwrap();
        let lookup = vault.lookup("tok_del").unwrap();

        assert!(deleted);
        assert_eq!(lookup, None);
    }

    #[test]
    fn test_delete_persists() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("tokens.jsonl");

        // Store and delete
        {
            let vault =
                FileVault::new(&vault_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            vault.store("tok_a", "value_a").unwrap();
            vault.store("tok_b", "value_b").unwrap();
            vault.delete("tok_a").unwrap();
        }

        // Reopen and verify deletion persisted
        {
            let vault =
                FileVault::new(&vault_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            assert_eq!(vault.lookup("tok_a").unwrap(), None);
            assert_eq!(vault.lookup("tok_b").unwrap(), Some("value_b".to_string()));
        }
    }

    #[test]
    fn test_delete_nonexistent() {
        let (_dir, vault) = temp_vault();

        let deleted = vault.delete("nonexistent").unwrap();

        assert!(!deleted);
    }

    #[test]
    fn test_find_by_original() {
        let (_dir, vault) = temp_vault();

        vault.store("tok_abc", "original").unwrap();
        let result = vault.find_by_original("original").unwrap();

        assert_eq!(result, Some("tok_abc".to_string()));
    }

    #[test]
    fn test_find_by_original_not_found() {
        let (_dir, vault) = temp_vault();

        let result = vault.find_by_original("unknown").unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn test_len_and_is_empty() {
        let (_dir, vault) = temp_vault();

        assert!(vault.is_empty());
        assert_eq!(vault.len(), 0);

        vault.store("tok_1", "val_1").unwrap();
        vault.store("tok_2", "val_2").unwrap();

        assert!(!vault.is_empty());
        assert_eq!(vault.len(), 2);
    }

    #[test]
    fn test_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("tokens.jsonl");

        let vault = FileVault::new(&nested_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
        vault.store("tok", "val").unwrap();

        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    fn test_multiple_tokens_persist() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("tokens.jsonl");

        // Store multiple tokens
        {
            let vault =
                FileVault::new(&vault_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            for i in 0..10 {
                vault
                    .store(&format!("tok_{}", i), &format!("val_{}", i))
                    .unwrap();
            }
        }

        // Verify all persist
        {
            let vault =
                FileVault::new(&vault_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            assert_eq!(vault.len(), 10);
            for i in 0..10 {
                let result = vault.lookup(&format!("tok_{}", i)).unwrap();
                assert_eq!(result, Some(format!("val_{}", i)));
            }
        }
    }

    #[test]
    fn test_find_by_original_after_reload() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("tokens.jsonl");

        // Store
        {
            let vault =
                FileVault::new(&vault_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            vault.store("tok_find", "original_find").unwrap();
        }

        // Reload and find
        {
            let vault =
                FileVault::new(&vault_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            let result = vault.find_by_original("original_find").unwrap();
            assert_eq!(result, Some("tok_find".to_string()));
        }
    }
}
