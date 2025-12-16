# Veil Dictionary Data

This directory contains dictionary files for PII detection.

## File Format

Each dictionary file is a simple text file with one entry per line:

```text
# Comments start with #
EntryName
EntryWithFrequency	0.85
```

- Lines starting with `#` are comments
- Empty lines are skipped
- Tab-separated frequency (0.0-1.0) is optional

## Available Dictionaries

| File | Category | Locale | Entries | Description |
|------|----------|--------|---------|-------------|
| `firstnames_de.txt` | FirstName | DE | ~30 | Common German first names |
| `lastnames_de.txt` | LastName | DE | ~30 | Common German surnames |
| `cities_de.txt` | City | DE | ~30 | Major German cities |
| `cities_at.txt` | City | AT | ~30 | Major Austrian cities |

## Sources

- First names: Common German first names from public statistics
- Last names: Common German surnames from public statistics
- Cities: Major municipalities from official statistics

## License

Dictionary data is compiled from publicly available statistics and is provided under the same license as the Veil project (MIT OR Apache-2.0).

## Adding Custom Dictionaries

To add a custom dictionary:

1. Create a text file with one entry per line
2. Optionally add tab-separated frequency values
3. Load using the `DictionaryRegistry::load()` method

```rust
use veil_detect::dictionary::{DictionaryRegistry, DictionaryLoadConfig, DictionaryCategory, Locale};

let mut registry = DictionaryRegistry::new();
let config = DictionaryLoadConfig::new(
    DictionaryCategory::Custom("my_category".to_string()),
    Locale::Generic,
);
registry.load(Path::new("my_dictionary.txt"), config)?;
```
