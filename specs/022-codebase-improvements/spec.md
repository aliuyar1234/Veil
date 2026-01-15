# Feature Specification: Codebase Excellence Initiative

**Feature Branch**: `022-codebase-improvements`
**Created**: 2025-12-18
**Status**: Draft

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Security Hardening (Priority: P1)

A security-conscious organization deploying Veil in production needs confidence that sensitive PII data is protected at rest, encryption keys can be rotated without downtime, and the system has been thoroughly tested against malformed inputs.

**Why this priority**: Security is the foundation of a PII protection tool. Organizations handling PII face regulatory requirements (GDPR, CCPA) that mandate encryption at rest.

**Acceptance Scenarios**:

1. **Given** a FileVault with stored tokens, **When** the vault file is accessed directly on disk, **Then** all mappings are encrypted and unreadable without the encryption key
2. **Given** an active system with encrypted data, **When** an administrator initiates key rotation, **Then** all data is re-encrypted without service interruption
3. **Given** malformed input files, **When** processed by parsers, **Then** no crashes, hangs, or memory corruption occurs (verified via fuzzing)

---

### User Story 2 - Comprehensive Test Coverage (Priority: P1)

A development team maintaining Veil needs confidence that changes do not introduce regressions. Some crates have insufficient test coverage and there is no mutation testing.

**Why this priority**: Tests are the safety net for all other improvements. Without comprehensive coverage, any new feature risks breaking existing functionality.

**Acceptance Scenarios**:

1. **Given** the veil-policy crate, **When** tests are run, **Then** YAML parsing, rule application, and policy inheritance are covered
2. **Given** the veil-wasm crate, **When** browser tests are run, **Then** all JavaScript API functions are tested in headless environments
3. **Given** any code change, **When** mutation testing runs, **Then** at least 80% of mutations are detected by existing tests

---

### User Story 3 - Complete API Documentation (Priority: P2)

A developer integrating Veil needs comprehensive documentation including API reference, architecture overview, and working examples.

**Why this priority**: Documentation reduces support burden and accelerates adoption.

**Acceptance Scenarios**:

1. **Given** any public function or struct, **When** documentation is generated, **Then** rustdoc shows comprehensive doc comments with examples
2. **Given** a new developer, **When** they read ARCHITECTURE.md, **Then** they understand the crate dependency graph within 10 minutes
3. **Given** the REST API, **When** developers access the OpenAPI spec, **Then** all endpoints are documented with schemas

---

### User Story 4 - Extended API Capabilities (Priority: P2)

An application developer needs batch processing endpoints and PDF support in browser environments.

**Why this priority**: These capabilities unlock new use cases and reduce API call overhead for bulk operations.

**Acceptance Scenarios**:

1. **Given** multiple files to process, **When** a batch request is sent, **Then** all files are processed in a single response
2. **Given** a PDF in browser, **When** processed through WASM, **Then** PII is detected without server round-trip

---

### User Story 5 - Performance Optimization (Priority: P2)

A platform processing millions of documents daily needs optimized detection performance to reduce infrastructure costs.

**Why this priority**: Performance directly impacts operational costs. Parallelization can provide 2-4x throughput improvement.

**Acceptance Scenarios**:

1. **Given** a document with multiple segments, **When** detection runs, **Then** segments are processed in parallel
2. **Given** a large JSON file, **When** parsed, **Then** streaming parsing avoids loading entire file into memory
3. **Given** benchmark suite, **When** run after optimization, **Then** throughput improves by at least 50%

---

### User Story 6 - Developer Experience & Maintainability (Priority: P3)

A contributor wanting to improve Veil needs clear guidelines and automated tooling.

**Why this priority**: Good contributor experience accelerates community growth.

**Acceptance Scenarios**:

1. **Given** a new contributor, **When** they read CONTRIBUTING.md, **Then** they understand the development setup and PR process
2. **Given** any commit, **When** pre-commit hooks run, **Then** formatting and linting are verified
3. **Given** a dependency update available, **When** dependabot runs, **Then** a PR is automatically created
4. **Given** a developer, **When** they need to run common tasks, **Then** justfile provides standardized commands

---

### User Story 7 - Lean Build (Priority: P3)

A deployer with size constraints needs a minimal build without heavy optional dependencies.

**Why this priority**: Smaller binaries reduce deployment costs and attack surface.

**Acceptance Scenarios**:

1. **Given** a minimal deployment, **When** building with default features, **Then** heavy optional dependencies are excluded

---

### User Story 8 - Code Quality Excellence (Priority: P3)

A developer maintaining the codebase needs readable code with no magic numbers.

**Why this priority**: Code quality reduces debugging time and improves maintainability.

**Acceptance Scenarios**:

