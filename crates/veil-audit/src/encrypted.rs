//! Encrypted audit logger implementation.
//!
//! Wraps the standard AuditLogger to provide at-rest encryption
//! for audit log files using AES-256-GCM.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use chrono::NaiveDate;
use zeroize::Zeroizing;

use crate::checksum::calculate_checksum_hmac;
use crate::entry::AuditEntry;
use crate::error::AuditError;
use crate::logger::AuditFilter;

/// Configuration for encrypted audit logging.
#[derive(Clone)]
pub struct EncryptionConfig {
    /// 256-bit AES key (32 bytes).
    key: Zeroizing<Vec<u8>>,
}

impl EncryptionConfig {
    /// Create a new encryption config with the given key.
    ///
    /// # Arguments
    /// * `key` - 32-byte AES-256 key
    ///
    /// # Errors
    /// Returns `AuditError::InvalidKey` if the key is not exactly 32 bytes.
    pub fn new(key: Vec<u8>) -> Result<Self, crate::error::AuditError> {
        if key.len() != 32 {
            return Err(crate::error::AuditError::InvalidKey {
                expected: 32,
                actual: key.len(),
            });
        }
        Ok(Self {
            key: Zeroizing::new(key),
        })
    }

    /// Get the encryption key.
    pub fn key(&self) -> &[u8] {
        self.key.as_slice()
    }
}

impl std::fmt::Debug for EncryptionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionConfig")
            .field("key", &"[REDACTED]")
            .finish()
    }
}

/// Encrypted audit logger for secure at-rest storage.
///
/// Each log entry is encrypted with AES-256-GCM before being written to disk.
/// The logger maintains the hash chain by computing checksums before encryption.
///
/// Note: Requires the `encryption` feature; otherwise `new` returns an error.
pub struct EncryptedAuditLogger {
    log_dir: PathBuf,
    config: EncryptionConfig,
    last_checksum: Option<String>,
}

impl EncryptedAuditLogger {
    /// Create a new encrypted audit logger.
    ///
    /// # Arguments
    /// * `log_dir` - Directory for encrypted log files
    /// * `config` - Encryption configuration with AES key
    pub fn new(log_dir: impl Into<PathBuf>, config: EncryptionConfig) -> Result<Self, AuditError> {
        if !cfg!(feature = "encryption") {
            let _ = config;
            return Err(AuditError::EncryptionUnavailable);
        }
        let log_dir = log_dir.into();

        // Create directory if it doesn't exist
        if !log_dir.exists() {
            fs::create_dir_all(&log_dir)?;
        }

        // Load last checksum from most recent log file
        let last_checksum = Self::load_last_checksum(&log_dir, &config)?;

        Ok(Self {
            log_dir,
            config,
            last_checksum,
        })
    }

    /// Log an encrypted audit entry.
    pub fn log(&mut self, mut entry: AuditEntry) -> Result<(), AuditError> {
        // Set previous checksum
        entry.previous_checksum = self.last_checksum.take();

        // Calculate checksum before encryption
        entry.checksum = calculate_checksum_hmac(&entry, self.config.key())?;

        // Serialize entry
        let json = serde_json::to_string(&entry)?;

        // Encrypt using veil-crypto
        let encrypted = encrypt_string(&json, &self.config)?;

        // Get today's log file
        let log_file = self.get_log_file_path(entry.timestamp.date_naive());

        // Append encrypted entry
        let mut options = OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&log_file)?;

        writeln!(file, "{}", encrypted)?;

        // Update last checksum
        self.last_checksum = Some(entry.checksum);

