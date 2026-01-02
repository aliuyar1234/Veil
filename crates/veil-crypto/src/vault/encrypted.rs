//! Encrypted token vault implementation.
//!
//! Provides encryption-at-rest for token-to-original mappings using
//! AES-256-GCM with envelope encryption pattern.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::sync::RwLock;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};

use crate::error::VaultError;
use crate::key_provider::{KeyProvider, SecretKey};
use crate::vault::TokenVault;

/// Version of the encrypted vault format.
const VAULT_VERSION: u8 = 1;

/// Nonce size for AES-256-GCM (96 bits).
const NONCE_SIZE: usize = 12;

/// An encrypted entry in the vault.
#[derive(Clone, Serialize, Deserialize)]
struct EncryptedEntry {
    /// AES-GCM nonce for this entry.
    nonce: [u8; NONCE_SIZE],
    /// Encrypted token-original pair.
    ciphertext: Vec<u8>,
}

/// Decrypted entry for internal use.
#[derive(Clone, Serialize, Deserialize)]
struct PlaintextEntry {
    token: String,
    original: String,
}

/// On-disk format for the encrypted vault.
#[derive(Serialize, Deserialize)]
struct VaultFile {
    /// Format version.
    version: u8,
    /// Key ID used for the KEK.
    key_id: String,
    /// Data Encryption Key (DEK) encrypted with KEK.
    encrypted_dek: Vec<u8>,
    /// Nonce used for DEK encryption.
    dek_nonce: [u8; NONCE_SIZE],
    /// Encrypted token entries.
    entries: Vec<EncryptedEntry>,
}

/// Encrypted token vault with envelope encryption.
///
/// Uses a two-tier encryption scheme:
/// 1. Key Encryption Key (KEK): Retrieved from KeyProvider
/// 2. Data Encryption Key (DEK): Generated per vault, encrypted with KEK
///
/// This pattern allows key rotation without re-encrypting all entries.
///
/// # Example
///
/// ```rust,ignore
/// use veil_crypto::vault::EncryptedVault;
/// use veil_crypto::key_provider::LocalKeyProvider;
/// use veil_crypto::PlaintextStoragePolicy;
///
/// let key_provider = LocalKeyProvider::new("keys.jsonl", PlaintextStoragePolicy::allow_insecure())?;
/// // Initialize the key first
/// key_provider.rotate_key("vault-key")?;
///
/// let vault = EncryptedVault::new("vault.enc", key_provider, "vault-key")?;
/// vault.store("tok_123", "sensitive_data")?;
/// ```
pub struct EncryptedVault<K: KeyProvider> {
    /// Path to the encrypted vault file.
    path: PathBuf,
    /// Key provider for KEK retrieval.
    key_provider: K,
    /// Key ID for the KEK.
    key_id: String,
    /// Data Encryption Key (decrypted, in memory).
    dek: RwLock<SecretKey>,
    /// In-memory token to original mapping (decrypted).
    tokens: RwLock<HashMap<String, String>>,
    /// In-memory original to token mapping.
    reverse: RwLock<HashMap<String, String>>,
}

