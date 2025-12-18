//! Basic example of streaming PII detection.

use std::io::Cursor;
use veil_stream::{ProcessingMode, StreamConfig, StreamEvent, StreamProcessor};

fn main() {
    // Example 1: Line-by-line processing
    println!("=== Example 1: Line-by-line processing ===\n");

    let data = r#"User registration data:
Email: john.doe@example.com
Phone: +1-555-0123
SSN: 123-45-6789
Address: 123 Main Street
"#;

    let processor = StreamProcessor::default();
    let reader = Cursor::new(data);

    let result = processor
        .process(reader, |event| match event {
            StreamEvent::LineProcessed {
                line_number,
                findings,
                content,
                ..
            } => {
                if !findings.is_empty() {
                    println!(
                        "Line {}: Found {} PII instance(s)",
                        line_number,
                        findings.len()
                    );
                    for finding in findings {
                        println!(
                            "  - {}: {} (confidence: {:.2})",
                            finding.category, finding.matched_text, finding.confidence
                        );
                    }
                    if let Some(content) = content {
                        println!("  Content: {}", content.trim());
                    }
                }
            }
            StreamEvent::EndOfStream {
                total_bytes,
                total_findings,
            } => {
                println!(
                    "\nProcessed {} bytes, found {} PII instances total",
                    total_bytes, total_findings
                );
            }
            _ => {}
        })
        .expect("Processing failed");

    println!("\nResult summary:");
    println!("  Lines processed: {}", result.items_processed);
    println!("  Total findings: {}", result.findings.len());
    println!("\nFindings by category:");
    for stat in &result.stats_by_category {
        println!("  {:?}: {}", stat.category, stat.count);
    }

    // Example 2: Chunk-by-chunk processing
    println!("\n\n=== Example 2: Chunk-by-chunk processing ===\n");

    let config = StreamConfig {
        mode: ProcessingMode::ChunkByChunk,
        chunk_size: 64,         // Small chunks for demonstration
        include_content: false, // Don't include content in events
        ..Default::default()
    };

    let processor = StreamProcessor::new(config);

    let data = "This is a stream of data that contains sensitive information like \
                test@example.com and phone numbers like 555-0123. We process it in \
                small chunks to demonstrate streaming capability.";

    let reader = Cursor::new(data);

    let result = processor
        .process(reader, |event| match event {
            StreamEvent::ChunkProcessed {
                offset,
                bytes_processed,
                findings,
                ..
            } => {
                println!(
                    "Chunk at offset {}: {} bytes, {} findings",
                    offset,
                    bytes_processed,
                    findings.len()
                );
            }
            StreamEvent::EndOfStream {
                total_bytes,
                total_findings,
            } => {
                println!(
                    "\nCompleted: {} bytes, {} findings",
                    total_bytes, total_findings
                );
            }
            _ => {}
        })
        .expect("Processing failed");

    println!("\nChunk processing summary:");
    println!("  Chunks processed: {}", result.items_processed);
    println!("  Total bytes: {}", result.total_bytes);

    // Example 3: Simple processing without event handlers
    println!("\n\n=== Example 3: Simple processing (no events) ===\n");

    let processor = StreamProcessor::default();
    let data = "Just get the results: admin@company.com and 555-9876";
    let reader = Cursor::new(data);

    let result = processor.process_all(reader).expect("Processing failed");

    println!("Found {} PII instances:", result.findings.len());
    for finding in &result.findings {
        println!(
            "  - {} at position {}..{}: {}",
            finding.category, finding.start, finding.end, finding.matched_text
        );
    }

    // Example 4: Category filtering
    println!("\n\n=== Example 4: Category filtering ===\n");

    use veil_detect::PiiCategory;

    let config = StreamConfig {
        categories: vec![PiiCategory::Email], // Only detect emails
        ..Default::default()
    };

    let processor = StreamProcessor::new(config);
    let data = "Email: contact@example.com, Phone: 555-1234, SSN: 123-45-6789";
    let reader = Cursor::new(data);

    let result = processor.process_all(reader).expect("Processing failed");

    println!(
        "With Email filter only, found {} instances:",
        result.findings.len()
    );
    for finding in &result.findings {
        println!(
            "  - {}: {}",
            finding.category,
            finding.matched_text.as_str()
        );
    }
}
