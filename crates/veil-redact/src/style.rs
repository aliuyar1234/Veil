//! Redaction style definitions.

use serde::{Deserialize, Serialize};

use crate::mask::MaskingRule;

/// Style of redaction to apply.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RedactionStyle {
    /// Replace with category label: [EMAIL], [IBAN]
    #[default]
    Label,

    /// Replace with solid characters: ████████
    BlackBar {
        /// Character to use for the bar (default: █)
        #[serde(default = "default_bar_char")]
        char: char,
    },

    /// Partial masking with rules
    Mask(MaskingRule),

    /// Custom replacement text
    Custom {
        /// The text to replace PII with
        text: String,
    },
}

fn default_bar_char() -> char {
    '█'
}

impl RedactionStyle {
    /// Create a label style.
    pub fn label() -> Self {
        RedactionStyle::Label
    }

    /// Create a black bar style with the default character.
    pub fn black_bar() -> Self {
        RedactionStyle::BlackBar {
            char: default_bar_char(),
        }
    }

    /// Create a black bar style with a custom character.
    pub fn black_bar_with_char(c: char) -> Self {
        RedactionStyle::BlackBar { char: c }
    }

    /// Create a masking style.
    pub fn mask(rule: MaskingRule) -> Self {
        RedactionStyle::Mask(rule)
    }

    /// Create a custom replacement style.
    pub fn custom(text: impl Into<String>) -> Self {
        RedactionStyle::Custom { text: text.into() }
    }
}
