# Implementation Plan: Data Discovery & Inventory

**Branch**: `016-data-discovery` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/016-data-discovery/spec.md`

## Summary

Build a data discovery and inventory system that automatically scans databases (PostgreSQL, MySQL) and cloud storage (AWS S3, Azure Blob) to identify PII across an organization's data landscape. The system samples data, classifies PII categories, generates comprehensive data maps, supports scheduled scans with delta reporting, and optionally tags discovered PII back to source systems. This enables GDPR Article 30 compliance and provides visibility into where sensitive data resides.

## Technical Context

**Language/Version**: Rust (stable, 1.75+)
**Primary Dependencies**:
  - Database: sqlx (async SQL with connection pooling), tokio-postgres, mysql_async
  - Cloud: aws-sdk-s3, azure_storage_blobs
  - Async: tokio (async runtime for I/O-bound operations)
  - Core: veil-detect (PII detection), veil-parsers (file parsing)
  - Scheduling: cron (cron expression parsing)
  - Other: serde, serde_json, thiserror, regex
**Storage**: Local filesystem (JSONL for scan results/data maps)
**Testing**: cargo test, integration tests with testcontainers for real databases
**Target Platform**: Cross-platform CLI/library (Linux, macOS, Windows)
**Project Type**: New crate (veil-discover) in workspace
**Performance Goals**:
  - 10,000 S3 objects scanned in <30 minutes
  - 1M row table sampled and classified in <10 seconds
  - Data map generation for 100+ sources in <5 seconds
**Constraints**:
  - Statistical sampling for large datasets (configurable sample size)
  - Respect cloud API rate limits with adaptive throttling
  - Read-only by default; write operations (tagging) require explicit flag
**Scale/Scope**: Enterprise-scale discovery (100+ data sources, millions of files/rows)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ⚠️ REVIEW | Handles credentials (DB passwords, cloud keys); must use secure storage |
| II. Stability & Error Handling | ✅ PASS | Result types for all operations; graceful handling of connection failures |
| III. Performance | ✅ PASS | Async I/O for database/cloud operations; sampling for large datasets |
| IV. Simplicity & Minimalism | ⚠️ REVIEW | Complex feature with multiple data sources; must justify each component |
| V. Test-First Development | ✅ PASS | Use testcontainers for database tests; mock cloud APIs |
| VI. Dependency Discipline | ⚠️ REVIEW | Multiple heavy dependencies (sqlx, AWS SDK, Azure SDK) - must justify |
| VII. Rust Standards | ✅ PASS | Async Rust patterns; clippy/fmt; documented API |

**Gate Result**: CONDITIONAL PASS (dependencies justified for enterprise data source integration; security review required for credential handling)

## Project Structure

### Documentation (this feature)

```text
specs/016-data-discovery/
├── plan.md              # This file
├── research.md          # Phase 0 output (data source APIs, sampling strategies)
├── data-model.md        # Phase 1 output (DataSource, DiscoveryResult, DataMap)
├── quickstart.md        # Phase 1 output (usage examples)
├── contracts/           # Phase 1 output (trait definitions)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
Cargo.toml               # Workspace root (add veil-discover)
crates/
└── veil-discover/
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs           # Public API exports
    │   ├── error.rs         # DiscoveryError (thiserror)
    │   ├── types.rs         # DataSource, DiscoveryResult, ScanOptions
    │   ├── config.rs        # Configuration, credentials
    │   ├── sampling.rs      # Sampling strategies (random, stratified)
    │   ├── sources/         # Data source connectors
    │   │   ├── mod.rs       # DataSourceConnector trait
    │   │   ├── postgres.rs  # PostgreSQL connector
    │   │   ├── mysql.rs     # MySQL connector
    │   │   ├── s3.rs        # AWS S3 connector
    │   │   └── azure.rs     # Azure Blob connector
    │   ├── scanner.rs       # Main scanner orchestration
    │   ├── classifier.rs    # Schema-based and content-based classification
    │   ├── datamap.rs       # Data map generation and aggregation
    │   ├── delta.rs         # Delta calculation between scans
    │   ├── tagging.rs       # Write classification tags back to sources
    │   └── schedule.rs      # Scheduled discovery with cron
    └── tests/
        ├── fixtures/        # Test data (SQL scripts, sample files)
        ├── postgres_test.rs
        ├── mysql_test.rs
        ├── s3_test.rs       # Mock S3 tests
        ├── azure_test.rs    # Mock Azure tests
        ├── scanner_test.rs
        ├── datamap_test.rs
        ├── delta_test.rs
        └── integration_test.rs
