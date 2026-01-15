# Data Model: PII Memory Zeroization

## New Types

### SensitiveString

A string wrapper that securely zeroes its contents when dropped.

```rust
// In crates/veil-core/src/sensitive.rs

use std::fmt;
use std::ops::Deref;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// A string that securely zeroes its contents when dropped.
///
/// This type is used for PII data that should not persist in memory
/// after use. It implements automatic zeroization via the `Drop` trait.
///
/// # Example
///
/// ```rust
/// use veil_core::SensitiveString;
///
/// let sensitive = SensitiveString::new("secret-pii-data");
/// // Use like a regular string...
/// println!("Length: {}", sensitive.len());
/// // When dropped, contents are securely zeroed
/// ```
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SensitiveString(String);

impl SensitiveString {
    /// Create a new sensitive string.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Create an empty sensitive string.
    pub fn empty() -> Self {
        Self(String::new())
    }

    /// Get the inner string (consumes self, transfers ownership).
    ///
    /// WARNING: The returned String will NOT be automatically zeroed.
    /// Only use this when you need ownership and will handle cleanup.
    pub fn into_inner(mut self) -> String {
        std::mem::take(&mut self.0)
    }

    /// Get the length in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Drop for SensitiveString {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl Deref for SensitiveString {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for SensitiveString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for SensitiveString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SensitiveString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Debug for SensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SensitiveString([REDACTED {} bytes])", self.0.len())
    }
}

impl fmt::Display for SensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display shows actual content (for legitimate use cases)
        write!(f, "{}", self.0)
    }
}

impl Default for SensitiveString {
    fn default() -> Self {
        Self::empty()
    }
}
```

## Modified Structs

### Finding (veil-detect)

```rust
// In crates/veil-detect/src/finding.rs

use veil_core::SensitiveString;

/// A detected PII instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// The matched text (securely zeroed on drop).
    pub matched_text: SensitiveString,  // Changed from String

    /// PII category.
    pub category: PiiCategory,

    /// Start position in the segment's content (byte offset).
    pub start: usize,

    /// End position (exclusive) in the segment's content (byte offset).
    pub end: usize,

    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,

    /// Validation status.
    pub validation: ValidationStatus,

    /// Index of the source segment (from ParseResult).
    pub segment_index: usize,

    /// Context reasoning (if context analysis was applied).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_reasoning: Option<Vec<String>>,
}
```

### TextSegment (veil-parsers)

```rust
// In crates/veil-parsers/src/types.rs

use veil_core::SensitiveString;

/// A text segment extracted from a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSegment {
    /// The extracted text content (securely zeroed on drop).
    pub content: SensitiveString,  // Changed from String

    /// Position in the original document.
    pub position: Position,
}
```

### ValidationStatus (veil-detect)

```rust
// In crates/veil-detect/src/finding.rs

use veil_core::SensitiveString;

/// Validation status of a detected PII match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    /// Pattern matched and validation passed.
    Valid,
    /// Pattern matched but validation failed.
    Invalid {
        /// Reason for validation failure (may contain PII context).
        reason: SensitiveString,  // Changed from String
    },
    /// Pattern matched, no validation available.
    Unvalidated,
}
```

## Dependency Graph

```text
veil-core (NEW)
    └── SensitiveString

veil-parsers
    └── depends on veil-core (for SensitiveString in TextSegment)

veil-detect
    └── depends on veil-core (for SensitiveString in Finding)
    └── depends on veil-parsers (existing)

veil-api
    └── depends on veil-detect (existing)
    └── depends on veil-core (for response cleanup)

veil-cli
    └── depends on veil-detect (existing)
    └── depends on veil-core (for output cleanup)
```

## Zeroization Behavior

### Automatic Zeroization (via Drop)

| Type | Field | When Zeroed |
|------|-------|-------------|
| `Finding` | `matched_text` | When Finding is dropped |
| `TextSegment` | `content` | When TextSegment is dropped |
| `ValidationStatus::Invalid` | `reason` | When ValidationStatus is dropped |

### Manual Zeroization (explicit call)

For cases where immediate cleanup is needed before the variable goes out of scope:

```rust
use zeroize::Zeroize;

let mut response_body = ScanResponse { ... };
// ... use response_body ...
response_body.zeroize();  // Explicit cleanup
```
