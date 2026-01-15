# Quickstart: PII Memory Zeroization

## Overview

This guide explains how to use the `SensitiveString` type for secure handling of PII data in Veil.

## Adding the Dependency

```toml
# Cargo.toml
[dependencies]
veil-core = { path = "../veil-core" }
```

## Basic Usage

### Creating a SensitiveString

```rust
use veil_core::SensitiveString;

// From &str
let sensitive = SensitiveString::new("secret-ssn-123-45-6789");

// From String
let owned = String::from("secret-data");
let sensitive = SensitiveString::from(owned);

// Empty
let empty = SensitiveString::empty();
```

### Using SensitiveString

```rust
use veil_core::SensitiveString;

let sensitive = SensitiveString::new("Hello, World!");

// Access as &str (via Deref)
println!("Length: {}", sensitive.len());
println!("Starts with: {}", sensitive.starts_with("Hello"));

// Pass to functions expecting &str
fn process(s: &str) { /* ... */ }
process(&sensitive);

// Display (shows content)
println!("{}", sensitive);  // "Hello, World!"

// Debug (redacted)
println!("{:?}", sensitive);  // SensitiveString([REDACTED 13 bytes])
```

### Automatic Cleanup

```rust
use veil_core::SensitiveString;

fn process_pii() {
    let pii = SensitiveString::new("123-45-6789");

    // ... use the PII data ...

    // When function returns, `pii` is dropped
    // and its contents are securely zeroed
}
```

## Integration Scenarios

### Scenario 1: Detecting PII

```rust
use veil_detect::{Finding, DetectorRegistry};
use veil_core::SensitiveString;

fn detect_and_process() {
    let registry = DetectorRegistry::default();
    let findings = registry.detect_all(&segments);

    for finding in &findings {
        // finding.matched_text is now SensitiveString
        process_finding(&finding.matched_text);
    }

    // When `findings` is dropped, all matched_text
    // values are securely zeroed
}
```

### Scenario 2: Parsing Documents

```rust
use veil_parsers::{parse_text, TextSegment};
use veil_core::SensitiveString;

fn parse_and_process(content: &str) {
    let result = parse_text(content)?;

    for segment in &result.segments {
        // segment.content is now SensitiveString
        analyze_segment(&segment.content);
    }

    // When `result` is dropped, all segment
    // content is securely zeroed
}
```

### Scenario 3: API Responses

```rust
use veil_core::SensitiveString;
use zeroize::Zeroize;

async fn scan_handler(body: Bytes) -> Response {
    // Process the scan
    let mut findings = scan_content(&body)?;

    // Build response
    let response = Json(&findings).into_response();

    // Explicit cleanup before returning
    // (findings will also be dropped and zeroed, but
    // this ensures cleanup happens before response returns)
    findings.zeroize();

    response
}
```

### Scenario 4: Streaming Processing

```rust
use veil_core::SensitiveString;
use veil_stream::StreamProcessor;

fn process_stream<R: Read>(reader: R) {
    let processor = StreamProcessor::new(reader);

    for chunk_result in processor {
        let chunk = chunk_result?;

        // Process chunk content (SensitiveString)
        detect_in_chunk(&chunk.content);

        // chunk is dropped here, content zeroed
    }

    // No PII remains in memory from any chunk
}
```

## Best Practices

### DO: Use SensitiveString for PII

```rust
// Good: PII is protected
struct UserRecord {
    id: u64,
    ssn: SensitiveString,      // Protected
    name: SensitiveString,      // Protected
    internal_id: String,        // Not PII, regular String OK
}
```

### DON'T: Convert to String unnecessarily

```rust
// Bad: Creates unprotected copy
let ssn: SensitiveString = /* ... */;
let unprotected: String = ssn.to_string();  // This copy is NOT zeroed!

// Good: Keep as SensitiveString
let ssn: SensitiveString = /* ... */;
use_ssn(&ssn);  // Pass reference, no copy
```

### DO: Use into_inner() only when necessary

```rust
// Only when you MUST have String ownership and will handle cleanup
let sensitive = SensitiveString::new("data");
let owned: String = sensitive.into_inner();
// WARNING: `owned` will NOT be automatically zeroed!
// You are now responsible for cleanup
```

### DO: Implement Zeroize for custom types

```rust
use zeroize::Zeroize;
use veil_core::SensitiveString;

struct CustomResult {
    pii_value: SensitiveString,
    metadata: String,
}

impl Zeroize for CustomResult {
    fn zeroize(&mut self) {
        // SensitiveString already zeroizes on drop, but
        // explicit call is safe and idempotent
        self.pii_value = SensitiveString::empty();
    }
}
```

## Verification

### Testing Zeroization

```rust
#[test]
fn test_zeroization() {
    let ptr: *const u8;
    let len: usize;

    {
        let sensitive = SensitiveString::new("secret");
        ptr = sensitive.as_ptr();
        len = sensitive.len();
    }
    // `sensitive` dropped here, memory zeroed

    // Verify memory is zeroed (unsafe, for testing only)
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        assert!(slice.iter().all(|&b| b == 0));
    }
}
```

## Limitations

1. **Debug builds**: Memory may not be fully zeroed in debug builds due to different optimization levels
2. **Swap space**: If memory is swapped to disk, the OS may retain copies
3. **WASM**: Browser memory model differs; zeroization is best-effort
4. **Clone**: Each clone is a new allocation that must be independently dropped

## Related Documentation

- [research.md](research.md) - Technical research on memory zeroization
- [data-model.md](data-model.md) - Full API specification
- [contracts/zeroization-tests.md](contracts/zeroization-tests.md) - Test cases
