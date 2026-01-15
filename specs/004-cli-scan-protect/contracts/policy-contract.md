# Contract: Policy Command

**Feature**: 004-cli-scan-protect | **Command**: `veil policy`

## Command Signature

```bash
veil policy <ACTION>
```

## Subcommands

### `veil policy validate`

Validates a policy file for syntax and semantic correctness.

```bash
veil policy validate <PATH>
```

#### Required Arguments

| Argument | Type | Description |
|----------|------|-------------|
| `PATH` | `PathBuf` | Path to YAML policy file to validate |

#### Optional Flags

None.

## Output Contract

### Success Case

```bash
$ veil policy validate gdpr.yaml
Policy valid: GDPR Compliance Policy
  Version: 1.0.0
  Locale: de-AT
  Detection rules: 5
  Protection rules: 5
```

**Rules**:
- Output goes to stdout
- Shows policy name, version, locale (if set), rule counts
- Exit code 0

### Error Case (Syntax Error)

```bash
$ veil policy validate bad.yaml
Policy invalid: YAML parse error at line 12: expected ':'
```

**Rules**:
- Error message goes to stderr
- Shows line number if available
- Exit code 1

### Error Case (Schema Validation Error)

```bash
$ veil policy validate invalid.yaml
Policy invalid: Unknown detector category 'FOOBAR' in detection rule 3
```

**Rules**:
- Error message goes to stderr
- Shows specific validation error
- Exit code 1

### Error Case (File Not Found)

```bash
$ veil policy validate missing.yaml
Policy invalid: File not found: missing.yaml
```

**Rules**:
- Error message goes to stderr
- Exit code 1

## Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Policy is valid |
| 1 | Policy is invalid (syntax error, schema error, file not found) |

## Behavioral Contracts

### BC-1: Validate Valid Policy

**Given**: A syntactically and semantically correct policy file
**When**: Running `veil policy validate policy.yaml`
**Then**:
- Policy is loaded and validated
- Success message with policy metadata is shown
- Exit code 0

### BC-2: Validate Policy with Syntax Error

**Given**: A policy file with YAML syntax error
**When**: Running `veil policy validate bad.yaml`
**Then**:
- Error message with line number is shown
- Exit code 1
- No policy metadata is shown

### BC-3: Validate Policy with Unknown Detector

**Given**: A policy file referencing unknown detector category
**When**: Running `veil policy validate policy.yaml`
**Then**:
- Warning or error about unknown detector
- Exit code 1 (validation fails)

### BC-4: Validate Policy with Invalid Confidence

**Given**: A policy with confidence value outside [0.0, 1.0]
**When**: Running `veil policy validate policy.yaml`
**Then**:
- Error message: "Invalid confidence value: must be between 0.0 and 1.0"
- Exit code 1

### BC-5: Validate Non-Existent File

**Given**: A non-existent policy file path
**When**: Running `veil policy validate missing.yaml`
**Then**:
- Error message: "File not found: missing.yaml"
- Exit code 1

### BC-6: Validate Empty Policy File

**Given**: An empty policy file
**When**: Running `veil policy validate empty.yaml`
**Then**:
- Error message: "Policy file is empty or invalid"
- Exit code 1

## Policy File Schema (Reference)

### Minimal Valid Policy

```yaml
name: Minimal Policy
version: 1.0
detection: []
protection: []
```

### Full Policy Example

```yaml
name: GDPR Compliance Policy
version: 1.0.0
locale: de-AT

detection:
  - category: EMAIL
    confidence: 0.7
    enabled: true

  - category: PHONE
    confidence: 0.8
    enabled: true

  - category: IBAN
    confidence: 0.9
    enabled: true

protection:
  - category: EMAIL
    action: redact
    style: label

  - category: PHONE
    action: mask
    mask_options:
      visible_prefix: 2
      visible_suffix: 2

  - category: IBAN
    action: encrypt
    key_ref: "env:ENCRYPTION_KEY"
```

## Validation Rules

### VR-1: Required Fields

- `name` (string): Policy name
- `version` (string): Semantic version
- `detection` (array): List of detection rules (can be empty)
- `protection` (array): List of protection rules (can be empty)

### VR-2: Optional Fields

- `locale` (string): Locale code (e.g., "en-US", "de-AT")
- Custom fields are ignored (forward compatibility)

### VR-3: Detection Rule Schema

Each detection rule must have:
- `category` (string): Valid detector category
- `confidence` (float): Value between 0.0 and 1.0 (optional, default: 0.5)
- `enabled` (bool): Whether detector is enabled (optional, default: true)

### VR-4: Protection Rule Schema

Each protection rule must have:
- `category` (string): Valid detector category
- `action` (string): One of: `redact`, `mask`, `encrypt`, `pseudonymize`
- Additional fields depend on action type

