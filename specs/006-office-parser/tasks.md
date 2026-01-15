# Tasks: Office Document Parser

**Feature**: 006-office-parser
**Input**: Design documents from `/specs/006-office-parser/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

**Tests**: Tests are included based on the spec's success criteria and constitution requirement for test-first development.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- Workspace root: `D:\Projekte\Veil\`
- New crate: `crates/veil-office/`
- Extended crate: `crates/veil-parsers/`
- Test fixtures: `crates/veil-office/tests/fixtures/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create veil-office crate structure at crates/veil-office/ with src/, tests/, Cargo.toml
- [x] T002 Add workspace dependencies to Cargo.toml: calamine 0.24, zip 0.6, quick-xml 0.31
- [x] T003 [P] Add veil-office to workspace members in root Cargo.toml
- [x] T004 [P] Create error module at crates/veil-office/src/error.rs with OfficeError enum
- [x] T005 [P] Create utils module at crates/veil-office/src/utils.rs for shared utilities
- [x] T006 [P] Create metadata module at crates/veil-office/src/metadata.rs for OfficeMetadata struct
- [x] T007 [P] Create detect module at crates/veil-office/src/detect.rs for Office format detection
- [x] T008 Create lib.rs at crates/veil-office/src/lib.rs with public API exports

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**WARNING: No user story work can begin until this phase is complete**

- [x] T009 Implement OfficeError with thiserror in crates/veil-office/src/error.rs (NotZipArchive, NotOfficeOpenXml, Encrypted, LegacyFormat, UnsupportedFormat, XmlError, Corrupted, FileTooLarge, Io)
- [x] T010 [P] Implement OfficeMetadata struct in crates/veil-office/src/metadata.rs with Dublin Core fields (title, subject, creator, keywords, last_modified_by, created, modified, company, manager)
- [x] T011 [P] Implement OfficeMetadata::to_text_segments() method in crates/veil-office/src/metadata.rs to convert metadata to TextSegments
- [x] T012 [P] Implement ZIP utilities in crates/veil-office/src/utils.rs: sanitize_zip_path() for path traversal prevention, check_zip_entry_size() for ZIP bomb protection
- [x] T013 [P] Implement Office format detection in crates/veil-office/src/detect.rs: detect_office_type() checks for [Content_Types].xml and format-specific folders
- [x] T014 [P] Implement encrypted file detection in crates/veil-office/src/detect.rs: is_encrypted() checks for EncryptedPackage/EncryptionInfo
- [x] T015 [P] Implement legacy format detection in crates/veil-office/src/detect.rs: is_legacy_format() checks for OLE2 signature
- [x] T016 [P] Implement metadata extraction in crates/veil-office/src/metadata.rs: extract_metadata() parses docProps/core.xml and docProps/app.xml
- [x] T017 Extend FileFormat enum in crates/veil-parsers/src/types.rs with Docx, Xlsx, Pptx variants
- [x] T018 Extend Position enum in crates/veil-parsers/src/types.rs with Docx, Xlsx, Pptx, OfficeMetadata variants
- [x] T019 [P] Add DocxSection enum in crates/veil-office/src/docx/parser.rs (Body, Header, Footer, Note, Table, TextBox, Comment)
- [x] T020 [P] Add TableCell struct in crates/veil-parsers/src/types.rs (table_index, row, column)
- [x] T021 [P] Add PptxElement enum in crates/veil-office/src/pptx/parser.rs (Title, Body, Note, Shape, Table)
- [x] T022 Extend ParseError enum in crates/veil-parsers/src/error.rs with Encrypted and UnsupportedFormat variants
- [x] T023 Implement From<OfficeError> for calamine::Error conversion in crates/veil-office/src/error.rs
- [x] T024 Extend detect module in crates/veil-parsers/src/detect.rs to detect Office formats via magic bytes and ZIP structure

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 2 - Extract Text from Excel Spreadsheets (Priority: P1) MVP

**Goal**: Extract text from all cells across all sheets in XLSX files with precise cell references (sheet name, row, column)

**Why Priority P1**: Excel files often contain bulk PII data (customer lists, employee records) requiring precise cell-level detection. Highest business value.

**Independent Test**: Provide XLSX with data across multiple sheets, extract, verify all cells captured with correct references including formulas showing display values and hidden sheets processed.

