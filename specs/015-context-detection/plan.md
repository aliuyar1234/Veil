# Implementation Plan: Context-Aware Detection

**Branch**: `015-context-detection` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/015-context-detection/spec.md`

## Summary

Implement context-aware PII detection that uses surrounding text, structural patterns, and language-specific markers to improve detection accuracy and reduce false positives. The system analyzes contextual markers (honorifics, labels, headers), multi-line address structures, and applies language-specific rules to boost or suppress detection confidence scores. This feature extends veil-detect with a post-processing context analysis layer.

## Technical Context

**Language/Version**: Rust (stable, 1.75+)
**Primary Dependencies**: regex, unicode-segmentation, serde, serde_yaml (for custom rules), once_cell
**Storage**: In-memory context rules; optional YAML config files for custom rules
**Testing**: cargo test (TDD workflow with multi-language test fixtures)
**Target Platform**: Cross-platform library (Linux, macOS, Windows, WASM-compatible)
**Project Type**: Extension to veil-detect crate
**Performance Goals**: <10% overhead on detection time; context analysis in <5ms per 1KB text
**Constraints**: Support EN/DE/FR minimum; maintain WASM compatibility; no blocking I/O in analysis
**Scale/Scope**: Post-process detection results; analyze surrounding context within ±50 words

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ✅ PASS | No unsafe needed; regex from trusted dependency; no code execution |
| II. Stability & Error Handling | ✅ PASS | Result<T, ContextError> for all operations; graceful degradation if context unclear |
| III. Performance | ✅ PASS | <10% overhead required by FR-009; lazy evaluation of context rules |
| IV. Simplicity & Minimalism | ✅ PASS | Single-purpose context analysis; additive to existing detection pipeline |
| V. Test-First Development | ✅ PASS | TDD with fixtures in EN/DE/FR; edge case coverage for ambiguous context |
| VI. Dependency Discipline | ✅ PASS | Reuses existing regex/unicode deps; serde_yaml only for custom config |
| VII. Rust Standards | ✅ PASS | Clippy clean; documented public API |

**Gate Result**: PASS - All principles satisfied

## Project Structure

### Documentation (this feature)

```text
specs/015-context-detection/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 output - context analysis techniques
├── data-model.md        # Phase 1 output - entity definitions
├── quickstart.md        # Phase 1 output - usage examples
├── contracts/           # Phase 1 output - trait definitions
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/veil-detect/
├── Cargo.toml           # Add serde_yaml dependency
└── src/
    ├── lib.rs           # Export context module types
    ├── context/         # NEW: Context analysis module
    │   ├── mod.rs       # Module exports
    │   ├── analyzer.rs  # ContextAnalyzer (main API)
    │   ├── marker.rs    # ContextMarker detection
    │   ├── rule.rs      # ContextRule, ContextAction
    │   ├── config.rs    # ContextConfig, language-specific rules
    │   ├── address.rs   # AddressBlock detection
    │   ├── table.rs     # TableContext for column headers
    │   ├── language.rs  # Language detection/markers
    │   ├── adjustment.rs# Confidence adjustment logic
    │   ├── loader.rs    # YAML config loader
    │   ├── error.rs     # ContextError
    │   └── builtin/     # Built-in context rules
    │       ├── mod.rs
    │       ├── en.rs    # English markers
    │       ├── de.rs    # German markers
    │       └── fr.rs    # French markers
    ├── finding.rs       # EXTEND: Add context_reasoning field
    └── detector.rs      # EXTEND: Apply context analysis post-detection

tests/
└── context/             # Context analysis integration tests
    ├── fixtures/        # Multi-language test files
    │   ├── en/
    │   ├── de/
    │   └── fr/
    ├── name_context_tests.rs
    ├── suppression_tests.rs
    ├── address_tests.rs
    ├── table_tests.rs
    └── custom_rules_tests.rs

