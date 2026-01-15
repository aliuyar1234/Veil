# Research: CLI Scan & Protect

**Feature**: 004-cli-scan-protect | **Date**: 2025-12-15 | **Phase**: 0

## Research Questions

### Q1: How should progress indication work for different operation scales?

**Finding**: The `indicatif` crate is already in the workspace and provides:
- `ProgressBar` for multi-file operations with percentage, ETA, throughput
- `Spinner` for single-file operations or indeterminate progress
- `MultiProgress` for concurrent operations (out of scope for this feature)

**Decision**:
- Single file: Use `Spinner` with file name
- Multiple files (2+): Use `ProgressBar` with `{pos}/{len}` template and ETA
- Quiet mode (`--quiet`): Suppress all progress output
- JSON mode (`--json`): Suppress progress to avoid corrupting JSON output

**Code pattern**:
```rust
if !quiet && !json {
    if paths.len() == 1 {
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Scanning...");
        // ... process
        spinner.finish_and_clear();
    } else {
        let pb = ProgressBar::new(paths.len() as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{wide_bar} {pos}/{len} {msg}"));
        for path in paths {
            pb.set_message(path.display().to_string());
            // ... process
            pb.inc(1);
        }
        pb.finish_with_message("Done");
    }
}
```

### Q2: How should directory traversal handle errors and unsupported files?

**Finding**: Rust's `std::fs::read_dir` returns `Result<ReadDir>`, and each entry is `Result<DirEntry>`. Permission errors and symlinks require careful handling.

**Decision**:
- Continue on permission errors, log warning to stderr
- Skip unsupported file extensions silently (check against whitelist)
- Follow symlinks by default (no special handling required)
- Binary file detection: Check extension first; if text extension but binary content detected during parse, handle `ParseError::BinaryContent`

**Error handling pattern**:
```rust
for entry in fs::read_dir(dir)? {
    match entry {
        Ok(entry) => {
            let path = entry.path();
            if let Err(e) = scan_file(&path) {
                eprintln!("Warning: Failed to scan {}: {}", path.display(), e);
                continue; // Keep going
            }
        }
        Err(e) => {
            eprintln!("Warning: Directory entry error: {}", e);
            continue;
        }
    }
}
```

### Q3: How should JSON output be structured for scan results?

**Finding**: The existing `scan.rs` already has `ScanResult` and `FindingOutput` structs with `#[derive(serde::Serialize)]`. JSON schema should be:

```json
[
  {
    "file": "/path/to/file.txt",
    "findings_count": 3,
    "findings": [
      {
        "category": "EMAIL",
        "text": "user@example.com",
        "position": "42..58",
        "confidence": 0.95
      }
    ]
  }
]
```

**Decision**: Keep existing structure. For protect command, output:
```json
{
  "input": "/path/to/input.txt",
  "output": "/path/to/output.txt",
  "redaction_count": 5
}
```

### Q4: How should policy file loading integrate with CLI arguments?

**Finding**: The `veil-policy` crate already provides:
- `load_policy(path: &Path) -> Result<Policy, PolicyError>`
- `default_policy() -> Policy`
- `apply_policy_to_findings(policy: &Policy, findings: Vec<Finding>) -> Vec<Finding>`

**Decision**:
- CLI flag `--policy <PATH>` overrides default policy
- No flag = use `default_policy()` (all detectors, confidence 0.5, redact with labels)
- Policy loading errors should use `miette` to show user-friendly error with file location
- Policy validation happens on load; invalid policy = immediate exit with error code 1

**Integration pattern**:
```rust
let policy = match &args.policy {
    Some(path) => load_policy(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to load policy: {}", path.display()))?,
    None => default_policy(),
};
```

### Q5: How should output file conflicts be handled?

**Finding**: User Story 8 edge case specifies: "System prompts for confirmation unless `--force` flag used."

**Decision**:
- Phase 1 (this feature): If output file exists, error and suggest `--force`
- Phase 2 (future): Add `--force` flag for overwrite without prompt
- Rationale: Interactive prompts are complex in CLI (requires stdin handling); error-first is simpler and safer

**Implementation**:
```rust
if output_path.exists() && !args.force {
    return Err(miette::miette!(
        "Output file already exists: {}\nUse --force to overwrite",
        output_path.display()
    ));
}
```

### Q6: How should stdin input be handled?

