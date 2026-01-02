//! Filesystem scanner for PII discovery.

use crate::error::{DiscoveryError, Result};
use crate::types::{
    DataSource, DiscoveredFile, DiscoveryOptions, DiscoveryResult, ErrorKind, PIILocation,
    ScanError, ScanProgress, ScanStatistics,
};
use chrono::Utc;
use glob::Pattern;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;
use veil_detect::{DetectorRegistry, Finding};
use veil_fs::{walk_files, WalkFilesOptions};
use veil_parsers::ParseOptions;

/// Filesystem scanner that discovers PII in files.
pub struct Scanner {
    options: DiscoveryOptions,
    registry: DetectorRegistry,
    progress_callback: Option<Box<dyn Fn(ScanProgress) + Send>>,
}

impl Scanner {
    /// Create a new scanner with the given options.
    pub fn new(options: DiscoveryOptions) -> Self {
        Self {
            options,
            registry: DetectorRegistry::default(),
            progress_callback: None,
        }
    }

    /// Create a new scanner with custom detector registry.
    pub fn with_registry(options: DiscoveryOptions, registry: DetectorRegistry) -> Self {
        Self {
            options,
            registry,
            progress_callback: None,
        }
    }

    /// Set a progress callback function.
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(ScanProgress) + Send + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Execute the discovery scan.
    pub fn scan(&self) -> Result<DiscoveryResult> {
        let start_time = Instant::now();

        // Validate root path
        if !self.options.root_path.exists() {
            return Err(DiscoveryError::NotADirectory(
                self.options.root_path.display().to_string(),
            ));
        }

        if !self.options.root_path.is_dir() {
            return Err(DiscoveryError::NotADirectory(
                self.options.root_path.display().to_string(),
            ));
        }

        // Compile patterns
        let include_patterns = self.compile_patterns(&self.options.include_patterns)?;
        let exclude_patterns = self.compile_patterns(&self.options.exclude_patterns)?;

        let mut discovered_files = Vec::new();
        let mut total_files_scanned = 0;
        let mut total_files_skipped = 0;
        let mut total_bytes_scanned = 0u64;
        let mut errors = Vec::new();
        let mut statistics = ScanStatistics::default();

        let walker = walk_files(
            &self.options.root_path,
            WalkFilesOptions {
                follow_symlinks: self.options.follow_symlinks,
                max_depth: self.options.max_depth,
            },
        );

        // Scan files
        for entry in walker {
            match entry {
                Ok(entry) => {
                    let path = entry.path();

                    // Check if path matches patterns
                    if !self.should_scan_file(path, &include_patterns, &exclude_patterns) {
                        total_files_skipped += 1;
                        continue;
                    }

                    // Report progress
                    if self.options.report_progress {
                        if let Some(ref callback) = self.progress_callback {
                            callback(ScanProgress {
                                files_scanned: total_files_scanned,
                                files_with_pii: discovered_files.len(),
                                bytes_scanned: total_bytes_scanned,
                                current_file: Some(path.to_path_buf()),
                            });
                        }
                    }

                    // Scan the file
                    match self.scan_file(path) {
                        Ok(Some(discovered_file)) => {
                            total_bytes_scanned += discovered_file.bytes_scanned as u64;

                            // Update statistics
                            for category in &discovered_file.pii_categories {
                                *statistics
                                    .findings_by_category
                                    .entry(category.clone())
                                    .or_insert(0) += discovered_file
                                    .locations
                                    .iter()
                                    .filter(|loc| &loc.category == category)
                                    .count();
                                *statistics
                                    .files_by_category
                                    .entry(category.clone())
                                    .or_insert(0) += 1;
                            }
                            statistics.total_findings += discovered_file.finding_count;
                            statistics.total_files_with_pii += 1;

                            discovered_files.push(discovered_file);
                        }
                        Ok(None) => {
                            // No PII found, but file was scanned
                            if let Ok(metadata) = fs::metadata(path) {
                                total_bytes_scanned += metadata
                                    .len()
                                    .min(self.options.max_file_size.unwrap_or(usize::MAX) as u64);
                            }
                        }
                        Err(e) => {
                            let error_kind = match &e {
                                DiscoveryError::Io(io_err) => match io_err.kind() {
                                    std::io::ErrorKind::PermissionDenied => {
                                        ErrorKind::PermissionDenied
                                    }
                                    std::io::ErrorKind::NotFound => ErrorKind::NotFound,
                                    _ => ErrorKind::IoError,
                                },
                                DiscoveryError::Parse(_) => ErrorKind::ParseError,
                                _ => ErrorKind::Other,
                            };

                            errors.push(ScanError {
                                path: path.to_path_buf(),
                                message: e.to_string(),
                                kind: error_kind,
                            });
                            total_files_skipped += 1;
                        }
                    }

                    total_files_scanned += 1;
                }
                Err(e) => {
                    errors.push(ScanError {
                        path: PathBuf::from(e.path().unwrap_or_else(|| Path::new("unknown"))),
                        message: e.to_string(),
                        kind: ErrorKind::IoError,
                    });
                }
            }
        }

        statistics.scan_duration_secs = start_time.elapsed().as_secs_f64();

        Ok(DiscoveryResult {
            timestamp: Utc::now(),
            source: DataSource::Filesystem,
            root_path: self.options.root_path.clone(),
            discovered_files,
            total_files_scanned,
            total_files_skipped,
            total_bytes_scanned,
            statistics,
            errors,
        })
    }

