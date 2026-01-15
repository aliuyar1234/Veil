# Tasks: Audit Trail & Reporting

**Input**: Design documents from `D:\Projekte\Veil\specs\011-audit-reporting\`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Tests are included following TDD approach as specified in the constitution.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Crate root: `D:\Projekte\Veil\crates\veil-audit\`
- Source: `D:\Projekte\Veil\crates\veil-audit\src\`
- Tests: `D:\Projekte\Veil\crates\veil-audit\tests\`

---

## Phase 1: Setup (Shared Infrastructure) ✅

**Purpose**: Project initialization and dependency setup

- [x] T001 Add csv dependency (version 1.3) to D:\Projekte\Veil\crates\veil-audit\Cargo.toml
- [x] T002 Add regex dependency (version 1.10) to D:\Projekte\Veil\crates\veil-audit\Cargo.toml
- [x] T003 [P] Create reports module directory at D:\Projekte\Veil\crates\veil-audit\src\reports\
- [x] T004 [P] Create reports module file at D:\Projekte\Veil\crates\veil-audit\src\reports\mod.rs

---

## Phase 2: Foundational (Blocking Prerequisites) ✅

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Extract AuditFilter to D:\Projekte\Veil\crates\veil-audit\src\filter.rs from logger.rs
- [x] T006 Update D:\Projekte\Veil\crates\veil-audit\src\logger.rs to remove AuditFilter (moved to filter.rs)
- [x] T007 [P] Extend AuditError enum in D:\Projekte\Veil\crates\veil-audit\src\error.rs with CsvError, InvalidIdentifier, UnsupportedFramework, RegexError variants
- [x] T008 [P] Create format module at D:\Projekte\Veil\crates\veil-audit\src\reports\format.rs for shared format conversion utilities
- [x] T009 Update D:\Projekte\Veil\crates\veil-audit\src\lib.rs to export new modules (filter, reports)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel ✅

---

## Phase 3: User Story 1 - Log All PII Operations (Priority: P1) 🎯 MVP ✅

**Goal**: Ensure all PII scan and protect operations are logged with required fields (timestamp, operation, file path, findings/redactions)

**Independent Test**: Perform scan and protect operations, verify all actions logged with complete required fields and entries are in chronological order

**Note**: This user story is already implemented in the existing veil-audit crate. This phase validates and extends the existing implementation.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL (or verify existing behavior) before implementation**

- [x] T010 [P] [US1] Verify scan operation logging test in D:\Projekte\Veil\crates\veil-audit\tests\integration_test.rs
- [x] T011 [P] [US1] Verify protect operation logging test in D:\Projekte\Veil\crates\veil-audit\tests\integration_test.rs
- [x] T012 [P] [US1] Verify chronological order test in D:\Projekte\Veil\crates\veil-audit\tests\integration_test.rs

### Implementation for User Story 1

- [x] T013 [US1] Review and validate existing AuditEntry structure in D:\Projekte\Veil\crates\veil-audit\src\entry.rs meets all FR-001 and FR-002 requirements
- [x] T014 [US1] Review and validate existing AuditLogger in D:\Projekte\Veil\crates\veil-audit\src\logger.rs supports append-only logging (FR-003)
- [x] T015 [US1] Review and validate existing hash chain implementation in D:\Projekte\Veil\crates\veil-audit\src\checksum.rs for tamper detection (FR-011)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently (already implemented) ✅

---

## Phase 4: User Story 2 - Generate Data Inventory Report (Priority: P1) ✅

**Goal**: Generate reports showing PII distribution across scanned files with per-file and per-category breakdowns

**Independent Test**: Scan multiple files (100+), generate inventory report, verify all findings accurately summarized in all three formats (JSON, CSV, text)

### Tests for User Story 2

- [x] T016 [P] [US2] Create test file D:\Projekte\Veil\crates\veil-audit\tests\report_inventory_test.rs with test_generate_empty_inventory
- [x] T017 [P] [US2] Add test_generate_single_file_inventory to D:\Projekte\Veil\crates\veil-audit\tests\report_inventory_test.rs
- [x] T018 [P] [US2] Add test_generate_multiple_files_inventory to D:\Projekte\Veil\crates\veil-audit\tests\report_inventory_test.rs
- [x] T019 [P] [US2] Add test_aggregate_findings_correctly to D:\Projekte\Veil\crates\veil-audit\tests\report_inventory_test.rs
- [x] T020 [P] [US2] Add test_inventory_export_json to D:\Projekte\Veil\crates\veil-audit\tests\report_inventory_test.rs
- [x] T021 [P] [US2] Add test_inventory_export_csv to D:\Projekte\Veil\crates\veil-audit\tests\report_inventory_test.rs
- [x] T022 [P] [US2] Add test_inventory_export_text to D:\Projekte\Veil\crates\veil-audit\tests\report_inventory_test.rs

### Implementation for User Story 2

- [x] T023 [P] [US2] Create InventoryReport struct in D:\Projekte\Veil\crates\veil-audit\src\reports\inventory.rs
- [x] T024 [P] [US2] Create FileSummary struct in D:\Projekte\Veil\crates\veil-audit\src\reports\inventory.rs
- [x] T025 [P] [US2] Create CategorySummary struct in D:\Projekte\Veil\crates\veil-audit\src\reports\inventory.rs
- [x] T026 [US2] Implement InventoryReport::to_json() method in D:\Projekte\Veil\crates\veil-audit\src\reports\inventory.rs
- [x] T027 [US2] Implement InventoryReport::to_csv() method in D:\Projekte\Veil\crates\veil-audit\src\reports\inventory.rs using csv crate
- [x] T028 [US2] Implement InventoryReport::to_text() method (Display trait) in D:\Projekte\Veil\crates\veil-audit\src\reports\inventory.rs
- [x] T029 [US2] Implement AuditLogger::generate_inventory() method in D:\Projekte\Veil\crates\veil-audit\src\logger.rs with HashMap aggregation
- [x] T030 [US2] Export InventoryReport types in D:\Projekte\Veil\crates\veil-audit\src\reports\mod.rs
- [x] T031 [US2] Update D:\Projekte\Veil\crates\veil-audit\src\lib.rs to export InventoryReport types

**Checkpoint**: At this point, User Story 2 should be fully functional - inventory reports can be generated in all three formats ✅

---

## Phase 5: User Story 3 - Generate Compliance Report (Priority: P2) ✅

**Goal**: Generate compliance reports mapping findings to GDPR requirements with pass/fail status and gap identification

**Independent Test**: Generate compliance report with --framework gdpr flag, verify GDPR article references, identify unprotected PII as compliance gaps, and show compliant status when all PII is protected

### Tests for User Story 3

- [x] T032 [P] [US3] Create test file D:\Projekte\Veil\crates\veil-audit\tests\report_compliance_test.rs with test_load_gdpr_mappings
- [x] T033 [P] [US3] Add test_generate_report_all_compliant to D:\Projekte\Veil\crates\veil-audit\tests\report_compliance_test.rs
- [x] T034 [P] [US3] Add test_generate_report_with_gaps to D:\Projekte\Veil\crates\veil-audit\tests\report_compliance_test.rs
- [x] T035 [P] [US3] Add test_identify_unprotected_pii to D:\Projekte\Veil\crates\veil-audit\tests\report_compliance_test.rs
- [x] T036 [P] [US3] Add test_calculate_overall_status to D:\Projekte\Veil\crates\veil-audit\tests\report_compliance_test.rs
- [x] T037 [P] [US3] Add test_compliance_export_json to D:\Projekte\Veil\crates\veil-audit\tests\report_compliance_test.rs
- [x] T038 [P] [US3] Add test_compliance_export_text to D:\Projekte\Veil\crates\veil-audit\tests\report_compliance_test.rs

### Implementation for User Story 3

- [x] T039 [P] [US3] Create ComplianceFramework enum in D:\Projekte\Veil\crates\veil-audit\src\reports\compliance.rs
- [x] T040 [P] [US3] Create ComplianceStatus enum in D:\Projekte\Veil\crates\veil-audit\src\reports\compliance.rs
- [x] T041 [P] [US3] Create ComplianceRequirement struct in D:\Projekte\Veil\crates\veil-audit\src\reports\compliance.rs
- [x] T042 [P] [US3] Create ComplianceGap struct in D:\Projekte\Veil\crates\veil-audit\src\reports\compliance.rs
- [x] T043 [P] [US3] Create ComplianceReport struct in D:\Projekte\Veil\crates\veil-audit\src\reports\compliance.rs
- [x] T044 [US3] Define GDPR_MAPPINGS constant with static mapping data in D:\Projekte\Veil\crates\veil-audit\src\reports\compliance.rs
- [x] T045 [US3] Implement ComplianceReport::to_json() method in D:\Projekte\Veil\crates\veil-audit\src\reports\compliance.rs
- [x] T046 [US3] Implement ComplianceReport::to_text() method in D:\Projekte\Veil\crates\veil-audit\src\reports\compliance.rs
- [x] T047 [US3] Implement gap detection logic in D:\Projekte\Veil\crates\veil-audit\src\reports\compliance.rs
- [x] T048 [US3] Implement AuditLogger::generate_compliance_report() method in D:\Projekte\Veil\crates\veil-audit\src\logger.rs
- [x] T049 [US3] Export ComplianceReport types in D:\Projekte\Veil\crates\veil-audit\src\reports\mod.rs
- [x] T050 [US3] Update D:\Projekte\Veil\crates\veil-audit\src\lib.rs to export ComplianceReport types

**Checkpoint**: At this point, User Stories 1, 2, AND 3 should all work independently - compliance reporting functional ✅

---

## Phase 6: User Story 4 - Export Audit Log for External Systems (Priority: P2) ✅

**Goal**: Support exporting audit logs in standard formats (JSON Lines, CSV) for SIEM integration with date range filtering

**Independent Test**: Export logs in JSONL format, verify each line is valid JSON; apply date range filter, verify only entries in range included; validate fields map correctly for SIEM ingestion

**Note**: JSONL export is already implemented (logs are stored as JSONL). This phase focuses on CSV export and validation.

### Tests for User Story 4

- [x] T051 [P] [US4] Add test_export_jsonl_format to D:\Projekte\Veil\crates\veil-audit\tests\integration_test.rs
- [x] T052 [P] [US4] Add test_export_with_date_filter to D:\Projekte\Veil\crates\veil-audit\tests\integration_test.rs
- [x] T053 [P] [US4] Add test_export_csv_format to D:\Projekte\Veil\crates\veil-audit\tests\integration_test.rs
- [x] T054 [P] [US4] Add test_siem_field_mapping to D:\Projekte\Veil\crates\veil-audit\tests\integration_test.rs

### Implementation for User Story 4

- [x] T055 [US4] Implement AuditEntry::to_csv_row() helper method in D:\Projekte\Veil\crates\veil-audit\src\entry.rs
- [x] T056 [US4] Implement AuditLogger::export_csv() method in D:\Projekte\Veil\crates\veil-audit\src\logger.rs
- [x] T057 [US4] Add CSV header generation in D:\Projekte\Veil\crates\veil-audit\src\reports\format.rs
- [x] T058 [US4] Validate existing JSONL export meets FR-007 requirements in D:\Projekte\Veil\crates\veil-audit\src\logger.rs

**Checkpoint**: At this point, audit logs can be exported in both JSONL (native) and CSV formats with filtering ✅

---

## Phase 7: User Story 5 - Support DSAR Response (Priority: P2) ✅

**Goal**: Search audit logs and scan results for data related to specific identifiers (email, name, phone) and generate DSAR response packages with excerpts

**Independent Test**: Search for subject by email, verify all related findings returned; generate export with file excerpts and PII highlighted; log deletion confirmation when processed

### Tests for User Story 5

- [x] T059 [P] [US5] Create test file D:\Projekte\Veil\crates\veil-audit\tests\report_dsar_test.rs with test_search_email_identifier
- [x] T060 [P] [US5] Add test_search_name_identifier to D:\Projekte\Veil\crates\veil-audit\tests\report_dsar_test.rs
- [x] T061 [P] [US5] Add test_search_phone_identifier to D:\Projekte\Veil\crates\veil-audit\tests\report_dsar_test.rs
- [x] T062 [P] [US5] Add test_search_custom_pattern to D:\Projekte\Veil\crates\veil-audit\tests\report_dsar_test.rs
- [x] T063 [P] [US5] Add test_search_with_date_filter to D:\Projekte\Veil\crates\veil-audit\tests\report_dsar_test.rs
- [x] T064 [P] [US5] Add test_extract_context_excerpts to D:\Projekte\Veil\crates\veil-audit\tests\report_dsar_test.rs
- [x] T065 [P] [US5] Add test_search_no_matches to D:\Projekte\Veil\crates\veil-audit\tests\report_dsar_test.rs
- [x] T066 [P] [US5] Add test_dsar_export_json to D:\Projekte\Veil\crates\veil-audit\tests\report_dsar_test.rs
- [x] T067 [P] [US5] Add test_dsar_export_text to D:\Projekte\Veil\crates\veil-audit\tests\report_dsar_test.rs

### Implementation for User Story 5

- [x] T068 [P] [US5] Create IdentifierType enum in D:\Projekte\Veil\crates\veil-audit\src\reports\dsar.rs
- [x] T069 [P] [US5] Create DsarRequest struct in D:\Projekte\Veil\crates\veil-audit\src\reports\dsar.rs
- [x] T070 [P] [US5] Create DsarExcerpt struct in D:\Projekte\Veil\crates\veil-audit\src\reports\dsar.rs
- [x] T071 [P] [US5] Create DsarResponse struct in D:\Projekte\Veil\crates\veil-audit\src\reports\dsar.rs
- [x] T072 [US5] Implement identifier-to-regex conversion logic in D:\Projekte\Veil\crates\veil-audit\src\reports\dsar.rs
- [x] T073 [US5] Implement two-phase search (metadata + content) in D:\Projekte\Veil\crates\veil-audit\src\reports\dsar.rs
- [x] T074 [US5] Implement context excerpt extraction with sanitization in D:\Projekte\Veil\crates\veil-audit\src\reports\dsar.rs
- [x] T075 [US5] Implement DsarResponse::to_json() method in D:\Projekte\Veil\crates\veil-audit\src\reports\dsar.rs
- [x] T076 [US5] Implement DsarResponse::to_text() method in D:\Projekte\Veil\crates\veil-audit\src\reports\dsar.rs
- [x] T077 [US5] Implement AuditLogger::search_dsar() method in D:\Projekte\Veil\crates\veil-audit\src\logger.rs
- [x] T078 [US5] Export DsarRequest and DsarResponse types in D:\Projekte\Veil\crates\veil-audit\src\reports\mod.rs
- [x] T079 [US5] Update D:\Projekte\Veil\crates\veil-audit\src\lib.rs to export DSAR types

**Checkpoint**: All user stories should now be independently functional - DSAR search and response generation complete ✅

---

## Phase 8: Log Rotation & Retention (Supporting Feature) ✅

**Goal**: Implement log rotation with configurable retention policies (FR-012)

**Independent Test**: Test default 7-year retention policy, test custom policy, verify old logs deleted correctly and recent logs preserved

### Tests for Log Rotation

- [x] T080 [P] Create test file D:\Projekte\Veil\crates\veil-audit\tests\retention_test.rs with test_default_retention_policy
- [x] T081 [P] Add test_custom_retention_policy to D:\Projekte\Veil\crates\veil-audit\tests\retention_test.rs
- [x] T082 [P] Add test_check_date_retained to D:\Projekte\Veil\crates\veil-audit\tests\retention_test.rs
- [x] T083 [P] Add test_rotate_delete_old_files to D:\Projekte\Veil\crates\veil-audit\tests\retention_test.rs
- [x] T084 [P] Add test_rotate_keep_recent_files to D:\Projekte\Veil\crates\veil-audit\tests\retention_test.rs
- [x] T085 [P] Add test_rotate_empty_directory to D:\Projekte\Veil\crates\veil-audit\tests\retention_test.rs
- [x] T086 [P] Add test_rotate_no_files_to_delete to D:\Projekte\Veil\crates\veil-audit\tests\retention_test.rs

### Implementation for Log Rotation

- [x] T087 [P] Create RetentionPolicy struct in D:\Projekte\Veil\crates\veil-audit\src\reports\retention.rs
- [x] T088 [P] Implement Default trait for RetentionPolicy (7 years) in D:\Projekte\Veil\crates\veil-audit\src\reports\retention.rs
- [x] T089 Implement RetentionPolicy::is_retained() method in D:\Projekte\Veil\crates\veil-audit\src\reports\retention.rs
- [x] T090 Implement AuditLogger::rotate_logs() method in D:\Projekte\Veil\crates\veil-audit\src\logger.rs
- [x] T091 Export RetentionPolicy in D:\Projekte\Veil\crates\veil-audit\src\reports\mod.rs
- [x] T092 Update D:\Projekte\Veil\crates\veil-audit\src\lib.rs to export RetentionPolicy

**Checkpoint**: Log rotation and retention fully functional ✅

---

## Phase 9: Integration Tests & Performance Validation ✅

**Purpose**: End-to-end workflows and performance validation

- [x] T093 [P] Add test_full_workflow_log_to_reports to D:\Projekte\Veil\crates\veil-audit\tests\integration_test.rs
- [x] T094 [P] Add test_multi_day_logs_with_rotation to D:\Projekte\Veil\crates\veil-audit\tests\integration_test.rs
- [x] T095 [P] Add test_large_dataset_10k_entries to D:\Projekte\Veil\crates\veil-audit\tests\integration_test.rs (performance)
- [x] T096 [P] Add test_hash_chain_integrity_after_rotation to D:\Projekte\Veil\crates\veil-audit\tests\integration_test.rs
- [x] T097 [P] Add test_export_all_report_formats to D:\Projekte\Veil\crates\veil-audit\tests\integration_test.rs
- [x] T098 Validate inventory report generation from 1M entries completes in <10 seconds (performance target SC-004)
- [x] T099 Validate DSAR search through 1M entries completes in <10 seconds (performance target SC-004)

**Checkpoint**: All integration tests pass, performance targets met, no regressions ✅

---

## Phase 10: Polish & Cross-Cutting Concerns ✅

**Purpose**: Documentation, cleanup, and final validation

- [x] T100 [P] Add module-level documentation to D:\Projekte\Veil\crates\veil-audit\src\lib.rs
- [x] T101 [P] Add rustdoc examples for InventoryReport in D:\Projekte\Veil\crates\veil-audit\src\reports\inventory.rs
- [x] T102 [P] Add rustdoc examples for ComplianceReport in D:\Projekte\Veil\crates\veil-audit\src\reports\compliance.rs
- [x] T103 [P] Add rustdoc examples for DsarRequest/Response in D:\Projekte\Veil\crates\veil-audit\src\reports\dsar.rs
- [x] T104 [P] Add rustdoc examples for RetentionPolicy in D:\Projekte\Veil\crates\veil-audit\src\reports\retention.rs
- [x] T105 Run cargo doc and verify no warnings
- [x] T106 Run cargo clippy -- -D warnings and fix any issues
- [x] T107 Run cargo fmt and ensure code formatting is consistent
- [x] T108 Verify all public items have documentation comments
- [x] T109 [P] Create or update README.md at D:\Projekte\Veil\crates\veil-audit\README.md with usage examples
- [x] T110 Run quickstart.md validation examples from D:\Projekte\Veil\specs\011-audit-reporting\quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phases 3-7)**: All depend on Foundational phase completion
  - Phase 3 (US1): Already implemented, validation only
  - Phase 4 (US2): Can start after Foundational
  - Phase 5 (US3): Can start after Foundational
  - Phase 6 (US4): Can start after Foundational
  - Phase 7 (US5): Can start after Foundational
- **Log Rotation (Phase 8)**: Can start after Foundational
- **Integration Tests (Phase 9)**: Depends on all desired user stories being complete
- **Polish (Phase 10)**: Depends on all implementation phases being complete

### User Story Dependencies

- **User Story 1 (P1)**: Already implemented - validation only
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 4 (P2)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 5 (P2)**: Can start after Foundational (Phase 2) - No dependencies on other stories

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Structs and types before methods
- Helper functions before main logic
- Core implementation before export formats
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, User Stories 2-5 can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Structs within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 2 (Inventory Reports)

```bash
# Launch all tests for User Story 2 together:
Task T016: "Create test file with test_generate_empty_inventory"
Task T017: "Add test_generate_single_file_inventory"
Task T018: "Add test_generate_multiple_files_inventory"
Task T019: "Add test_aggregate_findings_correctly"
Task T020: "Add test_inventory_export_json"
Task T021: "Add test_inventory_export_csv"
Task T022: "Add test_inventory_export_text"

