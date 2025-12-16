//! YAML configuration loader for custom context rules.
//!
//! This module provides loading of context rules from YAML configuration files,
//! allowing users to define custom boost/suppress patterns.

use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::error::ContextError;
use super::rule::{ContextAction, ContextRule};
use crate::category::PiiCategory;

/// Configuration file schema version.
const CURRENT_SCHEMA_VERSION: &str = "1.0";

/// A context configuration file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Schema version for compatibility checking.
    #[serde(default = "default_version")]
    pub version: String,

    /// Language this config applies to (ISO 639-1: en, de, fr, etc.).
    #[serde(default)]
    pub language: Option<String>,

    /// Description of this config file.
    #[serde(default)]
    pub description: Option<String>,

    /// The context rules defined in this config.
    pub rules: Vec<RuleConfig>,
}

fn default_version() -> String {
    CURRENT_SCHEMA_VERSION.to_string()
}

/// A rule as defined in the YAML config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Regex pattern to match.
    pub pattern: String,

    /// Action to take (boost, suppress, neutral).
    pub action: ActionConfig,

    /// Weight of the adjustment (0.0 - 1.0).
    pub weight: f32,

    /// PII category this rule applies to (optional).
    #[serde(default)]
    pub category: Option<String>,

    /// Language this rule applies to (optional, overrides config-level language).
    #[serde(default)]
    pub language: Option<String>,

    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,

    /// Context window size in characters.
    #[serde(default)]
    pub window_size: Option<usize>,
}

/// Action type in config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionConfig {
    Boost,
    Suppress,
    Neutral,
}

impl From<ActionConfig> for ContextAction {
    fn from(config: ActionConfig) -> Self {
        match config {
            ActionConfig::Boost => ContextAction::Boost,
            ActionConfig::Suppress => ContextAction::Suppress,
            ActionConfig::Neutral => ContextAction::Neutral,
        }
    }
}

