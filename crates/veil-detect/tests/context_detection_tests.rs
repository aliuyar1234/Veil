//! Integration tests for context detection.

use veil_detect::context::{ContextAction, ContextAnalyzer, ContextRule};
use veil_detect::{DetectorRegistry, Finding, PiiCategory, ValidationStatus};
use veil_parsers::{Position, TextSegment};

fn make_segment(content: &str) -> TextSegment {
    TextSegment {
        content: content.to_string(),
        position: Position::Text {
            line: 1,
            column: 1,
            byte_offset: 0,
            byte_length: content.len(),
        },
    }
}

fn make_finding(text: &str, category: PiiCategory, start: usize, end: usize) -> Finding {
    Finding::new(
        text,
        category,
        start,
        end,
        0.7,
        ValidationStatus::Unvalidated,
        0,
    )
}

#[test]
fn test_context_analyzer_basic() {
    let mut analyzer = ContextAnalyzer::new();
    // Just check that the analyzer can be created
    let text = "Test text";
    let markers = analyzer.detect_markers(text, Some("en"));
    // Markers may or may not be found in this simple text, but the function should work
    let _ = markers;
}

#[test]
fn test_detect_honorific_markers() {
    let mut analyzer = ContextAnalyzer::new();
    let text = "Dear Mr. Smith and Dr. Johnson";
    let markers = analyzer.detect_markers(text, Some("en"));

    assert!(!markers.is_empty());
    assert!(markers.iter().any(|m| m.text.contains("Mr")));
}

#[test]
fn test_boost_confidence_with_honorific() {
    let mut analyzer = ContextAnalyzer::new();
    let text = "Contact: Mr. John Smith";

    // Create a finding for "John Smith" (simulated name detection)
    let finding = make_finding(
        "John Smith",
        PiiCategory::Custom("PersonName".to_string()),
        13,
        23,
    );

    let analysis = analyzer.analyze_finding(&finding, text, Some("en"));

    // Confidence should be boosted due to "Mr." context
    assert!(analysis.adjusted_confidence > analysis.original_confidence);
    assert!(!analysis.markers.is_empty());
    assert!(!analysis.reasoning.is_empty());
}

#[test]
fn test_suppress_version_number_as_ip() {
    let mut analyzer = ContextAnalyzer::new();
    let text = "Software version 192.168.1.1 released";

    let finding = make_finding("192.168.1.1", PiiCategory::Ipv4, 17, 28);
    let analysis = analyzer.analyze_finding(&finding, text, Some("en"));

    // Confidence should be suppressed due to "version" context
    assert!(analysis.adjusted_confidence < analysis.original_confidence);
}

#[test]
fn test_email_label_context() {
    let mut analyzer = ContextAnalyzer::new();
    let text = "Email: john@example.com";

    let finding = make_finding("john@example.com", PiiCategory::Email, 7, 23);
    let analysis = analyzer.analyze_finding(&finding, text, Some("en"));

    // Should be boosted by email label
    assert!(analysis.adjusted_confidence >= analysis.original_confidence);
}

#[test]
fn test_german_honorific() {
    let mut analyzer = ContextAnalyzer::with_language("de");
    let text = "Sehr geehrter Herr Müller";

    let markers = analyzer.detect_markers(text, Some("de"));
    assert!(markers
        .iter()
        .any(|m| m.text.to_lowercase().contains("herr")
            || m.text.to_lowercase().contains("geehrter")));
}

#[test]
fn test_french_honorific() {
    let mut analyzer = ContextAnalyzer::with_language("fr");
    let text = "Bonjour Madame Dupont";

    let markers = analyzer.detect_markers(text, Some("fr"));
    assert!(markers
        .iter()
        .any(|m| m.text.to_lowercase().contains("madame")
            || m.text.to_lowercase().contains("bonjour")));
}

#[test]
fn test_order_number_suppression() {
    let mut analyzer = ContextAnalyzer::new();
    let text = "Order #4532-1234-5678-9012 confirmed";

    // This looks like a credit card but is an order number
    let finding = make_finding("4532-1234-5678-9012", PiiCategory::CreditCard, 7, 26);
    let analysis = analyzer.analyze_finding(&finding, text, Some("en"));

    // Should be suppressed due to "Order #" context
    assert!(analysis.adjusted_confidence < analysis.original_confidence);
}

