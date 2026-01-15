# Tasks: Identity Document Detection

**Input**: Design documents from `/specs/020-identity-document-detection/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests included based on TDD requirement from constitution.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md - Single crate modification:
- `crates/veil-detect/src/patterns/ssn.rs` - SSN detector (NEW)
- `crates/veil-detect/src/patterns/passport.rs` - Passport detector (NEW)
- `crates/veil-detect/src/patterns/drivers_license.rs` - DL detector (NEW)
- `crates/veil-detect/src/category.rs` - PII categories
- `crates/veil-detect/src/patterns/mod.rs` - Module exports

---

## Phase 1: Setup

**Purpose**: Verify baseline before making changes

- [x] T001 Verify all existing tests pass with `cargo test -p veil-detect`
- [x] T002 Read current patterns/mod.rs to understand module structure

---

## Phase 2: Foundational (PII Categories)

**Purpose**: Add new PII categories that all detectors will use

- [x] T003 Add `Ssn` variant to PiiCategory enum in crates/veil-detect/src/category.rs
- [x] T004 Add `Passport` variant to PiiCategory enum in crates/veil-detect/src/category.rs
- [x] T005 Add `DriversLicense` variant to PiiCategory enum in crates/veil-detect/src/category.rs
- [x] T006 Add Display implementations for new categories in crates/veil-detect/src/category.rs
- [x] T007 Add as_str implementations for new categories in crates/veil-detect/src/category.rs
- [x] T008 Verify project compiles after category changes with `cargo build -p veil-detect`

**Checkpoint**: New PII categories available for detectors ✅

---

## Phase 3: User Story 1 - Detect US Social Security Numbers (Priority: P1) MVP

**Goal**: Detect SSNs in hyphenated and space-separated formats with validation

**Independent Test**: Scan text with SSNs, verify detection and validation

### Tests for User Story 1

- [x] T009 [P] [US1] Add test: detect SSN hyphenated format 123-45-6789 in crates/veil-detect/src/patterns/ssn.rs
- [x] T010 [P] [US1] Add test: detect SSN space format 123 45 6789 in crates/veil-detect/src/patterns/ssn.rs
- [x] T011 [P] [US1] Add test: validate SSN rejects area 000 in crates/veil-detect/src/patterns/ssn.rs
- [x] T012 [P] [US1] Add test: validate SSN rejects area 666 in crates/veil-detect/src/patterns/ssn.rs
- [x] T013 [P] [US1] Add test: validate SSN rejects group 00 in crates/veil-detect/src/patterns/ssn.rs
- [x] T014 [P] [US1] Add test: validate SSN rejects serial 0000 in crates/veil-detect/src/patterns/ssn.rs

### Implementation for User Story 1

- [x] T015 [US1] Create ssn.rs with module doc and imports in crates/veil-detect/src/patterns/ssn.rs
- [x] T016 [US1] Add SSN_PATTERNS static with hyphenated and space regexes in crates/veil-detect/src/patterns/ssn.rs
- [x] T017 [US1] Add INVALID_AREAS constant for area validation in crates/veil-detect/src/patterns/ssn.rs
- [x] T018 [US1] Implement SsnDetector struct with new() and Default in crates/veil-detect/src/patterns/ssn.rs
- [x] T019 [US1] Implement Detector trait for SsnDetector (name, category, base_confidence) in crates/veil-detect/src/patterns/ssn.rs
- [x] T020 [US1] Implement detect() method with overlap prevention in crates/veil-detect/src/patterns/ssn.rs
- [x] T021 [US1] Implement validate() method with area/group/serial checks in crates/veil-detect/src/patterns/ssn.rs
- [x] T022 [US1] Export SsnDetector in crates/veil-detect/src/patterns/mod.rs
- [x] T023 [US1] Verify all SSN tests pass with `cargo test -p veil-detect ssn`

**Checkpoint**: SSN detection works for all common formats with validation ✅

---

## Phase 4: User Story 2 - Detect US Passport Numbers (Priority: P1)

**Goal**: Detect US passport numbers (9 digits, alphanumeric variants)

**Independent Test**: Scan text with US passport numbers, verify detection

### Tests for User Story 2

- [x] T024 [P] [US2] Add test: detect US passport 9-digit format in crates/veil-detect/src/patterns/passport.rs
- [x] T025 [P] [US2] Add test: detect US passport alphanumeric A12345678 in crates/veil-detect/src/patterns/passport.rs
- [x] T026 [P] [US2] Add test: validate passport length 6-9 chars in crates/veil-detect/src/patterns/passport.rs

### Implementation for User Story 2

- [x] T027 [US2] Create passport.rs with module doc and imports in crates/veil-detect/src/patterns/passport.rs
- [x] T028 [US2] Add PASSPORT_PATTERNS static with US patterns in crates/veil-detect/src/patterns/passport.rs
- [x] T029 [US2] Implement PassportDetector struct with new() and Default in crates/veil-detect/src/patterns/passport.rs
- [x] T030 [US2] Implement Detector trait for PassportDetector in crates/veil-detect/src/patterns/passport.rs
- [x] T031 [US2] Implement detect() method in crates/veil-detect/src/patterns/passport.rs
- [x] T032 [US2] Implement validate() method with length check in crates/veil-detect/src/patterns/passport.rs
- [x] T033 [US2] Export PassportDetector in crates/veil-detect/src/patterns/mod.rs
- [x] T034 [US2] Verify all passport tests pass with `cargo test -p veil-detect passport`

**Checkpoint**: US passport numbers detected ✅

---

## Phase 5: User Story 3 - Detect UK/EU Passport Numbers (Priority: P2)

**Goal**: Extend passport detection for UK and EU formats

**Independent Test**: Scan text with UK/EU passports, verify detection

### Tests for User Story 3

- [x] T035 [P] [US3] Add test: detect UK passport 9-digit in crates/veil-detect/src/patterns/passport.rs
- [x] T036 [P] [US3] Add test: detect German passport alphanumeric in crates/veil-detect/src/patterns/passport.rs
- [x] T037 [P] [US3] Add test: detect French passport alphanumeric in crates/veil-detect/src/patterns/passport.rs

### Implementation for User Story 3

- [x] T038 [US3] Add generic alphanumeric pattern to PASSPORT_PATTERNS in crates/veil-detect/src/patterns/passport.rs
- [x] T039 [US3] Verify all EU passport tests pass with `cargo test -p veil-detect passport`

**Checkpoint**: UK and EU passport numbers detected ✅

---

## Phase 6: User Story 4 - Detect Driver's License Numbers (Priority: P2)

**Goal**: Detect driver's license numbers from major US states (CA, NY, TX, FL, IL)

**Independent Test**: Scan text with DL numbers, verify detection

### Tests for User Story 4

- [x] T040 [P] [US4] Add test: detect California DL (1 letter + 7 digits) in crates/veil-detect/src/patterns/drivers_license.rs
- [x] T041 [P] [US4] Add test: detect Texas DL (8 digits) in crates/veil-detect/src/patterns/drivers_license.rs
- [x] T042 [P] [US4] Add test: detect Florida DL (1 letter + 12 digits) in crates/veil-detect/src/patterns/drivers_license.rs
- [x] T043 [P] [US4] Add test: detect Illinois DL (1 letter + 11 digits) in crates/veil-detect/src/patterns/drivers_license.rs
- [x] T044 [P] [US4] Add test: validate DL length 7-13 chars in crates/veil-detect/src/patterns/drivers_license.rs

### Implementation for User Story 4

- [x] T045 [US4] Create drivers_license.rs with module doc and imports in crates/veil-detect/src/patterns/drivers_license.rs
- [x] T046 [US4] Add DL_PATTERNS static with state-specific regexes in crates/veil-detect/src/patterns/drivers_license.rs
- [x] T047 [US4] Implement DriversLicenseDetector struct in crates/veil-detect/src/patterns/drivers_license.rs
- [x] T048 [US4] Implement Detector trait for DriversLicenseDetector in crates/veil-detect/src/patterns/drivers_license.rs
- [x] T049 [US4] Implement detect() method in crates/veil-detect/src/patterns/drivers_license.rs
- [x] T050 [US4] Implement validate() method with length check in crates/veil-detect/src/patterns/drivers_license.rs
- [x] T051 [US4] Export DriversLicenseDetector in crates/veil-detect/src/patterns/mod.rs
- [x] T052 [US4] Verify all DL tests pass with `cargo test -p veil-detect drivers_license`

**Checkpoint**: Driver's license numbers from major states detected ✅

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and documentation

- [x] T053 Run full test suite: `cargo test --workspace`
- [x] T054 Run clippy: `cargo clippy --workspace -- -D warnings`
- [x] T055 [P] Add integration test with mixed identity documents in crates/veil-detect/src/patterns/ssn.rs
- [x] T056 [P] Verify no overlapping matches between detectors
- [x] T057 Update CHANGELOG.md with new identity document detection capabilities

**Checkpoint**: All tasks complete ✅

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - verify baseline
- **Foundational (Phase 2)**: Depends on Setup - adds PII categories
- **User Stories (Phase 3-6)**: All depend on Foundational
  - US1 (SSN) and US2 (US Passport) are both P1 - can run in parallel
  - US3 (EU Passport) depends on US2 (extends passport patterns)
  - US4 (Driver's License) is independent - can run in parallel with others
- **Polish (Phase 7)**: Depends on all user stories complete

### User Story Dependencies

- **User Story 1 (SSN)**: Independent - creates new ssn.rs
- **User Story 2 (US Passport)**: Independent - creates new passport.rs
- **User Story 3 (EU Passport)**: Depends on US2 - extends passport.rs
- **User Story 4 (Driver's License)**: Independent - creates new drivers_license.rs

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Pattern additions are independent within each story
- Verification test at end of each story

### Parallel Opportunities

**After Foundational Phase completes:**
- US1, US2, US4 can run in parallel (different files)
- US3 must wait for US2 (extends same file)
- All test tasks within a story can run in parallel

---

## Parallel Example: SSN Tests

```bash
# Launch all SSN tests together (different test functions):
Task: "T009 [P] [US1] Add test: detect SSN hyphenated format"
Task: "T010 [P] [US1] Add test: detect SSN space format"
Task: "T011 [P] [US1] Add test: validate SSN rejects area 000"
Task: "T012 [P] [US1] Add test: validate SSN rejects area 666"
Task: "T013 [P] [US1] Add test: validate SSN rejects group 00"
Task: "T014 [P] [US1] Add test: validate SSN rejects serial 0000"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Complete Phase 1: Setup (verify baseline)
2. Complete Phase 2: Foundational (PII categories)
3. Complete Phase 3: User Story 1 (SSN detection)
4. **STOP and VALIDATE**: SSN detection works
5. Can deploy as incremental improvement for HIPAA compliance

### Full Feature

1. MVP (above)
2. Add User Story 2: US passport numbers
3. Add User Story 3: UK/EU passport numbers
4. Add User Story 4: Driver's license numbers
5. Complete Phase 7: Polish
6. Full release

---

## Summary

- **Total Tasks**: 57
- **US1 (SSN)**: 15 tasks
- **US2 (US Passport)**: 11 tasks
- **US3 (EU Passport)**: 5 tasks
- **US4 (Driver's License)**: 13 tasks
- **Setup/Foundational**: 8 tasks
- **Polish**: 5 tasks

**MVP Scope**: US1 (P1) = 15 tasks for SSN detection (HIPAA critical)

---

## Notes

- [P] tasks = different files/test functions, no dependencies
- [Story] label maps task to specific user story
- SSN validation includes area number checks per SSA rules
- Passport detection requires context to avoid false positives
- Driver's license formats vary by state - focus on high-population states
- All patterns compile at startup - no runtime errors
- Existing tests are regression baseline
