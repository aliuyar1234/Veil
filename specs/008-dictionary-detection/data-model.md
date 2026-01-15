# Data Model: Dictionary Detection

**Feature**: 008-dictionary-detection
**Date**: 2025-12-15

## Overview

This document defines the core data structures for dictionary-based PII detection.
The design integrates with the existing veil-detect crate (002).

## Core Entities

### DictionaryCategory

Categorizes the type of dictionary for appropriate handling.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DictionaryCategory {
    /// First names (given names)
    FirstName,
    /// Last names (family names)
    LastName,
    /// Full person names (first + last combined)
    PersonName,
    /// City/town names
    City,
    /// Street names
    Street,
    /// Company/organization names
    Company,
    /// Custom user-defined category
    Custom(String),
}
```

**Validation Rules**:
- Custom category names must be non-empty
- Custom category names should be lowercase with underscores

---

### Locale

Supported locales for dictionary selection.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// Austria
    At,
    /// Germany
    De,
    /// Switzerland
    Ch,
    /// Generic/International
    Generic,
}
```

---

### DictionaryEntry

A single entry in a dictionary with optional metadata.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryEntry {
    /// The dictionary term (normalized form for matching)
    pub term: String,

    /// Original form(s) as they appear in source data
    pub original_forms: Vec<String>,

    /// Frequency/commonality weight (0.0-1.0)
    /// Higher = more common = higher confidence when matched
    pub frequency: f32,

    /// Optional metadata (e.g., gender for names, population for cities)
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}
```

**Validation Rules**:
- `term` must be non-empty
- `frequency` must be in range [0.0, 1.0]
- `original_forms` should contain at least one entry

---

### Dictionary

A named collection of entries for a specific category and locale.

```rust
#[derive(Debug, Clone)]
pub struct Dictionary {
    /// Unique identifier for this dictionary
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Category of entries (names, cities, etc.)
    pub category: DictionaryCategory,

    /// Locale this dictionary targets
    pub locale: Locale,

    /// Whether this is a built-in or custom dictionary
    pub builtin: bool,

    /// Number of entries
    pub entry_count: usize,

    /// Internal FST for fast lookups (not serialized)
    #[serde(skip)]
    fst: fst::Set<Vec<u8>>,

    /// Entry metadata indexed by normalized term
    #[serde(skip)]
    entries: HashMap<String, DictionaryEntry>,
}
```

**Methods**:
```rust
impl Dictionary {
    /// Check if a term exists (exact match)
    pub fn contains(&self, term: &str) -> bool;

    /// Get entry details for a term
    pub fn get(&self, term: &str) -> Option<&DictionaryEntry>;

    /// Find fuzzy matches within threshold
    pub fn find_fuzzy(&self, term: &str, threshold: f64) -> Vec<FuzzyMatch>;

    /// Iterate all entries
    pub fn iter(&self) -> impl Iterator<Item = &DictionaryEntry>;
}
```

---

### FuzzyConfig

Configuration for fuzzy/approximate matching.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyConfig {
    /// Enable fuzzy matching
    pub enabled: bool,

    /// Similarity threshold (0.0-1.0)
    /// Higher = stricter matching
    #[serde(default = "default_threshold")]
    pub threshold: f64,

    /// Maximum edit distance for candidates (optimization)
    #[serde(default = "default_max_distance")]
    pub max_distance: usize,
}

fn default_threshold() -> f64 { 0.85 }
fn default_max_distance() -> usize { 2 }
```

**Validation Rules**:
- `threshold` must be in range [0.0, 1.0]
- `max_distance` should be 1-3 for reasonable performance

---

### FuzzyMatch

Result of a fuzzy match operation.

```rust
#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    /// The matched dictionary entry
    pub entry: DictionaryEntry,

    /// Similarity score (0.0-1.0)
    pub similarity: f64,

    /// The input term that was matched
    pub input: String,
}
```

---

### DictionaryMatch

A finding from dictionary detection (extends veil-detect Finding concept).

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryMatch {
    /// The matched text from the source document
    pub matched_text: String,

    /// Start position in source (byte offset)
    pub start: usize,

    /// End position in source (byte offset)
    pub end: usize,

    /// Dictionary category that matched
    pub category: DictionaryCategory,

    /// Locale of the matching dictionary
    pub locale: Locale,

    /// Dictionary ID that produced this match
    pub dictionary_id: String,

    /// The dictionary term that matched
    pub dictionary_term: String,

    /// Whether this was an exact or fuzzy match
    pub match_type: MatchType,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    Exact,
    Fuzzy { similarity: f64 },
}
```

---

### DictionaryDetectorConfig

Configuration for the dictionary detector.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryDetectorConfig {
    /// Enabled dictionary categories
    #[serde(default = "default_categories")]
    pub categories: Vec<DictionaryCategory>,

    /// Enabled locales
    #[serde(default = "default_locales")]
    pub locales: Vec<Locale>,

    /// Fuzzy matching configuration
    #[serde(default)]
    pub fuzzy: FuzzyConfig,

    /// Paths to custom dictionary files
    #[serde(default)]
    pub custom_dictionaries: Vec<PathBuf>,

    /// Minimum confidence threshold for reporting
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,

    /// Require word boundaries for matches
    #[serde(default = "default_require_boundaries")]
    pub require_word_boundaries: bool,
}

fn default_categories() -> Vec<DictionaryCategory> {
    vec![DictionaryCategory::FirstName, DictionaryCategory::LastName]
}

fn default_locales() -> Vec<Locale> {
    vec![Locale::At, Locale::De, Locale::Ch]
}

fn default_min_confidence() -> f32 { 0.5 }

fn default_require_boundaries() -> bool { true }
```