# Launch all struct definitions for User Story 2 together:
Task T023: "Create InventoryReport struct"
Task T024: "Create FileSummary struct"
Task T025: "Create CategorySummary struct"
```

---

## Parallel Example: User Story 5 (DSAR Support)

```bash
# Launch all tests for User Story 5 together:
Task T059: "test_search_email_identifier"
Task T060: "test_search_name_identifier"
Task T061: "test_search_phone_identifier"
Task T062: "test_search_custom_pattern"
Task T063: "test_search_with_date_filter"
Task T064: "test_extract_context_excerpts"
Task T065: "test_search_no_matches"
Task T066: "test_dsar_export_json"
Task T067: "test_dsar_export_text"

# Launch all struct definitions for User Story 5 together:
Task T068: "Create IdentifierType enum"
Task T069: "Create DsarRequest struct"
Task T070: "Create DsarExcerpt struct"
Task T071: "Create DsarResponse struct"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (validation only - already implemented)
4. Complete Phase 4: User Story 2 (inventory reports)
5. **STOP and VALIDATE**: Test User Stories 1 & 2 independently
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 (validation) → Test independently
3. Add User Story 2 (inventory) → Test independently → Deploy/Demo (MVP!)
4. Add User Story 3 (compliance) → Test independently → Deploy/Demo
5. Add User Story 4 (export) → Test independently → Deploy/Demo
6. Add User Story 5 (DSAR) → Test independently → Deploy/Demo
7. Add Phase 8 (rotation) → Test independently → Deploy/Demo
8. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 2 (Inventory Reports)
   - Developer B: User Story 3 (Compliance Reports)
   - Developer C: User Story 5 (DSAR Support)
   - Developer D: User Story 4 (Export) + Phase 8 (Rotation)
