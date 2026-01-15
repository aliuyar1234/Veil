# Library API Contract: veil-redact

**Version**: 0.1.0
**Type**: Rust Library (crate)
**Stability**: Alpha (breaking changes possible)

---

## Public API Surface

### Top-level Functions

#### `redact`

**Signature**:
```rust
pub fn redact(text: &str, findings: &[Finding]) -> RedactionResult
```

**Purpose**: Redact findings using default configuration (label style).

**Parameters**:
- `text`: Input text to redact
- `findings`: Slice of PII findings from veil-detect

**Returns**: `RedactionResult` with redacted text and metadata

**Panics**: Never (infallible)

**Example**:
```rust
let result = redact("Email: john@example.com", &findings);
assert_eq!(result.text, "Email: [EMAIL]");
```

**Contract**:
- `text` unchanged if `findings` is empty
- All findings with valid positions are redacted
- Non-PII text is preserved exactly
- Position map has one entry per redaction

---

#### `redact_with_style`

**Signature**:
```rust
pub fn redact_with_style(
    text: &str,
    findings: &[Finding],
    style: RedactionStyle,
) -> RedactionResult
```

**Purpose**: Redact with a specific style.

**Parameters**:
- `text`: Input text
- `findings`: PII findings
- `style`: Redaction style to apply

**Returns**: `RedactionResult`

**Example**:
```rust
let result = redact_with_style(
    "IBAN: DE89370400440532013000",
    &findings,
    RedactionStyle::black_bar(),
);
assert_eq!(result.text, "IBAN: ██████████████████████");
```

**Contract**: Same as `redact()`, but uses provided style.

---

## Core Types

### `RedactionEngine`

**Purpose**: Stateless engine for applying redactions.

**Methods**:

#### `new`

```rust
pub fn new(config: RedactionConfig) -> Self
```

**Purpose**: Create engine with configuration.

**Parameters**: `config` - Redaction configuration

**Returns**: `RedactionEngine` instance

**Example**:
```rust
let config = RedactionConfig::with_style(RedactionStyle::black_bar());
let engine = RedactionEngine::new(config);
```

---

#### `redact`

```rust
pub fn redact(&self, text: &str, findings: &[Finding]) -> RedactionResult
```

**Purpose**: Apply redactions based on engine's configuration.

**Parameters**:
- `text`: Input text
- `findings`: PII findings

**Returns**: `RedactionResult`

**Behavior**:
1. Sort findings by position (ascending), length (descending), confidence (descending)
2. Remove overlapping findings (prefer longer/higher confidence)
3. Apply redactions in order with offset tracking
4. Build `RedactionResult` with all metadata

**Complexity**: O(n² + nm) where n=findings, m=avg finding length

**Contract**:
- Findings are processed in position order (FR-009)
- Overlaps resolved by preferring longer matches (FR-005)
- Position map is 100% accurate (SC-002)
- No PII leakage in output (SC-001)

---

### `RedactionConfig`

**Purpose**: Configuration for redaction engine.

**Fields**:
- `pub default_style: RedactionStyle` - Default style for all categories
- `pub category_styles: HashMap<PiiCategory, RedactionStyle>` - Per-category overrides

**Methods**:

#### `default`

```rust
fn default() -> Self
```

**Returns**: Config with `Label` style for all categories.

---

#### `with_style`

```rust
pub fn with_style(style: RedactionStyle) -> Self
```

**Purpose**: Create config with a specific default style.

**Parameters**: `style` - Default style

**Returns**: `RedactionConfig`

---

#### `set_category_style`

```rust
pub fn set_category_style(&mut self, category: PiiCategory, style: RedactionStyle)
```

**Purpose**: Override style for a specific PII category.

**Parameters**:
- `category`: PII category to override
- `style`: Style to use for this category

**Example**:
```rust
let mut config = RedactionConfig::default();
config.set_category_style(PiiCategory::Email, RedactionStyle::mask(rule));
```

---

#### `get_style`

```rust
pub fn get_style(&self, category: &PiiCategory) -> &RedactionStyle
```

**Purpose**: Get effective style for a category (with fallback to default).

**Parameters**: `category` - PII category

**Returns**: Reference to `RedactionStyle`

---

### `RedactionStyle`

**Purpose**: Enum defining redaction style.

**Variants**:

```rust
pub enum RedactionStyle {
    Label,
    BlackBar { char: char },
    Mask(MaskingRule),
    Custom { text: String },
}
```

**Constructors**:

#### `label`

```rust
pub fn label() -> Self
```

**Returns**: `RedactionStyle::Label`

---

#### `black_bar`

```rust
pub fn black_bar() -> Self
```

**Returns**: `RedactionStyle::BlackBar { char: '█' }`

---

#### `black_bar_with_char`

