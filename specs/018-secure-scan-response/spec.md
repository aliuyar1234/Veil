# Feature Specification: Secure Scan Response (PII-Safe API)

**Feature Branch**: `018-secure-scan-response`
**Created**: 2025-12-17
**Status**: Draft
**Input**: User description: "Remove PII values from scan API responses - return only positions, categories, and confidence scores. Add optional include_values parameter that defaults to false and requires explicit opt-in with security acknowledgment. This fixes the critical data exposure vulnerability where raw PII is returned in scan results."

## Problem Statement

The current scan API, CLI, and WASM interfaces return raw PII values (matched_text) in their responses. This creates a critical data exposure vulnerability:

- Network logs capture actual PII values
- Client-side logs and debugging tools expose PII
- Audit trails may inadvertently store sensitive data
- Violates data minimization principles (GDPR Article 5)
- Incompatible with enterprise 0% data leak requirements

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Secure Scan Without PII Exposure (Priority: P1)

As a security engineer at an enterprise organization, I need to scan documents for PII without the scan results containing the actual PII values, so that my logging infrastructure, network monitoring tools, and debugging output never capture sensitive data.

**Why this priority**: This is the core security fix that addresses the critical vulnerability. Without this, the system cannot be used in any compliance-sensitive environment.

**Independent Test**: Can be fully tested by calling the scan API and verifying the response contains positions and categories but no actual PII values.

**Acceptance Scenarios**:

1. **Given** a document containing email addresses, **When** I call the scan endpoint without include_values, **Then** the response contains category "email", start/end positions, confidence score, but the "value" field is absent or null
2. **Given** a document containing multiple PII types (email, phone, IBAN), **When** I scan it with default settings, **Then** no response field contains any of the original PII strings
3. **Given** scan results are logged by the API server, **When** I review the server logs, **Then** no PII values appear in any log entry

---

### User Story 2 - Explicit Opt-In for PII Values (Priority: P2)

As a developer building a PII redaction preview UI, I need to optionally retrieve the actual PII values in scan results, but only after explicitly acknowledging the security implications.

**Why this priority**: Some legitimate use cases (redaction preview, debugging during development) require seeing the actual values. This must be opt-in with clear security acknowledgment.

**Independent Test**: Can be tested by calling the scan API with include_values=true and X-Acknowledge-PII-Exposure header, verifying values are returned.

**Acceptance Scenarios**:

1. **Given** I want to see actual PII values, **When** I call scan with include_values=true but without the acknowledgment header, **Then** the request is rejected with a clear error explaining the security requirement
2. **Given** I provide both include_values=true and X-Acknowledge-PII-Exposure: accepted header, **When** I scan a document, **Then** the response includes the matched PII values
3. **Given** I am using the CLI with --include-values flag, **When** I run the scan command, **Then** I am prompted to confirm I understand the security implications before values are shown

---

### User Story 3 - CLI Safe Output Mode (Priority: P1)

As a DevOps engineer running PII scans in CI/CD pipelines, I need the CLI to never output actual PII values by default, so that build logs and terminal history remain clean of sensitive data.

**Why this priority**: CLI output is frequently captured in logs, terminal scrollback, and CI/CD artifacts. This is equally critical as API security.

**Independent Test**: Can be tested by running the CLI scan command and verifying no PII values appear in stdout/stderr.

**Acceptance Scenarios**:

1. **Given** I run `veil scan document.txt`, **When** the scan completes, **Then** the output shows categories, positions, and confidence but not the actual matched text
2. **Given** I run `veil scan document.txt --json`, **When** I parse the JSON output, **Then** no field contains the original PII values
3. **Given** I explicitly run `veil scan document.txt --include-values`, **When** prompted for confirmation, **Then** I must type "yes" to proceed, and only then are values shown

---

### User Story 4 - WASM Secure Response (Priority: P2)

As a frontend developer using the WASM library in a browser application, I need scan results to exclude PII values by default, so that browser developer tools, console logs, and network inspector don't capture sensitive data.

**Why this priority**: Browser environments have extensive debugging capabilities that could expose PII. This aligns with the API and CLI behavior.

**Independent Test**: Can be tested by calling the WASM scan function and inspecting the JavaScript response object.

**Acceptance Scenarios**:

1. **Given** I call the scan function from JavaScript, **When** I inspect the returned findings array, **Then** each finding contains category, start, end, confidence but no value/matchedText property
2. **Given** I need values for a redaction preview, **When** I call scan with {includeValues: true, acknowledgeExposure: true}, **Then** values are included in the response

---

### Edge Cases

- What happens when a user programmatically sets include_values without the acknowledgment? Request must be rejected with HTTP 400 or equivalent error.
- How does system handle existing integrations that expect values in responses? Breaking change must be documented; migration guide provided.
- What if the acknowledgment header is present but has wrong value? Only exact value "accepted" is valid; other values treated as missing.
- How are audit logs affected? Audit entries must never contain raw PII values, only summaries (count by category).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST NOT include PII values (matched_text) in scan responses by default
- **FR-002**: System MUST include an optional parameter (include_values/includeValues) to enable PII value inclusion
- **FR-003**: System MUST require an explicit security acknowledgment when include_values is enabled
- **FR-004**: API MUST require header "X-Acknowledge-PII-Exposure: accepted" when include_values=true
- **FR-005**: CLI MUST require interactive confirmation when --include-values flag is used
- **FR-006**: WASM MUST require acknowledgeExposure: true option when includeValues is true
- **FR-007**: System MUST return HTTP 400 (Bad Request) when include_values is true without proper acknowledgment
- **FR-008**: Scan responses MUST always include: category, start position, end position, confidence score, segment index
- **FR-009**: Scan responses MUST optionally include: matched value (only when explicitly enabled with acknowledgment)
- **FR-010**: CLI output MUST redact or omit PII values in all output modes (text, JSON, quiet) by default
- **FR-011**: Error messages MUST NOT contain PII values under any circumstances
- **FR-012**: Server logs MUST NOT contain PII values from scan operations
- **FR-013**: System MUST provide a migration guide for existing integrations expecting values in responses

### Key Entities

- **ScanFinding**: Represents a detected PII instance - contains category, position (start/end), confidence, segment reference, and optionally the matched value
- **ScanOptions**: Request parameters including include_values flag and format preferences
- **SecurityAcknowledgment**: Marker indicating user has accepted responsibility for PII exposure (header, flag, or option)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of scan API responses exclude PII values when include_values is not specified or false
- **SC-002**: 100% of CLI scan outputs exclude PII values when --include-values flag is not provided
- **SC-003**: 0% of server-side logs contain raw PII values from scan operations
- **SC-004**: Users attempting to enable include_values without acknowledgment receive clear error within 1 second
- **SC-005**: Existing functionality (detection accuracy, performance) remains unchanged - less than 5% performance impact
- **SC-006**: System achieves enterprise compliance readiness for GDPR data minimization requirements

## Assumptions

- Breaking change to API response format is acceptable with proper versioning/documentation
- Interactive CLI confirmation is acceptable for the --include-values use case (non-interactive mode will reject)
- The acknowledgment mechanism is a security awareness measure, not a legal contract
- Performance impact of removing fields from responses will be negligible (simpler responses, less data)

## Out of Scope

- Changes to the internal Finding struct or detection logic
- Encryption of PII values as an alternative to omission
- Audit log encryption (separate feature)
- User authentication/authorization for the include_values feature (acknowledgment is sufficient)
