//! Policy loading and validation.

use std::fs;
use std::path::Path;

use crate::error::PolicyError;
use crate::schema::Policy;
use crate::validation::PolicyValidationResult;

/// Load a policy from a YAML file.
pub fn load_policy(path: impl AsRef<Path>) -> Result<Policy, PolicyError> {
    let content = fs::read_to_string(path)?;
    parse_policy(&content)
}

/// Parse a policy from a YAML string.
pub fn parse_policy(yaml: &str) -> Result<Policy, PolicyError> {
    let policy: Policy = serde_yaml::from_str(yaml)?;

    // Validate version
    if !policy.is_version_supported() {
        return Err(PolicyError::UnsupportedVersion(policy.version.clone()));
    }

    Ok(policy)
}

/// Validate a policy file and return detailed results.
pub fn validate_policy(path: impl AsRef<Path>) -> PolicyValidationResult {
    match load_policy(path) {
        Ok(policy) => validate_policy_content(&policy),
        Err(e) => PolicyValidationResult {
            valid: false,
            errors: vec![e.to_string()],
            warnings: Vec::new(),
        },
    }
}

/// Validate policy content.
fn validate_policy_content(policy: &Policy) -> PolicyValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check version
    if !policy.is_version_supported() {
        errors.push(format!("Unsupported version: {}", policy.version));
    }

    // Check for empty rules
    if policy.detection.is_empty() {
        warnings.push("No detection rules defined".to_string());
    }

    if policy.protection.is_empty() {
        warnings.push("No protection rules defined".to_string());
    }

    PolicyValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_policy() {
        let yaml = r#"
version: "1.0"
name: "Test Policy"
detection:
  - types: [email]
    confidence_threshold: 0.8
protection:
  - types: [email]
    action: redact
"#;
        let policy = parse_policy(yaml).unwrap();
        assert_eq!(policy.name, "Test Policy");
        assert_eq!(policy.detection.len(), 1);
    }

    #[test]
    fn test_invalid_version() {
        let yaml = r#"
version: "2.0"
name: "Future Policy"
"#;
        let result = parse_policy(yaml);
        assert!(result.is_err());
    }
}
