# Data Model: CLI Scan & Protect

**Feature**: 004-cli-scan-protect | **Date**: 2025-12-15 | **Phase**: 1

## Entity Definitions

### 1. ScanOptions

Configuration for the scan operation, derived from CLI arguments.

```rust
pub struct ScanOptions {
    /// Files or directories to scan
    pub paths: Vec<PathBuf>,

    /// Whether to scan directories recursively
    pub recursive: bool,

    /// Optional policy file path
    pub policy: Option<PathBuf>,

    /// Optional detector filter (e.g., ["email", "phone"])
    pub detect: Option<Vec<String>>,

    /// Exit with code 2 if findings are detected
    pub fail_on_findings: bool,

    /// Suppress progress output
    pub quiet: bool,

    /// Output in JSON format
    pub json: bool,
}
```

**Invariants**:
- `paths` must not be empty (enforced by clap's `required = true`)
- If `recursive` is false and a path is a directory, only top-level files are scanned
- If `detect` is Some, it filters detectors; if None, all detectors are used

**Lifecycle**: Created from `ScanArgs` + global flags in `commands::scan::run()`.

---

### 2. ProtectOptions

Configuration for the protect operation, derived from CLI arguments.

```rust
pub struct ProtectOptions {
    /// Input file path (or "-" for stdin)
    pub input: PathBuf,

    /// Optional output file path (None = stdout)
    pub output: Option<PathBuf>,

    /// Optional policy file path
    pub policy: Option<PathBuf>,

    /// Redaction style (label, bar, mask)
    pub style: String,

    /// Suppress progress output
    pub quiet: bool,

    /// Output metadata in JSON format
    pub json: bool,
}
```

**Invariants**:
- `input` must exist (or be "-" for stdin)
- `style` must be one of: "label", "bar", "mask" (validated in command handler)
- If `output` is None, result goes to stdout
- If `json` is true, JSON metadata goes to stderr (to avoid mixing with stdout content)

**Lifecycle**: Created from `ProtectArgs` + global flags in `commands::protect::run()`.

---

### 3. ScanResult

Output of a single file scan operation.

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanResult {
    /// Path of the scanned file
    pub file: String,

    /// Number of findings detected
    pub findings_count: usize,

    /// List of findings (empty if none detected)
    pub findings: Vec<FindingOutput>,
}
```

**Invariants**:
- `findings_count == findings.len()` (always consistent)
- `file` is the display form of the path (may be relative or absolute)

**Lifecycle**: Created per file in `scan_file()`, aggregated in `run()`, output as JSON array or text.

---

### 4. FindingOutput

Simplified finding representation for CLI output.

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct FindingOutput {
    /// PII category (e.g., "EMAIL", "PHONE", "IBAN")
    pub category: String,

    /// Matched text (the actual PII)
    pub text: String,

    /// Position in the file (e.g., "42..58")
    pub position: String,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}
```

**Invariants**:
- `category` is a valid detector category (enforced by `veil-detect`)
- `confidence` is in range [0.0, 1.0]
- `position` is formatted as "start..end" (byte offsets)

**Lifecycle**: Converted from `veil_detect::Finding` in `scan_file()`.

---

### 5. ProtectResult

Result of a protect operation (internal, not directly exposed).

```rust
struct ProtectResult {
    /// Redacted text content
    pub redacted_text: String,

    /// Number of redactions applied
    pub redaction_count: usize,
}
```

**Invariants**:
- `redacted_text` contains no PII from the original findings
- `redaction_count` matches the number of findings processed

**Lifecycle**: Created in `protect_file()`, written to output destination.

---

### 6. ProtectOutput (JSON metadata)

JSON output for protect operations.

```rust
#[derive(serde::Serialize)]
struct ProtectOutput {
    /// Input file path
    pub input: String,

    /// Output file path (None if stdout)
    pub output: Option<String>,

    /// Number of redactions applied
    pub redaction_count: usize,
}
```

**Invariants**:
- Only used when `--json` flag is set
- Output to stderr to avoid mixing with stdout content

**Lifecycle**: Created in `commands::protect::run()` after successful protection.

---

### 7. ProgressContext

Internal state for progress indication.

```rust
pub struct ProgressContext {
    /// Progress bar or spinner (hidden if quiet/json mode)
    pub bar: Option<ProgressBar>,

    /// Total number of files to process
    pub total_files: usize,

    /// Number of files processed
    pub processed_files: usize,
}
```

**Invariants**:
- `bar` is None if quiet or json mode
- `processed_files <= total_files`

**Lifecycle**: Created in `commands::scan::run()` or `commands::protect::run()`, updated per file.

---

## State Transitions

### Scan Command Flow

```
[CLI Args]
   ↓
[ScanOptions]
   ↓
[Load Policy] → [Policy] or [Default Policy]
   ↓
[Create ProgressContext]
   ↓
FOR EACH path:
   IF directory AND recursive:
      [Walk Directory] → [Vec<PathBuf>]
   ELSE:
      [Single File]
   ↓
   [Parse File] → [ParseResult]
   ↓
   [Detect PII] → [Vec<Finding>]
   ↓
   [Apply Policy] → [Filtered Vec<Finding>]
   ↓
   [Convert to FindingOutput] → [ScanResult]
   ↓
   [Update Progress]
   ↓
[Aggregate Results]
   ↓
[Output: JSON or Text]
   ↓
[Exit: 0 (no findings), 1 (error), or 2 (findings + --fail-on-findings)]
```

### Protect Command Flow

```
[CLI Args]
   ↓
[ProtectOptions]
   ↓
[Load Policy] → [Policy] or [Default Policy]
   ↓
[Read Input] → [String content]
   ↓
[Parse File] → [ParseResult]
   ↓
[Detect PII] → [Vec<Finding>]
   ↓
[Apply Policy] → [Filtered Vec<Finding>]
   ↓
[Convert to Absolute Offsets] → [Vec<Finding>]
   ↓
[Apply Redaction] → [ProtectResult]
   ↓
IF output is Some:
   [Write to File]
ELSE:
   [Write to Stdout]
   ↓
IF json:
   [Output JSON metadata to stderr]
   ↓
[Exit: 0 (success) or 1 (error)]
```

### Policy Validate Command Flow

```
[CLI Args: policy file path]
   ↓
[Load Policy]
   ↓
IF error:
   [Print error to stderr]
   [Exit: 1]
ELSE:
   [Print policy summary]
   [Exit: 0]
```

---

## Data Flow Diagram

```
┌─────────────┐
│  User Input │
│  (CLI Args) │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│  Command Parser │ (clap)
│  - scan         │
│  - protect      │
│  - policy       │
└──────┬──────────┘
       │
       ▼
┌─────────────────────────────────────────────┐
│  Command Handlers                           │
│  ┌───────────┐  ┌─────────────┐  ┌────────┐│
│  │ scan::run │  │protect::run │  │policy::│││
│  │           │  │             │  │run     │││
│  └─────┬─────┘  └──────┬──────┘  └───┬────┘│
└────────┼────────────────┼──────────────┼─────┘
         │                │              │
         ▼                ▼              ▼
    ┌─────────┐      ┌─────────┐   ┌──────────┐
    │ Walker  │      │ Parser  │   │ Policy   │
    │ (files) │      │(content)│   │ Loader   │
    └────┬────┘      └────┬────┘   └────┬─────┘
         │                │              │
         ▼                ▼              ▼
    ┌──────────────────────────────────────┐
    │  Core Libraries                      │
    │  ┌──────────┐  ┌────────┐  ┌───────┐│
    │  │ veil-    │→ │ veil-  │→ │ veil- ││
    │  │ parsers  │  │ detect │  │redact ││
    │  └──────────┘  └────────┘  └───────┘│
    │  ┌──────────┐  ┌─────────┐          │
    │  │ veil-    │  │ veil-   │          │
    │  │ policy   │  │ audit   │          │
    │  └──────────┘  └─────────┘          │
    └──────────────────────────────────────┘
         │
         ▼
    ┌─────────────────┐
    │  Output         │
    │  - Text (stdout)│
    │  - JSON (stdout)│
    │  - Errors(stderr│
    └─────────────────┘
```

---

## Validation Rules

### Input Validation

1. **File Paths**:
   - Must be valid UTF-8 (or display with lossy conversion)
   - Checked for existence before processing
   - Permission errors handled gracefully

2. **Policy Files**:
   - Must be valid YAML (checked by `serde_yaml`)
   - Must conform to policy schema (checked by `veil-policy`)
   - Errors include line numbers and descriptive messages

3. **Detector Filter**:
   - If `--detect` is provided, validate against known detector names
   - Unknown detectors → warning, not fatal error
   - Empty list → error (must specify at least one detector)

4. **Redaction Style**:
   - Must be one of: "label", "bar", "mask"
   - Unknown style → error with list of valid options

### Output Validation

1. **JSON Output**:
   - Must be valid JSON (guaranteed by `serde_json`)
   - Array of `ScanResult` for scan command
   - Single `ProtectOutput` object for protect command

2. **Exit Codes**:
   - 0: Success
   - 1: Error (invalid input, processing failure)
   - 2: Success but findings detected (only with `--fail-on-findings`)

---

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("Invalid policy file: {0}")]
    InvalidPolicy(#[from] veil_policy::PolicyError),

    #[error("Parse error: {0}")]
    ParseError(#[from] veil_parsers::ParseError),

    #[error("Unknown detector: {0}")]
    UnknownDetector(String),

    #[error("Invalid redaction style: {0}")]
    InvalidStyle(String),

    #[error("Output file exists: {0} (use --force to overwrite)")]
    OutputExists(PathBuf),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Error Conversion to Exit Codes

```rust
fn exit_code(result: Result<(), CliError>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {}", e);
            1
        }
    }
}
```

---

## Dependencies Between Entities

```
ScanOptions → ScanResult (produces)
ScanOptions → Policy (uses)
ScanResult → FindingOutput (contains)

ProtectOptions → ProtectResult (produces)
ProtectOptions → Policy (uses)
ProtectResult → ProtectOutput (converts to)

ProgressContext → ScanOptions (configured by)
ProgressContext → ProtectOptions (configured by)

Policy → veil_policy::Policy (wraps)
```

---

## Key Algorithms

### Directory Walking

```rust
fn collect_files(path: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && is_supported(&path) {
                files.push(path);
            } else if path.is_dir() && recursive {
                files.extend(collect_files(&path, recursive)?);
            }
        }
    }

    Ok(files)
}
```

### Finding Position Calculation

```rust
fn absolute_finding(finding: &Finding, segment: &Segment) -> Finding {
    let base_offset = match &segment.position {
        Position::Text { byte_offset, .. } => *byte_offset,
        Position::Html { byte_offset, .. } => *byte_offset,
        _ => 0, // Fallback for CSV/JSON
    };

    Finding {
        start: base_offset + finding.start,
        end: base_offset + finding.end,
        ..finding.clone()
    }
}
```

---

## Testing Strategy

### Unit Tests

- `ScanOptions` / `ProtectOptions` construction from CLI args
- Error type conversions
- Position calculation for different segment types

### Integration Tests

- Scan single file with known PII
- Scan directory recursively
- Protect file with different styles
- Policy validation
- Exit code verification

### Contract Tests

- CLI arguments parsing
- Output format stability (JSON schema)
- Error message format

---

## References

- Spec: `specs/004-cli-scan-protect/spec.md`
- Research: `specs/004-cli-scan-protect/research.md`
- Related Types:
  - `veil_detect::Finding`
  - `veil_parsers::ParseResult`
  - `veil_policy::Policy`
  - `veil_redact::RedactionResult`