3. Stories complete and integrate independently

---

## Success Metrics

### Functional Completeness (from spec.md)

- ✅ **FR-001**: All scan operations logged (US1 - already implemented)
- ✅ **FR-002**: All protect operations logged (US1 - already implemented)
- ✅ **FR-003**: Append-only audit log (US1 - already implemented)
- 🎯 **FR-004**: Data inventory reports (US2)
- 🎯 **FR-005**: GDPR compliance reports (US3)
- 🎯 **FR-006**: Multiple report formats (US2, US3, US5)
- ✅ **FR-007**: Audit log export in JSONL (US4 - already exists)
- ✅ **FR-008**: Filter by date, operation, path (already implemented)
- 🎯 **FR-009**: DSAR search by identifier (US5)
- 🎯 **FR-010**: DSAR response packages (US5)
- ✅ **FR-011**: Tamper detection via checksums (US1 - already implemented)
- 🎯 **FR-012**: Log rotation with retention (Phase 8)

### Performance Targets (from spec.md)

- ✅ **SC-001**: 100% of operations logged (US1 - already achieved)
- ✅ **SC-002**: Append-only logs (US1 - already achieved)
- 🎯 **SC-003**: Inventory reports accurate (US2)
- 🎯 **SC-004**: DSAR search <10s for 1M entries (US5, T099)
- 🎯 **SC-004**: Inventory generation <10s for 1M entries (US2, T098)
- ✅ **SC-005**: Valid JSONL export (US4 - already achieved)
- ✅ **SC-006**: Tamper detection 100% accurate (US1 - already achieved)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD approach per constitution)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

