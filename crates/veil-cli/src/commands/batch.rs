//! Batch processing command implementation.

use std::path::PathBuf;

use miette::Result;

use veil_batch::{BatchJob, BatchOptions, BatchProcessor, DefaultBatchProcessor};
use veil_detect::DetectorRegistry;

use crate::cli::BatchArgs;
use crate::output;

/// Run the batch command.
pub fn run(args: BatchArgs, quiet: bool, json: bool) -> Result<()> {
    let options = BatchOptions {
        recursive: true,
        max_file_size: (args.max_size * 1024 * 1024) as usize, // Convert MB to bytes
        max_parallelism: Some(args.jobs),
        include_patterns: args.include.unwrap_or_default(),
        exclude_patterns: args.exclude.unwrap_or_default(),
        archive_password: if args.zip { args.zip_password } else { None },
        ..Default::default()
    };

    let job = BatchJob::new(args.sources.clone(), options);
    let processor = DefaultBatchProcessor::new();

    if quiet || json {
        // Run without progress
        let result = processor
            .process(&job)
            .map_err(|e| miette::miette!("Batch processing error: {}", e))?;

        if json {
            output::print_json(&BatchOutput::from_result(&result, &args.sources))?;
        } else {
            println!(
                "Processed: {} | Failed: {} | Skipped: {}",
                result.summary.successful, result.summary.failed, result.summary.skipped
            );
        }
    } else {
        // Run with progress indication
        eprintln!("Processing files...");

        let result = processor
            .process(&job)
            .map_err(|e| miette::miette!("Batch processing error: {}", e))?;

        // Print summary
        println!("\nBatch Processing Complete");
        println!("========================");
        println!("  Successful: {}", result.summary.successful);
        println!("  Failed:     {}", result.summary.failed);
        println!("  Skipped:    {}", result.summary.skipped);
        println!(
            "  Duration:   {:.2}s",
            result.summary.duration.as_secs_f64()
        );

        // Run PII detection on results
        if !result.results.is_empty() {
            let registry = DetectorRegistry::default();
            let mut total_findings = 0;

            println!("\nPII Findings:");
            for file_result in &result.results {
                let findings = registry.detect_all(&file_result.result.segments);
                if !findings.is_empty() {
                    println!(
                        "  {}: {} findings",
                        file_result.path.display(),
                        findings.len()
                    );
                    total_findings += findings.len();
                }
            }
            println!("\nTotal PII findings: {}", total_findings);
        }
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct BatchOutput {
    sources: Vec<String>,
    successful: usize,
    failed: usize,
    skipped: usize,
    duration_secs: f64,
}

impl BatchOutput {
    fn from_result(result: &veil_batch::BatchResultType, sources: &[PathBuf]) -> Self {
        Self {
            sources: sources.iter().map(|p| p.display().to_string()).collect(),
            successful: result.summary.successful,
            failed: result.summary.failed,
            skipped: result.summary.skipped,
            duration_secs: result.summary.duration.as_secs_f64(),
        }
    }
}
