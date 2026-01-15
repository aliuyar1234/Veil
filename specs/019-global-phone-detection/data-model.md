# Data Model: Global Phone Number Detection

## Pattern Structure

### Current Structure (Unchanged)

```rust
/// Regex patterns for phone numbers in various formats.
static PHONE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Patterns compiled at startup
    ]
});
```

### Extended Pattern List

```rust
static PHONE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // === DACH Region (existing - keep first for backward compatibility) ===
        // International format: +43 664 1234567 or +49 89 12345678
        Regex::new(r"\+(?:43|49|41)\s?[\d\s/-]{7,15}").unwrap(),
        // With country code prefix: 0043, 0049, 0041
        Regex::new(r"00(?:43|49|41)\s?[\d\s/-]{7,15}").unwrap(),

        // === US/Canada (NANP) ===
        // E.164: +1 555 123 4567
        Regex::new(r"\+1[\s.-]?\d{3}[\s.-]?\d{3}[\s.-]?\d{4}").unwrap(),
        // With 1 prefix: 1-555-123-4567
        Regex::new(r"1[\s.-]\d{3}[\s.-]\d{3}[\s.-]\d{4}").unwrap(),
        // Parentheses: (555) 123-4567
        Regex::new(r"\(\d{3}\)[\s.-]?\d{3}[\s.-]?\d{4}").unwrap(),
        // 10-digit: 555-123-4567 (requires separators to avoid matching other numbers)
        Regex::new(r"\d{3}[\s.-]\d{3}[\s.-]\d{4}").unwrap(),

        // === UK ===
        // E.164: +44 20 7946 0958 or +44 7911 123456
        Regex::new(r"\+44[\s.-]?\d{2,4}[\s.-]?\d{3,4}[\s.-]?\d{3,6}").unwrap(),
        // Local mobile: 07911 123456
        Regex::new(r"07\d{3}[\s.-]?\d{6}").unwrap(),

        // === France ===
        // E.164: +33 1 23 45 67 89
        Regex::new(r"\+33[\s.-]?\d[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2}").unwrap(),

        // === Generic E.164 (catch-all for any country) ===
        // +[country code][number] - 7 to 15 digits total
        Regex::new(r"\+[1-9]\d{6,14}").unwrap(),

        // === Local formats (existing DACH, lower priority) ===
        // Austrian/German local format: 01/234567 or 089/12345678
        Regex::new(r"0\d{1,4}[/\s-]?\d{4,10}").unwrap(),
        // Parentheses format: (01) 234 567
        Regex::new(r"\(0\d{1,4}\)\s?\d{3,10}").unwrap(),
    ]
});
```

## Entities

### Match (Existing - No Change)

```rust
pub struct Match {
    /// Start byte offset in source text
    pub start: usize,
    /// End byte offset in source text
    pub end: usize,
    /// The matched text
    pub text: String,
}
```

### PhoneDetector (Existing - Minor Updates)

```rust
pub struct PhoneDetector;

impl Detector for PhoneDetector {
    fn name(&self) -> &str { "phone" }
    fn category(&self) -> PiiCategory { PiiCategory::Phone }
    fn detect(&self, text: &str) -> Vec<Match> { /* ... */ }
    fn validate(&self, matched: &str) -> ValidationStatus { /* ... */ }
    fn base_confidence(&self) -> f32 { 0.9 }  // May adjust per-pattern
}
```

## Validation Rules

### Digit Count Validation

| Condition | Status | Reason |
|-----------|--------|--------|
| < 7 digits | Invalid | Too short for phone number |
| 7-15 digits | Unvalidated | Valid length range |
| > 15 digits | Invalid | Exceeds E.164 maximum |

### Pattern Priority

Patterns are evaluated in order. First match wins. This ensures:
1. Specific country patterns match before generic E.164
2. DACH patterns maintain backward compatibility
3. No overlapping detections for the same number
