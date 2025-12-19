//! Key management abstraction for encryption operations.
//!
//! This module provides a trait-based abstraction for key management backends,
//! enabling different key storage strategies without changing encryption logic.
//!
//! # Example
//!
//! ```rust,ignore
//! use veil_crypto::key_provider::{KeyProvider, EnvKeyProvider};
//!
//! let provider = EnvKeyProvider::new("VEIL_KEY_");
//! let key = provider.get_key("default")?;
//! ```

use crate::error::CryptoError;
use aes_gcm::aead::generic_array::typenum::U32;
use aes_gcm::aead::generic_array::GenericArray;
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use zeroize::{Zeroize, ZeroizeOnDrop};

const PLAINTEXT_STORAGE_ENV: &str = "VEIL_ALLOW_PLAINTEXT_STORAGE";

fn plaintext_storage_allowed() -> bool {
    static ALLOW: OnceLock<bool> = OnceLock::new();
    *ALLOW.get_or_init(|| {
        std::env::var(PLAINTEXT_STORAGE_ENV)
            .ok()
            .map(|value| {
                let normalized = value.trim().to_ascii_lowercase();
                normalized == "1" || normalized == "true" || normalized == "yes"
            })
            .unwrap_or(false)
    })
}

fn create_secure_file(path: &Path) -> Result<File, CryptoError> {
    let mut options = OpenOptions::new();
    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .map_err(|e| CryptoError::KeyManagement(format!("Failed to create file: {}", e)))
}

/// A 256-bit secret key with automatic zeroization on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretKey {
    bytes: [u8; 32],
}

impl SecretKey {
    /// Create a new secret key from raw bytes.
    ///
    /// # Panics
    /// Panics if the slice length is not exactly 32 bytes.
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Self { bytes }
    }

    /// Generate a new random secret key.
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        thread_rng().fill_bytes(&mut bytes);
        Self { bytes }
    }

    /// Get the key bytes as a generic array for AES-GCM.
    pub fn as_aes_key(&self) -> &GenericArray<u8, U32> {
        GenericArray::from_slice(&self.bytes)
    }

    /// Get the raw key bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl std::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretKey")
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

/// Metadata about an encryption key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Unique identifier for the key.
    pub key_id: String,
    /// Version number (incremented on rotation).
    pub version: u32,
    /// Creation timestamp (Unix epoch seconds).
    pub created_at: u64,
    /// Whether this is the currently active key.
    pub active: bool,
}

/// Trait for key management backends.
///
/// Implementations must be thread-safe (Send + Sync) and should
/// securely handle key material.
pub trait KeyProvider: Send + Sync {
    /// Retrieve an encryption key by its identifier.
    ///
    /// # Arguments
    /// * `key_id` - The key identifier to look up
    ///
    /// # Returns
    /// * `Ok(SecretKey)` - The retrieved key
    /// * `Err(CryptoError::KeyNotFound)` - If the key doesn't exist
    fn get_key(&self, key_id: &str) -> Result<SecretKey, CryptoError>;

    /// Generate a new key version (key rotation).
    ///
    /// Creates a new key with an incremented version number.
    /// The old key remains available for decryption.
    ///
    /// # Arguments
    /// * `key_id` - The key identifier to rotate
    ///
    /// # Returns
    /// * `Ok(SecretKey)` - The new key version
    fn rotate_key(&self, key_id: &str) -> Result<SecretKey, CryptoError>;

    /// List all available keys.
    ///
    /// # Returns
    /// Metadata for all keys managed by this provider.
    fn list_keys(&self) -> Result<Vec<KeyMetadata>, CryptoError>;
}

/// File entry for persistent key storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyEntry {
    key_id: String,
    version: u32,
    /// Base64-encoded key bytes
    key_material: String,
    created_at: u64,
    active: bool,
}

/// File-based key storage provider.
///
/// Stores keys in a JSON Lines file. Keys are base64-encoded.
///
/// # Security Considerations
/// - Keys are stored unencrypted; use filesystem permissions for protection
/// - Disabled by default; set `VEIL_ALLOW_PLAINTEXT_STORAGE=1` to enable
/// - Suitable for development and single-node deployments
/// - For production, consider using a KMS-backed provider
#[derive(Debug)]
pub struct LocalKeyProvider {
    path: PathBuf,
    keys: RwLock<HashMap<String, KeyEntry>>,
}

impl LocalKeyProvider {
    /// Create a new local key provider.
    ///
    /// # Arguments
    /// * `path` - Path to the key storage file
    ///
    /// If the file exists, loads existing keys. Otherwise creates a new file.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, CryptoError> {
        let path = path.into();

