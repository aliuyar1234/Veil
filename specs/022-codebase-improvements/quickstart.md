# Quickstart: Codebase Excellence Initiative

**Branch**: 022-codebase-improvements | **Date**: 2025-12-18

## Overview

This guide helps developers get started with the codebase improvements. The initiative spans 9 categories with 50 functional requirements.

## Prerequisites

- Rust 1.75+ (stable)
- cargo-fuzz (for fuzzing)
- cargo-mutants (for mutation testing)
- cargo-llvm-cov (for coverage)
- wasm-pack (for WASM testing)
- pre-commit (for git hooks)
- just (for task runner)

## Installation

```bash
# Install development tools
cargo install cargo-fuzz cargo-mutants cargo-llvm-cov
cargo install wasm-pack
pip install pre-commit
cargo install just

# Clone and setup
git checkout 022-codebase-improvements
pre-commit install
```

## Key Commands

```bash
# Run all tests
just test

# Run with coverage
just coverage

# Run mutation testing (CI only - slow)
just mutants

# Run fuzzing (nightly required)
just fuzz

# Check formatting and linting
just check

# Build documentation
just docs

# Run benchmarks
just bench
```

## New Crates

### veil-types

Shared type definitions used across all crates.

```rust
use veil_types::{Finding, PiiCategory, Position};
```

### veil-plugin

Plugin loading infrastructure for custom detectors.

```rust
use veil_plugin::{PluginLoader, DetectorPlugin};

let loader = PluginLoader::new();
let plugin = loader.load("./my_detector.so")?;
```

## New Features

### Encrypted Vault

```rust
use veil_crypto::{EncryptedVault, LocalKeyProvider};

let key_provider = LocalKeyProvider::from_file("keys.json")?;
let vault = EncryptedVault::new(key_provider);
vault.store("token123", "original_value")?;
```

### Key Rotation

```rust
vault.rotate_key()?;  // Re-encrypts all data with new key
```

### WebSocket Streaming

```javascript
const ws = new WebSocket('ws://localhost:8080/ws/stream');
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.type === 'finding') {
    console.log('Found PII:', msg.category);
  }
};
ws.send(JSON.stringify({ type: 'data', chunk: btoa(text), final: true }));
```

### Batch Processing

```bash
curl -X POST http://localhost:8080/api/v1/batch \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "files": [
      {"name": "doc1.txt", "content": "base64..."},
      {"name": "doc2.txt", "content": "base64..."}
    ],
    "parallel": true
  }'
```

### Async Parsers

```rust
use veil_parsers::AsyncTextParser;

let parser = AsyncTextParser::new();
let result = parser.parse_async(reader).await?;
```

## Error Codes

All errors now include standardized codes:

| Range | Category |
|-------|----------|
| VEIL-1xxx | Parsing errors |
| VEIL-2xxx | Detection errors |
| VEIL-3xxx | Redaction errors |
| VEIL-4xxx | Crypto errors |
| VEIL-5xxx | Plugin errors |
| VEIL-6xxx | API errors |
| VEIL-7xxx | Configuration errors |

## Testing

### Unit Tests

```bash
cargo test -p veil-core
cargo test -p veil-policy
```

### Browser Tests (WASM)

```bash
cd crates/veil-wasm
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox
```

### Property Tests

```bash
cargo test --features proptest
```

### Fuzzing

```bash
cd fuzz
cargo +nightly fuzz run pdf_parser -- -max_total_time=60
```

## Contributing

See CONTRIBUTING.md for:
- Development setup
- Code style guidelines
- PR requirements
- Testing requirements

## Resources

- [spec.md](./spec.md) - Full specification
- [plan.md](./plan.md) - Implementation plan
- [research.md](./research.md) - Technical decisions
- [data-model.md](./data-model.md) - Entity definitions
- [contracts/](./contracts/) - API specifications