```rust
pub fn black_bar_with_char(c: char) -> Self
```

**Parameters**: `c` - Character to use for bars

**Returns**: `RedactionStyle::BlackBar { char: c }`

---

#### `mask`

```rust
pub fn mask(rule: MaskingRule) -> Self
```

**Parameters**: `rule` - Masking rule

**Returns**: `RedactionStyle::Mask(rule)`

---

#### `custom`

```rust
pub fn custom(text: impl Into<String>) -> Self
```

**Parameters**: `text` - Custom replacement text

**Returns**: `RedactionStyle::Custom { text }`

---

### `MaskingRule`

**Purpose**: Configuration for partial masking.

**Fields**:
- `pub show_first: usize` - Characters to show at start (default: 1)
- `pub show_last: usize` - Characters to show at end (default: 4)
- `pub mask_char: char` - Masking character (default: '*')
- `pub preserve: Vec<char>` - Characters never masked (default: empty)

**Methods**:

#### `new`

```rust
pub fn new(show_first: usize, show_last: usize) -> Self
```

**Parameters**:
- `show_first`: Characters visible at start
- `show_last`: Characters visible at end

**Returns**: `MaskingRule` with defaults for other fields

---

#### `with_mask_char`

```rust
pub fn with_mask_char(mut self, c: char) -> Self
```

**Purpose**: Builder method to set mask character.

**Parameters**: `c` - Mask character

**Returns**: `Self`

---

#### `with_preserve`

```rust
pub fn with_preserve(mut self, chars: Vec<char>) -> Self
```

**Purpose**: Builder method to set preserved characters.

**Parameters**: `chars` - Characters to preserve

**Returns**: `Self`

**Example**:
```rust
let rule = MaskingRule::new(1, 4)
    .with_mask_char('X')
    .with_preserve(vec!['@', '.']);
```

---

#### `apply`

```rust
pub fn apply(&self, text: &str) -> String
```

**Purpose**: Apply masking rule to text.

**Parameters**: `text` - Text to mask

**Returns**: Masked string

**Behavior**:
- If `text.len() <= show_first + show_last`, return unchanged
- Mask characters between first and last ranges
- Preserve characters in `preserve` list
- Use `mask_char` for masked positions

**Contract**:
- Output length equals input length (character count)
- Preserved characters remain in original positions

---

### `RedactionResult`

**Purpose**: Complete result of redaction operation.

**Fields**:
- `pub text: String` - Redacted text
- `pub redactions: Vec<AppliedRedaction>` - List of applied redactions
- `pub position_map: PositionMap` - Position mapping

**Methods**:

#### `new`

```rust
pub fn new(
    text: String,
    redactions: Vec<AppliedRedaction>,
    position_map: PositionMap,
) -> Self
```

**Purpose**: Construct result (typically used internally).

**Parameters**: All fields

**Returns**: `RedactionResult`

---

#### `redaction_count`

```rust
pub fn redaction_count(&self) -> usize
```

**Returns**: Number of redactions applied

---

#### `has_redactions`

```rust
pub fn has_redactions(&self) -> bool
```

**Returns**: `true` if any redactions were applied

---

### `AppliedRedaction`

**Purpose**: Record of a single applied redaction.

**Fields**:
- `pub original: String` - Original PII text
- `pub replacement: String` - Replacement text
- `pub original_position: (usize, usize)` - (start, end) in original
- `pub new_position: (usize, usize)` - (start, end) in redacted text
- `pub category: PiiCategory` - PII category

**Methods**:

#### `new`

```rust
pub fn new(
    original: impl Into<String>,
    replacement: impl Into<String>,
    original_position: (usize, usize),
    new_position: (usize, usize),
    category: PiiCategory,
) -> Self
```

**Purpose**: Construct applied redaction record.

**Parameters**: All fields

**Returns**: `AppliedRedaction`

---

### `PositionMap`

**Purpose**: Map original positions to redacted positions.

**Methods**:

#### `new`

```rust
pub fn new() -> Self
```

**Returns**: Empty position map

---

#### `add`

```rust
pub fn add(&mut self, entry: PositionMapEntry)
```

**Purpose**: Add entry to map (typically used internally).

**Parameters**: `entry` - Position mapping entry

---

#### `entries`

```rust
pub fn entries(&self) -> &[PositionMapEntry]
```

**Returns**: Slice of all position map entries

---

#### `map_position`

```rust
pub fn map_position(&self, original_pos: usize) -> Option<usize>
```

**Purpose**: Map original position to redacted position.

**Parameters**: `original_pos` - Position in original text

**Returns**:
- `Some(pos)` - Mapped position in redacted text
- `None` - Position mapping failed (should not happen for valid input)

**Behavior**:
- Positions before first redaction: offset by 0
- Positions within redaction: map to start of replacement
- Positions after redaction: offset by cumulative length change

