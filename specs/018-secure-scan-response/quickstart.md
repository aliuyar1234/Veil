# Quickstart: Secure Scan Response Implementation

## Overview

This feature removes PII values from scan responses by default across all interfaces (API, CLI, WASM).

## Key Files to Modify

### 1. API (veil-api)

**crates/veil-api/src/routes/scan.rs**:
- Line 78: Change `value: detection.matched_text.clone()` to conditional
- Add header extraction and validation

**crates/veil-api/src/models.rs** (create if doesn't exist, or find in lib.rs):
- Add `include_values: bool` to `ScanOptions`
- Change `value: String` to `value: Option<String>` in `Finding`
- Add `#[serde(skip_serializing_if = "Option::is_none")]`

### 2. CLI (veil-cli)

**crates/veil-cli/src/cli.rs**:
- Line ~47: Add `--include-values` flag to `ScanArgs`

**crates/veil-cli/src/commands/scan.rs**:
- Line 138: Change `text: f.matched_text.clone()` to conditional
- Add confirmation prompt logic

### 3. WASM (veil-wasm)

**crates/veil-wasm/src/types.rs**:
- Line 15: Change `pub value: String` to `pub value: Option<String>`
- Add `includeValues` and `acknowledgeExposure` to `ScanOptions`

**crates/veil-wasm/src/scan.rs**:
- Line 139: Change to conditional based on options

## Implementation Steps

### Step 1: Modify API Response Model
```rust
// In Finding struct
#[serde(skip_serializing_if = "Option::is_none")]
pub value: Option<String>,
```

### Step 2: Add ScanOptions Field
```rust
// In ScanOptions
#[serde(default)]
pub include_values: bool,
```

### Step 3: Validate Header in Handler
```rust
// In scan_file handler
if options.include_values {
    let header = headers.get("x-acknowledge-pii-exposure");
    if header.map(|v| v.to_str().ok()) != Some(Some("accepted")) {
        return Err(ApiError::BadRequest(
            "include_values requires X-Acknowledge-PII-Exposure: accepted header"
        ));
    }
}
```

### Step 4: Conditionally Include Value
```rust
// When building Finding
let value = if include_values {
    Some(detection.matched_text.clone())
} else {
    None
};
```

## Test Commands

```bash
# Run all tests
cargo test --workspace

# Test API specifically
cargo test -p veil-api

# Test CLI
cargo test -p veil-cli

# Manual API test (should NOT include values)
curl -X POST http://localhost:3000/api/v1/scan -F file=@test.txt | jq .

# Manual API test (should include values)
curl -X POST "http://localhost:3000/api/v1/scan?include_values=true" \
  -H "X-Acknowledge-PII-Exposure: accepted" \
  -F file=@test.txt | jq .

# Manual CLI test (should NOT include values)
cargo run -p veil-cli -- scan test.txt --json

# Manual CLI test (should prompt for confirmation)
cargo run -p veil-cli -- scan test.txt --include-values
```

## Verification Checklist

- [ ] API response omits `value` field by default
- [ ] API returns 400 if include_values=true without header
- [ ] API includes values when header is present
- [ ] CLI output omits values by default
- [ ] CLI prompts for confirmation with --include-values
- [ ] WASM scan omits values by default
- [ ] WASM validates acknowledgeExposure with includeValues
- [ ] All existing tests pass
- [ ] New tests cover all acceptance scenarios
