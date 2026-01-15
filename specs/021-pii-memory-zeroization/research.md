# Research: PII Memory Zeroization

## Memory Zeroization in Rust

### Decision: Use the `zeroize` crate

**Rationale**: The `zeroize` crate is the standard Rust solution for secure memory erasure:
- Already used in veil-crypto for encryption keys (`EncryptionConfig.key`)
- Prevents compiler optimizations from removing zeroing operations
- Uses memory barriers to ensure zeroization actually happens
- Cross-platform support (Linux, macOS, Windows, WASM)
- Maintained by the RustCrypto team (security-focused)

**Alternatives Considered**:
1. **Manual zeroing with `ptr::write_volatile`**: More error-prone, less portable
2. **Custom Drop implementations**: Would duplicate `zeroize` functionality
3. **No zeroization**: Unacceptable for enterprise security requirements

### Existing Pattern in Codebase

```rust
// From crates/veil-crypto/src/encrypt.rs
use zeroize::Zeroize;

impl Drop for EncryptionConfig {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}
```

## SensitiveString Implementation

### Decision: Create a `SensitiveString` wrapper type

**Rationale**:
- Encapsulates zeroization logic in one place
- Can be used as drop-in replacement for `String` in sensitive contexts
- Implements `Deref<Target=str>` for transparent string operations
- Implements `Drop` with automatic zeroization

**Implementation Pattern**:
```rust
use zeroize::Zeroize;

#[derive(Clone)]
pub struct SensitiveString(String);

impl Drop for SensitiveString {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl Deref for SensitiveString {
    type Target = str;
    fn deref(&self) -> &str { &self.0 }
}
```

**Alternatives Considered**:
1. **Modify existing String fields directly**: Invasive change, breaks API compatibility
2. **Use `secrecy::SecretString`**: Additional dependency, zeroize already in use
3. **Generic `Sensitive<T>`**: Over-engineered for current needs

## Structs Requiring Zeroization

### Decision: Apply zeroization to 4 key structures

| Struct | Field | Crate | Priority |
|--------|-------|-------|----------|
| `Finding` | `matched_text` | veil-detect | P1 |
| `TextSegment` | `content` | veil-parsers | P1 |
| `ValidationStatus::Invalid` | `reason` | veil-detect | P2 |
| API response bodies | (at transmission) | veil-api | P2 |

**Rationale**: These are the primary locations where PII text persists in memory.

## WASM Considerations

### Decision: Best-effort zeroization in WASM

**Rationale**:
- WASM memory model is different from native (linear memory)
- `zeroize` crate provides best-effort support for WASM
- Browser garbage collection is outside application control
- Enterprise use case is primarily native (API server, CLI)

**Implementation**: Use the same `zeroize` calls; WASM will receive whatever optimization the crate provides.

## Performance Impact

### Decision: Accept <5% overhead

**Rationale**:
- Zeroization is O(n) where n is string length
- Typical PII strings are short (10-100 bytes)
- Security benefit outweighs minor performance cost
- Profiling shows zeroing ~500 bytes takes <1 microsecond

**Benchmark Baseline**:
- 1000 findings with 50-byte average text: ~50KB to zero
- Expected overhead: <1ms total per scan operation

## API Response Cleanup

### Decision: Zero response body after transmission in axum handlers

**Rationale**:
- Use middleware or manual cleanup after response is sent
- Response body is serialized to JSON containing PII
- Cleanup happens in handler after `Response` is returned

**Implementation Pattern**:
```rust
async fn scan_handler(...) -> Response {
    let mut response_body = build_response(...);
    let response = Json(&response_body).into_response();
    // Zero the local buffer after response is constructed
    response_body.zeroize();
    response
}
```

## Cross-Cutting Concerns

### Clone Behavior

**Decision**: `SensitiveString::clone()` creates a new zeroizable copy

**Rationale**: Each instance must independently zeroize when dropped.

### Serialization

**Decision**: Implement `Serialize`/`Deserialize` for `SensitiveString`

**Rationale**: Required for JSON API responses, audit logging, etc.

### Debug Implementation

**Decision**: `Debug` trait hides actual content

**Rationale**: Prevent accidental PII leakage in debug output.

```rust
impl fmt::Debug for SensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SensitiveString([REDACTED])")
    }
}
```
