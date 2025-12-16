//! Encrypted audit logger implementation.
//!
//! Wraps the standard AuditLogger to provide at-rest encryption
//! for audit log files using AES-256-GCM.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use chrono::NaiveDate;

use crate::checksum::calculate_checksum;
use crate::entry::AuditEntry;
use crate::error::AuditError;
use crate::logger::AuditFilter;

/// Configuration for encrypted audit logging.
#[derive(Clone)]
pub struct EncryptionConfig {
    /// 256-bit AES key (32 bytes).
    key: Vec<u8>,
}

impl EncryptionConfig {
    /// Create a new encryption config with the given key.
    ///
    /// # Arguments
    /// * `key` - 32-byte AES-256 key
    ///
    /// # Panics
    /// Panics if the key is not exactly 32 bytes.
    pub fn new(key: Vec<u8>) -> Self {
        assert_eq!(key.len(), 32, "Key must be 32 bytes for AES-256");
        Self { key }
    }

    /// Get the encryption key.
    pub fn key(&self) -> &[u8] {
        &self.key
    }
}

/// Encrypted audit logger for secure at-rest storage.
///
/// Each log entry is encrypted with AES-256-GCM before being written to disk.
/// The logger maintains the hash chain by computing checksums before encryption.
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
        entry.checksum = calculate_checksum(&entry);

        // Serialize entry
        let json = serde_json::to_string(&entry)?;

        // Encrypt using veil-crypto
        let encrypted = encrypt_string(&json, &self.config)?;

        // Get today's log file
        let log_file = self.get_log_file_path(entry.timestamp.date_naive());

        // Append encrypted entry
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;

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
    fn load_last_checksum(log_dir: &PathBuf, config: &EncryptionConfig) -> Result<Option<String>, AuditError> {
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

        let crypto_config = CryptoConfig::new(config.key.clone())
            .with_format(OutputFormat::Base64);

        let result = encrypt(plaintext.as_bytes(), &crypto_config)
            .map_err(|e| AuditError::Io(std::io::Error::other(
                format!("Encryption failed: {}", e),
            )))?;

        Ok(result.value)
    }

    #[cfg(not(feature = "encryption"))]
    {
        let _ = config;
        // When encryption feature is disabled, store as base64 (not secure, just for format compatibility)
        Ok(base64_encode(plaintext.as_bytes()))
    }
}

/// Decrypt a string using veil-crypto.
fn decrypt_string(ciphertext: &str, config: &EncryptionConfig) -> Result<String, AuditError> {
    #[cfg(feature = "encryption")]
    {
        use veil_crypto::{decrypt, EncryptionConfig as CryptoConfig, CryptoResult, ProtectionMode, OutputFormat};

        let crypto_config = CryptoConfig::new(config.key.clone())
            .with_format(OutputFormat::Base64);

        let result = CryptoResult::new(ciphertext.to_string(), ProtectionMode::Encrypt);

        let decrypted = decrypt(&result, &crypto_config)
            .map_err(|e| AuditError::Io(std::io::Error::other(
                format!("Decryption failed: {}", e),
            )))?;

        String::from_utf8(decrypted)
            .map_err(|e| AuditError::Io(std::io::Error::other(
                format!("Invalid UTF-8: {}", e),
            )))
    }

    #[cfg(not(feature = "encryption"))]
    {
        let _ = config;
        // When encryption feature is disabled, decode base64
        let bytes = base64_decode(ciphertext)?;
        String::from_utf8(bytes)
            .map_err(|e| AuditError::Io(std::io::Error::other(
                format!("Invalid UTF-8: {}", e),
            )))
    }
}

#[cfg(not(feature = "encryption"))]
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[(b0 >> 2) & 0x3F] as char);
        result.push(ALPHABET[((b0 << 4) | (b1 >> 4)) & 0x3F] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 << 2) | (b2 >> 6)) & 0x3F] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3F] as char);
        } else {
            result.push('=');
        }
    }
    result
}

#[cfg(not(feature = "encryption"))]
fn base64_decode(data: &str) -> Result<Vec<u8>, AuditError> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = Vec::new();
    let bytes: Vec<u8> = data.bytes().filter(|&b| b != b'=').collect();

    for chunk in bytes.chunks(4) {
        if chunk.len() < 2 {
            break;
        }

        let decode = |b: u8| -> Result<u8, AuditError> {
            ALPHABET.iter().position(|&x| x == b)
                .map(|p| p as u8)
                .ok_or_else(|| AuditError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid base64",
                )))
        };

        let b0 = decode(chunk[0])?;
        let b1 = decode(chunk[1])?;
        result.push((b0 << 2) | (b1 >> 4));

        if chunk.len() > 2 {
            let b2 = decode(chunk[2])?;
            result.push((b1 << 4) | (b2 >> 2));

            if chunk.len() > 3 {
                let b3 = decode(chunk[3])?;
                result.push((b2 << 6) | b3);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{AuditOutcome, AuditParameters};
    use crate::operation::AuditOperation;
    use tempfile::TempDir;

    fn test_key() -> Vec<u8> {
        vec![0u8; 32] // Test key (don't use in production!)
    }

    fn test_config() -> EncryptionConfig {
        EncryptionConfig::new(test_key())
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
        assert_eq!(entries[1].previous_checksum, Some(entries[0].checksum.clone()));
        assert_eq!(entries[2].previous_checksum, Some(entries[1].checksum.clone()));
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
        let config = EncryptionConfig::new(key.clone());
        assert_eq!(config.key(), &key[..]);
    }

    #[test]
    #[should_panic(expected = "Key must be 32 bytes")]
    fn test_encryption_config_invalid_key_length() {
        let key = vec![0u8; 16]; // Too short
        let _ = EncryptionConfig::new(key);
    }

    #[test]
    fn test_encrypted_filter_by_operation() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = EncryptedAuditLogger::new(temp_dir.path(), test_config()).unwrap();

        // Log different operations
        logger.log(AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default())).unwrap();
        logger.log(AuditEntry::new(AuditOperation::Protect, Default::default(), Default::default())).unwrap();
        logger.log(AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default())).unwrap();

        // Query only scans
        let filter = AuditFilter {
            operations: Some(vec![AuditOperation::Scan]),
            ..Default::default()
        };
        let entries = logger.query(&filter).unwrap();
        assert_eq!(entries.len(), 2);
    }
}
