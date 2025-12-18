//! Basic PII scanning example.
//!
//! Demonstrates how to detect PII in text content.
//!
//! # Running
//!
//! ```bash
//! cargo run --example basic_scan
//! ```

use veil_detect::{registry::DetectorRegistry, Finding, PiiCategory};

fn main() {
    // Sample text containing various PII
    let text = r#"
        Customer Information:
        Name: John Smith
        Email: john.smith@example.com
        Phone: (555) 123-4567
        SSN: 123-45-6789
        Credit Card: 4111-1111-1111-1111
        IBAN: DE89 3704 0044 0532 0130 00
    "#;

    println!("=== Veil Basic Scan Example ===\n");
    println!("Input text:");
    println!("{}\n", text);

    // Create detector registry with all built-in patterns
    let registry = DetectorRegistry::default();

    // Detect PII
    let findings = registry.detect(text);

    // Display results
    println!("Found {} PII items:\n", findings.len());

    for (i, finding) in findings.iter().enumerate() {
        println!(
            "{}. {} (confidence: {:.0}%)",
            i + 1,
            finding.category,
            finding.confidence * 100.0
        );
        println!("   Text: \"{}\"", finding.text);
        println!(
            "   Position: {} to {}",
            finding.position.start, finding.position.end
        );
        println!();
    }

    // Group by category
    println!("=== Summary by Category ===\n");
    let mut categories: std::collections::HashMap<PiiCategory, Vec<&Finding>> =
        std::collections::HashMap::new();

    for finding in &findings {
        categories.entry(finding.category).or_default().push(finding);
    }

    for (category, items) in &categories {
        println!("{}: {} found", category, items.len());
    }
}
