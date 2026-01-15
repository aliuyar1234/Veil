# Tasks: PII Memory Zeroization

**Input**: Design documents from `/specs/021-pii-memory-zeroization/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests included based on TDD requirement from constitution.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md - Multi-crate workspace:
- `crates/veil-core/src/` - NEW: Shared types (SensitiveString)
- `crates/veil-detect/src/finding.rs` - Finding struct modification
- `crates/veil-parsers/src/types.rs` - TextSegment modification
- `crates/veil-api/src/routes/` - API response cleanup

---

## Phase 1: Setup

**Purpose**: Create veil-core crate and verify baseline

- [x] T001 Verify all existing tests pass with `cargo test --workspace`
- [x] T002 Create veil-core crate directory structure in crates/veil-core/
- [x] T003 Create Cargo.toml for veil-core with zeroize and serde dependencies in crates/veil-core/Cargo.toml
- [x] T004 Add veil-core to workspace members in Cargo.toml

---

## Phase 2: Foundational (SensitiveString Type)

**Purpose**: Create the core SensitiveString type that all user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Tests for Foundational

- [x] T005 [P] Add test: SensitiveString creation from &str in crates/veil-core/src/sensitive.rs
- [x] T006 [P] Add test: SensitiveString creation from String in crates/veil-core/src/sensitive.rs
- [x] T007 [P] Add test: SensitiveString deref to &str in crates/veil-core/src/sensitive.rs
- [x] T008 [P] Add test: SensitiveString clone creates independent copy in crates/veil-core/src/sensitive.rs
- [x] T009 [P] Add test: SensitiveString drop zeros memory in crates/veil-core/src/sensitive.rs
- [x] T010 [P] Add test: SensitiveString debug output is redacted in crates/veil-core/src/sensitive.rs
- [x] T011 [P] Add test: SensitiveString serialization in crates/veil-core/src/sensitive.rs

### Implementation for Foundational

- [x] T012 Create lib.rs with module exports in crates/veil-core/src/lib.rs
- [x] T013 Create sensitive.rs with SensitiveString struct in crates/veil-core/src/sensitive.rs
- [x] T014 Implement From<String> and From<&str> for SensitiveString in crates/veil-core/src/sensitive.rs
- [x] T015 Implement Deref<Target=str> for SensitiveString in crates/veil-core/src/sensitive.rs
- [x] T016 Implement Drop with zeroize for SensitiveString in crates/veil-core/src/sensitive.rs
- [x] T017 Implement Debug with redaction for SensitiveString in crates/veil-core/src/sensitive.rs
- [x] T018 Implement Serialize/Deserialize for SensitiveString in crates/veil-core/src/sensitive.rs
- [x] T019 Implement Clone, PartialEq, Eq, Hash for SensitiveString in crates/veil-core/src/sensitive.rs
- [x] T020 Implement Default and utility methods (empty, len, is_empty) in crates/veil-core/src/sensitive.rs
- [x] T021 Verify veil-core compiles and tests pass with `cargo test -p veil-core`

**Checkpoint**: SensitiveString type available for all user stories ✅

---

## Phase 3: User Story 1 - Automatic PII Cleanup on Scan Completion (Priority: P1) MVP

**Goal**: Detected PII values (Finding.matched_text) securely erased when Findings are dropped

**Independent Test**: Create Finding, drop it, verify matched_text memory is zeroed

### Tests for User Story 1

- [x] T022 [P] [US1] Add test: Finding drop zeros matched_text in crates/veil-detect/src/finding.rs
- [x] T023 [P] [US1] Add test: Vec<Finding> drop zeros all matched_text in crates/veil-detect/src/finding.rs
- [x] T024 [P] [US1] Add test: Finding clone creates independent SensitiveString in crates/veil-detect/src/finding.rs

### Implementation for User Story 1

- [x] T025 [US1] Add veil-core dependency to veil-detect in crates/veil-detect/Cargo.toml
- [x] T026 [US1] Change matched_text from String to SensitiveString in crates/veil-detect/src/finding.rs
- [x] T027 [US1] Update Finding::new() to accept impl Into<SensitiveString> in crates/veil-detect/src/finding.rs
- [x] T028 [US1] Update all detector detect() calls to create SensitiveString in crates/veil-detect/src/patterns/*.rs
- [x] T029 [US1] Update registry.rs to handle SensitiveString in Finding construction in crates/veil-detect/src/registry.rs
- [x] T030 [US1] Verify all veil-detect tests pass with `cargo test -p veil-detect`

**Checkpoint**: Finding.matched_text is automatically zeroed on drop ✅

---

## Phase 4: User Story 2 - Document Buffer Cleanup (Priority: P1)

**Goal**: Parsed document content (TextSegment.content) securely erased when segments are dropped

**Independent Test**: Parse document, drop result, verify segment content memory is zeroed

### Tests for User Story 2

- [x] T031 [P] [US2] Add test: TextSegment drop zeros content in crates/veil-parsers/src/types.rs
- [x] T032 [P] [US2] Add test: ParseResult drop zeros all segment content in crates/veil-parsers/src/types.rs

### Implementation for User Story 2

- [x] T033 [US2] Add veil-core dependency to veil-parsers in crates/veil-parsers/Cargo.toml
- [x] T034 [US2] Change TextSegment.content from String to SensitiveString in crates/veil-parsers/src/types.rs
- [x] T035 [US2] Update all parsers to create SensitiveString for segment content in crates/veil-parsers/src/*.rs
- [x] T036 [US2] Verify all veil-parsers tests pass with `cargo test -p veil-parsers`

**Checkpoint**: TextSegment.content is automatically zeroed on drop ✅

---

## Phase 5: User Story 3 - Redaction Buffer Cleanup (Priority: P2)

**Goal**: Intermediate buffers during redaction securely erased

**Independent Test**: Perform redaction, verify intermediate buffers are zeroed

### Tests for User Story 3

- [x] T037 [P] [US3] Add test: Redaction intermediate buffers are zeroed in crates/veil-redact/src/lib.rs

### Implementation for User Story 3

- [x] T038 [US3] Add veil-core dependency to veil-redact in crates/veil-redact/Cargo.toml
- [x] T039 [US3] Review redaction code for intermediate String buffers in crates/veil-redact/src/
- [x] T040 [US3] Convert intermediate buffers to SensitiveString where appropriate in crates/veil-redact/src/
- [x] T041 [US3] Verify all veil-redact tests pass with `cargo test -p veil-redact`

**Checkpoint**: Redaction buffers are zeroed after use ✅

---

## Phase 6: User Story 4 - API Response Cleanup (Priority: P2)

**Goal**: API response bodies containing PII zeroed after transmission

**Independent Test**: Make API request, verify response body zeroed in server memory

### Tests for User Story 4

- [x] T042 [P] [US4] Add test: Scan response body is zeroed after handler returns in crates/veil-api/tests/

### Implementation for User Story 4

- [x] T043 [US4] Add veil-core dependency to veil-api in crates/veil-api/Cargo.toml
- [x] T044 [US4] Implement response body cleanup in scan handler in crates/veil-api/src/routes/scan.rs
- [x] T045 [US4] Implement response body cleanup in protect handler in crates/veil-api/src/routes/protect.rs
- [x] T046 [US4] Verify all veil-api tests pass with `cargo test -p veil-api`

**Checkpoint**: API responses are zeroed after transmission ✅

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and integration testing

- [x] T047 Run full test suite: `cargo test --workspace`
- [x] T048 Run clippy: `cargo clippy --workspace -- -D warnings`
- [x] T049 [P] Update CHANGELOG.md with memory zeroization feature
- [x] T050 [P] Verify cross-crate integration (detect uses parsers output correctly)
- [x] T051 Run quickstart.md validation scenarios

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - verify baseline
- **Foundational (Phase 2)**: Depends on Setup - creates SensitiveString type
- **User Stories (Phase 3-6)**: All depend on Foundational
  - US1 (Finding) and US2 (TextSegment) are both P1 - can run in parallel
  - US3 (Redaction) and US4 (API) are P2 - can run after P1 stories
- **Polish (Phase 7)**: Depends on all user stories complete

### User Story Dependencies

- **User Story 1 (Finding)**: Depends only on veil-core (Foundational)
- **User Story 2 (TextSegment)**: Depends only on veil-core (Foundational)
- **User Story 3 (Redaction)**: Depends on US1 (redaction uses Finding)
- **User Story 4 (API)**: Depends on US1 and US2 (API uses both Finding and TextSegment)

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Type changes before function updates
- Core crate changes before dependent crate changes
- Verification test at end of each story

### Parallel Opportunities

**After Foundational Phase completes:**
- US1 and US2 can run in parallel (different crates)
- All test tasks within a story can run in parallel

---

## Parallel Example: Foundational Tests

```bash
# Launch all SensitiveString tests together:
Task: "T005 [P] Add test: SensitiveString creation from &str"
Task: "T006 [P] Add test: SensitiveString creation from String"
Task: "T007 [P] Add test: SensitiveString deref to &str"
Task: "T008 [P] Add test: SensitiveString clone creates independent copy"
Task: "T009 [P] Add test: SensitiveString drop zeros memory"
Task: "T010 [P] Add test: SensitiveString debug output is redacted"
Task: "T011 [P] Add test: SensitiveString serialization"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Complete Phase 1: Setup (verify baseline)
2. Complete Phase 2: Foundational (SensitiveString)
3. Complete Phase 3: User Story 1 (Finding cleanup)
4. **STOP and VALIDATE**: Finding.matched_text zeroization works
5. Can deploy as incremental security improvement

### Full Feature

1. MVP (above)
2. Add User Story 2: TextSegment cleanup
3. Add User Story 3: Redaction buffer cleanup
4. Add User Story 4: API response cleanup
5. Complete Phase 7: Polish
6. Full release

---

## Summary

- **Total Tasks**: 51
- **Setup**: 4 tasks
- **Foundational**: 17 tasks (SensitiveString type)
- **US1 (Finding)**: 9 tasks
- **US2 (TextSegment)**: 6 tasks
- **US3 (Redaction)**: 5 tasks
- **US4 (API)**: 5 tasks
- **Polish**: 5 tasks

**MVP Scope**: Foundational + US1 = 21 tasks for Finding.matched_text zeroization

---

## Notes

- [P] tasks = different files/test functions, no dependencies
- [Story] label maps task to specific user story
- SensitiveString uses existing zeroize crate pattern from veil-crypto
- Debug output is intentionally redacted to prevent PII leakage in logs
- Clone creates independent copy that zeroizes separately
- WASM support is best-effort per zeroize crate limitations