```

**Structure Decision**: Single crate for all discovery functionality. Data source connectors are modular (feature flags for optional cloud providers). Async is required due to I/O-bound nature of database and cloud operations. Core detection logic reuses existing veil-detect crate.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| sqlx crate | Production-grade async SQL with connection pooling | Synchronous postgres/mysql crates would block during scans of many sources |
| AWS SDK | Official SDK for S3 access with proper auth/retry | Hand-rolled S3 API client would be error-prone and incomplete |
| Azure SDK | Official SDK for Blob Storage | Same as AWS; official SDKs handle auth complexity |
| tokio async runtime | Required for async database and cloud I/O | Synchronous approach would serialize scans, making bulk discovery impractical |
| Sampling complexity | Large datasets require statistical sampling | Full table scans would be prohibitively slow and resource-intensive |

## Phase 0: Research

**Input**: Feature spec + existing codebase
**Output**: research.md
**Status**: 🔲 TODO

### Research Questions

1. **Database Connectivity**:
   - How does sqlx handle connection pooling for multiple data sources?
   - What's the best approach for schema introspection in PostgreSQL/MySQL?
   - How to handle different authentication methods (password, IAM, certificates)?

2. **Cloud Storage APIs**:
   - How to efficiently list and sample large S3 buckets (pagination, prefixes)?
   - What's the Azure Blob Storage equivalent API for listing/sampling?
   - How to handle rate limiting and retry strategies?

3. **Sampling Strategies**:
   - What sample size provides 95% confidence for PII detection?
   - Random sampling vs. stratified sampling for databases?
   - How to sample files efficiently (first N bytes, full download)?

4. **Data Map Structure**:
   - How to represent hierarchical data (Source → Database/Bucket → Table/Folder → Column/File)?
   - What JSON schema for data map export?
   - How to visualize in HTML (static HTML + JavaScript chart library)?

5. **Delta Calculation**:
   - What constitutes a "change" (new column, type change, PII category change)?
   - How to persist baseline for comparison?
   - Storage format for scan history?

6. **Security**:
   - How to securely store credentials (environment variables, AWS Secrets Manager, Azure Key Vault)?
   - How to avoid logging sensitive connection strings?
   - Audit trail for discovery operations?

### Research Deliverables

- Document sqlx connection pattern and schema introspection queries
- Document AWS SDK S3 pagination and sampling approach
- Document Azure SDK equivalent patterns
- Prototype sampling algorithm with confidence calculations
- Design data map JSON schema
- Design scan result storage format (JSONL)
- Security architecture for credential handling

## Phase 1: Design

**Input**: Research findings
**Output**: data-model.md + quickstart.md + contracts/
**Status**: 🔲 TODO

### Design Artifacts

1. **data-model.md**:
   - DataSource struct (type, connection config)
   - DiscoveryResult struct (containers, fields, PII classifications)
   - DataMap struct (hierarchy, aggregations)
   - ColumnClassification struct (column ref, PII type, confidence)
   - DeltaReport struct (new, modified, removed)
   - ScanOptions struct (sample size, sampling strategy, filters)

2. **quickstart.md**:
   - Example: Scan PostgreSQL database
   - Example: Scan S3 bucket
   - Example: Generate data map
   - Example: Schedule weekly scans
   - Example: Tag columns with PII classifications

3. **contracts/**:
   - DataSourceConnector trait (connect, list_containers, sample_data, tag_pii)
   - Scanner interface
   - Classifier interface
   - Export format trait (JSON, CSV, HTML)

### Key Design Decisions

- **Async Throughout**: All I/O operations are async to enable concurrent scanning
- **Feature Flags**: Cloud providers are optional features (reduce dependencies)
- **Credential Strategy**: Environment variables by default; extensible for secret managers
- **Sampling**: Random sampling with configurable size; stratified sampling deferred to future
- **Data Map Storage**: JSONL for scan results; JSON for final data map
- **Delta Storage**: Store previous scan result hash; full comparison on demand
- **Tagging**: Separate operation; requires explicit flag; uses transactions where possible

## Phase 2: Implementation Tasks

**Input**: Design documents
**Output**: tasks.md (via /speckit.tasks)
**Workflow**: Test-first (Red-Green-Refactor)

### High-Level Task Breakdown

#### 2.1: Foundation
- Error types (DiscoveryError)
- Core data structures (DataSource, DiscoveryResult, ScanOptions)
- Configuration and credential loading
- DataSourceConnector trait definition

#### 2.2: Database Connectors
- PostgreSQL connector (schema introspection, sampling)
- MySQL connector (schema introspection, sampling)
- Schema-based classification (column names, types)
- Content-based classification (sample data)

#### 2.3: Cloud Storage Connectors
- S3 connector (list objects, download samples)
- Azure Blob connector (list blobs, download samples)
- File-based classification (parse via veil-parsers, detect via veil-detect)

#### 2.4: Scanner Orchestration
- Scanner struct (manages multiple data sources)
- Concurrent scanning with connection pooling
- Progress tracking and error recovery
- Result aggregation

#### 2.5: Data Map Generation
- Hierarchical data map construction
- Aggregation (count by PII category, by source)
- JSON export
- HTML export with visualization

#### 2.6: Delta Calculation
- Baseline storage and loading
- Change detection (new, modified, removed)
- Delta report generation
- Delta export formats

#### 2.7: Scheduled Discovery
- Cron expression parsing
- Scheduling daemon (or integration point for external schedulers)
- Delta reporting on schedule

#### 2.8: Classification Tagging
- Write column comments (PostgreSQL COMMENT ON COLUMN)
- Write column comments (MySQL COMMENT syntax)
- Write S3 object tags
- Write Azure Blob metadata tags
- Transaction handling and rollback

#### 2.9: Integration Tests
- End-to-end: Scan database + S3, generate map
- End-to-end: Schedule scan, compute delta
- End-to-end: Tag PII back to sources
- Performance tests (large datasets)

#### 2.10: CLI Integration
- Add `veil discover` commands to veil-cli
- Progress bars for scanning operations
- Output formatting options

## Testing Strategy

### Unit Tests

**Coverage Target**: >85% for new code

**Test Categories**:
- Data source connector methods
- Sampling algorithms
- Classification logic (schema and content)
- Data map aggregation
- Delta calculation
- Credential parsing and validation

### Integration Tests

**Real Infrastructure** (via testcontainers):
- PostgreSQL container with test schema
- MySQL container with test schema

**Mocked Infrastructure**:
- Mock S3 API (using aws-sdk-s3 test utilities)
- Mock Azure API (using azure_core test utilities)

**Scenarios**:
- Full discovery workflow
- Multi-source scanning
- Delta reporting after data changes
- Tagging and verification
- Error handling (connection failures, rate limiting)

### Performance Tests

**Benchmarks**:
- Sample 1M row table in <10s
- Scan 10,000 S3 objects in <30 minutes
- Generate data map for 100+ sources in <5s

**Load Testing**:
- Concurrent connections to multiple databases
- S3 pagination with 100,000+ objects
- Memory usage during large scans

## Dependencies

### New Dependencies to Add

Add to `Cargo.toml` (workspace dependencies):

```toml
[workspace.dependencies]
# Database
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "mysql", "macros"] }

