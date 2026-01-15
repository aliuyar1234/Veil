# Contract: Scan Command

**Feature**: 004-cli-scan-protect | **Command**: `veil scan`

## Command Signature

```bash
veil scan [OPTIONS] <PATHS>...
```

## Required Arguments

| Argument | Type | Description |
|----------|------|-------------|
| `PATHS` | `Vec<PathBuf>` | One or more file or directory paths to scan |

## Optional Flags

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--recursive` | `-r` | bool | false | Scan directories recursively |
| `--policy` | `-p` | PathBuf | None | Path to YAML policy file |
| `--detect` | | Vec<String> | None | Comma-separated list of detector categories |
| `--fail-on-findings` | | bool | false | Exit with code 2 if PII is found |
| `--quiet` | `-q` | bool | false | Suppress progress output |
| `--json` | | bool | false | Output results in JSON format |

## Output Contract

### Text Mode (default)

```
Scanning: /path/to/file.txt

File: /path/to/file.txt
Findings: 2

  [EMAIL] user@example.com (42..58) confidence: 0.95
  [PHONE] +1-555-0123 (120..132) confidence: 0.90

Total: 2 findings in 1 files
```

**Rules**:
- Progress messages go to stderr
- Results go to stdout
- One blank line between files
- Findings indented with 2 spaces
- Position format: `start..end` (byte offsets)
- Confidence displayed as 0.00-1.00

### JSON Mode (`--json`)

```json
[
  {
    "file": "/path/to/file.txt",
    "findings_count": 2,
    "findings": [
      {
        "category": "EMAIL",
        "text": "user@example.com",
        "position": "42..58",
        "confidence": 0.95
      },
      {
        "category": "PHONE",
        "text": "+1-555-0123",
        "position": "120..132",
        "confidence": 0.90
      }
    ]
  }
]
```

**Rules**:
- Always a JSON array (even for single file)
- No progress output to avoid corrupting JSON
- All output to stdout
- Errors to stderr

## Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Success, no findings (or findings without `--fail-on-findings`) |
| 1 | Error (invalid arguments, file not found, parse error) |
| 2 | Success but findings detected (only with `--fail-on-findings`) |

## Behavioral Contracts

### BC-1: Single File Scan

**Given**: A single file path
**When**: Running `veil scan file.txt`
**Then**:
- File is parsed
- All detectors are run (unless `--detect` is specified)
- Findings are output
- Exit code 0 if successful

### BC-2: Directory Scan (Non-Recursive)

**Given**: A directory path without `--recursive`
**When**: Running `veil scan ./dir`
**Then**:
- Only top-level files are scanned
- Subdirectories are skipped with warning (unless `--quiet`)
- Exit code 0 if successful

### BC-3: Directory Scan (Recursive)

**Given**: A directory path with `--recursive`
**When**: Running `veil scan ./dir --recursive`
**Then**:
- All files in directory tree are scanned
- Progress bar shows file count and current file
- Unsupported file types are skipped silently
- Exit code 0 if successful

### BC-4: Policy Application

**Given**: A policy file with confidence threshold 0.8
**When**: Running `veil scan file.txt --policy policy.yaml`
**Then**:
- Policy is loaded and validated
- Findings below threshold 0.8 are filtered out
- Only filtered findings are output
- Exit code 0 if successful

### BC-5: Detector Filter

**Given**: `--detect email,phone`
**When**: Running `veil scan file.txt --detect email,phone`
**Then**:
- Only EMAIL and PHONE detectors run
- Other PII types (IBAN, SSN, etc.) are not detected
- Exit code 0 if successful

### BC-6: Fail on Findings

**Given**: A file with PII
**When**: Running `veil scan file.txt --fail-on-findings`
**Then**:
- Findings are output normally
- Exit code 2 (not 0)

### BC-7: Stdin Input

**Given**: Content piped to stdin
**When**: Running `echo "email: user@example.com" | veil scan -`
**Then**:
- Content is read from stdin
- Parsed as plain text
- Findings are output
- Exit code 0 if successful

### BC-8: Error Handling (File Not Found)

**Given**: A non-existent file path
**When**: Running `veil scan nonexistent.txt`
**Then**:
- Error message to stderr: "Error: File not found: nonexistent.txt"
- Exit code 1

### BC-9: Error Handling (Invalid Policy)

**Given**: A malformed YAML policy file
**When**: Running `veil scan file.txt --policy bad.yaml`
**Then**:
- Error message to stderr with YAML error details
- Exit code 1 (no scanning occurs)

### BC-10: Mixed File Types

**Given**: Directory with .txt, .csv, .bin files
**When**: Running `veil scan ./dir --recursive`
**Then**:
- Supported files (.txt, .csv) are scanned
- Unsupported files (.bin) are skipped
- No errors for unsupported types
- Exit code 0

## Edge Cases

### EC-1: Empty File

**Given**: A file with 0 bytes
**When**: Running `veil scan empty.txt`
**Then**:
- Output shows 0 findings
- Exit code 0

### EC-2: No PII in File

**Given**: A file with no detectable PII
**When**: Running `veil scan clean.txt`
**Then**:
- Output shows 0 findings
- Message: "No PII detected"
- Exit code 0

### EC-3: All Findings Filtered by Policy

**Given**: File with low-confidence findings and strict policy
**When**: Running `veil scan file.txt --policy strict.yaml`
**Then**:
- All findings filtered out
- Output shows 0 findings
- Exit code 0

### EC-4: Binary File

**Given**: A binary file with .txt extension
**When**: Running `veil scan fake.txt`
**Then**:
- Warning to stderr: "Skipping binary file: fake.txt"
- Exit code 0 (continue if batch, or just exit if single file)

### EC-5: Permission Denied

**Given**: A file without read permissions
**When**: Running `veil scan protected.txt`
**Then**:
- Error to stderr: "Permission denied: protected.txt"
- Exit code 1 (single file) or continue (batch)

## Validation Rules

### VR-1: Path Validation

- Paths must be valid UTF-8 (or displayed with lossy conversion)
- Paths are resolved relative to current working directory
- `~` expansion is NOT performed (use shell expansion)

### VR-2: Detector Name Validation

- If `--detect` includes unknown detector name:
  - Warning to stderr: "Unknown detector: xyz"
  - Continue with known detectors
- If `--detect` is empty list: error

### VR-3: Policy File Validation

- Policy file must exist
- Policy file must be valid YAML
- Policy must conform to schema (checked by veil-policy)

## Performance Guarantees

| Metric | Target | Notes |
|--------|--------|-------|
| Single file <1MB | <2 seconds | End-to-end (parse + detect + output) |
| 1000 files | <5 minutes | Recursive scan with progress |
| Progress update rate | 10 Hz | Smooth progress bar updates |

## Compatibility

| Platform | Support |
|----------|---------|
| Linux | ✅ Full support |
| macOS | ✅ Full support |
| Windows | ✅ Full support |

| Shell | Support |
|-------|---------|
| bash | ✅ Full support |
| zsh | ✅ Full support |
| fish | ✅ Full support |
| PowerShell | ✅ Full support |
| cmd.exe | ✅ Full support |

## Examples

### Example 1: Basic Scan

```bash
$ veil scan document.txt
Scanning: document.txt

