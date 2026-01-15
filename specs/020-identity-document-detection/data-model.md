# Data Model: Identity Document Detection

## New PII Categories

### Category Enum Extensions

```rust
// In crates/veil-detect/src/category.rs

pub enum PiiCategory {
    // ... existing categories ...

    /// US Social Security Number
    Ssn,
    /// Passport number (any country)
    Passport,
    /// Driver's license number
    DriversLicense,
}
```

### Display Implementation

```rust
impl fmt::Display for PiiCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // ... existing ...
            PiiCategory::Ssn => write!(f, "SSN"),
            PiiCategory::Passport => write!(f, "PASSPORT"),
            PiiCategory::DriversLicense => write!(f, "DRIVERS_LICENSE"),
        }
    }
}

impl PiiCategory {
    pub fn as_str(&self) -> &str {
        match self {
            // ... existing ...
            PiiCategory::Ssn => "ssn",
            PiiCategory::Passport => "passport",
            PiiCategory::DriversLicense => "drivers_license",
        }
    }
}
```

## Detector Structures

### SSN Detector

```rust
// In crates/veil-detect/src/patterns/ssn.rs

/// Regex patterns for US Social Security Numbers.
static SSN_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Hyphenated: 123-45-6789
        Regex::new(r"\d{3}-\d{2}-\d{4}").unwrap(),
        // Space-separated: 123 45 6789
        Regex::new(r"\d{3}\s\d{2}\s\d{4}").unwrap(),
    ]
});

/// Detector for US Social Security Numbers.
pub struct SsnDetector;

impl Detector for SsnDetector {
    fn name(&self) -> &str { "ssn" }
    fn category(&self) -> PiiCategory { PiiCategory::Ssn }
    fn detect(&self, text: &str) -> Vec<Match> { /* ... */ }
    fn validate(&self, matched: &str) -> ValidationStatus { /* ... */ }
    fn base_confidence(&self) -> f32 { 0.95 }
}
```

### Passport Detector

```rust
// In crates/veil-detect/src/patterns/passport.rs

/// Regex patterns for passport numbers.
static PASSPORT_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // US passport: 9 digits
        Regex::new(r"\d{9}").unwrap(),
        // US passport with letter: A12345678
        Regex::new(r"[A-Z]\d{8}").unwrap(),
        // Generic alphanumeric: ABC123456
        Regex::new(r"[A-Z0-9]{9}").unwrap(),
    ]
});

/// Detector for passport numbers.
pub struct PassportDetector;

impl Detector for PassportDetector {
    fn name(&self) -> &str { "passport" }
    fn category(&self) -> PiiCategory { PiiCategory::Passport }
    fn detect(&self, text: &str) -> Vec<Match> { /* ... */ }
    fn validate(&self, matched: &str) -> ValidationStatus { /* ... */ }
    fn base_confidence(&self) -> f32 { 0.85 }
}
```

### Driver's License Detector

```rust
// In crates/veil-detect/src/patterns/drivers_license.rs

/// Regex patterns for US driver's license numbers.
static DL_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // California: 1 letter + 7 digits
        Regex::new(r"[A-Z]\d{7}").unwrap(),
        // Texas: 8 digits
        Regex::new(r"\d{8}").unwrap(),
        // Florida: 1 letter + 12 digits
        Regex::new(r"[A-Z]\d{12}").unwrap(),
        // Illinois: 1 letter + 11 digits
        Regex::new(r"[A-Z]\d{11}").unwrap(),
    ]
});

/// Detector for driver's license numbers.
pub struct DriversLicenseDetector;

impl Detector for DriversLicenseDetector {
    fn name(&self) -> &str { "drivers_license" }
    fn category(&self) -> PiiCategory { PiiCategory::DriversLicense }
    fn detect(&self, text: &str) -> Vec<Match> { /* ... */ }
    fn validate(&self, matched: &str) -> ValidationStatus { /* ... */ }
    fn base_confidence(&self) -> f32 { 0.80 }
}
```

## Validation Rules

### SSN Validation

```rust
// In crates/veil-detect/src/validators/ssn.rs

/// Invalid SSN area numbers (first 3 digits).
const INVALID_AREAS: &[&str] = &["000", "666"];

/// Check if area number is in reserved range (900-999).
fn is_reserved_area(area: &str) -> bool {
    area.starts_with('9')
}

/// Validate SSN format and area number.
pub fn validate_ssn(ssn: &str) -> ValidationStatus {
    // Extract digits only
    let digits: String = ssn.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() != 9 {
        return ValidationStatus::Invalid {
            reason: format!("SSN must have 9 digits, got {}", digits.len()),
        };
    }

    let area = &digits[0..3];
    let group = &digits[3..5];
    let serial = &digits[5..9];

    // Check invalid area numbers
    if INVALID_AREAS.contains(&area) || is_reserved_area(area) {
        return ValidationStatus::Invalid {
            reason: format!("Invalid SSN area number: {}", area),
        };
    }

    // Check invalid group
    if group == "00" {
        return ValidationStatus::Invalid {
            reason: "Invalid SSN group number: 00".to_string(),
        };
    }

    // Check invalid serial
    if serial == "0000" {
        return ValidationStatus::Invalid {
            reason: "Invalid SSN serial number: 0000".to_string(),
        };
    }

    ValidationStatus::Unvalidated
}
```

## Context Rules

### SSN Context Labels

```yaml
# Addition to context rules configuration
- pattern: "SSN:?"
  category: ssn
  boost: 0.20

- pattern: "Social Security( Number)?:?"
  category: ssn
  boost: 0.20

- pattern: "SS#:?"
  category: ssn
  boost: 0.15
```

### Passport Context Labels

```yaml
- pattern: "Passport( No)?:?"
  category: passport
  boost: 0.20

- pattern: "Travel Document:?"
  category: passport
  boost: 0.15
```

### Driver's License Context Labels

```yaml
- pattern: "Driver'?s? License:?"
  category: drivers_license
  boost: 0.20

- pattern: "DL:?"
  category: drivers_license
  boost: 0.15

- pattern: "License No:?"
  category: drivers_license
  boost: 0.10
```

## Integration Points

### Registry Registration

```rust
// In crates/veil-detect/src/registry.rs

impl Default for DetectorRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        // ... existing detectors ...
        registry.register(Box::new(SsnDetector::new()));
        registry.register(Box::new(PassportDetector::new()));
        registry.register(Box::new(DriversLicenseDetector::new()));
        registry
    }
}
```

### Module Exports

```rust
// In crates/veil-detect/src/patterns/mod.rs

mod ssn;
mod passport;
mod drivers_license;

pub use ssn::SsnDetector;
pub use passport::PassportDetector;
pub use drivers_license::DriversLicenseDetector;
```