**Example**:
```rust
let result = redact("Email: john@example.com", &findings);
// Original "john@example.com" at 7-24, becomes "[EMAIL]" at 7-14

assert_eq!(result.position_map.map_position(10), Some(7));  // Inside email → start of [EMAIL]
assert_eq!(result.position_map.map_position(30), Some(20)); // After email, offset by -10
```

---

### `PositionMapEntry`

**Purpose**: Single entry in position map.

**Fields**:
- `pub original_start: usize`
- `pub original_end: usize`
- `pub redacted_start: usize`
- `pub redacted_end: usize`

**No public constructor** (created internally by engine).

---

## Serialization

All public types implement `serde::Serialize` and `serde::Deserialize`.

**Formats supported**: JSON, YAML, TOML (via serde)

**Example** (JSON):

```rust
use serde_json;

let result = redact(text, &findings);
let json = serde_json::to_string(&result)?;
```

**JSON Schema** (RedactionResult):

```json
{
  "text": "string",
  "redactions": [
    {
      "original": "string",
      "replacement": "string",
      "original_position": [number, number],
      "new_position": [number, number],
      "category": "email" | "iban" | "phone" | ...
    }
  ],
  "position_map": {
    "entries": [
      {
        "original_start": number,
        "original_end": number,
        "redacted_start": number,
        "redacted_end": number
      }
    ]
  }
}
```

---

## Error Handling

**Current**: No errors returned (infallible operations).

**Contract**:
- Invalid positions: Undefined behavior (caller responsibility to validate)
- Empty findings: Returns original text unchanged
- Overlapping findings: Automatically resolved

**Future**: Consider `Result<RedactionResult, RedactError>` if validation is added.

---

## Thread Safety

- All types are `Send + Sync` (safe to share across threads)
- `RedactionEngine` is immutable after construction (safe concurrent use)
- No internal mutability or shared state

**Usage**:
```rust
let engine = Arc::new(RedactionEngine::new(config));

for document in documents {
    let engine = Arc::clone(&engine);
    thread::spawn(move || {
        let result = engine.redact(&document.text, &document.findings);
        // Process result
    });
}
```

---

## Performance Guarantees

- **redact()**: O(n² + nm) where n=findings, m=avg finding length
- **Memory**: O(text.len() + findings.len())
- **Target**: 10,000 findings in <1 second (SC-003)

**No allocations** except for result construction.

---

## Stability & Breaking Changes

**Alpha (0.1.x)**: Breaking changes possible in any release.

**Planned for 1.0**:
- Stable API (semver guarantees)
- Error handling review
- Performance benchmarks published

**Deprecation policy** (post-1.0):
- Deprecated items remain for at least 1 minor version
- Warnings in documentation and compiler

---

## Dependencies

| Crate | Version | Purpose | Exposed in API? |
|-------|---------|---------|-----------------|
| `veil-detect` | workspace | `Finding`, `PiiCategory` | Yes (public types) |
| `serde` | 1.0 | Serialization | Yes (traits) |
| `thiserror` | 1.0 | Error types | No (not yet used) |

**Re-exports**:
```rust
pub use veil_detect::Finding;      // For convenience
pub use veil_detect::PiiCategory;  // Required in public API
```

---

## Testing Contract

**Unit tests**: Validate individual methods (see `#[cfg(test)]` in source).

**Integration tests**: Validate end-to-end workflows (needed - see tasks.md).

**Public test helpers** (none currently):

Future consideration:
```rust
#[cfg(test)]
pub fn make_test_finding(text: &str, start: usize, category: PiiCategory) -> Finding;
```

---

## Compatibility

**Rust Version**: 1.75+ (2021 edition)

**Platforms**: All (no platform-specific code)

**WASM**: Compatible (no I/O, no threads)

**no_std**: Not supported (uses `std::collections::HashMap`, `String`)

---

## Examples

See `quickstart.md` for comprehensive examples.

**Minimal example**:

```rust
use veil_redact::redact;
use veil_detect::{Finding, PiiCategory, ValidationStatus};

let text = "Email: john@example.com";
let findings = vec![
    Finding::new("john@example.com", PiiCategory::Email, 7, 24, 1.0, ValidationStatus::Valid, 0)
];

let result = redact(text, &findings);
assert_eq!(result.text, "Email: [EMAIL]");
```

---

## Migration Guide

**From pre-0.1**: N/A (initial version)

**Future breaking changes**: Will be documented here.

---

## References

- Source: `D:\Projekte\Veil\crates\veil-redact\src\lib.rs`
- Documentation: Run `cargo doc --open` in veil-redact crate
- Spec: `D:\Projekte\Veil\specs\003-redaction-engine\spec.md`