### VR-5: Known Detector Categories

Valid categories (as of this spec):
- `EMAIL`
- `PHONE`
- `IBAN`
- `SSN`
- `CREDIT_CARD`
- Custom categories (with warning)

## Edge Cases

### EC-1: Policy with Comments

**Given**: Policy file with YAML comments
**When**: Running `veil policy validate policy.yaml`
**Then**:
- Comments are ignored (standard YAML behavior)
- Policy validates successfully if otherwise valid
- Exit code 0

### EC-2: Policy with Unknown Fields

**Given**: Policy with fields not in schema
**When**: Running `veil policy validate policy.yaml`
**Then**:
- Unknown fields are ignored (forward compatibility)
- Warning message: "Unknown field 'foo' will be ignored"
- Policy validates successfully if otherwise valid
- Exit code 0

### EC-3: Policy with Duplicate Categories

**Given**: Policy with multiple rules for same category
**When**: Running `veil policy validate policy.yaml`
**Then**:
- Warning: "Duplicate detection rule for EMAIL; last one wins"
- Policy validates successfully
- Exit code 0

### EC-4: Policy with Circular Key References

**Given**: Policy with key_ref pointing to non-existent key
**When**: Running `veil policy validate policy.yaml`
**Then**:
- Validation passes (key resolution happens at runtime, not validation time)
- Warning: "Key reference 'env:MISSING_KEY' may not be resolvable at runtime"
- Exit code 0

### EC-5: Very Large Policy File

**Given**: Policy file with 1000+ rules
**When**: Running `veil policy validate policy.yaml`
**Then**:
- Validation completes (may take longer)
- Exit code 0 if valid

## Performance Guarantees

| Metric | Target | Notes |
|--------|--------|-------|
| Small policy (<10 rules) | <100ms | Parse + validate |
| Large policy (>100 rules) | <1 second | Parse + validate |

## Compatibility

| Platform | Support |
|----------|---------|
| Linux | ✅ Full support |
| macOS | ✅ Full support |
| Windows | ✅ Full support |

## Examples

### Example 1: Validate Valid Policy

```bash
$ cat gdpr.yaml
name: GDPR Policy
version: 1.0
detection:
  - category: EMAIL
    confidence: 0.8
protection:
  - category: EMAIL
    action: redact

$ veil policy validate gdpr.yaml
Policy valid: GDPR Policy
  Version: 1.0
  Detection rules: 1
  Protection rules: 1
```

### Example 2: Syntax Error

```bash
$ cat bad.yaml
name: Bad Policy
version: 1.0
detection:
  - category: EMAIL
    confidence: 0.8
  - category: PHONE
  confidence: 0.9  # Wrong indentation!

$ veil policy validate bad.yaml
Policy invalid: YAML parse error at line 7: invalid indentation
```

### Example 3: Invalid Confidence

```bash
$ cat invalid.yaml
name: Invalid Policy
version: 1.0
detection:
  - category: EMAIL
    confidence: 1.5  # Out of range!

$ veil policy validate invalid.yaml
Policy invalid: Invalid confidence value for EMAIL: 1.5 (must be between 0.0 and 1.0)
```

### Example 4: Unknown Detector

```bash
$ cat unknown.yaml
name: Unknown Detector
version: 1.0
detection:
  - category: FOOBAR  # Unknown category

$ veil policy validate unknown.yaml
Policy invalid: Unknown detector category 'FOOBAR' in detection rule 1
Valid categories: EMAIL, PHONE, IBAN, SSN, CREDIT_CARD
```

## Test Implementation

```rust
#[test]
fn contract_policy_validate_valid() {
    let temp = TempDir::new().unwrap();
    let policy = temp.path().join("policy.yaml");

    fs::write(&policy, r#"
name: Test Policy
version: 1.0
detection:
  - category: EMAIL
    confidence: 0.8
protection:
  - category: EMAIL
    action: redact
"#).unwrap();

    let output = Command::new("veil")
        .args(&["policy", "validate", policy.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Policy valid"));
    assert!(stdout.contains("Test Policy"));
}

#[test]
fn contract_policy_validate_invalid() {
    let temp = TempDir::new().unwrap();
    let policy = temp.path().join("policy.yaml");

    fs::write(&policy, "invalid: yaml: content:").unwrap();

    let output = Command::new("veil")
        .args(&["policy", "validate", policy.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Policy invalid"));
}
```

## Future Enhancements

### Planned Subcommands (Out of Scope for Phase 1)

1. **`veil policy init`**: Create a default policy file
2. **`veil policy list-detectors`**: Show available detector categories
3. **`veil policy merge`**: Combine multiple policy files
4. **`veil policy diff`**: Compare two policy files

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-15 | Initial contract definition |
