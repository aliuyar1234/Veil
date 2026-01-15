# Implementation Plan: Audit Trail & Reporting

**Branch**: `011-audit-reporting` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/011-audit-reporting/spec.md`

## Summary

Extend the existing `veil-audit` crate to add comprehensive audit reporting capabilities. The crate already has solid audit logging infrastructure with JSONL storage, hash chains, and basic querying. This feature adds data inventory reports, GDPR compliance reporting, DSAR support, and log rotation with retention policies.

## Technical Context

**Language/Version**: Rust (stable, 1.75+)
**Primary Dependencies**:
  - Existing: serde, serde_json, chrono, sha2, uuid, thiserror, veil-detect, veil-redact
  - NEW: csv (0.1), regex (1.10)
**Storage**: Local filesystem (JSONL audit logs)
**Testing**: cargo test with integration tests
**Target Platform**: Cross-platform library (Linux, macOS, Windows)
**Project Type**: Extend existing crate
**Performance Goals**:
  - Generate inventory report from 1M entries in <10 seconds
  - DSAR search through 1M entries in <10 seconds
  - Memory: <500MB for 1M entry processing
**Constraints**: No async required, pure library, append-only logs
**Scale/Scope**: Enterprise-grade audit reporting for compliance and privacy operations

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ✅ PASS | No key storage; DSAR excerpts sanitized to prevent PII leakage |
| II. Stability & Error Handling | ✅ PASS | Result<T, AuditError> for all new operations |
| III. Performance | ✅ PASS | HashMap aggregation for reports; indexed search for DSAR |
| IV. Simplicity & Minimalism | ✅ PASS | Extends existing crate; minimal new dependencies (csv, regex) |
| V. Test-First Development | ✅ PASS | TDD for new report modules |
| VI. Dependency Discipline | ✅ PASS | csv and regex are standard, well-maintained crates |
| VII. Rust Standards | ✅ PASS | Clippy clean; documented public API |

## Project Structure

### Documentation (this feature)

```text
specs/011-audit-reporting/
├── spec.md              # Feature specification (EXISTS)
├── plan.md              # This file
├── research.md          # Phase 0 output (COMPLETED)
├── data-model.md        # Phase 1 output (COMPLETED)
├── quickstart.md        # Phase 1 output (COMPLETED)
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/veil-audit/src/
├── lib.rs                # Public exports (EXTEND)
├── entry.rs              # AuditEntry, AuditParameters, AuditOutcome (EXISTS)
├── logger.rs             # AuditLogger (EXISTS - EXTEND with report methods)
├── operation.rs          # AuditOperation (EXISTS)
├── summary.rs            # FindingsSummary, RedactionsSummary (EXISTS)
├── checksum.rs           # Checksum and chain verification (EXISTS)
├── error.rs              # AuditError (EXISTS - EXTEND)
├── filter.rs             # NEW: AuditFilter moved from logger.rs
├── reports/              # NEW: Report generation module
│   ├── mod.rs            # Report type exports
│   ├── inventory.rs      # InventoryReport, FileSummary, CategorySummary
│   ├── compliance.rs     # ComplianceReport, GDPR mappings
│   ├── dsar.rs           # DsarRequest, DsarResponse, search logic
│   ├── format.rs         # Format conversions (JSON, CSV, text)
│   └── retention.rs      # RetentionPolicy, log rotation
└── tests/
    ├── integration_test.rs      # EXISTS - EXTEND
    ├── report_inventory_test.rs # NEW
    ├── report_compliance_test.rs# NEW
    ├── report_dsar_test.rs      # NEW
    └── retention_test.rs        # NEW