        if !plaintext_storage_allowed() {
            return Err(CryptoError::PlaintextStorageDisabled);
        }

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    CryptoError::KeyManagement(format!("Failed to create directory: {}", e))
                })?;
            }
        }

        let mut provider = Self {
            path,
            keys: RwLock::new(HashMap::new()),
        };

        provider.load()?;
        Ok(provider)
    }

    /// Load keys from the storage file.
    fn load(&mut self) -> Result<(), CryptoError> {
        if !self.path.exists() {
            return Ok(());
        }

        let file = File::open(&self.path)
            .map_err(|e| CryptoError::KeyManagement(format!("Failed to open key file: {}", e)))?;

        let reader = BufReader::new(file);
        let mut keys = self
            .keys
            .write()
            .map_err(|e| CryptoError::KeyManagement(e.to_string()))?;

        for line in reader.lines() {
            let line = line
                .map_err(|e| CryptoError::KeyManagement(format!("Failed to read line: {}", e)))?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: KeyEntry = serde_json::from_str(&line)
                .map_err(|e| CryptoError::KeyManagement(format!("Invalid key entry: {}", e)))?;

            keys.insert(entry.key_id.clone(), entry);
        }

        Ok(())
    }

    /// Save all keys to the storage file.
    fn save(&self) -> Result<(), CryptoError> {
        let keys = self
            .keys
            .read()
            .map_err(|e| CryptoError::KeyManagement(e.to_string()))?;

        let temp_path = self.path.with_extension("tmp");
        let mut file = create_secure_file(&temp_path)?;

        for entry in keys.values() {
            let json = serde_json::to_string(entry)
                .map_err(|e| CryptoError::KeyManagement(format!("Failed to serialize: {}", e)))?;
            writeln!(file, "{}", json)
                .map_err(|e| CryptoError::KeyManagement(format!("Failed to write: {}", e)))?;
        }

        fs::rename(&temp_path, &self.path)
            .map_err(|e| CryptoError::KeyManagement(format!("Failed to save keys: {}", e)))?;

        Ok(())
    }

    /// Create or update a key.
    fn store_key(
        &self,
        key_id: &str,
        key: &SecretKey,
        version: u32,
        active: bool,
    ) -> Result<(), CryptoError> {
        let entry = KeyEntry {
            key_id: key_id.to_string(),
            version,
            key_material: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                key.as_bytes(),
            ),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            active,
        };

        {
            let mut keys = self
                .keys
                .write()
                .map_err(|e| CryptoError::KeyManagement(e.to_string()))?;
            keys.insert(key_id.to_string(), entry);
        }

        self.save()
    }
}

impl KeyProvider for LocalKeyProvider {
    fn get_key(&self, key_id: &str) -> Result<SecretKey, CryptoError> {
        let keys = self
            .keys
            .read()
            .map_err(|e| CryptoError::KeyManagement(e.to_string()))?;

        let entry = keys
            .get(key_id)
            .ok_or_else(|| CryptoError::KeyNotFound(key_id.to_string()))?;

        let key_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &entry.key_material,
        )
        .map_err(|e| CryptoError::KeyManagement(format!("Invalid key encoding: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(CryptoError::KeyManagement(format!(
                "Invalid key length: expected 32, got {}",
                key_bytes.len()
            )));
        }

        Ok(SecretKey::from_slice(&key_bytes))
    }

    fn rotate_key(&self, key_id: &str) -> Result<SecretKey, CryptoError> {
        let current_version = {
            let keys = self
                .keys
                .read()
                .map_err(|e| CryptoError::KeyManagement(e.to_string()))?;
            keys.get(key_id).map(|e| e.version).unwrap_or(0)
        };

        let new_key = SecretKey::generate();
        self.store_key(key_id, &new_key, current_version + 1, true)?;

        Ok(new_key)
    }

    fn list_keys(&self) -> Result<Vec<KeyMetadata>, CryptoError> {
        let keys = self
            .keys
            .read()
            .map_err(|e| CryptoError::KeyManagement(e.to_string()))?;

        Ok(keys
            .values()
            .map(|e| KeyMetadata {
                key_id: e.key_id.clone(),
                version: e.version,
                created_at: e.created_at,
                active: e.active,
            })
            .collect())
    }
}

/// Environment variable-based key provider.
///
/// Reads keys from environment variables with a configurable prefix.
/// For example, with prefix "VEIL_KEY_", the key "default" would be
/// read from `VEIL_KEY_default`.
///
/// # Security Considerations
/// - Keys should be base64-encoded in environment variables
/// - Suitable for container deployments and secrets managers
/// - Rotation requires restarting the application
#[derive(Debug, Clone)]
pub struct EnvKeyProvider {
    prefix: String,
}

impl EnvKeyProvider {
    /// Create a new environment key provider.
    ///
    /// # Arguments
    /// * `prefix` - Environment variable prefix (e.g., "VEIL_KEY_")
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl KeyProvider for EnvKeyProvider {
    fn get_key(&self, key_id: &str) -> Result<SecretKey, CryptoError> {
        let var_name = format!("{}{}", self.prefix, key_id);

        let encoded = std::env::var(&var_name)
            .map_err(|_| CryptoError::KeyNotFound(format!("env::{}", var_name)))?;

        let key_bytes =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encoded)
                .map_err(|e| CryptoError::KeyManagement(format!("Invalid key encoding: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(CryptoError::KeyManagement(format!(
                "Invalid key length: expected 32, got {}",
                key_bytes.len()
            )));
        }

