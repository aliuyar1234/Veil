//! Masking rules for partial redaction.

use serde::{Deserialize, Serialize};

/// Rules for partial masking of PII.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingRule {
    /// Number of characters to show at start.
    #[serde(default)]
    pub show_first: usize,

    /// Number of characters to show at end.
    #[serde(default)]
    pub show_last: usize,

    /// Character to use for masking.
    #[serde(default = "default_mask_char")]
    pub mask_char: char,

    /// Characters to preserve (e.g., '@' in email).
    #[serde(default)]
    pub preserve: Vec<char>,
}

fn default_mask_char() -> char {
    '*'
}

impl Default for MaskingRule {
    fn default() -> Self {
        Self {
            show_first: 1,
            show_last: 4,
            mask_char: default_mask_char(),
            preserve: Vec::new(),
        }
    }
}

impl MaskingRule {
    /// Create a new masking rule.
    pub fn new(show_first: usize, show_last: usize) -> Self {
        Self {
            show_first,
            show_last,
            ..Default::default()
        }
    }

    /// Set the mask character.
    pub fn with_mask_char(mut self, c: char) -> Self {
        self.mask_char = c;
        self
    }

    /// Set characters to preserve.
    pub fn with_preserve(mut self, chars: Vec<char>) -> Self {
        self.preserve = chars;
        self
    }

    /// Apply the masking rule to a string.
    pub fn apply(&self, text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len <= self.show_first + self.show_last {
            // Too short to mask meaningfully
            return text.to_string();
        }

        let mut result = String::with_capacity(text.len());

        for (i, c) in chars.iter().enumerate() {
            let should_show =
                i < self.show_first || i >= len - self.show_last || self.preserve.contains(c);

            if should_show {
                result.push(*c);
            } else {
                result.push(self.mask_char);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_masking() {
        let rule = MaskingRule::new(1, 4);
        // Without preserve, @ is also masked
        assert_eq!(rule.apply("john@example.com"), "j***********.com");
    }

    #[test]
    fn test_preserve_at_sign() {
        let rule = MaskingRule::new(1, 4).with_preserve(vec!['@']);
        assert_eq!(rule.apply("john@example.com"), "j***@*******.com");
    }

    #[test]
    fn test_short_string() {
        let rule = MaskingRule::new(2, 2);
        assert_eq!(rule.apply("ab"), "ab");
    }

    #[test]
    fn test_custom_mask_char() {
        let rule = MaskingRule::new(0, 4).with_mask_char('X');
        assert_eq!(rule.apply("1234567890"), "XXXXXX7890");
    }
}
