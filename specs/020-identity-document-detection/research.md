# Research: Identity Document Detection

## US Social Security Number (SSN)

### Format Research

**Decision**: Support all common SSN formats with area number validation.

**Standard Format**: AAA-GG-SSSS
- Area Number (AAA): 3 digits, assigned geographically (historically)
- Group Number (GG): 2 digits
- Serial Number (SSSS): 4 digits

**Formats to detect**:
| Format | Example | Regex |
|--------|---------|-------|
| Hyphenated | 123-45-6789 | `\d{3}-\d{2}-\d{4}` |
| Space-separated | 123 45 6789 | `\d{3}\s\d{2}\s\d{4}` |
| Consecutive (with context) | 123456789 | `\d{9}` (only with SSN context) |

**Rationale**: Hyphenated is most common. Space-separated appears in some documents. Consecutive 9 digits require context to avoid false positives.

### Invalid SSN Patterns

**Decision**: Validate area numbers and flag known invalid patterns.

| Pattern | Status | Reason |
|---------|--------|--------|
| 000-XX-XXXX | Invalid | Area 000 never issued |
| 666-XX-XXXX | Invalid | Area 666 never issued |
| 9XX-XX-XXXX | Invalid (mostly) | Reserved for ITIN (9XX) |
| XXX-00-XXXX | Invalid | Group 00 never issued |
| XXX-XX-0000 | Invalid | Serial 0000 never issued |
| 078-05-1120 | Test | Woolworth wallet card SSN |
| 219-09-9999 | Test | Common test SSN |

**Rationale**: SSA has documented invalid ranges. Flagging these reduces false positives and identifies test data.

### Confidence Scoring

| Condition | Confidence | Rationale |
|-----------|------------|-----------|
| Hyphenated format + valid area | 0.95 | Strong match |
| With "SSN:" label context | 0.98 | Very high confidence |
| Space-separated format | 0.90 | Less common format |
| Consecutive 9 digits with context | 0.85 | Requires context |
| Consecutive 9 digits without context | 0.50 | High false positive risk |
| Known invalid area number | 0.40 | Likely test data |

## US Passport Numbers

### Format Research

**Decision**: Detect 9-digit US passport numbers with context boost.

**Format**: 9 alphanumeric characters
- Typically all numeric for recent passports
- Older passports may have leading letter

**Patterns to detect**:
| Format | Example | Regex |
|--------|---------|-------|
| 9-digit numeric | 123456789 | `\d{9}` (with passport context) |
| Alphanumeric | A12345678 | `[A-Z]\d{8}` |

**Rationale**: US passports use 9 characters. Without context, these overlap with SSN patterns, so context is essential.

### Confidence Scoring

| Condition | Confidence | Rationale |
|-----------|------------|-----------|
| With "Passport:" context | 0.95 | High confidence |
| In travel document context | 0.90 | Document structure indicates passport |
| 9 digits without context | 0.40 | Too ambiguous (could be SSN) |

## UK/EU Passport Numbers

### Format Research

**Decision**: Support UK and major EU country passport formats.

| Country | Format | Example | Regex |
|---------|--------|---------|-------|
| UK | 9 digits | 123456789 | `\d{9}` |
| Germany | 9 alphanumeric | C1234567D | `[CFGHJKLMNPRTVWXYZ0-9]{9}` |
| France | 9 alphanumeric | 12AB34567 | `[A-Z0-9]{9}` |
| Generic EU | 6-9 alphanumeric | ABC123456 | `[A-Z0-9]{6,9}` |

**Rationale**: EU passports vary by country but generally 9 alphanumeric. German passports exclude vowels and some letters to avoid offensive words.

### Confidence Scoring

| Condition | Confidence | Rationale |
|-----------|------------|-----------|
| With passport context | 0.90 | High confidence with context |
| Matches German format | 0.85 | Specific character set match |
| Generic alphanumeric with context | 0.80 | Broad pattern |
| Without context | 0.30 | Too ambiguous |

## US Driver's License Numbers

### Format Research

**Decision**: Support major US state formats (CA, NY, TX, FL, IL) covering ~40% of US population.

| State | Format | Example | Regex |
|-------|--------|---------|-------|
| California | 1 letter + 7 digits | A1234567 | `[A-Z]\d{7}` |
| New York | 9 digits | 123456789 | `\d{9}` |
| Texas | 8 digits | 12345678 | `\d{8}` |
| Florida | 1 letter + 12 digits | A123-456-78-901-2 | `[A-Z]\d{12}` |
| Illinois | 1 letter + 11 digits | A123-4567-8901 | `[A-Z]\d{11}` |

**Rationale**: State formats vary significantly. Focus on highest-population states first. Context is essential to distinguish from other number formats.

### Confidence Scoring

| Condition | Confidence | Rationale |
|-----------|------------|-----------|
| With "Driver's License:" context | 0.95 | High confidence |
| With "DL:" context | 0.90 | Common abbreviation |
| Matches state-specific format | 0.85 | Format match |
| Generic letter + digits | 0.60 | Ambiguous |

## Pattern Priority

**Decision**: Order patterns from most specific to least specific.

1. SSN patterns (most distinctive format)
2. State-specific DL patterns (distinctive prefixes)
3. Country-specific passport patterns
4. Generic passport pattern (catch-all)

**Rationale**: More specific patterns have fewer false positives and should match first.

## Context Labels

**Decision**: Define context labels that boost confidence significantly.

| Category | Labels | Boost |
|----------|--------|-------|
| SSN | "SSN:", "Social Security:", "SS#:" | +0.20 |
| Passport | "Passport:", "Passport No:", "Travel Document:" | +0.20 |
| DL | "Driver's License:", "DL:", "License No:" | +0.20 |

**Rationale**: Context labels are strong indicators. Existing context detection infrastructure can be reused.

## Validation Rules

### SSN Validation

1. Must have exactly 9 digits
2. Area number (first 3) not in invalid set: 000, 666, 9XX
3. Group number (middle 2) not 00
4. Serial number (last 4) not 0000

### Passport Validation

1. Must be 6-9 alphanumeric characters
2. Country-specific character sets where applicable

### Driver's License Validation

1. State-specific format validation where applicable
2. Length constraints by state

## Implementation Approach

**Decision**: Create three new detector modules following existing pattern.

### New Files
- `crates/veil-detect/src/patterns/ssn.rs`
- `crates/veil-detect/src/patterns/passport.rs`
- `crates/veil-detect/src/patterns/drivers_license.rs`
- `crates/veil-detect/src/validators/ssn.rs`

### New PII Categories
- `PiiCategory::Ssn`
- `PiiCategory::Passport`
- `PiiCategory::DriversLicense`

### Dependencies
- No new dependencies required
- Reuse regex, once_cell from existing patterns

## Performance Considerations

**Decision**: Compile all patterns at startup using `Lazy<Vec<Regex>>`.

- Current veil-detect: ~8 pattern detectors
- After: 11 pattern detectors (+3)
- Expected impact: <5% overhead

**Rationale**: Pattern matching is already fast; adding 3 more detectors is negligible.