File: document.txt
Findings: 1

  [EMAIL] user@example.com (42..58) confidence: 0.95

Total: 1 findings in 1 files
```

### Example 2: Recursive Scan with JSON

```bash
$ veil scan ./documents --recursive --json
[
  {
    "file": "./documents/file1.txt",
    "findings_count": 2,
    "findings": [...]
  },
  {
    "file": "./documents/subdir/file2.csv",
    "findings_count": 0,
    "findings": []
  }
]
```

### Example 3: Scan with Policy

```bash
$ veil scan document.txt --policy gdpr.yaml
Scanning: document.txt

File: document.txt
Findings: 1

  [IBAN] DE89370400440532013000 (10..32) confidence: 1.00

Total: 1 findings in 1 files
```

### Example 4: Fail on Findings

```bash
$ veil scan document.txt --fail-on-findings
Scanning: document.txt

File: document.txt
Findings: 1

  [EMAIL] user@example.com (42..58) confidence: 0.95

Total: 1 findings in 1 files
$ echo $?
2
```

## Test Implementation

```rust
#[test]
fn contract_scan_single_file() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.txt");
    fs::write(&file, "Email: user@example.com").unwrap();

    let output = Command::new("veil")
        .args(&["scan", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("EMAIL"));
    assert!(stdout.contains("user@example.com"));
}

#[test]
fn contract_scan_fail_on_findings() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.txt");
    fs::write(&file, "Email: user@example.com").unwrap();

    let output = Command::new("veil")
        .args(&["scan", file.to_str().unwrap(), "--fail-on-findings"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
}
```

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-15 | Initial contract definition |
