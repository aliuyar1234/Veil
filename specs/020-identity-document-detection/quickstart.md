# Quickstart: Identity Document Detection

## Overview

Add detection for US SSN, passport numbers (US, UK, EU), and driver's license numbers from major US states.

## New Files

### 1. SSN Detector

**`crates/veil-detect/src/patterns/ssn.rs`**

```rust
//! US Social Security Number detection.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

/// Regex patterns for US Social Security Numbers.
static SSN_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Hyphenated: 123-45-6789
        Regex::new(r"\d{3}-\d{2}-\d{4}").unwrap(),
        // Space-separated: 123 45 6789
        Regex::new(r"\d{3}\s\d{2}\s\d{4}").unwrap(),
    ]
});

/// Invalid SSN area numbers.
const INVALID_AREAS: &[&str] = &["000", "666"];

/// Detector for US Social Security Numbers.
pub struct SsnDetector;

impl SsnDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SsnDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for SsnDetector {
    fn name(&self) -> &str {
        "ssn"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::Ssn
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut seen_ranges: Vec<(usize, usize)> = Vec::new();

        for pattern in SSN_PATTERNS.iter() {
            for m in pattern.find_iter(text) {
                let range = (m.start(), m.end());
                if !seen_ranges.iter().any(|&(s, e)|
                    (range.0 >= s && range.0 < e) || (range.1 > s && range.1 <= e)
                ) {
                    matches.push(Match {
                        start: m.start(),
                        end: m.end(),
                        text: m.as_str().to_string(),
                    });
                    seen_ranges.push(range);
                }
            }
        }

        matches.sort_by_key(|m| m.start);
        matches
    }

    fn validate(&self, matched: &str) -> ValidationStatus {
        let digits: String = matched.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() != 9 {
            return ValidationStatus::Invalid {
                reason: format!("SSN must have 9 digits, got {}", digits.len()),
            };
        }

        let area = &digits[0..3];
        let group = &digits[3..5];
        let serial = &digits[5..9];

        if INVALID_AREAS.contains(&area) || area.starts_with('9') {
            return ValidationStatus::Invalid {
                reason: format!("Invalid SSN area number: {}", area),
            };
        }

        if group == "00" {
            return ValidationStatus::Invalid {
                reason: "Invalid SSN group number: 00".to_string(),
            };
        }

        if serial == "0000" {
            return ValidationStatus::Invalid {
                reason: "Invalid SSN serial number: 0000".to_string(),
            };
        }

        ValidationStatus::Unvalidated
    }

    fn base_confidence(&self) -> f32 {
        0.95
    }
}
```

### 2. Passport Detector

**`crates/veil-detect/src/patterns/passport.rs`**

```rust
//! Passport number detection for US, UK, and EU formats.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

/// Regex patterns for passport numbers.
/// NOTE: These require context to avoid excessive false positives.
static PASSPORT_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Alphanumeric with letter prefix: A12345678
        Regex::new(r"[A-Z]\d{8}").unwrap(),
        // Generic 9-character alphanumeric
        Regex::new(r"[A-Z0-9]{9}").unwrap(),
    ]
});

/// Detector for passport numbers.
pub struct PassportDetector;

impl PassportDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PassportDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for PassportDetector {
    fn name(&self) -> &str {
        "passport"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::Passport
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut seen_ranges: Vec<(usize, usize)> = Vec::new();

        for pattern in PASSPORT_PATTERNS.iter() {
            for m in pattern.find_iter(text) {
                let range = (m.start(), m.end());
                if !seen_ranges.iter().any(|&(s, e)|
                    (range.0 >= s && range.0 < e) || (range.1 > s && range.1 <= e)
                ) {
                    matches.push(Match {
                        start: m.start(),
                        end: m.end(),
                        text: m.as_str().to_string(),
                    });
                    seen_ranges.push(range);
                }
            }
        }

        matches.sort_by_key(|m| m.start);
        matches
    }

    fn validate(&self, matched: &str) -> ValidationStatus {
        let len = matched.len();
        if (6..=9).contains(&len) {
            ValidationStatus::Unvalidated
        } else {
            ValidationStatus::Invalid {
                reason: format!("Passport number should be 6-9 characters, got {}", len),
            }
        }
    }

    fn base_confidence(&self) -> f32 {
        0.85
    }
}
```

