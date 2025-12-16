//! Report generation for discovery results.

use crate::error::Result;
use crate::types::DiscoveryResult;
use std::fmt::Write as FmtWrite;

/// Format for the discovery report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// JSON format
    Json,
    /// Human-readable text format
    Text,
    /// Summary text format
    Summary,
}

/// Generate a report from discovery results.
pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate a report in the specified format.
    pub fn generate(result: &DiscoveryResult, format: ReportFormat) -> Result<String> {
        match format {
            ReportFormat::Json => Self::generate_json(result),
            ReportFormat::Text => Self::generate_text(result),
            ReportFormat::Summary => Self::generate_summary(result),
        }
    }

    /// Generate JSON report.
    fn generate_json(result: &DiscoveryResult) -> Result<String> {
        let json = serde_json::to_string_pretty(result)?;
        Ok(json)
    }

    /// Generate detailed text report.
    fn generate_text(result: &DiscoveryResult) -> Result<String> {
        let mut output = String::new();

        writeln!(output, "PII Discovery Report").unwrap();
        writeln!(output, "===================").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "Scan Time: {}", result.timestamp).unwrap();
        writeln!(output, "Root Path: {}", result.root_path.display()).unwrap();
        writeln!(output, "Source: {:?}", result.source).unwrap();
        writeln!(output).unwrap();

        writeln!(output, "Scan Statistics").unwrap();
        writeln!(output, "---------------").unwrap();
        writeln!(
            output,
            "Total Files Scanned: {}",
            result.total_files_scanned
        )
        .unwrap();
        writeln!(
            output,
            "Total Files Skipped: {}",
            result.total_files_skipped
        )
        .unwrap();
        writeln!(
            output,
            "Total Files with PII: {}",
            result.statistics.total_files_with_pii
        )
        .unwrap();
        writeln!(
            output,
            "Total Findings: {}",
            result.statistics.total_findings
        )
        .unwrap();
        writeln!(
            output,
            "Total Bytes Scanned: {} ({:.2} MB)",
            result.total_bytes_scanned,
            result.total_bytes_scanned as f64 / 1024.0 / 1024.0
        )
        .unwrap();
        writeln!(
            output,
            "Scan Duration: {:.2} seconds",
            result.statistics.scan_duration_secs
        )
        .unwrap();
        writeln!(output).unwrap();

        if !result.statistics.findings_by_category.is_empty() {
            writeln!(output, "Findings by PII Category").unwrap();
            writeln!(output, "------------------------").unwrap();
            let mut categories: Vec<_> = result.statistics.findings_by_category.iter().collect();
            categories.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

            for (category, count) in categories {
                writeln!(output, "  {:?}: {}", category, count).unwrap();
            }
            writeln!(output).unwrap();
        }

        if !result.discovered_files.is_empty() {
            writeln!(output, "Discovered Files").unwrap();
            writeln!(output, "----------------").unwrap();
            for (idx, file) in result.discovered_files.iter().enumerate() {
                writeln!(output, "{}. {}", idx + 1, file.path.display()).unwrap();
                writeln!(
                    output,
                    "   Size: {} bytes{}",
                    file.size,
                    if file.sampled { " (sampled)" } else { "" }
                )
                .unwrap();
                writeln!(output, "   Format: {:?}", file.format).unwrap();
                writeln!(output, "   Findings: {}", file.finding_count).unwrap();
                writeln!(output, "   PII Categories: {:?}", file.pii_categories).unwrap();

                if !file.locations.is_empty() {
                    writeln!(output, "   Locations:").unwrap();
                    for loc in &file.locations {
                        let position = if let (Some(line), Some(col)) = (loc.line, loc.column) {
                            format!("Line {}, Col {}", line, col)
                        } else {
                            format!("Offset {}", loc.byte_offset)
                        };
                        writeln!(
                            output,
                            "     - {:?} at {} (validated: {})",
                            loc.category, position, loc.validated
                        )
                        .unwrap();
                    }
                }
                writeln!(output).unwrap();
            }
        }

        if !result.errors.is_empty() {
            writeln!(output, "Errors Encountered").unwrap();
            writeln!(output, "------------------").unwrap();
            for error in &result.errors {
                writeln!(
                    output,
                    "  {:?}: {} - {}",
                    error.kind,
                    error.path.display(),
                    error.message
                )
                .unwrap();
            }
            writeln!(output).unwrap();
        }

        Ok(output)
    }

    /// Generate summary text report.
    fn generate_summary(result: &DiscoveryResult) -> Result<String> {
        let mut output = String::new();

        writeln!(output, "PII Discovery Summary").unwrap();
        writeln!(output, "====================").unwrap();
        writeln!(output).unwrap();
        writeln!(
            output,
            "Files Scanned: {} | With PII: {} | Total Findings: {}",
            result.total_files_scanned,
            result.statistics.total_files_with_pii,
            result.statistics.total_findings
        )
        .unwrap();
        writeln!(
            output,
            "Scan Duration: {:.2}s",
            result.statistics.scan_duration_secs
        )
        .unwrap();
        writeln!(output).unwrap();

        if !result.statistics.findings_by_category.is_empty() {
            writeln!(output, "PII Categories Found:").unwrap();
            let mut categories: Vec<_> = result.statistics.findings_by_category.iter().collect();
            categories.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

            for (category, count) in categories {
                let file_count = result
                    .statistics
                    .files_by_category
                    .get(category)
                    .unwrap_or(&0);
                writeln!(
                    output,
                    "  {:?}: {} findings in {} files",
                    category, count, file_count
                )
                .unwrap();
            }
        }

        Ok(output)
    }

    /// Generate a CSV report of discovered files.
    pub fn generate_csv(result: &DiscoveryResult) -> Result<String> {
        let mut output = String::new();

        // Header
        writeln!(
            output,
            "Path,Size,Sampled,Bytes Scanned,Format,Finding Count,PII Categories"
        )
        .unwrap();

        // Data rows
        for file in &result.discovered_files {
            let categories = file
                .pii_categories
                .iter()
                .map(|c| format!("{:?}", c))
                .collect::<Vec<_>>()
                .join(";");

            writeln!(
                output,
                "\"{}\",{},{},{},{:?},{},\"{}\"",
                file.path.display(),
                file.size,
                file.sampled,
                file.bytes_scanned,
                file.format,
                file.finding_count,
                categories
            )
            .unwrap();
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DataSource, DiscoveredFile, ScanStatistics};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use veil_detect::PiiCategory;

    fn create_test_result() -> DiscoveryResult {
        let mut findings_by_category = HashMap::new();
        findings_by_category.insert(PiiCategory::Email, 5);
        findings_by_category.insert(PiiCategory::Phone, 2);

        let mut files_by_category = HashMap::new();
        files_by_category.insert(PiiCategory::Email, 2);
        files_by_category.insert(PiiCategory::Phone, 1);

        DiscoveryResult {
            timestamp: Utc::now(),
            source: DataSource::Filesystem,
            root_path: PathBuf::from("/test"),
            discovered_files: vec![DiscoveredFile {
                path: PathBuf::from("/test/file.txt"),
                size: 1024,
                sampled: false,
                bytes_scanned: 1024,
                pii_categories: vec![PiiCategory::Email],
                locations: vec![],
                format: Some("Text".to_string()),
                finding_count: 5,
            }],
            total_files_scanned: 10,
            total_files_skipped: 2,
            total_bytes_scanned: 10240,
            statistics: ScanStatistics {
                findings_by_category,
                files_by_category,
                total_findings: 7,
                total_files_with_pii: 2,
                scan_duration_secs: 1.5,
            },
            errors: vec![],
        }
    }

    #[test]
    fn test_generate_json() {
        let result = create_test_result();
        let json = ReportGenerator::generate(&result, ReportFormat::Json).unwrap();
        // JSON uses lowercase for enum variants
        assert!(json.contains("\"filesystem\""));
        assert!(json.contains("\"total_files_scanned\""));
        assert!(json.contains("10"));
    }

    #[test]
    fn test_generate_text() {
        let result = create_test_result();
        let text = ReportGenerator::generate(&result, ReportFormat::Text).unwrap();
        assert!(text.contains("PII Discovery Report"));
        assert!(text.contains("Total Files Scanned: 10"));
        assert!(text.contains("Email"));
    }

    #[test]
    fn test_generate_summary() {
        let result = create_test_result();
        let summary = ReportGenerator::generate(&result, ReportFormat::Summary).unwrap();
        assert!(summary.contains("PII Discovery Summary"));
        assert!(summary.contains("Files Scanned: 10"));
        assert!(summary.contains("Email"));
    }

    #[test]
    fn test_generate_csv() {
        let result = create_test_result();
        let csv = ReportGenerator::generate_csv(&result).unwrap();
        assert!(csv.contains("Path,Size,Sampled"));
        assert!(csv.contains("/test/file.txt"));
    }
}
