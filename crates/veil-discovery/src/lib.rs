//! # veil-discovery
//!
//! Data discovery library for scanning filesystems to find PII.
//!
//! This crate provides functionality to:
//! - Scan directory trees for files containing PII
//! - Sample large files for efficient scanning
//! - Generate discovery reports in multiple formats
//! - Track file locations and PII categories found
//! - Support include/exclude patterns for filtering
//!
//! ## Example
//!
//! ```rust,no_run
//! use veil_discovery::{Scanner, DiscoveryOptions, ReportGenerator, ReportFormat};
//! use std::path::PathBuf;
//!
//! let options = DiscoveryOptions {
//!     root_path: PathBuf::from("/path/to/scan"),
//!     ..Default::default()
//! };
//!
//! let scanner = Scanner::new(options);
//! let result = scanner.scan().unwrap();
//!
//! // Generate a report
//! let report = ReportGenerator::generate(&result, ReportFormat::Summary).unwrap();
//! println!("{}", report);
//! ```

mod error;
mod report;
mod scanner;
mod types;

pub use error::{DiscoveryError, Result};
pub use report::{ReportFormat, ReportGenerator};
pub use scanner::Scanner;
pub use types::{
    DataSource, DiscoveredFile, DiscoveryOptions, DiscoveryResult, ErrorKind, PIILocation,
    ScanError, ScanProgress, ScanStatistics,
};

// Re-export commonly used types
pub use veil_detect::PiiCategory;
