//! Example of filtering PII from stdin to stdout.
//!
//! Run with: `echo "Email: test@example.com" | cargo run --example stdin_filter`

use std::io::{self, Write};
use veil_stream::{StreamEvent, StreamProcessor};

fn main() {
    let processor = StreamProcessor::default();

    eprintln!("Reading from stdin, filtering PII...");

    let result = processor
        .process(io::stdin(), |event| match event {
            StreamEvent::LineProcessed {
                line_number,
                findings,
                content: Some(content),
                ..
            } => {
                if findings.is_empty() {
                    // No PII, output as-is
                    print!("{}", content);
                } else {
                    // Has PII, redact it
                    let mut redacted = content.clone();
                    // Replace findings with [REDACTED]
                    // Note: In a real application, use veil-redact for proper redaction
                    for finding in findings.iter().rev() {
                        // Reverse order to maintain positions
                        let start = finding.start;
                        let end = finding.end;
                        if start < redacted.len() && end <= redacted.len() {
                            redacted.replace_range(start..end, "[REDACTED]");
                        }
                    }
                    print!("{}", redacted);

                    // Log to stderr
                    eprintln!(
                        "Line {}: Redacted {} PII instance(s)",
                        line_number,
                        findings.len()
                    );
                }
            }
            StreamEvent::LineProcessed { content: None, .. } => {}
            StreamEvent::EndOfStream {
                total_bytes,
                total_findings,
            } => {
                eprintln!(
                    "\nProcessed {} bytes, redacted {} PII instances",
                    total_bytes, total_findings
                );
            }
            _ => {}
        })
        .expect("Processing failed");

    // Write summary to stderr
    eprintln!("\nSummary:");
    eprintln!("  Lines processed: {}", result.items_processed);
    eprintln!("  Total findings: {}", result.findings.len());

    if !result.stats_by_category.is_empty() {
        eprintln!("\nPII types found:");
        for stat in &result.stats_by_category {
            eprintln!("  {:?}: {}", stat.category, stat.count);
        }
    }

    // Flush stdout
    io::stdout().flush().expect("Failed to flush stdout");
}