impl<K: KeyProvider> EncryptedVault<K> {
    /// Create or open an encrypted vault.
    ///
    /// # Arguments
    /// * `path` - Path to the vault file
    /// * `key_provider` - Provider for the Key Encryption Key
    /// * `key_id` - Identifier for the KEK in the provider
    ///
    /// # Security
    /// The DEK is kept decrypted in memory for performance.
    /// The KEK is only used for DEK encryption/decryption.
    pub fn new(
        path: impl Into<PathBuf>,
        key_provider: K,
        key_id: impl Into<String>,
    ) -> Result<Self, VaultError> {
        let path = path.into();
        let key_id = key_id.into();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| VaultError::io("create vault directory", parent, e))?;
            }
        }

        if path.exists() {
            // Load existing vault
            Self::open(path, key_provider, key_id)
        } else {
            // Create new vault
            Self::create(path, key_provider, key_id)
        }
    }

    /// Create a new encrypted vault.
    fn create(path: PathBuf, key_provider: K, key_id: String) -> Result<Self, VaultError> {
        // Generate a new DEK
        let dek = SecretKey::generate();

        // Encrypt DEK with KEK
        let kek = key_provider
            .get_key(&key_id)
            .map_err(|e| VaultError::Storage(format!("Failed to get KEK: {}", e)))?;

        let cipher = Aes256Gcm::new(kek.as_aes_key());
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted_dek = cipher
            .encrypt(nonce, dek.as_bytes().as_ref())
            .map_err(|e| VaultError::Storage(format!("Failed to encrypt DEK: {}", e)))?;

        // Create initial vault file
        let vault_file = VaultFile {
            version: VAULT_VERSION,
            key_id: key_id.clone(),
            encrypted_dek,
            dek_nonce: nonce_bytes,
            entries: Vec::new(),
        };

        let json = serde_json::to_string(&vault_file)
            .map_err(|e| VaultError::json("serialize vault", e))?;

        fs::write(&path, json).map_err(|e| VaultError::io("write vault file", &path, e))?;

        Ok(Self {
            path,
            key_provider,
            key_id,
            dek: RwLock::new(dek),
            tokens: RwLock::new(HashMap::new()),
            reverse: RwLock::new(HashMap::new()),
        })
    }

    /// Open an existing encrypted vault.
    fn open(path: PathBuf, key_provider: K, key_id: String) -> Result<Self, VaultError> {
        let mut file =
            File::open(&path).map_err(|e| VaultError::io("open vault file", &path, e))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| VaultError::io("read vault file", &path, e))?;

        let vault_file: VaultFile =
            serde_json::from_str(&contents).map_err(|e| VaultError::json("parse vault file", e))?;

        if vault_file.version != VAULT_VERSION {
            return Err(VaultError::Storage(format!(
                "Unsupported vault version: {}",
                vault_file.version
            )));
        }

        // Decrypt DEK
        let kek = key_provider
            .get_key(&key_id)
            .map_err(|e| VaultError::Storage(format!("Failed to get KEK: {}", e)))?;

        let cipher = Aes256Gcm::new(kek.as_aes_key());
        let nonce = Nonce::from_slice(&vault_file.dek_nonce);

        let dek_bytes = cipher
            .decrypt(nonce, vault_file.encrypted_dek.as_ref())
            .map_err(|e| VaultError::Storage(format!("Failed to decrypt DEK: {}", e)))?;

        if dek_bytes.len() != 32 {
            return Err(VaultError::Storage(format!(
                "Invalid DEK length: {}",
                dek_bytes.len()
            )));
        }

        let dek = SecretKey::from_slice(&dek_bytes);

        // Decrypt entries
        let entry_cipher = Aes256Gcm::new(dek.as_aes_key());
        let mut tokens = HashMap::new();
        let mut reverse = HashMap::new();

        for entry in &vault_file.entries {
            let nonce = Nonce::from_slice(&entry.nonce);
            let plaintext = entry_cipher
                .decrypt(nonce, entry.ciphertext.as_ref())
                .map_err(|e| VaultError::Storage(format!("Failed to decrypt entry: {}", e)))?;

            let pt_entry: PlaintextEntry = serde_json::from_slice(&plaintext)
                .map_err(|e| VaultError::json("parse vault entry", e))?;

            tokens.insert(pt_entry.token.clone(), pt_entry.original.clone());
            reverse.insert(pt_entry.original, pt_entry.token);
        }

        Ok(Self {
            path,
            key_provider,
            key_id,
            dek: RwLock::new(dek),
            tokens: RwLock::new(tokens),
            reverse: RwLock::new(reverse),
        })
    }

    /// Persist the vault to disk.
    fn save(&self) -> Result<(), VaultError> {
        let dek = self
            .dek
            .read()
            .map_err(|_| VaultError::lock_poisoned("acquire DEK read lock"))?;
        let tokens = self
            .tokens
            .read()
            .map_err(|_| VaultError::lock_poisoned("acquire tokens read lock"))?;

        // Encrypt DEK
        let kek = self
            .key_provider
            .get_key(&self.key_id)
            .map_err(|e| VaultError::Storage(format!("Failed to get KEK: {}", e)))?;

        let kek_cipher = Aes256Gcm::new(kek.as_aes_key());
        let mut dek_nonce_bytes = [0u8; NONCE_SIZE];
        thread_rng().fill_bytes(&mut dek_nonce_bytes);
        let dek_nonce = Nonce::from_slice(&dek_nonce_bytes);

        let encrypted_dek = kek_cipher
            .encrypt(dek_nonce, dek.as_bytes().as_ref())
            .map_err(|e| VaultError::Storage(format!("Failed to encrypt DEK: {}", e)))?;

        // Encrypt entries
        let entry_cipher = Aes256Gcm::new(dek.as_aes_key());
        let mut entries = Vec::with_capacity(tokens.len());

        for (token, original) in tokens.iter() {
            let pt_entry = PlaintextEntry {
                token: token.clone(),
                original: original.clone(),
            };

            let plaintext = serde_json::to_vec(&pt_entry)
                .map_err(|e| VaultError::json("serialize vault entry", e))?;

            let mut nonce_bytes = [0u8; NONCE_SIZE];
            thread_rng().fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let ciphertext = entry_cipher
                .encrypt(nonce, plaintext.as_ref())
                .map_err(|e| VaultError::Storage(format!("Failed to encrypt entry: {}", e)))?;

            entries.push(EncryptedEntry {
                nonce: nonce_bytes,
                ciphertext,
            });
        }

        let vault_file = VaultFile {
            version: VAULT_VERSION,
            key_id: self.key_id.clone(),
            encrypted_dek,
            dek_nonce: dek_nonce_bytes,
            entries,
        };

        let json = serde_json::to_string(&vault_file)
            .map_err(|e| VaultError::json("serialize vault", e))?;

        // Write atomically via temp file
        let temp_path = self.path.with_extension("tmp");
        fs::write(&temp_path, &json)
            .map_err(|e| VaultError::io("write temp vault file", &temp_path, e))?;

        fs::rename(&temp_path, &self.path)
            .map_err(|e| VaultError::io("save vault file", &self.path, e))?;

        Ok(())
    }

    /// Rotate the encryption key.
    ///
    /// This re-encrypts the DEK with a new KEK version.
    /// All existing entries remain encrypted with the same DEK.
    pub fn rotate_key(&self) -> Result<(), VaultError> {
        // Generate new KEK
        self.key_provider
            .rotate_key(&self.key_id)
            .map_err(|e| VaultError::Storage(format!("Failed to rotate key: {}", e)))?;

        // Re-save the vault (DEK will be encrypted with new KEK)
        self.save()
    }

    /// Get the number of stored tokens.
    pub fn len(&self) -> usize {
        self.tokens.read().map(|t| t.len()).unwrap_or(0)
    }

    /// Check if the vault is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K: KeyProvider> TokenVault for EncryptedVault<K> {
    fn store(&self, token: &str, original: &str) -> Result<(), VaultError> {
        let mut tokens = self
            .tokens
            .write()
            .map_err(|_| VaultError::lock_poisoned("acquire tokens write lock"))?;

        let mut reverse = self
            .reverse
            .write()
            .map_err(|_| VaultError::lock_poisoned("acquire reverse write lock"))?;

        if tokens.contains_key(token) {
            return Err(VaultError::Duplicate(token.to_string()));
        }

        tokens.insert(token.to_string(), original.to_string());
        reverse.insert(original.to_string(), token.to_string());

        drop(tokens);
        drop(reverse);

        self.save()
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

        if let Some(original) = tokens.remove(token) {
            reverse.remove(&original);
            drop(tokens);
            drop(reverse);
            self.save()?;
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
    use crate::key_provider::LocalKeyProvider;
    use crate::PlaintextStoragePolicy;
    use tempfile::TempDir;

    fn setup_vault() -> (TempDir, EncryptedVault<LocalKeyProvider>) {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("keys.jsonl");
        let vault_path = temp_dir.path().join("vault.enc");

        let key_provider =
            LocalKeyProvider::new(&key_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
        // Create the initial key
        key_provider.rotate_key("test-key").unwrap();

        let vault = EncryptedVault::new(&vault_path, key_provider, "test-key").unwrap();
        (temp_dir, vault)
    }

    #[test]
    fn test_create_and_store() {
        let (_dir, vault) = setup_vault();

        vault.store("tok_123", "secret_value").unwrap();
        let result = vault.lookup("tok_123").unwrap();

        assert_eq!(result, Some("secret_value".to_string()));
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("keys.jsonl");
        let vault_path = temp_dir.path().join("vault.enc");

        // Create and store
        {
            let key_provider =
                LocalKeyProvider::new(&key_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            key_provider.rotate_key("persist-key").unwrap();

            let vault = EncryptedVault::new(&vault_path, key_provider, "persist-key").unwrap();
            vault.store("tok_persist", "persisted_secret").unwrap();
        }

        // Reopen and verify
        {
            let key_provider =
                LocalKeyProvider::new(&key_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            let vault = EncryptedVault::new(&vault_path, key_provider, "persist-key").unwrap();

            let result = vault.lookup("tok_persist").unwrap();
            assert_eq!(result, Some("persisted_secret".to_string()));
        }
    }

    #[test]
    fn test_duplicate_error() {
        let (_dir, vault) = setup_vault();

        vault.store("tok_dup", "value1").unwrap();
        let result = vault.store("tok_dup", "value2");

        assert!(matches!(result, Err(VaultError::Duplicate(_))));
    }

    #[test]
    fn test_delete() {
        let (_dir, vault) = setup_vault();

        vault.store("tok_del", "value").unwrap();
        let deleted = vault.delete("tok_del").unwrap();
        let lookup = vault.lookup("tok_del").unwrap();

        assert!(deleted);
        assert_eq!(lookup, None);
    }

    #[test]
    fn test_find_by_original() {
        let (_dir, vault) = setup_vault();

        vault.store("tok_find", "original_value").unwrap();
        let result = vault.find_by_original("original_value").unwrap();

        assert_eq!(result, Some("tok_find".to_string()));
    }

    #[test]
    fn test_key_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("keys.jsonl");
        let vault_path = temp_dir.path().join("vault.enc");

        // Create vault and store data
        {
            let key_provider =
                LocalKeyProvider::new(&key_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            key_provider.rotate_key("rotate-key").unwrap();

            let vault = EncryptedVault::new(&vault_path, key_provider, "rotate-key").unwrap();
            vault.store("tok_before", "before_rotation").unwrap();
            vault.rotate_key().unwrap();
            vault.store("tok_after", "after_rotation").unwrap();
        }

        // Reopen and verify both entries still accessible
        {
            let key_provider =
                LocalKeyProvider::new(&key_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            let vault = EncryptedVault::new(&vault_path, key_provider, "rotate-key").unwrap();

            assert_eq!(
                vault.lookup("tok_before").unwrap(),
                Some("before_rotation".to_string())
            );
            assert_eq!(
                vault.lookup("tok_after").unwrap(),
                Some("after_rotation".to_string())
            );
        }
    }

    #[test]
    fn test_len_and_is_empty() {
        let (_dir, vault) = setup_vault();

        assert!(vault.is_empty());
        assert_eq!(vault.len(), 0);

        vault.store("tok_1", "val_1").unwrap();
        vault.store("tok_2", "val_2").unwrap();

        assert!(!vault.is_empty());
        assert_eq!(vault.len(), 2);
    }

    #[test]
    fn test_wrong_key_fails() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("keys.jsonl");
        let vault_path = temp_dir.path().join("vault.enc");

        // Create vault with one key
        {
            let key_provider =
                LocalKeyProvider::new(&key_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            key_provider.rotate_key("key1").unwrap();

            let vault = EncryptedVault::new(&vault_path, key_provider, "key1").unwrap();
            vault.store("tok", "secret").unwrap();
        }

        // Try to open with different key
        {
            let key_provider =
                LocalKeyProvider::new(&key_path, PlaintextStoragePolicy::allow_insecure()).unwrap();
            key_provider.rotate_key("key2").unwrap();

            let result = EncryptedVault::new(&vault_path, key_provider, "key2");
            assert!(result.is_err());
        }
    }
}
