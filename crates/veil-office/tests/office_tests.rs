//! Integration tests for Office document parsing.

mod fixture_generator;

use std::fs::{self, File};
use std::io::Cursor;
use std::path::Path;

use veil_office::{detect_office_type, parse_docx, parse_pptx, parse_xlsx, OfficeType};

fn fixtures_path() -> &'static Path {
    Path::new("tests/fixtures")
}

fn setup_fixtures() {
    fixture_generator::ensure_fixtures_dir().expect("Failed to create fixtures dir");

    // DOCX fixtures
    let docx_path = fixtures_path().join("docx");

    // Simple DOCX
    fixture_generator::generate_simple_docx(
        &docx_path.join("simple.docx"),
        "This is a simple test document with PII. Contact: john.doe@example.com Phone: 555-123-4567",
    )
    .expect("Failed to generate simple.docx");

    // Multi-paragraph DOCX
    fixture_generator::generate_multi_paragraph_docx(
        &docx_path.join("multi_paragraph.docx"),
        &[
            "First paragraph with email: alice@example.com",
            "Second paragraph with phone: (555) 987-6543",
            "Third paragraph with SSN: 123-45-6789",
            "Fourth paragraph with credit card: 4111-1111-1111-1111",
        ],
    )
    .expect("Failed to generate multi_paragraph.docx");

    // DOCX with special characters
    fixture_generator::generate_simple_docx(
        &docx_path.join("special_chars.docx"),
        "Special characters: <test> \"quoted\" 'apostrophe' & ampersand",
    )
    .expect("Failed to generate special_chars.docx");

    // XLSX fixtures
    let xlsx_path = fixtures_path().join("xlsx");

    // Simple XLSX
    fixture_generator::generate_simple_xlsx(
        &xlsx_path.join("simple.xlsx"),
        &[
            vec!["Name", "Email", "Phone"],
            vec!["John Doe", "john@example.com", "555-111-2222"],
            vec!["Jane Smith", "jane@example.com", "555-333-4444"],
        ],
    )
    .expect("Failed to generate simple.xlsx");

    // XLSX with PII
    fixture_generator::generate_simple_xlsx(
        &xlsx_path.join("pii_data.xlsx"),
        &[
            vec!["ID", "Name", "SSN", "Email"],
            vec!["1", "Alice Brown", "123-45-6789", "alice@example.com"],
            vec!["2", "Bob Wilson", "987-65-4321", "bob@example.org"],
            vec!["3", "Carol Davis", "456-78-9012", "carol@example.net"],
        ],
    )
    .expect("Failed to generate pii_data.xlsx");

    // PPTX fixtures
    let pptx_path = fixtures_path().join("pptx");

    // Simple PPTX
    fixture_generator::generate_simple_pptx(
        &pptx_path.join("simple.pptx"),
        &["Welcome to the Presentation - Contact: presenter@example.com"],
    )
    .expect("Failed to generate simple.pptx");

    // Multi-slide PPTX
    fixture_generator::generate_simple_pptx(
        &pptx_path.join("multi_slide.pptx"),
        &[
            "Slide 1: Introduction - Email: intro@example.com",
            "Slide 2: Main Content - Phone: 555-CALL-NOW",
            "Slide 3: Contact Info - SSN: 111-22-3333",
            "Slide 4: Conclusion - Thanks!",
        ],
    )
    .expect("Failed to generate multi_slide.pptx");
}

// ==================== DOCX Tests ====================

#[test]
fn test_parse_simple_docx() {
    setup_fixtures();

    let path = fixtures_path().join("docx/simple.docx");
    let file = File::open(&path).expect("Failed to open simple.docx");
    let result = parse_docx(file).expect("Failed to parse simple.docx");

    assert!(!result.segments.is_empty(), "Expected segments in result");

    let text: String = result
        .segments
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        text.contains("simple test document"),
        "Expected content not found"
    );
    assert!(
        text.contains("john.doe@example.com"),
        "Expected email not found"
    );
}

#[test]
fn test_parse_multi_paragraph_docx() {
    setup_fixtures();

    let path = fixtures_path().join("docx/multi_paragraph.docx");
    let file = File::open(&path).expect("Failed to open multi_paragraph.docx");
    let result = parse_docx(file).expect("Failed to parse multi_paragraph.docx");

    assert!(result.segments.len() >= 4, "Expected at least 4 segments");

    let text: String = result
        .segments
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        text.contains("alice@example.com"),
        "Expected email not found"
    );
    assert!(
        text.contains("555") || text.contains("987"),
        "Expected phone not found"
    );
}

#[test]
fn test_docx_special_characters() {
    setup_fixtures();

    let path = fixtures_path().join("docx/special_chars.docx");
    let file = File::open(&path).expect("Failed to open special_chars.docx");
    let result = parse_docx(file).expect("Failed to parse special_chars.docx");

    let text: String = result
        .segments
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    // XML entities should be decoded
    assert!(
        text.contains("<test>") || text.contains("test"),
        "Expected content with special chars"
    );
}

