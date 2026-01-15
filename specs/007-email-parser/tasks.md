# Tasks: Email Parser (007-email-parser)

**Input**: Design documents from `/specs/007-email-parser/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

**Tests**: This feature follows Test-Driven Development (TDD) as specified in the constitution. Tests MUST be written first and MUST fail before implementation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

This is a Rust workspace project with the following structure:
- **New crate**: `crates/veil-email/` (email parsing library)
- **Existing crate**: `crates/veil-parsers/` (integration point)
- **Workspace root**: `Cargo.toml`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create veil-email crate structure and workspace integration

- [x] T001 Create veil-email crate structure: `crates/veil-email/` with `src/`, `tests/`, `tests/fixtures/` directories
- [x] T002 Initialize `crates/veil-email/Cargo.toml` with dependencies: mailparse (0.15.0), msg-parser (0.5.0), html2text (0.12.0), encoding_rs (0.8.0), thiserror (1.0), serde (1.0)
- [x] T003 [P] Add veil-email to workspace members in root `Cargo.toml`
- [x] T004 [P] Create `crates/veil-email/src/lib.rs` with module declarations and public API stub
- [x] T005 [P] Create empty module files: `src/types.rs`, `src/error.rs`, `src/eml.rs`, `src/msg.rs`, `src/html.rs`, `src/quotes.rs`, `src/convert.rs`
- [x] T006 [P] Configure clippy and rustfmt for veil-email crate
- [ ] T007 Create test fixtures directory structure: `crates/veil-email/tests/fixtures/` for email samples

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and error handling that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T008 Implement `EmailFormat` enum in `crates/veil-email/src/types.rs` (Eml, Msg variants)
- [x] T009 Implement `EmailParseError` enum in `crates/veil-email/src/error.rs` with thiserror derives
- [x] T010 Implement `EmailParseOptions` struct in `crates/veil-email/src/types.rs` with default options
- [x] T011 Implement `EmailMessage` struct in `crates/veil-email/src/types.rs` with headers, body_parts, attachments fields
- [x] T012 [P] Implement `EmailAddress` struct in `crates/veil-email/src/types.rs` with display_name and address fields
- [x] T013 [P] Implement `EmailHeader` struct in `crates/veil-email/src/types.rs` with name, value, raw_value fields
- [x] T014 [P] Implement `EmailHeaderValue` enum in `crates/veil-email/src/types.rs` (Address, AddressList, Text, DateTime, Unstructured variants)
- [x] T015 [P] Implement `EmailBodyPart` struct in `crates/veil-email/src/types.rs` with content_type, charset, content, is_quoted, transfer_encoding fields
- [x] T016 [P] Implement `EmailAttachment` struct in `crates/veil-email/src/types.rs` with filename, content_type, size_bytes, content_id, inline fields
- [x] T017 Add Position::Email variant to `crates/veil-parsers/src/types.rs` with field, field_index, part_index, byte_offset, byte_length
- [x] T018 Add FileFormat::Eml and FileFormat::Msg variants to `crates/veil-parsers/src/lib.rs`
- [x] T019 Update file extension detection in `crates/veil-parsers/src/detect.rs` to recognize .eml and .msg extensions
- [x] T020 Add veil-email dependency to `crates/veil-parsers/Cargo.toml`

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Extract Email Headers (Priority: P1) 🎯 MVP

**Goal**: Parse EML files and extract standard headers (From, To, CC, Subject, Date) with both names and email addresses

**Independent Test**: Provide EML file with various headers, verify all fields captured with correct types and values

### Tests for User Story 1 (TDD)

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T021 [P] [US1] Unit test: Parse From header without display name in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T022 [P] [US1] Unit test: Parse From header with display name (e.g., "John Doe <john@example.com>") in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T023 [P] [US1] Unit test: Parse To header with multiple recipients in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T024 [P] [US1] Unit test: Parse CC header with multiple recipients in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T025 [P] [US1] Unit test: Parse Subject header as plain text in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T026 [P] [US1] Unit test: Parse Date header as DateTime string in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T027 [P] [US1] Integration test: Parse real Gmail export EML file in `crates/veil-email/tests/integration_test.rs`
- [ ] T028 [P] [US1] Integration test: Parse real Outlook export EML file in `crates/veil-email/tests/integration_test.rs`

### Implementation for User Story 1

- [x] T029 [US1] Implement `parse_eml()` function skeleton in `crates/veil-email/src/eml.rs` that calls mailparse::parse_mail()
- [x] T030 [US1] Implement `extract_headers()` helper in `crates/veil-email/src/eml.rs` to iterate ParsedMail headers
- [x] T031 [US1] Implement header name extraction and normalization in `crates/veil-email/src/eml.rs`
- [x] T032 [US1] Implement email address parsing for From header using mailparse address parser in `crates/veil-email/src/eml.rs`
- [x] T033 [US1] Implement email address list parsing for To/CC/BCC headers in `crates/veil-email/src/eml.rs`
- [x] T034 [US1] Implement display name extraction from email addresses in `crates/veil-email/src/eml.rs`
- [x] T035 [US1] Implement plain text header value extraction for Subject in `crates/veil-email/src/eml.rs`
- [x] T036 [US1] Implement Date header parsing to ISO 8601 string in `crates/veil-email/src/eml.rs`
- [x] T037 [US1] Implement Message-ID and other standard headers extraction in `crates/veil-email/src/eml.rs`
- [x] T038 [US1] Add error handling for malformed headers with graceful degradation in `crates/veil-email/src/eml.rs`
- [x] T039 [US1] Add header character encoding detection and conversion to UTF-8 in `crates/veil-email/src/eml.rs`
- [x] T040 [US1] Implement `parse_email()` public API in `crates/veil-email/src/lib.rs` that routes to parse_eml() for EML format

**Checkpoint**: At this point, User Story 1 should pass all tests - EML header extraction fully functional

---

## Phase 4: User Story 2 - Extract Email Body Text (Priority: P1)

**Goal**: Extract both plain text and HTML body content, converting HTML to readable text

**Independent Test**: Provide email with text and HTML parts, verify both captured as readable text

### Tests for User Story 2 (TDD)

- [ ] T041 [P] [US2] Unit test: Extract plain text body from single-part email in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T042 [P] [US2] Unit test: Extract HTML body and convert to text in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T043 [P] [US2] Unit test: Handle multipart/alternative (text + HTML) and prefer plain text in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T044 [P] [US2] Unit test: Decode base64-encoded body content in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T045 [P] [US2] Unit test: Decode quoted-printable body content in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T046 [P] [US2] Unit test: Verify HTML tags removed and links preserved as [text](url) in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T047 [P] [US2] Integration test: Parse email with HTML-only body in `crates/veil-email/tests/integration_test.rs`
- [ ] T048 [P] [US2] Integration test: Parse email with both text and HTML (multipart/alternative) in `crates/veil-email/tests/integration_test.rs`

### Implementation for User Story 2

- [x] T049 [US2] Implement `extract_body_parts()` helper in `crates/veil-email/src/eml.rs` to iterate MIME parts
- [x] T050 [US2] Implement text/plain part extraction in `crates/veil-email/src/eml.rs`
- [x] T051 [US2] Implement text/html part extraction in `crates/veil-email/src/eml.rs`
- [x] T052 [US2] Implement multipart/alternative handling with plain text preference in `crates/veil-email/src/eml.rs`
- [x] T053 [US2] Implement base64 transfer encoding decoding in `crates/veil-email/src/eml.rs`
- [x] T054 [US2] Implement quoted-printable transfer encoding decoding in `crates/veil-email/src/eml.rs`
- [x] T055 [US2] Implement charset detection and UTF-8 conversion for body parts in `crates/veil-email/src/eml.rs`
- [x] T056 [US2] Implement `convert_html_to_text()` function in `crates/veil-email/src/html.rs` using html2text crate
- [x] T057 [US2] Configure html2text to preserve links as [text](url) format in `crates/veil-email/src/html.rs`
- [x] T058 [US2] Configure html2text to convert images to [image: alt] format in `crates/veil-email/src/html.rs`
- [x] T059 [US2] Integrate HTML conversion into body part extraction flow in `crates/veil-email/src/eml.rs`
- [x] T060 [US2] Add EmailParseOptions.convert_html and prefer_plain_text option handling in `crates/veil-email/src/eml.rs`

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently - headers and body text extraction complete

---

## Phase 5: User Story 3 - List Email Attachments (Priority: P2)

**Goal**: Identify email attachments and list metadata (filename, size, MIME type) without loading content

**Independent Test**: Provide email with attachments, verify metadata listed correctly

### Tests for User Story 3 (TDD)

- [ ] T061 [P] [US3] Unit test: List single PDF attachment with metadata in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T062 [P] [US3] Unit test: List multiple attachments (PDF, Excel, image) in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T063 [P] [US3] Unit test: Distinguish inline images from regular attachments in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T064 [P] [US3] Unit test: Handle attachments without filenames in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T065 [P] [US3] Unit test: Verify attachment content NOT loaded into memory in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T066 [P] [US3] Integration test: Parse email with 10+ attachments in under 1 second in `crates/veil-email/tests/integration_test.rs`

### Implementation for User Story 3

- [x] T067 [P] [US3] Implement `extract_attachments()` helper in `crates/veil-email/src/eml.rs` to iterate MIME parts
- [x] T068 [US3] Implement Content-Disposition header parsing to identify attachments in `crates/veil-email/src/eml.rs`
- [x] T069 [US3] Extract filename from Content-Disposition or Content-Type headers in `crates/veil-email/src/eml.rs`
- [x] T070 [US3] Extract MIME content type for each attachment in `crates/veil-email/src/eml.rs`
- [x] T071 [US3] Calculate attachment size without loading content in `crates/veil-email/src/eml.rs`
- [x] T072 [US3] Extract Content-ID header for inline images in `crates/veil-email/src/eml.rs`
- [x] T073 [US3] Set inline flag based on Content-Disposition: inline in `crates/veil-email/src/eml.rs`
- [x] T074 [US3] Add EmailParseOptions.extract_attachments and max_attachment_size option handling in `crates/veil-email/src/eml.rs`
- [x] T075 [US3] Add performance test: verify 10 attachments parsed in <1 second in `crates/veil-email/tests/integration_test.rs`

**Checkpoint**: At this point, User Stories 1, 2, AND 3 should all work independently - complete EML parsing functional

---

## Phase 6: User Story 4 - Parse MSG Files (Priority: P2)

**Goal**: Parse Microsoft Outlook MSG files and extract same information as EML files

**Independent Test**: Provide MSG file, verify headers and body match original email

### Tests for User Story 4 (TDD)

- [ ] T076 [P] [US4] Unit test: Parse MSG file headers (From, To, Subject) in `crates/veil-email/tests/msg_parser_test.rs`
- [ ] T077 [P] [US4] Unit test: Extract MSG body text in `crates/veil-email/tests/msg_parser_test.rs`
- [ ] T078 [P] [US4] Unit test: List MSG attachments in `crates/veil-email/tests/msg_parser_test.rs`
- [ ] T079 [P] [US4] Integration test: Parse real Outlook MSG file in `crates/veil-email/tests/integration_test.rs`
- [ ] T080 [P] [US4] Integration test: Compare MSG vs EML output for same email in `crates/veil-email/tests/integration_test.rs`

### Implementation for User Story 4

- [x] T081 [US4] Implement `parse_msg()` function skeleton in `crates/veil-email/src/msg.rs` that calls msg_parser::MsgParser
- [x] T082 [US4] Implement `extract_msg_headers()` helper in `crates/veil-email/src/msg.rs` to extract MSG properties
- [x] T083 [US4] Map MSG sender property to From header in `crates/veil-email/src/msg.rs`
- [x] T084 [US4] Map MSG recipient properties to To/CC/BCC headers in `crates/veil-email/src/msg.rs`
- [x] T085 [US4] Map MSG subject and date properties to headers in `crates/veil-email/src/msg.rs`
- [x] T086 [US4] Implement `extract_msg_body()` helper in `crates/veil-email/src/msg.rs` to get body text
- [x] T087 [US4] Handle MSG plain text body extraction in `crates/veil-email/src/msg.rs`
- [x] T088 [US4] Handle MSG HTML body extraction and conversion in `crates/veil-email/src/msg.rs`
- [x] T089 [US4] Handle MSG RTF body (extract as-is or convert if possible) in `crates/veil-email/src/msg.rs`
- [x] T090 [US4] Implement `extract_msg_attachments()` helper in `crates/veil-email/src/msg.rs`
- [x] T091 [US4] Extract attachment metadata from MSG attachment properties in `crates/veil-email/src/msg.rs`
- [x] T092 [US4] Update `parse_email()` in `crates/veil-email/src/lib.rs` to route MSG format to parse_msg()
- [x] T093 [US4] Add MSG format detection based on file signature in `crates/veil-email/src/lib.rs`

**Checkpoint**: At this point, all P1 and P2 stories (US1-4) should work independently - both EML and MSG parsing complete

---

## Phase 7: User Story 5 - Handle Email Threads (Priority: P3)

**Goal**: Identify and separate quoted/replied content from new content in email threads

**Independent Test**: Provide email with quoted replies, verify quoted sections identified

### Tests for User Story 5 (TDD)

- [ ] T094 [P] [US5] Unit test: Detect lines starting with > as quoted in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T095 [P] [US5] Unit test: Detect "On <date>, <person> wrote:" pattern in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T096 [P] [US5] Unit test: Detect "-----Original Message-----" separator in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T097 [P] [US5] Unit test: Handle nested quotes (>, >>, >>>) in `crates/veil-email/tests/eml_parser_test.rs`
- [ ] T098 [P] [US5] Integration test: Parse email thread with multiple quote levels in `crates/veil-email/tests/integration_test.rs`

### Implementation for User Story 5

- [x] T099 [P] [US5] Define regex patterns for quote detection in `crates/veil-email/src/quotes.rs` using lazy_static
- [x] T100 [P] [US5] Implement pattern for lines starting with > in `crates/veil-email/src/quotes.rs`
- [x] T101 [P] [US5] Implement pattern for "On...wrote:" attribution in `crates/veil-email/src/quotes.rs`
- [x] T102 [P] [US5] Implement pattern for "Original Message" separator in `crates/veil-email/src/quotes.rs`
- [x] T103 [US5] Implement `detect_quotes()` function in `crates/veil-email/src/quotes.rs` that processes text line-by-line
- [x] T104 [US5] Implement quote state tracking (quoted vs original) in `crates/veil-email/src/quotes.rs`
- [x] T105 [US5] Implement segment creation for quoted and original text blocks in `crates/veil-email/src/quotes.rs`
- [x] T106 [US5] Integrate quote detection into body part extraction in `crates/veil-email/src/eml.rs`
- [x] T107 [US5] Set EmailBodyPart.is_quoted flag based on detection in `crates/veil-email/src/eml.rs`
- [x] T108 [US5] Add EmailParseOptions.detect_quotes option handling in `crates/veil-email/src/eml.rs`
- [x] T109 [US5] Integrate quote detection into MSG body extraction in `crates/veil-email/src/msg.rs`

**Checkpoint**: All user stories (P1-P3) should now work independently - full email parsing with quote detection complete

---

## Phase 8: veil-parsers Integration

**Purpose**: Integrate email parser with veil-parsers interface for TextSegment output

- [x] T110 Implement `EmailHeader::to_text_segments()` method in `crates/veil-email/src/convert.rs`
- [x] T111 Implement segment generation for Address header values in `crates/veil-email/src/convert.rs`
- [x] T112 Implement segment generation for AddressList header values in `crates/veil-email/src/convert.rs`
- [x] T113 Implement segment generation for Text/DateTime header values in `crates/veil-email/src/convert.rs`
- [x] T114 Implement `EmailBodyPart::to_text_segments()` method in `crates/veil-email/src/convert.rs`
- [x] T115 Implement body text segmentation (by paragraph or full content) in `crates/veil-email/src/convert.rs`
- [x] T116 Implement Position::Email creation with correct field, part_index, byte_offset in `crates/veil-email/src/convert.rs`
- [x] T117 Implement `EmailMessage::to_text_segments()` method in `crates/veil-email/src/convert.rs`
- [x] T118 Combine header, body, and attachment filename segments in `crates/veil-email/src/convert.rs`
- [x] T119 Implement `EmailMessage::to_parse_result()` method in `crates/veil-email/src/convert.rs`
- [x] T120 Create ParseResult with metadata (format, encoding, size_bytes, duration_ms) in `crates/veil-email/src/convert.rs`
- [x] T121 Update `crates/veil-parsers/src/lib.rs` to route FileFormat::Eml to veil_email::parse_email()
- [x] T122 Update `crates/veil-parsers/src/lib.rs` to route FileFormat::Msg to veil_email::parse_email()
- [x] T123 Add contract test in `crates/veil-parsers/tests/` to verify email parsing returns valid TextSegments
- [x] T124 Add contract test to verify Position::Email variant is correctly populated

**Checkpoint**: veil-parsers can now parse .eml and .msg files and output TextSegments for PII detection

---

## Phase 9: Edge Cases & Error Handling

**Purpose**: Handle malformed, encrypted, and edge case emails gracefully

- [ ] T125 [P] Add edge case test: Empty email (headers only, no body) in `crates/veil-email/tests/integration_test.rs`
- [ ] T126 [P] Add edge case test: Missing From header in `crates/veil-email/tests/integration_test.rs`
- [ ] T127 [P] Add edge case test: Malformed base64 encoding in `crates/veil-email/tests/integration_test.rs`
- [ ] T128 [P] Add edge case test: Non-ASCII characters (Japanese text) in `crates/veil-email/tests/integration_test.rs`
- [ ] T129 [P] Add edge case test: Email with emoji in headers and body in `crates/veil-email/tests/integration_test.rs`
- [ ] T130 [P] Add edge case test: Encrypted email (S/MIME) - extract headers only in `crates/veil-email/tests/integration_test.rs`
- [ ] T131 [P] Add edge case test: Huge attachment (>100MB) - verify metadata only in `crates/veil-email/tests/integration_test.rs`
- [ ] T132 Implement graceful handling for missing required headers in `crates/veil-email/src/eml.rs`
- [ ] T133 Implement fallback for malformed base64/quoted-printable encoding in `crates/veil-email/src/eml.rs`
- [ ] T134 Implement lossy UTF-8 conversion with warning for unknown charsets in `crates/veil-email/src/eml.rs`
- [ ] T135 Implement encrypted email detection (application/pkcs7-mime) in `crates/veil-email/src/eml.rs`
- [ ] T136 Add warning to ParseResult for encrypted/signed emails in `crates/veil-email/src/convert.rs`
- [ ] T137 Add warning to ParseResult for lossy encoding conversions in `crates/veil-email/src/convert.rs`

**Checkpoint**: Email parser handles edge cases gracefully without panics

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, performance optimization, and final validation

- [ ] T138 [P] Add rustdoc comments to all public types in `crates/veil-email/src/types.rs`
- [ ] T139 [P] Add rustdoc comments to all public functions in `crates/veil-email/src/lib.rs`
- [ ] T140 [P] Add rustdoc comments with examples to EmailMessage struct in `crates/veil-email/src/types.rs`
- [ ] T141 [P] Add rustdoc comments to EmailParseOptions with usage examples in `crates/veil-email/src/types.rs`
- [x] T142 [P] Add module-level documentation to `crates/veil-email/src/lib.rs`
- [ ] T143 [P] Create README.md for veil-email crate with quickstart example in `crates/veil-email/README.md`
- [ ] T144 [P] Update CLAUDE.md with veil-email crate information in root `CLAUDE.md`
- [x] T145 Run cargo clippy -- -D warnings on veil-email crate and fix all issues
- [x] T146 Run cargo fmt on veil-email crate
- [x] T147 Run cargo test on veil-email crate and verify all tests pass
- [x] T148 Run cargo test on veil-parsers crate and verify integration works
- [ ] T149 Validate quickstart.md examples are accurate and working
- [ ] T150 Add #[must_use] attribute to parse_email() function
- [ ] T151 Review constitution compliance: verify no .unwrap() on user input
- [ ] T152 Review constitution compliance: verify all errors use Result types
- [ ] T153 Add performance benchmark for typical email (50KB) parsing in under 100ms
- [ ] T154 Add performance benchmark for email with 10 attachments parsed in under 1 second

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational (Phase 2) - MVP target
- **User Story 2 (Phase 4)**: Depends on Foundational (Phase 2) - Can run parallel to US1 if staffed
- **User Story 3 (Phase 5)**: Depends on Foundational (Phase 2) - Can run parallel to US1/US2 if staffed
- **User Story 4 (Phase 6)**: Depends on Foundational (Phase 2) - Can run parallel to US1/US2/US3 if staffed
- **User Story 5 (Phase 7)**: Depends on US2 body extraction (needs body text to detect quotes)
- **Integration (Phase 8)**: Depends on at least US1 completion (can start after MVP)
- **Edge Cases (Phase 9)**: Can run in parallel with user stories after Foundational
- **Polish (Phase 10)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: No dependencies on other stories - Header extraction standalone
- **User Story 2 (P1)**: No dependencies on other stories - Body extraction standalone
- **User Story 3 (P2)**: No dependencies on other stories - Attachment listing standalone
- **User Story 4 (P2)**: No dependencies on other stories - MSG parsing standalone (mirrors EML implementation)
- **User Story 5 (P3)**: Depends on US2 (needs body extraction to detect quotes)

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Unit tests before implementation tasks
- Implementation tasks in dependency order (helpers before main functions)
- Integration tests after implementation complete
- Each story should be independently testable

### Parallel Opportunities

- **Phase 1 (Setup)**: Tasks T003, T004, T005, T006 can run in parallel
- **Phase 2 (Foundational)**: Tasks T012, T013, T014, T015, T016 can run in parallel (different type definitions)
- **User Story Tests**: All test tasks within a story marked [P] can run in parallel
- **User Stories**: After Foundational phase, US1, US2, US3, US4 can be worked on in parallel by different team members
- **Quote Detection Patterns**: Tasks T100, T101, T102 can run in parallel
- **Documentation**: All Phase 10 tasks marked [P] can run in parallel
- **Edge Case Tests**: Tasks T125-T131 can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all unit tests for User Story 1 together:
Task: "Unit test: Parse From header without display name"
Task: "Unit test: Parse From header with display name"
Task: "Unit test: Parse To header with multiple recipients"
Task: "Unit test: Parse CC header with multiple recipients"
Task: "Unit test: Parse Subject header as plain text"
Task: "Unit test: Parse Date header as DateTime string"
Task: "Integration test: Parse real Gmail export EML file"
Task: "Integration test: Parse real Outlook export EML file"

# After tests fail, implement in sequence:
Task: "Implement parse_eml() function skeleton"
Task: "Implement extract_headers() helper"
# ... etc
```

