# Tasks: WASM Browser Integration

**Input**: Design documents from `/specs/013-wasm-browser/`
**Prerequisites**: spec.md, research.md, data-model.md

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: US1, US2, US3, US4, US5 (maps to user stories from spec.md)

## Path Conventions

```text
Cargo.toml                    # Workspace manifest (add veil-wasm member)
crates/veil-wasm/
├── Cargo.toml
├── src/
│   ├── lib.rs               # WASM exports, init()
│   ├── types.rs             # Finding, ScanResult, ProtectResult, etc.
│   ├── error.rs             # WasmError, ErrorCode
│   ├── scan.rs              # scan(), scan_with_progress()
│   ├── protect.rs           # protect(), protect_with_progress()
│   └── utils.rs             # Format detection, validation helpers
├── tests/
│   └── web.rs               # wasm-pack browser tests
└── pkg/                     # wasm-pack build output (generated)

examples/
└── browser/
    ├── index.html           # Demo page
    ├── main.js              # Demo JS
    └── worker.js            # Web Worker example
```

---

## Phase 1: Setup (Shared Infrastructure) ✅

**Purpose**: Create veil-wasm crate and configure WASM build tooling

- [x] T001 Add veil-wasm to workspace members in Cargo.toml
- [x] T002 Create crates/veil-wasm/Cargo.toml with WASM dependencies (wasm-bindgen, js-sys, web-sys, serde-wasm-bindgen, wasm-bindgen-futures)
- [x] T003 [P] Create crates/veil-wasm/src/lib.rs with module declarations and #[wasm_bindgen] setup
- [x] T004 [P] Configure release profile for WASM optimization (opt-level="z", lto=true) in crates/veil-wasm/Cargo.toml
- [x] T005 Verify `cargo build -p veil-wasm --target wasm32-unknown-unknown` succeeds

**Checkpoint**: WASM crate compiles to wasm32 target ✅

---

## Phase 2: Foundational (Blocking Prerequisites) ✅

**Purpose**: Core types shared by all user stories - MUST complete before user stories

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T006 Create ErrorCode enum (InvalidInput, UnsupportedFormat, FileTooLarge, InvalidConfig, InternalError) in crates/veil-wasm/src/error.rs
- [x] T007 Create WasmError struct with code and message in crates/veil-wasm/src/error.rs
- [x] T008 [P] Create Finding struct (category, value, start, end, confidence) in crates/veil-wasm/src/types.rs
- [x] T009 [P] Create ScanOptions struct (filename, categories, min_confidence) in crates/veil-wasm/src/types.rs
- [x] T010 [P] Create ScanStats struct (bytes_processed, duration_ms, category_counts) in crates/veil-wasm/src/types.rs
- [x] T011 Create ScanResult struct (findings, stats) in crates/veil-wasm/src/types.rs
- [x] T012 [P] Create ProtectStyle enum (Labels, Redact, Mask) in crates/veil-wasm/src/types.rs
- [x] T013 [P] Create ProtectOptions struct (filename, style, categories) in crates/veil-wasm/src/types.rs
- [x] T014 [P] Create ProtectStats struct (replacements, protected_categories, duration_ms) in crates/veil-wasm/src/types.rs
- [x] T015 Create ProtectResult struct (data, stats) in crates/veil-wasm/src/types.rs
- [x] T016 Implement validation helpers (file size check, format detection) in crates/veil-wasm/src/utils.rs
- [x] T017 Export all types from crates/veil-wasm/src/lib.rs
- [x] T018 Verify `cargo build -p veil-wasm --target wasm32-unknown-unknown` succeeds with types

**Checkpoint**: Foundation ready - all types compile, user story implementation can begin ✅

---

## Phase 3: User Story 1 - Scan Document in Browser (Priority: P1) MVP ✅

**Goal**: User selects file, receives PII scan results entirely client-side

**Independent Test**: Load WASM in browser, call scan() with test data, verify findings returned

### Implementation for User Story 1

- [x] T019 [US1] Implement init() function for WASM module initialization in crates/veil-wasm/src/lib.rs
- [x] T020 [US1] Implement internal scan logic wrapping veil-parsers and veil-detect in crates/veil-wasm/src/scan.rs
- [x] T021 [US1] Implement scan(data: &[u8], options: JsValue) -> Result<JsValue, JsValue> in crates/veil-wasm/src/scan.rs
- [x] T022 [US1] Add format detection from filename option in crates/veil-wasm/src/scan.rs
- [x] T023 [US1] Add category filtering based on ScanOptions.categories in crates/veil-wasm/src/scan.rs
- [x] T024 [US1] Add confidence threshold filtering based on ScanOptions.min_confidence in crates/veil-wasm/src/scan.rs
- [x] T025 [US1] Wire scan() export in crates/veil-wasm/src/lib.rs
- [x] T026 [US1] Add unit test for scan with text input in crates/veil-wasm/src/scan.rs
- [x] T027 [US1] Verify wasm-pack build succeeds and scan() callable from JS

