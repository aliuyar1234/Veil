# Tasks: PDF Parser

**Input**: Design documents from `/specs/005-pdf-parser/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

**Tests**: Following TDD per constitution - tests written before implementation.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure) ✅

**Purpose**: Project initialization and dependency setup

- [x] T001 Add pdf-extract dependency to crates/veil-parsers/Cargo.toml
- [x] T002 Create pdf module structure in crates/veil-parsers/src/pdf/mod.rs
- [x] T003 [P] Create error types in crates/veil-parsers/src/pdf/error.rs
- [ ] T004 [P] Create options types in crates/veil-parsers/src/pdf/options.rs

---

## Phase 2: Foundational (Blocking Prerequisites) - Partial

**Purpose**: Core infrastructure that MUST be complete before ANY user story

- [x] T005 Extend Position enum with Pdf variant in crates/veil-parsers/src/lib.rs
- [x] T006 Add Format::Pdf variant to format detection in crates/veil-parsers/src/detect.rs
- [x] T007 Implement PDF magic bytes detection (%PDF-) in crates/veil-parsers/src/detect.rs
- [ ] T008 Create PdfDocument struct in crates/veil-parsers/src/pdf/document.rs
- [ ] T009 Create PdfPage struct in crates/veil-parsers/src/pdf/document.rs
- [ ] T010 Create PdfTextBlock struct in crates/veil-parsers/src/pdf/text.rs
- [x] T011 Export pdf module from crates/veil-parsers/src/lib.rs

**Checkpoint**: Foundation partial - basic parsing works but dedicated structures missing

---

## Phase 3: User Story 1 - Extract Text from PDF (Priority: P1) 🎯 MVP - Partial

**Goal**: Extract all text content from PDF preserving reading order

**Independent Test**: Provide PDF with known text, verify extracted content matches

### Tests for User Story 1

- [ ] T012 [P] [US1] Create simple.pdf test fixture in tests/fixtures/pdf/simple.pdf
- [x] T013 [P] [US1] Write test for basic text extraction in crates/veil-parsers/src/pdf/tests.rs
- [ ] T014 [P] [US1] Write test for multi-page extraction in crates/veil-parsers/src/pdf/tests.rs
- [ ] T015 [P] [US1] Write test for reading order (columns) in crates/veil-parsers/src/pdf/tests.rs

### Implementation for User Story 1

- [x] T016 [US1] Implement PdfDocument::from_bytes() using pdf-extract in crates/veil-parsers/src/pdf/document.rs
- [x] T017 [US1] Implement page iteration in crates/veil-parsers/src/pdf/document.rs
- [x] T018 [US1] Implement text block extraction in crates/veil-parsers/src/pdf/text.rs
- [ ] T019 [US1] Implement reading order sorting (Y-cluster, X-sort) in crates/veil-parsers/src/pdf/text.rs
- [x] T020 [US1] Implement PdfDocument::extract_text() returning Vec<TextSegment> in crates/veil-parsers/src/pdf/document.rs
- [x] T021 [US1] Integrate with parse_bytes() for Format::Pdf in crates/veil-parsers/src/lib.rs

**Checkpoint**: Basic PDF text extraction works (simplified implementation)

---

## Phase 4: User Story 2 - Preserve Position Information (Priority: P1) - NOT IMPLEMENTED

**Goal**: Provide page number and bounding box for each text segment

**Independent Test**: Extract text with positions, verify coordinates map to visual locations

**Status**: pdf-extract doesn't provide position info - all coordinates set to 0.0

### Tests for User Story 2

- [ ] T022 [P] [US2] Write test for page number accuracy in crates/veil-parsers/src/pdf/tests.rs
- [ ] T023 [P] [US2] Write test for bounding box extraction in crates/veil-parsers/src/pdf/tests.rs
- [ ] T024 [P] [US2] Write test for byte offset calculation in crates/veil-parsers/src/pdf/tests.rs

### Implementation for User Story 2

- [ ] T025 [US2] Implement bounding box extraction from text objects in crates/veil-parsers/src/pdf/text.rs
- [ ] T026 [US2] Implement page dimension tracking in crates/veil-parsers/src/pdf/document.rs
- [ ] T027 [US2] Implement byte offset accumulation across pages in crates/veil-parsers/src/pdf/document.rs
- [ ] T028 [US2] Populate Position::Pdf in TextSegment output in crates/veil-parsers/src/pdf/document.rs

**Checkpoint**: NOT REACHED - requires different PDF library for position extraction

---

## Phase 5: User Story 3 - Handle Scanned PDFs Gracefully (Priority: P2) - Minimal

**Goal**: Detect scanned/image-only PDFs and report appropriately

**Independent Test**: Provide image-only PDF, verify warning message

**Status**: Only warning for empty text exists, no is_scanned() methods

### Tests for User Story 3

- [ ] T029 [P] [US3] Create scanned.pdf test fixture (image-only) in tests/fixtures/pdf/scanned.pdf
- [ ] T030 [P] [US3] Write test for scanned page detection in crates/veil-parsers/src/pdf/tests.rs
- [ ] T031 [P] [US3] Write test for NoTextContent error in crates/veil-parsers/src/pdf/tests.rs

### Implementation for User Story 3

- [ ] T032 [US3] Implement has_minimal_text() heuristic in crates/veil-parsers/src/pdf/document.rs
- [ ] T033 [US3] Implement is_scanned() for PdfPage in crates/veil-parsers/src/pdf/document.rs
- [ ] T034 [US3] Implement is_scanned() for PdfDocument in crates/veil-parsers/src/pdf/document.rs
- [ ] T035 [US3] Return NoTextContent error when all pages scanned in crates/veil-parsers/src/pdf/document.rs

**Checkpoint**: NOT REACHED - scanned detection not implemented

---

## Phase 6: User Story 4 - Extract Text from PDF Forms (Priority: P2) - NOT IMPLEMENTED

**Goal**: Extract form field names and values

**Independent Test**: Provide PDF with form fields, verify values extracted

**Status**: forms.rs does not exist, no form extraction implemented

### Tests for User Story 4

- [ ] T036 [P] [US4] Create forms.pdf test fixture in tests/fixtures/pdf/forms.pdf
- [ ] T037 [P] [US4] Write test for text field extraction in crates/veil-parsers/src/pdf/tests.rs
- [ ] T038 [P] [US4] Write test for checkbox extraction in crates/veil-parsers/src/pdf/tests.rs
- [ ] T039 [P] [US4] Write test for dropdown extraction in crates/veil-parsers/src/pdf/tests.rs

### Implementation for User Story 4

- [ ] T040 [US4] Create PdfFormField struct in crates/veil-parsers/src/pdf/forms.rs
- [ ] T041 [US4] Create PdfFieldType enum in crates/veil-parsers/src/pdf/forms.rs
- [ ] T042 [US4] Implement AcroForm field extraction in crates/veil-parsers/src/pdf/forms.rs
- [ ] T043 [US4] Extract text field values in crates/veil-parsers/src/pdf/forms.rs
- [ ] T044 [US4] Extract checkbox/radio values in crates/veil-parsers/src/pdf/forms.rs
- [ ] T045 [US4] Extract dropdown/combobox values in crates/veil-parsers/src/pdf/forms.rs
- [ ] T046 [US4] Include form field text in extract_text() output in crates/veil-parsers/src/pdf/document.rs

**Checkpoint**: NOT REACHED - form extraction not implemented

---

## Phase 7: Edge Cases & Error Handling - Partial

**Purpose**: Handle encrypted, corrupted, and edge case PDFs

**Status**: Basic error mapping exists, no test fixtures created

- [ ] T047 [P] Create encrypted.pdf test fixture (password: "test") <!-- pragma: allowlist secret --> in tests/fixtures/pdf/encrypted.pdf
- [ ] T048 [P] Write test for encrypted PDF detection in crates/veil-parsers/src/pdf/tests.rs
- [x] T049 [P] Write test for corrupted PDF handling in crates/veil-parsers/src/pdf/tests.rs
- [x] T050 Implement encrypted PDF detection in crates/veil-parsers/src/pdf/document.rs
- [ ] T051 Implement password handling in PdfParseOptions in crates/veil-parsers/src/pdf/document.rs
- [x] T052 Handle corrupted PDF gracefully with descriptive error in crates/veil-parsers/src/pdf/document.rs

**Checkpoint**: Basic error handling exists, fixtures and password handling missing

---

## Phase 8: Polish & Cross-Cutting Concerns - NOT STARTED

**Purpose**: Documentation, performance validation, final integration

- [ ] T053 [P] Add documentation comments to all public API items in crates/veil-parsers/src/pdf/
- [ ] T054 [P] Write integration test with veil-detect in crates/veil-parsers/tests/pdf_integration.rs
- [ ] T055 Create multipage.pdf test fixture (10+ pages) in tests/fixtures/pdf/multipage.pdf
- [ ] T056 Create columns.pdf test fixture (2-column layout) in tests/fixtures/pdf/columns.pdf
- [ ] T057 Benchmark 100-page PDF extraction time
- [ ] T058 Validate memory usage for large PDF
- [x] T059 Run clippy and fix any warnings in crates/veil-parsers/src/pdf/
- [x] T060 Run cargo fmt on all pdf module files
- [ ] T061 Run quickstart.md validation scenarios

---

## Summary

**Total Tasks**: 61
**Completed**: ~18 (30%)
**Pending**: ~43 (70%)

**What Works**:
- Basic text extraction from PDFs using pdf-extract
- PDF format detection (magic bytes)
- Basic error handling for corrupted/encrypted files
- 5 unit tests passing

**What's Missing**:
- Test fixtures (no PDF files created)
- Position/bounding box extraction (pdf-extract limitation)
- Form field extraction
- Scanned document detection
- Integration tests
- Performance benchmarks

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational - MVP target
- **US2 (Phase 4)**: Can proceed in parallel with US1 or after
- **US3 (Phase 5)**: Depends on US1 (needs extraction to detect emptiness)
- **US4 (Phase 6)**: Depends on US1 (extends extraction)
- **Edge Cases (Phase 7)**: Depends on US1
- **Polish (Phase 8)**: Depends on all user stories

### User Story Dependencies

- **US1 (Text Extraction)**: Foundation only - MVP
- **US2 (Positions)**: Can start after Foundation, integrates with US1
- **US3 (Scanned Detection)**: Depends on US1 (needs to try extraction first)
- **US4 (Forms)**: Depends on US1 (extends text output)

### Parallel Opportunities

- T003, T004: Error and options types in parallel
- T012-T015: All US1 test fixtures and tests in parallel
- T022-T024: All US2 tests in parallel
- T029-T031: All US3 tests in parallel
- T036-T039: All US4 tests in parallel
- T047-T049: Edge case fixtures and tests in parallel
- T053, T054: Documentation and integration tests in parallel

---

## Implementation Strategy

### MVP First (US1 + US2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (Text Extraction)
4. Complete Phase 4: User Story 2 (Positions)
5. **STOP and VALIDATE**: Parse real PDFs, verify text + positions
6. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1 (Text) → MVP extraction works
3. US2 (Positions) → Full metadata support
4. US3 (Scanned) → Better error handling
5. US4 (Forms) → Complete extraction
6. Edge Cases → Production hardening
7. Polish → Documentation and benchmarks

---

## Test Fixtures Required

| File | Purpose | Creation Method |
|------|---------|-----------------|
| simple.pdf | Basic extraction | Create with text content |
| multipage.pdf | Multi-page | Create 10+ page document |
| forms.pdf | Form fields | Create with text/checkbox/dropdown |
| columns.pdf | Reading order | Create 2-column layout |
| scanned.pdf | Detection | Image-only PDF |
| encrypted.pdf | Password handling | Encrypt with "test" |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Test fixtures can be created manually or with PDF tools
- pdf-extract may have limitations - document any workarounds needed
- Memory target: <500MB for 100MB PDF
- Performance target: <10s for 100 pages
