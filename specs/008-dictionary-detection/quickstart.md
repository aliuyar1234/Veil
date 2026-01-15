# Quickstart: Dictionary Detection

**Feature**: 008-dictionary-detection

## Overview

Dictionary detection extends veil-detect with name, location, and company detection using
locale-specific dictionaries. This enables detection of PII that cannot be caught by regex patterns.

## Basic Usage

### Rust API

```rust
use veil_detect::{DetectorRegistry, DictionaryDetectorConfig, Locale, DictionaryCategory};

// Create registry with dictionary detection enabled
let mut registry = DetectorRegistry::default();

// Configure dictionary detector
let dict_config = DictionaryDetectorConfig {
    categories: vec![DictionaryCategory::FirstName, DictionaryCategory::LastName],
    locales: vec![Locale::At, Locale::De],
    fuzzy: FuzzyConfig {
        enabled: true,
        threshold: 0.85,
        ..Default::default()
    },
    ..Default::default()
};

// Add dictionary detector
registry.register(Box::new(DictionaryDetector::new(dict_config)?));

// Detect PII in text
let text = "Kontaktperson: Max Mustermann aus Wien";
let segments = vec![TextSegment::from_str(text)];
let findings = registry.detect_all(&segments);

for finding in findings {
    println!("{}: {} (confidence: {})",
        finding.category,
        finding.matched_text,
        finding.confidence
    );
}
// Output:
// first_name: Max (confidence: 0.92)
// last_name: Mustermann (confidence: 0.88)
// city: Wien (confidence: 0.95)
```

### CLI Usage

```bash
# Scan with dictionary detection enabled (default)
veil scan document.txt

# Scan with specific categories
veil scan --categories email,iban,first_name,last_name document.txt

# Scan with specific locales
veil scan --locales at,de document.txt

# Disable fuzzy matching
veil scan --no-fuzzy document.txt

# Set fuzzy threshold
veil scan --fuzzy-threshold 0.9 document.txt
```

## Custom Dictionaries

### File Format

Simple text file with one entry per line:

```text
# comments start with #
Projektname1
Projektname2
KundenID123
```

With frequency weights (tab-separated):

```text
# term<TAB>frequency
Mustermann	0.95
Maier	0.90
Huber	0.85
```

### Loading Custom Dictionaries

```rust
use veil_detect::{DictionaryRegistry, DictionaryLoadConfig, DictionaryCategory, Locale};

let mut registry = DictionaryRegistry::with_builtins()?;

// Load custom dictionary
let config = DictionaryLoadConfig {
    category: DictionaryCategory::Custom("project_names".to_string()),
    locale: Locale::Generic,
    ..Default::default()
};

registry.load_from_file("custom_names.txt", config)?;
```

### CLI with Custom Dictionaries

```bash
# Load custom dictionary
veil scan --dictionary ./my_names.txt:custom_names document.txt

# Multiple custom dictionaries
veil scan \
  --dictionary ./employees.txt:employee_names \
  --dictionary ./clients.txt:client_names \
  document.txt
```

## Fuzzy Matching

Fuzzy matching catches typos and name variations.

### Examples

| Dictionary Entry | Input | Similarity | Detected? (0.85 threshold) |
|-----------------|-------|------------|---------------------------|
| Maximilian | Maximilian | 1.00 | ✅ Yes |
| Maximilian | Maximilain | 0.96 | ✅ Yes |
| Maximilian | Max | 0.78 | ❌ No |
| Maximilian | Maxi | 0.81 | ❌ No |
| Müller | Mueller | 0.91 | ✅ Yes |
| Müller | Muller | 0.89 | ✅ Yes |

### Configuration

```rust
let fuzzy_config = FuzzyConfig {
    enabled: true,
    threshold: 0.85,  // 0.0-1.0, higher = stricter
    max_distance: 2,  // Max edit distance for candidates
};
```

## Detection Categories

