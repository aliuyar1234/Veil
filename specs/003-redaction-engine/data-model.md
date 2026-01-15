# Data Model: Redaction Engine

**Feature**: 003-redaction-engine
**Date**: 2025-12-15

## Entity Overview

The redaction engine operates on six primary entities, organized in a pipeline:

```
Finding[] → RedactionEngine(RedactionConfig) → RedactionResult
                                                      ↓
                                        (text, AppliedRedaction[], PositionMap)
```

---

## Core Entities

### 1. RedactionStyle

**Purpose**: Defines the type and parameters of redaction to apply.

**Type**: Enum (tagged union)

**Variants**:

```rust
pub enum RedactionStyle {
    /// Replace with category label: [EMAIL], [IBAN]
    Label,

    /// Replace with solid characters: ████████
    BlackBar {
        /// Character to use (default: █)
        char: char,
    },

    /// Partial masking with rules
    Mask(MaskingRule),

    /// Custom replacement text
    Custom {
        text: String,
    },
}
```

**Validation Rules**:
- `BlackBar.char` must be single Unicode character
- `Mask` variant must contain valid `MaskingRule`
- `Custom.text` must not be empty

**Related Requirements**: FR-001, FR-002, FR-003, FR-008

---

### 2. MaskingRule

**Purpose**: Configuration for partial masking (showing some characters, hiding others).

**Type**: Struct

**Fields**:

| Field | Type | Description | Constraints | Default |
|-------|------|-------------|-------------|---------|
| `show_first` | `usize` | Characters visible at start | ≥ 0 | 1 |
| `show_last` | `usize` | Characters visible at end | ≥ 0 | 4 |
| `mask_char` | `char` | Character for masked positions | Single char | '*' |
| `preserve` | `Vec<char>` | Characters never masked (e.g., '@') | - | empty |

**Validation Rules**:
- If `show_first + show_last ≥ text.len()`, return text unchanged
- `mask_char` must be valid Unicode character
- `preserve` entries must be unique

**Behavior**:

```rust
// Example: MaskingRule { show_first: 1, show_last: 4, mask_char: '*', preserve: ['@'] }
apply("john.doe@example.com") → "j******@*******.com"
```

**Related Requirements**: FR-003, SC-005

---

### 3. RedactionConfig

**Purpose**: Configuration for the redaction engine, including default style and per-category overrides.

**Type**: Struct

**Fields**:

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `default_style` | `RedactionStyle` | Style applied to all categories | Valid style |
| `category_styles` | `HashMap<PiiCategory, RedactionStyle>` | Per-category overrides | - |

**Validation Rules**:
- `default_style` must be valid `RedactionStyle`
- All values in `category_styles` must be valid

**Methods**:

```rust
impl RedactionConfig {
    fn get_style(&self, category: &PiiCategory) -> &RedactionStyle;
    fn set_category_style(&mut self, category: PiiCategory, style: RedactionStyle);
}
```

**Related Requirements**: FR-001 through FR-003, FR-008

---

### 4. RedactionEngine

**Purpose**: Stateless engine that applies redactions based on configuration.

**Type**: Struct

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `config` | `RedactionConfig` | Configuration to use |

**Primary Method**:

```rust
pub fn redact(&self, text: &str, findings: &[Finding]) -> RedactionResult
```

**Algorithm** (FR-005, FR-009):

1. **Sort findings**: By position (ascending), then length (descending), then confidence (descending)
2. **Remove overlaps**: Keep longer/higher-confidence findings; discard dominated ones
3. **Apply redactions**: Process in order, tracking position offset changes
4. **Build result**: Construct `RedactionResult` with redacted text, applied list, position map

**Overlap Resolution Logic**:

```
For each finding F:
  If F overlaps with existing finding E in result set:
    If len(E) > len(F) OR (len(E) == len(F) AND conf(E) ≥ conf(F)):
      Discard F (dominated)
    Else:
      Remove E, add F (F dominates)
  Else:
    Add F to result set
```

**Related Requirements**: FR-005, FR-006, FR-009, FR-010

---

### 5. AppliedRedaction

**Purpose**: Record of a single redaction that was applied.

**Type**: Struct

**Fields**:

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `original` | `String` | Text before redaction | `Finding.matched_text` |
| `replacement` | `String` | Text after redaction | Computed by `apply_style()` |
| `original_position` | `(usize, usize)` | (start, end) in original text | `Finding.start/end` |
| `new_position` | `(usize, usize)` | (start, end) in redacted text | Computed with offset |
| `category` | `PiiCategory` | PII category | `Finding.category` |