**Checkpoint**: scan() works - User Story 1 complete, MVP functional ✅

---

## Phase 4: User Story 2 - Protect Document in Browser (Priority: P1) ✅

**Goal**: User clicks Protect, downloads redacted file entirely client-side

**Independent Test**: Call protect() with test data containing PII, verify output has redactions

### Implementation for User Story 2

- [x] T028 [US2] Implement internal protect logic wrapping veil-detect and veil-redact in crates/veil-wasm/src/protect.rs
- [x] T029 [US2] Implement protect(data: &[u8], options: JsValue) -> Result<JsValue, JsValue> in crates/veil-wasm/src/protect.rs
- [x] T030 [US2] Map ProtectStyle enum to veil-redact RedactionStyle in crates/veil-wasm/src/protect.rs
- [x] T031 [US2] Add category filtering for selective protection in crates/veil-wasm/src/protect.rs
- [x] T032 [US2] Return ProtectResult with data as Uint8Array in crates/veil-wasm/src/protect.rs
- [x] T033 [US2] Wire protect() export in crates/veil-wasm/src/lib.rs
- [x] T034 [US2] Add unit test for protect with PII input in crates/veil-wasm/src/protect.rs
- [x] T035 [US2] Verify protect() callable from JS and returns redacted content

**Checkpoint**: protect() works - User Story 2 complete ✅

---

## Phase 5: User Story 3 - Integrate WASM Module in Web App (Priority: P1) ✅

**Goal**: Developer can npm install and import scan/protect with TypeScript types

**Independent Test**: Import module in TypeScript, verify types work, call functions

### Implementation for User Story 3

- [x] T036 [US3] Add #[wasm_bindgen(typescript_custom_section)] for custom TS types in crates/veil-wasm/src/lib.rs
- [x] T037 [US3] Ensure all public functions have proper JSDoc comments in crates/veil-wasm/src/lib.rs
- [x] T038 [US3] Create package.json template for npm publishing in crates/veil-wasm/package.json
- [x] T039 [US3] Create README.md with API documentation in crates/veil-wasm/README.md
- [x] T040 [US3] Create examples/browser/index.html demo page
- [x] T041 [US3] Create examples/browser/main.js with scan/protect usage
- [x] T042 [US3] Verify wasm-pack build --target web generates correct pkg/ output
- [x] T043 [US3] Verify TypeScript types are generated and correct in pkg/veil_wasm.d.ts

**Checkpoint**: npm package structure ready - User Story 3 complete ✅

---

## Phase 6: User Story 4 - Handle Large Files Efficiently (Priority: P2) ✅

**Goal**: 10MB+ files process without freezing browser, with progress updates

**Independent Test**: Pass 10MB buffer to scan_with_progress, verify progress callback fires

### Implementation for User Story 4

- [x] T044 [US4] Implement chunked processing for large inputs in crates/veil-wasm/src/scan.rs
- [x] T045 [US4] Implement scan_with_progress(data, options, on_progress: &Function) in crates/veil-wasm/src/scan.rs
- [x] T046 [US4] Implement protect_with_progress(data, options, on_progress: &Function) in crates/veil-wasm/src/protect.rs
- [x] T047 [US4] Add file size validation (reject >50MB with FileTooLarge error) in crates/veil-wasm/src/utils.rs
- [x] T048 [US4] Wire progress exports in crates/veil-wasm/src/lib.rs
- [x] T049 [US4] Create examples/browser/worker.js Web Worker example
- [x] T050 [US4] Update examples/browser/main.js to demonstrate progress callbacks
- [x] T051 [US4] Add test for progress callback invocation in crates/veil-wasm/src/scan.rs

**Checkpoint**: Large file handling works - User Story 4 complete ✅

---

## Phase 7: User Story 5 - Work Offline (Priority: P2) ✅

**Goal**: After initial load, app works without network connection

**Independent Test**: Load demo, disconnect network, scan file, verify it works

### Implementation for User Story 5

- [x] T052 [US5] Create examples/browser/sw.js Service Worker with WASM caching
- [x] T053 [US5] Add Service Worker registration to examples/browser/index.html
- [x] T054 [US5] Configure cache versioning strategy in examples/browser/sw.js
- [x] T055 [US5] Test offline functionality manually (load, disconnect, scan)

**Checkpoint**: Offline support works - User Story 5 complete ✅

---

## Phase 8: Polish & Cross-Cutting Concerns ✅

**Purpose**: Final validation, optimization, and cleanup

