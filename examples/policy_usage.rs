//! Policy-based protection example.
//!
//! Demonstrates how to use YAML policies to control PII protection.
//!
//! # Running
//!
//! ```bash
//! cargo run --example policy_usage
//! ```

use veil_detect::registry::DetectorRegistry;
use veil_policy::{parse_policy, PolicyExecutor};

fn main() {
    println!("=== Veil Policy Usage Example ===\n");

    // Sample document with PII
    let document = r#"
        Invoice #12345

        Bill To:
        John Doe
        john.doe@company.com
        Phone: 555-123-4567

        Payment Method:
        Credit Card: 4111-1111-1111-1111

        Shipping Address:
        123 Main Street
        Anytown, ST 12345
    "#;

    println!("Original document:");
    println!("{}\n", document);

    // Define a protection policy
    let policy_yaml = r#"
version: "1.0"
name: "Invoice Protection Policy"

# Detection rules - filter which PII to act on
detection:
  - pii_type: email
    min_confidence: 0.8
  - pii_type: phone
    min_confidence: 0.7
  - pii_type: credit_card
    min_confidence: 0.9

# Protection rules - how to handle each PII type
protection:
  # Redact emails with label
  - pii_type: email
    action: redact

  # Mask phone numbers, showing last 4
  - pii_type: phone
    action: mask
    mask_char: "*"

  # Fully redact credit cards
  - pii_type: credit_card
    action: redact
"#;

    println!("Policy configuration:");
    println!("{}\n", policy_yaml);

    // Parse the policy
    let policy = match parse_policy(policy_yaml) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to parse policy: {}", e);
            return;
        }
    };

    println!("Loaded policy: {} (version {})\n", policy.name, policy.version);

    // Detect PII in document
    let registry = DetectorRegistry::default();
    let findings = registry.detect(document);

    println!("Detected {} PII items\n", findings.len());

    // Create policy executor
    let mut executor = match PolicyExecutor::from_policy(&policy) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to create executor: {}", e);
            return;
        }
    };

    // Apply policy to protect the document
    let result = match executor.process(document, &findings, &policy) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to process document: {}", e);
            return;
        }
    };

    println!("=== Protected Document ===\n");
    println!("{}\n", result.protected_text);

    println!("=== Processing Statistics ===\n");
    println!("Total findings: {}", result.stats.total_findings);
    println!("Actions applied: {}", result.stats.actions_applied);
    println!("Findings filtered: {}", result.stats.findings_filtered);

    println!("\n=== Applied Actions ===\n");
    for action in &result.applied_actions {
        println!(
            "- {} at position {}-{}: {}",
            action.pii_category, action.position.start, action.position.end, action.action_type
        );
    }
}