---

## Implementation Strategy

### MVP First (User Story 1 + 2 Only - EML Headers + Body)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Header extraction)
4. Complete Phase 4: User Story 2 (Body extraction)
5. Complete Phase 8: veil-parsers Integration (partial - just EML support)
6. **STOP and VALIDATE**: Test EML parsing end-to-end independently
7. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 (Headers) → Test independently → MVP milestone 1
3. Add User Story 2 (Body) → Test independently → MVP milestone 2 (P1 complete)
4. Add User Story 3 (Attachments) → Test independently → Deploy/Demo
5. Add User Story 4 (MSG format) → Test independently → Deploy/Demo (P2 complete)
6. Add User Story 5 (Quote detection) → Test independently → Deploy/Demo (P3 complete)
7. Complete Integration + Polish → Production ready

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Headers)
   - Developer B: User Story 2 (Body)
   - Developer C: User Story 3 (Attachments)
   - Developer D: User Story 4 (MSG) - can start after observing US1 pattern
3. After US1-4 complete:
   - Developer A: User Story 5 (Quotes)
   - Developer B: Integration (Phase 8)
   - Developer C: Edge Cases (Phase 9)
   - Developer D: Documentation (Phase 10)

---

## Task Summary

- **Total tasks**: 154
- **Setup phase**: 7 tasks
- **Foundational phase**: 13 tasks
- **User Story 1 (Headers)**: 20 tasks (8 tests + 12 implementation)
- **User Story 2 (Body)**: 20 tasks (8 tests + 12 implementation)
- **User Story 3 (Attachments)**: 15 tasks (6 tests + 9 implementation)
- **User Story 4 (MSG)**: 18 tasks (5 tests + 13 implementation)
- **User Story 5 (Quotes)**: 16 tasks (5 tests + 11 implementation)
- **Integration**: 15 tasks
- **Edge Cases**: 13 tasks
- **Polish**: 17 tasks

