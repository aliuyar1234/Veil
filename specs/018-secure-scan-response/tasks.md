# Tasks: Secure Scan Response (PII-Safe API)

**Input**: Design documents from `/specs/018-secure-scan-response/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests included based on TDD requirement from constitution (Test-First Development)

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md - Multi-crate workspace:
- `crates/veil-api/src/` - API server
- `crates/veil-cli/src/` - CLI tool
- `crates/veil-wasm/src/` - WASM bindings

---

## Phase 1: Setup

**Purpose**: No new project setup needed - modifying existing crates

- [x] T001 Verify all existing tests pass with `cargo test --workspace`
- [x] T002 Create feature branch tracking file in specs/018-secure-scan-response/

---

## Phase 2: Foundational (Shared Model Changes)

**Purpose**: Core data model changes that affect multiple interfaces

**CRITICAL**: These changes MUST be complete before user story implementation

- [x] T003 Add `include_values` field to API ScanOptions in crates/veil-api/src/models.rs
- [x] T004 [P] Change Finding.value from `String` to `Option<String>` with serde skip_serializing_if in crates/veil-api/src/models.rs
- [x] T005 [P] Add `include_values` flag to CLI ScanArgs in crates/veil-cli/src/cli.rs
- [x] T006 [P] Change FindingOutput.text from `String` to `Option<String>` in crates/veil-cli/src/commands/scan.rs
- [x] T007 [P] Add `include_values` and `acknowledge_exposure` to WASM ScanOptions in crates/veil-wasm/src/types.rs
- [x] T008 [P] Change Finding.value from `String` to `Option<String>` in crates/veil-wasm/src/types.rs

**Checkpoint**: Data models updated - implementation can proceed

---

## Phase 3: User Story 1 - Secure Scan Without PII Exposure (Priority: P1) MVP

**Goal**: API scan responses exclude PII values by default

**Independent Test**: Call scan API without include_values, verify no value field in response

### Tests for User Story 1

- [x] T009 [P] [US1] Add test: scan response excludes value field by default in crates/veil-api/src/routes/scan.rs
- [x] T010 [P] [US1] Add test: response JSON has no value key when include_values=false in crates/veil-api/src/routes/scan.rs

### Implementation for User Story 1

- [x] T011 [US1] Modify scan_file handler to check include_values option in crates/veil-api/src/routes/scan.rs
- [x] T012 [US1] Update Finding construction to set value=None when include_values=false in crates/veil-api/src/routes/scan.rs
- [x] T013 [US1] Verify no PII in server logs - review tracing statements in crates/veil-api/src/routes/scan.rs

**Checkpoint**: API returns findings without PII values by default

---

## Phase 4: User Story 2 - Explicit Opt-In for PII Values (Priority: P2)

**Goal**: Enable PII values in response only with explicit acknowledgment header

**Independent Test**: Call API with include_values=true, verify 400 without header, 200 with header

### Tests for User Story 2

- [x] T014 [P] [US2] Add test: 400 response when include_values=true without header in crates/veil-api/src/routes/scan.rs
- [x] T015 [P] [US2] Add test: values included when header X-Acknowledge-PII-Exposure: accepted present in crates/veil-api/src/routes/scan.rs
- [x] T016 [P] [US2] Add test: 400 response when header has wrong value in crates/veil-api/src/routes/scan.rs

### Implementation for User Story 2

- [x] T017 [US2] Extract X-Acknowledge-PII-Exposure header in scan_file handler in crates/veil-api/src/routes/scan.rs
- [x] T018 [US2] Add validation: return ApiError::BadRequest if include_values=true without header in crates/veil-api/src/routes/scan.rs
- [x] T019 [US2] Update Finding construction to include value when acknowledgment valid in crates/veil-api/src/routes/scan.rs
- [x] T020 [US2] Add clear error message explaining security requirement in crates/veil-api/src/error.rs

**Checkpoint**: API acknowledgment mechanism working

---

## Phase 5: User Story 3 - CLI Safe Output Mode (Priority: P1)

**Goal**: CLI scan output excludes PII values by default, requires confirmation for --include-values

**Independent Test**: Run `veil scan file.txt`, verify no PII in output

### Tests for User Story 3

- [x] T021 [P] [US3] Add test: FindingOutput serializes without text by default in crates/veil-cli/src/commands/scan.rs
- [x] T022 [P] [US3] Add test: scan output format excludes matched text in crates/veil-cli/src/commands/scan.rs

### Implementation for User Story 3

- [x] T023 [US3] Modify FindingOutput creation to set text=None by default in crates/veil-cli/src/commands/scan.rs
- [x] T024 [US3] Update output::print_scan_result to handle optional text in crates/veil-cli/src/output.rs
- [x] T025 [US3] Add confirmation prompt when --include-values flag used in crates/veil-cli/src/commands/scan.rs
- [x] T026 [US3] Add --yes flag to bypass confirmation for scripted use in crates/veil-cli/src/cli.rs
- [x] T027 [US3] Update JSON output to exclude text field when not requested in crates/veil-cli/src/commands/scan.rs

**Checkpoint**: CLI outputs findings without PII values by default

---

## Phase 6: User Story 4 - WASM Secure Response (Priority: P2)

**Goal**: WASM scan results exclude PII values by default, require acknowledgment option

**Independent Test**: Call scan() from JS, verify no value property in findings

### Tests for User Story 4

- [x] T028 [P] [US4] Add test: scan returns findings without value by default in crates/veil-wasm/src/scan.rs
- [x] T029 [P] [US4] Add test: error when includeValues=true without acknowledgeExposure in crates/veil-wasm/src/scan.rs
- [x] T030 [P] [US4] Add test: values included when both options true in crates/veil-wasm/src/scan.rs

### Implementation for User Story 4

- [x] T031 [US4] Modify perform_scan to check includeValues option in crates/veil-wasm/src/scan.rs
- [x] T032 [US4] Add validation: return WasmError if includeValues without acknowledgeExposure in crates/veil-wasm/src/scan.rs
- [x] T033 [US4] Update Finding::new call to conditionally include value in crates/veil-wasm/src/scan.rs
- [x] T034 [US4] Update Finding struct serialization for optional value in crates/veil-wasm/src/types.rs

**Checkpoint**: WASM returns findings without PII values by default

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, validation, and final cleanup

- [x] T035 [P] Update CHANGELOG.md with breaking change notice
- [x] T036 [P] Verify migration guide in specs/018-secure-scan-response/contracts/api-changes.md is complete
- [x] T037 Run full test suite: `cargo test --workspace`
- [x] T038 Run clippy: `cargo clippy --workspace -- -D warnings`
- [x] T039 Manual API test: verify default response has no value field
- [x] T040 Manual CLI test: verify output format without --include-values
- [x] T041 Review all changes for any remaining PII in logs or error messages

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - verify baseline
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational completion
  - US1 and US3 are both P1, can run in parallel
  - US2 depends on US1 (API changes)
  - US4 can run in parallel with others
- **Polish (Phase 7)**: Depends on all user stories complete

### User Story Dependencies

- **User Story 1 (API Default)**: Can start after Foundational - Independent
- **User Story 2 (API Opt-in)**: Depends on US1 - Extends the handler
- **User Story 3 (CLI)**: Can start after Foundational - Independent of API stories
- **User Story 4 (WASM)**: Can start after Foundational - Independent

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Model changes before handler changes
- Validation before feature logic
- Story complete before moving to next

### Parallel Opportunities

**After Foundational Phase completes:**
- US1 (API) and US3 (CLI) can run in parallel (different crates)
- US4 (WASM) can run in parallel with others (different crate)
- US2 should follow US1 (same file, extends functionality)

---

## Parallel Example: Foundational Phase

```bash
# Launch all model updates together (different files):
Task: "T004 [P] Change Finding.value in veil-api/src/models.rs"
Task: "T005 [P] Add include_values to CLI ScanArgs in veil-cli/src/cli.rs"
Task: "T006 [P] Change FindingOutput.text in veil-cli/src/commands/scan.rs"
Task: "T007 [P] Add options to WASM ScanOptions in veil-wasm/src/types.rs"
Task: "T008 [P] Change Finding.value in veil-wasm/src/types.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 + 3)

1. Complete Phase 1: Setup (verify baseline)
2. Complete Phase 2: Foundational (model changes)
3. Complete Phase 3: User Story 1 (API default behavior)
4. Complete Phase 5: User Story 3 (CLI default behavior)
5. **STOP and VALIDATE**: Both API and CLI secure by default
6. Deploy as security patch

### Full Feature

1. MVP (above)
2. Add User Story 2: API opt-in with acknowledgment
3. Add User Story 4: WASM secure by default
4. Complete Phase 7: Polish
5. Full release

---

## Summary

- **Total Tasks**: 41
- **US1 (API Default)**: 5 tasks
- **US2 (API Opt-in)**: 7 tasks
- **US3 (CLI)**: 7 tasks
- **US4 (WASM)**: 7 tasks
- **Setup/Foundational**: 8 tasks
- **Polish**: 7 tasks

**MVP Scope**: US1 + US3 (P1 priorities) = 12 implementation tasks + 8 foundational

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Each user story is independently testable
- Constitution requires tests fail before implementation
- Commit after each task or logical group
