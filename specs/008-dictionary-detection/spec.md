# Feature Specification: Dictionary Detection

**Feature Branch**: `008-dictionary-detection`
**Created**: 2025-12-08
**Status**: Draft
**Input**: Dictionary-based PII detection for names, cities, and custom lists

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Detect Person Names (Priority: P1)

A privacy analyst scans documents for person names. The system uses locale-specific name dictionaries (first names, last names) to detect potential names with fuzzy matching support.

**Why this priority**: Names are fundamental PII but cannot be detected with regex patterns alone.

**Independent Test**: Provide text with known names from dictionary, verify detection with expected confidence.

**Acceptance Scenarios**:

1. **Given** text containing "Max Mustermann", **When** scanned with DE locale, **Then** detected as "person_name".
2. **Given** text with common name "Maria", **When** scanned, **Then** detected with confidence based on context.
3. **Given** uncommon name not in dictionary, **When** scanned, **Then** not detected (no false positives on random words).

---

### User Story 2 - Detect Location Names (Priority: P2)

A compliance team needs to identify location data (cities, streets) in documents. The system uses geographic dictionaries to detect Austrian, German, and Swiss location names.

**Why this priority**: Addresses and locations are PII under GDPR when they can identify individuals.

**Independent Test**: Provide text with city names, verify detection with locale information.

**Acceptance Scenarios**:

1. **Given** text "Wohnhaft in Wien", **When** scanned, **Then** "Wien" detected as "city" with locale AT.
2. **Given** street name "Mariahilfer Straße", **When** scanned with street dictionary, **Then** detected as "street".
3. **Given** ambiguous word (city name that's also common word), **When** scanned, **Then** confidence reflects ambiguity.

---

### User Story 3 - Detect Company Names (Priority: P2)

An analyst scans for company names that could indicate business relationships or employment information. The system uses a company name dictionary with legal form patterns (GmbH, AG, etc.).

**Why this priority**: Company associations can be sensitive; company names have recognizable patterns.

**Independent Test**: Provide text with company names including legal forms, verify detection.

**Acceptance Scenarios**:

1. **Given** "Mitarbeiter bei Siemens AG", **When** scanned, **Then** "Siemens AG" detected as "company".
2. **Given** company with "GmbH" suffix, **When** scanned, **Then** detected with legal form indicator.
3. **Given** company name without legal form, **When** in dictionary, **Then** detected with lower confidence.

---

### User Story 4 - Use Custom Dictionaries (Priority: P1)

An organization maintains internal lists (employee names, client names, project codes) that should be detected as PII. The system supports loading custom dictionaries at runtime.

**Why this priority**: Organizations have domain-specific PII that generic dictionaries cannot cover.

**Independent Test**: Load custom dictionary, scan text with entries from that dictionary, verify detection.

**Acceptance Scenarios**:

1. **Given** custom dictionary with "ProjectX", **When** text contains "ProjectX", **Then** detected as custom category.
2. **Given** CSV file with names, **When** loaded as dictionary, **Then** all entries become detectable.
3. **Given** dictionary update, **When** reloaded, **Then** new entries immediately detectable.

---

### User Story 5 - Handle Name Variations (Priority: P2)

A privacy analyst needs to detect name variations (nicknames, abbreviations, common misspellings). The system supports fuzzy matching with configurable similarity thresholds.

**Why this priority**: Real-world data contains variations that exact matching would miss.

**Independent Test**: Provide text with name variations, verify fuzzy matching catches them.

**Acceptance Scenarios**:

1. **Given** "Maxi" when "Maximilian" in dictionary, **When** fuzzy matching enabled, **Then** detected as potential match.
2. **Given** typo "Maximilain", **When** fuzzy matching with threshold 0.8, **Then** detected as likely "Maximilian".
3. **Given** fuzzy matching disabled, **When** scanning variations, **Then** only exact matches detected.

---

### Edge Cases

- What happens with very large dictionaries (1M+ entries)? System uses efficient data structures for O(1) or O(log n) lookup.
- What happens when dictionary entry appears as substring? System requires word boundaries by default.
- What happens with case variations? System normalizes case for matching, preserves original in findings.
- What happens with special characters in names? System handles Unicode normalization (e.g., ü vs ue).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support first name dictionaries for AT, DE, CH locales.
- **FR-002**: System MUST support last name dictionaries for AT, DE, CH locales.
- **FR-003**: System MUST support city/town dictionaries for AT, DE, CH.
- **FR-004**: System MUST support street name dictionaries (optional, loadable).
- **FR-005**: System MUST support company name dictionaries with legal form recognition.
- **FR-006**: System MUST support custom dictionaries loaded from files (one entry per line or CSV).
- **FR-007**: System MUST provide configurable fuzzy matching with similarity threshold.
- **FR-008**: System MUST require word boundaries for matches (no substring matches by default).
- **FR-009**: System MUST normalize case and Unicode for matching.
- **FR-010**: System MUST report confidence score based on dictionary frequency/commonality.
- **FR-011**: System MUST integrate with detection engine (Spec 002) output format.
- **FR-012**: System MUST support dictionary hot-reload without restart.

### Key Entities

- **Dictionary**: A named list of terms; has locale, category (name, city, etc.), and entries.
- **DictionaryEntry**: A single dictionary term; contains normalized form, original forms, frequency/weight.
- **DictionaryMatch**: A finding from dictionary detection; includes matched term, dictionary source, confidence.
- **FuzzyMatcher**: Configuration for approximate matching; includes algorithm (Levenshtein, etc.) and threshold.
- **DictionaryConfig**: Settings for dictionary detection; specifies enabled dictionaries, custom paths, fuzzy settings.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Common DACH first/last names are detected with 90% recall.
- **SC-002**: Dictionary lookup completes in under 1ms per word for dictionaries up to 100K entries.
- **SC-003**: Fuzzy matching with 0.85 threshold catches 80% of single-character typos.
- **SC-004**: Custom dictionaries are loaded and active within 1 second for files up to 10K entries.
- **SC-005**: Word boundary enforcement eliminates 99% of false substring matches.
- **SC-006**: Memory usage for all built-in dictionaries is under 100MB.

## Assumptions

- Built-in dictionaries are bundled with the application; custom dictionaries are user-provided.
- Dictionary detection runs after regex detection; results are merged in the detection pipeline.
- Frequency data in dictionaries helps rank confidence (common names = higher confidence of being names).
- The system does not include comprehensive dictionaries for all languages; DACH focus for v1.
