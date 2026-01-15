# Tasks: Cryptographic Protection

**Input**: Design documents from `/specs/010-crypto-protection/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Included per constitution requirement (TDD - tests fail before implementation)

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story (US1-US5 maps to spec.md user stories)
- Exact file paths included in descriptions

## Path Conventions

- **Crate location**: `crates/veil-crypto/`
- **Tests**: `crates/veil-crypto/tests/`
- **Integration tests**: `tests/crypto/`

---

## Phase 1: Setup (Project Infrastructure) ✅

**Purpose**: Create crate structure and add dependencies

- [x] T001 Create crate directory structure per plan.md in crates/veil-crypto/
- [x] T002 Create Cargo.toml with dependencies (aes-gcm, sha2, hmac, rand, fake, uuid, base64, zeroize, subtle) in crates/veil-crypto/Cargo.toml
- [x] T003 Add veil-crypto to workspace members in Cargo.toml (workspace root)
- [x] T004 [P] Create lib.rs with module declarations in crates/veil-crypto/src/lib.rs
- [x] T005 [P] Create error.rs with CryptoError enum in crates/veil-crypto/src/error.rs
- [x] T006 [P] Create types.rs with shared types (ProtectionMode, OutputFormat) in crates/veil-crypto/src/types.rs

---

## Phase 2: Foundational (Blocking Prerequisites) ✅

**Purpose**: Core types and utilities that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T007 Implement ProtectionMode enum in crates/veil-crypto/src/types.rs
- [x] T008 [P] Implement OutputFormat enum (Base64, Hex) in crates/veil-crypto/src/types.rs
- [x] T009 [P] Implement CryptoResult struct in crates/veil-crypto/src/types.rs
- [x] T010 [P] Implement CryptoMetadata struct in crates/veil-crypto/src/types.rs
- [x] T011 Implement base64/hex encoding utilities in crates/veil-crypto/src/types.rs
- [x] T012 Add CryptoError variants (InvalidKeyLength, InvalidIvLength, EncryptionFailed, DecryptionFailed, InvalidCiphertext, HashingFailed, PseudonymFailed, TokenFailed, Vault, InvalidConfig) in crates/veil-crypto/src/error.rs
- [x] T013 Verify crate compiles with cargo build -p veil-crypto

**Checkpoint**: Foundation ready - user story implementation can now begin ✅

---

## Phase 3: User Story 1 - Encrypt Sensitive Data (Priority: P1) 🎯 MVP ✅

**Goal**: AES-256-GCM encryption with key, reversible decryption

**Independent Test**: Encrypt text, verify ciphertext differs, decrypt with key, verify original recovered

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T014 [P] [US1] Test encrypt produces base64 output in crates/veil-crypto/src/encrypt.rs
- [x] T015 [P] [US1] Test decrypt recovers original plaintext in crates/veil-crypto/src/encrypt.rs
- [x] T016 [P] [US1] Test wrong key fails decryption in crates/veil-crypto/src/encrypt.rs
- [x] T017 [P] [US1] Test invalid key length returns error in crates/veil-crypto/src/encrypt.rs
- [x] T018 [P] [US1] Test tampered ciphertext fails authentication in crates/veil-crypto/src/encrypt.rs

### Implementation for User Story 1

- [x] T019 [US1] Create EncryptionConfig struct in crates/veil-crypto/src/encrypt.rs
- [x] T020 [US1] Implement encrypt() function using aes-gcm in crates/veil-crypto/src/encrypt.rs
- [x] T021 [US1] Implement decrypt() function using aes-gcm in crates/veil-crypto/src/encrypt.rs
- [x] T022 [US1] Add IV generation using rand::OsRng in crates/veil-crypto/src/encrypt.rs
- [x] T023 [US1] Implement output format encoding (base64/hex) in crates/veil-crypto/src/encrypt.rs
- [x] T024 [US1] Add Zeroize derive for EncryptionConfig key in crates/veil-crypto/src/encrypt.rs
- [x] T025 [US1] Export encrypt module from lib.rs in crates/veil-crypto/src/lib.rs
- [x] T026 [US1] Verify all US1 tests pass with cargo test -p veil-crypto

**Checkpoint**: Encryption/decryption fully functional and tested ✅

---

## Phase 4: User Story 2 - Hash Data Irreversibly (Priority: P1) ✅

**Goal**: SHA-256/SHA-512 hashing with salts, consistent output for same input+salt

**Independent Test**: Hash same value twice with same salt, verify identical output

### Tests for User Story 2

- [x] T027 [P] [US2] Test sha256 hash produces hex output in crates/veil-crypto/src/hash.rs
- [x] T028 [P] [US2] Test sha512 hash produces hex output in crates/veil-crypto/src/hash.rs
- [x] T029 [P] [US2] Test same input+salt produces same hash in crates/veil-crypto/src/hash.rs
- [x] T030 [P] [US2] Test different salt produces different hash in crates/veil-crypto/src/hash.rs
- [x] T031 [P] [US2] Test keyed hash (HMAC) produces consistent output in crates/veil-crypto/src/hash.rs

### Implementation for User Story 2

- [x] T032 [US2] Create HashConfig struct in crates/veil-crypto/src/hash.rs
- [x] T033 [US2] Create HashAlgorithm enum (Sha256, Sha512) in crates/veil-crypto/src/hash.rs
- [x] T034 [US2] Implement hash() function for SHA-256 in crates/veil-crypto/src/hash.rs
- [x] T035 [US2] Implement hash() function for SHA-512 in crates/veil-crypto/src/hash.rs
- [x] T036 [US2] Implement keyed hash (HMAC) support in crates/veil-crypto/src/hash.rs
- [x] T037 [US2] Add salt generation utility in crates/veil-crypto/src/hash.rs
- [x] T038 [US2] Export hash module from lib.rs in crates/veil-crypto/src/lib.rs
- [x] T039 [US2] Verify all US2 tests pass with cargo test -p veil-crypto

**Checkpoint**: Hashing fully functional and tested ✅

---

## Phase 5: User Story 3 - Pseudonymize with Fake Data (Priority: P1) ✅

**Goal**: Replace real data with realistic fake data per locale

**Independent Test**: Pseudonymize names, verify fake names are realistic

### Tests for User Story 3

- [x] T040 [P] [US3] Test pseudonymize name produces different name in crates/veil-crypto/src/pseudonym.rs
- [x] T041 [P] [US3] Test pseudonymize email produces valid email in crates/veil-crypto/src/pseudonym.rs
- [x] T042 [P] [US3] Test locale affects output format in crates/veil-crypto/src/pseudonym.rs
- [x] T043 [P] [US3] Test consistent mode returns same output in crates/veil-crypto/src/pseudonym.rs

### Implementation for User Story 3

- [x] T044 [US3] Create PseudonymConfig struct in crates/veil-crypto/src/pseudonym.rs
- [x] T045 [US3] Create PseudonymDataType enum in crates/veil-crypto/src/pseudonym.rs
- [x] T046 [US3] Implement pseudonymize() for names using fake crate in crates/veil-crypto/src/pseudonym.rs
- [x] T047 [US3] Implement pseudonymize() for emails using fake crate in crates/veil-crypto/src/pseudonym.rs
- [x] T048 [US3] Implement pseudonymize() for phone/address in crates/veil-crypto/src/pseudonym.rs
- [x] T049 [US3] Add locale support (de_DE, en_US) in crates/veil-crypto/src/pseudonym.rs
- [x] T050 [US3] Add consistent mode with seeded RNG in crates/veil-crypto/src/pseudonym.rs
- [x] T051 [US3] Export pseudonym module from lib.rs in crates/veil-crypto/src/lib.rs
- [x] T052 [US3] Verify all US3 tests pass with cargo test -p veil-crypto

**Checkpoint**: Pseudonymization fully functional and tested ✅

---

## Phase 6: User Story 4 - Tokenize with Vault Storage (Priority: P2) ✅

**Goal**: Replace PII with tokens, store mapping in vault for reversal

**Independent Test**: Tokenize values, verify tokens are random, verify detokenization works

### Tests for User Story 4

- [x] T053 [P] [US4] Test tokenize produces UUID token in crates/veil-crypto/src/tokenize.rs
- [x] T054 [P] [US4] Test detokenize retrieves original in crates/veil-crypto/src/tokenize.rs
- [x] T055 [P] [US4] Test consistent mode returns same token in crates/veil-crypto/src/tokenize.rs
- [x] T056 [P] [US4] Test token with prefix format in crates/veil-crypto/src/tokenize.rs

### Vault Implementation for User Story 4

- [x] T057 [US4] Create vault/mod.rs with TokenVault trait in crates/veil-crypto/src/vault/mod.rs
- [x] T058 [US4] Create VaultError enum in crates/veil-crypto/src/vault/mod.rs
- [x] T059 [US4] Implement InMemoryVault with store/lookup/delete in crates/veil-crypto/src/vault/memory.rs
- [x] T060 [US4] Implement find_by_original for consistency in crates/veil-crypto/src/vault/memory.rs
- [x] T061 [P] [US4] Test InMemoryVault operations in crates/veil-crypto/src/vault/memory.rs

### Tokenization Implementation for User Story 4

- [x] T062 [US4] Create TokenConfig struct in crates/veil-crypto/src/tokenize.rs
- [x] T063 [US4] Create TokenFormat enum (Uuid, Hex16, Hex32, Alphanumeric) in crates/veil-crypto/src/tokenize.rs
- [x] T064 [US4] Implement tokenize() function in crates/veil-crypto/src/tokenize.rs
- [x] T065 [US4] Implement detokenize() function in crates/veil-crypto/src/tokenize.rs
- [x] T066 [US4] Add token prefix support in crates/veil-crypto/src/tokenize.rs
- [x] T067 [US4] Export tokenize and vault modules from lib.rs in crates/veil-crypto/src/lib.rs
- [x] T068 [US4] Verify all US4 tests pass with cargo test -p veil-crypto

**Checkpoint**: Tokenization with vault fully functional and tested ✅

---

## Phase 7: User Story 5 - Consistent Pseudonyms Across Sessions (Priority: P2) ✅

**Goal**: Same name always gets same pseudonym when using same seed

**Independent Test**: Pseudonymize same name in two sessions with same seed, verify same output

### Tests for User Story 5

- [x] T069 [P] [US5] Test seed produces deterministic output in crates/veil-crypto/src/pseudonym.rs
- [x] T070 [P] [US5] Test different seeds produce different outputs in crates/veil-crypto/src/pseudonym.rs
- [x] T071 [P] [US5] Test input-derived seed for automatic consistency in crates/veil-crypto/src/pseudonym.rs

### Implementation for User Story 5

- [x] T072 [US5] Add seed derivation from input hash in crates/veil-crypto/src/pseudonym.rs
- [x] T073 [US5] Implement deterministic RNG from seed in crates/veil-crypto/src/pseudonym.rs
- [x] T074 [US5] Add cross-session consistency documentation in crates/veil-crypto/src/pseudonym.rs
- [x] T075 [US5] Verify all US5 tests pass with cargo test -p veil-crypto

**Checkpoint**: Cross-session consistency fully functional and tested ✅

---

## Phase 8: High-Level API & Integration ✅

**Purpose**: Unified Protector API and cross-module integration

### Protector API

- [x] T076 [P] Test Protector with all protection modes in crates/veil-crypto/src/protector.rs
- [x] T077 Create CryptoConfig builder in crates/veil-crypto/src/protector.rs
- [x] T078 Create Protector struct in crates/veil-crypto/src/protector.rs
- [x] T079 Implement protect() method dispatching to mode-specific functions in crates/veil-crypto/src/protector.rs
- [x] T080 Add vault injection for tokenization in crates/veil-crypto/src/protector.rs
- [x] T081 Export protector module from lib.rs in crates/veil-crypto/src/lib.rs

### Integration

- [x] T082 Add veil-crypto dependency to veil-wasm in crates/veil-wasm/Cargo.toml
- [x] T083 [P] Create WASM bindings for encrypt/decrypt in crates/veil-wasm/src/crypto.rs
- [x] T084 [P] Create WASM bindings for hash in crates/veil-wasm/src/crypto.rs
- [x] T085 [P] Create WASM bindings for pseudonymize in crates/veil-wasm/src/crypto.rs
- [x] T086 Export crypto module from veil-wasm lib.rs in crates/veil-wasm/src/lib.rs

---

## Phase 9: Polish & Cross-Cutting Concerns ✅

**Purpose**: Documentation, performance, final validation

- [x] T087 [P] Add documentation comments to all public items in crates/veil-crypto/src/lib.rs
- [x] T088 [P] Add documentation comments to encrypt.rs in crates/veil-crypto/src/encrypt.rs
- [x] T089 [P] Add documentation comments to hash.rs in crates/veil-crypto/src/hash.rs
- [x] T090 [P] Add documentation comments to pseudonym.rs in crates/veil-crypto/src/pseudonym.rs
- [x] T091 [P] Add documentation comments to tokenize.rs in crates/veil-crypto/src/tokenize.rs
- [x] T092 Run cargo clippy -p veil-crypto -- -D warnings
- [x] T093 Run cargo fmt --check -p veil-crypto
- [x] T094 Performance test: 10,000 encryptions under 1 second
- [x] T095 Run full workspace tests with cargo test
- [x] T096 Validate quickstart.md examples compile

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Story 1-3 (Phase 3-5)**: All P1, can run in parallel after Foundational
- **User Story 4-5 (Phase 6-7)**: P2 stories, can run in parallel after Foundational
- **High-Level API (Phase 8)**: Depends on all user stories complete
- **Polish (Phase 9)**: Depends on Phase 8 complete

### User Story Dependencies

- **US1 (Encryption)**: Independent - no dependencies on other stories
- **US2 (Hashing)**: Independent - no dependencies on other stories
- **US3 (Pseudonymization)**: Independent - no dependencies on other stories
- **US4 (Tokenization)**: Independent - vault is self-contained
- **US5 (Consistent Pseudonyms)**: Extends US3 - implements after US3 complete

### Within Each User Story

- Tests MUST fail before implementation
- Config structs before functions
- Core functions before utilities
- Exports last
- All tests pass before checkpoint

### Parallel Opportunities

- T004, T005, T006 can run in parallel (different files)
- T008, T009, T010 can run in parallel (same file but independent sections)
- All test tasks within a story can run in parallel
- US1, US2, US3 can run in parallel (different modules)
- US4, US5 can run in parallel (different modules)
- T083, T084, T085 can run in parallel (WASM bindings)
- T087-T091 can run in parallel (documentation)

---

## Parallel Example: User Story 1

```bash
# Launch all tests for US1 together:
Task: "Test encrypt produces base64 output"
Task: "Test decrypt recovers original plaintext"
Task: "Test wrong key fails decryption"
Task: "Test invalid key length returns error"
Task: "Test tampered ciphertext fails authentication"

