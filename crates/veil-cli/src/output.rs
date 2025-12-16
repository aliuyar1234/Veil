//! Output formatting for CLI.

use miette::{IntoDiagnostic, Result};
use serde::Serialize;

use crate::commands::scan::ScanResult;

/// Print results as JSON.
pub fn print_json<T: Serialize>(value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value).into_diagnostic()?;
    println!("{}", json);
    Ok(())
}

/// Print scan result in human-readable format.
pub fn print_scan_result(result: &ScanResult) {
    println!("\n{} [{}]", result.file, result.format);
    println!(
        "{}",
        "=".repeat(result.file.len() + result.format.len() + 3)
    );

    if result.findings.is_empty() {
        println!("  No PII found");
    } else {
        for finding in &result.findings {
            println!(
                "  [{}] {} @ {} (confidence: {:.0}%)",
                finding.category,
                finding.text,
                finding.position,
                finding.confidence * 100.0
            );
        }
    }
}