        Ok(SecretKey::from_slice(&key_bytes))
    }

    fn rotate_key(&self, _key_id: &str) -> Result<SecretKey, CryptoError> {
        Err(CryptoError::KeyManagement(
            "Key rotation not supported for environment provider".to_string(),
        ))
    }

    fn list_keys(&self) -> Result<Vec<KeyMetadata>, CryptoError> {
        let mut keys = Vec::new();
        let prefix_len = self.prefix.len();

        for (key, _) in std::env::vars() {
            if key.starts_with(&self.prefix) {
                let key_id = &key[prefix_len..];
                keys.push(KeyMetadata {
                    key_id: key_id.to_string(),
                    version: 1,
                    created_at: 0,
                    active: true,
                });
            }
        }

        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    use tempfile::TempDir;

    fn allow_plaintext_storage() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            std::env::set_var(PLAINTEXT_STORAGE_ENV, "1");
        });
    }

    #[test]
    fn test_secret_key_generate() {
        let key1 = SecretKey::generate();
        let key2 = SecretKey::generate();

        // Keys should be different
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_secret_key_from_slice() {
        let bytes = [42u8; 32];
        let key = SecretKey::from_slice(&bytes);

        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn test_local_key_provider_create_and_get() {
        allow_plaintext_storage();
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("keys.jsonl");

        let provider = LocalKeyProvider::new(&key_path).unwrap();

        // Create a key via rotation
        let key = provider.rotate_key("test-key").unwrap();

        // Retrieve the key
        let retrieved = provider.get_key("test-key").unwrap();

        assert_eq!(key.as_bytes(), retrieved.as_bytes());
    }

    #[test]
    fn test_local_key_provider_persistence() {
        allow_plaintext_storage();
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("keys.jsonl");

        let original_bytes: [u8; 32];

        // Create provider and key
        {
            let provider = LocalKeyProvider::new(&key_path).unwrap();
            let key = provider.rotate_key("persist-key").unwrap();
            original_bytes = *key.as_bytes();
        }

        // Reopen and verify
        {
            let provider = LocalKeyProvider::new(&key_path).unwrap();
            let key = provider.get_key("persist-key").unwrap();
            assert_eq!(key.as_bytes(), &original_bytes);
        }
    }

    #[test]
    fn test_local_key_provider_rotation() {
        allow_plaintext_storage();
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("keys.jsonl");

        let provider = LocalKeyProvider::new(&key_path).unwrap();

        let key1 = provider.rotate_key("rotate-key").unwrap();
        let key2 = provider.rotate_key("rotate-key").unwrap();

        // Keys should be different after rotation
        assert_ne!(key1.as_bytes(), key2.as_bytes());

        // Latest key should be returned
        let current = provider.get_key("rotate-key").unwrap();
        assert_eq!(current.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_local_key_provider_list_keys() {
        allow_plaintext_storage();
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("keys.jsonl");

        let provider = LocalKeyProvider::new(&key_path).unwrap();

        provider.rotate_key("key-a").unwrap();
        provider.rotate_key("key-b").unwrap();

        let keys = provider.list_keys().unwrap();

        assert_eq!(keys.len(), 2);
        assert!(keys.iter().any(|k| k.key_id == "key-a"));
        assert!(keys.iter().any(|k| k.key_id == "key-b"));
    }

    #[test]
    fn test_local_key_provider_key_not_found() {
        allow_plaintext_storage();
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("keys.jsonl");

        let provider = LocalKeyProvider::new(&key_path).unwrap();

        let result = provider.get_key("nonexistent");

        assert!(matches!(result, Err(CryptoError::KeyNotFound(_))));
    }

    #[test]
    fn test_env_key_provider() {
        // Set up test environment variable
        let key = SecretKey::generate();
        let encoded =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key.as_bytes());
        std::env::set_var("TEST_VEIL_KEY_mykey", &encoded);

        let provider = EnvKeyProvider::new("TEST_VEIL_KEY_");
        let retrieved = provider.get_key("mykey").unwrap();

        assert_eq!(retrieved.as_bytes(), key.as_bytes());

        // Clean up
        std::env::remove_var("TEST_VEIL_KEY_mykey");
    }

    #[test]
    fn test_env_key_provider_not_found() {
        let provider = EnvKeyProvider::new("NONEXISTENT_PREFIX_");

        let result = provider.get_key("missing");

        assert!(matches!(result, Err(CryptoError::KeyNotFound(_))));
    }

    #[test]
    fn test_env_key_provider_rotation_not_supported() {
        let provider = EnvKeyProvider::new("TEST_PREFIX_");

        let result = provider.rotate_key("any");

        assert!(matches!(result, Err(CryptoError::KeyManagement(_))));
    }
}
