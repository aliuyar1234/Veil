# Implementation Plan: PII Memory Zeroization

**Branch**: `021-pii-memory-zeroization` | **Date**: 2025-12-17 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/021-pii-memory-zeroization/spec.md`

## Summary

Implement secure memory zeroization for PII data structures to meet enterprise security requirements (SOC2, HIPAA, PCI-DSS). This extends the existing zeroization pattern used for encryption keys in veil-crypto to cover all sensitive text in the system: `Finding.matched_text`, `TextSegment.content`, and API response bodies. Uses the existing `zeroize` crate dependency.

## Technical Context

**Language/Version**: Rust 1.75+ (stable, 2021 edition)
**Primary Dependencies**: zeroize (already in use), serde (for serialization)
**Storage**: N/A (in-memory operations only)
**Testing**: cargo test, memory inspection tests
**Target Platform**: Cross-platform (Linux, macOS, Windows, WASM)
**Project Type**: Multi-crate workspace modification
**Performance Goals**: <5% overhead on typical scan operations
**Constraints**: Must not break existing API compatibility
**Scale/Scope**: Modifying 4 crates: veil-detect, veil-parsers, veil-api, veil-core (new)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ✅ PASS | Core security feature - secure memory erasure |
| II. Stability & Error Handling | ✅ PASS | Using Drop for cleanup ensures execution even on panic |
| III. Performance | ✅ PASS | <5% overhead acceptable for security benefit |
| IV. Simplicity & Minimalism | ✅ PASS | Single SensitiveString type encapsulates all logic |
| V. Test-First Development | ✅ PASS | Tests verify zeroization behavior |
| VI. Dependency Discipline | ✅ PASS | Using existing zeroize crate, no new dependencies |
| VII. Rust Standards | ✅ PASS | Idiomatic Drop implementation, clippy clean |

## Project Structure

### Documentation (this feature)

```text
specs/021-pii-memory-zeroization/
├── plan.md              # This file
├── research.md          # Zeroization research
├── data-model.md        # SensitiveString definition
├── quickstart.md        # Integration guide
├── contracts/           # Test specifications
└── tasks.md             # Implementation tasks
```

### Source Code (repository root)

```text
crates/
├── veil-core/           # NEW: Shared types (SensitiveString)
│   └── src/
│       ├── lib.rs
│       └── sensitive.rs # SensitiveString type
├── veil-detect/         # MODIFY: Use SensitiveString for Finding
│   └── src/
│       └── finding.rs   # Update matched_text type
├── veil-parsers/        # MODIFY: Use SensitiveString for TextSegment
│   └── src/
│       └── types.rs     # Update content type
└── veil-api/            # MODIFY: Add response cleanup
    └── src/
        └── routes/      # Cleanup in handlers
```

**Structure Decision**: Creating new `veil-core` crate for shared types to avoid circular dependencies between veil-detect and veil-parsers. Both crates will depend on veil-core.

## Complexity Tracking

> No violations - implementation follows existing patterns.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| New veil-core crate | Shared SensitiveString type needed by multiple crates | Putting in veil-detect would create dependency from veil-parsers to veil-detect |

## Implementation Approach

### Phase 1: Create veil-core with SensitiveString

1. Create new `crates/veil-core` crate
2. Implement `SensitiveString` wrapper type with zeroization
3. Add tests for zeroization behavior

### Phase 2: Integrate SensitiveString into veil-detect

1. Add veil-core dependency
2. Change `Finding.matched_text` from `String` to `SensitiveString`
3. Update all code paths that create/use `matched_text`

### Phase 3: Integrate SensitiveString into veil-parsers

1. Add veil-core dependency
2. Change `TextSegment.content` from `String` to `SensitiveString`
3. Update all code paths that create/use segment content

### Phase 4: API Response Cleanup

1. Add response body zeroization in API handlers
2. Ensure serialized response data is zeroed after transmission

### Phase 5: Validation

1. Run full test suite
2. Verify zeroization with memory inspection tests
3. Performance benchmarking
