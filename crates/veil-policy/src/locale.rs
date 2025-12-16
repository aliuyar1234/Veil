//! Locale definitions for region-specific detection.

use serde::{Deserialize, Serialize};

/// Supported locales for region-specific PII detection.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Locale {
    /// Austria
    DeAt,
    /// Germany
    DeDe,
    /// Switzerland
    DeCh,
    /// English (international)
    #[default]
    En,
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Locale::DeAt => write!(f, "de-AT"),
            Locale::DeDe => write!(f, "de-DE"),
            Locale::DeCh => write!(f, "de-CH"),
            Locale::En => write!(f, "en"),
        }
    }
}
