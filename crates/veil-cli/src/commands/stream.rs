//! Stream processing command implementation.

use std::io::{self, BufRead};

use miette::Result;

use veil_detect::DetectorRegistry;
use veil_parsers::{Position, TextSegment};

use crate::cli::StreamArgs;

/// Run the stream command - reads from stdin and outputs findings.
pub fn run(_args: StreamArgs, _quiet: bool, json: bool) -> Result<()> {
    let registry = DetectorRegistry::default();
    let stdin = io::stdin();
    let mut line_number = 0;
    let mut total_findings = 0;

    if json {
        println!("[");
    }

    let mut first = true;
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| miette::miette!("Read error: {}", e))?;
        line_number += 1;

        // Create segment for this line
        let segment = TextSegment {
            content: line.clone().into(),
            position: Position::Text {
                line: line_number,
                column: 1,
                byte_offset: 0,
                byte_length: line.len(),
            },
        };

        // Detect PII
        let findings = registry.detect_all(&[segment]);

        for finding in &findings {
            total_findings += 1;

            if json {
                if !first {
                    println!(",");
                }
                first = false;
                print!(
                    "  {{\"line\": {}, \"category\": \"{}\", \"text\": \"{}\", \"confidence\": {}}}",
                    line_number,
                    finding.category,
                    finding.matched_text.replace('\"', "\\\""),
                    finding.confidence
                );
            } else {
                println!(
                    "Line {}: [{}] \"{}\" (confidence: {:.2})",
                    line_number, finding.category, finding.matched_text, finding.confidence
                );
            }
        }
    }

    if json {
        println!("\n]");
    } else {
        eprintln!("\nTotal findings: {}", total_findings);
    }

    Ok(())
}
