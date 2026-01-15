# Quickstart: CLI Scan & Protect

**Feature**: 004-cli-scan-protect | **Date**: 2025-12-15 | **Phase**: 1

This guide helps developers quickly understand and work with the CLI Scan & Protect feature.

---

## Quick Command Reference

### Build and Run

```bash
# Build the CLI
cargo build --bin veil

# Run directly via cargo
cargo run --bin veil -- scan document.txt

# Or build and use the binary
cargo build --release
./target/release/veil scan document.txt
```

### Basic Operations

```bash
# Scan a single file
veil scan document.txt

# Scan a directory recursively
veil scan ./documents/ --recursive

# Get JSON output
veil scan document.txt --format json

# Protect (redact) a file
veil protect input.txt -o output.txt

# Use a custom policy
veil scan document.txt --policy gdpr.yaml

# Validate a policy file
veil policy validate gdpr.yaml
```

---

## 5-Minute Integration Guide

### 1. Add a New Command

To add a new subcommand to the CLI:

**Step 1**: Add to `cli.rs` enum:

```rust
// crates/veil-cli/src/cli.rs

#[derive(Subcommand)]
pub enum Commands {
    Scan(ScanArgs),
    Protect(ProtectArgs),
    Policy(PolicyArgs),
    YourCommand(YourArgs),  // Add here
}

#[derive(Parser, Debug)]
pub struct YourArgs {
    // Define arguments
}
```

**Step 2**: Create command handler:

```rust
// crates/veil-cli/src/commands/your_command.rs

use miette::Result;
use crate::cli::YourArgs;

pub fn run(args: YourArgs) -> Result<()> {
    // Implementation
    Ok(())
}
```

**Step 3**: Wire up in `main.rs`:

```rust
// crates/veil-cli/src/main.rs

match cli.command {
    Commands::Scan(args) => commands::scan::run(args, cli.quiet, cli.json),
    Commands::Protect(args) => commands::protect::run(args, cli.quiet, cli.json),
    Commands::Policy(args) => commands::policy::run(args),
    Commands::YourCommand(args) => commands::your_command::run(args),
}
```

### 2. Add Progress Indication

Use `indicatif` for progress bars:

```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(total as u64);
pb.set_style(ProgressStyle::default_bar()
    .template("{wide_bar} {pos}/{len} {eta} {msg}")
    .unwrap());

for item in items {
    pb.set_message(format!("Processing {}", item));
    // ... process item
    pb.inc(1);
}

pb.finish_with_message("Done");
```

### 3. Handle Errors with Miette

```rust
use miette::{Context, IntoDiagnostic, Result};

fn process_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read {}", path.display()))?;

    // ... process

    Ok(result)
}
```

### 4. Output JSON

```rust
use serde::Serialize;

#[derive(Serialize)]
struct Output {
    field: String,
}

fn output_json(data: &Output) -> Result<()> {
    let json = serde_json::to_string_pretty(data)
        .into_diagnostic()?;
    println!("{}", json);
    Ok(())
}
```

---

## Common Patterns

### Pattern 1: Process Multiple Files

```rust
let mut results = Vec::new();

for path in &args.paths {
    match process_file(path) {
        Ok(result) => results.push(result),
        Err(e) => {
            if !quiet {
                eprintln!("Warning: {}", e);
            }
            continue; // Keep going
        }
    }
}

// Output aggregated results
output_results(&results, json)?;
```

### Pattern 2: Conditional Progress

```rust
let progress = if quiet || json {
    None
} else {
    Some(ProgressBar::new(total as u64))
};

for item in items {
    if let Some(pb) = &progress {
        pb.set_message(format!("Processing {}", item));
        pb.inc(1);
    }
    // ... process
}

if let Some(pb) = progress {
    pb.finish();
}
```

### Pattern 3: Load Policy or Use Default

```rust
use veil_policy::{load_policy, default_policy};

let policy = match &args.policy {
    Some(path) => load_policy(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to load policy: {}", path.display()))?,
    None => default_policy(),
};

// Use policy...
```

### Pattern 4: Convert Library Types to CLI Types

```rust
use veil_detect::Finding;

fn to_cli_finding(finding: &Finding) -> FindingOutput {
    FindingOutput {
        category: finding.category.to_string(),
        text: finding.matched_text.clone(),
        position: format!("{}..{}", finding.start, finding.end),
        confidence: finding.confidence,
    }
}
```

---

## Testing Quick Reference

### Run All Tests

```bash
# Unit + integration tests
cargo test --package veil-cli

# Run specific test
cargo test --package veil-cli scan_single_file

# Run with output
cargo test --package veil-cli -- --nocapture
```

### Create Test Fixtures

```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn test_scan_directory() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("file1.txt");
    fs::write(&file1, "Contact: user@example.com").unwrap();

    // Run scan
    let result = scan_file(&file1, &registry, &policy, true, false).unwrap();

    assert_eq!(result.findings_count, 1);
    assert_eq!(result.findings[0].category, "EMAIL");
}
```

### Test CLI Arguments

