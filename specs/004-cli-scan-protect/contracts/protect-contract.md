# Contract: Protect Command

**Feature**: 004-cli-scan-protect | **Command**: `veil protect`

## Command Signature

```bash
veil protect [OPTIONS] <INPUT>
```

## Required Arguments

| Argument | Type | Description |
|----------|------|-------------|
| `INPUT` | `PathBuf` | Input file path (or `-` for stdin) |

## Optional Flags

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--output` | `-o` | PathBuf | None | Output file path (default: stdout) |
| `--policy` | `-p` | PathBuf | None | Path to YAML policy file |
| `--style` | | String | "label" | Redaction style: label, bar, mask |
| `--quiet` | `-q` | bool | false | Suppress progress output |
| `--json` | | bool | false | Output metadata in JSON format |

## Output Contract

### Text Mode (default, output to stdout)

```bash
$ veil protect input.txt
Contact: [EMAIL]
Phone: [PHONE]
Account: [IBAN]
```

**Rules**:
- Redacted content goes to stdout
- Progress/status messages go to stderr
- Exit code 0 if successful

### Text Mode (output to file)

```bash
$ veil protect input.txt -o output.txt
Protected input.txt -> output.txt (3 redactions)
```

**Rules**:
- Redacted content written to file
- Summary message to stdout (unless `--quiet`)
- Exit code 0 if successful

### JSON Mode (`--json`)

**Output to stderr** (to avoid mixing with stdout content):

```json
{
  "input": "input.txt",
  "output": "output.txt",
  "redaction_count": 3
}
```

**Rules**:
- JSON metadata to stderr
- Redacted content to stdout or file (as specified)
- Exit code 0 if successful

## Redaction Styles

### Style: `label` (default)

```
Original: Contact user@example.com for more info.
Redacted: Contact [EMAIL] for more info.
```

### Style: `bar`

```
Original: Contact user@example.com for more info.
Redacted: Contact ████████████████ for more info.
```

### Style: `mask`

```
Original: Contact user@example.com for more info.
Redacted: Contact u***@e******.com for more info.
```

## Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Success (redaction completed) |
| 1 | Error (invalid arguments, file not found, parse error) |

## Behavioral Contracts

### BC-1: Protect File to Stdout

**Given**: An input file with PII
**When**: Running `veil protect input.txt`
**Then**:
- File is parsed and scanned for PII
- PII is redacted using default style (label)
- Redacted content output to stdout
- Exit code 0

### BC-2: Protect File to Output File

**Given**: An input file with PII
**When**: Running `veil protect input.txt -o output.txt`
**Then**:
- File is parsed and scanned for PII
- PII is redacted using default style (label)
- Redacted content written to `output.txt`
- Summary message to stdout: "Protected input.txt -> output.txt (N redactions)"
- Exit code 0

### BC-3: Protect with Custom Style

**Given**: An input file with PII
**When**: Running `veil protect input.txt --style bar`
**Then**:
- PII is redacted with black bars (███)
- Bars match the length of original text
- Exit code 0

### BC-4: Protect with Policy

**Given**: A policy file with specific redaction rules
**When**: Running `veil protect input.txt --policy policy.yaml`
**Then**:
- Policy is loaded and validated
- Findings filtered by policy confidence threshold
- Redaction applied only to filtered findings
- Exit code 0

### BC-5: Protect from Stdin

**Given**: Content piped to stdin
**When**: Running `echo "Email: user@example.com" | veil protect -`
**Then**:
- Content read from stdin
- Parsed as plain text
- Redacted content to stdout
- Exit code 0

### BC-6: No PII in File

**Given**: A file with no detectable PII
**When**: Running `veil protect clean.txt -o output.txt`
**Then**:
- Output file is identical to input file
- Summary: "Protected clean.txt -> output.txt (0 redactions)"
- Exit code 0

### BC-7: Error Handling (File Not Found)

**Given**: A non-existent input file
**When**: Running `veil protect nonexistent.txt`
**Then**:
- Error to stderr: "Error: File not found: nonexistent.txt"
- Exit code 1
- No output file created

### BC-8: Error Handling (Output Exists)

**Given**: Output file already exists
**When**: Running `veil protect input.txt -o existing.txt`
**Then**:
- Error to stderr: "Error: Output file already exists: existing.txt (use --force to overwrite)"
- Exit code 1
- Existing file is NOT modified

**Note**: `--force` flag is out of scope for Phase 1; will be added in future enhancement.

### BC-9: Error Handling (Invalid Style)

**Given**: Unknown redaction style
**When**: Running `veil protect input.txt --style unknown`
**Then**:
- Error to stderr: "Error: Invalid redaction style: unknown (valid: label, bar, mask)"
- Exit code 1

### BC-10: Error Handling (Invalid Policy)

**Given**: Malformed policy file
**When**: Running `veil protect input.txt --policy bad.yaml`
**Then**:
- Error to stderr with YAML error details
- Exit code 1
- No output file created

## Edge Cases

### EC-1: Empty File

**Given**: A file with 0 bytes
**When**: Running `veil protect empty.txt -o output.txt`
**Then**:
- Output file is also empty
- Summary: "Protected empty.txt -> output.txt (0 redactions)"
- Exit code 0

### EC-2: Binary File

**Given**: A binary file
**When**: Running `veil protect binary.bin -o output.txt`
**Then**:
- Error to stderr: "Error: Cannot parse binary file: binary.bin"
- Exit code 1

### EC-3: Permission Denied (Read)

**Given**: Input file without read permissions
**When**: Running `veil protect protected.txt`
**Then**:
- Error to stderr: "Error: Permission denied: protected.txt"
- Exit code 1

### EC-4: Permission Denied (Write)

**Given**: Output directory without write permissions
**When**: Running `veil protect input.txt -o /readonly/output.txt`
**Then**:
- Error to stderr: "Error: Permission denied: /readonly/output.txt"
- Exit code 1

### EC-5: Large File

**Given**: Input file >100MB
**When**: Running `veil protect large.txt -o output.txt`
**Then**:
- File is processed (may take longer)
- Spinner or progress indication shown (unless `--quiet`)
- Exit code 0 if successful

## Validation Rules

### VR-1: Input Validation

- Input path must be valid UTF-8 (or displayed with lossy conversion)
- Input path `-` is treated as stdin
- Input file must exist (unless stdin)
- Input file must be readable

### VR-2: Output Validation

- Output path must be valid UTF-8
- Output directory must exist and be writable
- Output file must NOT exist (or `--force` flag required - future enhancement)

### VR-3: Style Validation

- Style must be one of: `label`, `bar`, `mask`
- Unknown styles result in error (exit code 1)

### VR-4: Policy Validation

- Policy file must exist
- Policy file must be valid YAML
- Policy must conform to schema

## Correctness Guarantees

### CG-1: Complete Redaction

**Guarantee**: All PII detected by the scan phase MUST be redacted in the output.

**Verification**:
- Scan the output with the same policy
- Result MUST have 0 findings

**Test**:
```rust
#[test]
fn test_complete_redaction() {
    let input = "Email: user@example.com";
    let output = protect(&input, RedactionStyle::Label);
    let findings = scan(&output, &policy);
    assert_eq!(findings.len(), 0, "Redacted output still contains PII");
}
```

### CG-2: No False Redactions

**Guarantee**: Text that is NOT PII MUST NOT be redacted.

**Verification**:
- Non-PII text remains unchanged
- Only matched findings are redacted

### CG-3: Preserves Non-PII Structure

**Guarantee**: Whitespace, punctuation, and non-PII text structure are preserved.

**Example**:
```
Input:  "Hello,\nEmail: user@example.com\nThank you."
Output: "Hello,\nEmail: [EMAIL]\nThank you."
```

## Performance Guarantees

| Metric | Target | Notes |
|--------|--------|-------|
| Files <1MB | <2 seconds | End-to-end (parse + detect + redact + write) |
| Files 1-10MB | <5 seconds | Sequential processing |
| Files >10MB | <1 minute | May show progress indication |

## Compatibility

| Platform | Support |
|----------|---------|
| Linux | ✅ Full support |
| macOS | ✅ Full support |
| Windows | ✅ Full support |

| Input Format | Support |
|--------------|---------|
| Plain text (.txt) | ✅ Full support |
| CSV (.csv) | ✅ Full support |
| JSON (.json) | ✅ Full support |
| HTML (.html, .htm) | ✅ Full support |
| PDF (.pdf) | Future (Spec 005) |
| Office (.docx, .xlsx) | Future (Spec 006) |

## Examples

### Example 1: Basic Protection (stdout)

```bash
$ cat input.txt
Contact: user@example.com
Phone: +1-555-0123