- [x] T056 [P] Add doc comments to all public items in crates/veil-wasm/src/lib.rs
- [x] T057 [P] Add doc comments to all types in crates/veil-wasm/src/types.rs
- [x] T058 Run wasm-pack build --release and verify bundle size <5MB
- [x] T059 Run cargo clippy -p veil-wasm -- -D warnings
- [x] T060 Run cargo fmt --check -p veil-wasm
- [x] T061 Test in Chrome, Firefox, Safari, Edge (manual browser test)
- [x] T062 Measure scan performance on 1MB file (target: <3 seconds)
- [x] T063 Verify zero network requests during scan/protect (DevTools check)

---

## Dependencies & Execution Order

### Phase Dependencies

```text
Phase 1 (Setup) → Phase 2 (Foundational) → User Stories can begin
                                                  ↓
                              ┌─────────────────────────────────────┐
                              │ P1 Stories (can run in parallel)    │
                              │ Phase 3 (US1) ──┬── Phase 4 (US2)   │
                              │                 └── Phase 5 (US3)   │
                              └─────────────────────────────────────┘
                                                  ↓
                              ┌─────────────────────────────────────┐
                              │ P2 Stories (after P1 complete)      │
                              │ Phase 6 (US4) ──┬── Phase 7 (US5)   │
                              └─────────────────────────────────────┘
                                                  ↓
                                        Phase 8 (Polish)
```

### User Story Dependencies

- **US1 (Scan)**: No dependencies on other stories - MVP core
- **US2 (Protect)**: No dependencies on other stories - MVP core
- **US3 (Integration)**: Depends on US1 and US2 being functional
- **US4 (Large Files)**: Extends US1/US2 with progress callbacks
- **US5 (Offline)**: Independent of other stories, just needs working WASM

### Crate Dependencies

```text
veil-wasm depends on:
├── veil-parsers (document parsing)
├── veil-detect (PII detection)
└── veil-redact (redaction engine)
```

---

## Parallel Opportunities

### Phase 2 (Foundational Types)

```bash
# These types can be created in parallel:
Task: "Create Finding struct in crates/veil-wasm/src/types.rs"
Task: "Create ScanOptions struct in crates/veil-wasm/src/types.rs"
Task: "Create ScanStats struct in crates/veil-wasm/src/types.rs"
Task: "Create ProtectStyle enum in crates/veil-wasm/src/types.rs"
Task: "Create ProtectOptions struct in crates/veil-wasm/src/types.rs"
Task: "Create ProtectStats struct in crates/veil-wasm/src/types.rs"
```

### P1 User Stories (After Foundational)

```bash
# US1, US2, US3 can proceed in parallel if staffed:
Developer A: Phase 3 (US1 - Scan)
Developer B: Phase 4 (US2 - Protect)
Developer C: Phase 5 (US3 - Integration/Docs)
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (Scan)
4. **STOP and VALIDATE**: wasm-pack build, test scan() in browser
5. MVP deliverable: Can scan files in browser

### Full P1 Delivery

1. Setup + Foundational → Crate compiles to WASM
2. Add US1 (Scan) → scan() works
3. Add US2 (Protect) → protect() works
4. Add US3 (Integration) → npm-ready package with types
5. **P1 Complete**: Full browser scanning and protection

### Incremental P2 Delivery

1. P1 Complete → Core functionality working
2. Add US4 (Large Files) → Progress callbacks, chunked processing
3. Add US5 (Offline) → Service Worker caching
4. Polish → Bundle optimization, cross-browser testing

---

## Summary

| Phase | Scope | Task Count | Status |
|-------|-------|------------|--------|
| 1. Setup | Crate initialization | 5 | ✅ |
| 2. Foundational | Core types | 13 | ✅ |
| 3. US1 (P1) | Scan in browser | 9 | ✅ |
| 4. US2 (P1) | Protect in browser | 8 | ✅ |
| 5. US3 (P1) | Developer integration | 8 | ✅ |
| 6. US4 (P2) | Large file handling | 8 | ✅ |
| 7. US5 (P2) | Offline support | 4 | ✅ |
| 8. Polish | Optimization/testing | 8 | ✅ |
| **Total** | | **63** | **✅** |

| Metric | Value |
|--------|-------|
| Total tasks | 63 |
| Completed | 63 ✅ |
| Parallel opportunities | 14 tasks marked [P] |
| MVP scope | Phase 1 + 2 + 3 (27 tasks) ✅ |
| Full P1 scope | Phase 1-5 (43 tasks) ✅ |

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Run `cargo clippy -p veil-wasm -- -D warnings` frequently
- Use `wasm-pack build --target web` for browser-compatible output
- Test with `wasm-pack test --headless --chrome` for automated browser tests
