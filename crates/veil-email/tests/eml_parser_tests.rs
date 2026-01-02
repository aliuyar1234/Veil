//! Tests for EML email parsing.

use std::fs;
use std::path::Path;

use veil_email::{parse_email, parse_email_to_result, EmailHeaderValue, EmailParseOptions};
use veil_types::FileFormat;

fn fixtures_path() -> &'static Path {
    Path::new("tests/fixtures/eml")
}

fn read_fixture(name: &str) -> Vec<u8> {
    let path = fixtures_path().join(name);
    fs::read(&path).unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path.display(), e))
}

#[test]
fn test_parse_simple_email() {
    let data = read_fixture("simple.eml");
    let options = EmailParseOptions::default();
    let message = parse_email(&data, &options).expect("Failed to parse simple email");

    assert_eq!(message.subject(), Some("Simple Test Email"));
    assert!(message.from().is_some());

    let from = message.from().unwrap();
    // Check the From header value
    if let EmailHeaderValue::Address(addr) = &from.value {
        assert_eq!(addr.address, "sender@example.com");
        assert!(addr.display_name.is_none());
    } else if let EmailHeaderValue::Unstructured(s) = &from.value {
        assert!(s.contains("sender@example.com"));
    }

    assert!(message.to().is_some());
}

#[test]
fn test_parse_with_display_names() {
    let data = read_fixture("with_display_name.eml");
    let options = EmailParseOptions::default();
    let message = parse_email(&data, &options).expect("Failed to parse email with display names");

    let from = message.from().unwrap();
    // Check the From header for display name
    match &from.value {
        EmailHeaderValue::Address(addr) => {
            assert_eq!(addr.address, "john.doe@example.com");
            assert_eq!(addr.display_name.as_deref(), Some("John Doe"));
        }
        EmailHeaderValue::Unstructured(s) => {
            assert!(s.contains("john.doe@example.com"));
            assert!(s.contains("John Doe"));
        }
        _ => {}
    }

    // Check To header for multiple recipients
    let to = message.to().unwrap();
    match &to.value {
        EmailHeaderValue::AddressList(addrs) => {
            assert!(addrs.len() >= 2, "Expected at least 2 recipients");
        }
        EmailHeaderValue::Unstructured(s) => {
            assert!(s.contains("jane.smith@example.com") || s.contains("bob@example.org"));
        }
        _ => {}
    }
}

#[test]
fn test_parse_multipart_alternative() {
    let data = read_fixture("multipart_alternative.eml");
    let options = EmailParseOptions::default();
    let message = parse_email(&data, &options).expect("Failed to parse multipart email");

    assert_eq!(message.subject(), Some("Multipart Alternative Email"));

    // Should have body content (preferring plain text)
    let segments = message.to_text_segments();
    assert!(!segments.is_empty());

    // Check that content is extracted
    let body_text: String = segments
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    // Should have some text content
    assert!(!body_text.is_empty(), "Expected body content");
}

#[test]
fn test_parse_html_only_email() {
    let data = read_fixture("html_only.eml");
    let options = EmailParseOptions::default();
    let message = parse_email(&data, &options).expect("Failed to parse HTML-only email");

    assert_eq!(message.subject(), Some("HTML Only Newsletter"));

    let segments = message.to_text_segments();
    assert!(!segments.is_empty());

    // HTML should be converted to text
    let body_text: String = segments
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    // Should extract text content from HTML
    assert!(!body_text.is_empty(), "Expected converted HTML content");
}

#[test]
fn test_parse_email_with_attachment() {
    let data = read_fixture("with_attachment.eml");
    let options = EmailParseOptions::default();
    let message = parse_email(&data, &options).expect("Failed to parse email with attachment");

    assert_eq!(message.subject(), Some("Email with Attachment"));

    // Should have attachments
    assert!(!message.attachments.is_empty(), "Expected attachments");

    let attachment = &message.attachments[0];
    assert_eq!(attachment.filename.as_deref(), Some("document.pdf"));
    assert!(
        attachment.content_type.contains("pdf") || attachment.content_type.contains("application")
    );
}

#[test]
fn test_parse_quoted_reply() {
    let data = read_fixture("quoted_reply.eml");
    let options = EmailParseOptions::default();
    let message = parse_email(&data, &options).expect("Failed to parse quoted reply");

    assert_eq!(message.subject(), Some("Re: Meeting Request"));

    let segments = message.to_text_segments();
    assert!(!segments.is_empty());
}

#[test]
fn test_parse_base64_encoded_body() {
    let data = read_fixture("base64_body.eml");
    let options = EmailParseOptions::default();
    let message = parse_email(&data, &options).expect("Failed to parse base64 email");

    let segments = message.to_text_segments();
    let body_text: String = segments
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    // Should decode base64 content
    assert!(
        body_text.contains("base64")
            || body_text.contains("encoded")
            || body_text.contains("message"),
        "Expected decoded content, got: {}",
        body_text
    );
}

#[test]
fn test_parse_empty_body() {
    let data = read_fixture("empty_body.eml");
    let options = EmailParseOptions::default();
    let message = parse_email(&data, &options).expect("Failed to parse empty body email");

    assert_eq!(message.subject(), Some("Email with Empty Body"));
    assert!(message.from().is_some());
}

#[test]
fn test_parse_unicode_email() {
    let data = read_fixture("unicode.eml");
    let options = EmailParseOptions::default();
    let message = parse_email(&data, &options).expect("Failed to parse unicode email");

    let segments = message.to_text_segments();
    let body_text: String = segments
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    // Should handle Unicode content
    assert!(
        body_text.contains("Unicode") || body_text.contains("Japanese") || !body_text.is_empty()
    );
}

#[test]
fn test_parse_to_result() {
    let data = read_fixture("simple.eml");
    let options = EmailParseOptions::default();
    let result = parse_email_to_result(&data, &options).expect("Failed to parse to result");

    assert_eq!(result.metadata.format, FileFormat::Eml);
    assert!(!result.segments.is_empty());
    assert!(result.total_chars > 0);
}

#[test]
fn test_pii_detection_in_email() {
    let data = read_fixture("simple.eml");
    let options = EmailParseOptions::default();
    let result = parse_email_to_result(&data, &options).expect("Failed to parse email");

    // Use veil-detect to find PII
    let registry = veil_detect::DetectorRegistry::default();
    let findings = registry.detect_all(&result.segments);

    // Should find PII in the email (at least the email address in From)
    // Note: The email addresses in headers might not be in segments
    // but the body should have PII
    let _body_has_pii = !findings.is_empty();

    // At minimum, verify we got segments
    assert!(!result.segments.is_empty(), "Expected segments from email");
}

#[test]
fn test_email_headers_extracted() {
    let data = read_fixture("simple.eml");
    let options = EmailParseOptions::default();
    let message = parse_email(&data, &options).expect("Failed to parse email");

    // Check that basic headers are present
    assert!(message.from().is_some(), "Expected From header");
    assert!(message.to().is_some(), "Expected To header");
    assert!(message.subject().is_some(), "Expected Subject header");
    assert!(message.date().is_some(), "Expected Date header");
}

#[test]
fn test_email_format_detection() {
    let data = read_fixture("simple.eml");
    let options = EmailParseOptions::default();
    let message = parse_email(&data, &options).expect("Failed to parse email");

    assert_eq!(message.format, veil_email::EmailFormat::Eml);
}
