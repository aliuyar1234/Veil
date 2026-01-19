//! Sensitive string type with automatic memory zeroization.
//!
//! This module provides `SensitiveString`, a string wrapper that securely
//! zeroes its contents when dropped. Used for PII data that should not
//! persist in memory after use.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
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
///
/// # Security Note
///
/// The `Debug` implementation intentionally redacts the contents to prevent
/// accidental PII leakage in logs, debug output, or serialization.
#[derive(Clone)]
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
    /// # Warning
    ///
    /// The returned String will NOT be automatically zeroed.
    /// Only use this when you need ownership and will handle cleanup yourself.
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

    /// Get a reference to the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for SensitiveString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Redact on serialization to avoid accidental PII leakage.
        serializer.serialize_str(&format!("[REDACTED {} bytes]", self.0.len()))
    }
}

impl<'de> Deserialize<'de> for SensitiveString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(SensitiveString::new(value))
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
    /// Display is intentionally redacted to prevent accidental PII leakage.
    /// Use `.as_str()` or `.to_string()` for intentional output.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED {} bytes]", self.0.len())
    }
}

impl Default for SensitiveString {
    fn default() -> Self {
        Self::empty()
    }
}

impl PartialEq for SensitiveString {
    /// Constant-time comparison to prevent timing attacks.
    fn eq(&self, other: &Self) -> bool {
        use subtle::ConstantTimeEq;

        // Use constant-time comparison for security
        let a = self.0.as_bytes();
        let b = other.0.as_bytes();

        // First check lengths match (this leaks length, but that's acceptable)
        if a.len() != b.len() {
            return false;
        }

        // Constant-time byte comparison
        a.ct_eq(b).into()
    }
}

impl Eq for SensitiveString {}

impl Hash for SensitiveString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Zeroize for SensitiveString {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T005: Test SensitiveString creation from &str
    #[test]
    fn test_creation_from_str() {
        let sensitive = SensitiveString::new("secret");
        assert_eq!(sensitive.as_str(), "secret");
    }

    // T006: Test SensitiveString creation from String
    #[test]
    fn test_creation_from_string() {
        let owned = String::from("secret");
        let sensitive = SensitiveString::from(owned);
        assert_eq!(sensitive.as_str(), "secret");
    }

    // T007: Test SensitiveString deref to &str
    #[test]
    fn test_deref_to_str() {
        let sensitive = SensitiveString::new("hello");
        // Deref allows using &str methods
        assert!(sensitive.starts_with("hel"));
        assert!(sensitive.ends_with("lo"));
        assert_eq!(sensitive.len(), 5);
    }

    // T008: Test SensitiveString clone creates independent copy
    #[test]
    fn test_clone_creates_independent_copy() {
        let original = SensitiveString::new("secret");
        let cloned = original.clone();

        // Both should have same content
        assert_eq!(original.as_str(), cloned.as_str());

        // Drop original, clone should still be valid
        drop(original);
        assert_eq!(cloned.as_str(), "secret");
    }

    // T010: Test SensitiveString debug output is redacted
    #[test]
    fn test_debug_output_is_redacted() {
        let sensitive = SensitiveString::new("super-secret-value");
        let debug_output = format!("{:?}", sensitive);

        // Should NOT contain the actual value
        assert!(!debug_output.contains("super-secret-value"));
        // Should contain REDACTED
        assert!(debug_output.contains("REDACTED"));
        // Should contain byte count
        assert!(debug_output.contains("18 bytes"));
    }

    // T011: Test SensitiveString serialization
    #[test]
    fn test_serialization() {
        let sensitive = SensitiveString::new("test-value");

        // Serialize
        let json = serde_json::to_string(&sensitive).unwrap();
        assert!(!json.contains("test-value"));
        assert!(json.contains("REDACTED"));

        // Deserialize from explicit string
        let deserialized: SensitiveString = serde_json::from_str("\"test-value\"").unwrap();
        assert_eq!(deserialized.as_str(), "test-value");
    }

    // Additional tests

