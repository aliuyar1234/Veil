# Passport Test Cases

## US Passport Numbers

### 9-Digit Numeric
| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `Passport: 123456789` | Yes | Yes | 0.95 | With context |
| `Passport No: 123456789` | Yes | Yes | 0.95 | Variant label |
| `123456789` | None | Low | 0.40 | Ambiguous (could be SSN) |

### Alphanumeric (Older Format)
| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `Passport: A12345678` | Yes | Yes | 0.90 | Letter prefix |
| `A12345678` | None | Low | 0.50 | Ambiguous |

## UK Passport Numbers

### 9-Digit Format
| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `Passport: 123456789` | Yes | Yes | 0.90 | UK format |
| `UK Passport: 123456789` | Yes | Yes | 0.95 | Country context |

## EU Passport Numbers

### German Passport (9 Alphanumeric)
| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `Passport: C1234567D` | Yes | Yes | 0.90 | German format |
| `Reisepass: C1234567D` | Yes | Yes | 0.95 | German label |
| `CFGHJK123` | None | Low | 0.60 | German charset |

### French Passport (9 Alphanumeric)
| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `Passport: 12AB34567` | Yes | Yes | 0.90 | French format |
| `Passeport: 12AB34567` | Yes | Yes | 0.95 | French label |

### Generic EU (6-9 Alphanumeric)
| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `Passport: ABC123` | Yes | Yes | 0.80 | 6 chars |
| `Passport: ABCD12345` | Yes | Yes | 0.80 | 9 chars |

## Edge Cases

### Should NOT Match
| Input | Notes |
|-------|-------|
| `12345` | Too short (5 chars) |
| `1234567890` | Too long for passport |
| `PASSPORT123456789` | Preceded by letters |

### Should Match (lower confidence)
| Input | Notes |
|-------|-------|
| `Travel Document: 123456789` | Alternate context |
| `Passport #: 123456789` | Hash symbol variant |

## Context Patterns

### Labels That Boost Confidence
| Pattern | Category | Boost |
|---------|----------|-------|
| `Passport:` | passport | +0.20 |
| `Passport No:` | passport | +0.20 |
| `Passport Number:` | passport | +0.20 |
| `Reisepass:` | passport | +0.20 |
| `Passeport:` | passport | +0.20 |
| `Travel Document:` | passport | +0.15 |
| `Document No:` | passport | +0.10 |
