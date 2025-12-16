//! Detector registry for managing multiple detectors.

use std::collections::{HashMap, HashSet};

use veil_parsers::TextSegment;

use crate::context::ContextAnalyzer;
use crate::detector::Detector;
use crate::finding::{Finding, ValidationStatus};
use crate::patterns::{CreditCardDetector, EmailDetector, IbanDetector, PhoneDetector};

/// Registry of PII detectors.
pub struct DetectorRegistry {
    detectors: HashMap<String, Box<dyn Detector>>,
    enabled: HashSet<String>,
    context_analyzer: Option<ContextAnalyzer>,
}

impl DetectorRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            detectors: HashMap::new(),
            enabled: HashSet::new(),
            context_analyzer: None,
        }
    }

    /// Register a detector.
    pub fn register(&mut self, detector: Box<dyn Detector>) {
        let name = detector.name().to_string();
        self.enabled.insert(name.clone());
        self.detectors.insert(name, detector);
    }

    /// Enable a detector by name.
    pub fn enable(&mut self, name: &str) {
        self.enabled.insert(name.to_string());
    }

    /// Disable a detector by name.
    pub fn disable(&mut self, name: &str) {
        self.enabled.remove(name);
    }

    /// Check if a detector is enabled.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.enabled.contains(name)
    }

    /// Get list of all detector names.
    pub fn detector_names(&self) -> Vec<&str> {
        self.detectors.keys().map(|s| s.as_str()).collect()
    }

    /// Get list of enabled detector names.
    pub fn enabled_detector_names(&self) -> Vec<&str> {
        self.enabled.iter().map(|s| s.as_str()).collect()
    }

    /// Enable context analysis with the default analyzer.
    pub fn enable_context_analysis(&mut self) {
        self.context_analyzer = Some(ContextAnalyzer::new());
    }

    /// Enable context analysis with a custom analyzer.
    pub fn with_context_analyzer(mut self, analyzer: ContextAnalyzer) -> Self {
        self.context_analyzer = Some(analyzer);
        self
    }

    /// Set the context analyzer.
    pub fn set_context_analyzer(&mut self, analyzer: ContextAnalyzer) {
        self.context_analyzer = Some(analyzer);
    }

    /// Disable context analysis.
    pub fn disable_context_analysis(&mut self) {
        self.context_analyzer = None;
    }

    /// Check if context analysis is enabled.
    pub fn has_context_analysis(&self) -> bool {
        self.context_analyzer.is_some()
    }

    /// Detect PII in all segments.
    pub fn detect_all(&self, segments: &[TextSegment]) -> Vec<Finding> {
        let mut findings = Vec::new();

        for (segment_index, segment) in segments.iter().enumerate() {
            for detector in self.detectors.values() {
                if !self.enabled.contains(detector.name()) {
                    continue;
                }

                let matches = detector.detect(&segment.content);
                for m in matches {
                    let validation = detector.validate(&m.text);
                    let confidence = match &validation {
                        ValidationStatus::Valid => detector.base_confidence(),
                        ValidationStatus::Invalid { .. } => detector.base_confidence() * 0.3,
                        ValidationStatus::Unvalidated => detector.base_confidence() * 0.8,
                    };

                    findings.push(Finding::new(
                        m.text,
                        detector.category(),
                        m.start,
                        m.end,
                        confidence,
                        validation,
                        segment_index,
                    ));
                }
            }
        }

        // Sort by position
        findings.sort_by(|a, b| {
            a.segment_index
                .cmp(&b.segment_index)
                .then(a.start.cmp(&b.start))
        });

        findings
    }

    /// Detect PII in all segments with context analysis.
    pub fn detect_all_with_context(
        &mut self,
        segments: &[TextSegment],
        language: Option<&str>,
    ) -> Vec<Finding> {
        let mut findings = self.detect_all(segments);

        // Apply context analysis if enabled
        if let Some(analyzer) = &mut self.context_analyzer {
            for (segment_index, segment) in segments.iter().enumerate() {
                // Collect indices of findings for this segment (avoids cloning)
                let indices: Vec<usize> = findings
                    .iter()
                    .enumerate()
                    .filter_map(|(i, f)| {
                        if f.segment_index == segment_index {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect();

                if indices.is_empty() {
                    continue;
                }

                // Build temporary references for context analysis
                let segment_findings: Vec<&Finding> =
                    indices.iter().map(|&i| &findings[i]).collect();

                // Analyze findings with context (accepts &[&Finding] via coercion)
                let analyses = analyzer.analyze_findings_refs(&segment_findings, &segment.content, language);

                // Update findings with context adjustments
                for (idx, analysis) in indices.iter().zip(analyses.iter()) {
                    findings[*idx].confidence = analysis.adjusted_confidence;
                    if !analysis.reasoning.is_empty() {
                        findings[*idx].add_context_reasoning(analysis.reasoning.clone());
                    }
                }
            }
        }

        findings
    }
}

impl Default for DetectorRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        // Register built-in detectors
        registry.register(Box::new(EmailDetector::new()));
        registry.register(Box::new(IbanDetector::new()));
        registry.register(Box::new(PhoneDetector::new()));
        registry.register(Box::new(CreditCardDetector::new()));

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veil_parsers::Position;

    fn make_segment(content: &str) -> TextSegment {
        TextSegment {
            content: content.to_string(),
            position: Position::Text {
                line: 1,
                column: 1,
                byte_offset: 0,
                byte_length: content.len(),
            },
        }
    }

    #[test]
    fn test_detect_email() {
        let registry = DetectorRegistry::default();
        let segments = vec![make_segment("Contact: john@example.com")];
        let findings = registry.detect_all(&segments);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].matched_text, "john@example.com");
    }

    #[test]
    fn test_detect_multiple() {
        let registry = DetectorRegistry::default();
        let segments = vec![make_segment(
            "Email: test@test.org, IBAN: DE89370400440532013000",
        )];
        let findings = registry.detect_all(&segments);

        assert!(findings.len() >= 2);
    }
}