    /// Scan a single file for PII.
    fn scan_file(&self, path: &Path) -> Result<Option<DiscoveredFile>> {
        let metadata = fs::metadata(path)?;
        let size = metadata.len();

        // Determine if we need to sample
        let max_size = self.options.max_file_size.unwrap_or(usize::MAX);
        let sampled = size > max_size as u64;
        let bytes_to_read = if sampled {
            self.options.sample_size
        } else {
            size as usize
        };

        // Read file content
        let mut file = fs::File::open(path)?;
        let mut buffer = vec![0u8; bytes_to_read];
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        // Parse the file
        let parse_options = ParseOptions {
            format: None, // Auto-detect
            max_size_bytes: Some(bytes_to_read),
            ..Default::default()
        };

        let parse_result = match veil_parsers::parse_bytes(&buffer, &parse_options) {
            Ok(result) => result,
            Err(veil_parsers::ParseError::UnsupportedFormat(_)) => {
                // Skip unsupported formats
                return Ok(None);
            }
            Err(e) => return Err(e.into()),
        };

        // Detect PII
        let findings = veil_detect::detect_pii(&parse_result.segments, &self.registry);

        if findings.is_empty() {
            return Ok(None);
        }

        // Convert findings to PIILocation
        let mut locations = Vec::new();
        let mut pii_categories = Vec::new();
        let mut category_set = std::collections::HashSet::new();

        for finding in &findings {
            if category_set.insert(finding.category.clone()) {
                pii_categories.push(finding.category.clone());
            }

            locations.push(self.finding_to_location(finding));
        }

        Ok(Some(DiscoveredFile {
            path: path.to_path_buf(),
            size,
            sampled,
            bytes_scanned: bytes_read,
            pii_categories,
            locations,
            format: Some(format!("{:?}", parse_result.metadata.format)),
            finding_count: findings.len(),
        }))
    }

    /// Convert a Finding to a PIILocation.
    fn finding_to_location(&self, finding: &Finding) -> PIILocation {
        // Use .as_str() to get the actual value (Display is intentionally redacted)
        let snippet = if self.options.include_snippets {
            Some(finding.matched_text.as_str().to_string())
        } else {
            None
        };

        PIILocation {
            category: finding.category.clone(),
            line: None,
            column: None,
            byte_offset: finding.start,
            byte_length: finding.end - finding.start,
            snippet,
            validated: finding.is_valid(),
        }
    }

    /// Check if a file should be scanned based on include/exclude patterns.
    fn should_scan_file(
        &self,
        path: &Path,
        include_patterns: &[Pattern],
        exclude_patterns: &[Pattern],
    ) -> bool {
        let path_str = path.to_string_lossy();

        // Check exclude patterns first
        for pattern in exclude_patterns {
            if pattern.matches(&path_str) {
                return false;
            }
        }

        // Check include patterns
        for pattern in include_patterns {
            if pattern.matches(&path_str) {
                return true;
            }
        }

        false
    }

    /// Compile glob patterns.
    fn compile_patterns(&self, patterns: &[String]) -> Result<Vec<Pattern>> {
        patterns
            .iter()
            .map(|p| Pattern::new(p).map_err(DiscoveryError::from))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;
    use veil_detect::PiiCategory;

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let options = DiscoveryOptions {
            root_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let scanner = Scanner::new(options);
        let result = scanner.scan().unwrap();

        assert_eq!(result.total_files_scanned, 0);
        assert_eq!(result.discovered_files.len(), 0);
    }

    #[test]
    fn test_scan_file_with_email() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "Contact: john.doe@example.com").unwrap();

        let options = DiscoveryOptions {
            root_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let scanner = Scanner::new(options);
        let result = scanner.scan().unwrap();

        assert_eq!(result.total_files_scanned, 1);
        assert_eq!(result.discovered_files.len(), 1);

        let discovered = &result.discovered_files[0];
        assert_eq!(discovered.path, file_path);
        assert!(discovered.pii_categories.contains(&PiiCategory::Email));
        assert!(discovered.finding_count > 0);
    }

    #[test]
    fn test_scan_file_without_pii() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "This is a normal file without any PII.").unwrap();

        let options = DiscoveryOptions {
            root_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let scanner = Scanner::new(options);
        let result = scanner.scan().unwrap();

        assert_eq!(result.total_files_scanned, 1);
        assert_eq!(result.discovered_files.len(), 0);
    }

    #[test]
    fn test_exclude_patterns() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file in a .git directory
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        let git_file = git_dir.join("config");
        let mut file = fs::File::create(&git_file).unwrap();
        writeln!(file, "email = test@example.com").unwrap();

        // Create a normal file
        let normal_file = temp_dir.path().join("data.txt");
        let mut file = fs::File::create(&normal_file).unwrap();
        writeln!(file, "email = test@example.com").unwrap();

        let options = DiscoveryOptions {
            root_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let scanner = Scanner::new(options);
        let result = scanner.scan().unwrap();

        // Should only find PII in data.txt, not in .git/config
        assert_eq!(result.discovered_files.len(), 1);
        assert_eq!(result.discovered_files[0].path, normal_file);
    }

    #[test]
    fn test_sampling_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");

        let mut file = fs::File::create(&file_path).unwrap();
        // Write a large file with PII at the beginning
        writeln!(file, "Email: test@example.com").unwrap();
        for _ in 0..10000 {
            writeln!(file, "This is filler content to make the file large.").unwrap();
        }

        let options = DiscoveryOptions {
            root_path: temp_dir.path().to_path_buf(),
            max_file_size: Some(1024), // 1KB max
            sample_size: 512,          // Sample 512 bytes
            ..Default::default()
        };

        let scanner = Scanner::new(options);
        let result = scanner.scan().unwrap();

        assert_eq!(result.discovered_files.len(), 1);
        let discovered = &result.discovered_files[0];
        assert!(discovered.sampled);
        assert_eq!(discovered.bytes_scanned, 512);
        assert!(discovered.pii_categories.contains(&PiiCategory::Email));
    }
}
