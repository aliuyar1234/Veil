# Tasks: Policy Engine

**Input**: Design documents from `/specs/009-policy-engine/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Included per constitution requirement (TDD)

**Organization**: Tasks grouped by user story. Extends existing `veil-policy` crate.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story (US1-US5 maps to spec.md)
- Exact file paths included

## Path Conventions

- **Crate location**: `crates/veil-policy/`
- **Tests**: Inline in modules + `crates/veil-policy/tests/`

---

## Phase 1: Setup (Extend Existing Crate) ✅

**Purpose**: Add veil-crypto dependency and prepare new modules

- [x] T001 Add veil-crypto dependency to Cargo.toml in crates/veil-policy/Cargo.toml
- [x] T002 [P] Create keyref.rs module file in crates/veil-policy/src/keyref.rs
- [x] T003 [P] Create executor.rs module file in crates/veil-policy/src/executor.rs
- [x] T004 [P] Create protect.rs module file in crates/veil-policy/src/protect.rs
- [x] T005 Add new modules to lib.rs in crates/veil-policy/src/lib.rs
- [x] T006 Verify crate compiles with cargo build -p veil-policy

---

## Phase 2: Foundational (Shared Types) ✅

**Purpose**: Core types needed by multiple user stories

**⚠️ CRITICAL**: Must complete before user story implementation

- [x] T007 [P] Test KeyRefScheme enum in crates/veil-policy/src/keyref.rs
- [x] T008 [P] Test KeyRef parsing from string in crates/veil-policy/src/keyref.rs
- [x] T009 Implement KeyRefScheme enum (Env, File) in crates/veil-policy/src/keyref.rs
- [x] T010 Implement KeyRef struct with parse logic in crates/veil-policy/src/keyref.rs
- [x] T011 Implement serde TryFrom/Into for KeyRef in crates/veil-policy/src/keyref.rs
- [x] T012 Add KeyRefError variants to error.rs in crates/veil-policy/src/error.rs
- [x] T013 Add key_ref field to ProtectionRule in crates/veil-policy/src/rules.rs
- [x] T014 Verify foundational types compile with cargo build -p veil-policy

**Checkpoint**: KeyRef type ready for use in protection rules ✅

---

## Phase 3: User Story 1 - Apply Detection Rules (Priority: P1) 🎯 MVP ✅

**Goal**: Filter detection results according to policy rules

**Independent Test**: Create policy with specific detectors and thresholds, verify only matching findings reported

**Note**: Basic filtering already exists in apply.rs. Extend with more tests.

### Tests for User Story 1

- [x] T015 [P] [US1] Test filter by PII type in crates/veil-policy/src/apply.rs
- [x] T016 [P] [US1] Test filter by confidence threshold in crates/veil-policy/src/apply.rs
- [x] T017 [P] [US1] Test disabled rules are skipped in crates/veil-policy/src/apply.rs
- [x] T018 [P] [US1] Test default policy allows all when no rules in crates/veil-policy/src/apply.rs

### Implementation for User Story 1

- [x] T019 [US1] Verify existing apply_policy_to_findings handles type filtering in crates/veil-policy/src/apply.rs
- [x] T020 [US1] Add test for empty detection rules (allow all) in crates/veil-policy/src/apply.rs
- [x] T021 [US1] Export apply_policy_to_findings in lib.rs in crates/veil-policy/src/lib.rs
- [x] T022 [US1] Verify all US1 tests pass with cargo test -p veil-policy

**Checkpoint**: Detection filtering works per policy rules ✅

---

## Phase 4: User Story 2 - Configure Protection Actions (Priority: P1) ✅

**Goal**: Apply correct protection method (redact/mask/encrypt/etc.) per PII category

**Independent Test**: Create policy with different actions per type, verify correct action applied

### Tests for User Story 2

- [x] T023 [P] [US2] Test protect with redact action in crates/veil-policy/src/protect.rs
- [x] T024 [P] [US2] Test protect with mask action in crates/veil-policy/src/protect.rs
- [x] T025 [P] [US2] Test protect with hash action in crates/veil-policy/src/protect.rs
- [x] T026 [P] [US2] Test protect with encrypt action in crates/veil-policy/src/protect.rs
- [x] T027 [P] [US2] Test protect with pseudonymize action in crates/veil-policy/src/protect.rs
- [x] T028 [P] [US2] Test protect with tokenize action in crates/veil-policy/src/protect.rs

### Implementation for User Story 2

- [x] T029 [US2] Create ProtectedValue struct in crates/veil-policy/src/protect.rs
- [x] T030 [US2] Create protect_value function signature in crates/veil-policy/src/protect.rs
- [x] T031 [US2] Implement redact action dispatch to veil-redact in crates/veil-policy/src/protect.rs
- [x] T032 [US2] Implement mask action dispatch to veil-redact in crates/veil-policy/src/protect.rs
- [x] T033 [US2] Implement hash action dispatch to veil-crypto in crates/veil-policy/src/protect.rs
- [x] T034 [US2] Implement encrypt action dispatch to veil-crypto in crates/veil-policy/src/protect.rs
- [x] T035 [US2] Implement pseudonymize action dispatch to veil-crypto in crates/veil-policy/src/protect.rs
- [x] T036 [US2] Implement tokenize action dispatch to veil-crypto in crates/veil-policy/src/protect.rs
- [x] T037 [US2] Export protect module from lib.rs in crates/veil-policy/src/lib.rs
- [x] T038 [US2] Verify all US2 tests pass with cargo test -p veil-policy

**Checkpoint**: All protection actions work via policy ✅

---

## Phase 5: User Story 3 - Locale-Specific Policies (Priority: P2) ✅

**Goal**: Locale setting activates region-specific detectors

**Independent Test**: Create policy with locale, verify locale-specific detection

**Note**: Basic Locale enum already exists. Extend validation.

### Tests for User Story 3

- [x] T039 [P] [US3] Test locale parsing from policy in crates/veil-policy/src/locale.rs
- [x] T040 [P] [US3] Test locale affects detector selection in crates/veil-policy/src/locale.rs
- [x] T041 [P] [US3] Test default locale when none specified in crates/veil-policy/src/locale.rs

### Implementation for User Story 3

- [x] T042 [US3] Add get_locale_detectors() function in crates/veil-policy/src/locale.rs
- [x] T043 [US3] Implement locale-to-detector mapping in crates/veil-policy/src/locale.rs
- [x] T044 [US3] Add locale validation to policy validation in crates/veil-policy/src/validation.rs
- [x] T045 [US3] Verify all US3 tests pass with cargo test -p veil-policy

**Checkpoint**: Locale-specific detection configured via policy ✅

---

## Phase 6: User Story 4 - Consistent Pseudonymization (Priority: P2) ✅

**Goal**: Same name always maps to same pseudonym within scope

**Independent Test**: Pseudonymize document with repeated names, verify same pseudonym used

### Tests for User Story 4

- [x] T046 [P] [US4] Test consistent pseudonymization same value in crates/veil-policy/src/executor.rs
- [x] T047 [P] [US4] Test non-consistent gives different values in crates/veil-policy/src/executor.rs
- [x] T048 [P] [US4] Test cache clear resets consistency in crates/veil-policy/src/executor.rs

### Implementation for User Story 4

- [x] T049 [US4] Create PolicyExecutor struct with cache in crates/veil-policy/src/executor.rs
- [x] T050 [US4] Implement PolicyExecutor::new() in crates/veil-policy/src/executor.rs
- [x] T051 [US4] Implement pseudonym caching logic in crates/veil-policy/src/executor.rs
- [x] T052 [US4] Implement clear_cache() method in crates/veil-policy/src/executor.rs
- [x] T053 [US4] Integrate cache with protect_value for pseudonymize in crates/veil-policy/src/executor.rs
- [x] T054 [US4] Verify all US4 tests pass with cargo test -p veil-policy

**Checkpoint**: Consistent pseudonymization works across document ✅

---

## Phase 7: User Story 5 - Reference External Keys (Priority: P2) ✅

**Goal**: Keys resolved from environment or files, never embedded in policy

**Independent Test**: Create policy with key reference, verify encryption uses resolved key

### Tests for User Story 5

- [x] T055 [P] [US5] Test env:// key resolution in crates/veil-policy/src/keyref.rs
- [x] T056 [P] [US5] Test file:// key resolution in crates/veil-policy/src/keyref.rs
- [x] T057 [P] [US5] Test missing env var returns error in crates/veil-policy/src/keyref.rs
- [x] T058 [P] [US5] Test missing file returns error in crates/veil-policy/src/keyref.rs
- [x] T059 [P] [US5] Test invalid key ref format error in crates/veil-policy/src/keyref.rs

### Implementation for User Story 5

- [x] T060 [US5] Implement KeyRef::resolve() method in crates/veil-policy/src/keyref.rs
- [x] T061 [US5] Implement env:// resolution via std::env in crates/veil-policy/src/keyref.rs
- [x] T062 [US5] Implement file:// resolution via std::fs in crates/veil-policy/src/keyref.rs
- [x] T063 [US5] Add key resolution to PolicyExecutor in crates/veil-policy/src/executor.rs
- [x] T064 [US5] Integrate resolved key with encrypt action in crates/veil-policy/src/protect.rs
- [x] T065 [US5] Export KeyRef and KeyRefError from lib.rs in crates/veil-policy/src/lib.rs
- [x] T066 [US5] Verify all US5 tests pass with cargo test -p veil-policy

**Checkpoint**: External key references work for encryption ✅

---

## Phase 8: PolicyExecutor Integration ✅

**Purpose**: High-level executor combining all features

### Tests

- [x] T067 [P] Test PolicyExecutor::from_policy() in crates/veil-policy/src/executor.rs
- [x] T068 [P] Test PolicyExecutor::process() full pipeline in crates/veil-policy/src/executor.rs
- [x] T069 [P] Test ProcessResult contains all actions in crates/veil-policy/src/executor.rs

### Implementation

- [x] T070 Create ProcessResult struct in crates/veil-policy/src/executor.rs
- [x] T071 Create AppliedAction struct in crates/veil-policy/src/executor.rs
- [x] T072 Create ProcessStats struct in crates/veil-policy/src/executor.rs
- [x] T073 Implement PolicyExecutor::from_policy() in crates/veil-policy/src/executor.rs
- [x] T074 Implement PolicyExecutor::process() orchestrating full pipeline in crates/veil-policy/src/executor.rs
- [x] T075 Implement protect_finding() method in crates/veil-policy/src/executor.rs
- [x] T076 Export PolicyExecutor and result types from lib.rs in crates/veil-policy/src/lib.rs

---

## Phase 9: Polish & Cross-Cutting Concerns ✅

**Purpose**: Documentation, validation, final checks

- [x] T077 [P] Add documentation comments to keyref.rs in crates/veil-policy/src/keyref.rs
- [x] T078 [P] Add documentation comments to executor.rs in crates/veil-policy/src/executor.rs
- [x] T079 [P] Add documentation comments to protect.rs in crates/veil-policy/src/protect.rs
- [x] T080 Extend policy validation for key_ref in crates/veil-policy/src/validation.rs
- [x] T081 Run cargo clippy -p veil-policy -- -D warnings
- [x] T082 Run cargo fmt --check -p veil-policy
- [x] T083 Run full workspace tests with cargo test
- [x] T084 Validate quickstart.md examples work

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 - detection filtering
- **US2 (Phase 4)**: Depends on Phase 2 - protection actions
- **US3 (Phase 5)**: Depends on Phase 2 - locale handling
- **US4 (Phase 6)**: Depends on Phase 4 (US2) - consistency requires protection
- **US5 (Phase 7)**: Depends on Phase 2 - key resolution
- **Integration (Phase 8)**: Depends on all user stories
- **Polish (Phase 9)**: Depends on Phase 8

### User Story Dependencies

- **US1 (Detection Rules)**: Independent - extends existing functionality
- **US2 (Protection Actions)**: Independent - core protection dispatch
- **US3 (Locale)**: Independent - extends existing locale
- **US4 (Consistent Pseudonymization)**: Depends on US2 (needs protect dispatch)
- **US5 (Key References)**: Independent - key resolution

### Parallel Opportunities

- T002, T003, T004 can run in parallel (different files)
- T007, T008 can run in parallel (tests)
- T015-T018 can run in parallel (US1 tests)
- T023-T028 can run in parallel (US2 tests)
- T055-T059 can run in parallel (US5 tests)
- T077-T079 can run in parallel (documentation)

---

## Parallel Example: User Story 2

```bash
# Launch all tests for US2 together:
Task: "Test protect with redact action"
Task: "Test protect with mask action"
Task: "Test protect with hash action"
Task: "Test protect with encrypt action"
Task: "Test protect with pseudonymize action"
Task: "Test protect with tokenize action"