/// Loader for context configuration files.
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load context rules from a YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Vec<ContextRule>, ContextError> {
        let config: ContextConfig = serde_yaml::from_str(yaml)
            .map_err(|e| ContextError::ConfigLoadError(format!("Invalid YAML: {}", e)))?;

        Self::validate_config(&config)?;
        Self::convert_config(config)
    }

    /// Load context rules from a YAML file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Vec<ContextRule>, ContextError> {
        let path = path.as_ref();
        let mut file = std::fs::File::open(path).map_err(|e| {
            ContextError::ConfigLoadError(format!("Failed to open {}: {}", path.display(), e))
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            ContextError::ConfigLoadError(format!("Failed to read {}: {}", path.display(), e))
        })?;

        Self::from_yaml(&contents)
    }

    /// Load context rules from a reader.
    pub fn from_reader<R: Read>(reader: R) -> Result<Vec<ContextRule>, ContextError> {
        let config: ContextConfig = serde_yaml::from_reader(reader)
            .map_err(|e| ContextError::ConfigLoadError(format!("Invalid YAML: {}", e)))?;

        Self::validate_config(&config)?;
        Self::convert_config(config)
    }

    /// Load and merge multiple config files.
    pub fn from_files(paths: &[impl AsRef<Path>]) -> Result<Vec<ContextRule>, ContextError> {
        let mut all_rules = Vec::new();

        for path in paths {
            let rules = Self::from_file(path)?;
            all_rules.extend(rules);
        }

        Ok(all_rules)
    }

    /// Validate a config before conversion.
    fn validate_config(config: &ContextConfig) -> Result<(), ContextError> {
        // Check version compatibility
        if config.version != CURRENT_SCHEMA_VERSION {
            return Err(ContextError::ConfigLoadError(format!(
                "Unsupported config version: {} (expected {})",
                config.version, CURRENT_SCHEMA_VERSION
            )));
        }

        // Validate each rule
        for (idx, rule) in config.rules.iter().enumerate() {
            // Check pattern is valid regex
            if regex::Regex::new(&rule.pattern).is_err() {
                return Err(ContextError::ConfigLoadError(format!(
                    "Rule {}: Invalid regex pattern: {}",
                    idx + 1,
                    rule.pattern
                )));
            }

            // Check weight is in valid range
            if !(0.0..=1.0).contains(&rule.weight) {
                return Err(ContextError::ConfigLoadError(format!(
                    "Rule {}: Weight must be between 0.0 and 1.0, got {}",
                    idx + 1,
                    rule.weight
                )));
            }
        }

        Ok(())
    }

    /// Convert config to ContextRule objects.
    fn convert_config(config: ContextConfig) -> Result<Vec<ContextRule>, ContextError> {
        let config_language = config.language;

        config
            .rules
            .into_iter()
            .map(|rule_config| {
                let mut rule = ContextRule::new(
                    rule_config.pattern,
                    rule_config.action.into(),
                    rule_config.weight,
                );

                // Set language (rule-level overrides config-level)
                if let Some(lang) = rule_config.language.or_else(|| config_language.clone()) {
                    rule = rule.with_language(lang);
                }

                // Set category if provided
                if let Some(cat_str) = rule_config.category {
                    let category = Self::parse_category(&cat_str);
                    rule = rule.with_category(category);
                }

                // Set description if provided
                if let Some(desc) = rule_config.description {
                    rule = rule.with_description(desc);
                }

                // Set window size if provided
                if let Some(size) = rule_config.window_size {
                    rule = rule.with_window_size(size);
                }

                Ok(rule)
            })
            .collect()
    }

    /// Parse a category string into PiiCategory.
    fn parse_category(cat: &str) -> PiiCategory {
        match cat.to_lowercase().as_str() {
            "email" => PiiCategory::Email,
            "phone" => PiiCategory::Phone,
            "ssn" | "social_security" => PiiCategory::Custom("SSN".to_string()),
            "credit_card" | "creditcard" => PiiCategory::CreditCard,
            "ipv4" | "ip_address" | "ip" => PiiCategory::Ipv4,
            "ipv6" => PiiCategory::Ipv6,
            "date_of_birth" | "dob" => PiiCategory::Custom("DateOfBirth".to_string()),
            "passport" => PiiCategory::Custom("Passport".to_string()),
            "drivers_license" | "driver_license" => {
                PiiCategory::Custom("DriversLicense".to_string())
            }
            "bank_account" => PiiCategory::Custom("BankAccount".to_string()),
            "iban" => PiiCategory::Iban,
            "svnr_at" => PiiCategory::SvnrAt,
            "svnr_de" => PiiCategory::SvnrDe,
            "tax_id" => PiiCategory::TaxId,
            "mac_address" => PiiCategory::MacAddress,
            _ => PiiCategory::Custom(cat.to_string()),
        }
    }

    /// Generate a sample config file as a string.
    pub fn sample_config() -> String {
        r#"# Veil Context Configuration
# Custom rules for context-aware PII detection

version: "1.0"
language: en
description: "Custom context rules for my organization"

rules:
  # Boost confidence when "patient name" label appears
  - pattern: "(?i)\\bpatient\\s+name\\s*:"
    action: boost
    weight: 0.4
    category: PersonName
    description: "Medical patient name label"

  # Suppress false positives for product codes
  - pattern: "(?i)\\bproduct\\s+code\\s*:"
    action: suppress
    weight: 0.5
    category: CreditCard
    description: "Product code is not a credit card"

  # Suppress version numbers that look like IPs
  - pattern: "(?i)\\bv(?:ersion)?\\s*\\d"
    action: suppress
    weight: 0.6
    category: Ipv4
    description: "Version number, not IP address"
"#
        .to_string()
    }

    /// Write a sample config file to disk.
    pub fn write_sample_config(path: impl AsRef<Path>) -> Result<(), ContextError> {
        let sample = Self::sample_config();
        std::fs::write(path.as_ref(), sample).map_err(|e| {
            ContextError::ConfigLoadError(format!("Failed to write sample config: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_simple_config() {
        let yaml = r#"
version: "1.0"
language: en
rules:
  - pattern: "(?i)\\bmr\\.\\s+"
    action: boost
    weight: 0.3
    description: "Honorific"
"#;

        let rules = ConfigLoader::from_yaml(yaml).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].action, ContextAction::Boost);
        assert_eq!(rules[0].weight, 0.3);
    }

    #[test]
    fn test_load_with_category() {
        let yaml = r#"
version: "1.0"
rules:
  - pattern: "(?i)\\bversion\\s+"
    action: suppress
    weight: 0.5
    category: Ipv4
"#;

        let rules = ConfigLoader::from_yaml(yaml).unwrap();
        assert_eq!(rules[0].category, Some(PiiCategory::Ipv4));
    }

    #[test]
    fn test_invalid_version() {
        let yaml = r#"
version: "2.0"
rules:
  - pattern: "test"
    action: boost
    weight: 0.5
"#;

        let result = ConfigLoader::from_yaml(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("version"));
    }

    #[test]
    fn test_invalid_regex() {
        let yaml = r#"
version: "1.0"
rules:
  - pattern: "[invalid"
    action: boost
    weight: 0.5
"#;

        let result = ConfigLoader::from_yaml(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("regex"));
    }

    #[test]
    fn test_invalid_weight() {
        let yaml = r#"
version: "1.0"
rules:
  - pattern: "test"
    action: boost
    weight: 1.5
"#;

        let result = ConfigLoader::from_yaml(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Weight"));
    }

    #[test]
    fn test_language_inheritance() {
        let yaml = r#"
version: "1.0"
language: de
rules:
  - pattern: "(?i)\\bherr\\s+"
    action: boost
    weight: 0.3
  - pattern: "(?i)\\bmr\\.\\s+"
    action: boost
    weight: 0.3
    language: en
"#;

        let rules = ConfigLoader::from_yaml(yaml).unwrap();
        assert_eq!(rules[0].language, Some("de".to_string()));
        assert_eq!(rules[1].language, Some("en".to_string())); // Overridden
    }

    #[test]
    fn test_multiple_rules() {
        let yaml = r#"
version: "1.0"
rules:
  - pattern: "rule1"
    action: boost
    weight: 0.1
  - pattern: "rule2"
    action: suppress
    weight: 0.2
  - pattern: "rule3"
    action: neutral
    weight: 0.3
"#;

        let rules = ConfigLoader::from_yaml(yaml).unwrap();
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].action, ContextAction::Boost);
        assert_eq!(rules[1].action, ContextAction::Suppress);
        assert_eq!(rules[2].action, ContextAction::Neutral);
    }

    #[test]
    fn test_sample_config_is_valid() {
        let sample = ConfigLoader::sample_config();
        let result = ConfigLoader::from_yaml(&sample);
        assert!(
            result.is_ok(),
            "Sample config should be valid: {:?}",
            result
        );
    }

    #[test]
    fn test_parse_categories() {
        assert_eq!(ConfigLoader::parse_category("email"), PiiCategory::Email);
        assert_eq!(
            ConfigLoader::parse_category("SSN"),
            PiiCategory::Custom("SSN".to_string())
        );
        assert_eq!(
            ConfigLoader::parse_category("credit_card"),
            PiiCategory::CreditCard
        );
        assert_eq!(ConfigLoader::parse_category("ipv4"), PiiCategory::Ipv4);
        assert_eq!(
            ConfigLoader::parse_category("PersonName"),
            PiiCategory::Custom("PersonName".to_string())
        );
    }

    #[test]
    fn test_window_size() {
        let yaml = r#"
version: "1.0"
rules:
  - pattern: "test"
    action: boost
    weight: 0.3
    window_size: 100
"#;

        let rules = ConfigLoader::from_yaml(yaml).unwrap();
        assert_eq!(rules[0].window_size, 100);
    }
}