# After tests fail, implement sequentially:
Task: "Create EncryptionConfig struct"
Task: "Implement encrypt() function"
Task: "Implement decrypt() function"
...
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (Encryption)
4. **STOP and VALIDATE**: Test encryption independently
5. Can deploy with encrypt/decrypt capability

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 (Encryption) → Reversible protection available
3. Add US2 (Hashing) → Irreversible protection available
4. Add US3 (Pseudonymization) → Fake data available
5. Add US4 (Tokenization) → Token vault available
6. Add US5 (Consistent Pseudonyms) → Cross-session consistency
7. High-Level API → Unified interface
8. Polish → Production ready

### Parallel Team Strategy

With multiple developers after Foundational:
- Developer A: US1 (Encryption) + US2 (Hashing)
- Developer B: US3 (Pseudonymization) + US5 (Consistency)
- Developer C: US4 (Tokenization with Vault)

---

## Summary

- **Total Tasks**: 96
- **Completed**: 96 ✅
- **Phase 1 (Setup)**: 6/6 ✅
- **Phase 2 (Foundational)**: 7/7 ✅
- **Phase 3 (US1 Encryption)**: 13/13 ✅
- **Phase 4 (US2 Hashing)**: 13/13 ✅
- **Phase 5 (US3 Pseudonymization)**: 13/13 ✅
- **Phase 6 (US4 Tokenization)**: 16/16 ✅
- **Phase 7 (US5 Consistency)**: 7/7 ✅
- **Phase 8 (High-Level API)**: 11/11 ✅
- **Phase 9 (Polish)**: 10/10 ✅
- **Parallel Opportunities**: 47 tasks marked [P]
- **MVP Scope**: Phases 1-3 (26 tasks for encryption capability) ✅

---

## Notes

- [P] tasks = different files, no dependencies
- [US#] label maps task to specific user story
- Each story independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story
- Constitution: Use Result<T,E>, no unwrap on user input, audited crypto libs
