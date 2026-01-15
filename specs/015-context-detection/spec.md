# Feature Specification: Context-Aware Detection

**Feature Branch**: `015-context-detection`
**Created**: 2025-12-15
**Status**: Draft
**Input**: Surrounding context analysis to improve PII detection accuracy

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Detect Named Entities by Context (Priority: P1)

A privacy analyst scans text where names appear without explicit labels. The system uses surrounding words (e.g., "Dear", "Mr.", "signed by") to identify person names that aren't in any dictionary.

**Why this priority**: Many names aren't in dictionaries; context is essential for accurate name detection.

**Independent Test**: Provide text with names preceded by contextual markers, verify names detected.

**Acceptance Scenarios**:

1. **Given** text "Dear Mr. Tanaka", **When** scanned, **Then** "Tanaka" detected as PersonName with high confidence.
2. **Given** text "signed by: Alexandra Petrova", **When** scanned, **Then** "Alexandra Petrova" detected as PersonName.
3. **Given** text "Contact person: Wei Chen", **When** scanned, **Then** "Wei Chen" detected as PersonName.

---

### User Story 2 - Reduce False Positives with Context (Priority: P1)

A compliance team sees too many false positives where common words match PII patterns. The system uses context to suppress detections that are clearly not PII (e.g., "version 1.2.3.4" is not an IP address).

**Why this priority**: False positives waste analyst time and erode trust in the system.

**Independent Test**: Provide text with PII-like patterns in non-PII context, verify suppressed.

**Acceptance Scenarios**:

1. **Given** text "version 192.168.1.1", **When** scanned, **Then** IP address detection suppressed due to "version" context.
2. **Given** text "order #4532-1234-5678-9012", **When** scanned, **Then** credit card detection suppressed due to "order #" context.
3. **Given** text "ISBN 978-3-16-148410-0", **When** scanned, **Then** not detected as credit card or phone.

---

### User Story 3 - Detect Addresses from Structure (Priority: P2)

A data protection officer scans documents for postal addresses. The system recognizes multi-line address patterns (street, city, postal code, country) as a single Address entity.

**Why this priority**: Addresses span multiple lines and require structural analysis.

**Independent Test**: Provide multi-line addresses in various formats, verify detected as single entity.

**Acceptance Scenarios**:

1. **Given** text with "123 Main St\nNew York, NY 10001", **When** scanned, **Then** entire block detected as Address.
2. **Given** German address "Hauptstraße 42\n80331 München", **When** scanned, **Then** detected as Address.
3. **Given** address with country "Vienna, Austria 1010", **When** scanned, **Then** detected as Address.

---

### User Story 4 - Detect PII in Tables (Priority: P2)

A security analyst scans CSV/Excel data where column headers indicate PII type. The system uses header context to boost confidence for values in PII-labeled columns.

**Why this priority**: Tabular data often has explicit column labels that indicate content type.

**Independent Test**: Provide CSV with "Email" header, verify column values detected with boosted confidence.

**Acceptance Scenarios**:

1. **Given** CSV with header "Email", **When** values scanned, **Then** email-like values in that column get confidence boost.
2. **Given** CSV with header "Customer Name", **When** values scanned, **Then** all values in column flagged as potential names.
3. **Given** header "ID" with numeric values, **When** scanned, **Then** values not falsely detected as SSN/phone.

---

### User Story 5 - Language-Aware Context (Priority: P2)

A multinational organization scans documents in multiple languages. The system applies language-specific context rules (e.g., "Herr" in German, "Monsieur" in French).

**Why this priority**: Context markers vary by language; single-language rules miss international documents.

**Independent Test**: Provide German text with "Herr/Frau" honorifics, verify name detection.

**Acceptance Scenarios**:

1. **Given** German text "Sehr geehrter Herr Müller", **When** scanned, **Then** "Müller" detected as PersonName.
2. **Given** French text "Madame Dupont", **When** scanned, **Then** "Dupont" detected as PersonName.
3. **Given** mixed language document, **When** scanned, **Then** context rules applied per-section.

---

### User Story 6 - Configurable Context Rules (Priority: P3)

A privacy engineer needs to add custom context rules for domain-specific patterns. The system supports user-defined context patterns that boost or suppress detection.

**Why this priority**: Different industries have unique patterns that need custom context rules.

**Independent Test**: Define custom context rule, verify it affects detection confidence.

**Acceptance Scenarios**:

1. **Given** custom rule `boost: "patient name:"`, **When** text matches, **Then** following words get PersonName boost.
2. **Given** custom rule `suppress: "product code:"`, **When** text matches, **Then** following pattern not detected as PII.
3. **Given** YAML context rule file, **When** loaded, **Then** rules applied during detection.

---

### Edge Cases

- What happens with ambiguous context? System uses confidence scoring; high-confidence patterns override context.
- What happens with nested context? System applies innermost context rule.
- What happens with OCR-extracted text (poor quality)? System applies fuzzy context matching.
- What happens with code/logs mixed with text? System detects code-like sections and suppresses false positives.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST detect person names using contextual markers (honorifics, labels).
- **FR-002**: System MUST suppress false positives when context indicates non-PII usage.
- **FR-003**: System MUST recognize multi-line postal addresses as single entities.
- **FR-004**: System MUST use column headers to boost detection confidence in tabular data.
- **FR-005**: System MUST support language-specific context markers (EN, DE, FR minimum).
- **FR-006**: System MUST support user-defined context rules via YAML configuration.
- **FR-007**: System MUST adjust confidence scores based on context analysis.
- **FR-008**: System MUST handle multi-language documents with section-aware context.
- **FR-009**: Context analysis MUST NOT significantly impact detection performance (<10% overhead).
- **FR-010**: System MUST provide context reasoning in detection metadata.

### Key Entities

- **ContextRule**: A rule for context-based adjustment; contains pattern, action (boost/suppress), and weight.
- **ContextMarker**: A detected context indicator; contains type, text, position, language.
- **AddressBlock**: A detected address; contains components (street, city, postal, country) and span.
- **ContextConfig**: Configuration for context detection; contains rules by language and category.
- **ContextAnalysis**: Result of context analysis; contains markers found, adjustments applied.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Name detection recall improves by 30% with context markers vs dictionary-only.
- **SC-002**: False positive rate for IP-like patterns reduced by 50% with context suppression.
- **SC-003**: Multi-line addresses detected with 90% accuracy across EN/DE/FR formats.
- **SC-004**: Column header context boosts detection precision by 20% for tabular data.
- **SC-005**: Context analysis adds <10% processing time overhead.
- **SC-006**: Custom context rules work correctly when loaded from YAML.

## Assumptions

- Context analysis runs as a post-processing step after pattern/dictionary detection.
- Language detection is handled separately (or per-document setting).
- Context rules are additive to base detection; they adjust confidence, not replace detection.
- Address detection focuses on postal addresses; geolocation coordinates are separate PII type.
