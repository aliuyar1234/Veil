//! Data discovery command implementation.

use std::fs;

use miette::{IntoDiagnostic, Result};

use veil_discovery::{DiscoveryOptions, ReportFormat, ReportGenerator, Scanner};

use crate::cli::{DiscoverArgs, ReportFormatArg};

/// Run the discover command.
pub fn run(args: DiscoverArgs, quiet: bool, json: bool) -> Result<()> {
    let options = DiscoveryOptions {
        root_path: args.path.clone(),
        include_patterns: if args.include.is_some() {
            args.include.unwrap()
        } else {
            vec!["**/*".to_string()]
        },
        exclude_patterns: args.exclude.unwrap_or_default(),
        sample_size: args.sample.unwrap_or(100 * 1024), // Default 100KB sample
        ..Default::default()
    };

    let scanner = Scanner::new(options);

    if !quiet && !json {
        eprintln!("Discovering PII in: {}", args.path.display());
    }

    // Run scan
    let result = scanner
        .scan()
        .map_err(|e| miette::miette!("Discovery error: {}", e))?;

    // Generate report
    let format = if json {
        ReportFormat::Json
    } else {
        match args.format {
            ReportFormatArg::Summary => ReportFormat::Summary,
            ReportFormatArg::Text => ReportFormat::Text,
            ReportFormatArg::Json => ReportFormat::Json,
        }
    };

    let report = ReportGenerator::generate(&result, format)
        .map_err(|e| miette::miette!("Report generation error: {}", e))?;

    // Output
    if let Some(output_path) = &args.output {
        fs::write(output_path, &report).into_diagnostic()?;
        if !quiet {
            eprintln!("Report written to: {}", output_path.display());
        }
    } else {
        println!("{}", report);
    }

    // Print summary if not quiet and not json
    if !quiet && !json {
        println!("\nDiscovery Summary");
        println!("=================");
        println!("  Files scanned: {}", result.total_files_scanned);
        println!(
            "  Files with PII: {}",
            result.statistics.total_files_with_pii
        );
        println!("  Total findings: {}", result.statistics.total_findings);
        println!("  Files skipped: {}", result.total_files_skipped);
    }

    Ok(())
}