data/context/            # NEW: Built-in context patterns
├── markers_en.yaml      # English contextual markers
├── markers_de.yaml      # German contextual markers
├── markers_fr.yaml      # French contextual markers
└── README.md            # Context rule documentation
```

**Structure Decision**: Extend existing veil-detect crate with a new `context` module. This keeps context analysis coupled with detection while maintaining modularity. Context runs as a post-processing step that adjusts Finding confidence scores and adds reasoning metadata.

## Implementation Phases

### Phase 0: Research & Decisions

**Deliverable**: `research.md`

Topics to investigate:
1. **Context Window Size**: Optimal character/word distance for context markers (±25 words? ±50 words?)
2. **Address Pattern Libraries**: Existing address parsing libraries (libpostal analysis, performance)
3. **Language Detection**: Lightweight lang detection for section-aware context (whatlang vs lingua?)
4. **Confidence Scoring**: Algorithm for context-based boost/suppress (additive? multiplicative? cap at 1.0?)
5. **Table Structure Detection**: Heuristics for CSV vs plain text with aligned columns
6. **Performance Profiling**: Baseline detection speed; target overhead budget allocation

### Phase 1: Design & Contracts

**Deliverables**: `data-model.md`, `quickstart.md`, `contracts/`

1. **data-model.md**: Define ContextRule, ContextMarker, ContextAnalysis, AddressBlock, TableContext entities
2. **quickstart.md**: Example usage of ContextAnalyzer API, custom rule YAML format
3. **contracts/**: Trait definitions for ContextAnalyzer, ContextRuleProvider

### Phase 2: Core Context Types (TDD)

**Order**: FR-007 (confidence adjustment)

1. Create `context/` module structure
2. Define `ContextRule` (pattern, action, weight, language)
3. Define `ContextAction` enum (Boost, Suppress, Neutral)
4. Define `ContextMarker` (type, text, position, language)
5. Define `ContextAnalysis` (markers found, adjustments applied, reasoning)
6. Implement confidence adjustment algorithm
7. Tests: Boost/suppress confidence scores

### Phase 3: Marker Detection (TDD)

**Order**: FR-001 (name detection), FR-005 (language-specific)

1. Implement `ContextMarker` detection
2. Build honorific patterns (Mr., Dr., Herr, Frau, Monsieur, etc.)
3. Build label patterns ("Dear", "signed by", "Contact person")
4. Implement language-aware marker matching
5. Tests: Detect markers in EN/DE/FR text

### Phase 4: Built-in Rules (TDD)

**Order**: FR-001, FR-002, FR-005

1. Create YAML rule files (markers_en/de/fr.yaml)
2. Define boost rules for person names (honorifics, labels)
3. Define suppress rules for false positives ("version", "order #", "ISBN")
4. Implement rule loader from YAML
5. Tests: Verify rules loaded and applied correctly

### Phase 5: Suppression Logic (TDD)

**Order**: FR-002 (reduce false positives)

1. Implement pattern-specific suppression
2. Add IP address suppression ("version", "internal")
3. Add credit card suppression ("order #", "SKU")
4. Add phone suppression ("ISBN", "product code")
5. Tests: False positive reduction scenarios

### Phase 6: Address Detection (TDD)

**Order**: FR-003 (multi-line addresses)

1. Implement AddressBlock detection
2. Define address component patterns (street, city, postal, country)
3. Support EN/DE/FR address formats
4. Multi-line address merging
5. Tests: Address detection across formats

### Phase 7: Table Context (TDD)

**Order**: FR-004 (column header context)

1. Implement TableContext detection
2. Detect CSV vs aligned columns
3. Extract column headers
4. Apply header-based confidence boost
5. Tests: CSV with "Email", "Name", "ID" headers

### Phase 8: Analyzer Integration (TDD)

**Order**: FR-007, FR-008, FR-010

1. Implement ContextAnalyzer main API
2. Integrate with detection pipeline (post-process Findings)
3. Add context_reasoning field to Finding
4. Apply multi-language section detection
5. Tests: End-to-end context analysis

### Phase 9: Custom Rules (TDD)

**Order**: FR-006 (configurable rules)

1. Implement custom rule YAML schema
2. Add user-defined pattern support
3. Implement ContextConfig loader
4. Support runtime rule registration
5. Tests: Load and apply custom rules

### Phase 10: Performance & Polish

**Order**: FR-009 (<10% overhead)

1. Profile context analysis performance
2. Optimize regex compilation (once_cell caching)
3. Benchmark against baseline detection
4. Add performance tests
5. Documentation and examples
6. CLI integration (optional --context flag)

## Dependencies to Add

```toml
# In crates/veil-detect/Cargo.toml
[dependencies]
# Already present:
# regex.workspace = true
# unicode-segmentation.workspace = true
# serde.workspace = true
# once_cell.workspace = true

# New for context detection:
serde_yaml.workspace = true  # Custom rule YAML loading
```

Note: All other required dependencies (regex, unicode-segmentation, serde, once_cell) are already in veil-detect.

## Context Rule YAML Format

```yaml
# Example: data/context/markers_en.yaml
version: "1.0"
language: "en"
rules:
  - pattern: "(?i)\\b(mr|mrs|ms|dr|prof)\\.?\\s+"
    action: Boost
    weight: 0.3
    category: PersonName
    description: "Honorific before name"

  - pattern: "(?i)\\bversion\\s+\\d"
    action: Suppress
    weight: 0.5
    category: IpAddress
    description: "Version number, not IP"

  - pattern: "(?i)\\border\\s*#"
    action: Suppress
    weight: 0.6
    category: CreditCard
    description: "Order number, not card"
```

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Context analysis performance | Lazy regex compilation with once_cell; limit context window; profile early |
| Ambiguous context conflicts | Use confidence scoring; higher base confidence wins over context |
| Language detection accuracy | Allow manual language hints; default to EN; section-aware fallback |
| False suppression | Configurable thresholds; context only adjusts, doesn't eliminate detections |
| YAML parsing errors | Strict validation; graceful fallback to built-in rules on error |
| WASM compatibility | No file I/O in core; YAML loading optional (compile-time embed for WASM) |

## Success Metrics (from spec)

- [ ] SC-001: Name detection recall +30% with context markers vs dictionary-only
- [ ] SC-002: False positive rate -50% for IP-like patterns with context suppression
- [ ] SC-003: Multi-line address detection 90% accuracy across EN/DE/FR
- [ ] SC-004: Column header context +20% detection precision for tabular data
- [ ] SC-005: Context analysis <10% processing time overhead
- [ ] SC-006: Custom context rules work correctly when loaded from YAML

## Complexity Tracking

> No violations identified - design follows constitution principles.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | - | - |

## Post-Design Constitution Re-Check

*Re-evaluated after Phase 1 design completion (to be filled during Phase 1)*

| Principle | Status | Post-Design Notes |
|-----------|--------|-------------------|
| I. Security First | TBD | |
| II. Stability & Error Handling | TBD | |
| III. Performance | TBD | |
| IV. Simplicity & Minimalism | TBD | |
| V. Test-First Development | TBD | |
| VI. Dependency Discipline | TBD | |
| VII. Rust Standards | TBD | |

**Post-Design Gate Result**: TBD - To be completed after Phase 1