```

**Structure Decision**:
- Create new `reports/` module to organize reporting functionality
- Move `AuditFilter` to its own module for clarity
- Keep core audit types (entry, logger) in existing modules
- All report types implement format conversion methods

## Complexity Tracking

> No violations identified.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | - | - |

## Phase 0: Research ✅ COMPLETED

**Input**: Feature spec + existing codebase
**Output**: [research.md](./research.md)
**Status**: ✅ COMPLETED

Key findings:
- Existing audit infrastructure is solid and complete
- Daily JSONL rotation already works
- Hash chain provides tamper detection
- Need to add: reports, DSAR, compliance, retention

## Phase 1: Design ✅ COMPLETED

**Input**: Research findings
**Output**: [data-model.md](./data-model.md) + [quickstart.md](./quickstart.md)
**Status**: ✅ COMPLETED

Key design decisions:
- In-memory aggregation for reports (simpler, fast enough)
- Static GDPR mappings in code (stable framework)
- Two-phase DSAR search (metadata + content)
- Format conversion via dedicated methods (not trait objects)

## Phase 2: Implementation Tasks

**Input**: Design documents
**Output**: tasks.md (via /speckit.tasks)
**Workflow**: Test-first (Red-Green-Refactor)

### Task Breakdown

#### 2.1: Foundation (filter.rs)

**Goal**: Extract AuditFilter to its own module

**Files**:
- `src/filter.rs` (NEW)
- `src/logger.rs` (MODIFY - remove AuditFilter)
- `src/lib.rs` (MODIFY - update exports)

**Tests**:
- Filter construction
- Filter validation

**Exit Criteria**: AuditFilter in separate module, all existing tests pass

---

#### 2.2: Inventory Reports (reports/inventory.rs)

**Goal**: Implement data inventory report generation

**Files**:
- `src/reports/mod.rs` (NEW)
- `src/reports/inventory.rs` (NEW)
- `src/reports/format.rs` (NEW - shared formatters)
- `src/logger.rs` (MODIFY - add generate_inventory method)
- `src/lib.rs` (MODIFY - export report types)
- `tests/report_inventory_test.rs` (NEW)

**Tests**:
1. Generate inventory from empty logs
2. Generate inventory with single file, single category
3. Generate inventory with multiple files, multiple categories
4. Aggregate findings correctly across files
5. Export to JSON (parse back to validate)
6. Export to CSV (validate headers and data)
7. Export to text (validate format)

**Exit Criteria**:
- InventoryReport struct complete
- AuditLogger::generate_inventory() works
- All 3 export formats (JSON, CSV, text) implemented
- Tests pass

---

#### 2.3: Compliance Reports (reports/compliance.rs)

**Goal**: Implement GDPR compliance reporting

**Files**:
- `src/reports/compliance.rs` (NEW)
- `src/logger.rs` (MODIFY - add generate_compliance_report method)
- `src/lib.rs` (MODIFY - export compliance types)
- `tests/report_compliance_test.rs` (NEW)

**Tests**:
1. Load GDPR mappings
2. Generate report with all compliant data
3. Generate report with compliance gaps
4. Identify unprotected PII correctly
5. Calculate overall status correctly
6. Export to JSON
7. Export to text with gap details

**Exit Criteria**:
- ComplianceReport struct complete
- GDPR mappings defined
- AuditLogger::generate_compliance_report() works
- Gap detection accurate
- Tests pass

---

#### 2.4: DSAR Support (reports/dsar.rs)

**Goal**: Implement data subject access request search

**Files**:
- `src/reports/dsar.rs` (NEW)
- `src/logger.rs` (MODIFY - add search_dsar method)
- `src/lib.rs` (MODIFY - export DSAR types)
- `src/error.rs` (MODIFY - add regex error variant)
- `tests/report_dsar_test.rs` (NEW)

**Tests**:
1. Search for email identifier
2. Search for name identifier
3. Search for phone identifier
4. Search with custom regex pattern
5. Search with date range filter
6. Extract context excerpts correctly
7. Handle no matches gracefully
8. Export to JSON
9. Export to text

**Exit Criteria**:
- DsarRequest and DsarResponse structs complete
- AuditLogger::search_dsar() works
- Regex search accurate
- Context extraction correct
- Tests pass

---

#### 2.5: Log Rotation (reports/retention.rs)

**Goal**: Implement log rotation with retention policies

**Files**:
- `src/reports/retention.rs` (NEW)
- `src/logger.rs` (MODIFY - add rotate_logs method)
- `src/lib.rs` (MODIFY - export RetentionPolicy)
- `tests/retention_test.rs` (NEW)

**Tests**:
1. Default retention policy (7 years)
2. Custom retention policy
3. Check if date is retained
4. Rotate logs - delete old files
5. Rotate logs - keep recent files
6. Handle empty log directory
7. Handle no files to delete

**Exit Criteria**:
- RetentionPolicy struct complete
- AuditLogger::rotate_logs() works
- Old logs deleted correctly
- Recent logs preserved
- Tests pass

---

#### 2.6: Integration Tests

**Goal**: End-to-end workflows

**Files**:
- `tests/integration_test.rs` (EXTEND)

**Tests**:
1. Full workflow: log → inventory → compliance → DSAR
2. Multi-day logs with rotation
3. Large dataset (10k entries) performance
4. Hash chain integrity after rotation
5. Export all report formats

**Exit Criteria**:
- All integration tests pass
- Performance targets met
- No regressions in existing functionality

---

#### 2.7: Documentation

**Goal**: Update crate documentation

**Files**:
- `src/lib.rs` (MODIFY - update module docs)
- `README.md` (NEW or MODIFY - add examples)

**Exit Criteria**:
- All public items documented
- Examples in rustdoc
- cargo doc builds without warnings

---

## Phase 3: Integration (CLI)

**Goal**: Integrate with veil-cli

**Files** (in veil-cli crate):
- `src/commands/audit.rs` (NEW)
- `src/main.rs` (MODIFY - add audit subcommand)

**Commands**:
```bash
veil audit inventory --format [json|csv|text] --output <file>
veil audit compliance --framework gdpr --output <file>
veil audit dsar --email <email> --output <file>
veil audit rotate --retention-days <days>
veil audit verify [--date <date>]
```

**Exit Criteria**:
- CLI commands functional
- Help text complete
- Error messages user-friendly

---

## Testing Strategy

### Unit Tests

**Coverage Target**: >90% for new code

**Test Categories**:
- Report generation logic
- Format conversion (JSON, CSV, text)
- DSAR search and matching
- Retention policy calculation
- Error handling

### Integration Tests

**Scenarios**:
- Multi-operation workflows
- Large datasets (performance validation)
- Edge cases (empty logs, malformed data)
- Hash chain integrity

### Performance Tests

**Benchmarks**:
- 1M entry inventory generation: <10s
- 1M entry DSAR search: <10s
- Memory usage: <500MB for 1M entries

**Tool**: criterion (if needed for formal benchmarks)

---

## Dependencies

### New Dependencies to Add

Add to `crates/veil-audit/Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...
csv = "1.3"
regex = "1.10"
```

**Justification**:
- `csv`: Standard library for CSV writing; well-maintained
- `regex`: Standard library for pattern matching; well-maintained

---

## Success Metrics

### Functional Completeness

- ✅ FR-001: All scan operations logged ✅ (Already exists)
- ✅ FR-002: All protect operations logged ✅ (Already exists)
- ✅ FR-003: Append-only audit log ✅ (Already exists)
- 🔲 FR-004: Data inventory reports
- 🔲 FR-005: GDPR compliance reports
- 🔲 FR-006: Multiple report formats (JSON, CSV, text)
- 🔲 FR-007: Audit log export in JSONL ✅ (Already exists)
- 🔲 FR-008: Filter by date, operation, path ✅ (Already exists)
- 🔲 FR-009: DSAR search by identifier
- 🔲 FR-010: DSAR response packages
- 🔲 FR-011: Tamper detection via checksums ✅ (Already exists)
- 🔲 FR-012: Log rotation with retention

### Performance Targets

- 🔲 SC-001: 100% of operations logged ✅ (Already achieved)
- 🔲 SC-002: Append-only logs ✅ (Already achieved)
- 🔲 SC-003: Inventory reports accurate
- 🔲 SC-004: DSAR search <10s for 1M entries
- 🔲 SC-005: Valid JSONL export ✅ (Already achieved)
- 🔲 SC-006: Tamper detection 100% accurate ✅ (Already achieved)

---

## Risk Assessment

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| DSAR search performance on large logs | Medium | Medium | Use regex compilation caching; consider indexing if needed |
| CSV export escaping issues | Low | Low | Use csv crate; test with special characters |
| Retention policy edge cases | Low | Medium | Comprehensive date/time tests |
| Report aggregation memory usage | Low | Medium | Stream if needed (defer optimization) |

### Dependency Risks

| Dependency | Risk Level | Justification |
|------------|------------|---------------|
| csv | Low | Standard crate, stable API |
| regex | Low | Standard crate, widely used |

---

## Deployment Considerations

### Breaking Changes

**None** - This is a purely additive change. All existing APIs remain unchanged.

### Migration Path

No migration needed. Existing audit logs remain compatible.

### Rollback Plan

If issues arise, simply don't use the new report methods. Core logging functionality is unchanged.

---

## Future Enhancements (Out of Scope)

These are explicitly deferred for future iterations:

1. **Real-time log streaming** (Requires async)
2. **External storage backends** (S3, database)
3. **Automatic background rotation** (Requires daemon/scheduler)
4. **Policy inheritance for retention** (Complex, single policy sufficient)
5. **Advanced indexing for DSAR** (Optimize if needed based on real-world usage)
6. **Additional compliance frameworks** (CCPA, HIPAA - add when requested)
7. **Machine-readable compliance exports** (SCAP, OSCAL - enterprise feature)

---

## Acceptance Criteria

### Must Have (P1)

- ✅ All user stories in spec have tests
- ✅ Data inventory reports (FR-004)
- ✅ Report export formats (FR-006)
- ✅ All tests pass
- ✅ Clippy clean
- ✅ Documentation complete

### Should Have (P2)

- ✅ GDPR compliance reports (FR-005)
- ✅ DSAR support (FR-009, FR-010)
- ✅ Log rotation (FR-012)
- ✅ Performance targets met

### Could Have (P3)

- Integration with veil-cli (Phase 3)
- Performance benchmarks with criterion
- Examples in repository

---

## Timeline Estimate

**Estimated Effort**: 8-12 hours

| Phase | Estimated Time |
|-------|----------------|
| 2.1 Foundation | 1 hour |
| 2.2 Inventory Reports | 2-3 hours |
| 2.3 Compliance Reports | 2-3 hours |
| 2.4 DSAR Support | 2-3 hours |
| 2.5 Log Rotation | 1 hour |
| 2.6 Integration Tests | 1 hour |
| 2.7 Documentation | 0.5 hour |
| 3 CLI Integration | 1-2 hours (optional) |

---

## Sign-off

**Stakeholder**: Development Team
**Status**: Ready for Implementation
**Next Step**: Run `/speckit.tasks` to generate tasks.md
