# Feature Specification: Regex Detection Engine

**Feature Branch**: `002-regex-detection`
**Created**: 2025-12-08
**Status**: Draft
**Input**: Pattern-based PII detection with validation (Email, IBAN, Phone, etc.)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Detect Email Addresses (Priority: P1)

A privacy analyst scans a document and the system identifies all email addresses. Each finding includes the matched text, its position in the document, a confidence score, and the PII category.

**Why this priority**: Email addresses are the most common PII type and serve as the foundation pattern for the detection engine architecture.

**Independent Test**: Provide text containing valid emails, invalid emails, and email-like strings. Verify correct detection with zero false negatives on valid emails.

**Acceptance Scenarios**:

1. **Given** text containing `user@example.com`, **When** scanned, **Then** the system detects it as PII type "email" with 100% confidence.
2. **Given** text containing `invalid@`, **When** scanned, **Then** the system does not report it as a valid email.
3. **Given** text with multiple emails, **When** scanned, **Then** all emails are detected with correct positions.

---

### User Story 2 - Detect IBAN Numbers (Priority: P1)

A compliance officer scans financial documents for bank account numbers. The system detects IBAN numbers and validates them using the checksum algorithm to reduce false positives.

**Why this priority**: IBANs are high-sensitivity financial data. Checksum validation demonstrates the pattern+validation architecture.

**Independent Test**: Provide valid IBANs (multiple countries), invalid IBANs (wrong checksum), and IBAN-like strings. Verify only valid IBANs are detected.

**Acceptance Scenarios**:

1. **Given** a valid German IBAN `DE89370400440532013000`, **When** scanned, **Then** detected as "iban" with checksum validation passed.
2. **Given** an IBAN with invalid checksum, **When** scanned, **Then** detected with lower confidence or flagged as potentially invalid.
3. **Given** Austrian, Swiss, and other country IBANs, **When** scanned, **Then** all are correctly detected and validated.

---

### User Story 3 - Detect Phone Numbers (Priority: P2)

A data protection officer scans documents for phone numbers in various formats (international, local, with/without country codes). The system normalizes and detects phone numbers for AT/DE/CH locales.

**Why this priority**: Phone numbers have many format variations. Supporting multiple locales demonstrates flexible pattern configuration.

**Independent Test**: Provide phone numbers in formats: +43 1 234567, 0043-1-234567, 01/234567, (01) 234 567. Verify all are detected.

**Acceptance Scenarios**:

1. **Given** `+43 664 1234567`, **When** scanned, **Then** detected as "phone" with country code AT.
2. **Given** `089/12345678` (German format), **When** scanned, **Then** detected as "phone" with locale hint DE.
3. **Given** a number too short to be valid, **When** scanned, **Then** not detected or flagged as low confidence.

---

### User Story 4 - Detect Credit Card Numbers (Priority: P2)

A security team scans logs and documents for accidentally exposed credit card numbers. The system detects card number patterns and validates using the Luhn algorithm.

**Why this priority**: Credit cards are highly sensitive. Luhn validation is essential to avoid false positives on random 16-digit numbers.

**Independent Test**: Provide valid card numbers (Visa, Mastercard), invalid numbers (wrong Luhn), and 16-digit non-card numbers.

**Acceptance Scenarios**:

1. **Given** a valid Visa number `4111111111111111`, **When** scanned, **Then** detected as "credit_card" with Luhn validation passed.
2. **Given** `1234567890123456` (invalid Luhn), **When** scanned, **Then** not detected or flagged as low confidence.
3. **Given** card numbers with spaces or dashes, **When** scanned, **Then** correctly detected after normalization.

---

### User Story 5 - Detect Austrian Social Security Numbers (Priority: P2)

An HR department scans employee documents for Austrian SVNr (Sozialversicherungsnummer). The system detects the 10-digit format and validates structure.

**Why this priority**: Country-specific ID numbers are core to GDPR compliance. SVNr demonstrates locale-specific patterns.

**Independent Test**: Provide valid SVNr formats, invalid formats, and 10-digit numbers that aren't SVNr.

