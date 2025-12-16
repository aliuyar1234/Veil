//! File filtering with glob patterns.

use crate::error::BatchResult;
use glob::Pattern;
use std::path::Path;

/// Trait for filtering files.
pub trait FileFilter: Send + Sync {
    /// Check if a file should be processed.
    ///
    /// # Arguments
    /// * `path` - Path to check
    ///
    /// # Returns
    /// true if the file should be processed
    fn should_process(&self, path: &Path) -> bool;
}

/// Glob-based file filter.
#[derive(Debug, Clone)]
pub struct GlobFilter {
    /// Include patterns.
    include: Vec<Pattern>,
    /// Exclude patterns.
    exclude: Vec<Pattern>,
}

impl GlobFilter {
    /// Create a new glob filter.
    ///
    /// # Arguments
    /// * `include_patterns` - Patterns to include (empty = include all)
    /// * `exclude_patterns` - Patterns to exclude
    ///
    /// # Returns
    /// Result containing the filter or an error
    pub fn new(include_patterns: &[String], exclude_patterns: &[String]) -> BatchResult<Self> {
        let mut include = Vec::new();
        for pattern_str in include_patterns {
            let pattern = Pattern::new(pattern_str)?;
            include.push(pattern);
        }

        let mut exclude = Vec::new();
        for pattern_str in exclude_patterns {
            let pattern = Pattern::new(pattern_str)?;
            exclude.push(pattern);
        }

        Ok(Self { include, exclude })
    }

    /// Check if path matches any pattern.
    fn matches_any(path: &Path, patterns: &[Pattern]) -> bool {
        if patterns.is_empty() {
            return false;
        }

        let path_str = path.to_string_lossy();

        for pattern in patterns {
            // Try matching against full path
            if pattern.matches(&path_str) {
                return true;
            }

            // Try matching against just the filename
            if let Some(filename) = path.file_name() {
                if pattern.matches(&filename.to_string_lossy()) {
                    return true;
                }
            }
        }

        false
    }
}

impl FileFilter for GlobFilter {
    fn should_process(&self, path: &Path) -> bool {
        // If path matches any exclude pattern, skip it
        if Self::matches_any(path, &self.exclude) {
            return false;
        }

        // If there are include patterns, path must match at least one
        if !self.include.is_empty() {
            return Self::matches_any(path, &self.include);
        }

        // No patterns specified = include all
        true
    }
}

/// Apply filter to a list of file entries.
pub fn apply_filter<T, F>(entries: Vec<T>, filter: &F, get_path: impl Fn(&T) -> &Path) -> Vec<T>
where
    F: FileFilter,
{
    entries
        .into_iter()
        .filter(|entry| filter.should_process(get_path(entry)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_empty_filter_allows_all() {
        let filter = GlobFilter::new(&[], &[]).unwrap();
        assert!(filter.should_process(Path::new("test.txt")));
        assert!(filter.should_process(Path::new("data.csv")));
        assert!(filter.should_process(Path::new("any/path/file.json")));
    }

    #[test]
    fn test_include_patterns() {
        let filter = GlobFilter::new(&["*.csv".to_string()], &[]).unwrap();
        assert!(filter.should_process(Path::new("data.csv")));
        assert!(filter.should_process(Path::new("path/to/file.csv")));
        assert!(!filter.should_process(Path::new("data.txt")));
    }

    #[test]
    fn test_exclude_patterns() {
        let filter = GlobFilter::new(&[], &["*.log".to_string()]).unwrap();
        assert!(filter.should_process(Path::new("data.csv")));
        assert!(filter.should_process(Path::new("data.txt")));
        assert!(!filter.should_process(Path::new("debug.log")));
        assert!(!filter.should_process(Path::new("path/to/error.log")));
    }

    #[test]
    fn test_include_and_exclude() {
        let filter = GlobFilter::new(&["*.csv".to_string()], &["test_*.csv".to_string()]).unwrap();
        assert!(filter.should_process(Path::new("data.csv")));
        assert!(filter.should_process(Path::new("production.csv")));
        assert!(!filter.should_process(Path::new("test_data.csv")));
        assert!(!filter.should_process(Path::new("data.txt")));
    }

    #[test]
    fn test_complex_patterns() {
        let filter = GlobFilter::new(&["**/reports/*.pdf".to_string()], &[]).unwrap();
        assert!(filter.should_process(Path::new("reports/summary.pdf")));
        assert!(filter.should_process(Path::new("2024/reports/summary.pdf")));
        assert!(!filter.should_process(Path::new("data/summary.pdf")));
    }

    #[test]
    fn test_multiple_include_patterns() {
        let filter = GlobFilter::new(&["*.csv".to_string(), "*.json".to_string()], &[]).unwrap();
        assert!(filter.should_process(Path::new("data.csv")));
        assert!(filter.should_process(Path::new("config.json")));
        assert!(!filter.should_process(Path::new("data.txt")));
    }

    #[test]
    fn test_apply_filter() {
        let filter = GlobFilter::new(&["*.csv".to_string()], &[]).unwrap();
        let entries = vec![
            PathBuf::from("data.csv"),
            PathBuf::from("config.json"),
            PathBuf::from("report.csv"),
        ];

        let filtered = apply_filter(entries, &filter, |p| p.as_path());
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0], PathBuf::from("data.csv"));
        assert_eq!(filtered[1], PathBuf::from("report.csv"));
    }
}