```rust
use clap::Parser;
use crate::cli::{Cli, Commands};

#[test]
fn test_scan_args() {
    let cli = Cli::parse_from(&["veil", "scan", "file.txt", "--recursive"]);

    match cli.command {
        Commands::Scan(args) => {
            assert_eq!(args.paths.len(), 1);
            assert!(args.recursive);
        }
        _ => panic!("Expected Scan command"),
    }
}
```

---

## Debugging Tips

### 1. Enable Verbose Error Output

```bash
# See full error traces
RUST_BACKTRACE=1 cargo run --bin veil -- scan document.txt
```

### 2. Test JSON Output

```bash
# Validate JSON with jq
cargo run --bin veil -- scan document.txt --format json | jq .

# Pretty print
cargo run --bin veil -- scan document.txt --format json | jq '.'
```

### 3. Test Policy Loading

```bash
# Check policy validation
cargo run --bin veil -- policy validate policy.yaml

# See what detectors are enabled
cargo run --bin veil -- scan --policy policy.yaml document.txt
```

### 4. Isolate Command Handlers

```rust
// In tests, call handlers directly
use crate::commands::scan;

#[test]
fn test_scan_handler() {
    let args = ScanArgs {
        paths: vec![PathBuf::from("test.txt")],
        recursive: false,
        policy: None,
        detect: None,
        fail_on_findings: false,
    };

    let result = scan::run(args, true, false);
    assert!(result.is_ok());
}
```

---

## Code Organization

```
crates/veil-cli/
├── src/
│   ├── main.rs          # Entry point, minimal logic
│   ├── cli.rs           # Clap argument definitions
│   ├── output.rs        # Output formatting (text/JSON)
│   ├── progress.rs      # Progress bar utilities (NEW)
│   ├── error.rs         # CLI-specific error types (NEW)
│   ├── walker.rs        # Directory traversal (NEW)
│   └── commands/
│       ├── mod.rs       # Command module exports
│       ├── scan.rs      # Scan command logic
│       ├── protect.rs   # Protect command logic
│       └── policy.rs    # Policy validation command
│
tests/
├── integration/
│   ├── scan_tests.rs    # End-to-end scan tests
│   ├── protect_tests.rs # End-to-end protect tests
│   └── fixtures/        # Test files
│       ├── sample.txt
│       ├── nested/
│       └── policies/
└── contract/
    └── cli_contract.rs  # CLI behavior contracts
```

---

## Dependencies Flow

```
main.rs
  ↓
cli.rs (clap parsing)
  ↓
commands/{scan,protect,policy}.rs
  ↓ (uses)
  ├── walker.rs (file collection)
  ├── progress.rs (UI feedback)
  └── output.rs (formatting)
  ↓ (calls)
  ├── veil-parsers (file parsing)
  ├── veil-detect (PII detection)
  ├── veil-redact (redaction)
  └── veil-policy (policy engine)
```

---

## Common Gotchas

### 1. Exit Codes

Don't use `?` operator in `main()` for commands that need specific exit codes:

```rust
// ❌ Wrong
fn main() -> Result<()> {
    commands::scan::run(args)?;
    Ok(())
}

// ✅ Correct
fn main() -> Result<()> {
    let result = commands::scan::run(args, quiet, json);

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
```

### 2. JSON vs Text Mode

Suppress progress bars in JSON mode:

```rust
let show_progress = !quiet && !json;

if show_progress {
    let pb = ProgressBar::new(total);
    // ...
}
```

### 3. Segment Offsets

When protecting files, convert segment-relative offsets to absolute:

```rust
let base_offset = match &segment.position {
    Position::Text { byte_offset, .. } => *byte_offset,
    Position::Html { byte_offset, .. } => *byte_offset,
    _ => 0, // Fallback
};

let absolute_finding = Finding {
    start: base_offset + finding.start,
    end: base_offset + finding.end,
    ..finding
};
```

---

## Next Steps

After implementing this feature:

1. **Spec 011 (Audit Reporting)**: Integrate audit logging
2. **Spec 014 (Batch Processing)**: Add parallel processing with rayon
3. **Enhancement**: Add `--force` flag for overwriting files
4. **Enhancement**: Add `--format sarif` for SARIF output

---

## FAQ

**Q: How do I test the CLI manually?**

```bash
# Create test file
echo "Contact: user@example.com" > test.txt

# Run scan
cargo run --bin veil -- scan test.txt

# Run protect
cargo run --bin veil -- protect test.txt -o redacted.txt
cat redacted.txt
```

**Q: How do I add a new output format?**

1. Add enum variant to `OutputFormat` in `cli.rs`
2. Implement formatting logic in `output.rs`
3. Update `output_results()` match statement

**Q: How do I customize progress bar style?**

Edit `progress.rs`:

```rust
let style = ProgressStyle::default_bar()
    .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
    .unwrap()
    .progress_chars("█▓▒░  ");
```

**Q: How do I handle Windows vs Unix paths?**

Use `PathBuf` and `Path` from `std::path`. They handle platform differences automatically.

---

## References

- **Main Spec**: [spec.md](./spec.md)
- **Data Model**: [data-model.md](./data-model.md)
- **Research**: [research.md](./research.md)
- **Clap Docs**: https://docs.rs/clap/
- **Indicatif Docs**: https://docs.rs/indicatif/
- **Miette Docs**: https://docs.rs/miette/
