//! Confidence scoring for dictionary matches.

use super::entry::DictionaryEntry;

/// Context signals that can boost confidence.
#[derive(Debug, Clone, Default)]
pub struct ContextSignals {
    /// Text before the match.
    pub prefix: Option<String>,
    /// Whether the match starts with a capital letter.
    pub is_capitalized: bool,
}

impl ContextSignals {
    /// Create from surrounding text.
    pub fn from_context(text: &str, start: usize, end: usize) -> Self {
        // Get prefix (up to 20 chars before)
        let prefix_start = start.saturating_sub(20);
        let prefix = if prefix_start < start {
            Some(text[prefix_start..start].to_string())
        } else {
            None
        };

        // Check capitalization
        let matched = &text[start..end];
        let is_capitalized = matched
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);

        Self {
            prefix,
            is_capitalized,
        }
    }

    /// Check for German honorific prefixes (Herr, Frau, etc.).
    pub fn has_honorific_prefix(&self) -> bool {
        if let Some(prefix) = &self.prefix {
            let lower = prefix.to_lowercase();
            lower.ends_with("herr ")
                || lower.ends_with("frau ")
                || lower.ends_with("hr. ")
                || lower.ends_with("fr. ")
                || lower.ends_with("dr. ")
                || lower.ends_with("prof. ")
        } else {
            false
        }
    }

    /// Calculate context bonus (1.0-1.2).
    pub fn context_bonus(&self) -> f32 {
        let mut bonus: f32 = 1.0;

        // Boost for honorific prefix
        if self.has_honorific_prefix() {
            bonus += 0.15;
        }

        // Small boost for capitalization
        if self.is_capitalized {
            bonus += 0.05;
        }

        bonus.min(1.2)
    }
}

/// Calculate confidence from match type.
pub fn calculate_confidence(
    entry: &DictionaryEntry,
    similarity: Option<f64>,
    context: &ContextSignals,
) -> f32 {
    let base = entry.frequency;
    let match_factor = similarity.map(|s| s as f32).unwrap_or(1.0);
    let bonus = context.context_bonus();

    (base * match_factor * bonus).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::FuzzyMatch;

    #[test]
    fn test_context_signals_from_context() {
        let text = "Herr Maximilian ist hier";
        let signals = ContextSignals::from_context(text, 5, 15);

        assert!(signals.prefix.is_some());
        assert!(signals.prefix.unwrap().contains("Herr"));
        assert!(signals.is_capitalized);
    }

    #[test]
    fn test_honorific_prefix() {
        let signals = ContextSignals {
            prefix: Some("Sehr geehrter Herr ".to_string()),
            is_capitalized: true,
        };

        assert!(signals.has_honorific_prefix());
        assert!(signals.context_bonus() > 1.1);
    }

    #[test]
    fn test_no_honorific() {
        let signals = ContextSignals {
            prefix: Some("Der Name ist ".to_string()),
            is_capitalized: true,
        };

        assert!(!signals.has_honorific_prefix());
    }

    #[test]
    fn test_exact_confidence() {
        let entry = DictionaryEntry::with_frequency("Maria", 0.92);
        let context = ContextSignals::default();

        let confidence = calculate_confidence(&entry, None, &context);
        assert!((confidence - 0.92).abs() < 0.01);
    }

    #[test]
    fn test_exact_confidence_with_bonus() {
        let entry = DictionaryEntry::with_frequency("Maria", 0.92);
        let context = ContextSignals {
            prefix: Some("Frau ".to_string()),
            is_capitalized: true,
        };

        let confidence = calculate_confidence(&entry, None, &context);
        // Should be boosted by honorific + capitalization
        assert!(confidence > 0.92);
    }

    #[test]
    fn test_fuzzy_confidence() {
        let entry = DictionaryEntry::with_frequency("Maximilian", 0.80);
        let fuzzy_match = FuzzyMatch {
            entry,
            similarity: 0.96,
            input: "Maximilain".to_string(),
        };
        let context = ContextSignals::default();

        let confidence =
            calculate_confidence(&fuzzy_match.entry, Some(fuzzy_match.similarity), &context);
        // 0.80 * 0.96 = 0.768
        assert!((confidence - 0.768).abs() < 0.01);
    }

    #[test]
    fn test_confidence_capped_at_one() {
        let entry = DictionaryEntry::with_frequency("Maria", 0.95);
        let context = ContextSignals {
            prefix: Some("Frau ".to_string()),
            is_capitalized: true,
        };

        let confidence = calculate_confidence(&entry, None, &context);
        assert!(confidence <= 1.0);
    }
}
