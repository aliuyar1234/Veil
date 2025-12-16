//! Context analyzer for adjusting detection confidence based on surrounding text.

use crate::finding::Finding;

use super::builtin;
use super::marker::{ContextMarker, MarkerType};
use super::rule::{ContextAction, ContextRule};

/// Result of context analysis for a finding.
#[derive(Debug, Clone)]
pub struct ContextAnalysis {
    /// Original confidence score.
    pub original_confidence: f32,

    /// Adjusted confidence score after context analysis.
    pub adjusted_confidence: f32,

    /// Context markers that influenced this finding.
    pub markers: Vec<ContextMarker>,

    /// Human-readable reasoning for the adjustment.
    pub reasoning: Vec<String>,
}

impl ContextAnalysis {
    /// Create a new context analysis with no adjustments.
    pub fn no_adjustment(confidence: f32) -> Self {
        Self {
            original_confidence: confidence,
            adjusted_confidence: confidence,
            markers: Vec::new(),
            reasoning: Vec::new(),
        }
    }

    /// Check if any adjustment was made.
    pub fn has_adjustment(&self) -> bool {
        (self.adjusted_confidence - self.original_confidence).abs() > 0.01
    }
}

/// Context analyzer that adjusts detection confidence based on surrounding text.
pub struct ContextAnalyzer {
    /// Context rules to apply.
    rules: Vec<ContextRule>,

    /// Default language for analysis.
    default_language: String,
}

impl ContextAnalyzer {
    /// Create a new context analyzer with built-in rules.
    pub fn new() -> Self {
        let mut rules = builtin::all_builtin_rules();
        // Compile all regexes upfront
        for rule in &mut rules {
            let _ = rule.compile();
        }

        Self {
            rules,
            default_language: "en".to_string(),
        }
    }

    /// Create a context analyzer with only rules for a specific language.
    pub fn with_language(lang: impl Into<String>) -> Self {
        let lang = lang.into();
        let mut rules = builtin::rules_for_language(&lang);
        for rule in &mut rules {
            let _ = rule.compile();
        }

        Self {
            rules,
            default_language: lang,
        }
    }

    /// Create an empty context analyzer.
    pub fn empty() -> Self {
        Self {
            rules: Vec::new(),
            default_language: "en".to_string(),
        }
    }

    /// Add a custom rule to the analyzer.
    pub fn add_rule(&mut self, mut rule: ContextRule) {
        let _ = rule.compile();
        self.rules.push(rule);
    }

    /// Add multiple custom rules.
    pub fn add_rules(&mut self, rules: Vec<ContextRule>) {
        for rule in rules {
            self.add_rule(rule);
        }
    }

    /// Detect context markers in text.
    pub fn detect_markers(&mut self, text: &str, language: Option<&str>) -> Vec<ContextMarker> {
        let lang = language.unwrap_or(&self.default_language);
        let mut markers = Vec::new();

        for rule in &mut self.rules {
            if !rule.applies_to_language(lang) {
                continue;
            }

            // Clone necessary fields before borrowing
            let action = rule.action.clone();
            let weight = rule.weight;
            let pattern = rule.pattern.clone();

            if let Ok(regex) = rule.regex() {
                for cap in regex.captures_iter(text) {
                    if let Some(m) = cap.get(0) {
                        let marker_type = match &action {
                            ContextAction::Boost => {
                                if pattern.to_lowercase().contains("email") {
                                    MarkerType::Label
                                } else if pattern.to_lowercase().contains("hon") {
                                    MarkerType::Honorific
                                } else {
                                    MarkerType::Label
                                }
                            }
                            ContextAction::Suppress => MarkerType::Suppression,
                            ContextAction::Neutral => MarkerType::Custom("neutral".to_string()),
                        };

                        markers.push(
                            ContextMarker::new(
                                marker_type,
                                m.as_str(),
                                m.start(),
                                m.end(),
                                action.clone(),
                                weight,
                            )
                            .with_language(lang),
                        );
                    }
                }
            }
        }

        // Sort by position
        markers.sort_by_key(|m| m.start);
        markers
    }

    /// Analyze a single finding and adjust its confidence based on context.
    pub fn analyze_finding(
        &mut self,
        finding: &Finding,
        text: &str,
        language: Option<&str>,
    ) -> ContextAnalysis {
        let markers = self.detect_markers(text, language);
        self.apply_context_to_finding(finding, &markers, text)
    }