1. **Given** any numeric constant in code, **When** reviewing, **Then** it is a named constant with documentation
2. **Given** detection hot paths, **When** profiled, **Then** optimized byte scanning is used

---

### Edge Cases

- Key rotation during active encryption/decryption operations
- Batch processing with partial failures
- Large PDF files (>100MB) in WASM environments
- Feature flag combinations that create incompatible builds
- Dependabot PRs with breaking changes

## Requirements *(mandatory)*

### Functional Requirements

#### Security Hardening

- **FR-001**: System MUST encrypt FileVault contents at rest using authenticated encryption
- **FR-002**: System MUST support key rotation without data loss or downtime
- **FR-003**: System MUST provide a KeyProvider abstraction for external key management
- **FR-004**: System MUST include fuzz testing targets for all parsers
- **FR-005**: CI pipeline MUST run cargo-audit for dependency vulnerabilities
- **FR-006**: System MUST generate SBOM for each release

#### Test Coverage

- **FR-007**: veil-policy MUST have integration tests for YAML parsing and rule application
- **FR-008**: veil-core MUST have unit tests for SensitiveString behavior
- **FR-009**: veil-wasm MUST have browser-based tests in headless Chrome/Firefox
- **FR-010**: veil-redact MUST have tests for all mask styles
- **FR-011**: veil-discovery MUST have tests for symlink and archive handling
- **FR-012**: CI MUST run mutation testing targeting 80% mutation score
- **FR-013**: CI MUST generate coverage reports targeting 90% line coverage
- **FR-014**: All validators MUST have property-based tests using proptest

#### Documentation

- **FR-015**: All public items MUST have rustdoc comments with examples
- **FR-016**: Codebase MUST include ARCHITECTURE.md with Mermaid diagrams
- **FR-017**: Repository MUST include examples/ directory
- **FR-018**: Repository MUST include CHANGELOG.md (keepachangelog format)
- **FR-019**: veil-api MUST provide OpenAPI specification
- **FR-020**: veil-wasm MUST generate TypeScript type definitions

#### API Extensions

- **FR-021**: veil-api MUST provide POST /api/v1/batch endpoint
- **FR-022**: veil-wasm MUST support PDF text extraction
- **FR-023**: REST API MUST support ETag headers

#### Performance

- **FR-024**: Detection engine MUST parallelize segment processing
- **FR-025**: JSON parser MUST support streaming for large files
- **FR-026**: HTML parser MUST support streaming for large documents
- **FR-027**: Regex compilation MUST be cached with LRU eviction
- **FR-028**: System MUST include criterion benchmarks
- **FR-029**: Memory usage MUST be bounded and configurable

#### Maintainability

- **FR-030**: Repository MUST include CONTRIBUTING.md
- **FR-031**: Repository MUST include GitHub issue templates
- **FR-032**: Repository MUST include PR template with checklist
- **FR-033**: CI MUST automate changelog generation and releases
- **FR-034**: Repository MUST include pre-commit hooks
- **FR-035**: All crates MUST compile with #![deny(missing_docs)]
- **FR-036**: Repository MUST include dependabot.yml for automated dependency updates
- **FR-037**: Repository MUST include justfile or Makefile for common development tasks

#### Architecture

- **FR-038**: Heavy optional dependencies MUST be gated behind feature flags

#### Code Quality

- **FR-039**: All magic numbers MUST be extracted to named constants with documentation
- **FR-040**: Detection hot paths MUST use optimized byte scanning for performance

### Key Entities

- **EncryptedVault**: Vault storage with envelope encryption and key versioning
- **KeyProvider**: Abstraction for key retrieval (local, env, external KMS)
- **StreamingParser**: Parser with incremental processing and bounded memory
- **BenchmarkSuite**: Collection of criterion benchmarks

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All vault data encrypted at rest; no plaintext on disk
- **SC-002**: Key rotation < 60s for 10,000 tokens, zero data loss
- **SC-003**: 1M fuzz iterations per parser, zero crashes
- **SC-004**: Test coverage > 90% line coverage
- **SC-005**: Mutation score > 80%
- **SC-006**: New developer integration < 30 minutes using docs only
- **SC-007**: All public API items have rustdoc documentation
- **SC-008**: Batch processes 100 files without timeout
- **SC-009**: WASM PDF processing < 5s for 10MB documents
- **SC-010**: Parallel detection improves throughput by 50%+ on 4-core
- **SC-011**: Memory < 200MB for 100MB file processing
- **SC-012**: New contributor can submit valid PR < 2 hours
- **SC-013**: Zero cargo-audit warnings for dependencies
- **SC-014**: Dependabot PRs created within 24h of dependency updates
- **SC-015**: Default build binary size reduced by 30%+ via feature flags
- **SC-016**: Zero magic numbers in production code
- **SC-017**: Detection hot paths show measurable speedup from optimized byte scanning
