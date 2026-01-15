# SSN Test Cases

## Valid SSN Formats

### Hyphenated Format (Primary)
| Input | Should Match | Confidence | Notes |
|-------|--------------|------------|-------|
| `123-45-6789` | Yes | 0.95 | Standard format |
| `001-01-0001` | Yes | 0.95 | Low numbers valid |
| `899-99-9999` | Yes | 0.95 | High area valid |
| `555-55-5555` | Yes | 0.95 | Repeated digits valid |

### Space-Separated Format
| Input | Should Match | Confidence | Notes |
|-------|--------------|------------|-------|
| `123 45 6789` | Yes | 0.90 | Less common format |
| `001 01 0001` | Yes | 0.90 | Space variant |

### Consecutive Digits (Context Required)
| Input | Context | Should Match | Confidence | Notes |
|-------|---------|--------------|------------|-------|
| `SSN: 123456789` | SSN label | Yes | 0.85 | Context boost |
| `Social Security: 123456789` | SS label | Yes | 0.85 | Context boost |
| `123456789` | None | Low | 0.50 | Ambiguous |

## Invalid SSN Patterns

### Invalid Area Numbers
| Input | Should Match | Validation | Notes |
|-------|--------------|------------|-------|
| `000-12-3456` | Yes (detect) | Invalid | Area 000 never issued |
| `666-12-3456` | Yes (detect) | Invalid | Area 666 never issued |
| `900-12-3456` | Yes (detect) | Invalid | 9XX reserved for ITIN |
| `999-12-3456` | Yes (detect) | Invalid | 9XX reserved |

### Invalid Group/Serial Numbers
| Input | Should Match | Validation | Notes |
|-------|--------------|------------|-------|
| `123-00-3456` | Yes (detect) | Invalid | Group 00 never issued |
| `123-45-0000` | Yes (detect) | Invalid | Serial 0000 never issued |

### Known Test SSNs
| Input | Should Match | Validation | Notes |
|-------|--------------|------------|-------|
| `078-05-1120` | Yes (detect) | Test | Woolworth wallet SSN |
| `219-09-9999` | Yes (detect) | Test | Common test SSN |
| `987-65-4320` | Yes (detect) | Test | IRS test SSN |

## Edge Cases

### Should NOT Match
| Input | Notes |
|-------|-------|
| `12-345-6789` | Wrong grouping |
| `1234-56-789` | Wrong grouping |
| `123-456-789` | Wrong grouping |
| `12345678` | Only 8 digits |
| `1234567890` | 10 digits |
| `SSN-123-45-6789` | Preceded by letters |

### Should Match (with lower confidence)
| Input | Notes |
|-------|-------|
| `SSN: 1XX-XX-6789` | Partially masked |
| `***-**-6789` | Masked SSN |

## Context Patterns

### Labels That Boost Confidence
| Pattern | Category | Boost |
|---------|----------|-------|
| `SSN:` | ssn | +0.20 |
| `Social Security:` | ssn | +0.20 |
| `Social Security Number:` | ssn | +0.20 |
| `SS#:` | ssn | +0.15 |
| `Tax ID:` | ssn | +0.10 |