    /// Apply context markers to a finding and calculate adjusted confidence.
    fn apply_context_to_finding(
        &self,
        finding: &Finding,
        markers: &[ContextMarker],
        _text: &str,
    ) -> ContextAnalysis {
        let mut analysis = ContextAnalysis::no_adjustment(finding.confidence);
        let mut boost_total = 0.0;
        let mut suppress_multiplier = 1.0;

        // Default context window: 200 characters before/after the finding
        const CONTEXT_WINDOW: usize = 200;

        // Find markers that are relevant to this finding
        for marker in markers {
            // Calculate distance from marker to finding
            let marker_end_to_finding_start = finding.start.saturating_sub(marker.end);
            let finding_end_to_marker_start = marker.start.saturating_sub(finding.end);
            let distance = marker_end_to_finding_start.min(finding_end_to_marker_start);

            // Check if marker is within context window
            let is_nearby =
                distance <= CONTEXT_WINDOW || marker.overlaps(finding.start, finding.end);

            if is_nearby {
                match &marker.action {
                    ContextAction::Boost => {
                        boost_total += marker.weight;
                        analysis.markers.push(marker.clone());
                        analysis.reasoning.push(format!(
                            "Boosted by context marker '{}' at position {}",
                            marker.text, marker.start
                        ));
                    }
                    ContextAction::Suppress => {
                        suppress_multiplier *= 1.0 - marker.weight;
                        analysis.markers.push(marker.clone());
                        analysis.reasoning.push(format!(
                            "Suppressed by context marker '{}' at position {}",
                            marker.text, marker.start
                        ));
                    }
                    ContextAction::Neutral => {}
                }
            }
        }

        // Apply adjustments
        let mut adjusted = finding.confidence;

        // Apply boost (additive)
        if boost_total > 0.0 {
            adjusted = (adjusted + boost_total).min(1.0);
        }

        // Apply suppression (multiplicative)
        if suppress_multiplier < 1.0 {
            adjusted *= suppress_multiplier;
        }

        analysis.adjusted_confidence = adjusted;
        analysis
    }

    /// Analyze multiple findings and adjust their confidence scores.
    pub fn analyze_findings(
        &mut self,
        findings: &[Finding],
        text: &str,
        language: Option<&str>,
    ) -> Vec<ContextAnalysis> {
        let markers = self.detect_markers(text, language);

        findings
            .iter()
            .map(|finding| self.apply_context_to_finding(finding, &markers, text))
            .collect()
    }

    /// Analyze multiple findings from references (avoids cloning in caller).
    pub fn analyze_findings_refs(
        &mut self,
        findings: &[&Finding],
        text: &str,
        language: Option<&str>,
    ) -> Vec<ContextAnalysis> {
        let markers = self.detect_markers(text, language);

        findings
            .iter()
            .map(|&finding| self.apply_context_to_finding(finding, &markers, text))
            .collect()
    }
}

impl Default for ContextAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::PiiCategory;
    use crate::finding::ValidationStatus;

    fn make_finding(text: &str, category: PiiCategory, start: usize, end: usize) -> Finding {
        Finding::new(
            text,
            category,
            start,
            end,
            0.7,
            ValidationStatus::Unvalidated,
            0,
        )
    }

    #[test]
    fn test_context_analyzer_creation() {
        let analyzer = ContextAnalyzer::new();
        assert!(!analyzer.rules.is_empty());
    }

    #[test]
    fn test_detect_markers() {
        let mut analyzer = ContextAnalyzer::new();
        let text = "Contact: Mr. John Smith at john@example.com";
        let markers = analyzer.detect_markers(text, Some("en"));

        assert!(!markers.is_empty());
        // Should find "Mr." and possibly "Contact:"
        assert!(markers.iter().any(|m| m.text.contains("Mr")));
    }

    #[test]
    fn test_boost_with_honorific() {
        let mut analyzer = ContextAnalyzer::new();
        let text = "Dear Mr. Smith";
        let finding = make_finding(
            "Smith",
            PiiCategory::Custom("PersonName".to_string()),
            9,
            14,
        );

        let analysis = analyzer.analyze_finding(&finding, text, Some("en"));

        // Should be boosted due to "Mr."
        assert!(analysis.adjusted_confidence > analysis.original_confidence);
        assert!(!analysis.markers.is_empty());
    }

    #[test]
    fn test_suppress_version_number() {
        let mut analyzer = ContextAnalyzer::new();
        let text = "version 192.168.1.1 released";
        let finding = make_finding("192.168.1.1", crate::PiiCategory::Ipv4, 8, 19);

        let analysis = analyzer.analyze_finding(&finding, text, Some("en"));

        // Should be suppressed due to "version"
        assert!(analysis.adjusted_confidence < analysis.original_confidence);
    }

    #[test]
    fn test_no_adjustment_without_context() {
        let mut analyzer = ContextAnalyzer::new();
        let text = "Some random text with 192.168.1.1 in it";
        let finding = make_finding("192.168.1.1", crate::PiiCategory::Ipv4, 22, 33);

        let analysis = analyzer.analyze_finding(&finding, text, Some("en"));

        // Should have minimal or no adjustment
        assert!((analysis.adjusted_confidence - analysis.original_confidence).abs() < 0.1);
    }
}
