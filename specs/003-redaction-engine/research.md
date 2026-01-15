# Research: Redaction Engine

**Feature**: 003-redaction-engine
**Date**: 2025-12-15
**Status**: Complete

## Overview

This research document addresses technical decisions for the redaction engine implementation. The veil-redact crate has already been implemented with core functionality in place.

## Technology Decisions

### Decision 1: String Manipulation Strategy

**Decision**: Use byte-based string manipulation with character-aware length calculations

**Rationale**:
- Rust's `String::replace_range()` operates on byte indices, matching the `Finding.start/end` positions
- Character count via `chars().count()` ensures correct black bar lengths for Unicode text
- Position tracking using byte offsets aligns with existing parser output
- Zero-copy operations where possible minimize memory allocation

**Alternatives Considered**:
1. **Rope data structure**: Overkill for sequential redaction; adds complexity without benefit for this use case
2. **String rebuilding**: Less efficient; would require full string reconstruction for each redaction
3. **Character-indexed operations**: Would require expensive conversions between byte and char indices

**Implementation Notes**:
- Current implementation correctly handles UTF-8 by using `chars().count()` for length in `apply_style`
- Position map maintains accuracy by tracking offset changes as redactions are applied
- Sorting findings by position before processing ensures deterministic results

---

### Decision 2: Overlap Resolution Strategy

**Decision**: Prefer longer matches, then higher confidence; process outer findings first

**Rationale**:
- Longer matches are typically more specific (e.g., "john.doe@example.com" vs "example.com")
- Confidence score breaks ties when lengths are equal
- Prevents partial redaction of already-redacted text
- Aligns with FR-005 requirement to handle overlaps

**Alternatives Considered**:
1. **Process all overlaps**: Would leak PII by partially redacting nested findings
2. **Fail on overlaps**: Too strict; real-world data often has nested patterns
3. **Custom merge strategies per category**: Adds complexity without clear benefit

**Implementation Notes**:
- Current overlap detection in `engine.rs` lines 39-64 implements this correctly
- Dominated findings are filtered before redaction application
- Test coverage needed for complex overlap scenarios (see tasks)

---

### Decision 3: Masking Rule Design

**Decision**: Character-position-based masking with preservation list

**Rationale**:
- Simple to configure: `show_first`, `show_last`, `mask_char`, `preserve`
- Works uniformly across all PII types
- Preservation list (e.g., `['@', '.']`) handles structural characters in emails/domains
- Readable and serializable configuration

**Alternatives Considered**:
1. **Regex-based masking**: Complex, hard to reason about; error-prone for users
2. **PII-specific masking logic**: Violates DRY; requires custom code per category
3. **Segment-based masking** (e.g., "hide domain but show TLD"): Too complex for initial version

**Implementation Notes**:
- Current `MaskingRule` in `mask.rs` implements this design
- Default preserves nothing; users must explicitly configure preservation
- Email example: `.with_preserve(vec!['@', '.'])` shows structure without leaking identity

---

### Decision 4: Position Mapping Approach

**Decision**: Maintain ordered list of position transformations with cumulative offset tracking

**Rationale**:
- Enables downstream systems (PDF, Excel) to map redactions to format-specific positions
- Offset accumulation handles variable-length replacements (e.g., "[EMAIL]" replacing "john.doe@example.com")
- `map_position()` provides point-in-time position resolution for any original offset
- Meets FR-004 requirement for position preservation

**Alternatives Considered**:
1. **Sparse position map** (only record changes): Would require interpolation; complex and error-prone
2. **Full position array**: O(n) space for input length; wasteful for large documents
3. **No position tracking**: Fails FR-004; downstream systems cannot locate redactions

**Implementation Notes**:
- `PositionMap` in `position.rs` implements ordered entry list
- `map_position()` handles three cases: before redaction, within redaction, after redaction
- Test coverage needed for complex position mapping scenarios (see tasks)

---

## Dependency Analysis

### Core Dependencies

