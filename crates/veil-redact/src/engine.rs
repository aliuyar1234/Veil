//! Redaction engine implementation.

use veil_detect::Finding;

use crate::applied::AppliedRedaction;
use crate::config::RedactionConfig;
use crate::position::{PositionMap, PositionMapEntry};
use crate::result::RedactionResult;
use crate::style::RedactionStyle;

/// Engine for applying redactions to text.
pub struct RedactionEngine {
    config: RedactionConfig,
}

impl RedactionEngine {
    /// Create a new redaction engine with the given configuration.
    pub fn new(config: RedactionConfig) -> Self {
        Self { config }
    }

    /// Redact findings in the given text.
    pub fn redact(&self, text: &str, findings: &[Finding]) -> RedactionResult {
        if findings.is_empty() {
            return RedactionResult::new(text.to_string(), Vec::new(), PositionMap::new());
        }

        // Sort findings by position (ascending), then by length (descending) to prefer longer matches
        let mut sorted_findings: Vec<&Finding> = findings.iter().collect();
        sorted_findings.sort_by(|a, b| {
            a.start
                .cmp(&b.start)
                .then_with(|| (b.end - b.start).cmp(&(a.end - a.start)))
                .then_with(|| {
                    b.confidence
                        .partial_cmp(&a.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        // Remove overlapping findings (keep the longer/higher confidence one)
        let mut non_overlapping: Vec<&Finding> = Vec::new();
        for finding in sorted_findings {
            let dominated = non_overlapping.iter().any(|existing| {
                // Check if the current finding overlaps with an existing one
                if finding.end <= existing.start || finding.start >= existing.end {
                    return false; // No overlap
                }
                // If overlapping, check if existing is better (longer or higher confidence)
                let existing_len = existing.end - existing.start;
                let finding_len = finding.end - finding.start;
                existing_len >= finding_len || existing.confidence > finding.confidence
            });

            if !dominated {
                // Remove any existing findings that this one dominates
                non_overlapping.retain(|existing| {
                    if finding.end <= existing.start || finding.start >= existing.end {
                        return true; // No overlap, keep it
                    }
                    // If overlapping, keep existing if it's longer or higher confidence
                    let existing_len = existing.end - existing.start;
                    let finding_len = finding.end - finding.start;
                    existing_len > finding_len
                        || (existing_len == finding_len
                            && existing.confidence >= finding.confidence)
                });
                non_overlapping.push(finding);
            }
        }

        let mut result_text = text.to_string();
        let mut redactions = Vec::new();
        let mut position_map = PositionMap::new();
        let mut offset: i64 = 0;

        for finding in non_overlapping {
            let style = self.config.get_style(&finding.category);
            let replacement = self.apply_style(style, &finding.matched_text, &finding.category);

            let original_start = finding.start;
            let original_end = finding.end;

            // Calculate new positions with offset (using saturating arithmetic to prevent overflow)
            let adjusted_start = if offset >= 0 {
                original_start.saturating_add(offset as usize)
            } else {
                original_start.saturating_sub((-offset) as usize)
            };
            let adjusted_end = if offset >= 0 {
                original_end.saturating_add(offset as usize)
            } else {
                original_end.saturating_sub((-offset) as usize)
            };

            // Bounds check to prevent panic on replace_range
            if adjusted_start > result_text.len() || adjusted_end > result_text.len() {
                continue; // Skip this finding if positions are out of bounds
            }

            // Apply replacement
            result_text.replace_range(adjusted_start..adjusted_end, &replacement);

            let new_end = adjusted_start + replacement.len();

            // Update offset
            let original_len = original_end - original_start;
            let new_len = replacement.len();
            offset += new_len as i64 - original_len as i64;

            // Record the redaction
            redactions.push(AppliedRedaction::new(
                finding.matched_text.as_str(),
                &replacement,
                (original_start, original_end),
                (adjusted_start, new_end),
                finding.category.clone(),
            ));

            position_map.add(PositionMapEntry {
                original_start,
                original_end,
                redacted_start: adjusted_start,
                redacted_end: new_end,
            });
        }

        RedactionResult::new(result_text, redactions, position_map)
    }

    /// Apply a redaction style to text.
    fn apply_style(
        &self,
        style: &RedactionStyle,
        text: &str,
        category: &veil_detect::PiiCategory,
    ) -> String {
        match style {
            RedactionStyle::Label => {
                format!("[{}]", category)
            }
            RedactionStyle::BlackBar { char: c } => c.to_string().repeat(text.chars().count()),
            RedactionStyle::Mask(rule) => rule.apply(text),
            RedactionStyle::Custom { text: replacement } => replacement.clone(),
        }
    }
}

impl Default for RedactionEngine {
    fn default() -> Self {
        Self::new(RedactionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veil_detect::{Finding, PiiCategory, ValidationStatus};

    fn make_finding(text: &str, start: usize, category: PiiCategory) -> Finding {
        Finding::new(
            text,
            category,
            start,
            start + text.len(),
            1.0,
            ValidationStatus::Valid,
            0,
        )
    }

    #[test]
    fn test_label_redaction() {
        let engine = RedactionEngine::default();
        let text = "Contact: john@example.com";
        let findings = vec![make_finding("john@example.com", 9, PiiCategory::Email)];

        let result = engine.redact(text, &findings);

        assert_eq!(result.text, "Contact: [EMAIL]");
        assert_eq!(result.redactions.len(), 1);
    }

    #[test]
    fn test_black_bar_redaction() {
        let config = RedactionConfig::with_style(RedactionStyle::black_bar());
        let engine = RedactionEngine::new(config);
        let text = "IBAN: DE89370400440532013000";
        let findings = vec![make_finding("DE89370400440532013000", 6, PiiCategory::Iban)];

        let result = engine.redact(text, &findings);

        assert_eq!(result.text, "IBAN: ██████████████████████");
        assert_eq!(result.text.chars().count() - 6, 22); // Original IBAN length
    }

    #[test]
    fn test_multiple_redactions() {
        let engine = RedactionEngine::default();
        let text = "Email: a@b.com, Phone: +43 664 1234567";
        let findings = vec![
            make_finding("a@b.com", 7, PiiCategory::Email),
            make_finding("+43 664 1234567", 23, PiiCategory::Phone),
        ];

        let result = engine.redact(text, &findings);

        assert!(result.text.contains("[EMAIL]"));
        assert!(result.text.contains("[PHONE]"));
        assert_eq!(result.redactions.len(), 2);
    }

    #[test]
    fn test_no_findings() {
        let engine = RedactionEngine::default();
        let text = "No PII here";
        let findings: Vec<Finding> = Vec::new();

        let result = engine.redact(text, &findings);

        assert_eq!(result.text, text);
        assert!(result.redactions.is_empty());
    }
}