### Tests for User Story 2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T025 [P] [US2] Create test fixture simple.xlsx in crates/veil-office/tests/fixtures/xlsx/ with single sheet and basic cells
- [ ] T026 [P] [US2] Create test fixture multi_sheet.xlsx in crates/veil-office/tests/fixtures/xlsx/ with multiple sheets
- [ ] T027 [P] [US2] Create test fixture formulas.xlsx in crates/veil-office/tests/fixtures/xlsx/ with cells containing formulas
- [ ] T028 [P] [US2] Create test fixture large.xlsx in crates/veil-office/tests/fixtures/xlsx/ with 100K rows for streaming test
- [ ] T029 [P] [US2] Create test fixture hidden_sheet.xlsx in crates/veil-office/tests/fixtures/xlsx/ with hidden sheet containing data
- [ ] T030 [P] [US2] Create test fixture metadata.xlsx in crates/veil-office/tests/fixtures/xlsx/ with rich metadata
- [ ] T031 [P] [US2] Write unit test in crates/veil-office/tests/xlsx_tests.rs for simple.xlsx extraction verifying all cells extracted
- [ ] T032 [P] [US2] Write unit test in crates/veil-office/tests/xlsx_tests.rs for multi_sheet.xlsx verifying cells from all sheets extracted
- [ ] T033 [P] [US2] Write unit test in crates/veil-office/tests/xlsx_tests.rs for formulas.xlsx verifying display values (not formulas) extracted
- [ ] T034 [P] [US2] Write unit test in crates/veil-office/tests/xlsx_tests.rs for large.xlsx verifying 100K rows processed without memory issues
- [ ] T035 [P] [US2] Write unit test in crates/veil-office/tests/xlsx_tests.rs for hidden_sheet.xlsx verifying hidden sheets processed
- [ ] T036 [P] [US2] Write unit test in crates/veil-office/tests/metadata_tests.rs for metadata.xlsx verifying metadata fields extracted

### Implementation for User Story 2

- [x] T037 [P] [US2] Create xlsx module directory at crates/veil-office/src/xlsx/
- [x] T038 [P] [US2] Create mod.rs at crates/veil-office/src/xlsx/mod.rs with public exports
- [x] T039 [P] [US2] Create cell_ref.rs at crates/veil-office/src/xlsx/cell_ref.rs with CellReference utility
- [x] T040 [US2] Implement CellReference::column_letter() in crates/veil-office/src/xlsx/cell_ref.rs to convert column index to Excel letters (0->A, 25->Z, 26->AA)
- [x] T041 [US2] Implement CellReference::to_string() in crates/veil-office/src/xlsx/cell_ref.rs to format as Sheet1!B5 style references
- [x] T042 [P] [US2] Create parser.rs at crates/veil-office/src/xlsx/parser.rs for main XLSX parsing logic
- [x] T043 [US2] Implement parse_xlsx() in crates/veil-office/src/xlsx/parser.rs using calamine to open workbook and iterate sheets
- [x] T044 [US2] Implement sheet extraction in crates/veil-office/src/xlsx/parser.rs to get all sheets including hidden sheets
- [x] T045 [US2] Implement cell iteration in crates/veil-office/src/xlsx/parser.rs to extract cell values (display values, not formulas)
- [x] T046 [US2] Implement cell-to-TextSegment conversion in crates/veil-office/src/xlsx/parser.rs with Position::Xlsx containing sheet, row, column, column_letter, cell_ref, hidden_sheet flag
- [ ] T047 [P] [US2] Create streaming.rs at crates/veil-office/src/xlsx/streaming.rs for large file streaming support
- [ ] T048 [US2] Implement streaming row iterator in crates/veil-office/src/xlsx/streaming.rs using calamine's streaming API to process rows without loading entire sheet
- [x] T049 [US2] Add parse_xlsx() to public API in crates/veil-office/src/lib.rs
- [x] T050 [US2] Integrate XLSX parser into veil-parsers dispatch in crates/veil-parsers/src/lib.rs for FileFormat::Xlsx
- [x] T051 [US2] Run all User Story 2 tests and verify they pass

**Checkpoint**: At this point, XLSX parsing should be fully functional with streaming support for large files, all cells extracted with correct references

---

## Phase 4: User Story 1 - Extract Text from Word Documents (Priority: P1)

**Goal**: Extract all text content from DOCX files including body text, headers, footers, tables, and text boxes for PII detection

