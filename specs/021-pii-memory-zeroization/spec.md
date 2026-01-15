# Feature Specification: PII Memory Zeroization

**Feature Branch**: `021-pii-memory-zeroization`
**Created**: 2025-12-17
**Status**: Draft
**Input**: User description: "Implement memory zeroization for PII data structures. Currently only encryption keys are zeroized on drop. Finding.matched_text, parsed document content, and redaction buffers remain in memory until garbage collected. For enterprise security, sensitive data must be securely erased from memory when no longer needed."

## Problem Statement

Sensitive PII data currently persists in memory after use:

- `Finding.matched_text` - Contains actual detected PII values
- Parsed document content - Raw text buffers containing PII
- Redaction working buffers - Temporary strings during redaction process
- API response bodies - Serialized findings before transmission

This creates security risks:
- Memory dumps can expose PII
- Core dumps include sensitive data
- Swap space may contain PII
- Memory forensics can recover "deleted" data
- Debugging tools can view sensitive strings

Enterprise security standards (SOC2, HIPAA, PCI-DSS) require secure erasure of sensitive data.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Automatic PII Cleanup on Scan Completion (Priority: P1)

As a security architect, I need detected PII values to be securely erased from memory when scan operations complete, so that memory dumps or debugging cannot expose customer data.

**Why this priority**: This is the core security requirement - PII found during scanning is the primary sensitive data that needs protection.

**Independent Test**: Can be tested by running a scan, then examining process memory to verify PII strings are not present after operation completes.

**Acceptance Scenarios**:

1. **Given** a scan operation detects PII in a document, **When** the scan completes and results are returned, **Then** the matched PII text is zeroed in memory
2. **Given** a Finding object goes out of scope, **When** the object is deallocated, **Then** the matched_text field is overwritten with zeros before memory is released
3. **Given** multiple findings from a batch scan, **When** processing completes, **Then** all PII values are securely erased from memory

---

### User Story 2 - Document Buffer Cleanup (Priority: P1)

As a compliance officer, I need parsed document content to be securely erased after processing, so that sensitive document text doesn't persist in application memory.

**Why this priority**: Raw document content may contain extensive PII beyond what's detected; all content should be treated as potentially sensitive.

**Independent Test**: Can be tested by parsing a document, completing processing, then verifying document content is not in memory.

**Acceptance Scenarios**:

1. **Given** a document is parsed into text segments, **When** processing completes, **Then** the segment content buffers are securely zeroed
2. **Given** a large file is streamed through the system, **When** each chunk is processed, **Then** the chunk buffer is zeroed before the next chunk
3. **Given** document parsing fails with an error, **When** the error is returned, **Then** any partially parsed content is still securely erased

---

### User Story 3 - Redaction Buffer Cleanup (Priority: P2)

As a developer integrating the redaction API, I need working buffers used during redaction to be securely erased, so that intermediate states containing PII don't persist.

**Why this priority**: Redaction operations create intermediate copies of text; these temporary buffers need protection.

**Independent Test**: Can be tested by performing redaction and verifying intermediate buffers are zeroed.

**Acceptance Scenarios**:

1. **Given** text is being redacted, **When** redaction completes, **Then** any intermediate string buffers containing original text are zeroed
2. **Given** redaction fails partway through, **When** the error is handled, **Then** partial buffers are still securely erased
3. **Given** the original text before redaction, **When** redaction completes successfully, **Then** only the redacted output remains in memory

---

### User Story 4 - API Response Cleanup (Priority: P2)

As an API consumer, I need the API server to securely erase response bodies containing PII after transmission, so that server memory doesn't accumulate sensitive data.

**Why this priority**: API responses may contain PII (if explicitly requested); these should be cleaned up after transmission.

**Independent Test**: Can be tested by making API requests and verifying response data is zeroed after transmission.

**Acceptance Scenarios**:

1. **Given** an API response containing findings is prepared, **When** the response is transmitted to the client, **Then** the response body is zeroed in server memory
2. **Given** multiple concurrent API requests, **When** each request completes, **Then** each response is independently zeroed

---

### Edge Cases

- What happens if the process crashes during cleanup? Partial cleanup is acceptable; complete crash means OS reclaims memory anyway.
- How to handle memory that's been swapped to disk? Swap encryption is an OS-level concern outside application scope; application should still zero its own memory.
- What about stack-allocated strings vs heap-allocated? Both should be zeroed where possible; stack strings may be overwritten by subsequent operations.
- How does this affect performance? Zeroization should add less than 5% overhead; use optimized memory zeroing where available.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST securely zero `matched_text` field in Finding struct when Finding is dropped
- **FR-002**: System MUST securely zero TextSegment content when segments are dropped
- **FR-003**: System MUST securely zero intermediate buffers during redaction operations
- **FR-004**: System MUST securely zero API response bodies after successful transmission
- **FR-005**: System MUST securely zero parsed document content when ParseResult is dropped
- **FR-006**: System MUST use memory barriers to prevent compiler optimization from removing zeroing operations
- **FR-007**: System MUST zero memory even when operations fail or panic (cleanup in Drop implementations)
- **FR-008**: System MUST zero memory before deallocation, not after
- **FR-009**: System SHOULD provide a "sensitive string" type that automatically zeros on drop
- **FR-010**: System MUST ensure zeroization works correctly on all supported platforms (Linux, macOS, Windows, WASM)

### Key Entities

- **SensitiveString**: A string wrapper that securely zeros its contents when dropped
- **Finding**: Detection result that now uses SensitiveString for matched_text
- **TextSegment**: Parsed text unit that now uses SensitiveString for content
- **RedactionBuffer**: Working buffer during redaction that zeros on completion

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of Finding.matched_text values are zeroed when Findings are dropped
- **SC-002**: 100% of parsed document content is zeroed when processing completes
- **SC-003**: Memory scan of process after scan operation finds 0 instances of original PII strings
- **SC-004**: Performance overhead of zeroization is less than 5% on typical workloads
- **SC-005**: All zeroization operations complete even when errors occur (verified by test coverage)
- **SC-006**: System passes security audit for memory handling of sensitive data

## Assumptions

- Zeroization prevents recovery from application memory; OS-level swap encryption is separate concern
- WASM environment may have different memory semantics; best-effort zeroization is acceptable
- Performance impact of zeroization is acceptable for enterprise security requirements
- Compiler optimizations will not remove zeroing operations when proper barriers are used

## Out of Scope

- OS-level memory encryption (swap, hibernation)
- Hardware-level secure memory enclaves (SGX, TrustZone)
- Preventing memory access by malicious code running in same process
- Zeroing memory in third-party libraries that may hold PII copies
