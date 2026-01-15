# Tasks: Codebase Excellence Initiative

**Input**: Design documents from `/specs/022-codebase-improvements/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/openapi.yaml

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, etc.)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Install development tools and configure CI baseline

- [x] T001 Install cargo-fuzz, cargo-mutants, cargo-llvm-cov, wasm-pack, pre-commit, just
- [x] T002 [P] Add memchr = "2" to crates/veil-detect/Cargo.toml
- [x] T003 [P] Add criterion = "0.5" as dev-dependency to workspace Cargo.toml
- [x] T004 [P] Add utoipa = "4" to crates/veil-api/Cargo.toml
- [x] T005 [P] Add proptest = "1" as dev-dependency to relevant crates

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: CI infrastructure that validates all subsequent changes

**⚠️ CRITICAL**: Complete before user story work begins

- [x] T006 Add cargo-audit step to .github/workflows/ci.yml
- [x] T007 [P] Add cargo-llvm-cov step to .github/workflows/ci.yml with 90% threshold
- [x] T008 [P] Add cargo-mutants step to .github/workflows/ci.yml with 80% threshold
- [x] T009 Create .github/workflows/release.yml with SBOM generation using cargo-sbom

**Checkpoint**: CI now enforces security, coverage, and mutation testing

---

## Phase 3: User Story 1 - Security Hardening (Priority: P1) 🎯 MVP

**Goal**: Encrypt vault contents at rest with key rotation support

**Independent Test**: `cargo test -p veil-crypto --features encrypted-vault`

**FR Coverage**: FR-001, FR-002, FR-003, FR-004, FR-005, FR-006

### Implementation for User Story 1

- [x] T010 [P] [US1] Define KeyProvider trait in crates/veil-crypto/src/key_provider.rs
- [x] T011 [P] [US1] Implement LocalKeyProvider (file-based) in crates/veil-crypto/src/key_provider.rs
- [x] T012 [P] [US1] Implement EnvKeyProvider (env vars) in crates/veil-crypto/src/key_provider.rs
- [x] T013 [US1] Define EncryptedVault struct with AES-256-GCM in crates/veil-crypto/src/vault.rs
- [x] T014 [US1] Implement vault encryption/decryption in crates/veil-crypto/src/vault.rs
- [x] T015 [US1] Implement key rotation mechanism in crates/veil-crypto/src/vault.rs
- [x] T016 [US1] Add tests for EncryptedVault in crates/veil-crypto/src/vault.rs
- [x] T017 [P] [US1] Create fuzz/fuzz_targets/pdf_parser.rs with cargo-fuzz target
- [x] T018 [P] [US1] Create fuzz/fuzz_targets/email_parser.rs with cargo-fuzz target
- [x] T019 [P] [US1] Create fuzz/fuzz_targets/office_parser.rs with cargo-fuzz target
- [x] T020 [P] [US1] Create fuzz/fuzz_targets/policy_yaml.rs with cargo-fuzz target
- [x] T021 [US1] Create fuzz/Cargo.toml with fuzz target definitions

**Checkpoint**: Vault data encrypted, key rotation works, fuzz targets exist

---

## Phase 4: User Story 2 - Comprehensive Test Coverage (Priority: P1)

**Goal**: Fill test gaps and enable mutation testing CI gate

**Independent Test**: `cargo test --workspace && cargo mutants --check`

**FR Coverage**: FR-007, FR-008, FR-009, FR-010, FR-011, FR-012, FR-013, FR-014

### Implementation for User Story 2

- [x] T022 [P] [US2] Add YAML parsing integration tests in crates/veil-policy/tests/yaml_parsing.rs
- [x] T023 [P] [US2] Add rule application tests in crates/veil-policy/tests/rule_application.rs
- [x] T024 [P] [US2] Add policy inheritance tests in crates/veil-policy/tests/inheritance.rs
- [x] T025 [P] [US2] Add SensitiveString unit tests in crates/veil-core/src/sensitive.rs
- [x] T026 [P] [US2] Add constant-time comparison tests for SensitiveString in crates/veil-core/src/sensitive.rs
- [ ] T027 [US2] Configure wasm-pack test for headless Chrome in crates/veil-wasm/Cargo.toml
- [ ] T028 [US2] Add browser-based tests in crates/veil-wasm/tests/web.rs
- [x] T029 [P] [US2] Add mask style tests in crates/veil-redact/tests/styles.rs
- [x] T030 [P] [US2] Add symlink handling tests in crates/veil-discovery/tests/symlinks.rs
- [x] T031 [P] [US2] Add archive handling tests in crates/veil-discovery/tests/archives.rs
- [x] T032 [P] [US2] Add proptest for email validator in crates/veil-detect/tests/proptest_email.rs
- [x] T033 [P] [US2] Add proptest for phone validator in crates/veil-detect/tests/proptest_phone.rs
- [x] T034 [P] [US2] Add proptest for SSN validator in crates/veil-detect/tests/proptest_ssn.rs

**Checkpoint**: All crates have comprehensive tests, mutation testing passes

---

## Phase 5: User Story 3 - Complete API Documentation (Priority: P2)

**Goal**: Full rustdoc coverage, architecture docs, and OpenAPI spec

**Independent Test**: `cargo doc --no-deps && ls docs/ARCHITECTURE.md`

**FR Coverage**: FR-015, FR-016, FR-017, FR-018, FR-019, FR-020

### Implementation for User Story 3

- [ ] T035 [P] [US3] Add rustdoc comments to crates/veil-core/src/lib.rs public items
- [ ] T036 [P] [US3] Add rustdoc comments to crates/veil-detect/src/lib.rs public items
- [ ] T037 [P] [US3] Add rustdoc comments to crates/veil-parsers/src/lib.rs public items
- [ ] T038 [P] [US3] Add rustdoc comments to crates/veil-redact/src/lib.rs public items
- [ ] T039 [P] [US3] Add rustdoc comments to crates/veil-policy/src/lib.rs public items
- [ ] T040 [P] [US3] Add rustdoc comments to crates/veil-api/src/lib.rs public items
- [ ] T041 [P] [US3] Add rustdoc comments to crates/veil-wasm/src/lib.rs public items
- [x] T042 [US3] Create docs/ARCHITECTURE.md with crate dependency Mermaid diagram
- [x] T043 [P] [US3] Create examples/basic_scan.rs with documented example
- [x] T044 [P] [US3] Create examples/batch_processing.rs with documented example
- [x] T045 [P] [US3] Create examples/policy_usage.rs with documented example
- [x] T046 [P] [US3] Create examples/streaming.rs with documented example
- [x] T047 [US3] Create CHANGELOG.md in keepachangelog format at repository root
- [ ] T048 [US3] Add utoipa annotations to crates/veil-api/src/handlers.rs
- [ ] T049 [US3] Generate OpenAPI spec to docs/api/openapi.yaml
- [ ] T050 [US3] Add ts-rs derive macros to crates/veil-wasm/src/types.rs
- [ ] T051 [US3] Generate TypeScript definitions to crates/veil-wasm/pkg/veil_wasm.d.ts

**Checkpoint**: `cargo doc` succeeds, ARCHITECTURE.md exists, OpenAPI spec generated

---

## Phase 6: User Story 4 - Extended API Capabilities (Priority: P2)

**Goal**: Batch endpoint and WASM PDF support

**Independent Test**: `curl -X POST localhost:8080/api/v1/batch` and WASM PDF test

**FR Coverage**: FR-021, FR-022, FR-023

### Implementation for User Story 4

- [ ] T052 [US4] Define BatchRequest and BatchResponse types in crates/veil-api/src/types.rs
- [ ] T053 [US4] Implement POST /api/v1/batch handler in crates/veil-api/src/handlers/batch.rs
- [ ] T054 [US4] Add parallel processing option to batch handler in crates/veil-api/src/handlers/batch.rs
- [ ] T055 [US4] Add ETag header support to scan/protect endpoints in crates/veil-api/src/middleware.rs
- [ ] T056 [US4] Add pdf-extract dependency to crates/veil-wasm/Cargo.toml with wasm feature
- [ ] T057 [US4] Implement PDF text extraction in crates/veil-wasm/src/pdf.rs
- [ ] T058 [US4] Add PDF scanning JavaScript API in crates/veil-wasm/src/lib.rs
- [ ] T059 [US4] Add batch endpoint tests in crates/veil-api/tests/batch.rs

**Checkpoint**: Batch endpoint works, WASM can process PDFs

---

## Phase 7: User Story 5 - Performance Optimization (Priority: P2)

**Goal**: Parallel detection, streaming parsers, regex caching, benchmarks

**Independent Test**: `cargo bench` shows 50%+ improvement

**FR Coverage**: FR-024, FR-025, FR-026, FR-027, FR-028, FR-029

### Implementation for User Story 5

- [ ] T060 [US5] Add rayon parallel iterator to segment processing in crates/veil-detect/src/engine.rs
- [ ] T061 [US5] Add StreamingParserConfig struct in crates/veil-parsers/src/config.rs
- [ ] T062 [US5] Implement streaming JSON parser using serde_json::StreamDeserializer in crates/veil-parsers/src/json.rs
- [ ] T063 [US5] Implement streaming HTML parser in crates/veil-parsers/src/html.rs
- [ ] T064 [US5] Add LRU regex cache in crates/veil-detect/src/cache.rs
- [ ] T065 [US5] Integrate regex cache into detection engine in crates/veil-detect/src/engine.rs
- [ ] T066 [US5] Add memory_used and streaming_used fields to ParseResult in crates/veil-parsers/src/result.rs
- [ ] T067 [US5] Add configurable memory bounds in crates/veil-parsers/src/config.rs
- [ ] T068 [P] [US5] Create benches/detection.rs with criterion benchmarks
- [ ] T069 [P] [US5] Create benches/parsing.rs with criterion benchmarks
- [ ] T070 [P] [US5] Create benches/encryption.rs with criterion benchmarks
- [ ] T071 [US5] Add benchmark CI job to .github/workflows/ci.yml

**Checkpoint**: Benchmarks show 50%+ throughput improvement, memory bounded

---

## Phase 8: User Story 6 - Developer Experience & Maintainability (Priority: P3)

**Goal**: Contributor docs, pre-commit hooks, dependabot, justfile

**Independent Test**: `pre-commit run --all-files && just test`

**FR Coverage**: FR-030, FR-031, FR-032, FR-033, FR-034, FR-035, FR-036, FR-037

### Implementation for User Story 6

- [x] T072 [US6] Create CONTRIBUTING.md with setup instructions at repository root
- [x] T073 [P] [US6] Create .github/ISSUE_TEMPLATE/bug_report.md
- [x] T074 [P] [US6] Create .github/ISSUE_TEMPLATE/feature_request.md
- [x] T075 [P] [US6] Create .github/PULL_REQUEST_TEMPLATE.md with checklist
- [ ] T076 [US6] Add release-please or git-cliff to .github/workflows/release.yml
- [x] T077 [US6] Create .pre-commit-config.yaml with rustfmt and clippy hooks
- [ ] T078 [US6] Add #![deny(missing_docs)] to crates/veil-core/src/lib.rs
- [ ] T079 [P] [US6] Add #![deny(missing_docs)] to crates/veil-detect/src/lib.rs
- [ ] T080 [P] [US6] Add #![deny(missing_docs)] to crates/veil-parsers/src/lib.rs
- [ ] T081 [P] [US6] Add #![deny(missing_docs)] to crates/veil-redact/src/lib.rs
- [ ] T082 [P] [US6] Add #![deny(missing_docs)] to crates/veil-api/src/lib.rs
- [ ] T083 [P] [US6] Add #![deny(missing_docs)] to crates/veil-wasm/src/lib.rs
- [x] T084 [US6] Create .github/dependabot.yml for cargo and GitHub Actions
- [x] T085 [US6] Create justfile with test, coverage, mutants, fuzz, check, docs, bench commands

**Checkpoint**: `pre-commit install` works, `just test` runs all tests

---

## Phase 9: User Story 7 - Lean Build (Priority: P3)

**Goal**: Feature flags to exclude heavy dependencies

**Independent Test**: Compare binary size with/without features

**FR Coverage**: FR-038

### Implementation for User Story 7

- [ ] T086 [US7] Define feature flags in workspace Cargo.toml (pdf, email, office)
- [ ] T087 [US7] Gate pdf-extract behind "pdf" feature in crates/veil-parsers/Cargo.toml
- [ ] T088 [P] [US7] Gate email parsing behind "email" feature in crates/veil-email/Cargo.toml
- [ ] T089 [P] [US7] Gate office parsing behind "office" feature in crates/veil-office/Cargo.toml
- [ ] T090 [US7] Update default features to minimal set in workspace Cargo.toml
- [ ] T091 [US7] Add feature documentation to README.md

**Checkpoint**: Default build excludes heavy deps, binary size reduced 30%+

---

## Phase 10: User Story 8 - Code Quality Excellence (Priority: P3)

**Goal**: Named constants, memchr optimization in hot paths

**Independent Test**: `grep -r "magic number" crates/` returns nothing

**FR Coverage**: FR-039, FR-040

### Implementation for User Story 8

- [ ] T092 [US8] Extract magic numbers to constants in crates/veil-detect/src/constants.rs
- [ ] T093 [P] [US8] Extract magic numbers to constants in crates/veil-parsers/src/constants.rs
- [ ] T094 [P] [US8] Extract magic numbers to constants in crates/veil-crypto/src/constants.rs
- [ ] T095 [US8] Replace byte iteration with memchr in crates/veil-detect/src/scanner.rs
- [ ] T096 [US8] Add memchr to pattern matching hot paths in crates/veil-detect/src/matchers.rs
- [ ] T097 [US8] Add benchmark comparison for memchr vs naive iteration in benches/detection.rs

**Checkpoint**: No magic numbers, benchmarks show memchr speedup

---

## Phase 11: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and cleanup

- [ ] T098 Run full test suite with `cargo test --workspace --all-features`
- [ ] T099 Run coverage and verify 90% threshold with `cargo llvm-cov`
- [ ] T100 Run mutation testing and verify 80% score with `cargo mutants`
- [ ] T101 Run cargo-audit and verify zero warnings
- [ ] T102 Validate quickstart.md instructions work end-to-end
- [ ] T103 Update README.md with new features and badges

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Stories (Phase 3-10)**: All depend on Foundational completion
  - US1 (Security) and US2 (Testing) are P1 - do first
  - US3-5 are P2 - can run in parallel after P1
  - US6-8 are P3 - can run in parallel after P2
- **Polish (Phase 11)**: Depends on all user stories complete

### User Story Dependencies

| Story | Priority | Dependencies | Can Parallelize With |
|-------|----------|--------------|---------------------|
| US1 | P1 | Foundational | US2 |
| US2 | P1 | Foundational | US1 |
| US3 | P2 | Foundational | US4, US5 |
| US4 | P2 | Foundational | US3, US5 |
| US5 | P2 | Foundational | US3, US4 |
| US6 | P3 | Foundational, US3 (for docs lint) | US7, US8 |
| US7 | P3 | Foundational | US6, US8 |
| US8 | P3 | Foundational, US5 (for benchmarks) | US6, US7 |

### Parallel Opportunities

**Within Phase 1 (Setup):**
```
T002, T003, T004, T005 can run in parallel
```

**Within Phase 3 (US1 - Security):**
```
T010, T011, T012 can run in parallel (KeyProvider implementations)
T017, T018, T019, T020 can run in parallel (fuzz targets)
```

**Within Phase 4 (US2 - Testing):**
```
T022, T023, T024, T025, T026, T029, T030, T031 can run in parallel
T032, T033, T034 can run in parallel (proptests)
```

**Within Phase 5 (US3 - Documentation):**
```
T035-T041 can run in parallel (rustdoc per crate)
T043, T044, T045, T046 can run in parallel (examples)
```

---

## Implementation Strategy

### MVP First (User Stories 1-2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CI gates)
3. Complete Phase 3: User Story 1 (Security)
4. Complete Phase 4: User Story 2 (Testing)
5. **STOP and VALIDATE**: Security + Testing foundation complete
6. All subsequent changes are protected by CI gates

### Incremental Delivery

1. Setup + Foundational → CI enforces quality
2. US1 + US2 → Security and testing foundation (MVP)
3. US3 → Documentation complete
4. US4 + US5 → API and performance improvements
5. US6 + US7 + US8 → Developer experience polish

---

## Summary

| Metric | Count |
|--------|-------|
| Total Tasks | 103 |
| Setup Tasks | 5 |
| Foundational Tasks | 4 |
| US1 Tasks | 12 |
| US2 Tasks | 13 |
| US3 Tasks | 17 |
| US4 Tasks | 8 |
| US5 Tasks | 12 |
| US6 Tasks | 14 |
| US7 Tasks | 6 |
| US8 Tasks | 6 |
| Polish Tasks | 6 |
| Parallelizable Tasks | 52 |

---

## Notes

- [P] tasks can run in parallel (different files)
- Each user story is independently testable
- Commit after each task or logical group
- Stop at checkpoints to validate progress
- US1 and US2 form the MVP - complete first