**Why Priority P1**: Word documents are ubiquitous in business environments and often contain sensitive information.

**Independent Test**: Provide DOCX with various text locations (body, headers, footers, tables), extract, verify all text areas captured with location metadata.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T052 [P] [US1] Create test fixture simple.docx in crates/veil-office/tests/fixtures/docx/ with basic paragraph text
- [ ] T053 [P] [US1] Create test fixture table.docx in crates/veil-office/tests/fixtures/docx/ with document containing tables
- [ ] T054 [P] [US1] Create test fixture header_footer.docx in crates/veil-office/tests/fixtures/docx/ with headers and footers
- [ ] T055 [P] [US1] Create test fixture metadata.docx in crates/veil-office/tests/fixtures/docx/ with rich metadata
- [ ] T056 [P] [US1] Create test fixture encrypted.docx in crates/veil-office/tests/fixtures/docx/ that is password-protected
- [ ] T057 [P] [US1] Create test fixture corrupted.docx in crates/veil-office/tests/fixtures/docx/ with malformed ZIP
- [ ] T058 [P] [US1] Write unit test in crates/veil-office/tests/docx_tests.rs for simple.docx verifying all paragraphs extracted in order
- [ ] T059 [P] [US1] Write unit test in crates/veil-office/tests/docx_tests.rs for table.docx verifying table cells extracted with row/column positions
- [ ] T060 [P] [US1] Write unit test in crates/veil-office/tests/docx_tests.rs for header_footer.docx verifying header/footer text extracted with location metadata
- [ ] T061 [P] [US1] Write unit test in crates/veil-office/tests/error_tests.rs for encrypted.docx verifying encrypted file rejected with proper error
- [ ] T062 [P] [US1] Write unit test in crates/veil-office/tests/error_tests.rs for corrupted.docx verifying corrupted file handled gracefully

### Implementation for User Story 1

- [x] T063 [P] [US1] Create docx module directory at crates/veil-office/src/docx/
- [x] T064 [P] [US1] Create mod.rs at crates/veil-office/src/docx/mod.rs with public exports
- [x] T065 [P] [US1] Create parser.rs at crates/veil-office/src/docx/parser.rs for main document.xml parsing
- [x] T066 [US1] Implement ZIP extraction in crates/veil-office/src/docx/parser.rs to open DOCX as ZIP archive
- [x] T067 [US1] Implement document.xml extraction in crates/veil-office/src/docx/parser.rs using quick-xml event-driven parsing
- [x] T068 [US1] Implement paragraph extraction in crates/veil-office/src/docx/parser.rs to parse w:p elements into text with paragraph numbers
- [x] T069 [US1] Implement text run extraction in crates/veil-office/src/docx/parser.rs to parse w:t elements within paragraphs
- [ ] T070 [P] [US1] Create tables.rs at crates/veil-office/src/docx/tables.rs for table extraction
- [x] T071 [US1] Implement table extraction in crates/veil-office/src/docx/parser.rs to parse w:tbl elements with row/column positions
- [x] T072 [US1] Implement table cell extraction in crates/veil-office/src/docx/parser.rs to parse w:tc elements within tables
- [ ] T073 [P] [US1] Create headers.rs at crates/veil-office/src/docx/headers.rs for header/footer extraction
- [x] T074 [US1] Implement header extraction in crates/veil-office/src/docx/parser.rs to parse header*.xml files from word/ folder
- [x] T075 [US1] Implement footer extraction in crates/veil-office/src/docx/parser.rs to parse footer*.xml files from word/ folder
- [x] T076 [US1] Implement paragraph-to-TextSegment conversion in crates/veil-office/src/docx/parser.rs with Position::Docx containing section, paragraph, char_offset, char_length, optional page number
- [x] T077 [US1] Implement table-to-TextSegment conversion in crates/veil-office/src/docx/parser.rs with Position::Docx including table_cell information
- [x] T078 [US1] Add parse_docx() to public API in crates/veil-office/src/lib.rs
- [x] T079 [US1] Integrate DOCX parser into veil-parsers dispatch in crates/veil-parsers/src/lib.rs for FileFormat::Docx
- [x] T080 [US1] Run all User Story 1 tests and verify they pass

**Checkpoint**: At this point, DOCX parsing should be fully functional with text from body, headers, footers, and tables extracted correctly

---

