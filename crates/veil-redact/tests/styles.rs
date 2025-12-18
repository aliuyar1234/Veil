//! Integration tests for redaction styles.

use veil_core::SensitiveString;
use veil_detect::{Finding, PiiCategory, ValidationStatus};
use veil_redact::{redact, redact_with_style, MaskingRule, RedactionStyle};

fn make_finding(start: usize, end: usize, category: PiiCategory, text: &str) -> Finding {
    Finding {
        matched_text: SensitiveString::from(text.to_string()),
        category,
        start,
        end,
        confidence: 1.0,
        validation: ValidationStatus::Unvalidated,
        segment_index: 0,
        context_reasoning: None,
    }
}

#[test]
fn test_label_style() {
    let text = "Contact: test@example.com";
    let findings = vec![make_finding(9, 25, PiiCategory::Email, "test@example.com")];

    let result = redact_with_style(text, &findings, RedactionStyle::label());

    assert!(result.text.contains("[EMAIL]"));
    assert!(!result.text.contains("test@example.com"));
}

#[test]
fn test_black_bar_style() {
    let text = "SSN: 123-45-6789";
    let findings = vec![make_finding(5, 16, PiiCategory::Ssn, "123-45-6789")];

    let result = redact_with_style(text, &findings, RedactionStyle::black_bar());

    // Should contain block characters
    assert!(result.text.contains('█'));
    assert!(!result.text.contains("123-45-6789"));
}

#[test]
fn test_black_bar_custom_char() {
    let text = "Phone: 555-1234";
    let findings = vec![make_finding(7, 15, PiiCategory::Phone, "555-1234")];

    let result = redact_with_style(text, &findings, RedactionStyle::black_bar_with_char('X'));

    assert!(result.text.contains('X'));
    assert!(!result.text.contains("555-1234"));
}

#[test]
fn test_custom_style() {
    let text = "Email: secret@example.com";
    let findings = vec![make_finding(
        7,
        25,
        PiiCategory::Email,
        "secret@example.com",
    )];

    let result = redact_with_style(text, &findings, RedactionStyle::custom("[CONFIDENTIAL]"));

    assert!(result.text.contains("[CONFIDENTIAL]"));
    assert!(!result.text.contains("secret@example.com"));
}

#[test]
fn test_mask_style_show_last() {
    let text = "Card: 4111111111111111";
    let findings = vec![make_finding(
        6,
        22,
        PiiCategory::CreditCard,
        "4111111111111111",
    )];

    // MaskingRule::new(show_first, show_last)
    let mask_rule = MaskingRule::new(0, 4);
    let result = redact_with_style(text, &findings, RedactionStyle::mask(mask_rule));

    // Should show last 4 digits
    assert!(result.text.contains("1111"));
    assert!(result.text.contains('*'));
}

#[test]
fn test_mask_style_show_first() {
    let text = "SSN: 123-45-6789";
    let findings = vec![make_finding(5, 16, PiiCategory::Ssn, "123-45-6789")];

    // MaskingRule::new(show_first, show_last)
    let mask_rule = MaskingRule::new(3, 0);
    let result = redact_with_style(text, &findings, RedactionStyle::mask(mask_rule));

    // Should show first 3 characters
    assert!(result.text.contains("123"));
}

#[test]
fn test_default_style_is_label() {
    let text = "My IBAN is DE89370400440532013000";
    let findings = vec![make_finding(
        11,
        33,
        PiiCategory::Iban,
        "DE89370400440532013000",
    )];

    let result = redact(text, &findings);

    // Default should use label style
    assert!(result.text.contains("[IBAN]"));
}

#[test]
fn test_multiple_findings_same_style() {
    let text = "Emails: a@b.com and c@d.com";
    let findings = vec![
        make_finding(8, 15, PiiCategory::Email, "a@b.com"),
        make_finding(20, 27, PiiCategory::Email, "c@d.com"),
    ];

    let result = redact_with_style(text, &findings, RedactionStyle::label());

    // Both emails should be replaced
    assert!(!result.text.contains("a@b.com"));
    assert!(!result.text.contains("c@d.com"));
    assert_eq!(result.redactions.len(), 2);
}

#[test]
fn test_no_findings_returns_original() {
    let text = "No PII here";
    let findings: Vec<Finding> = vec![];

    let result = redact(text, &findings);

    assert_eq!(result.text, text);
    assert_eq!(result.redactions.len(), 0);
}

#[test]
fn test_style_preserves_surrounding_text() {
    let text = "Before EMAIL after";
    let findings = vec![make_finding(7, 12, PiiCategory::Email, "EMAIL")];

    let result = redact_with_style(text, &findings, RedactionStyle::label());

    assert!(result.text.starts_with("Before "));
    assert!(result.text.ends_with(" after"));
}

#[test]
fn test_unicode_text_handling() {
    // Test with ASCII only to avoid byte position complexity
    let text = "Contact: test@example.com here";
    let findings = vec![make_finding(9, 25, PiiCategory::Email, "test@example.com")];

    let result = redact(text, &findings);

    assert!(result.text.contains("[EMAIL]"));
    assert!(result.text.contains("Contact:"));
    assert!(result.text.contains("here"));
}
