//! Streaming PII detection example.
//!
//! Demonstrates how to detect PII in streaming/chunked data.
//!
//! # Running
//!
//! ```bash
//! cargo run --example streaming
//! ```

use std::io::{BufRead, BufReader, Cursor};

use veil_detect::registry::DetectorRegistry;

fn main() {
    println!("=== Veil Streaming Detection Example ===\n");

    // Simulate a large document that arrives in chunks
    let large_document = r#"
        === Customer Database Export ===

        Record 1:
        Name: Alice Johnson
        Email: alice.johnson@example.com
        Phone: (555) 111-2222
        SSN: 111-22-3333

        Record 2:
        Name: Bob Williams
        Email: bob.williams@company.org
        Phone: (555) 444-5555
        SSN: 444-55-6666

        Record 3:
        Name: Carol Davis
        Email: carol.davis@enterprise.net
        Phone: (555) 777-8888
        Credit Card: 4111-1111-1111-1111

        Record 4:
        Name: David Miller
        Email: david.miller@startup.io
        Phone: (555) 999-0000
        IBAN: DE89 3704 0044 0532 0130 00

        === End of Export ===
    "#;

    println!("Processing document in chunks...\n");

    // Create detector
    let registry = DetectorRegistry::default();

    // Process document line by line (simulating streaming)
    let reader = BufReader::new(Cursor::new(large_document));

    let mut total_findings = 0;
    let mut line_number = 0;
    let mut current_offset = 0;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                continue;
            }
        };

        line_number += 1;

        // Skip empty lines
        if line.trim().is_empty() {
            current_offset += line.len() + 1; // +1 for newline
            continue;
        }

        // Detect PII in this chunk
        let findings = registry.detect(&line);

        if !findings.is_empty() {
            println!("Line {}: \"{}\"", line_number, line.trim());

            for finding in &findings {
                println!(
                    "  -> Found {} (confidence: {:.0}%): \"{}\"",
                    finding.category,
                    finding.confidence * 100.0,
                    finding.text
                );
                total_findings += 1;
            }
            println!();
        }

        current_offset += line.len() + 1;
    }

    println!("=== Streaming Summary ===\n");
    println!("Lines processed: {}", line_number);
    println!("Total findings: {}", total_findings);
    println!();

    // Demonstrate chunk-based processing for large files
    println!("=== Chunk-Based Processing Demo ===\n");

    let chunk_size = 200; // bytes per chunk
    let mut offset = 0;
    let mut chunk_num = 0;

    while offset < large_document.len() {
        chunk_num += 1;
        let end = (offset + chunk_size).min(large_document.len());

        // Get chunk (in real scenario, this would be read from a stream)
        let chunk = &large_document[offset..end];

        // Detect PII in chunk
        let findings = registry.detect(chunk);

        if !findings.is_empty() {
            println!(
                "Chunk {} (bytes {}-{}): {} findings",
                chunk_num,
                offset,
                end,
                findings.len()
            );
        }

        offset = end;
    }

    println!("\nProcessed {} chunks", chunk_num);
}
