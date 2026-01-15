# Quickstart: Policy Engine

**Feature**: 009-policy-engine
**Date**: 2025-12-15

## Policy File Format

Create a YAML policy file (e.g., `policy.yaml`):

```yaml
version: "1.0"
name: "GDPR Standard"
locale: "de-AT"

detection:
  - types: [email, phone, address]
    confidence_threshold: 0.8
    enabled: true

  - types: [person_name]
    confidence_threshold: 0.6
    enabled: true

protection:
  - types: [email, phone]
    action: redact
    style: label

  - types: [person_name]
    action: pseudonymize
    consistent: true

  - types: [iban, credit_card]
    action: encrypt
    key_ref: "env://VEIL_ENCRYPTION_KEY"
```

## Loading a Policy

```rust
use veil_policy::{load_policy, Policy};

// Load from file
let policy = load_policy("policy.yaml")?;

// Or use default policy
let policy = Policy::default();

println!("Loaded policy: {} v{}", policy.name(), policy.version());
```

## Validating a Policy

```rust
use veil_policy::validate_policy;

let result = validate_policy("policy.yaml");

if result.valid {
    println!("Policy is valid");
} else {
    for error in result.errors {
        eprintln!("Error: {}", error);
    }
}

for warning in result.warnings {
    println!("Warning: {}", warning);
}
```

## Filtering Detection Results

```rust
use veil_policy::{apply_policy_to_findings, load_policy};
use veil_detect::DetectorRegistry;
use veil_parsers::parse_bytes;

// Parse document
let content = b"Contact: max@example.com, Phone: +43 123 456789";
let parsed = parse_bytes(content, &Default::default())?;

// Detect PII
let registry = DetectorRegistry::default();
let findings = registry.detect_all(&parsed.segments);

// Load policy and filter
let policy = load_policy("policy.yaml")?;
let filtered = apply_policy_to_findings(&policy, findings);

// Only findings matching policy rules remain
for finding in filtered {
    println!("{:?}: {}", finding.category, finding.matched_text);
}
```

## Full Protection Pipeline

```rust
use veil_policy::{PolicyExecutor, load_policy};
use veil_parsers::parse_bytes;

// Load policy
let policy = load_policy("policy.yaml")?;

// Create executor
let mut executor = PolicyExecutor::from_policy(&policy)?;

// Process content
let content = "Contact Max Müller at max@example.com. IBAN: DE89370400440532013000";
let result = executor.process(content, &policy)?;

println!("Protected content: {}", result.content);
println!("Findings processed: {}", result.stats.findings_protected);

// Show what was done
for action in &result.actions {
    println!(
        "{:?}: '{}' -> '{}'",
        action.action, action.original, action.protected
    );
}
```

## Key References

### Environment Variable

Set the encryption key in your environment:

```bash
export VEIL_ENCRYPTION_KEY="your-32-byte-base64-encoded-key"
```

Policy configuration:

```yaml
protection:
  - types: [iban]
    action: encrypt
    key_ref: "env://VEIL_ENCRYPTION_KEY"
```

### File-Based Key

Store the key in a file (raw bytes or base64):

```yaml
protection:
  - types: [credit_card]
    action: encrypt
    key_ref: "file:///etc/veil/encryption.key"
```

## Consistent Pseudonymization

For documents where the same person should always get the same pseudonym:

```yaml
protection:
  - types: [person_name]
    action: pseudonymize
    consistent: true
```

```rust
let content = "Max Müller said hello. Later, Max Müller left.";
let result = executor.process(content, &policy)?;
// Both "Max Müller" instances will be replaced with the same pseudonym
```

To maintain consistency across multiple documents in a session, reuse the executor:

```rust
let mut executor = PolicyExecutor::from_policy(&policy)?;

// Process first document
let result1 = executor.process(doc1, &policy)?;

// Process second document (same pseudonyms for same names)
let result2 = executor.process(doc2, &policy)?;

// Clear cache for new session
executor.clear_cache();
```

## Action Types

| Action | Description | Reversible | Requires Key |
|--------|-------------|------------|--------------|
| `redact` | Replace with label like `[EMAIL]` | No | No |
| `mask` | Partial masking like `****@example.com` | No | No |
| `hash` | SHA-256 hash | No | No |
| `pseudonymize` | Fake realistic data | No | No |
| `encrypt` | AES-256-GCM | Yes | Yes |
| `tokenize` | Random token with vault | Yes | No |

## Error Handling

```rust
use veil_policy::{load_policy, PolicyError};

match load_policy("policy.yaml") {
    Ok(policy) => println!("Loaded: {}", policy.name()),
    Err(PolicyError::Io(e)) => eprintln!("File error: {}", e),
    Err(PolicyError::Yaml(e)) => eprintln!("YAML syntax error: {}", e),
    Err(PolicyError::UnsupportedVersion(v)) => eprintln!("Version {} not supported", v),
    Err(PolicyError::KeyRefError(e)) => eprintln!("Key reference error: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Locale-Specific Detection

Locale affects which detectors and dictionaries are active:

```yaml
version: "1.0"
name: "Austrian Policy"
locale: "de-AT"

detection:
  - types: [person_name]  # Uses Austrian name dictionaries
    confidence_threshold: 0.7
```

Supported locales:
- `de-AT` - Austria (German)
- `de-DE` - Germany (German)
- `de-CH` - Switzerland (German)
- `en-US` - United States (English)
- `en-GB` - United Kingdom (English)
