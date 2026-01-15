# Quickstart: Global Phone Number Detection

## Overview

Extend `PhoneDetector` to recognize US, UK, French, and generic E.164 international phone formats.

## Primary File

**`crates/veil-detect/src/patterns/phone.rs`**

## Implementation Steps

### Step 1: Extend PHONE_PATTERNS

Add new patterns after existing DACH patterns:

```rust
static PHONE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // === DACH Region (existing - keep first) ===
        Regex::new(r"\+(?:43|49|41)\s?[\d\s/-]{7,15}").unwrap(),
        Regex::new(r"00(?:43|49|41)\s?[\d\s/-]{7,15}").unwrap(),

        // === US/Canada (NANP) - NEW ===
        Regex::new(r"\+1[\s.-]?\d{3}[\s.-]?\d{3}[\s.-]?\d{4}").unwrap(),
        Regex::new(r"1[\s.-]\d{3}[\s.-]\d{3}[\s.-]\d{4}").unwrap(),
        Regex::new(r"\(\d{3}\)[\s.-]?\d{3}[\s.-]?\d{4}").unwrap(),
        Regex::new(r"\d{3}[\s.-]\d{3}[\s.-]\d{4}").unwrap(),

        // === UK - NEW ===
        Regex::new(r"\+44[\s.-]?\d{2,4}[\s.-]?\d{3,4}[\s.-]?\d{3,6}").unwrap(),
        Regex::new(r"07\d{3}[\s.-]?\d{6}").unwrap(),

        // === France - NEW ===
        Regex::new(r"\+33[\s.-]?\d[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2}").unwrap(),

        // === Generic E.164 (catch-all) - NEW ===
        Regex::new(r"\+[1-9]\d{6,14}").unwrap(),

        // === Local formats (existing) ===
        Regex::new(r"0\d{1,4}[/\s-]?\d{4,10}").unwrap(),
        Regex::new(r"\(0\d{1,4}\)\s?\d{3,10}").unwrap(),
    ]
});
```

### Step 2: Update Module Doc Comment

```rust
//! Phone number detection for global formats.
//!
//! Supports:
//! - DACH region (Germany, Austria, Switzerland)
//! - US/Canada (NANP)
//! - UK (landline and mobile)
//! - France
//! - Generic E.164 international format
```

### Step 3: Add Tests

Add test functions for each format:

```rust
#[test]
fn test_detect_us_e164() {
    let detector = PhoneDetector::new();
    let matches = detector.detect("Call +1 555 123 4567 for info");
    assert_eq!(matches.len(), 1);
    assert!(matches[0].text.contains("+1"));
}

#[test]
fn test_detect_us_parentheses() {
    let detector = PhoneDetector::new();
    let matches = detector.detect("Phone: (555) 123-4567");
    assert_eq!(matches.len(), 1);
}

#[test]
fn test_detect_uk_mobile() {
    let detector = PhoneDetector::new();
    let matches = detector.detect("Mobile: +44 7911 123456");
    assert_eq!(matches.len(), 1);
}

#[test]
fn test_detect_france() {
    let detector = PhoneDetector::new();
    let matches = detector.detect("Tel: +33 1 23 45 67 89");
    assert_eq!(matches.len(), 1);
}

#[test]
fn test_detect_generic_e164() {
    let detector = PhoneDetector::new();
    let matches = detector.detect("Japan: +81 3 1234 5678");
    assert_eq!(matches.len(), 1);
}

#[test]
fn test_existing_dach_still_works() {
    let detector = PhoneDetector::new();
    // Verify all existing tests still pass
    let matches = detector.detect("Call: +43 664 1234567");
    assert_eq!(matches.len(), 1);
}
```

## Test Commands

```bash
# Run phone detector tests only
cargo test -p veil-detect phone

# Run all detect tests
cargo test -p veil-detect

# Run with output
cargo test -p veil-detect phone -- --nocapture
```

## Verification Checklist

- [ ] All existing DACH tests pass (backward compatibility)
- [ ] US E.164 format detected: +1 555 123 4567
- [ ] US parentheses format detected: (555) 123-4567
- [ ] US 10-digit format detected: 555-123-4567
- [ ] UK E.164 format detected: +44 20 7946 0958
- [ ] UK mobile detected: +44 7911 123456
- [ ] UK local mobile detected: 07911 123456
- [ ] French E.164 format detected: +33 1 23 45 67 89
- [ ] Generic E.164 catches Japan: +81 3 1234 5678
- [ ] No overlapping matches for same number
- [ ] Minimum 7 digits enforced
- [ ] Maximum 15 digits enforced
