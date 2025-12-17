//! Example demonstrating context-aware PII detection.
//!
//! Run with: cargo run --example context_detection -p veil-detect

use veil_detect::DetectorRegistry;
use veil_parsers::{Position, TextSegment};

fn main() {
    println!("=== Context-Aware PII Detection Example ===\n");

    // Example 1: Name detection with honorific
    println!("Example 1: Name detection with honorific context");
    let text1 = "Dear Mr. Smith, please contact us at support@example.com";
    demonstrate_detection(text1, "en");

    // Example 2: Version number suppression
    println!("\nExample 2: Version number (not IP address)");
    let text2 = "Software version 192.168.1.1 was released yesterday";
    demonstrate_detection(text2, "en");

    // Example 3: Email with label
    println!("\nExample 3: Email with label context");
    let text3 = "Contact Email: john.doe@company.com for more information";
    demonstrate_detection(text3, "en");

    // Example 4: German text
    println!("\nExample 4: German text with honorific");
    let text4 = "Sehr geehrter Herr Müller, Ihre E-Mail: mueller@beispiel.de";
    demonstrate_detection(text4, "de");

    // Example 5: Order number (not credit card)
    println!("\nExample 5: Order number suppression");
    let text5 = "Your order #4532-1234-5678-9012 has been shipped";
    demonstrate_detection(text5, "en");
}

fn demonstrate_detection(text: &str, language: &str) {
    println!("Text: \"{}\"", text);
    println!("Language: {}", language);

    let segment = TextSegment {
        content: text.to_string().into(),
        position: Position::Text {
            line: 1,
            column: 1,
            byte_offset: 0,
            byte_length: text.len(),
        },
    };

    // Create registry with context analysis
    let mut registry = DetectorRegistry::default();
    registry.enable_context_analysis();

    // Detect with context
    let findings = registry.detect_all_with_context(&[segment], Some(language));

    if findings.is_empty() {
        println!("No PII detected.");
    } else {
        println!("Detections:");
        for finding in findings {
            println!(
                "  - {} ({:?}): \"{}\" [confidence: {:.2}]",
                finding.category, finding.validation, finding.matched_text.as_str(), finding.confidence
            );
            if let Some(reasoning) = &finding.context_reasoning {
                for reason in reasoning {
                    println!("    → {}", reason);
                }
            }
        }
    }
}
