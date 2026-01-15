# Research: Dictionary Detection

**Feature**: 008-dictionary-detection
**Date**: 2025-12-15

## 1. Data Structure for Dictionary Storage

**Decision**: Use `fst` (Finite State Transducer) for primary dictionary storage

**Rationale**:
- O(1) lookup time regardless of dictionary size
- Memory-efficient: 100K entries in ~1-2MB
- Supports ordered iteration and prefix queries
- Rust-native, well-maintained crate
- Used by tantivy (search engine) proving production-readiness

**Alternatives Considered**:
- `HashMap`: Simple but higher memory usage (~10x for large sets)
- `HashSet`: No metadata storage (need frequency/weight)
- `BTreeSet`: O(log n) lookup, unnecessary ordering overhead
- `trie`: Good for prefix matching but custom implementation needed

**Configuration**:
```toml
[dependencies]
fst = "0.4"
```

## 2. Fuzzy Matching Algorithm

**Decision**: Use `strsim` crate with Jaro-Winkler distance for name matching

**Rationale**:
- Jaro-Winkler specifically designed for name matching
- Weights prefix matches higher (good for nicknames: Max → Maximilian)
- Score 0.0-1.0 maps directly to confidence threshold
- Lightweight, no-std compatible

**Alternatives Considered**:
- Levenshtein: Better for typos but doesn't favor prefix matches
- Soundex/Metaphone: Language-specific, poor for German names
- n-gram: More complex, overkill for single-word matching

**Configuration**:
```toml
[dependencies]
strsim = "0.11"
```

**Default Threshold**: 0.85 (catches single-character typos while avoiding false positives)

## 3. Dictionary File Format

**Decision**: Use line-delimited text files with optional frequency annotation

**Rationale**:
- Simple to create and edit manually
- Easy to version control
- Can be generated from various sources
- Optional frequency field for confidence weighting

**Format**:
```text
# Simple format (one entry per line)
Maximilian
Maria
Alexander

# With frequency (tab-separated)
Maximilian	0.85
Maria	0.92
Alexander	0.78
```

**Alternatives Considered**:
- CSV: Overkill for single-column data
- JSON: Verbose, harder to diff
- SQLite: Adds dependency, unnecessary for read-only data
- Binary format: Not human-editable

## 4. Built-in Dictionary Sources

**Decision**: Bundle curated DACH name/location lists derived from public data

**Rationale**:
- Austrian/German/Swiss focus per spec requirements
- Public statistics data is legally unencumbered
- Curated lists avoid noise from raw data dumps

**Sources**:
| Dictionary | Source | Approximate Size |
|------------|--------|------------------|
| First names (AT/DE/CH) | Public statistics offices | ~5,000 entries |
| Last names (DE) | Surname frequency statistics | ~10,000 entries |
| Cities (AT) | Statistik Austria | ~2,100 municipalities |
| Cities (DE) | Statistisches Bundesamt | ~11,000 municipalities |
| Cities (CH) | BFS Switzerland | ~2,200 municipalities |

**Total estimated**: ~30K entries, ~500KB uncompressed

## 5. Word Boundary Detection

**Decision**: Use Unicode word boundary detection via `unicode-segmentation`

**Rationale**:
- Handles German compound words correctly
- Proper handling of hyphens in names (Anna-Maria)
- Unicode-aware (handles ü, ö, ä, ß correctly)
- Rust standard for text segmentation

**Configuration**:
```toml
[dependencies]
unicode-segmentation = "1.10"
```

**Boundary Rules**:
- Match must start and end at word boundaries
- Hyphenated names treated as single unit
- Apostrophes handled (O'Brien, M'Baku)

## 6. Case and Unicode Normalization

**Decision**: NFD normalization + case folding for matching, preserve original in output

**Rationale**:
- NFD handles umlauts: "ü" → "u" + combining diaeresis
- Case folding for matching: "MÜLLER" matches "Müller"
- Original form preserved for accurate position reporting

**Implementation**:
```rust
use unicode_normalization::UnicodeNormalization;

fn normalize_for_matching(s: &str) -> String {
    s.nfd().collect::<String>().to_lowercase()
}
```

**Configuration**:
```toml
[dependencies]
unicode-normalization = "0.1"
```

## 7. Confidence Scoring

**Decision**: Composite score from frequency weight × match quality × context signals

**Rationale**:
- Frequency: Common names more likely to be actual names
- Match quality: Exact > fuzzy, higher threshold = higher confidence
- Context: Capitalization, surrounding words (Herr/Frau prefix)

**Formula**:
```
confidence = base_frequency × match_factor × context_bonus
```

Where:
- `base_frequency`: 0.5-1.0 from dictionary (default 0.7 if not provided)
- `match_factor`: 1.0 for exact, similarity_score for fuzzy
- `context_bonus`: 1.0-1.2 based on contextual clues

## 8. Hot Reload Strategy

**Decision**: File watcher with debounced reload, atomic swap of dictionary instances

**Rationale**:
- No downtime during dictionary updates
- Debounce prevents thrashing on rapid file changes
- Arc<Dictionary> allows lock-free reads during swap

**Implementation Pattern**:
```rust
// Dictionary wrapped in Arc for cheap cloning
let dict: Arc<Dictionary> = Arc::new(load_dictionary(path)?);

// On file change, build new dictionary then swap
let new_dict = Arc::new(load_dictionary(path)?);
// Atomic swap - old readers continue with old dict
```

**Dependencies**:
```toml
[dependencies]
notify = "6.0"  # File watching (optional, for daemon mode)
```

## 9. Integration with Detection Pipeline

**Decision**: Implement `Detector` trait, run after regex detectors in pipeline

**Rationale**:
- Consistent interface with existing detectors (002)
- Dictionary detection is slower than regex, run second
- Findings merge into common output format

**Pipeline Order**:
1. Regex detectors (email, IBAN, phone, credit card) - fast
2. Dictionary detectors (names, locations, companies) - slower
3. Results merged, deduplicated, sorted by position

## 10. Memory Budget

**Decision**: Lazy loading with configurable preload, target <100MB for all built-in

**Rationale**:
- Not all dictionaries needed for every scan
- Lazy loading reduces startup time
- 100MB budget per spec requirement SC-006

**Strategy**:
- Built-in dictionaries: Preloaded on first use, kept in memory
- Custom dictionaries: Loaded on demand, can be unloaded
- Large dictionaries (>10MB): Stream from disk with caching

## Summary of Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| fst | 0.4 | Dictionary storage |
| strsim | 0.11 | Fuzzy matching |
| unicode-segmentation | 1.10 | Word boundary detection |
| unicode-normalization | 0.1 | Case/Unicode normalization |
| notify | 6.0 | File watching (optional) |

## Open Questions Resolved

1. **Q: How to handle very large dictionaries (1M+ entries)?**
   A: FST handles efficiently; for extreme cases, use disk-backed FST with mmap

2. **Q: How to handle German compound nouns?**
   A: Word boundary detection + option to split compounds (future enhancement)

3. **Q: How to prevent false positives on common words?**
   A: Require word boundaries, use context signals, configurable confidence threshold

4. **Q: How to handle names that are also common words (e.g., "Rose")?**
   A: Lower confidence for ambiguous entries, context-aware boosting