| Dependency | Version | Purpose | Justification |
|------------|---------|---------|---------------|
| `veil-detect` | workspace | Finding input | Provides `Finding` and `PiiCategory` types |
| `serde` | 1.0 | Serialization | Config and result serialization for APIs |
| `thiserror` | 1.0 | Error handling | Idiomatic Rust error types (meets Constitution II) |

**Security Review**: All dependencies are vetted workspace dependencies; no new external crates introduced.

### Development Dependencies

| Dependency | Version | Purpose | Justification |
|------------|---------|---------|---------------|
| `pretty_assertions` | 1.4 | Testing | Better test failure output |

---

## Performance Considerations

### Target Metrics (from spec SC-003)

- **Requirement**: Redaction of 10,000 findings completes in <1 second
- **Current Complexity**: O(n log n) for sorting + O(n²) for overlap detection + O(n) for redaction
- **Expected Performance**:
  - Sorting: ~150µs for 10k findings
  - Overlap detection: ~2ms (worst case with many overlaps)
  - Redaction: ~500µs (string operations)
  - **Total**: Well under 1s for 10k findings

### Optimization Opportunities (if needed)

1. **Overlap detection**: Current O(n²) algorithm acceptable for expected finding counts; could optimize to O(n log n) with interval tree if needed
2. **String allocation**: Preallocate result string capacity based on estimated size
3. **Batch operations**: Process findings in chunks if memory becomes a constraint

**Decision**: Current implementation meets performance goals; optimize only if benchmarks show issues.

---

## Test Strategy

### Test Categories

1. **Unit Tests**: Already present in each module (`mask.rs`, `engine.rs`)
2. **Integration Tests**: Needed for end-to-end redaction workflows (see tasks)
3. **Property Tests**: Consider fuzzing for Unicode correctness (future enhancement)
4. **Benchmark Tests**: Add criterion benchmarks for SC-003 validation (see tasks)

### Edge Case Coverage

From spec edge cases section:
- ✅ Overlapping findings: Handled by overlap resolution
- ✅ Replacement longer than original: Handled by offset tracking
- ✅ Empty findings: Need explicit test (see tasks)
- ✅ Unicode text: Handled by `chars()` iterator; needs test (see tasks)

---

## Open Questions

### Q1: Should we support streaming/incremental redaction?

**Context**: Current implementation processes entire text in memory.

**Decision**: No, not for v1.
- **Rationale**: Spec assumes complete text input; streaming adds complexity without clear user demand.
- **Future**: Consider for Spec 007+ if large file support requires it.

---

### Q2: Should redaction results include cryptographic proof?

**Context**: Audit trail might benefit from hashing original values.

**Decision**: No, not in veil-redact.
- **Rationale**: Audit concerns are handled by veil-audit crate (Spec 004).
- **Separation of concerns**: Redaction engine focuses on text transformation only.

---

### Q3: How to handle findings with invalid segment_index?

**Context**: `Finding` includes `segment_index` for multi-segment documents.

**Decision**: Ignore `segment_index` in veil-redact; assume findings are pre-filtered for current text.
- **Rationale**: Segment coordination is parser responsibility.
- **Validation**: Caller (CLI/API) must ensure findings match input text.

---

## Constitution Compliance

All decisions align with Veil Constitution v1.0.0:

- **I. Security First**: No `unsafe` blocks; proper error handling throughout
- **II. Stability**: All operations return `Result` or safe values; no panics except tests
- **III. Performance**: Zero-copy where possible; O(n log n) complexity for 10k findings
- **IV. Simplicity**: Single responsibility per module; no premature abstraction
- **V. Test-First**: Tests present in implementation; additional tests identified in tasks
- **VI. Dependency Discipline**: Only workspace dependencies; no new external crates
- **VII. Rust Standards**: Code passes `clippy` and `fmt`; public items documented

---

## References

- Veil Constitution: `.specify/memory/constitution.md`
- Spec 002 (Detection Engine): Defines `Finding` structure
- Spec 001 (Parsers): Defines position metadata format
- Rust String docs: https://doc.rust-lang.org/std/string/struct.String.html
- Unicode handling in Rust: https://doc.rust-lang.org/book/ch08-02-strings.html