## Phase 5: User Story 3 - Extract Text from PowerPoint Presentations (Priority: P2)

**Goal**: Extract text from PPTX files including slides, speaker notes, and text boxes for PII detection

**Why Priority P2**: Presentations are shared widely and may contain sensitive information in various locations.

**Independent Test**: Provide PPTX with text in slides, notes, and shapes, verify all text extracted with slide numbers and element types.

### Tests for User Story 3

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T081 [P] [US3] Create test fixture simple.pptx in crates/veil-office/tests/fixtures/pptx/ with title and body slides
- [ ] T082 [P] [US3] Create test fixture notes.pptx in crates/veil-office/tests/fixtures/pptx/ with speaker notes
- [ ] T083 [P] [US3] Create test fixture shapes.pptx in crates/veil-office/tests/fixtures/pptx/ with text in shapes and text boxes
- [ ] T084 [P] [US3] Create test fixture metadata.pptx in crates/veil-office/tests/fixtures/pptx/ with rich metadata
- [ ] T085 [P] [US3] Write unit test in crates/veil-office/tests/pptx_tests.rs for simple.pptx verifying text from all slides extracted with slide numbers
- [ ] T086 [P] [US3] Write unit test in crates/veil-office/tests/pptx_tests.rs for notes.pptx verifying speaker notes extracted with association to slide numbers
- [ ] T087 [P] [US3] Write unit test in crates/veil-office/tests/pptx_tests.rs for shapes.pptx verifying all text content from shapes captured

### Implementation for User Story 3

- [x] T088 [P] [US3] Create pptx module directory at crates/veil-office/src/pptx/
- [x] T089 [P] [US3] Create mod.rs at crates/veil-office/src/pptx/mod.rs with public exports
- [x] T090 [P] [US3] Create parser.rs at crates/veil-office/src/pptx/parser.rs for main presentation parsing
- [x] T091 [US3] Implement ZIP extraction in crates/veil-office/src/pptx/parser.rs to open PPTX as ZIP archive
- [x] T092 [US3] Implement presentation.xml parsing in crates/veil-office/src/pptx/parser.rs to get slide list
- [ ] T093 [P] [US3] Create slides.rs at crates/veil-office/src/pptx/slides.rs for slide content extraction
- [x] T094 [US3] Implement slide XML extraction in crates/veil-office/src/pptx/parser.rs to parse ppt/slides/slideN.xml files
- [x] T095 [US3] Implement slide text extraction in crates/veil-office/src/pptx/parser.rs to parse p:txBody elements from DrawingML
- [x] T096 [US3] Implement shape text extraction in crates/veil-office/src/pptx/parser.rs to parse a:t elements within shapes
- [ ] T097 [P] [US3] Create notes.rs at crates/veil-office/src/pptx/notes.rs for speaker notes extraction
- [x] T098 [US3] Implement notes extraction in crates/veil-office/src/pptx/parser.rs to parse ppt/notesSlides/notesSlideN.xml files
- [x] T099 [US3] Implement slide-to-TextSegment conversion in crates/veil-office/src/pptx/parser.rs with Position::Pptx containing slide, element, text_index, char_offset, char_length
- [x] T100 [US3] Add parse_pptx() to public API in crates/veil-office/src/lib.rs
- [x] T101 [US3] Integrate PPTX parser into veil-parsers dispatch in crates/veil-parsers/src/lib.rs for FileFormat::Pptx
- [x] T102 [US3] Run all User Story 3 tests and verify they pass

**Checkpoint**: At this point, PPTX parsing should be fully functional with text from slides, notes, and shapes extracted correctly

---

## Phase 6: User Story 4 - Handle Document Metadata (Priority: P2)

**Goal**: Extract document metadata (author, company, last modified by) from all Office formats for PII detection

**Why Priority P2**: Document metadata often contains names and organizational information that may be PII.

**Independent Test**: Provide documents with metadata, verify author, company, and other fields extracted correctly; documents without metadata cause no errors.

### Tests for User Story 4

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T103 [P] [US4] Write unit test in crates/veil-office/tests/metadata_tests.rs for metadata.docx verifying author, title, company fields extracted as metadata segments
- [ ] T104 [P] [US4] Write unit test in crates/veil-office/tests/metadata_tests.rs for metadata.xlsx verifying last_modified_by field extracted with metadata label
- [ ] T105 [P] [US4] Write unit test in crates/veil-office/tests/metadata_tests.rs for metadata.pptx verifying multiple metadata fields extracted correctly
- [ ] T106 [P] [US4] Write unit test in crates/veil-office/tests/metadata_tests.rs for document without metadata verifying no metadata segments returned and no error