$ veil protect input.txt
Contact: [EMAIL]
Phone: [PHONE]
```

### Example 2: Protection to File

```bash
$ veil protect input.txt -o output.txt
Protected input.txt -> output.txt (2 redactions)

$ cat output.txt
Contact: [EMAIL]
Phone: [PHONE]
```

### Example 3: Bar Style

```bash
$ veil protect input.txt --style bar
Contact: ████████████████
Phone: ████████████
```

### Example 4: Mask Style

```bash
$ veil protect input.txt --style mask
Contact: u***@e******.com
Phone: +*-***-****
```

### Example 5: Protection with Policy

```bash
$ cat gdpr.yaml
name: GDPR Policy
version: 1.0
detection:
  - category: EMAIL
    confidence: 0.7
  - category: PHONE
    confidence: 0.7

$ veil protect input.txt --policy gdpr.yaml -o output.txt
Protected input.txt -> output.txt (2 redactions)
```

### Example 6: Stdin Input

```bash
$ echo "Email: user@example.com" | veil protect -
Email: [EMAIL]
```

### Example 7: JSON Metadata

```bash
$ veil protect input.txt -o output.txt --json
{
  "input": "input.txt",
  "output": "output.txt",
  "redaction_count": 2
}
```

## Test Implementation

```rust
#[test]
fn contract_protect_to_file() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("input.txt");
    let output = temp.path().join("output.txt");

    fs::write(&input, "Email: user@example.com").unwrap();

    let status = Command::new("veil")
        .args(&[
            "protect",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .status()
        .unwrap();

    assert!(status.success());

    let result = fs::read_to_string(&output).unwrap();
    assert!(result.contains("[EMAIL]"));
    assert!(!result.contains("user@example.com"));
}

#[test]
fn contract_protect_complete_redaction() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("input.txt");
    let output = temp.path().join("output.txt");

    fs::write(&input, "Email: user@example.com\nPhone: +1-555-0123").unwrap();

    // Protect
    Command::new("veil")
        .args(&[
            "protect",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .status()
        .unwrap();

    // Scan output - should find 0 findings
    let scan_output = Command::new("veil")
        .args(&["scan", output.to_str().unwrap(), "--json"])
        .output()
        .unwrap();

    let results: Vec<ScanResult> =
        serde_json::from_slice(&scan_output.stdout).unwrap();
    assert_eq!(results[0].findings_count, 0, "Protected file still contains PII");
}
```

## Security Considerations

### SEC-1: No PII in Logs

**Requirement**: PII MUST NOT appear in progress messages, error messages, or logs.

**Example**:
```
❌ Bad: "Redacting user@example.com at position 42"
✅ Good: "Redacting EMAIL at position 42"
```

### SEC-2: No Partial Redaction

**Requirement**: Each PII finding MUST be completely redacted. Partial redaction is a security violation.

**Example**:
```
❌ Bad: "user@*****.com" (domain still visible)
✅ Good: "[EMAIL]" or full mask
```

### SEC-3: Output File Permissions

**Requirement**: Output file MUST have same or stricter permissions as input file.

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-15 | Initial contract definition |
