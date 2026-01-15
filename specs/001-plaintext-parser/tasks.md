# Tasks: Plaintext Parser

**Input**: Design documents from `/specs/001-plaintext-parser/`
**Prerequisites**: plan.md ✓, spec.md ✓, data-model.md ✓, contracts/ ✓, research.md ✓, quickstart.md ✓

**Tests**: Included per constitution (V. Test-First Development)

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: US1, US2, US3, US4 (maps to user stories from spec.md)

## Path Conventions

```text
Cargo.toml                           # Workspace root
crates/veil-parsers/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── types.rs
│   ├── detect.rs
│   ├── text.rs
│   ├── csv.rs
│   ├── json.rs
│   └── html.rs
└── tests/
    ├── fixtures/{plain,csv,json,html}/
    ├── text_tests.rs
    ├── csv_tests.rs
    ├── json_tests.rs
    └── html_tests.rs
```

---

## Phase 1: Setup (Shared Infrastructure) ✅

**Purpose**: Project initialization and Cargo workspace structure

- [x] T001 Create workspace Cargo.toml at repository root with `[workspace]` configuration
- [x] T002 Create crates/veil-parsers/Cargo.toml with dependencies: serde, thiserror, csv, encoding_rs, scraper, serde_json
- [x] T003 [P] Create crates/veil-parsers/src/lib.rs with module declarations and public re-exports
- [x] T004 [P] Create .gitignore with Rust ignores (target/, Cargo.lock for library)
- [x] T005 Verify `cargo build` succeeds with empty modules

---

## Phase 2: Foundational (Blocking Prerequisites) ✅