# Cloud (optional features)
aws-config = "1.1"
aws-sdk-s3 = "1.13"
azure_storage = "0.19"
azure_storage_blobs = "0.19"

# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Scheduling
cron = "0.12"

# Testing
testcontainers = "0.15"
```

Add to `crates/veil-discover/Cargo.toml`:

```toml
[dependencies]
veil-detect = { path = "../veil-detect" }
veil-parsers = { path = "../veil-parsers" }
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tokio.workspace = true
sqlx.workspace = true
regex.workspace = true
cron.workspace = true

# Optional cloud features
aws-config = { workspace = true, optional = true }
aws-sdk-s3 = { workspace = true, optional = true }
azure_storage = { workspace = true, optional = true }
azure_storage_blobs = { workspace = true, optional = true }

[dev-dependencies]
testcontainers.workspace = true
pretty_assertions.workspace = true
tempfile.workspace = true

[features]
default = ["postgres", "mysql"]
postgres = ["sqlx/postgres"]
mysql = ["sqlx/mysql"]
aws = ["aws-config", "aws-sdk-s3"]
azure = ["azure_storage", "azure_storage_blobs"]
cloud = ["aws", "azure"]
```

**Dependency Justification**:
- **sqlx**: Industry-standard async SQL with compile-time query checking
- **aws-sdk-s3**: Official AWS SDK with proper auth, retry, and rate limiting
- **azure_storage_blobs**: Official Azure SDK for Blob Storage
- **tokio**: Required for async runtime; standard in Rust ecosystem
- **cron**: Standard cron expression parsing
- **testcontainers**: Enables real database testing without manual setup

## Success Metrics

### Functional Completeness

- 🔲 FR-001: PostgreSQL connection and table scanning
- 🔲 FR-002: MySQL connection and table scanning
- 🔲 FR-003: AWS S3 object scanning
- 🔲 FR-004: Azure Blob Storage scanning
- 🔲 FR-005: Configurable data sampling
- 🔲 FR-006: Schema-based PII prediction
- 🔲 FR-007: Data map generation (JSON, HTML)
- 🔲 FR-008: Scheduled discovery with cron
- 🔲 FR-009: Delta reporting between scans
- 🔲 FR-010: Classification tagging to sources
- 🔲 FR-011: Discovery statistics reporting
- 🔲 FR-012: Secure credential handling

### Performance Targets

- 🔲 SC-001: Schema analysis 95% accuracy
- 🔲 SC-002: Data sampling 90% accuracy with 1% sample
- 🔲 SC-003: 10,000 S3 objects in <30 minutes
- 🔲 SC-004: Data map for 100+ sources without performance issues
- 🔲 SC-005: Delta reporting within 24 hours of change
- 🔲 SC-006: Column tagging without corruption

## Risk Assessment

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Database connection pool exhaustion | Medium | High | Connection pooling with limits; sequential scanning if needed |
| S3 rate limiting (list operations) | Medium | Medium | Exponential backoff; adaptive throttling; respect retry-after headers |
| Memory usage with large data maps | Low | Medium | Stream aggregation if needed; defer optimization |
| Sampling bias in PII detection | Medium | Medium | Document sampling limitations; allow manual override |
| Credential exposure in logs | Low | Critical | Redact credentials in debug output; secure storage |
| Delta false positives | Low | Low | Test with schema changes; document change detection logic |

### Dependency Risks

| Dependency | Risk Level | Justification |
|------------|------------|---------------|
| sqlx | Low | Widely used, maintained by community, stable API |
| aws-sdk-s3 | Low | Official AWS SDK, well-maintained |
| azure_storage | Medium | Community-maintained, but official Azure SDK is evolving |
| tokio | Low | Standard async runtime, stable API |
| testcontainers | Low | Standard testing tool, isolated to dev dependencies |

### Security Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Credential leakage | Low | Critical | Environment variables only; no hardcoding; audit logs redact secrets |
| Unauthorized data access | Medium | Critical | Require explicit credentials; respect database permissions |
| PII exposure in scan results | Medium | High | Redact actual PII in reports; store only metadata |
| Tagging errors corrupting data | Low | Medium | Test tagging thoroughly; use transactions; require explicit flag |

## Deployment Considerations

### Breaking Changes

**None** - This is a new crate; no existing API to break.

### Configuration

Users must provide:
- Database connection strings (via environment variables or config file)
- Cloud credentials (via AWS/Azure environment variables or IAM roles)
- Scan options (sample size, filters, schedule)

### Example Configuration

```yaml
# config/discovery.yaml
sources:
  - type: postgres
    connection: "postgresql://user:pass@localhost/db"
    sample_size: 1000

  - type: s3
    bucket: "my-bucket"
    region: "us-east-1"
    sample_size: 100