---

## Task Summary

**Total Tasks**: 110
**Completed**: 110 ✅

**Tasks per Phase**:
- Phase 1 (Setup): 4/4 ✅
- Phase 2 (Foundational): 5/5 ✅
- Phase 3 (US1 - Validation): 6/6 ✅
- Phase 4 (US2 - Inventory): 16/16 ✅
- Phase 5 (US3 - Compliance): 19/19 ✅
- Phase 6 (US4 - Export): 8/8 ✅
- Phase 7 (US5 - DSAR): 21/21 ✅
- Phase 8 (Rotation): 13/13 ✅
- Phase 9 (Integration): 7/7 ✅
- Phase 10 (Polish): 11/11 ✅

**Parallel Opportunities**: 67 tasks marked [P] can run in parallel with other tasks

**Independent Test Criteria**:
- US1: All PII operations logged with complete required fields ✅ (already implemented)
- US2: Generate inventory from 100 files with accurate summaries in all formats
- US3: Generate GDPR report with article references and gap identification
- US4: Export logs in JSONL/CSV with date filtering
- US5: Search by identifier and generate DSAR response with excerpts

**Suggested MVP Scope**: Phase 1-4 (Setup + Foundational + US1 validation + US2 inventory reports)

**Format Validation**: ✅ All tasks follow the required checklist format (checkbox, ID, labels, file paths)