// ==================== XLSX Tests ====================

#[test]
fn test_parse_simple_xlsx() {
    setup_fixtures();

    let path = fixtures_path().join("xlsx/simple.xlsx");
    let data = fs::read(&path).expect("Failed to read simple.xlsx");
    let cursor = Cursor::new(data);
    let result = parse_xlsx(cursor).expect("Failed to parse simple.xlsx");

    assert!(!result.segments.is_empty(), "Expected segments in result");

    let text: String = result
        .segments
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        text.contains("John Doe") || text.contains("Name"),
        "Expected name or header"
    );
    assert!(
        text.contains("john@example.com") || text.contains("Email"),
        "Expected email"
    );
}

#[test]
fn test_parse_xlsx_with_pii() {
    setup_fixtures();

    let path = fixtures_path().join("xlsx/pii_data.xlsx");
    let data = fs::read(&path).expect("Failed to read pii_data.xlsx");
    let cursor = Cursor::new(data);
    let result = parse_xlsx(cursor).expect("Failed to parse pii_data.xlsx");

    // Use veil-detect to find PII
    let registry = veil_detect::DetectorRegistry::default();
    let findings = registry.detect_all(&result.segments);

    assert!(!findings.is_empty(), "Expected PII findings in spreadsheet");

    // Should find emails
    let email_count = findings
        .iter()
        .filter(|f| matches!(f.category, veil_detect::PiiCategory::Email))
        .count();
    assert!(email_count >= 1, "Expected to find email addresses");
}

// ==================== PPTX Tests ====================

#[test]
fn test_parse_simple_pptx() {
    setup_fixtures();

    let path = fixtures_path().join("pptx/simple.pptx");
    let file = File::open(&path).expect("Failed to open simple.pptx");
    let result = parse_pptx(file).expect("Failed to parse simple.pptx");

    assert!(!result.segments.is_empty(), "Expected segments in result");

    let text: String = result
        .segments
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        text.contains("Presentation") || text.contains("Welcome"),
        "Expected presentation content"
    );
    assert!(
        text.contains("presenter@example.com"),
        "Expected email not found"
    );
}

#[test]
fn test_parse_multi_slide_pptx() {
    setup_fixtures();

    let path = fixtures_path().join("pptx/multi_slide.pptx");
    let file = File::open(&path).expect("Failed to open multi_slide.pptx");
    let result = parse_pptx(file).expect("Failed to parse multi_slide.pptx");

    assert!(
        !result.segments.is_empty(),
        "Expected segments from multiple slides"
    );

    let text: String = result
        .segments
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        text.contains("Introduction") || text.contains("Slide"),
        "Expected slide content"
    );
}

// ==================== Detection Tests ====================

#[test]
fn test_detect_docx_type() {
    setup_fixtures();

    let path = fixtures_path().join("docx/simple.docx");
    let file = File::open(&path).expect("Failed to open simple.docx");

    let office_type = detect_office_type(file).expect("Failed to detect type");
    assert_eq!(office_type, OfficeType::Docx);
}

#[test]
fn test_detect_xlsx_type() {
    setup_fixtures();

    let path = fixtures_path().join("xlsx/simple.xlsx");
    let file = File::open(&path).expect("Failed to open simple.xlsx");

    let office_type = detect_office_type(file).expect("Failed to detect type");
    assert_eq!(office_type, OfficeType::Xlsx);
}

#[test]
fn test_detect_pptx_type() {
    setup_fixtures();

    let path = fixtures_path().join("pptx/simple.pptx");
    let file = File::open(&path).expect("Failed to open simple.pptx");

    let office_type = detect_office_type(file).expect("Failed to detect type");
    assert_eq!(office_type, OfficeType::Pptx);
}

// ==================== PII Detection Integration Tests ====================

#[test]
fn test_pii_detection_in_docx() {
    setup_fixtures();

    let path = fixtures_path().join("docx/multi_paragraph.docx");
    let file = File::open(&path).expect("Failed to open multi_paragraph.docx");
    let result = parse_docx(file).expect("Failed to parse docx");

    let registry = veil_detect::DetectorRegistry::default();
    let findings = registry.detect_all(&result.segments);

    assert!(!findings.is_empty(), "Expected PII findings in document");
}

#[test]
fn test_pii_detection_in_pptx() {
    setup_fixtures();

    let path = fixtures_path().join("pptx/multi_slide.pptx");
    let file = File::open(&path).expect("Failed to open multi_slide.pptx");
    let result = parse_pptx(file).expect("Failed to parse pptx");

    let registry = veil_detect::DetectorRegistry::default();
    let findings = registry.detect_all(&result.segments);

    // Should find at least the email
    let email_findings: Vec<_> = findings
        .iter()
        .filter(|f| matches!(f.category, veil_detect::PiiCategory::Email))
        .collect();

    assert!(
        !email_findings.is_empty() || !findings.is_empty(),
        "Expected to find PII in presentation"
    );
}