# Then implement sequentially:
Task: "Create ProtectedValue struct"
Task: "Create protect_value function"
Task: "Implement redact action dispatch"
...
```

---

## Implementation Strategy

### MVP First (US1 + US2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: US1 (Detection filtering)
4. Complete Phase 4: US2 (Protection actions)
5. **STOP and VALIDATE**: Full detection + protection pipeline works

### Incremental Delivery

1. Setup + Foundational → Types ready
2. Add US1 → Detection filtering works
3. Add US2 → All protection actions work → **MVP Complete**
4. Add US3 → Locale-specific detection
5. Add US4 → Consistent pseudonymization
6. Add US5 → External key references
7. Integration → Full PolicyExecutor
8. Polish → Production ready

---

## Summary

- **Total Tasks**: 84
- **Completed**: 84 ✅
- **Phase 1 (Setup)**: 6/6 ✅
- **Phase 2 (Foundational)**: 8/8 ✅
- **Phase 3 (US1)**: 8/8 ✅
- **Phase 4 (US2)**: 16/16 ✅
- **Phase 5 (US3)**: 7/7 ✅
- **Phase 6 (US4)**: 9/9 ✅
- **Phase 7 (US5)**: 12/12 ✅
- **Phase 8 (Integration)**: 10/10 ✅
- **Phase 9 (Polish)**: 8/8 ✅
- **Parallel Opportunities**: 32 tasks marked [P]
- **MVP Scope**: Phases 1-4 (38 tasks) ✅

---

## Notes

- Extends existing veil-policy crate (not new crate)
- Uses veil-crypto for encrypt/hash/pseudonymize/tokenize
- Uses veil-redact for redact/mask
- Constitution: Result<T,E>, no unwrap, documented API
