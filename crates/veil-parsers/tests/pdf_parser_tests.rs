//! Integration tests for PDF parsing.

mod fixture_generator;

use std::fs;
use std::path::Path;
use std::sync::Once;

use veil_parsers::{parse_bytes, parse_file, FileFormat, ParseOptions};

static INIT: Once = Once::new();

fn fixtures_path() -> &'static Path {
    Path::new("tests/fixtures/pdf")
}

fn setup_fixtures() {
    INIT.call_once(|| {
        fixture_generator::setup_fixtures().expect("Failed to set up fixtures");
    });
}

fn read_fixture(name: &str) -> Vec<u8> {
    setup_fixtures();
    let path = fixtures_path().join(name);
    fs::read(&path).unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path.display(), e))
}

fn fixture_path(name: &str) -> std::path::PathBuf {
    setup_fixtures();
    fixtures_path().join(name)
}

#[test]
fn test_parse_simple_pdf() {
    let bytes = read_fixture("simple.pdf");
    let options = ParseOptions::default();
    let result = parse_bytes(&bytes, &options).expect("Failed to parse simple PDF");

    assert_eq!(result.metadata.format, FileFormat::Pdf);
    assert!(!result.segments.is_empty(), "Expected segments from PDF");
    assert!(result.total_chars > 0, "Expected characters in PDF");

    // Check that we extracted some text
    let all_text: String = result.segments.iter().map(|s| s.content.as_str()).collect();
    assert!(
        all_text.contains("simple") || all_text.contains("PDF") || all_text.contains("document"),
        "Expected to find content in PDF: {}",
        all_text
    );
}

#[test]
fn test_parse_pdf_from_file() {
    let path = fixture_path("simple.pdf");
    let options = ParseOptions::default();
    let result = parse_file(&path, &options).expect("Failed to parse PDF from file");

    assert_eq!(result.metadata.format, FileFormat::Pdf);
    assert!(!result.segments.is_empty());
}

#[test]
fn test_parse_multipage_pdf() {
    let bytes = read_fixture("multipage.pdf");
    let options = ParseOptions::default();
    let result = parse_bytes(&bytes, &options).expect("Failed to parse multipage PDF");

    assert_eq!(result.metadata.format, FileFormat::Pdf);
    assert!(!result.segments.is_empty());

    // Should have content from multiple pages
    let all_text: String = result.segments.iter().map(|s| s.content.as_str()).collect();

    // Check for content from different pages
    let has_page_content = all_text.contains("Page")
        || all_text.contains("page")
        || all_text.contains("first")
        || all_text.contains("second");
    assert!(
        has_page_content,
        "Expected content from multiple pages: {}",
        all_text
    );
}

#[test]
fn test_parse_pdf_with_pii() {
    let bytes = read_fixture("with_pii.pdf");
    let options = ParseOptions::default();
    let result = parse_bytes(&bytes, &options).expect("Failed to parse PII PDF");

    assert_eq!(result.metadata.format, FileFormat::Pdf);

    // Use veil-detect to find PII
    let registry = veil_detect::DetectorRegistry::default();
    let findings = registry.detect_all(&result.segments);

    // Check by category
    let has_email = findings
        .iter()
        .any(|f| matches!(f.category, veil_detect::PiiCategory::Email));
    let has_phone = findings
        .iter()
        .any(|f| matches!(f.category, veil_detect::PiiCategory::Phone));

    // We should find at least one type of PII
    assert!(
        has_email || has_phone || !findings.is_empty(),
        "Expected to find PII in document. Segments: {:?}",
        result.segments
    );
}

#[test]
fn test_parse_empty_pdf() {
    let bytes = read_fixture("empty.pdf");
    let options = ParseOptions::default();
    let result = parse_bytes(&bytes, &options);

    // Should succeed but with empty or minimal content
    match result {
        Ok(r) => {
            // Empty PDF should have minimal content
            assert_eq!(r.metadata.format, FileFormat::Pdf);
            // May have a warning about no extractable text
        }
        Err(_) => {
            // Empty PDF parsing may fail - that's acceptable
        }
    }
}

#[test]
fn test_parse_invalid_pdf() {
    // Data with PDF magic bytes but invalid structure
    let invalid_bytes = b"%PDF-1.4\nThis is corrupted PDF data that should fail to parse properly";
    let options = ParseOptions::default();
    let result = parse_bytes(invalid_bytes, &options);

    // Should either fail or return a warning/empty result
    match result {
        Ok(r) => {
            // May succeed but with warnings or no content
            assert!(
                r.segments.is_empty() || !r.warnings.is_empty(),
                "Expected warnings or no content for corrupted PDF"
            );
        }
        Err(_) => {
            // Error is also acceptable for corrupted PDF
        }
    }
}

#[test]
fn test_pdf_metadata() {
    let bytes = read_fixture("simple.pdf");
    let options = ParseOptions::default();
    let result = parse_bytes(&bytes, &options).expect("Failed to parse PDF");

    assert_eq!(result.metadata.format, FileFormat::Pdf);
    assert!(result.metadata.size_bytes.is_some());
    assert_eq!(result.metadata.encoding, "UTF-8");
}

#[test]
fn test_pdf_segment_positions() {
    let bytes = read_fixture("simple.pdf");
    let options = ParseOptions::default();
    let result = parse_bytes(&bytes, &options).expect("Failed to parse PDF");

    // Check that segments have PDF position information
    for segment in &result.segments {
        match &segment.position {
            veil_parsers::Position::Pdf {
                page, byte_length, ..
            } => {
                assert!(*page >= 1, "Page should be at least 1");
                assert_eq!(*byte_length, segment.content.len());
            }
            other => panic!("Expected PDF position, got {:?}", other),
        }
    }
}

#[test]
fn test_pdf_format_detection() {
    let bytes = read_fixture("simple.pdf");

    // PDF should be detected by magic bytes
    let format = veil_parsers::detect_format(&bytes, Some("test.pdf"));
    assert_eq!(format, FileFormat::Pdf, "Should detect PDF format");
}

#[test]
fn test_parse_pdf_bytes_vs_file() {
    let path = fixture_path("simple.pdf");
    let bytes = read_fixture("simple.pdf");
    let options = ParseOptions::default();

    let result_bytes = parse_bytes(&bytes, &options).expect("Failed to parse bytes");
    let result_file = parse_file(&path, &options).expect("Failed to parse file");

    // Results should be equivalent
    assert_eq!(result_bytes.metadata.format, result_file.metadata.format);
    assert_eq!(result_bytes.segments.len(), result_file.segments.len());
    assert_eq!(result_bytes.total_chars, result_file.total_chars);
}

#[test]
fn test_pdf_parse_performance() {
    let bytes = read_fixture("simple.pdf");
    let options = ParseOptions::default();
    let result = parse_bytes(&bytes, &options).expect("Failed to parse PDF");

    // Duration should be reasonable
    assert!(
        result.duration_ms < 10000,
        "PDF parsing should complete in reasonable time"
    );
}
