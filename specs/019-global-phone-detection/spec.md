# Feature Specification: Global Phone Number Detection

**Feature Branch**: `019-global-phone-detection`
**Created**: 2025-12-17
**Status**: Draft
**Input**: User description: "Add global phone number detection patterns supporting US, UK, France, and international formats. Current implementation only supports DACH region (Germany, Austria, Switzerland). This causes data leakage for any non-DACH phone numbers."

## Problem Statement

The current phone number detector only recognizes DACH region formats (Germany +49, Austria +43, Switzerland +41). Phone numbers from other regions pass through undetected, causing PII leakage:

- US phone numbers: +1 (555) 123-4567, 555-123-4567
- UK phone numbers: +44 20 7946 0958, 020 7946 0958
- French phone numbers: +33 1 23 45 67 89
- Generic international format: +XX XXX XXX XXXX

This is a critical gap for any enterprise operating globally or processing international data.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Detect US Phone Numbers (Priority: P1)

As a compliance officer at a US-based company, I need the system to detect US phone numbers in all common formats, so that our customer data is properly protected regardless of how phone numbers are formatted.

**Why this priority**: US is one of the largest markets; US phone formats are extremely common in global business data.

**Independent Test**: Can be fully tested by scanning documents containing US phone numbers in various formats and verifying all are detected.

**Acceptance Scenarios**:

1. **Given** a document containing "+1 (555) 123-4567", **When** I scan it, **Then** the phone number is detected with category "phone"
2. **Given** a document containing "555-123-4567", **When** I scan it, **Then** the phone number is detected (10-digit US format)
3. **Given** a document containing "(555) 123-4567", **When** I scan it, **Then** the phone number is detected
4. **Given** a document containing "1-800-555-1234", **When** I scan it, **Then** the toll-free number is detected

---

### User Story 2 - Detect UK Phone Numbers (Priority: P1)

As a data protection officer at a UK organization, I need UK phone numbers to be detected in all standard formats, so that we maintain GDPR compliance for personal contact information.

**Why this priority**: UK is a major market with distinct phone number formats that differ from both US and EU patterns.

**Independent Test**: Can be tested by scanning documents with UK phone numbers and verifying detection.

**Acceptance Scenarios**:

1. **Given** a document containing "+44 20 7946 0958", **When** I scan it, **Then** the phone number is detected
2. **Given** a document containing "020 7946 0958", **When** I scan it, **Then** the phone number is detected (UK local format)
3. **Given** a document containing "+44 7911 123456", **When** I scan it, **Then** the mobile number is detected
4. **Given** a document containing "07911 123456", **When** I scan it, **Then** the UK mobile format is detected

---

### User Story 3 - Detect International Format Phone Numbers (Priority: P1)

As a global enterprise user, I need phone numbers in E.164 international format to be detected regardless of country code, so that no phone number in standard international format goes undetected.

**Why this priority**: E.164 is the universal standard for international phone numbers and must be detected for true global coverage.

**Independent Test**: Can be tested by scanning documents with various country codes in E.164 format.

**Acceptance Scenarios**:

1. **Given** a document containing "+33 1 23 45 67 89" (France), **When** I scan it, **Then** the phone number is detected
2. **Given** a document containing "+81 3 1234 5678" (Japan), **When** I scan it, **Then** the phone number is detected
3. **Given** a document containing "+61 2 1234 5678" (Australia), **When** I scan it, **Then** the phone number is detected
4. **Given** a document containing "+91 98765 43210" (India), **When** I scan it, **Then** the phone number is detected

---

### User Story 4 - Maintain Existing DACH Detection (Priority: P2)

As an existing user relying on DACH phone detection, I need the current German, Austrian, and Swiss phone number detection to continue working exactly as before, so that existing workflows are not disrupted.

**Why this priority**: Backward compatibility ensures no regression for current users.

**Independent Test**: Run existing phone detection tests and verify they all pass.

**Acceptance Scenarios**:

1. **Given** the existing test suite for phone detection, **When** I run all tests, **Then** all DACH format tests pass unchanged
2. **Given** a document containing "+49 89 12345678", **When** I scan it, **Then** it is detected exactly as before

---

### Edge Cases

- What happens with ambiguous numbers like "123-4567" (too short, could be extension)? Numbers with fewer than 7 digits should not be detected as phone numbers.
- How to handle numbers with extensions like "+1 555-123-4567 ext. 890"? The main number should be detected; extension is optional metadata.
- What about numbers that look like phones but aren't (e.g., product codes "SKU-555-1234")? Context analysis should help reduce false positives; confidence score reflects uncertainty.
- How to handle numbers with letters like "1-800-FLOWERS"? Vanity numbers should be detected as phone numbers.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST detect US phone numbers in formats: +1 XXX-XXX-XXXX, (XXX) XXX-XXXX, XXX-XXX-XXXX, 1-XXX-XXX-XXXX
- **FR-002**: System MUST detect UK phone numbers in formats: +44 XX XXXX XXXX, 0XX XXXX XXXX, +44 7XXX XXXXXX, 07XXX XXXXXX
- **FR-003**: System MUST detect phone numbers in E.164 international format: +[country code] [number] for all country codes
- **FR-004**: System MUST detect French phone numbers in formats: +33 X XX XX XX XX, 0X XX XX XX XX
- **FR-005**: System MUST continue detecting existing DACH formats (DE +49, AT +43, CH +41) without regression
- **FR-006**: System MUST detect toll-free numbers (800, 888, 877, 866, 855, 844, 833 in US)
- **FR-007**: System MUST require minimum 7 digits for phone number detection to avoid false positives
- **FR-008**: System MUST support common separators: spaces, hyphens, dots, parentheses
- **FR-009**: System MUST assign appropriate confidence scores based on format specificity (E.164 = high, ambiguous = lower)
- **FR-010**: System MUST avoid overlapping detections when the same number matches multiple patterns

### Key Entities

- **PhonePattern**: A detection pattern for a specific phone number format, including country/region, format description, and validation rules
- **PhoneMatch**: A detected phone number with start/end positions, matched text, detected format, and confidence score
- **CountryCode**: ISO country code associated with a phone pattern for categorization

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: System detects 95%+ of phone numbers in standard US formats (NANP)
- **SC-002**: System detects 95%+ of phone numbers in standard UK formats
- **SC-003**: System detects 95%+ of phone numbers in E.164 international format
- **SC-004**: False positive rate remains below 5% (numbers incorrectly flagged as phones)
- **SC-005**: All existing DACH phone detection tests pass (0 regressions)
- **SC-006**: Detection performance impact is less than 10% compared to DACH-only detection

## Assumptions

- E.164 format (+[country code][number]) is the universal standard for international detection
- Local formats (without country code) will have lower confidence than international formats
- The system will not attempt to validate that phone numbers are actually in service
- Common vanity numbers with letters (1-800-FLOWERS) should be detected

## Out of Scope

- Phone number validation against carrier databases
- Detection of every country's local format (focus on major markets + E.164)
- Formatting or normalization of detected numbers
- Country-specific validation rules (e.g., valid area codes)
