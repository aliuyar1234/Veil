//! Policy command implementation.

use std::fs;

use miette::{IntoDiagnostic, Result};

use veil_policy::load_policy;

use crate::cli::{PolicyAction, PolicyArgs};

/// Default policy template.
const DEFAULT_POLICY_TEMPLATE: &str = r#"# Veil Policy Configuration
# See documentation for all options

name: "default"
version: "1.0"
locale: "en"

# Detection rules - which PII types to detect
detection:
  - type: email
    enabled: true
    min_confidence: 0.8
  - type: phone
    enabled: true
    min_confidence: 0.7
  - type: credit_card
    enabled: true
    min_confidence: 0.9
  - type: ssn
    enabled: true
    min_confidence: 0.9
  - type: iban
    enabled: true
    min_confidence: 0.85

# Protection rules - how to handle detected PII
protection:
  - type: email
    action: redact
    style: label
  - type: phone
    action: mask
    mask_char: "*"
    visible_chars: 4
  - type: credit_card
    action: mask
    mask_char: "*"
    visible_chars: 4
  - type: ssn
    action: redact
    style: bar
  - type: iban
    action: mask
    mask_char: "X"
    visible_chars: 4
"#;

/// Run the policy command.
pub fn run(args: PolicyArgs) -> Result<()> {
    match args.action {
        PolicyAction::Validate { path } => match load_policy(&path) {
            Ok(policy) => {
                println!("Policy valid: {}", policy.name());
                println!("  Version: {}", policy.version());
                if let Some(ref locale) = policy.locale {
                    println!("  Locale: {}", locale);
                }
                println!("  Detection rules: {}", policy.detection.len());
                println!("  Protection rules: {}", policy.protection.len());
                Ok(())
            }
            Err(e) => {
                eprintln!("Policy invalid: {}", e);
                std::process::exit(1);
            }
        },
        PolicyAction::Init { output } => {
            if output.exists() {
                return Err(miette::miette!(
                    "File already exists: {}. Use a different name or delete the existing file.",
                    output.display()
                ));
            }
            fs::write(&output, DEFAULT_POLICY_TEMPLATE).into_diagnostic()?;
            println!("Created policy file: {}", output.display());
            println!("Edit the file to customize detection and protection rules.");
            Ok(())
        }
    }
}
