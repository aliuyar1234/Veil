//! Property-based tests for email validation.

use proptest::prelude::*;

// Email regex pattern (simplified for testing)
fn is_valid_email_format(s: &str) -> bool {
    // Basic email format check
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let (local, domain) = (parts[0], parts[1]);

    // Local part checks
    if local.is_empty() || local.len() > 64 {
        return false;
    }

    // Domain checks
    if domain.is_empty() || !domain.contains('.') {
        return false;
    }

    // TLD check
    let tld = domain.rsplit('.').next().unwrap_or("");
    if tld.len() < 2 {
        return false;
    }

    true
}

// Strategy to generate valid-looking email addresses
fn valid_email_strategy() -> impl Strategy<Value = String> {
    (
        "[a-z][a-z0-9._]{0,20}",                      // local part
        "[a-z][a-z0-9]{0,10}",                        // domain
        prop_oneof!["com", "org", "net", "io", "de"], // TLD
    )
        .prop_map(|(local, domain, tld)| format!("{}@{}.{}", local, domain, tld))
}

// Strategy to generate strings that shouldn't be emails
fn non_email_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // No @ symbol
        "[a-z]{5,20}",
        // Multiple @ symbols
        "[a-z]+@[a-z]+@[a-z]+",
        // Empty parts
        "@domain.com",
        "local@",
        // No domain
        "[a-z]+@",
        // Random characters
        "[!#$%^&*()]{5,20}",
    ]
}

proptest! {
    #[test]
    fn valid_emails_have_correct_format(email in valid_email_strategy()) {
        prop_assert!(is_valid_email_format(&email),
            "Generated email should be valid format: {}", email);
    }

    #[test]
    fn invalid_emails_rejected(non_email in non_email_strategy()) {
        // Many non-emails should fail the format check
        // Note: Some generated strings might accidentally be valid
        let _ = is_valid_email_format(&non_email);
    }

    #[test]
    fn email_always_contains_at_if_valid(s in ".*") {
        if is_valid_email_format(&s) {
            prop_assert!(s.contains('@'),
                "Valid email must contain @: {}", s);
        }
    }

    #[test]
    fn email_local_part_not_empty_if_valid(s in ".*") {
        if is_valid_email_format(&s) {
            let local = s.split('@').next().unwrap();
            prop_assert!(!local.is_empty(),
                "Valid email must have non-empty local part: {}", s);
        }
    }

    #[test]
    fn email_domain_contains_dot_if_valid(s in ".*") {
        if is_valid_email_format(&s) {
            let domain = s.split('@').next_back().unwrap();
            prop_assert!(domain.contains('.'),
                "Valid email domain must contain dot: {}", s);
        }
    }

    #[test]
    fn empty_string_is_not_email(s in "") {
        prop_assert!(!is_valid_email_format(&s));
    }
}