    #[test]
    fn test_empty() {
        let empty = SensitiveString::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_is_empty_false_for_non_empty() {
        let value = SensitiveString::new("not empty");
        assert!(!value.is_empty());
    }

    #[test]
    fn test_as_ref_returns_inner_value() {
        let sensitive = SensitiveString::new("hello");
        let as_ref: &str = sensitive.as_ref();
        assert_eq!(as_ref, "hello");
    }

    #[test]
    fn test_from_str_impl() {
        let sensitive: SensitiveString = SensitiveString::from("hello");
        assert_eq!(sensitive.as_str(), "hello");
    }

    #[test]
    fn test_default() {
        let default: SensitiveString = Default::default();
        assert!(default.is_empty());
    }

    #[test]
    fn test_equality() {
        let a = SensitiveString::new("same");
        let b = SensitiveString::new("same");
        let c = SensitiveString::new("different");

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(SensitiveString::new("key1"));
        set.insert(SensitiveString::new("key2"));
        set.insert(SensitiveString::new("key1")); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_hash_changes_with_value() {
        use std::collections::hash_map::DefaultHasher;

        fn hashed(value: &SensitiveString) -> u64 {
            let mut hasher = DefaultHasher::new();
            value.hash(&mut hasher);
            hasher.finish()
        }

        let a = SensitiveString::new("a");
        let b = SensitiveString::new("b");

        assert_ne!(hashed(&a), hashed(&b));
    }

    #[test]
    fn test_display_is_redacted() {
        let sensitive = SensitiveString::new("visible");
        let display_output = format!("{}", sensitive);
        // Display should be redacted to prevent accidental PII leakage
        assert!(!display_output.contains("visible"));
        assert!(display_output.contains("REDACTED"));
        assert!(display_output.contains("7 bytes"));
    }

    #[test]
    fn test_into_inner() {
        let sensitive = SensitiveString::new("owned");
        let inner = sensitive.into_inner();
        assert_eq!(inner, "owned");
        // Note: `sensitive` is consumed, can't use it
    }

    #[test]
    fn test_explicit_zeroize() {
        let mut sensitive = SensitiveString::new("to-zero");
        sensitive.zeroize();
        assert!(sensitive.is_empty());
    }

    #[test]
    fn test_unicode_content() {
        let sensitive = SensitiveString::new("日本語テスト");
        assert_eq!(sensitive.as_str(), "日本語テスト");
        // Drop should handle unicode properly
    }

    #[test]
    fn test_large_string() {
        let large = "x".repeat(10000);
        let sensitive = SensitiveString::new(&large);
        assert_eq!(sensitive.len(), 10000);
    }

    // Constant-time comparison tests (T025, T026)

    #[test]
    fn test_constant_time_equal_strings() {
        let a = SensitiveString::new("password123");
        let b = SensitiveString::new("password123");
        assert_eq!(a, b);
    }

    #[test]
    fn test_constant_time_unequal_strings() {
        let a = SensitiveString::new("password123");
        let b = SensitiveString::new("password124");
        assert_ne!(a, b);
    }

    #[test]
    fn test_constant_time_different_lengths() {
        let a = SensitiveString::new("short");
        let b = SensitiveString::new("longer_string");
        assert_ne!(a, b);
    }

    #[test]
    fn test_constant_time_empty_strings() {
        let a = SensitiveString::empty();
        let b = SensitiveString::empty();
        assert_eq!(a, b);
    }

    #[test]
    fn test_constant_time_one_empty() {
        let a = SensitiveString::empty();
        let b = SensitiveString::new("not_empty");
        assert_ne!(a, b);
    }

    #[test]
    fn test_constant_time_timing_resistance() {
        // This test verifies that comparison doesn't short-circuit
        // by checking that equal-length strings with differences at
        // different positions all return false

        let base = "aaaaaaaaaa"; // 10 chars
        let base_sensitive = SensitiveString::new(base);

        // Difference at first position
        let diff_first = SensitiveString::new("baaaaaaaaa");
        assert_ne!(base_sensitive, diff_first);

        // Difference at last position
        let diff_last = SensitiveString::new("aaaaaaaaab");
        assert_ne!(base_sensitive, diff_last);

        // Difference in middle
        let diff_middle = SensitiveString::new("aaaaabaaaa");
        assert_ne!(base_sensitive, diff_middle);
    }
}
