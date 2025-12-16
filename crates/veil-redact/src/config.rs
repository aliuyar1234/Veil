//! Redaction configuration.

use std::collections::HashMap;

use veil_detect::PiiCategory;

use crate::style::RedactionStyle;

/// Configuration for the redaction engine.
#[derive(Debug, Clone)]
pub struct RedactionConfig {
    /// Default style for all categories.
    pub default_style: RedactionStyle,

    /// Per-category style overrides.
    pub category_styles: HashMap<PiiCategory, RedactionStyle>,
}

impl Default for RedactionConfig {
    fn default() -> Self {
        Self {
            default_style: RedactionStyle::Label,
            category_styles: HashMap::new(),
        }
    }
}

impl RedactionConfig {
    /// Create a new configuration with the given default style.
    pub fn with_style(style: RedactionStyle) -> Self {
        Self {
            default_style: style,
            category_styles: HashMap::new(),
        }
    }

    /// Set the style for a specific category.
    pub fn set_category_style(&mut self, category: PiiCategory, style: RedactionStyle) {
        self.category_styles.insert(category, style);
    }

    /// Get the style for a category (falls back to default).
    pub fn get_style(&self, category: &PiiCategory) -> &RedactionStyle {
        self.category_styles
            .get(category)
            .unwrap_or(&self.default_style)
    }
}