**Acceptance Scenarios**:

1. **Given** a valid Austrian SVNr `1234 010190`, **When** scanned, **Then** detected as "svnr_at" with high confidence.
2. **Given** a 10-digit number not matching SVNr structure, **When** scanned, **Then** not detected as SVNr.

---

### User Story 6 - Detect IP and MAC Addresses (Priority: P3)

A security analyst scans logs for IP addresses (v4 and v6) and MAC addresses that could identify individuals or devices.

**Why this priority**: Network identifiers are PII under GDPR. These are well-defined patterns with clear validation rules.

**Independent Test**: Provide valid IPv4, IPv6, MAC addresses, and similar-looking non-addresses.

**Acceptance Scenarios**:

1. **Given** `192.168.1.1`, **When** scanned, **Then** detected as "ipv4".
2. **Given** `2001:0db8:85a3:0000:0000:8a2e:0370:7334`, **When** scanned, **Then** detected as "ipv6".
3. **Given** `00:1A:2B:3C:4D:5E`, **When** scanned, **Then** detected as "mac_address".

---

### Edge Cases

- What happens when PII patterns overlap (e.g., phone number inside longer number)? System reports all valid matches with positions, allows downstream deduplication.
- What happens with Unicode confusables (e.g., Cyrillic 'а' vs Latin 'a')? System normalizes to ASCII equivalents before pattern matching.
- What happens when patterns span line breaks? System handles multiline matching where appropriate.
- What happens with extremely long input? System processes in chunks without missing matches at boundaries.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST detect email addresses using RFC 5322 compliant pattern matching.
- **FR-002**: System MUST detect IBAN numbers for all SEPA countries and validate using MOD-97 checksum.
- **FR-003**: System MUST detect phone numbers in international and local formats for AT, DE, CH locales.
- **FR-004**: System MUST detect credit card numbers (Visa, Mastercard, Amex) and validate using Luhn algorithm.
- **FR-005**: System MUST detect Austrian SVNr and German Sozialversicherungsnummer with format validation.
- **FR-006**: System MUST detect IPv4, IPv6, and MAC addresses.
- **FR-007**: System MUST detect German/Austrian tax IDs (Steuernummer, UID) with format validation.
- **FR-008**: System MUST return for each finding: matched text, start position, end position, PII category, confidence score (0.0-1.0).
- **FR-009**: System MUST support custom regex patterns provided via configuration.
- **FR-010**: System MUST allow enabling/disabling specific detectors.
- **FR-011**: System MUST process text segments from the parser output format (TextSegment with position metadata).
- **FR-012**: System MUST preserve original position information through detection for accurate source location.

### Key Entities

- **Detector**: A named pattern matcher for a specific PII type; has a regex pattern, optional validator function, and confidence scoring logic.
- **Finding**: A detected PII instance; contains the matched text, position (start/end), PII category, confidence score, and validation status.
- **DetectorConfig**: Configuration for the detection engine; specifies which detectors are enabled, locale settings, and custom patterns.
- **ValidationResult**: The outcome of validating a potential match; includes pass/fail status and reason.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Email detection achieves 99% precision and 99% recall on standard test datasets.
- **SC-002**: IBAN detection with checksum validation achieves 100% precision (no false positives on invalid checksums).
- **SC-003**: Phone number detection achieves 95% recall across AT/DE/CH format variations.
- **SC-004**: Credit card detection with Luhn validation achieves 100% precision.
- **SC-005**: Detection processes 1MB of text in under 500ms.
- **SC-006**: All built-in detectors can be individually enabled/disabled without code changes.
- **SC-007**: Custom patterns can be added via configuration and work identically to built-in patterns.

## Assumptions

- Input text is provided as TextSegment objects from the plaintext-parser (Spec 001).
- Confidence scores are relative within the system; 1.0 means pattern+validation passed, lower scores indicate partial matches.
- Phone number detection focuses on DACH region; other regions can be added as custom patterns.
- The system detects but does not redact; redaction is handled by Spec 003.