**Invariants**:
- `original_position.1 > original_position.0`
- `new_position.1 > new_position.0`
- `original.len() == original_position.1 - original_position.0` (byte length)

**Related Requirements**: FR-007, FR-009

---

### 6. PositionMap

**Purpose**: Mapping from original text positions to redacted text positions.

**Type**: Struct

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `entries` | `Vec<PositionMapEntry>` | Ordered list of position transformations |

**PositionMapEntry**:

| Field | Type | Description |
|-------|------|-------------|
| `original_start` | `usize` | Start position in original text |
| `original_end` | `usize` | End position in original text |
| `redacted_start` | `usize` | Start position in redacted text |
| `redacted_end` | `usize` | End position in redacted text |

**Methods**:

```rust
impl PositionMap {
    pub fn map_position(&self, original_pos: usize) -> Option<usize>;
}
```

**Position Mapping Algorithm**:

```
For position P in original text:
  offset = 0
  For each entry E in entries (ordered):
    If P < E.original_start:
      Return P + offset
    Else if P in [E.original_start, E.original_end):
      Return E.redacted_start (P is within redacted region)
    Else:
      offset += (E.redacted_end - E.redacted_start) - (E.original_end - E.original_start)
  Return P + offset
```

**Related Requirements**: FR-004, FR-007

---

### 7. RedactionResult

**Purpose**: Complete result of a redaction operation.

**Type**: Struct

**Fields**:

| Field | Type | Description | Related Requirements |
|-------|------|-------------|----------------------|
| `text` | `String` | Redacted text | FR-010 (non-PII preserved) |
| `redactions` | `Vec<AppliedRedaction>` | List of applied redactions | FR-007 |
| `position_map` | `PositionMap` | Position mapping | FR-004, FR-007 |

**Methods**:

```rust
impl RedactionResult {
    pub fn redaction_count(&self) -> usize;
    pub fn has_redactions(&self) -> bool;
}
```

**Validation Rules**:
- `text` contains no original PII values (SC-001)
- `redactions.len() == position_map.entries().len()`
- Positions in `redactions` and `position_map` are consistent

**Related Requirements**: FR-007, SC-001, SC-002

---

## Data Flow

### Input: Finding (from veil-detect)

```rust
pub struct Finding {
    pub matched_text: String,      // "john@example.com"
    pub category: PiiCategory,      // Email
    pub start: usize,               // 10
    pub end: usize,                 // 27
    pub confidence: f32,            // 1.0
    pub validation: ValidationStatus, // Valid
    pub segment_index: usize,       // 0 (ignored by redact engine)
}
```

**Note**: `segment_index` is **not used** by the redaction engine. The caller must ensure all findings correspond to the input text.

---

### Processing: RedactionEngine

```rust
let config = RedactionConfig::default(); // Label style
let engine = RedactionEngine::new(config);

let text = "Contact me at john@example.com for details.";
let findings = vec![
    Finding {
        matched_text: "john@example.com".into(),
        category: PiiCategory::Email,
        start: 14,
        end: 31,
        // ... other fields
    }
];

let result = engine.redact(text, &findings);
```

---

### Output: RedactionResult

```rust
RedactionResult {
    text: "Contact me at [EMAIL] for details.",
    redactions: vec![
        AppliedRedaction {
            original: "john@example.com",
            replacement: "[EMAIL]",
            original_position: (14, 31),  // 17 bytes
            new_position: (14, 21),       // 7 bytes
            category: PiiCategory::Email,
        }
    ],
    position_map: PositionMap {
        entries: vec![
            PositionMapEntry {
                original_start: 14,
                original_end: 31,
                redacted_start: 14,
                redacted_end: 21,
            }
        ]
    }
}
```

**Position mapping example**:
- Original position 20 (inside "john@example.com") → maps to 14 (start of "[EMAIL]")
- Original position 35 (in "for") → maps to 25 (35 - 10, offset of -10 from replacement)

---

## State Transitions

The redaction engine is **stateless**; each `redact()` call is independent.

### Within a Single Redaction Call

```
1. Input: (text, findings[])
   ↓
2. Sort findings by (position ASC, length DESC, confidence DESC)
   ↓
3. Remove overlapping findings (keep longer/higher confidence)
   ↓
4. For each finding (in order):
     a. Get style for finding.category
     b. Compute replacement text
     c. Apply replacement (update result_text)
     d. Track offset change
     e. Record AppliedRedaction
     f. Add PositionMapEntry
   ↓
5. Output: RedactionResult
```

**No persistent state** between calls.

---

## Validation & Invariants

### Pre-conditions

