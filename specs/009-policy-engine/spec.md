# Feature Specification: Policy Engine

**Feature Branch**: `009-policy-engine`
**Created**: 2025-12-08
**Status**: Draft
**Input**: YAML-based policy configuration for detection and protection rules

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Apply Detection Rules (Priority: P1)

A compliance officer defines which PII types to detect and their confidence thresholds in a YAML policy file. The system filters detection results according to these rules.

**Why this priority**: Policies are essential for customizing behavior without code changes.

**Independent Test**: Create policy with specific detectors and thresholds, run detection, verify only matching findings reported.

**Acceptance Scenarios**:

1. **Given** policy enabling only `email` and `iban`, **When** scanning, **Then** phone numbers not reported.
2. **Given** confidence threshold `>= 0.8`, **When** finding has 0.7 confidence, **Then** finding is filtered out.
3. **Given** no policy file, **When** scanning, **Then** default policy (all detectors, threshold 0.5) applied.

---

### User Story 2 - Configure Protection Actions (Priority: P1)

A privacy analyst specifies how different PII types should be protected (redact, mask, encrypt) in the policy. The system applies the correct protection method per PII category.

**Why this priority**: Different PII types require different protection approaches (e.g., names masked, IBANs encrypted).

**Independent Test**: Create policy with different actions per type, run protection, verify correct action applied.

**Acceptance Scenarios**:

1. **Given** rule `detect: email, action: redact`, **When** protecting, **Then** emails are redacted with `[EMAIL]`.
2. **Given** rule `detect: iban, action: mask`, **When** protecting, **Then** IBANs are partially masked.
3. **Given** rule `detect: credit_card, action: encrypt`, **When** protecting, **Then** cards are encrypted.

---

### User Story 3 - Use Locale-Specific Policies (Priority: P2)

A multinational company uses different detection patterns per country. The policy specifies locale settings that activate region-specific detectors and dictionaries.

**Why this priority**: PII patterns vary by region; locale support enables proper detection per jurisdiction.

**Independent Test**: Create policy with locale, verify locale-specific detectors activated.

**Acceptance Scenarios**:

1. **Given** `locale: de-AT`, **When** scanning, **Then** Austrian SVNr detector enabled.
2. **Given** `locale: de-DE`, **When** scanning, **Then** German Sozialversicherungsnummer detector enabled.
3. **Given** no locale, **When** scanning, **Then** all DACH detectors enabled by default.

---

### User Story 4 - Define Consistent Pseudonymization (Priority: P2)

A data protection officer needs consistent pseudonymization where the same name always maps to the same pseudonym within a document or session. The policy enables this with a `consistent: true` flag.

**Why this priority**: Consistency is required for documents where entities need to remain trackable (e.g., legal documents).

**Independent Test**: Pseudonymize document with repeated names, verify same pseudonym used throughout.

**Acceptance Scenarios**:

1. **Given** `action: pseudonymize, consistent: true`, **When** "Max Müller" appears 5 times, **Then** same pseudonym used all 5 times.
2. **Given** `consistent: false`, **When** same name appears, **Then** different pseudonyms may be used.
3. **Given** consistent mode across multiple files, **When** same name in different files, **Then** same pseudonym used.

---

### User Story 5 - Reference External Keys (Priority: P2)

A security team stores encryption keys in environment variables or external vaults. The policy references these keys without embedding secrets.

**Why this priority**: Security best practice requires separating secrets from configuration.

**Independent Test**: Create policy with key reference, verify encryption uses resolved key.

**Acceptance Scenarios**:

1. **Given** `key_ref: "env://VEIL_KEY"`, **When** protecting, **Then** key read from environment variable.
2. **Given** missing environment variable, **When** protecting, **Then** clear error about missing key.
3. **Given** `key_ref: "file:///path/to/key"`, **When** protecting, **Then** key read from file.

---

### Edge Cases

- What happens with invalid YAML syntax? System reports parse error with line number.
- What happens with unknown detector name in policy? System warns but continues with valid rules.
- What happens when policy conflicts (same PII type, different actions)? Later rule wins (last definition precedence).
- What happens with empty policy file? System uses defaults with warning.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST parse YAML policy files with version, name, and rules sections.
- **FR-002**: System MUST support detection rules with: PII types, confidence thresholds, enabled/disabled flag.
- **FR-003**: System MUST support protection rules with: PII types, action (redact/mask/encrypt/pseudonymize/tokenize/hash), style options.
- **FR-004**: System MUST support locale setting affecting detector selection.
- **FR-005**: System MUST support `consistent: true/false` for pseudonymization.
- **FR-006**: System MUST support key references via `env://`, `file://` URI schemes.
- **FR-007**: System MUST validate policy on load and report all errors.
- **FR-008**: System MUST support policy inheritance/composition (base policy + overrides).
- **FR-009**: System MUST provide default policy when none specified.
- **FR-010**: System MUST support named policies for selection via CLI flag.
- **FR-011**: System MUST reject policies with unsupported versions.
- **FR-012**: Policy changes MUST take effect without application restart.

### Key Entities

- **Policy**: A complete policy definition; has version, name, locale, detection rules, protection rules.
- **DetectionRule**: A rule for filtering findings; specifies PII types, confidence threshold, enabled flag.
- **ProtectionRule**: A rule for applying protection; specifies PII types, action, style, options.
- **KeyReference**: A pointer to an encryption key; uses URI scheme to identify source (env, file).
- **PolicyValidationResult**: Outcome of policy validation; lists errors, warnings, resolved configuration.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Valid YAML policies parse in under 100ms.
- **SC-002**: Policy validation catches 100% of schema violations with clear error messages.
- **SC-003**: Detection filtering matches policy rules with 100% accuracy.
- **SC-004**: Protection actions are applied exactly as specified in policy.
- **SC-005**: Consistent pseudonymization produces identical output for identical input within scope.
- **SC-006**: Key references resolve correctly from environment and file sources.

## Assumptions

- Policy files are trusted; malicious policies could cause denial of service but not code execution.
- YAML is the only policy format; JSON could be supported later if needed.
- Policy versioning follows semver; incompatible policy versions cause load failure.
- The default policy is permissive (detect all, redact all) to ensure no PII is missed by accident.

## Policy Schema Example

```yaml
version: "1.0"
name: "GDPR Standard"
locale: "de-AT"

detection:
  - types: [email, phone, address]
    confidence: ">= 0.8"
    enabled: true

  - types: [person_name]
    confidence: ">= 0.6"
    enabled: true

protection:
  - types: [email, phone]
    action: redact
    style: category_label

  - types: [person_name]
    action: pseudonymize
    consistent: true

  - types: [iban, credit_card]
    action: encrypt
    key_ref: "env://VEIL_ENCRYPTION_KEY"
```