### 3. Driver's License Detector

**`crates/veil-detect/src/patterns/drivers_license.rs`**

```rust
//! US Driver's License number detection for major states.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::category::PiiCategory;
use crate::detector::{Detector, Match};
use crate::finding::ValidationStatus;

/// Regex patterns for US driver's license numbers.
static DL_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Florida: 1 letter + 12 digits
        Regex::new(r"[A-Z]\d{12}").unwrap(),
        // Illinois: 1 letter + 11 digits
        Regex::new(r"[A-Z]\d{11}").unwrap(),
        // California: 1 letter + 7 digits
        Regex::new(r"[A-Z]\d{7}").unwrap(),
        // Texas: 8 digits
        Regex::new(r"\d{8}").unwrap(),
    ]
});

/// Detector for driver's license numbers.
pub struct DriversLicenseDetector;

impl DriversLicenseDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DriversLicenseDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for DriversLicenseDetector {
    fn name(&self) -> &str {
        "drivers_license"
    }

    fn category(&self) -> PiiCategory {
        PiiCategory::DriversLicense
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut seen_ranges: Vec<(usize, usize)> = Vec::new();

        for pattern in DL_PATTERNS.iter() {
            for m in pattern.find_iter(text) {
                let range = (m.start(), m.end());
                if !seen_ranges.iter().any(|&(s, e)|
                    (range.0 >= s && range.0 < e) || (range.1 > s && range.1 <= e)
                ) {
                    matches.push(Match {
                        start: m.start(),
                        end: m.end(),
                        text: m.as_str().to_string(),
                    });
                    seen_ranges.push(range);
                }
            }
        }

        matches.sort_by_key(|m| m.start);
        matches
    }

    fn validate(&self, matched: &str) -> ValidationStatus {
        let len = matched.len();
        if (7..=13).contains(&len) {
            ValidationStatus::Unvalidated
        } else {
            ValidationStatus::Invalid {
                reason: format!("Driver's license should be 7-13 characters, got {}", len),
            }
        }
    }

    fn base_confidence(&self) -> f32 {
        0.80
    }
}
```

## Update Existing Files

### 1. Add PII Categories

**`crates/veil-detect/src/category.rs`**

Add to `PiiCategory` enum:
```rust
/// US Social Security Number
Ssn,
/// Passport number
Passport,
/// Driver's license number
DriversLicense,
```

Add to `Display` impl:
```rust
PiiCategory::Ssn => write!(f, "SSN"),
PiiCategory::Passport => write!(f, "PASSPORT"),
PiiCategory::DriversLicense => write!(f, "DRIVERS_LICENSE"),
```

Add to `as_str` method:
```rust
PiiCategory::Ssn => "ssn",
PiiCategory::Passport => "passport",
PiiCategory::DriversLicense => "drivers_license",
```

### 2. Export New Detectors

**`crates/veil-detect/src/patterns/mod.rs`**

```rust
mod ssn;
mod passport;
mod drivers_license;

pub use ssn::SsnDetector;
pub use passport::PassportDetector;
pub use drivers_license::DriversLicenseDetector;
```

## Test Commands

```bash
# Run SSN detector tests only
cargo test -p veil-detect ssn

# Run all identity document tests
cargo test -p veil-detect ssn passport drivers_license

# Run all detect tests
cargo test -p veil-detect

# Run with output
cargo test -p veil-detect ssn -- --nocapture
```

## Verification Checklist

- [ ] SSN hyphenated format detected: 123-45-6789
- [ ] SSN space format detected: 123 45 6789
- [ ] SSN invalid areas flagged: 000, 666, 9XX
- [ ] SSN invalid groups flagged: XX-00-XXXX
- [ ] US passport detected with context: Passport: 123456789
- [ ] UK passport detected with context
- [ ] EU passport alphanumeric detected
- [ ] California DL detected: A1234567
- [ ] Texas DL detected: 12345678
- [ ] Florida DL detected: A123456789012
- [ ] Illinois DL detected: A12345678901
- [ ] All existing tests still pass
- [ ] No overlapping matches
- [ ] Clippy warnings resolved