1. Input text is valid UTF-8 (guaranteed by Rust `&str`)
2. Finding positions are valid byte offsets within text
3. `finding.start < finding.end` for all findings
4. `finding.matched_text == text[finding.start..finding.end]` (byte slice)

### Post-conditions

1. **SC-001**: `result.text` contains no original PII (all findings replaced)
2. **SC-002**: Position map accuracy 100% (all original positions map correctly)
3. **SC-004**: Black bar length matches original (for BlackBar style)
4. **FR-010**: Non-PII text exactly preserved

### Runtime Invariants

1. Offset tracking: cumulative offset matches sum of (replacement_len - original_len) for all processed findings
2. Position consistency: `redactions[i]` corresponds to `position_map.entries[i]`
3. Monotonic positions: `redactions[i].original_position.0 < redactions[i+1].original_position.0`

---

## Error Handling

The current implementation **does not return errors**. All operations are infallible:

- Invalid positions: Assumed to be validated by caller
- Empty findings: Handled gracefully (return original text)
- Unicode issues: `chars()` iterator handles all valid UTF-8

**Future**: Consider adding `Result<RedactionResult, RedactError>` if validation is needed. For now, follows "garbage in, garbage out" principle with safe behavior.

---

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Sort findings | O(n log n) | `n` = number of findings |
| Overlap removal | O(n²) worst case | Acceptable for expected n < 10k |
| Apply redactions | O(n × m) | `m` = avg text length per finding |
| Position mapping | O(k) | `k` = number of entries (= n) |
| **Total** | **O(n² + nm)** | Dominates: O(n²) for n > m |

**Expected performance**: <1ms for 1000 findings, <100ms for 10k findings (SC-003: <1s for 10k ✅)

---

## Serialization

All public types implement `serde::Serialize` and `serde::Deserialize`:

### JSON Example

```json
{
  "text": "Contact me at [EMAIL] for details.",
  "redactions": [
    {
      "original": "john@example.com",
      "replacement": "[EMAIL]",
      "original_position": [14, 31],
      "new_position": [14, 21],
      "category": "email"
    }
  ],
  "position_map": {
    "entries": [
      {
        "original_start": 14,
        "original_end": 31,
        "redacted_start": 14,
        "redacted_end": 21
      }
    ]
  }
}
```

**Use cases**:
- API responses
- Audit logs (combined with veil-audit)
- Configuration files (RedactionConfig)

---

## Integration Points

### Upstream: veil-detect

**Inputs**: `Finding[]`, `PiiCategory`

**Contract**: Findings must have valid positions for input text.

---

### Downstream: Format-specific processors

**Outputs**: `RedactionResult` with `position_map`

**Usage**: PDF/Excel processors use `position_map.map_position()` to locate redactions in format-specific coordinates.

**Example** (hypothetical PDF processor):

```rust
let result = engine.redact(text, findings);

for entry in result.position_map.entries() {
    let pdf_coords = text_to_pdf_coords(entry.original_start);
    annotate_pdf(pdf_coords, "[REDACTED]");
}
```

---

## Testing Strategy

### Unit Tests (per entity)

- ✅ `MaskingRule::apply()` with various configurations
- ✅ `RedactionStyle` constructors and serialization
- ✅ `RedactionEngine::redact()` basic cases
- ⚠️ **Needed**: `PositionMap::map_position()` edge cases
- ⚠️ **Needed**: Overlap resolution complex scenarios

### Integration Tests

- ⚠️ **Needed**: End-to-end redaction with all styles
- ⚠️ **Needed**: Unicode text (emojis, multi-byte chars)
- ⚠️ **Needed**: Empty findings edge case
- ⚠️ **Needed**: Large-scale performance (10k findings)

### Property Tests (future)

- Invariant: `map_position()` round-trip consistency
- Invariant: No PII leakage in output
- Fuzz testing: Random findings with random text

---

## Open Design Questions

### Q1: Should we track redaction metadata (e.g., timestamp)?

**Decision**: No, not in RedactionResult.
- **Rationale**: Metadata belongs in audit layer (veil-audit).
- **Alternative**: Caller can wrap RedactionResult with timestamp if needed.

---

### Q2: Should we support undo/revert operations?

**Decision**: No, redaction is one-way.
- **Rationale**: Security by design; original PII should not be recoverable from RedactionResult.
- **Alternative**: Caller maintains original text separately if revert is needed.

---

## References

- Spec: `D:\Projekte\Veil\specs\003-redaction-engine\spec.md`
- Implementation: `D:\Projekte\Veil\crates\veil-redact\src\`
- veil-detect Finding: `D:\Projekte\Veil\crates\veil-detect\src\finding.rs`