### Implementation for User Story 4

- [x] T107 [US4] Implement docProps/core.xml parsing in crates/veil-office/src/metadata.rs using quick-xml to extract Dublin Core properties (dc:creator, dc:title, dc:subject, cp:lastModifiedBy, dcterms:created, dcterms:modified)
- [x] T108 [US4] Implement docProps/app.xml parsing in crates/veil-office/src/metadata.rs using quick-xml to extract extended properties (company, manager)
- [x] T109 [US4] Integrate metadata extraction into parse_docx() in crates/veil-office/src/docx/parser.rs to include metadata segments in output
- [x] T110 [US4] Integrate metadata extraction into parse_xlsx() in crates/veil-office/src/xlsx/parser.rs to include metadata segments in output
- [x] T111 [US4] Integrate metadata extraction into parse_pptx() in crates/veil-office/src/pptx/parser.rs to include metadata segments in output
- [x] T112 [US4] Run all User Story 4 tests and verify they pass

**Checkpoint**: All user stories should now be independently functional with complete metadata extraction across all formats

---

## Phase 7: Edge Cases and Error Handling

**Purpose**: Handle all edge cases specified in the spec

- [ ] T113 [P] Create test fixture old.doc in crates/veil-office/tests/fixtures/legacy/ (legacy Word binary format)
- [ ] T114 [P] Create test fixture old.xls in crates/veil-office/tests/fixtures/legacy/ (legacy Excel binary format)
- [ ] T115 [P] Create test fixture old.ppt in crates/veil-office/tests/fixtures/legacy/ (legacy PowerPoint binary format)
- [ ] T116 [P] Write unit test in crates/veil-office/tests/error_tests.rs for old.doc verifying legacy format rejected with clear error message suggesting conversion
- [ ] T117 [P] Write unit test in crates/veil-office/tests/error_tests.rs for old.xls verifying legacy format rejected
- [ ] T118 [P] Write unit test in crates/veil-office/tests/error_tests.rs for old.ppt verifying legacy format rejected
- [ ] T119 [P] Write unit test in crates/veil-office/tests/error_tests.rs verifying ZIP bomb protection (file exceeding 50MB limit)
- [ ] T120 [P] Write unit test in crates/veil-office/tests/error_tests.rs verifying path traversal prevention (ZIP entry with .. in path)
- [ ] T121 Run all edge case and error handling tests and verify they pass

---

## Phase 8: Integration and Performance

**Purpose**: Integration tests with veil-parsers and performance validation

- [ ] T122 [P] Write integration test in crates/veil-office/tests/integration_tests.rs for parse_file() with DOCX verifying end-to-end parsing via veil-parsers interface
- [ ] T123 [P] Write integration test in crates/veil-office/tests/integration_tests.rs for parse_file() with XLSX verifying end-to-end parsing
- [ ] T124 [P] Write integration test in crates/veil-office/tests/integration_tests.rs for parse_file() with PPTX verifying end-to-end parsing
- [ ] T125 [P] Create performance benchmark test in crates/veil-office/tests/performance_tests.rs for 10MB DOCX file verifying parsing completes in under 5 seconds
- [ ] T126 [P] Create performance benchmark test in crates/veil-office/tests/performance_tests.rs for 10MB XLSX file verifying parsing completes in under 5 seconds
- [ ] T127 [P] Create performance benchmark test in crates/veil-office/tests/performance_tests.rs for large.xlsx (100K rows) verifying memory usage stays under 500MB
- [ ] T128 Verify acceptance criteria SC-001: DOCX text extraction matches copy-paste from Word with 99% accuracy
- [ ] T129 Verify acceptance criteria SC-002: XLSX cells extracted with 100% correct cell references
- [ ] T130 Verify acceptance criteria SC-003: PPTX text from all slides and notes extracted completely
- [ ] T131 Verify acceptance criteria SC-004: Document metadata extracted when present
- [ ] T132 Verify acceptance criteria SC-005: 10MB Office document parsed in under 5 seconds
- [ ] T133 Verify acceptance criteria SC-006: Excel files with 100K rows processed without memory issues
- [ ] T134 Run all integration and performance tests and verify they pass

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, code quality, and final refinements