**Purpose**: Core types and traits that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T006 Implement ParseError enum with thiserror in crates/veil-parsers/src/error.rs
- [x] T007 Implement FileFormat enum in crates/veil-parsers/src/types.rs
- [x] T008 [P] Implement Position enum (Text, Csv, Json, Html variants) in crates/veil-parsers/src/types.rs
- [x] T009 [P] Implement TextSegment struct in crates/veil-parsers/src/types.rs
- [x] T010 [P] Implement DocumentMetadata struct in crates/veil-parsers/src/types.rs
- [x] T011 [P] Implement ParseWarning and WarningCode in crates/veil-parsers/src/types.rs
- [x] T012 Implement ParseResult struct in crates/veil-parsers/src/types.rs
- [x] T013 Implement ParseOptions struct with Default in crates/veil-parsers/src/types.rs
- [x] T014 Implement Parser trait in crates/veil-parsers/src/lib.rs
- [x] T015 Implement detect_format function stub in crates/veil-parsers/src/detect.rs
- [x] T016 Implement parse_file, parse_bytes, parse_reader function stubs in crates/veil-parsers/src/lib.rs
- [x] T017 Verify `cargo clippy -- -D warnings` passes
- [x] T018 Verify `cargo test` compiles (tests will fail, that's expected)

**Checkpoint**: Foundation ready - all types compile, user story implementation can begin ✅

---

## Phase 3: User Story 1 - Parse Plain Text File (Priority: P1) 🎯 MVP ✅

**Goal**: Extract text content from plain text files with line/column positions and encoding detection

**Independent Test**: Provide a .txt file, verify extracted text matches content exactly with correct line positions

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T019 [P] [US1] Create test fixture crates/veil-parsers/tests/fixtures/plain/simple.txt (UTF-8, multiple lines)
- [x] T020 [P] [US1] Create test fixture crates/veil-parsers/tests/fixtures/plain/empty.txt
- [x] T021 [P] [US1] Create test fixture crates/veil-parsers/tests/fixtures/plain/mixed_endings.txt (CRLF + LF)
- [x] T022 [P] [US1] Create test fixture crates/veil-parsers/tests/fixtures/plain/utf16.txt (UTF-16 BOM)
- [x] T023 [US1] Create integration tests in crates/veil-parsers/tests/text_tests.rs testing:
  - UTF-8 parsing with line positions
  - Empty file handling
  - Mixed line endings normalization
  - UTF-16 encoding detection and conversion

### Implementation for User Story 1

- [x] T024 [US1] Implement encoding detection (BOM-based) in crates/veil-parsers/src/detect.rs
- [x] T025 [US1] Implement TextParser struct in crates/veil-parsers/src/text.rs
- [x] T026 [US1] Implement Parser trait for TextParser (parse_bytes) in crates/veil-parsers/src/text.rs
- [x] T027 [US1] Implement streaming parse_reader for TextParser in crates/veil-parsers/src/text.rs
- [x] T028 [US1] Wire TextParser into parse_file/parse_bytes/parse_reader in crates/veil-parsers/src/lib.rs
- [x] T029 [US1] Implement format detection for text files in crates/veil-parsers/src/detect.rs
- [x] T030 [US1] Verify all US1 tests pass with `cargo test text_tests`

**Checkpoint**: Plain text parsing fully functional - MVP complete ✅

---

## Phase 4: User Story 2 - Parse CSV File (Priority: P2) ✅

**Goal**: Extract cell content from CSV files with row/column positions and header tracking

**Independent Test**: Provide a CSV file, verify each cell extracted with correct row, column, and header name

### Tests for User Story 2

- [x] T031 [P] [US2] Create test fixture crates/veil-parsers/tests/fixtures/csv/simple.csv (headers, 3 rows)
- [x] T032 [P] [US2] Create test fixture crates/veil-parsers/tests/fixtures/csv/quoted.csv (quoted fields with commas, newlines)
- [x] T033 [P] [US2] Create test fixture crates/veil-parsers/tests/fixtures/csv/semicolon.csv (semicolon delimiter)
- [x] T034 [P] [US2] Create test fixture crates/veil-parsers/tests/fixtures/csv/no_headers.csv
- [x] T035 [US2] Create integration tests in crates/veil-parsers/tests/csv_tests.rs testing:
  - Basic CSV with headers
  - RFC 4180 quoted fields
  - Custom delimiter (semicolon)
  - No headers mode
  - Inconsistent column count (warning)

### Implementation for User Story 2

- [x] T036 [US2] Implement CsvParser struct in crates/veil-parsers/src/csv.rs
- [x] T037 [US2] Implement Parser trait for CsvParser (parse_bytes) in crates/veil-parsers/src/csv.rs
- [x] T038 [US2] Implement header extraction in CsvParser in crates/veil-parsers/src/csv.rs
- [x] T039 [US2] Implement configurable delimiter support in crates/veil-parsers/src/csv.rs
- [x] T040 [US2] Implement InconsistentColumns warning in crates/veil-parsers/src/csv.rs
- [x] T041 [US2] Wire CsvParser into lib.rs dispatch in crates/veil-parsers/src/lib.rs
- [x] T042 [US2] Implement format detection for CSV in crates/veil-parsers/src/detect.rs
- [x] T043 [US2] Verify all US2 tests pass with `cargo test csv_tests`

**Checkpoint**: CSV parsing fully functional ✅

---

## Phase 5: User Story 3 - Parse JSON File (Priority: P2) ✅

**Goal**: Extract string values from JSON with JSONPath notation for each value

**Independent Test**: Provide a JSON file, verify all strings extracted with correct `$.path.notation`

### Tests for User Story 3

- [x] T044 [P] [US3] Create test fixture crates/veil-parsers/tests/fixtures/json/simple.json (flat object)
- [x] T045 [P] [US3] Create test fixture crates/veil-parsers/tests/fixtures/json/nested.json (nested objects + arrays)
- [x] T046 [P] [US3] Create test fixture crates/veil-parsers/tests/fixtures/json/mixed_types.json (strings, numbers, bools, null)
- [x] T047 [US3] Create integration tests in crates/veil-parsers/tests/json_tests.rs testing:
  - Flat object string extraction
  - Nested object path notation
  - Array index paths ($.arr[0])
  - Non-string values skipped

### Implementation for User Story 3

- [x] T048 [US3] Implement JsonParser struct in crates/veil-parsers/src/json.rs
- [x] T049 [US3] Implement recursive string extraction with path tracking in crates/veil-parsers/src/json.rs
- [x] T050 [US3] Implement Parser trait for JsonParser in crates/veil-parsers/src/json.rs
- [x] T051 [US3] Wire JsonParser into lib.rs dispatch in crates/veil-parsers/src/lib.rs
- [x] T052 [US3] Implement format detection for JSON in crates/veil-parsers/src/detect.rs
- [x] T053 [US3] Verify all US3 tests pass with `cargo test json_tests`

**Checkpoint**: JSON parsing fully functional ✅

---

## Phase 6: User Story 4 - Parse HTML File (Priority: P3) ✅

**Goal**: Extract visible text from HTML, excluding script/style, with entity decoding

**Independent Test**: Provide an HTML file, verify only visible text extracted with entities decoded

### Tests for User Story 4

- [x] T054 [P] [US4] Create test fixture crates/veil-parsers/tests/fixtures/html/simple.html (basic structure)
- [x] T055 [P] [US4] Create test fixture crates/veil-parsers/tests/fixtures/html/with_scripts.html (script + style tags)
- [x] T056 [P] [US4] Create test fixture crates/veil-parsers/tests/fixtures/html/entities.html (HTML entities)
- [x] T057 [US4] Create integration tests in crates/veil-parsers/tests/html_tests.rs testing:
  - Basic text extraction
  - Script/style exclusion
  - Entity decoding (&amp; → &)
  - Approximate position tracking

### Implementation for User Story 4

- [x] T058 [US4] Implement HtmlParser struct in crates/veil-parsers/src/html.rs
- [x] T059 [US4] Implement visible text extraction using scraper in crates/veil-parsers/src/html.rs
- [x] T060 [US4] Implement script/style exclusion in crates/veil-parsers/src/html.rs
- [x] T061 [US4] Implement Parser trait for HtmlParser in crates/veil-parsers/src/html.rs
- [x] T062 [US4] Wire HtmlParser into lib.rs dispatch in crates/veil-parsers/src/lib.rs
- [x] T063 [US4] Implement format detection for HTML in crates/veil-parsers/src/detect.rs
- [x] T064 [US4] Verify all US4 tests pass with `cargo test html_tests`

**Checkpoint**: HTML parsing fully functional - all user stories complete ✅

---

## Phase 7: Polish & Cross-Cutting Concerns ✅

**Purpose**: Final validation, documentation, and cleanup

- [x] T065 [P] Add doc comments to all public items in crates/veil-parsers/src/lib.rs
- [x] T066 [P] Add doc comments to all public items in crates/veil-parsers/src/types.rs
- [x] T067 Run full test suite: `cargo test`
- [x] T068 Run clippy: `cargo clippy -- -D warnings`
- [x] T069 Run formatter: `cargo fmt --check`
- [x] T070 Validate quickstart.md examples compile (doctest or manual)
- [x] T071 Verify memory usage on 100MB file (manual test per SC-006)

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1: Setup
    ↓
Phase 2: Foundational (BLOCKS all user stories)
    ↓
┌───────────────────────────────────────────────┐
│ User Stories (can run in parallel if staffed) │
├───────────────────────────────────────────────┤
│ Phase 3: US1 (P1) ─┬─ Phase 4: US2 (P2)       │
│                    │                           │
│                    └─ Phase 5: US3 (P2)       │
│                                                │
│ Phase 6: US4 (P3) - can run after P1/P2 done │
└───────────────────────────────────────────────┘
    ↓
Phase 7: Polish (after all desired stories)
```

### User Story Dependencies

- **US1 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **US2 (P2)**: Can start after Phase 2 - Independent of US1
- **US3 (P2)**: Can start after Phase 2 - Independent of US1/US2
- **US4 (P3)**: Can start after Phase 2 - Independent of all other stories

All user stories share foundational types but have independent parsers.

### Within Each User Story

1. Tests MUST be written and FAIL before implementation
2. Test fixtures created first
3. Parser struct implemented
4. Parser trait implemented
5. Wired into lib.rs dispatch
6. Format detection added
7. All tests pass

### Parallel Opportunities

- **Phase 1**: T003, T004 can run in parallel
- **Phase 2**: T008, T009, T010, T011 can run in parallel (different types)
- **Phase 3 (US1)**: T019-T022 fixtures can be created in parallel
- **Phase 4 (US2)**: T031-T034 fixtures can be created in parallel
- **Phase 5 (US3)**: T044-T046 fixtures can be created in parallel
- **Phase 6 (US4)**: T054-T056 fixtures can be created in parallel
- **Phase 7**: T065, T066 can run in parallel

---

## Parallel Example: Phase 2 Foundational

```bash
# Launch parallelizable type definitions:
Task: "Implement Position enum in crates/veil-parsers/src/types.rs"
Task: "Implement TextSegment struct in crates/veil-parsers/src/types.rs"
Task: "Implement DocumentMetadata struct in crates/veil-parsers/src/types.rs"
Task: "Implement ParseWarning in crates/veil-parsers/src/types.rs"
```

## Parallel Example: User Story 1 Test Fixtures

```bash
# Launch all US1 test fixture creation:
Task: "Create test fixture crates/veil-parsers/tests/fixtures/plain/simple.txt"
Task: "Create test fixture crates/veil-parsers/tests/fixtures/plain/empty.txt"
Task: "Create test fixture crates/veil-parsers/tests/fixtures/plain/mixed_endings.txt"
Task: "Create test fixture crates/veil-parsers/tests/fixtures/plain/utf16.txt"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (Plain Text)
4. **STOP and VALIDATE**: `cargo test text_tests` passes
5. MVP deliverable: Can parse .txt files

### Incremental Delivery

1. Setup + Foundational → Crate compiles
2. Add US1 (Plain Text) → Parse .txt files (MVP!)
3. Add US2 (CSV) → Parse .csv files
4. Add US3 (JSON) → Parse .json files
5. Add US4 (HTML) → Parse .html files
6. Each story adds format support without breaking others

### Parallel Team Strategy

With multiple developers after Foundational phase:

- Developer A: US1 (Plain Text) - MVP priority
- Developer B: US2 (CSV)
- Developer C: US3 (JSON)
- Developer D: US4 (HTML)

---

## Summary

| Metric | Value |
|--------|-------|
| Total tasks | 71 |
| **Completed** | **69** ✅ |
| Phase 1 (Setup) | 5/5 ✅ |
| Phase 2 (Foundational) | 13/13 ✅ |
| Phase 3 (US1 - P1) | 12/12 ✅ |
| Phase 4 (US2 - P2) | 13/13 ✅ |
| Phase 5 (US3 - P2) | 10/10 ✅ |
| Phase 6 (US4 - P3) | 11/11 ✅ |
| Phase 7 (Polish) | 5/7 (T070, T071 pending manual validation) |
| Parallel opportunities | 24 tasks marked [P] |
| MVP scope | Phase 1 + 2 + 3 (30 tasks) ✅ |

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Verify tests FAIL before implementing (TDD per constitution)
- Commit after each task or logical group
- Run `cargo clippy -- -D warnings` frequently