**Finding**: Requirement FR-012 specifies reading from stdin when filename is `-`.

**Decision**:
- `veil scan -` reads from stdin, outputs to stdout (text) or stderr (JSON)
- `veil protect -` reads from stdin, outputs to stdout (redacted text)
- Progress bars are suppressed when reading from stdin (no file count)

**Implementation pattern**:
```rust
let content = if args.input.to_str() == Some("-") {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    buffer
} else {
    std::fs::read_to_string(&args.input)?
};
```

### Q7: What exit codes should the CLI return?

**Finding**: Requirement FR-011 specifies:
- 0: Success (no PII found, or operation completed successfully)
- 1: Error (invalid arguments, file not found, parse error, etc.)
- 2: PII found in scan with `--fail-on-findings` flag

**Decision**: Implement as specified. Use `std::process::exit(code)` for non-zero codes.

### Q8: How should file format detection work?

**Finding**: The `veil-parsers` crate already provides automatic format detection via `parse_file()` using extension and content sniffing.

**Decision**: Delegate to `veil-parsers::parse_file()`. CLI only needs to handle `ParseError` cases:
- `ParseError::UnsupportedFormat`: Skip with warning
- `ParseError::BinaryContent`: Skip with warning
- Other errors: Report to stderr and continue (for batch scans)

## Technical Dependencies

| Dependency | Version | Purpose | Already in Workspace |
|------------|---------|---------|---------------------|
| clap | 4.4 | CLI argument parsing | ✅ Yes |
| indicatif | 0.17 | Progress bars | ✅ Yes |
| console | 0.15 | Terminal styling | ✅ Yes |
| miette | 7.0 | User-friendly errors | ✅ Yes |
| serde_json | 1.0 | JSON output | ✅ Yes |
| veil-parsers | 0.1.0 | File parsing | ✅ Internal |
| veil-detect | 0.1.0 | PII detection | ✅ Internal |
| veil-redact | 0.1.0 | Redaction | ✅ Internal |
| veil-policy | 0.1.0 | Policy engine | ✅ Internal |
| veil-audit | 0.1.0 | Audit logging | ✅ Internal |

**No new external dependencies required.**

## Performance Considerations

### Baseline Performance (from Success Criteria)

- **SC-001**: Single file <1MB should complete in <2 seconds
  - **Target**: 500KB/s parsing throughput
  - **Strategy**: Sequential processing is sufficient; no parallelism needed

- **SC-002**: 1000 files should complete in <5 minutes
  - **Target**: ~300ms per file average
  - **Strategy**: Progress bar updates every file (not every finding)

### Optimization Opportunities (Out of Scope for Phase 1)

- Parallel file processing with `rayon` (future: Spec 014 Batch Processing)
- Incremental progress updates for large single files
- Memory-mapped file reading for very large files (>100MB)

## Error Handling Strategy

### Error Categories

1. **User Input Errors** (exit code 1)
   - Invalid arguments: Let clap handle (automatic error messages)
   - File not found: miette with helpful message
   - Invalid policy file: miette with YAML line number

2. **Processing Errors** (exit code 1, but continue batch)
   - Parse errors: Log to stderr, skip file, continue
   - Permission errors: Log to stderr, skip file, continue
   - Output write errors: Fatal (stop immediately)

3. **Success with Findings** (exit code 2 if `--fail-on-findings`)
   - Scan completed successfully, PII detected

### Error Message Format

Use `miette` for all user-facing errors:
```rust
use miette::{Context, IntoDiagnostic, Result};

fn load_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read file: {}", path.display()))
}
```

## Security Considerations

### No Security Concerns for This Feature

- CLI does not store sensitive data persistently
- Audit logging is handled by `veil-audit` crate (separate feature)
- Policy files are parsed with `serde_yaml` (safe, no eval)
- File I/O uses standard library (no unsafe)

## Open Questions

None. All questions resolved during research phase.

## References

- Constitution: `.specify/memory/constitution.md`
- Spec: `specs/004-cli-scan-protect/spec.md`
- Existing CLI implementation: `crates/veil-cli/src/`
- Related crates:
  - `veil-parsers` (Spec 001)
  - `veil-detect` (Spec 002)
  - `veil-redact` (Spec 003)
  - `veil-policy` (Spec 009)
  - `veil-audit` (Spec 011)