        Ok(())
    }

    /// Query encrypted audit entries.
    pub fn query(&self, filter: &AuditFilter) -> Result<Vec<AuditEntry>, AuditError> {
        let mut entries = Vec::new();

        // Get list of log files
        let log_files = self.get_log_files_in_range(filter.from, filter.to)?;

        for log_file in log_files {
            let file = File::open(&log_file)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }

                // Decrypt line
                let decrypted = decrypt_string(&line, &self.config)?;
                let entry: AuditEntry = serde_json::from_str(&decrypted)?;

                // Apply filters
                if let Some(ref ops) = filter.operations {
                    if !ops.contains(&entry.operation) {
                        continue;
                    }
                }

                if let Some(ref from) = filter.from {
                    if entry.timestamp < *from {
                        continue;
                    }
                }

                if let Some(ref to) = filter.to {
                    if entry.timestamp > *to {
                        continue;
                    }
                }

                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Get the log file path for a specific date.
    fn get_log_file_path(&self, date: NaiveDate) -> PathBuf {
        self.log_dir.join(format!("audit-{}.enc.jsonl", date))
    }

    /// Get log files in date range.
    fn get_log_files_in_range(
        &self,
        _from: Option<chrono::DateTime<chrono::Utc>>,
        _to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<PathBuf>, AuditError> {
        let mut files = Vec::new();

        if self.log_dir.exists() {
            for entry in fs::read_dir(&self.log_dir)? {
                let entry = entry?;
                let path = entry.path();
                // Look for encrypted log files
                if path.to_string_lossy().ends_with(".enc.jsonl") {
                    files.push(path);
                }
            }
        }

        files.sort();
        Ok(files)
    }

    /// Load the last checksum from the most recent log file.
    fn load_last_checksum(
        log_dir: &PathBuf,
        config: &EncryptionConfig,
    ) -> Result<Option<String>, AuditError> {
        if !log_dir.exists() {
            return Ok(None);
        }

        let mut log_files: Vec<_> = fs::read_dir(log_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.to_string_lossy().ends_with(".enc.jsonl"))
            .collect();

        log_files.sort();

        if let Some(latest) = log_files.last() {
            let file = File::open(latest)?;
            let reader = BufReader::new(file);

            if let Some(last_line) = reader.lines().map_while(Result::ok).last() {
                if !last_line.trim().is_empty() {
                    let decrypted = decrypt_string(&last_line, config)?;
                    let entry: AuditEntry = serde_json::from_str(&decrypted)?;
                    return Ok(Some(entry.checksum));
                }
            }
        }

        Ok(None)
    }
}

/// Encrypt a string using veil-crypto.
fn encrypt_string(plaintext: &str, config: &EncryptionConfig) -> Result<String, AuditError> {
    #[cfg(feature = "encryption")]
    {
        use veil_crypto::{encrypt, EncryptionConfig as CryptoConfig, OutputFormat};

        let crypto_config =
            CryptoConfig::with_format(config.key.as_slice().to_vec(), OutputFormat::Base64);

        let result = encrypt(plaintext.as_bytes(), &crypto_config).map_err(|e| {
            AuditError::Io(std::io::Error::other(format!("Encryption failed: {}", e)))
        })?;

        Ok(result.value)
    }

    #[cfg(not(feature = "encryption"))]
    {
        let _ = config;
        let _ = plaintext;
        Err(AuditError::EncryptionUnavailable)
    }
}

/// Decrypt a string using veil-crypto.
fn decrypt_string(ciphertext: &str, config: &EncryptionConfig) -> Result<String, AuditError> {
    #[cfg(feature = "encryption")]
    {
        use veil_crypto::{
            decrypt, CryptoResult, EncryptionConfig as CryptoConfig, OutputFormat, ProtectionMode,
        };

        let crypto_config =
            CryptoConfig::with_format(config.key.as_slice().to_vec(), OutputFormat::Base64);

        let result = CryptoResult::new(ciphertext.to_string(), ProtectionMode::Encrypt);

        let decrypted = decrypt(&result, &crypto_config).map_err(|e| {
            AuditError::Io(std::io::Error::other(format!("Decryption failed: {}", e)))
        })?;

        String::from_utf8(decrypted)
            .map_err(|e| AuditError::Io(std::io::Error::other(format!("Invalid UTF-8: {}", e))))
    }

    #[cfg(not(feature = "encryption"))]
    {
        let _ = config;
        let _ = ciphertext;
        Err(AuditError::EncryptionUnavailable)
    }
}

#[cfg(all(test, feature = "encryption"))]
mod tests {
    use super::*;
    use crate::entry::{AuditOutcome, AuditParameters};
    use crate::operation::AuditOperation;
    use tempfile::TempDir;

    fn test_key() -> Vec<u8> {
        vec![0u8; 32] // Test key (don't use in production!)
    }

    fn test_config() -> EncryptionConfig {
        EncryptionConfig::new(test_key()).unwrap()
    }

    #[test]
    fn test_encrypted_log_and_query() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = EncryptedAuditLogger::new(temp_dir.path(), test_config()).unwrap();

        let entry = AuditEntry::new(
            AuditOperation::Scan,
            AuditParameters::default(),
            AuditOutcome::success(),
        );

        logger.log(entry).unwrap();

        let entries = logger.query(&AuditFilter::default()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].operation, AuditOperation::Scan);
    }

    #[test]
    fn test_encrypted_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config();

        // Log an entry
        {
            let mut logger = EncryptedAuditLogger::new(temp_dir.path(), config.clone()).unwrap();
            let entry = AuditEntry::new(
                AuditOperation::Protect,
                AuditParameters::default(),
                AuditOutcome::success(),
            );
            logger.log(entry).unwrap();
        }

        // Reopen and verify
        {
            let logger = EncryptedAuditLogger::new(temp_dir.path(), config).unwrap();
            let entries = logger.query(&AuditFilter::default()).unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].operation, AuditOperation::Protect);
        }
    }

    #[test]
    fn test_encrypted_checksum_chain() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = EncryptedAuditLogger::new(temp_dir.path(), test_config()).unwrap();

        // Log multiple entries
        for _ in 0..3 {
            let entry = AuditEntry::new(
                AuditOperation::Scan,
                AuditParameters::default(),
                AuditOutcome::success(),
            );
            logger.log(entry).unwrap();
        }

        let entries = logger.query(&AuditFilter::default()).unwrap();
        assert_eq!(entries.len(), 3);

        // Verify chain
        assert!(entries[0].previous_checksum.is_none());
        assert_eq!(
            entries[1].previous_checksum,
            Some(entries[0].checksum.clone())
        );
        assert_eq!(
            entries[2].previous_checksum,
            Some(entries[1].checksum.clone())
        );
    }

    #[test]
    fn test_encrypted_log_file_extension() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = EncryptedAuditLogger::new(temp_dir.path(), test_config()).unwrap();

        let entry = AuditEntry::new(
            AuditOperation::Scan,
            AuditParameters::default(),
            AuditOutcome::success(),
        );
        logger.log(entry).unwrap();

        // Check that file has .enc.jsonl extension
        let files: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();

        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().ends_with(".enc.jsonl"));
    }

    #[test]
    fn test_encryption_config_creation() {
        let key = vec![1u8; 32];
        let config = EncryptionConfig::new(key.clone()).unwrap();
        assert_eq!(config.key(), &key[..]);
    }

    #[test]
    fn test_encryption_config_invalid_key_length() {
        let key = vec![0u8; 16]; // Too short
        let result = EncryptionConfig::new(key);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            crate::error::AuditError::InvalidKey {
                expected: 32,
                actual: 16
            }
        ));
    }

    #[test]
    fn test_encrypted_filter_by_operation() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = EncryptedAuditLogger::new(temp_dir.path(), test_config()).unwrap();

        // Log different operations
        logger
            .log(AuditEntry::new(
                AuditOperation::Scan,
                Default::default(),
                Default::default(),
            ))
            .unwrap();
        logger
            .log(AuditEntry::new(
                AuditOperation::Protect,
                Default::default(),
                Default::default(),
            ))
            .unwrap();
        logger
            .log(AuditEntry::new(
                AuditOperation::Scan,
                Default::default(),
                Default::default(),
            ))
            .unwrap();

        // Query only scans
        let filter = AuditFilter {
            operations: Some(vec![AuditOperation::Scan]),
            ..Default::default()
        };
        let entries = logger.query(&filter).unwrap();
        assert_eq!(entries.len(), 2);
    }
}

#[cfg(all(test, not(feature = "encryption")))]
mod tests_no_encryption {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_encrypted_logger_requires_feature() {
        let temp_dir = TempDir::new().unwrap();
        let config = EncryptionConfig::new(vec![0u8; 32]).unwrap();
        let result = EncryptedAuditLogger::new(temp_dir.path(), config);
        assert!(matches!(result, Err(AuditError::EncryptionUnavailable)));
    }
}
