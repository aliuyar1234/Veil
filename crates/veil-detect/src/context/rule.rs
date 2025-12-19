//! Context rule types and actions.

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::category::PiiCategory;

/// Action to take when a context pattern matches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextAction {
    /// Boost confidence score for nearby PII matches.
    Boost,
    /// Suppress/reduce confidence for nearby PII matches.
    Suppress,
    /// Neutral - no confidence adjustment.
    Neutral,
}

/// A context rule that adjusts detection confidence based on surrounding text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRule {
    /// Regex pattern to match contextual markers.
    pub pattern: String,

    /// Action to take when pattern matches.
    pub action: ContextAction,

    /// Weight of the adjustment (0.0 - 1.0).
    /// For boost: adds to confidence (e.g., 0.3 means +30%).
    /// For suppress: multiplies confidence (e.g., 0.5 means reduce to 50%).
    pub weight: f32,

    /// PII category this rule applies to (None means all categories).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<PiiCategory>,

    /// Language this rule applies to (ISO 639-1 code: en, de, fr, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Human-readable description of the rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Context window size in characters (how far to look before/after match).
    /// Default is 200 characters (~50 words).
    #[serde(default = "default_window_size")]
    pub window_size: usize,

    /// Compiled regex (not serialized).
    #[serde(skip)]
    compiled: Option<Regex>,
}

fn default_window_size() -> usize {
    200
}

impl ContextRule {
    /// Create a new context rule.
    pub fn new(pattern: impl Into<String>, action: ContextAction, weight: f32) -> Self {
        Self {
            pattern: pattern.into(),
            action,
            weight: weight.clamp(0.0, 1.0),
            category: None,
            language: None,
            description: None,
            window_size: default_window_size(),
            compiled: None,
        }
    }

    /// Set the PII category this rule applies to.
    pub fn with_category(mut self, category: PiiCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set the language this rule applies to.
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the context window size.
    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size;
        self
    }

    /// Maximum allowed regex pattern length to prevent ReDoS attacks.
    const MAX_PATTERN_LENGTH: usize = 1000;

    /// Compile the regex pattern.
    ///
    /// # Security
    ///
    /// This method includes ReDoS protection:
    /// - Pattern length is limited to prevent extremely long patterns
    /// - Nested quantifiers are detected and rejected
    pub fn compile(&mut self) -> Result<(), regex::Error> {
        if self.compiled.is_none() {
            // Security: Limit pattern length to prevent ReDoS
            if self.pattern.len() > Self::MAX_PATTERN_LENGTH {
                return Err(regex::Error::Syntax(format!(
                    "Pattern too long: {} bytes (max {})",
                    self.pattern.len(),
                    Self::MAX_PATTERN_LENGTH
                )));
            }

            // Security: Check for potentially dangerous nested quantifiers
            if Self::has_nested_quantifiers(&self.pattern) {
                return Err(regex::Error::Syntax(
                    "Pattern contains nested quantifiers which may cause ReDoS".to_string(),
                ));
            }

            self.compiled = Some(Regex::new(&self.pattern)?);
        }
        Ok(())
    }

    /// Check if pattern contains dangerous nested quantifiers that could cause ReDoS.
    ///
    /// Detects patterns like: (a+)+, (a*)+, (a+)*, (?:a+)+, etc.
    fn has_nested_quantifiers(pattern: &str) -> bool {
        // Simple heuristic: look for quantifier followed by group with quantifier
        // This catches patterns like: (a+)+, (.*)+, (?:x+)*, etc.
        let dangerous_patterns = [
            r"(\+|\*|\?|\})\s*\)", // quantifier before closing paren
        ];

        let bytes = pattern.as_bytes();
        let mut paren_depth = 0;
        let mut has_quantifier_in_group = false;

        for (i, &b) in bytes.iter().enumerate() {
            match b {
                b'(' => {
                    paren_depth += 1;
                    has_quantifier_in_group = false;
                }
                b')' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                        // Check if next char is a quantifier
                        if has_quantifier_in_group {
                            if let Some(&next) = bytes.get(i + 1) {
                                if next == b'+' || next == b'*' || next == b'{' {
                                    return true; // Nested quantifier detected
                                }
                            }
                        }
                    }
                }
                b'+' | b'*' => {
                    if paren_depth > 0 {
                        // Check if previous char was not escaped
                        let prev = if i > 0 { bytes.get(i - 1) } else { None };
                        if prev != Some(&b'\\') {
                            has_quantifier_in_group = true;
                        }
                    }
                }
                _ => {}
            }
        }

        // Also check for some known dangerous patterns
        for pattern_check in dangerous_patterns {
            if regex::Regex::new(pattern_check)
                .map(|re| re.is_match(pattern))
                .unwrap_or(false)
            {
                return true;
            }
        }

        false
    }

    /// Get the compiled regex, compiling if necessary.
    pub fn regex(&mut self) -> Result<&Regex, regex::Error> {
        if self.compiled.is_none() {
            self.compile()?;
        }
        Ok(self.compiled.as_ref().unwrap())
    }

    /// Check if this rule applies to the given category.
    pub fn applies_to_category(&self, category: &PiiCategory) -> bool {
        self.category.is_none() || self.category.as_ref() == Some(category)
    }

    /// Check if this rule applies to the given language.
    pub fn applies_to_language(&self, lang: &str) -> bool {
        self.language.is_none() || self.language.as_deref() == Some(lang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_rule_creation() {
        let mut rule = ContextRule::new(r"(?i)\bmr\.\s+", ContextAction::Boost, 0.3)
            .with_description("Honorific before name");

        assert_eq!(rule.action, ContextAction::Boost);
        assert_eq!(rule.weight, 0.3);
        assert!(rule.compile().is_ok());
    }

    #[test]
    fn test_weight_clamping() {
        let rule = ContextRule::new("test", ContextAction::Boost, 1.5);
        assert_eq!(rule.weight, 1.0);

        let rule = ContextRule::new("test", ContextAction::Suppress, -0.5);
        assert_eq!(rule.weight, 0.0);
    }

    #[test]
    fn test_category_filtering() {
        let rule =
            ContextRule::new("test", ContextAction::Boost, 0.5).with_category(PiiCategory::Email);

        assert!(rule.applies_to_category(&PiiCategory::Email));
        assert!(!rule.applies_to_category(&PiiCategory::Phone));
    }

    #[test]
    fn test_language_filtering() {
        let rule = ContextRule::new("test", ContextAction::Boost, 0.5).with_language("de");

        assert!(rule.applies_to_language("de"));
        assert!(!rule.applies_to_language("en"));
    }
}
