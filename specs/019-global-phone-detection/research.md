# Research: Global Phone Number Detection

## Phone Format Research

### US Phone Numbers (NANP - North American Numbering Plan)

**Decision**: Support multiple US formats with appropriate regex patterns.

**Formats to detect**:
| Format | Example | Regex |
|--------|---------|-------|
| E.164 | +1 555 123 4567 | `\+1[\s.-]?\d{3}[\s.-]?\d{3}[\s.-]?\d{4}` |
| With 1 prefix | 1-555-123-4567 | `1[\s.-]?\d{3}[\s.-]?\d{3}[\s.-]?\d{4}` |
| Parentheses | (555) 123-4567 | `\(\d{3}\)[\s.-]?\d{3}[\s.-]?\d{4}` |
| 10-digit local | 555-123-4567 | `\d{3}[\s.-]\d{3}[\s.-]\d{4}` |
| Toll-free | 1-800-555-1234 | `1?[\s.-]?8(?:00|88|77|66|55|44|33)[\s.-]?\d{3}[\s.-]?\d{4}` |

**Rationale**: NANP is well-defined with consistent 10-digit format (3-3-4). Multiple visual separators are common.

### UK Phone Numbers

**Decision**: Support +44 international and local formats.

**Formats to detect**:
| Format | Example | Regex |
|--------|---------|-------|
| E.164 landline | +44 20 7946 0958 | `\+44[\s.-]?\d{2,4}[\s.-]?\d{3,4}[\s.-]?\d{3,4}` |
| E.164 mobile | +44 7911 123456 | `\+44[\s.-]?7\d{3}[\s.-]?\d{6}` |
| Local landline | 020 7946 0958 | `0\d{2,4}[\s.-]?\d{3,4}[\s.-]?\d{3,4}` |
| Local mobile | 07911 123456 | `07\d{3}[\s.-]?\d{6}` |

**Rationale**: UK has variable-length area codes (2-5 digits) making patterns more complex. Mobile always starts with 07.

### French Phone Numbers

**Decision**: Support +33 international and local formats.

**Formats to detect**:
| Format | Example | Regex |
|--------|---------|-------|
| E.164 | +33 1 23 45 67 89 | `\+33[\s.-]?\d[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2}` |
| Local | 01 23 45 67 89 | `0\d[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2}` |

**Rationale**: French numbers are 10 digits with consistent 2-2-2-2-2 grouping after region code.

### E.164 International Format

**Decision**: Add generic E.164 pattern as catch-all for any country.

**Format**: `+[country code][subscriber number]`
- Country code: 1-3 digits
- Subscriber number: 4-14 digits
- Total: 7-15 digits after +

**Regex**: `\+[1-9]\d{6,14}`

**Rationale**: E.164 is the ITU standard. This catches any properly formatted international number.

### Pattern Priority

**Decision**: Order patterns from most specific to least specific.

1. Specific country patterns (DACH, US, UK, FR) - Higher confidence
2. Generic E.164 pattern - Lower confidence (catch-all)

**Rationale**: More specific patterns provide better confidence scores and prevent the generic pattern from matching everything.

## Confidence Scoring

| Pattern Type | Base Confidence | Rationale |
|-------------|-----------------|-----------|
| E.164 with known country | 0.95 | Explicit international format |
| US/UK/FR specific format | 0.90 | Matches known regional pattern |
| Generic E.164 | 0.85 | Could be any country |
| Local format (no country) | 0.80 | Ambiguous without context |

## Validation Rules

1. **Minimum digits**: 7 (excluding country code)
2. **Maximum digits**: 15 (per E.164 spec)
3. **No overlapping matches**: Later patterns skip already-matched ranges
4. **Digit-only validation**: Strip separators, count digits

## Edge Cases

| Case | Handling |
|------|----------|
| 123-4567 (7 digits, no area) | Detect with low confidence |
| +1 555-123-4567 ext 890 | Detect main number only |
| SKU-555-1234 | Context analysis in detector |
| 1-800-FLOWERS | Future: letter-to-digit conversion |

## Backward Compatibility

**Decision**: Keep existing DACH patterns at the top of the list.

- Existing tests must pass unchanged
- DACH patterns match before generic E.164
- No changes to validation logic

## Performance Considerations

**Decision**: Compile all patterns at startup using `Lazy<Vec<Regex>>`.

- Current: 4 patterns
- After: ~12 patterns
- Expected impact: <10% overhead (regex matching is fast)
