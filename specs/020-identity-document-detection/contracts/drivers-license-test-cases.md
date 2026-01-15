# Driver's License Test Cases

## California (1 Letter + 7 Digits)

| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `DL: A1234567` | Yes | Yes | 0.95 | Standard CA format |
| `Driver's License: A1234567` | Yes | Yes | 0.95 | Full label |
| `CA DL: A1234567` | Yes | Yes | 0.98 | State context |
| `A1234567` | None | Low | 0.60 | Ambiguous |

## New York (9 Digits)

| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `DL: 123456789` | Yes | Yes | 0.90 | NY format |
| `NY License: 123456789` | Yes | Yes | 0.95 | State context |
| `123456789` | None | Low | 0.40 | Ambiguous (SSN-like) |

## Texas (8 Digits)

| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `DL: 12345678` | Yes | Yes | 0.90 | TX format |
| `Texas DL: 12345678` | Yes | Yes | 0.95 | State context |
| `12345678` | None | Low | 0.50 | Ambiguous |

## Florida (1 Letter + 12 Digits)

| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `DL: A123456789012` | Yes | Yes | 0.95 | FL format |
| `A123-456-78-901-2` | Yes | Yes | 0.90 | FL with dashes |
| `FL License: A123456789012` | Yes | Yes | 0.98 | State context |

## Illinois (1 Letter + 11 Digits)

| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `DL: A12345678901` | Yes | Yes | 0.95 | IL format |
| `A123-4567-8901` | Yes | Yes | 0.90 | IL with dashes |
| `IL DL: A12345678901` | Yes | Yes | 0.98 | State context |

## Generic Format

| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `License: ABC123` | Yes | Yes | 0.70 | Generic short |
| `DL: 1234567` | Yes | Yes | 0.75 | Generic 7 digits |
| `License No: XY1234567` | Yes | Yes | 0.80 | Generic format |

## Edge Cases

### Should NOT Match
| Input | Notes |
|-------|-------|
| `12345` | Too short |
| `ABCDEFGHIJ` | All letters |
| `DL-A1234567` | Prefixed with letters |

### Should Match (lower confidence)
| Input | Notes |
|-------|-------|
| `License #: A1234567` | Hash variant |
| `Operator License: A1234567` | Alt terminology |

## Context Patterns

### Labels That Boost Confidence
| Pattern | Category | Boost |
|---------|----------|-------|
| `Driver's License:` | drivers_license | +0.20 |
| `Drivers License:` | drivers_license | +0.20 |
| `DL:` | drivers_license | +0.20 |
| `License No:` | drivers_license | +0.15 |
| `License Number:` | drivers_license | +0.15 |
| `Operator License:` | drivers_license | +0.10 |
| `CDL:` | drivers_license | +0.10 |

### State Prefixes That Boost Confidence
| Pattern | Category | Boost |
|---------|----------|-------|
| `CA DL:` | drivers_license | +0.25 |
| `NY License:` | drivers_license | +0.25 |
| `TX DL:` | drivers_license | +0.25 |
| `FL License:` | drivers_license | +0.25 |
| `IL DL:` | drivers_license | +0.25 |
