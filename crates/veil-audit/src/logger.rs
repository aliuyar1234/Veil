//! Audit logger implementation.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, Utc};

use crate::checksum::calculate_checksum;
use crate::entry::AuditEntry;
use crate::error::AuditError;
use crate::operation::AuditOperation;

/// Filter for querying audit logs.
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    /// Start of date range.
    pub from: Option<DateTime<Utc>>,
    /// End of date range.
    pub to: Option<DateTime<Utc>>,
    /// Filter by operation types.
    pub operations: Option<Vec<AuditOperation>>,
    /// Filter by file paths.
    pub paths: Option<Vec<PathBuf>>,
}

/// Audit logger for recording PII operations.
pub struct AuditLogger {
    log_dir: PathBuf,
    last_checksum: Option<String>,
}

impl AuditLogger {
    /// Create a new audit logger.
    pub fn new(log_dir: impl Into<PathBuf>) -> Result<Self, AuditError> {
        let log_dir = log_dir.into();

        // Create directory if it doesn't exist
        if !log_dir.exists() {
            fs::create_dir_all(&log_dir)?;
        }

        // Load last checksum from most recent log file
        let last_checksum = Self::load_last_checksum(&log_dir)?;

        Ok(Self {
            log_dir,
            last_checksum,
        })
    }

    /// Log an audit entry.
    pub fn log(&mut self, mut entry: AuditEntry) -> Result<(), AuditError> {
        // Set previous checksum
        entry.previous_checksum = self.last_checksum.take();

        // Calculate checksum
        entry.checksum = calculate_checksum(&entry);

        // Get today's log file
        let log_file = self.get_log_file_path(entry.timestamp.date_naive());

        // Append to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;

        let json = serde_json::to_string(&entry)?;
        writeln!(file, "{}", json)?;

        // Update last checksum
        self.last_checksum = Some(entry.checksum);

        Ok(())
    }

    /// Query audit entries matching a filter.
    pub fn query(&self, filter: &AuditFilter) -> Result<Vec<AuditEntry>, AuditError> {
        let mut entries = Vec::new();

        // Get list of log files to read
        let log_files = self.get_log_files_in_range(filter.from, filter.to)?;

        for log_file in log_files {
            let file = File::open(&log_file)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }

                let entry: AuditEntry = serde_json::from_str(&line)?;

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
        self.log_dir.join(format!("audit-{}.jsonl", date))
    }

    /// Get log files in date range.
    fn get_log_files_in_range(
        &self,
        _from: Option<DateTime<Utc>>,
        _to: Option<DateTime<Utc>>,
    ) -> Result<Vec<PathBuf>, AuditError> {
        let mut files = Vec::new();

        if self.log_dir.exists() {
            for entry in fs::read_dir(&self.log_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "jsonl") {
                    files.push(path);
                }
            }
        }

        files.sort();
        Ok(files)
    }

    /// Load the last checksum from the most recent log file.
    fn load_last_checksum(log_dir: &PathBuf) -> Result<Option<String>, AuditError> {
        if !log_dir.exists() {
            return Ok(None);
        }

        let mut log_files: Vec<_> = fs::read_dir(log_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
            .collect();

        log_files.sort();

        if let Some(latest) = log_files.last() {
            let file = File::open(latest)?;
            let reader = BufReader::new(file);

            if let Some(last_line) = reader.lines().map_while(Result::ok).last() {
                if !last_line.trim().is_empty() {
                    let entry: AuditEntry = serde_json::from_str(&last_line)?;
                    return Ok(Some(entry.checksum));
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{AuditOutcome, AuditParameters};
    use tempfile::TempDir;

    #[test]
    fn test_log_and_query() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = AuditLogger::new(temp_dir.path()).unwrap();

        let entry = AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());

        logger.log(entry).unwrap();

        let entries = logger.query(&AuditFilter::default()).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_logger_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("new_dir");

        assert!(!log_dir.exists());
        let _logger = AuditLogger::new(&log_dir).unwrap();
        assert!(log_dir.exists());
    }

    #[test]
    fn test_log_multiple_entries() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = AuditLogger::new(temp_dir.path()).unwrap();

        for _ in 0..5 {
            let entry =
                AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());
            logger.log(entry).unwrap();
        }

        let entries = logger.query(&AuditFilter::default()).unwrap();
        assert_eq!(entries.len(), 5);
    }

    #[test]
    fn test_log_with_checksums() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = AuditLogger::new(temp_dir.path()).unwrap();

        let entry1 =
            AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());
        logger.log(entry1).unwrap();

        let entry2 =
            AuditEntry::new(AuditOperation::Protect, Default::default(), Default::default());
        logger.log(entry2).unwrap();

        let entries = logger.query(&AuditFilter::default()).unwrap();
        assert_eq!(entries.len(), 2);

        // First entry should have no previous checksum
        assert!(entries[0].previous_checksum.is_none());
        // Second entry should have first entry's checksum as previous
        assert_eq!(entries[1].previous_checksum, Some(entries[0].checksum.clone()));
    }

    #[test]
    fn test_query_filter_by_operation() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = AuditLogger::new(temp_dir.path()).unwrap();

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

        // Query only protect
        let filter = AuditFilter {
            operations: Some(vec![AuditOperation::Protect]),
            ..Default::default()
        };
        let entries = logger.query(&filter).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_query_empty_log() {
        let temp_dir = TempDir::new().unwrap();
        let logger = AuditLogger::new(temp_dir.path()).unwrap();

        let entries = logger.query(&AuditFilter::default()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_log_file_format() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = AuditLogger::new(temp_dir.path()).unwrap();

        let entry =
            AuditEntry::new(AuditOperation::Scan, Default::default(), Default::default());
        logger.log(entry).unwrap();

        // Check that a .jsonl file was created
        let files: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();

        assert_eq!(files.len(), 1);
        assert!(files[0].extension().is_some_and(|e| e == "jsonl"));
    }

    #[test]
    fn test_audit_filter_default() {
        let filter = AuditFilter::default();
        assert!(filter.from.is_none());
        assert!(filter.to.is_none());
        assert!(filter.operations.is_none());
        assert!(filter.paths.is_none());
    }

    #[test]
    fn test_log_preserves_entry_data() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = AuditLogger::new(temp_dir.path()).unwrap();

        let mut params = AuditParameters::default();
        params.input.push(PathBuf::from("/test/file.txt"));
        params.policy = Some("strict".to_string());

        let outcome = AuditOutcome::failure("test error");

        let entry = AuditEntry::new(AuditOperation::Protect, params, outcome);
        logger.log(entry).unwrap();

        let entries = logger.query(&AuditFilter::default()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].operation, AuditOperation::Protect);
        assert_eq!(entries[0].parameters.input.len(), 1);
        assert_eq!(entries[0].parameters.policy, Some("strict".to_string()));
        assert!(!entries[0].outcome.success);
        assert_eq!(entries[0].outcome.error, Some("test error".to_string()));
    }
}