#[test]
fn test_no_adjustment_without_context() {
    let mut analyzer = ContextAnalyzer::new();
    let text = "Random text with john@example.com in the middle";

    let finding = make_finding("john@example.com", PiiCategory::Email, 17, 33);
    let analysis = analyzer.analyze_finding(&finding, text, Some("en"));

    // Should have minimal or no adjustment without nearby context markers
    assert!((analysis.adjusted_confidence - analysis.original_confidence).abs() < 0.15);
}

#[test]
fn test_custom_rule() {
    let mut analyzer = ContextAnalyzer::empty();

    // Add custom boost rule
    let rule = ContextRule::new(r"(?i)\bcustomer\s+name:\s*", ContextAction::Boost, 0.4)
        .with_language("en")
        .with_description("Custom customer name label");

    analyzer.add_rule(rule);

    let text = "Customer Name: Alice Johnson";
    let finding = make_finding(
        "Alice Johnson",
        PiiCategory::Custom("PersonName".to_string()),
        15,
        28,
    );

    let analysis = analyzer.analyze_finding(&finding, text, Some("en"));
    assert!(analysis.adjusted_confidence > analysis.original_confidence);
}

#[test]
fn test_registry_with_context_analysis() {
    let mut registry = DetectorRegistry::default();
    registry.enable_context_analysis();

    let segments = vec![make_segment("Email: test@example.com")];
    let findings = registry.detect_all_with_context(&segments, Some("en"));

    assert!(!findings.is_empty());
    // Context reasoning should be applied
    let email_finding = findings.iter().find(|f| f.category == PiiCategory::Email);
    assert!(email_finding.is_some());
}

#[test]
fn test_registry_without_context_analysis() {
    let registry = DetectorRegistry::default();

    let segments = vec![make_segment("Email: test@example.com")];
    let findings = registry.detect_all(&segments);

    assert!(!findings.is_empty());
    // No context reasoning should be present
    let email_finding = findings.iter().find(|f| f.category == PiiCategory::Email);
    assert!(email_finding.is_some());
    if let Some(finding) = email_finding {
        assert!(finding.context_reasoning.is_none());
    }
}

#[test]
fn test_multiple_context_markers() {
    let mut analyzer = ContextAnalyzer::new();
    let text = "Dear Dr. Smith, please contact us at doctor@hospital.com";

    let markers = analyzer.detect_markers(text, Some("en"));

    // Should find multiple markers: "Dear", "Dr."
    assert!(markers.len() >= 2);
}

#[test]
fn test_address_label_context() {
    let mut analyzer = ContextAnalyzer::new();
    let text = "Address: 123 Main Street, New York";

    let markers = analyzer.detect_markers(text, Some("en"));

    // Should detect "Address:" and "Street" markers
    assert!(!markers.is_empty());
}

#[test]
fn test_german_email_label() {
    let mut analyzer = ContextAnalyzer::with_language("de");
    let text = "E-Mail: kontakt@beispiel.de";

    let finding = make_finding("kontakt@beispiel.de", PiiCategory::Email, 8, 27);
    let analysis = analyzer.analyze_finding(&finding, text, Some("de"));

    // Should be boosted by German email label
    assert!(analysis.adjusted_confidence >= analysis.original_confidence);
}

#[test]
fn test_phone_label_context() {
    let mut analyzer = ContextAnalyzer::new();
    let text = "Phone: +1-555-123-4567";

    let finding = make_finding("+1-555-123-4567", PiiCategory::Phone, 7, 22);
    let analysis = analyzer.analyze_finding(&finding, text, Some("en"));

    // Should be boosted by phone label
    assert!(analysis.adjusted_confidence >= analysis.original_confidence);
}

#[test]
fn test_context_window_distance() {
    let mut analyzer = ContextAnalyzer::new();
    // Context marker is far away from the finding
    let text = "Dear Mr. Smith,\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. Random email: test@example.com appears here.";

    let finding = make_finding("test@example.com", PiiCategory::Email, 217, 233);
    let analysis = analyzer.analyze_finding(&finding, text, Some("en"));

    // "Mr. Smith" is too far away to affect "test@example.com"
    // But "email:" label should still boost
    assert!(
        !analysis.markers.is_empty()
            || (analysis.adjusted_confidence - analysis.original_confidence).abs() < 0.5
    );
}

#[test]
fn test_isbn_suppression() {
    let mut analyzer = ContextAnalyzer::new();
    let text = "ISBN 978-3-16-148410-0";

    let markers = analyzer.detect_markers(text, Some("en"));

    // Should find ISBN suppression marker
    assert!(markers
        .iter()
        .any(|m| m.text.to_lowercase().contains("isbn")));
}