---

### DictionaryRegistry

Central registry managing all loaded dictionaries.

```rust
pub struct DictionaryRegistry {
    /// Loaded dictionaries indexed by ID
    dictionaries: HashMap<String, Arc<Dictionary>>,

    /// Index by category for fast lookup
    by_category: HashMap<DictionaryCategory, Vec<String>>,

    /// Index by locale
    by_locale: HashMap<Locale, Vec<String>>,
}
```

**Methods**:
```rust
impl DictionaryRegistry {
    /// Create empty registry
    pub fn new() -> Self;

    /// Create registry with built-in dictionaries
    pub fn with_builtins() -> Result<Self, DictionaryError>;

    /// Load dictionary from file
    pub fn load(&mut self, path: &Path) -> Result<String, DictionaryError>;

    /// Load dictionary from reader
    pub fn load_from_reader<R: BufRead>(
        &mut self,
        reader: R,
        config: DictionaryLoadConfig,
    ) -> Result<String, DictionaryError>;

    /// Unload a dictionary by ID
    pub fn unload(&mut self, id: &str) -> bool;

    /// Get dictionary by ID
    pub fn get(&self, id: &str) -> Option<Arc<Dictionary>>;

    /// Get all dictionaries for a category
    pub fn by_category(&self, category: DictionaryCategory) -> Vec<Arc<Dictionary>>;

    /// Get all dictionaries for a locale
    pub fn by_locale(&self, locale: Locale) -> Vec<Arc<Dictionary>>;

    /// Reload a dictionary from its source
    pub fn reload(&mut self, id: &str) -> Result<(), DictionaryError>;
}
```

---

### DictionaryError

Error types for dictionary operations.

```rust
#[derive(Debug, thiserror::Error)]
pub enum DictionaryError {
    #[error("Failed to read dictionary file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid dictionary format at line {line}: {message}")]
    ParseError { line: usize, message: String },

    #[error("Dictionary not found: {0}")]
    NotFound(String),

    #[error("Dictionary already loaded: {0}")]
    AlreadyLoaded(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("FST build error: {0}")]
    FstError(String),
}
```

---

## Entity Relationships

```
DictionaryRegistry
    │
    ├── Dictionary (1..n)
    │       │
    │       ├── DictionaryEntry (0..n)
    │       │       └── metadata: HashMap
    │       │
    │       ├── category: DictionaryCategory
    │       └── locale: Locale
    │
    └── by_category/by_locale indexes

DictionaryDetector
    │
    ├── config: DictionaryDetectorConfig
    │       │
    │       ├── fuzzy: FuzzyConfig
    │       └── custom_dictionaries: Vec<PathBuf>
    │
    ├── registry: DictionaryRegistry
    │
    └── produces: Vec<DictionaryMatch>
```

---

## State Transitions

### Dictionary Loading

```
┌─────────────┐
│ File Path   │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Parse Lines │ → [Error: ParseError]
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Build FST   │ → [Error: FstError]
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Register    │ → [Error: AlreadyLoaded]
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Dictionary  │ (ready for queries)
└─────────────┘
```

### Detection Flow

```
┌─────────────┐
│ Input Text  │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Tokenize    │ (word boundary detection)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Normalize   │ (case folding, NFD)
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────────┐
│ For each token:                      │
│   1. Exact lookup in all dicts      │
│   2. Fuzzy lookup if enabled        │
│   3. Calculate confidence           │
│   4. Filter by min_confidence       │
└──────┬──────────────────────────────┘
       │
       ▼
┌─────────────┐
│ Merge &     │
│ Deduplicate │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│ Vec<DictionaryMatch> │
└─────────────────┘
```

---

## Size Limits

| Entity | Limit | Rationale |
|--------|-------|-----------|
| Dictionary entries | 1,000,000 | FST handles efficiently |
| Term length | 100 chars | Reasonable for names/places |
| Custom dictionaries | 100 | Prevent registry bloat |
| Metadata per entry | 10 keys | Keep entries lightweight |
| Total memory | 100 MB | Per spec SC-006 |

---

## Integration with veil-detect

The `DictionaryDetector` implements the existing `Detector` trait:

```rust
impl Detector for DictionaryDetector {
    fn name(&self) -> &str {
        "dictionary"
    }

    fn category(&self) -> PiiCategory {
        // Returns based on match type
        PiiCategory::Custom("dictionary".to_string())
    }

    fn detect(&self, text: &str) -> Vec<Match> {
        // Convert DictionaryMatch to Match
    }

    fn validate(&self, _matched: &str) -> ValidationStatus {
        ValidationStatus::Unvalidated
    }

    fn base_confidence(&self) -> f32 {
        0.7 // Overridden per-match
    }
}
```

This allows dictionary detection to plug into the existing `DetectorRegistry` pipeline.
