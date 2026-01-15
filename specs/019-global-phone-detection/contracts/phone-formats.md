# Phone Format Test Cases

## US Phone Numbers (NANP)

### E.164 Format
| Input | Should Match | Notes |
|-------|--------------|-------|
| `+1 555 123 4567` | Yes | Standard E.164 |
| `+1-555-123-4567` | Yes | Hyphen separator |
| `+1.555.123.4567` | Yes | Dot separator |
| `+15551234567` | Yes | No separators |

### With Country Code Prefix
| Input | Should Match | Notes |
|-------|--------------|-------|
| `1-555-123-4567` | Yes | Leading 1 |
| `1 555 123 4567` | Yes | Space separator |
| `1.555.123.4567` | Yes | Dot separator |

### Parentheses Format
| Input | Should Match | Notes |
|-------|--------------|-------|
| `(555) 123-4567` | Yes | Standard US format |
| `(555)123-4567` | Yes | No space after paren |
| `(555) 123 4567` | Yes | Space separator |

### 10-Digit Local
| Input | Should Match | Notes |
|-------|--------------|-------|
| `555-123-4567` | Yes | Hyphen separator |
| `555.123.4567` | Yes | Dot separator |
| `555 123 4567` | Yes | Space separator |

### Toll-Free
| Input | Should Match | Notes |
|-------|--------------|-------|
| `1-800-555-1234` | Yes | 800 toll-free |
| `1-888-555-1234` | Yes | 888 toll-free |
| `1-877-555-1234` | Yes | 877 toll-free |
| `800-555-1234` | Yes | Without leading 1 |

## UK Phone Numbers

### E.164 Format
| Input | Should Match | Notes |
|-------|--------------|-------|
| `+44 20 7946 0958` | Yes | London landline |
| `+44 121 234 5678` | Yes | Birmingham |
| `+44 7911 123456` | Yes | Mobile |
| `+447911123456` | Yes | Mobile no spaces |

### Local Format
| Input | Should Match | Notes |
|-------|--------------|-------|
| `020 7946 0958` | Yes | London |
| `0121 234 5678` | Yes | Birmingham |
| `07911 123456` | Yes | Mobile |
| `07911-123456` | Yes | Hyphen separator |

## French Phone Numbers

### E.164 Format
| Input | Should Match | Notes |
|-------|--------------|-------|
| `+33 1 23 45 67 89` | Yes | Paris |
| `+33 6 12 34 56 78` | Yes | Mobile |
| `+33123456789` | Yes | No spaces |

### Local Format
| Input | Should Match | Notes |
|-------|--------------|-------|
| `01 23 45 67 89` | Yes | Paris |
| `06 12 34 56 78` | Yes | Mobile |

## International E.164 (Generic)

| Input | Should Match | Notes |
|-------|--------------|-------|
| `+81 3 1234 5678` | Yes | Japan |
| `+61 2 1234 5678` | Yes | Australia |
| `+91 98765 43210` | Yes | India |
| `+86 10 1234 5678` | Yes | China |
| `+7 495 123 4567` | Yes | Russia |
| `+55 11 1234 5678` | Yes | Brazil |

## DACH Region (Backward Compatibility)

| Input | Should Match | Notes |
|-------|--------------|-------|
| `+43 664 1234567` | Yes | Austria |
| `+49 89 12345678` | Yes | Germany |
| `+41 44 1234567` | Yes | Switzerland |
| `0043 664 1234567` | Yes | Austria with 00 |
| `0049 89 12345678` | Yes | Germany with 00 |
| `01/2345678` | Yes | Local Austrian |
| `089/12345678` | Yes | Local German |

## Edge Cases

### Should NOT Match
| Input | Notes |
|-------|-------|
| `123-4567` | Only 7 digits, no area code |
| `12-34-56` | Too short |
| `SKU-555-1234` | Preceded by letters |
| `Order #555-123-4567` | Number sign prefix |

### Should Match (with lower confidence)
| Input | Notes |
|-------|-------|
| `+1 555-123-4567 ext 890` | Main number only |
| `Call 555-123-4567 now` | Embedded in text |
