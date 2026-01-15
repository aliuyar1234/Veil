# Feature Specification: Data Discovery & Inventory

**Feature Branch**: `016-data-discovery`
**Created**: 2025-12-15
**Status**: Draft
**Input**: Automated discovery of PII across data stores and systems

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scan Database Tables (Priority: P1)

A data protection officer needs to find PII in a PostgreSQL/MySQL database. The system connects to the database, samples data from tables, and identifies columns containing PII.

**Why this priority**: Databases are primary stores of structured PII; discovery is essential for GDPR Article 30.

**Independent Test**: Connect to test database with known PII columns, verify columns correctly identified.

**Acceptance Scenarios**:

1. **Given** PostgreSQL connection string, **When** discovery run, **Then** all tables scanned.
2. **Given** column named "email" with email data, **When** scanned, **Then** column flagged as Email PII.
3. **Given** column with mixed data, **When** sampled, **Then** PII presence reported with percentage.

---

### User Story 2 - Scan S3/Blob Storage (Priority: P1)

A cloud security team needs to find PII in cloud storage buckets. The system lists objects, downloads samples, and reports files containing PII.

**Why this priority**: Cloud storage accumulates unstructured data that often contains undiscovered PII.

**Independent Test**: Connect to S3 bucket with test files, verify PII files identified.

**Acceptance Scenarios**:

1. **Given** S3 bucket path, **When** discovery run, **Then** all supported file types scanned.
2. **Given** CSV file in bucket with emails, **When** scanned, **Then** file flagged with PII categories.
3. **Given** bucket with 10,000 objects, **When** scanned with sampling, **Then** representative subset analyzed.

---

### User Story 3 - Generate Data Map (Priority: P1)

A compliance team needs a visual map of where PII exists across systems. The system generates a data map showing PII categories by data source, table/folder, and column/file.

**Why this priority**: Data mapping is required for GDPR compliance and risk assessment.

**Independent Test**: Run discovery across database and S3, generate map, verify all sources included.

**Acceptance Scenarios**:

1. **Given** multiple data sources scanned, **When** map generated, **Then** shows hierarchy: Source → Container → Field.
2. **Given** JSON output format, **When** generated, **Then** machine-readable for integration.
3. **Given** HTML output format, **When** generated, **Then** interactive visualization of PII distribution.

---

### User Story 4 - Schema-Based Detection (Priority: P2)

A database administrator wants quick detection based on column names and types, without scanning actual data. The system analyzes schema metadata to predict likely PII columns.

**Why this priority**: Schema analysis is fast and non-intrusive; suitable for initial triage.

**Independent Test**: Provide schema with PII-named columns, verify flagged without data access.

**Acceptance Scenarios**:

1. **Given** column named "ssn" or "social_security", **When** schema analyzed, **Then** flagged as likely SSN.
2. **Given** column type VARCHAR(255) named "address", **When** analyzed, **Then** flagged as likely Address.
3. **Given** column named "created_at" with TIMESTAMP type, **When** analyzed, **Then** not flagged as PII.

---

### User Story 5 - Scheduled Discovery Scans (Priority: P2)

A privacy engineer wants weekly scans to detect new PII as data grows. The system supports scheduled discovery with delta reporting (new PII since last scan).

**Why this priority**: Data constantly changes; continuous discovery maintains accurate inventory.

**Independent Test**: Run two scans with data added between, verify delta report shows new PII.

**Acceptance Scenarios**:

1. **Given** cron schedule "0 0 * * 0", **When** configured, **Then** discovery runs weekly.
2. **Given** new table added since last scan, **When** delta computed, **Then** new table flagged as "New".
3. **Given** column PII type changed, **When** delta computed, **Then** change flagged with before/after.

---

### User Story 6 - Classification Tagging (Priority: P2)

A data governance team wants discovered PII to be tagged in the source system. The system writes classification metadata back to database comments or cloud object tags.

**Why this priority**: Tagging enables downstream systems to apply appropriate access controls.

**Independent Test**: Run discovery with tagging enabled, verify column comments updated.

**Acceptance Scenarios**:

1. **Given** `--tag-columns` flag, **When** PII found in column, **Then** column comment updated with PII type.
2. **Given** S3 object with PII, **When** `--tag-objects` enabled, **Then** object tags include `pii:email`.
3. **Given** read-only mode, **When** discovery run, **Then** no tags written, only reported.

---

### Edge Cases

- What happens with access denied to table/bucket? System logs error and continues with accessible resources.
- What happens with very large tables (billions of rows)? System uses statistical sampling with configurable sample size.
- What happens with encrypted columns? System reports column as "encrypted, not scanned".
- What happens with connection timeout? System retries with exponential backoff; fails after max retries.
- What happens with rate limiting (cloud APIs)? System respects rate limits with adaptive throttling.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST connect to PostgreSQL databases for table scanning.
- **FR-002**: System MUST connect to MySQL databases for table scanning.
- **FR-003**: System MUST connect to AWS S3 for object scanning.
- **FR-004**: System MUST support Azure Blob Storage for object scanning.
- **FR-005**: System MUST sample data when full scan is impractical (configurable sample size).
- **FR-006**: System MUST analyze database schema metadata for PII prediction.
- **FR-007**: System MUST generate data map in JSON and HTML formats.
- **FR-008**: System MUST support scheduled discovery with cron-like syntax.
- **FR-009**: System MUST compute delta between scans (new, changed, removed PII).
- **FR-010**: System MUST support writing classification tags back to source systems.
- **FR-011**: System MUST report discovery statistics: sources scanned, PII found, errors.
- **FR-012**: System MUST handle connection credentials securely (env vars, secret managers).

### Key Entities

- **DataSource**: A connection to a data store; contains type (postgres, mysql, s3), connection config.
- **DiscoveryResult**: Result of scanning a source; contains containers (tables/buckets), fields with PII.
- **DataMap**: Aggregated view of PII across sources; contains hierarchy and PII category distribution.
- **ColumnClassification**: PII classification for a database column; contains column ref, PII type, confidence.
- **DiscoveryScan**: A discovery job; contains sources, options, schedule, last run status.
- **DeltaReport**: Changes since last scan; contains new, modified, removed PII locations.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Database tables with clear PII column names detected with 95% accuracy (schema analysis).
- **SC-002**: Data sampling correctly identifies PII in 90% of cases with 1% sample size.
- **SC-003**: S3 bucket with 10,000 objects scanned in under 30 minutes.
- **SC-004**: Data map visualization renders for 100+ data sources without performance issues.
- **SC-005**: Delta reports correctly identify new PII within 24 hours of data change.
- **SC-006**: Column tagging writes valid comments without data corruption.

## Assumptions

- Database connections require appropriate credentials with read access (and comment write for tagging).
- Cloud storage access uses IAM roles or explicit credentials.
- Sampling is random per table/bucket; stratified sampling is future enhancement.
- Data map is point-in-time snapshot; real-time updates not supported.
- Only structured/semi-structured data sources supported initially; SharePoint, Confluence etc. are future.