- [ ] T135 [P] Add documentation comments to all public items in crates/veil-office/src/lib.rs
- [ ] T136 [P] Add documentation comments to all public items in crates/veil-office/src/error.rs
- [ ] T137 [P] Add documentation comments to all public items in crates/veil-office/src/metadata.rs
- [ ] T138 [P] Add documentation comments to all public items in crates/veil-office/src/xlsx/mod.rs
- [ ] T139 [P] Add documentation comments to all public items in crates/veil-office/src/docx/mod.rs
- [ ] T140 [P] Add documentation comments to all public items in crates/veil-office/src/pptx/mod.rs
- [x] T141 Run cargo clippy on veil-office crate and fix all warnings
- [x] T142 Run cargo fmt on veil-office crate to ensure consistent formatting
- [x] T143 Run cargo test on entire workspace and verify all tests pass
- [ ] T144 [P] Update crates/veil-parsers/README.md with Office format support information
- [ ] T145 [P] Update root README.md with Office parser features
- [ ] T146 [P] Update D:\Projekte\Veil\CLAUDE.md with Office parser technology stack and dependencies
- [ ] T147 Create quickstart example in crates/veil-office/examples/ demonstrating Office file parsing
- [ ] T148 Run cargo test --release on entire workspace for final validation
- [ ] T149 Verify test coverage is above 80% for veil-office crate
- [ ] T150 Final constitution compliance check: Security, Stability, Performance, Simplicity principles all satisfied

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 2 - XLSX (Phase 3)**: Depends on Foundational phase completion - Highest priority
- **User Story 1 - DOCX (Phase 4)**: Depends on Foundational phase completion - Can run in parallel with Phase 3
- **User Story 3 - PPTX (Phase 5)**: Depends on Foundational phase completion - Can run in parallel with Phases 3 & 4
- **User Story 4 - Metadata (Phase 6)**: Depends on Phases 3, 4, 5 completion - Integrates into all parsers
- **Edge Cases (Phase 7)**: Depends on Phases 3, 4, 5 completion - Can run in parallel with Phase 6
- **Integration (Phase 8)**: Depends on all user stories complete
- **Polish (Phase 9)**: Depends on all previous phases complete

### User Story Dependencies

