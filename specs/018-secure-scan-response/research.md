# Research: Secure Scan Response

## Decision Log

### 1. Response Field Handling

**Decision**: Use `Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]` for the value field.

**Rationale**: This is the idiomatic Rust/serde approach for conditionally omitting fields. When `None`, the field is completely absent from JSON output rather than being `null`.

**Alternatives Considered**:
- Separate response structs (SecureFinding vs Finding): Rejected - code duplication
- Always include null value: Rejected - still reveals field existence, wastes bandwidth

### 2. Acknowledgment Header Name

**Decision**: Use `X-Acknowledge-PII-Exposure` with value `accepted`.

**Rationale**:
- `X-` prefix indicates non-standard header (appropriate for app-specific security)
- Descriptive name makes purpose clear in logs/documentation
- Exact value match prevents accidental acknowledgment

**Alternatives Considered**:
- `Authorization` header: Rejected - semantically incorrect, not about auth
- Query parameter: Rejected - appears in URL logs, less secure
- Request body field: Rejected - changes content structure

### 3. CLI Confirmation Approach

**Decision**: Interactive prompt using `dialoguer` or stdin confirmation, with `--yes`/`-y` bypass for scripted use with explicit warning.

**Rationale**: Interactive confirmation ensures user awareness. The `--yes` bypass allows scripted use but makes the security trade-off explicit.

**Alternatives Considered**:
- Always require typing "yes": Rejected - too cumbersome for legitimate batch use
- Environment variable bypass: Rejected - too easy to set globally and forget
- No bypass: Rejected - blocks legitimate automation use cases

### 4. WASM Acknowledgment Pattern

**Decision**: Require `{ includeValues: true, acknowledgeExposure: true }` in options.

**Rationale**: JavaScript convention uses camelCase. Requiring both flags ensures deliberate opt-in.

**Alternatives Considered**:
- Single flag: Rejected - too easy to accidentally enable
- Callback confirmation: Rejected - complicates API unnecessarily

### 5. Error Response Format

**Decision**: Return HTTP 400 with JSON body explaining the security requirement.

**Rationale**: 400 Bad Request is semantically correct for invalid request parameters. JSON body allows programmatic handling.

**Example Response**:
```json
{
  "error": "pii_exposure_not_acknowledged",
  "message": "include_values=true requires X-Acknowledge-PII-Exposure: accepted header",
  "docs": "https://docs.veil.io/api/security#pii-exposure"
}
```

### 6. Breaking Change Strategy

**Decision**: Direct breaking change with clear documentation and CHANGELOG entry.

**Rationale**:
- The old behavior was a security vulnerability
- Gradual deprecation would extend the exposure window
- Enterprise users need immediate protection

**Migration Path**:
1. Update to new version
2. Scan results no longer include `value` field
3. If values needed: add header + parameter
4. Update client code to handle optional field

## Existing Code Analysis

### veil-api (scan.rs)
- Line 78: `value: detection.matched_text.clone()` - CHANGE to conditional
- `ScanOptions` struct needs `include_values` field
- Need to extract and validate header in handler

### veil-cli (scan.rs)
- Line 138: `text: f.matched_text.clone()` - CHANGE to conditional
- `FindingOutput` struct needs optional `text` field
- `ScanArgs` needs `--include-values` flag

### veil-wasm (types.rs, scan.rs)
- Line 15: `pub value: String` - CHANGE to `Option<String>`
- Line 139: `finding.matched_text.clone()` - CHANGE to conditional
- `ScanOptions` needs `includeValues` and `acknowledgeExposure` fields

## Security Considerations

1. **No PII in Error Messages**: Error messages must not include the attempted PII value
2. **No PII in Logs**: Server logs must not include matched_text even during debugging
3. **Header Case Sensitivity**: HTTP headers are case-insensitive; handle accordingly
4. **Timing Attacks**: Acknowledgment validation should be constant-time (though not critical here)