options:
  output_dir: "./discovery-results"
  schedule: "0 0 * * 0"  # Weekly on Sunday
  delta_enabled: true
  tagging_enabled: false
```

### Migration Path

N/A - New feature.

### Rollback Plan

N/A - New feature; no rollback needed.

## Future Enhancements (Out of Scope)

These are explicitly deferred for future iterations:

1. **Additional Data Sources**: Snowflake, BigQuery, DynamoDB, MongoDB
2. **Stratified Sampling**: Sample proportionally across data distributions
3. **Machine Learning Classification**: Train models on discovered PII patterns
4. **Real-time Discovery**: Stream-based discovery for data lakes
5. **Data Lineage Tracking**: Track PII flow between systems
6. **Custom Classification Rules**: User-defined PII patterns
7. **Distributed Scanning**: Scale across multiple workers
8. **GraphQL API**: REST/GraphQL API for discovery results
9. **UI Dashboard**: Web UI for data map visualization
10. **Advanced Scheduling**: Dependencies between discovery jobs

## Acceptance Criteria

### Must Have (P1)

- ✅ All user stories in spec have tests
- ✅ PostgreSQL and MySQL database scanning (FR-001, FR-002)
- ✅ S3 and Azure Blob scanning (FR-003, FR-004)
- ✅ Data map generation (FR-007)
- ✅ All tests pass
- ✅ Clippy clean
- ✅ Documentation complete

### Should Have (P2)

- ✅ Scheduled discovery (FR-008)
- ✅ Delta reporting (FR-009)
- ✅ Classification tagging (FR-010)
- ✅ Performance targets met
- ✅ Integration with veil-cli

### Could Have (P3)

- Schema-based detection optimization
- Advanced HTML visualization
- Performance benchmarks with criterion
- Example discovery configurations

## Timeline Estimate

**Estimated Effort**: 20-30 hours

| Phase | Estimated Time |
|-------|----------------|
| 0 Research | 3-4 hours |
| 1 Design | 2-3 hours |
| 2.1 Foundation | 2 hours |
| 2.2 Database Connectors | 4-5 hours |
| 2.3 Cloud Storage Connectors | 4-5 hours |
| 2.4 Scanner Orchestration | 2 hours |
| 2.5 Data Map Generation | 2-3 hours |
| 2.6 Delta Calculation | 2 hours |
| 2.7 Scheduled Discovery | 1-2 hours |
| 2.8 Classification Tagging | 2-3 hours |
| 2.9 Integration Tests | 2-3 hours |
| 2.10 CLI Integration | 2 hours |
| Documentation | 1 hour |

## Sign-off

**Stakeholder**: Development Team
**Status**: Ready for Research Phase
**Next Step**: Begin Phase 0 research to answer research questions and create research.md

## Post-Design Constitution Re-Check

*Re-evaluate after Phase 1 design completion*

| Principle | Status | Post-Design Notes |
|-----------|--------|-------------------|
| I. Security First | TBD | Credentials handling approach to be validated |
| II. Stability & Error Handling | TBD | Error handling patterns for network/database failures |
| III. Performance | TBD | Async patterns and sampling strategies validated |
| IV. Simplicity & Minimalism | TBD | Architecture complexity justified by requirements |
| V. Test-First Development | TBD | Test strategy with testcontainers validated |
| VI. Dependency Discipline | TBD | All dependencies justified and reviewed |
| VII. Rust Standards | TBD | Async patterns follow Rust best practices |

**Post-Design Gate Result**: TBD - To be completed after Phase 1 design
