# Tasks: Dictionary Detection

**Input**: Design documents from `/specs/008-dictionary-detection/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

**Tests**: Following TDD per constitution - tests written before implementation.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure) ✅

**Purpose**: Project initialization and dependency setup

- [x] T001 Add dependencies (fst, strsim, unicode-segmentation, unicode-normalization) to crates/veil-detect/Cargo.toml
- [x] T002 Create dictionary module structure in crates/veil-detect/src/dictionary/mod.rs
- [x] T003 [P] Create error types in crates/veil-detect/src/dictionary/error.rs
- [x] T004 [P] Create DictionaryCategory and Locale enums in crates/veil-detect/src/dictionary/category.rs

---

## Phase 2: Foundational (Blocking Prerequisites) ✅

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Implement DictionaryEntry struct in crates/veil-detect/src/dictionary/entry.rs
- [x] T006 Implement Dictionary struct with FST storage in crates/veil-detect/src/dictionary/dictionary.rs
- [x] T007 Implement Dictionary::contains() for exact lookup in crates/veil-detect/src/dictionary/dictionary.rs
- [x] T008 Implement Dictionary::get() to retrieve entry details in crates/veil-detect/src/dictionary/dictionary.rs
- [x] T009 Implement DictionaryRegistry struct in crates/veil-detect/src/dictionary/registry.rs
- [x] T010 Implement DictionaryRegistry::register() and ::get() in crates/veil-detect/src/dictionary/registry.rs
- [x] T011 Implement normalize_for_matching() (NFD + lowercase) in crates/veil-detect/src/dictionary/normalize.rs
- [x] T012 [P] Write unit tests for normalization in crates/veil-detect/src/dictionary/normalize.rs
- [x] T013 Export dictionary module from crates/veil-detect/src/lib.rs

**Checkpoint**: Foundation ready - user story implementation can now begin ✅

---

## Phase 3: User Story 1 - Detect Person Names (Priority: P1) ✅

**Goal**: Detect first names and last names using locale-specific dictionaries (AT/DE/CH)

**Independent Test**: Provide text with known names from dictionary, verify detection with expected confidence

### Tests for User Story 1

- [x] T014 [P] [US1] Write test for exact first name detection in crates/veil-detect/src/dictionary/tests/name_detection.rs
- [x] T015 [P] [US1] Write test for exact last name detection in crates/veil-detect/src/dictionary/tests/name_detection.rs
- [x] T016 [P] [US1] Write test for name not in dictionary (no false positives) in crates/veil-detect/src/dictionary/tests/name_detection.rs

### Implementation for User Story 1

- [x] T017 [US1] Implement DictionaryDetectorConfig struct in crates/veil-detect/src/dictionary/config.rs
- [x] T018 [US1] Implement DictionaryDetector struct in crates/veil-detect/src/dictionary/detector.rs
- [x] T019 [US1] Implement Detector trait for DictionaryDetector in crates/veil-detect/src/dictionary/detector.rs
- [x] T020 [US1] Implement word boundary detection using unicode-segmentation in crates/veil-detect/src/dictionary/boundaries.rs
- [x] T021 [US1] Implement confidence scoring (frequency x match_factor x context_bonus) in crates/veil-detect/src/dictionary/confidence.rs
- [x] T022 [US1] Create sample first names dictionary data/dictionaries/firstnames_de.txt (100 entries for testing)
- [x] T023 [US1] Create sample last names dictionary data/dictionaries/lastnames_de.txt (100 entries for testing)
- [x] T024 [US1] Wire DictionaryDetector into DetectorRegistry in crates/veil-detect/src/registry.rs

**Checkpoint**: Person name detection should work for DE locale with sample dictionaries ✅

---

## Phase 4: User Story 4 - Use Custom Dictionaries (Priority: P1) ✅

**Goal**: Load custom dictionaries from files at runtime

**Independent Test**: Load custom dictionary, scan text with entries from that dictionary, verify detection

### Tests for User Story 4

- [x] T025 [P] [US4] Write test for loading simple line-delimited dictionary in crates/veil-detect/src/dictionary/tests/loader_tests.rs
- [x] T026 [P] [US4] Write test for loading dictionary with frequency weights in crates/veil-detect/src/dictionary/tests/loader_tests.rs
- [x] T027 [P] [US4] Write test for custom category detection in crates/veil-detect/src/dictionary/tests/loader_tests.rs

### Implementation for User Story 4

- [x] T028 [US4] Implement DictionaryLoadConfig struct in crates/veil-detect/src/dictionary/loader.rs
- [x] T029 [US4] Implement load_from_file() for line-delimited format in crates/veil-detect/src/dictionary/loader.rs
- [x] T030 [US4] Implement load_from_file() for tab-separated format with frequencies in crates/veil-detect/src/dictionary/loader.rs
- [x] T031 [US4] Implement DictionaryRegistry::load() to register custom dictionaries in crates/veil-detect/src/dictionary/registry.rs
- [x] T032 [US4] Implement DictionaryRegistry::unload() to remove dictionaries in crates/veil-detect/src/dictionary/registry.rs
- [x] T033 [US4] Implement DictionaryRegistry::reload() for hot-reload support in crates/veil-detect/src/dictionary/registry.rs

**Checkpoint**: Custom dictionaries can be loaded and used for detection ✅

---

## Phase 5: User Story 5 - Handle Name Variations (Priority: P2) ✅

**Goal**: Detect name variations using fuzzy matching with configurable thresholds

**Independent Test**: Provide text with name variations, verify fuzzy matching catches them

### Tests for User Story 5

- [x] T034 [P] [US5] Write test for fuzzy match with single character typo in crates/veil-detect/src/dictionary/tests/fuzzy_tests.rs
- [x] T035 [P] [US5] Write test for threshold filtering (below threshold = no match) in crates/veil-detect/src/dictionary/tests/fuzzy_tests.rs
- [x] T036 [P] [US5] Write test for fuzzy matching disabled in crates/veil-detect/src/dictionary/tests/fuzzy_tests.rs

### Implementation for User Story 5

- [x] T037 [US5] Implement FuzzyConfig struct in crates/veil-detect/src/dictionary/fuzzy.rs
- [x] T038 [US5] Implement FuzzyMatch result struct in crates/veil-detect/src/dictionary/fuzzy.rs
- [x] T039 [US5] Implement jaro_winkler_similarity() wrapper using strsim in crates/veil-detect/src/dictionary/fuzzy.rs
- [x] T040 [US5] Implement Dictionary::find_fuzzy() with candidate generation in crates/veil-detect/src/dictionary/dictionary.rs
- [x] T041 [US5] Integrate fuzzy matching into DictionaryDetector in crates/veil-detect/src/dictionary/detector.rs
- [x] T042 [US5] Add fuzzy match confidence adjustment (similarity score as factor) in crates/veil-detect/src/dictionary/confidence.rs

**Checkpoint**: Fuzzy matching catches typos and variations within threshold ✅

---

## Phase 6: User Story 2 - Detect Location Names (Priority: P2) ✅

**Goal**: Detect city and street names using geographic dictionaries (AT/DE/CH)

**Independent Test**: Provide text with city names, verify detection with locale information

### Tests for User Story 2

- [x] T043 [P] [US2] Write test for Austrian city detection in crates/veil-detect/src/dictionary/tests/location_tests.rs
- [x] T044 [P] [US2] Write test for German city detection in crates/veil-detect/src/dictionary/tests/location_tests.rs
- [x] T045 [P] [US2] Write test for ambiguous word (city that is also common word) in crates/veil-detect/src/dictionary/tests/location_tests.rs

### Implementation for User Story 2

- [x] T046 [US2] Create sample cities dictionary data/dictionaries/cities_at.txt (100 entries for testing)
- [x] T047 [US2] Create sample cities dictionary data/dictionaries/cities_de.txt (100 entries for testing)
- [x] T048 [US2] Add City category support to DictionaryDetector in crates/veil-detect/src/dictionary/detector.rs
- [x] T049 [US2] Add locale-based confidence adjustment (AT city in AT context = higher) in crates/veil-detect/src/dictionary/confidence.rs

**Checkpoint**: Location detection works for AT and DE cities ✅

---

## Phase 7: User Story 3 - Detect Company Names (Priority: P2) ✅

**Goal**: Detect company names with legal form patterns (GmbH, AG, etc.)

**Independent Test**: Provide text with company names including legal forms, verify detection

### Tests for User Story 3

- [x] T050 [P] [US3] Write test for company with AG suffix in crates/veil-detect/src/dictionary/tests/company_tests.rs
- [x] T051 [P] [US3] Write test for company with GmbH suffix in crates/veil-detect/src/dictionary/tests/company_tests.rs
- [x] T052 [P] [US3] Write test for company without legal form (lower confidence) in crates/veil-detect/src/dictionary/tests/company_tests.rs

### Implementation for User Story 3

- [x] T053 [US3] Create sample companies dictionary data/dictionaries/companies_dach.txt (50 entries for testing)
- [x] T054 [US3] Add Company category support to DictionaryDetector in crates/veil-detect/src/dictionary/detector.rs
- [x] T055 [US3] Implement legal form pattern recognition (GmbH, AG, KG, etc.) in crates/veil-detect/src/dictionary/legal_forms.rs
- [x] T056 [US3] Add legal form context boost to confidence scoring in crates/veil-detect/src/dictionary/confidence.rs

**Checkpoint**: Company detection works with legal form recognition ✅

---

## Phase 8: Built-in Dictionaries ✅

**Purpose**: Expand sample dictionaries to production-ready built-in dictionaries

- [x] T057 Expand firstnames_de.txt to ~2000 common German first names in data/dictionaries/firstnames_de.txt
- [x] T058 [P] Create firstnames_at.txt with ~500 Austrian-specific names in data/dictionaries/firstnames_at.txt
- [x] T059 [P] Create firstnames_ch.txt with ~500 Swiss-specific names in data/dictionaries/firstnames_ch.txt
- [x] T060 Expand lastnames_de.txt to ~5000 common German surnames in data/dictionaries/lastnames_de.txt
- [x] T061 Expand cities_at.txt to all Austrian municipalities (~2100) in data/dictionaries/cities_at.txt
- [x] T062 [P] Expand cities_de.txt to major German cities (~1000) in data/dictionaries/cities_de.txt
- [x] T063 [P] Create cities_ch.txt with Swiss municipalities (~500) in data/dictionaries/cities_ch.txt
- [x] T064 Implement lazy loading for built-in dictionaries in crates/veil-detect/src/dictionary/builtins.rs
- [x] T065 Embed dictionaries as compile-time resources using include_str! in crates/veil-detect/src/dictionary/builtins.rs
- [x] T066 Create data/dictionaries/README.md documenting sources and licenses

**Checkpoint**: All built-in dictionaries loaded and working ✅

---

## Phase 9: Polish & Cross-Cutting Concerns ✅

**Purpose**: Documentation, performance validation, and final integration

- [x] T067 [P] Add documentation comments to all public API items in crates/veil-detect/src/dictionary/
- [x] T068 [P] Write integration test scanning full document with all detector types in crates/veil-detect/tests/integration_dictionary.rs
- [x] T069 Create benchmark for dictionary lookup performance in crates/veil-detect/benches/dictionary_bench.rs
- [x] T070 Validate memory usage (<100MB for all built-ins) in crates/veil-detect/benches/dictionary_bench.rs
- [x] T071 Run clippy and fix any warnings in crates/veil-detect/src/dictionary/
- [x] T072 Run cargo fmt on all dictionary module files
- [x] T073 Update veil-detect README with dictionary detection examples in crates/veil-detect/README.md
- [x] T074 Run quickstart.md validation scenarios manually

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1 (Person Names) + US4 (Custom Dictionaries): Can proceed in parallel after Foundational
  - US5 (Fuzzy Matching): Depends on US1 for testing name variations
  - US2 (Locations): Can proceed after Foundational (independent of US1)
  - US3 (Companies): Can proceed after Foundational (independent of US1)
- **Built-in Dictionaries (Phase 8)**: Depends on all user stories being stable
- **Polish (Phase 9)**: Depends on all previous phases

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 4 (P1)**: Can start after Foundational - Independent of US1
- **User Story 5 (P2)**: Can start after US1 (uses name dictionaries for testing fuzzy)
- **User Story 2 (P2)**: Can start after Foundational - Independent
- **User Story 3 (P2)**: Can start after Foundational - Independent

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Models/structs before services/logic
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- T003, T004: Setup error types and enums in parallel
- T014, T015, T016: All US1 tests in parallel
- T025, T026, T027: All US4 tests in parallel
- T034, T035, T036: All US5 tests in parallel
- T043, T044, T045: All US2 tests in parallel
- T050, T051, T052: All US3 tests in parallel
- T058, T059: AT and CH first names in parallel
- T062, T063: DE and CH cities in parallel
- T067, T068: Documentation and integration tests in parallel

---

## Parallel Example: Phase 2 Foundational

```bash
# Launch in sequence (dependencies):
T005 → T006 → T007, T008 (parallel after T006) → T009 → T010, T011 (parallel) → T012 → T013
```

## Parallel Example: User Story 1

```bash
# Launch all tests together (they should all fail initially):
T014: "Write test for exact first name detection"
T015: "Write test for exact last name detection"
T016: "Write test for name not in dictionary"

# Then implement sequentially:
T017 → T018 → T019 → T020 → T021 → T022, T023 (parallel) → T024
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 4)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Person Names)
4. Complete Phase 4: User Story 4 (Custom Dictionaries)
5. **STOP and VALIDATE**: Test name detection with custom and built-in dictionaries
6. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1 (Person Names) → Test independently → MVP!
3. US4 (Custom Dictionaries) → Test independently → Enhanced MVP
4. US5 (Fuzzy Matching) → Test independently → Better recall
5. US2 (Locations) → Test independently → Expanded coverage
6. US3 (Companies) → Test independently → Full coverage
7. Built-in Dictionaries → Production-ready data
8. Polish → Documentation and benchmarks

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Memory budget: <100MB for all built-in dictionaries (SC-006)
- Performance target: <1ms lookup per word (SC-002)