### Built-in Dictionaries

| Category | Locales | Entries | Description |
|----------|---------|---------|-------------|
| `first_name` | AT, DE, CH | ~5,000 | Common first names |
| `last_name` | DE | ~10,000 | Common surnames |
| `city` | AT, DE, CH | ~15,000 | Municipalities |

### Category Mapping to PiiCategory

```rust
// Dictionary matches map to PiiCategory::Custom
match dictionary_match.category {
    DictionaryCategory::FirstName => PiiCategory::Custom("first_name".into()),
    DictionaryCategory::LastName => PiiCategory::Custom("last_name".into()),
    DictionaryCategory::City => PiiCategory::Custom("city".into()),
    DictionaryCategory::Company => PiiCategory::Custom("company".into()),
    DictionaryCategory::Custom(name) => PiiCategory::Custom(name),
    // ...
}
```

## Confidence Scoring

Confidence is calculated from multiple factors:

```
confidence = base_frequency × match_factor × context_bonus
```

| Factor | Range | Description |
|--------|-------|-------------|
| `base_frequency` | 0.5-1.0 | How common the name is |
| `match_factor` | 0.0-1.0 | Exact (1.0) or fuzzy similarity |
| `context_bonus` | 1.0-1.2 | Contextual signals (Herr/Frau prefix) |

### Examples

| Match | Frequency | Match Type | Context | Final Confidence |
|-------|-----------|------------|---------|------------------|
| "Maria" | 0.95 | Exact | None | 0.95 |
| "Maria" | 0.95 | Exact | "Frau Maria" | 1.0 (capped) |
| "Maximilain" | 0.80 | Fuzzy (0.96) | None | 0.77 |

## Word Boundaries

By default, matches require word boundaries to avoid false positives.

| Text | Dictionary | Match? | Reason |
|------|-----------|--------|--------|
| "Max is here" | Max | ✅ Yes | Word boundaries |
| "Maximum value" | Max | ❌ No | Substring of "Maximum" |
| "Anna-Maria" | Anna | ✅ Yes | Hyphen is boundary |
| "email@maria.com" | Maria | ❌ No | Part of domain |

Disable with:

```rust
let config = DictionaryDetectorConfig {
    require_word_boundaries: false,  // Not recommended
    ..Default::default()
};
```

## Integration with Protection

Dictionary findings integrate with veil-redact:

```rust
use veil_detect::DetectorRegistry;
use veil_redact::{redact_with_style, RedactionStyle};

let registry = DetectorRegistry::default(); // Includes dictionary detector
let findings = registry.detect_all(&segments);

// Redact all findings including dictionary matches
let result = redact_with_style(&text, &findings, RedactionStyle::Label);
// "Kontaktperson: [FIRST_NAME] [LAST_NAME] aus [CITY]"
```

## Performance Tips

1. **Limit locales**: Only enable locales you need
2. **Disable fuzzy for speed**: Exact matching is 10x faster
3. **Increase threshold**: Higher threshold = fewer candidates to check
4. **Preload dictionaries**: Load at startup, not per-document

```rust
// Fast configuration
let config = DictionaryDetectorConfig {
    locales: vec![Locale::At],  // Single locale
    fuzzy: FuzzyConfig { enabled: false, ..Default::default() },
    ..Default::default()
};
```

## Troubleshooting

### No matches found

1. Check locale is enabled: `--locales at,de,ch`
2. Check category is enabled: `--categories first_name,last_name`
3. Verify word boundaries aren't blocking: try `--no-word-boundaries`
4. Lower confidence threshold: `--min-confidence 0.3`

### Too many false positives

1. Raise confidence threshold: `--min-confidence 0.8`
2. Disable fuzzy matching: `--no-fuzzy`
3. Ensure word boundaries enabled (default)

### Memory usage high

1. Reduce enabled locales
2. Disable categories you don't need
3. Use file-backed dictionaries for very large custom lists
