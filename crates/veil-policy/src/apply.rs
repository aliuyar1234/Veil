//! Apply policy to findings and get redaction config.

use veil_detect::Finding;
use veil_redact::{RedactionConfig, RedactionStyle};

use crate::schema::Policy;

/// Apply policy detection rules to filter findings.
pub fn apply_policy_to_findings(policy: &Policy, findings: Vec<Finding>) -> Vec<Finding> {
    findings
        .into_iter()
        .filter(|finding| {
            // Check if any detection rule allows this finding
            for rule in &policy.detection {
                if !rule.enabled {
                    continue;
                }

                // Check if this rule applies to this category
                if rule.types.is_empty() || rule.types.contains(&finding.category) {
                    // Check confidence threshold
                    if finding.confidence >= rule.confidence_threshold {
                        return true;
                    }
                }
            }

            // If no rules defined, allow all
            policy.detection.is_empty()
        })
        .collect()
}

/// Get redaction configuration from policy protection rules.
pub fn get_redaction_config(policy: &Policy) -> RedactionConfig {
    let mut config = RedactionConfig::default();

    for rule in &policy.protection {
        let style = match rule.style.as_deref() {
            Some("label") | None => RedactionStyle::Label,
            Some("bar") | Some("black_bar") => RedactionStyle::black_bar(),
            Some("mask") => RedactionStyle::mask(Default::default()),
            Some(other) => RedactionStyle::custom(format!("[{}]", other)),
        };

        for category in &rule.types {
            config.set_category_style(category.clone(), style.clone());
        }

        // If no specific types, set as default
        if rule.types.is_empty() {
            config.default_style = style;
        }
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use veil_detect::{PiiCategory, ValidationStatus};

    fn make_finding(category: PiiCategory, confidence: f32) -> Finding {
        Finding::new(
            "test",
            category,
            0,
            4,
            confidence,
            ValidationStatus::Valid,
            0,
        )
    }

    #[test]
    fn test_filter_by_confidence() {
        let policy = Policy {
            version: "1.0".to_string(),
            name: "Test".to_string(),
            locale: None,
            detection: vec![crate::rules::DetectionRule {
                types: vec![],
                confidence_threshold: 0.8,
                enabled: true,
            }],
            protection: vec![],
        };

        let findings = vec![
            make_finding(PiiCategory::Email, 0.9),
            make_finding(PiiCategory::Email, 0.7),
        ];

        let filtered = apply_policy_to_findings(&policy, findings);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].confidence, 0.9);
    }
}