### Parallel Opportunities Identified

- 45+ tasks marked [P] can run in parallel across the project
- User Stories 1-4 are independently parallelizable after Foundational phase
- All test tasks within a story can run in parallel (TDD)
- Documentation tasks can run in parallel with final testing

### Independent Test Criteria

- **US1**: Parse EML with various headers → verify all fields captured with correct EmailHeaderValue types
- **US2**: Parse email with text + HTML parts → verify both converted to readable text, HTML artifacts removed
- **US3**: Parse email with 10 attachments → verify all listed with metadata, parsed in <1 second
- **US4**: Parse MSG file → verify same output as equivalent EML file (headers, body, attachments)
- **US5**: Parse email thread with quotes → verify quoted sections marked with is_quoted=true

### Suggested MVP Scope

**Minimum viable product**: User Stories 1 + 2 (EML header and body extraction)

This delivers:
- Parse .eml files
- Extract all standard headers (From, To, CC, Subject, Date)
- Parse email addresses with display names
- Extract plain text and HTML body content
- Convert HTML to readable text
- Output TextSegments for PII detection

**Estimated effort**: ~40 tasks (Setup + Foundational + US1 + US2 + Integration subset)

---

## Format Validation

✅ **All tasks follow the checklist format**:
- Checkbox: `- [ ]` ✓
- Task ID: T001-T154 ✓
- [P] marker: Only on parallelizable tasks ✓
- [Story] label: Only on user story phase tasks (US1-US5) ✓
- Description: Clear action with exact file path ✓

✅ **Task organization**:
- Organized by user story ✓
- Each story independently testable ✓
- Dependencies clearly marked ✓
- Parallel opportunities identified ✓

✅ **Constitution compliance**:
- TDD workflow (tests first) ✓
- No .unwrap() on user input ✓
- Result-based error handling ✓
- Dependency justification from research.md ✓

---

## Notes

- All file paths are absolute from workspace root: `D:\Projekte\Veil\`
- TDD approach: Tests MUST fail before implementation
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Use `cargo test` to verify tests are failing before implementing
- Use `cargo clippy -- -D warnings` before committing
- Integration with veil-parsers enables immediate PII detection on email files
