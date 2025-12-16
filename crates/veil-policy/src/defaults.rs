//! Default policy definition.

use veil_detect::PiiCategory;

use crate::rules::{DetectionRule, ProtectionAction, ProtectionRule};
use crate::schema::Policy;

/// Create the default policy.
///
/// Default policy:
/// - Enables all built-in detectors
/// - Confidence threshold: 0.5
/// - Protection: Redact with labels
pub fn default_policy() -> Policy {
    Policy {
        version: "1.0".to_string(),
        name: "Default".to_string(),
        locale: None,
        detection: vec![DetectionRule {
            types: vec![
                PiiCategory::Email,
                PiiCategory::Iban,
                PiiCategory::Phone,
                PiiCategory::CreditCard,
            ],
            confidence_threshold: 0.5,
            enabled: true,
        }],
        protection: vec![ProtectionRule {
            types: vec![
                PiiCategory::Email,
                PiiCategory::Iban,
                PiiCategory::Phone,
                PiiCategory::CreditCard,
            ],
            action: ProtectionAction::Redact,
            style: Some("label".to_string()),
            consistent: false,
            key_ref: None,
        }],
    }
}
