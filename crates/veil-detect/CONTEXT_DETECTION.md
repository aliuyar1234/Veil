# Context Detection Feature

## Overview

The context detection feature enhances PII detection accuracy by analyzing surrounding text for contextual markers like honorifics, labels, and suppression patterns.

## Features

### 1. Context-Aware Confidence Adjustment
- **Boost**: Increases confidence when context confirms PII (e.g., "Email:" before an email address)
- **Suppress**: Decreases confidence when context indicates non-PII (e.g., "version" before an IP-like pattern)

### 2. Multi-Language Support
Built-in rules for:
- **English (en)**: Mr., Dr., Email:, Phone:, etc.
- **German (de)**: Herr, Frau, E-Mail:, Telefon:, etc.
- **French (fr)**: Monsieur, Madame, Téléphone:, etc.

### 3. Context Marker Types
- **Honorifics**: Mr., Dr., Herr, Frau, Monsieur, Madame
- **Labels**: Email:, Phone:, Name:, Address:
- **Suppression**: version, order #, ISBN, SKU
- **Table Headers**: Column names in structured data
- **Address Components**: Street, Avenue, Straße, Rue

## Usage

### Basic Usage

```rust
use veil_detect::DetectorRegistry;
use veil_types::{Position, TextSegment};

// Create registry and enable context analysis
let mut registry = DetectorRegistry::default();
registry.enable_context_analysis();

// Create text segment
let segment = TextSegment {
    content: "Contact: Mr. Smith at smith@example.com".to_string().into(),
    position: Position::Text {
        line: 1,
        column: 1,
        byte_offset: 0,
        byte_length: "Contact: Mr. Smith at smith@example.com".len(),
    },
};

// Detect with context
let findings = registry.detect_all_with_context(&[segment], Some("en"));

// Check context reasoning
for finding in findings {
    if let Some(reasoning) = &finding.context_reasoning {
        for reason in reasoning {
            println!("Context: {}", reason);
        }
    }
}
```

### Language-Specific Analysis

```rust
use veil_detect::ContextAnalyzer;

// Create analyzer for specific language
let mut analyzer = ContextAnalyzer::with_language("de");

// Or use default (English) and specify language per-analysis
let mut analyzer = ContextAnalyzer::new();
let text = "Sehr geehrter Herr Müller";
let markers = analyzer.detect_markers(text, Some("de"));
```

### Custom Context Rules

```rust
use veil_detect::context::{ContextAnalyzer, ContextRule, ContextAction};
use veil_detect::PiiCategory;

let mut analyzer = ContextAnalyzer::empty();

// Add custom boost rule
let rule = ContextRule::new(
    r"(?i)\bcustomer\s+name:\s*",
    ContextAction::Boost,
    0.4,
)
.with_language("en")
.with_category(PiiCategory::Custom("PersonName".to_string()))
.with_description("Custom customer name label");

analyzer.add_rule(rule);
```

## Architecture

### Core Components

1. **ContextRule**: Defines patterns and actions
   - Pattern: Regex to match contextual markers
   - Action: Boost, Suppress, or Neutral
   - Weight: Adjustment strength (0.0 - 1.0)
   - Category: Optional PII category filter
   - Language: Optional language filter

2. **ContextMarker**: Detected context indicator
   - Type: Honorific, Label, Suppression, etc.
   - Position: Location in text
   - Action: What to do with nearby findings
   - Weight: How strongly to adjust

3. **ContextAnalyzer**: Main analysis engine
   - Detects markers using compiled regexes
   - Calculates confidence adjustments
   - Provides reasoning for adjustments

4. **ContextAnalysis**: Result of analysis
   - Original confidence
   - Adjusted confidence
   - Markers that influenced adjustment
   - Human-readable reasoning

### Integration with DetectorRegistry

The `DetectorRegistry` can optionally use context analysis:

```rust
// Enable with default analyzer
registry.enable_context_analysis();

// Or use custom analyzer
let analyzer = ContextAnalyzer::with_language("de");
registry.set_context_analyzer(analyzer);

// Detect with context
let findings = registry.detect_all_with_context(&segments, Some("de"));
```

## Built-in Rules

### English (en)
- **Boost**: Mr., Mrs., Ms., Dr., Prof., Dear, Email:, Phone:, Address:
- **Suppress**: version, order #, ISBN, SKU, product code

### German (de)
- **Boost**: Herr, Frau, Dr., Prof., E-Mail:, Telefon:, Adresse:
- **Suppress**: Version, Bestellung, Auftrag

### French (fr)
- **Boost**: Monsieur, Madame, M., Mme, Dr., E-mail:, Téléphone:
- **Suppress**: version, commande

## Confidence Adjustment Algorithm

1. **Boost (Additive)**: confidence += weight (capped at 1.0)
2. **Suppress (Multiplicative)**: confidence *= (1 - weight)
3. **Context Window**: 200 characters before/after finding

### Example

Original confidence: 0.7

With boost (weight=0.3):
- Adjusted = min(0.7 + 0.3, 1.0) = 1.0

With suppress (weight=0.6):
- Adjusted = 0.7 * (1 - 0.6) = 0.28

## Performance

- **Overhead**: <10% additional processing time
- **Context Window**: 200 characters (configurable)
- **Regex Compilation**: Cached using once_cell
- **Memory**: Minimal - rules compiled once at initialization

## Testing

Run tests:
```bash
cargo test -p veil-detect context
cargo test -p veil-detect --test context_detection_tests
```

Run example:
```bash
cargo run --example context_detection -p veil-detect
```

## Future Enhancements

- Table structure detection (CSV column headers)
- Multi-line address detection
- Configurable YAML rule loading
- Language auto-detection
- Machine learning-based context analysis