- **User Story 2 - XLSX (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories - HIGHEST BUSINESS VALUE
- **User Story 1 - DOCX (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 3 - PPTX (P2)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 4 - Metadata (P2)**: Depends on US1, US2, US3 being implemented to integrate metadata extraction into all parsers

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Module structure before implementation
- Core parsing logic before specialized extraction
- Text extraction before TextSegment conversion
- Integration into lib.rs API after implementation complete
- Integration into veil-parsers after API stable

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel (T003-T007)
- All Foundational tasks marked [P] can run in parallel within phase groups (T010-T011, T012-T016, T019-T021)
- Once Foundational phase completes, User Stories 1, 2, 3 can start in parallel (recommend starting US2 first for business value)
- All test fixture creation tasks marked [P] can run in parallel within each user story
- All test writing tasks marked [P] can run in parallel within each user story
- All documentation tasks marked [P] can run in parallel in Phase 9

---

## Parallel Example: User Story 2 (XLSX)

```bash
# Launch all test fixtures for XLSX together:
Task T025: "Create test fixture simple.xlsx in crates/veil-office/tests/fixtures/xlsx/"
Task T026: "Create test fixture multi_sheet.xlsx in crates/veil-office/tests/fixtures/xlsx/"
Task T027: "Create test fixture formulas.xlsx in crates/veil-office/tests/fixtures/xlsx/"
Task T028: "Create test fixture large.xlsx in crates/veil-office/tests/fixtures/xlsx/"
Task T029: "Create test fixture hidden_sheet.xlsx in crates/veil-office/tests/fixtures/xlsx/"
Task T030: "Create test fixture metadata.xlsx in crates/veil-office/tests/fixtures/xlsx/"

# Launch all test writing for XLSX together:
Task T031: "Write unit test in crates/veil-office/tests/xlsx_tests.rs for simple.xlsx"
Task T032: "Write unit test in crates/veil-office/tests/xlsx_tests.rs for multi_sheet.xlsx"
Task T033: "Write unit test in crates/veil-office/tests/xlsx_tests.rs for formulas.xlsx"
Task T034: "Write unit test in crates/veil-office/tests/xlsx_tests.rs for large.xlsx"
Task T035: "Write unit test in crates/veil-office/tests/xlsx_tests.rs for hidden_sheet.xlsx"
Task T036: "Write unit test in crates/veil-office/tests/metadata_tests.rs for metadata.xlsx"

# Launch module structure creation together:
Task T037: "Create xlsx module directory at crates/veil-office/src/xlsx/"
Task T038: "Create mod.rs at crates/veil-office/src/xlsx/mod.rs"
Task T039: "Create cell_ref.rs at crates/veil-office/src/xlsx/cell_ref.rs"
Task T042: "Create parser.rs at crates/veil-office/src/xlsx/parser.rs"
Task T047: "Create streaming.rs at crates/veil-office/src/xlsx/streaming.rs"
```

---

## Implementation Strategy

### MVP First (User Story 2 - XLSX Only)

**Rationale**: XLSX parsing provides highest business value - Excel files often contain bulk PII data like customer lists and employee records.

1. Complete Phase 1: Setup (T001-T008)
2. Complete Phase 2: Foundational (T009-T024) - CRITICAL: blocks all stories
3. Complete Phase 3: User Story 2 - XLSX (T025-T051)
4. **STOP and VALIDATE**: Test XLSX parsing independently with all test fixtures
5. Deploy/demo if ready - MVP delivers Excel parsing capability

### Incremental Delivery

1. Complete Setup + Foundational (Phases 1-2) → Foundation ready
2. Add User Story 2 - XLSX (Phase 3) → Test independently → Deploy/Demo (MVP - highest value!)
3. Add User Story 1 - DOCX (Phase 4) → Test independently → Deploy/Demo
4. Add User Story 3 - PPTX (Phase 5) → Test independently → Deploy/Demo
5. Add User Story 4 - Metadata (Phase 6) → Test independently → Deploy/Demo
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (Phases 1-2)
2. Once Foundational is done:
   - Developer A: User Story 2 - XLSX (Phase 3) - PRIORITY
   - Developer B: User Story 1 - DOCX (Phase 4)
   - Developer C: User Story 3 - PPTX (Phase 5)
3. Stories complete and integrate independently
4. Developer A then works on User Story 4 - Metadata (Phase 6) which integrates all three parsers

---

## Task Summary

**Total Tasks**: 150
- **Phase 1 - Setup**: 8 tasks
- **Phase 2 - Foundational**: 16 tasks (BLOCKS all user stories)
- **Phase 3 - User Story 2 (XLSX)**: 27 tasks (12 tests + 15 implementation)
- **Phase 4 - User Story 1 (DOCX)**: 29 tasks (11 tests + 18 implementation)
- **Phase 5 - User Story 3 (PPTX)**: 22 tasks (7 tests + 15 implementation)
- **Phase 6 - User Story 4 (Metadata)**: 12 tasks (4 tests + 8 implementation)
- **Phase 7 - Edge Cases**: 9 tasks
- **Phase 8 - Integration**: 13 tasks
- **Phase 9 - Polish**: 16 tasks

**Parallel Opportunities**: 79 tasks marked [P] can run in parallel with appropriate dependencies

**Independent Test Criteria**:
- **US2 (XLSX)**: Parse multi-sheet Excel file, verify all cells extracted with correct Sheet!A1 style references, formulas show display values, hidden sheets processed
- **US1 (DOCX)**: Parse Word document with body, headers, footers, tables, verify all text extracted with section and position metadata
- **US3 (PPTX)**: Parse PowerPoint with slides and notes, verify all text extracted with slide numbers and element types
- **US4 (Metadata)**: Parse documents with and without metadata, verify metadata fields extracted as segments

**Suggested MVP Scope**: Phase 1 + Phase 2 + Phase 3 (User Story 2 - XLSX only)
- Rationale: Excel parsing provides highest business value for PII detection in bulk data files

**Format Validation**: All 150 tasks follow the required checklist format with checkbox, Task ID, optional [P] and [Story] labels, and file paths

---

## Notes

- [P] tasks = different files, no dependencies, can run in parallel
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Tests must fail before implementing (TDD approach per constitution)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- User Story 2 (XLSX) prioritized first due to highest business value for PII detection
- All tasks include specific file paths for immediate executability
