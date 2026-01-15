# Feature Specification: Redaction Engine

**Feature Branch**: `003-redaction-engine`
**Created**: 2025-12-08
**Status**: Draft
**Input**: Text redaction and masking for detected PII

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Redact with Replacement Labels (Priority: P1)

A privacy analyst wants to replace detected PII with category labels like `[EMAIL]`, `[PHONE]`, `[IBAN]`. The original text is replaced with the label, making the redaction visible and the category clear.

**Why this priority**: Label replacement is the most common redaction style for document sanitization and provides clear indication of what was removed.

**Independent Test**: Provide text with detected findings, apply label redaction, verify each finding is replaced with its category label.

**Acceptance Scenarios**:

1. **Given** text `Contact me at john@example.com`, **When** redacted with labels, **Then** output is `Contact me at [EMAIL]`.
2. **Given** multiple findings of same type, **When** redacted, **Then** each gets its own label (not merged).
3. **Given** adjacent findings, **When** redacted, **Then** both are replaced correctly without overlap issues.

---

### User Story 2 - Redact with Black Bars (Priority: P1)

A legal team needs to create documents where PII is visually blocked out with solid characters (e.g., `████████`) matching the length of the original text, suitable for formal document redaction.

**Why this priority**: Black bar redaction is standard for legal documents and preserves document layout by maintaining character count.

**Independent Test**: Provide text with findings, apply black bar redaction, verify output length matches input length.

**Acceptance Scenarios**:

1. **Given** text `IBAN: DE89370400440532013000`, **When** redacted with bars, **Then** IBAN becomes `████████████████████████` (22 chars).
2. **Given** multiline text with PII, **When** redacted, **Then** line structure is preserved.
3. **Given** findings of different lengths, **When** redacted, **Then** each bar matches its original length.

---

### User Story 3 - Mask Partial Data (Priority: P2)

A customer service team needs to show partial information for verification (e.g., `****@****.com` for email, `DE89****0532013000` for IBAN) while hiding the sensitive core.

**Why this priority**: Partial masking allows data verification without full exposure, useful for customer-facing scenarios.

**Independent Test**: Provide various PII types, apply masking rules, verify exposed portions match expected patterns.

**Acceptance Scenarios**:

1. **Given** email `john.doe@example.com`, **When** masked, **Then** output is `j*******@*******.com` (preserving first char and domain extension).
2. **Given** IBAN, **When** masked, **Then** country code and last 4 digits visible: `DE89**************3000`.
3. **Given** phone number, **When** masked, **Then** last 4 digits visible: `+43 *** *** **67`.

---

### User Story 4 - Apply Redaction to Original Positions (Priority: P1)

A developer integrates redaction into a pipeline. The redaction engine receives findings with position information and produces output that can be mapped back to original document positions for format-specific redaction (e.g., PDF annotation, Excel cell update).

**Why this priority**: Position-preserving redaction is essential for downstream format-specific protection.

**Independent Test**: Provide findings with positions, redact, verify position mapping is preserved in output.

**Acceptance Scenarios**:

1. **Given** finding at positions 10-25, **When** redacted, **Then** output includes mapping from original to redacted positions.
2. **Given** CSV finding with row/column, **When** redacted, **Then** cell coordinates are preserved in output.
3. **Given** JSON finding with path, **When** redacted, **Then** JSON path is preserved for targeted update.

---

### Edge Cases

- What happens when redaction overlaps (nested findings)? System processes outer finding; inner findings within the redacted region are skipped.
- What happens when replacement is longer than original? System adjusts positions for subsequent findings to maintain accuracy.
- What happens with empty findings? System skips empty matches without error.
- What happens with Unicode text? System correctly handles multi-byte characters in position calculations.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support label replacement redaction using PII category names (e.g., `[EMAIL]`, `[IBAN]`).
- **FR-002**: System MUST support black bar redaction using configurable replacement character (default: `█`).
- **FR-003**: System MUST support partial masking with configurable exposure rules per PII type.
- **FR-004**: System MUST preserve position mapping between original and redacted text.
- **FR-005**: System MUST handle overlapping findings by processing outermost first.
- **FR-006**: System MUST correctly handle Unicode text with accurate character-level positioning.
- **FR-007**: System MUST return redacted text along with a mapping of original-to-redacted positions.
- **FR-008**: System MUST support custom replacement text per PII category.
- **FR-009**: System MUST process findings in position order to ensure consistent output.
- **FR-010**: System MUST preserve non-PII text exactly as provided.

### Key Entities

- **RedactionStyle**: The type of redaction to apply; one of: Label, BlackBar, Mask, Custom.
- **RedactionConfig**: Settings for the redaction engine; includes style, replacement character, masking rules per PII type.
- **RedactionResult**: Output of redaction; contains redacted text, position mapping, and list of applied redactions.
- **PositionMap**: Mapping between original and redacted positions; enables downstream systems to locate redactions in original format.
- **MaskingRule**: For partial masking; defines which parts of a PII value to expose (e.g., first N chars, last N chars, domain).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All detected findings are redacted with zero leakage of original PII values.
- **SC-002**: Position mapping accuracy is 100% - redacted positions map correctly to originals.
- **SC-003**: Redaction of 10,000 findings completes in under 1 second.
- **SC-004**: Black bar redaction preserves exact character count of original text.
- **SC-005**: Partial masking rules are configurable without code changes.
- **SC-006**: Unicode text is handled correctly with no position drift.

## Assumptions

- Input findings come from the detection engine (Spec 002) with position metadata.
- Redaction operates on text; format-specific application (PDF, Excel) is handled by format parsers.
- Masking rules for each PII type have sensible defaults; custom rules can override.
- The redaction engine is stateless; each call processes independently.
