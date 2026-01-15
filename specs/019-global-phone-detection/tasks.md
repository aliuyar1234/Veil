# Tasks: Global Phone Number Detection

**Input**: Design documents from `/specs/019-global-phone-detection/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests included based on TDD requirement from constitution.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different patterns, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md - Single crate modification:
- `crates/veil-detect/src/patterns/phone.rs` - Primary file

---

## Phase 1: Setup

**Purpose**: Verify baseline before making changes

- [x] T001 Verify all existing phone detection tests pass with `cargo test -p veil-detect phone`
- [x] T002 Read current phone.rs implementation to understand pattern structure

---

## Phase 2: Foundational (Pattern Infrastructure)

**Purpose**: Update module documentation and prepare pattern list structure

- [x] T003 Update module doc comment to reflect global coverage in crates/veil-detect/src/patterns/phone.rs
- [x] T004 Add comments organizing pattern sections (DACH, US, UK, France, E.164) in crates/veil-detect/src/patterns/phone.rs

**Checkpoint**: Module structure ready for new patterns

---

## Phase 3: User Story 1 - Detect US Phone Numbers (Priority: P1) MVP

**Goal**: Detect US phone numbers in NANP formats

**Independent Test**: Scan text with US phone numbers, verify detection

### Tests for User Story 1

- [x] T005 [P] [US1] Add test: detect US E.164 format +1 555 123 4567 in crates/veil-detect/src/patterns/phone.rs
- [x] T006 [P] [US1] Add test: detect US parentheses format (555) 123-4567 in crates/veil-detect/src/patterns/phone.rs
- [x] T007 [P] [US1] Add test: detect US 10-digit format 555-123-4567 in crates/veil-detect/src/patterns/phone.rs
- [x] T008 [P] [US1] Add test: detect US toll-free 1-800-555-1234 in crates/veil-detect/src/patterns/phone.rs

### Implementation for User Story 1

- [x] T009 [US1] Add regex pattern for US E.164: \+1[\s.-]?\d{3}[\s.-]?\d{3}[\s.-]?\d{4} in crates/veil-detect/src/patterns/phone.rs
- [x] T010 [US1] Add regex pattern for US with 1 prefix: 1[\s.-]\d{3}[\s.-]\d{3}[\s.-]\d{4} in crates/veil-detect/src/patterns/phone.rs
- [x] T011 [US1] Add regex pattern for parentheses: \(\d{3}\)[\s.-]?\d{3}[\s.-]?\d{4} in crates/veil-detect/src/patterns/phone.rs
- [x] T012 [US1] Add regex pattern for 10-digit: \d{3}[\s.-]\d{3}[\s.-]\d{4} in crates/veil-detect/src/patterns/phone.rs
- [x] T013 [US1] Verify all US tests pass with cargo test -p veil-detect phone

**Checkpoint**: US phone numbers detected in all common formats

---

## Phase 4: User Story 2 - Detect UK Phone Numbers (Priority: P1)

**Goal**: Detect UK phone numbers in international and local formats

**Independent Test**: Scan text with UK phone numbers, verify detection

### Tests for User Story 2

- [x] T014 [P] [US2] Add test: detect UK E.164 landline +44 20 7946 0958 in crates/veil-detect/src/patterns/phone.rs
- [x] T015 [P] [US2] Add test: detect UK E.164 mobile +44 7911 123456 in crates/veil-detect/src/patterns/phone.rs
- [x] T016 [P] [US2] Add test: detect UK local mobile 07911 123456 in crates/veil-detect/src/patterns/phone.rs

### Implementation for User Story 2

- [x] T017 [US2] Add regex pattern for UK E.164: \+44[\s.-]?\d{2,4}[\s.-]?\d{3,4}[\s.-]?\d{3,6} in crates/veil-detect/src/patterns/phone.rs
- [x] T018 [US2] Add regex pattern for UK local mobile: 07\d{3}[\s.-]?\d{6} in crates/veil-detect/src/patterns/phone.rs
- [x] T019 [US2] Verify all UK tests pass with cargo test -p veil-detect phone

**Checkpoint**: UK phone numbers detected in international and local formats

---

## Phase 5: User Story 3 - Detect International E.164 Format (Priority: P1)

**Goal**: Detect any phone number in E.164 international format

**Independent Test**: Scan text with various country codes, verify detection

### Tests for User Story 3

- [x] T020 [P] [US3] Add test: detect France +33 1 23 45 67 89 in crates/veil-detect/src/patterns/phone.rs
- [x] T021 [P] [US3] Add test: detect Japan +81 3 1234 5678 in crates/veil-detect/src/patterns/phone.rs
- [x] T022 [P] [US3] Add test: detect Australia +61 2 1234 5678 in crates/veil-detect/src/patterns/phone.rs
- [x] T023 [P] [US3] Add test: detect India +91 98765 43210 in crates/veil-detect/src/patterns/phone.rs

### Implementation for User Story 3

- [x] T024 [US3] Add regex pattern for French E.164: \+33[\s.-]?\d[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2} in crates/veil-detect/src/patterns/phone.rs
- [x] T025 [US3] Add regex pattern for generic E.164: \+[1-9]\d{6,14} in crates/veil-detect/src/patterns/phone.rs
- [x] T026 [US3] Verify all E.164 tests pass with cargo test -p veil-detect phone

**Checkpoint**: International phone numbers detected in E.164 format

---

## Phase 6: User Story 4 - Maintain DACH Detection (Priority: P2)

**Goal**: Verify backward compatibility with existing DACH patterns

**Independent Test**: Run existing DACH tests, verify no regressions

### Tests for User Story 4

- [x] T027 [P] [US4] Verify existing test_detect_international_format still passes in crates/veil-detect/src/patterns/phone.rs
- [x] T028 [P] [US4] Verify existing test_detect_german_format still passes in crates/veil-detect/src/patterns/phone.rs
- [x] T029 [P] [US4] Verify existing test_detect_local_format still passes in crates/veil-detect/src/patterns/phone.rs
- [x] T030 [P] [US4] Verify existing test_detect_with_country_prefix still passes in crates/veil-detect/src/patterns/phone.rs

### Implementation for User Story 4

- [x] T031 [US4] Ensure DACH patterns remain at top of pattern list (highest priority) in crates/veil-detect/src/patterns/phone.rs
- [x] T032 [US4] Run full phone test suite to confirm 0 regressions: cargo test -p veil-detect phone

**Checkpoint**: All existing DACH phone detection works unchanged

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and documentation

- [x] T033 Run full test suite: `cargo test --workspace`
- [x] T034 Run clippy: `cargo clippy --workspace -- -D warnings`
- [x] T035 [P] Add integration test with mixed phone formats in document in crates/veil-detect/src/patterns/phone.rs
- [x] T036 [P] Verify no overlapping matches for same number
- [x] T037 Update CHANGELOG.md with new phone detection capabilities

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - verify baseline
- **Foundational (Phase 2)**: Depends on Setup - prepare structure
- **User Stories (Phase 3-6)**: All depend on Foundational
  - US1, US2, US3 are all P1 - can run in parallel (adding different patterns)
  - US4 must run after all patterns added (regression testing)
- **Polish (Phase 7)**: Depends on all user stories complete

### User Story Dependencies

- **User Story 1 (US)**: Independent - adds US patterns
- **User Story 2 (UK)**: Independent - adds UK patterns
- **User Story 3 (E.164)**: Independent - adds international patterns
- **User Story 4 (DACH)**: Depends on US1, US2, US3 - regression verification

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Pattern additions are independent within each story
- Verification test at end of each story

### Parallel Opportunities

**After Foundational Phase completes:**
- US1, US2, US3 can run in parallel (different pattern groups)
- All test tasks within a story can run in parallel
- US4 must wait for patterns to be added

---

## Parallel Example: US Pattern Tests

```bash
# Launch all US tests together (different test functions):
Task: "T005 [P] [US1] Add test: detect US E.164 format"
Task: "T006 [P] [US1] Add test: detect US parentheses format"
Task: "T007 [P] [US1] Add test: detect US 10-digit format"
Task: "T008 [P] [US1] Add test: detect US toll-free"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Complete Phase 1: Setup (verify baseline)
2. Complete Phase 2: Foundational (organize structure)
3. Complete Phase 3: User Story 1 (US phone numbers)
4. **STOP and VALIDATE**: US phone numbers detected
5. Can deploy as incremental improvement

### Full Feature

1. MVP (above)
2. Add User Story 2: UK phone numbers
3. Add User Story 3: International E.164
4. Add User Story 4: Verify DACH regression
5. Complete Phase 7: Polish
6. Full release

---

## Summary

- **Total Tasks**: 37
- **US1 (US Phones)**: 9 tasks
- **US2 (UK Phones)**: 6 tasks
- **US3 (E.164)**: 7 tasks
- **US4 (DACH)**: 6 tasks
- **Setup/Foundational**: 4 tasks
- **Polish**: 5 tasks

**MVP Scope**: US1 (P1) = 9 tasks for US phone detection

---

## Notes

- [P] tasks = different patterns/tests, no dependencies
- [Story] label maps task to specific user story
- Single file modification (phone.rs) - careful with parallel edits
- All patterns compile at startup - no runtime errors
- Existing tests are regression baseline
