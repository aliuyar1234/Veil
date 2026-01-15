# Research: Policy Engine

**Feature**: 009-policy-engine
**Date**: 2025-12-15

## Existing Implementation Analysis

The `veil-policy` crate already has basic structure:
- Policy schema with version, name, locale, detection/protection rules
- YAML parsing via serde_yaml
- Basic validation
- `apply_policy_to_findings()` for detection filtering
- `get_redaction_config()` for redaction-only protection

## Missing Features (per spec)

1. **Key References** - FR-006: `env://`, `file://` URI schemes
2. **Full Protection Executor** - FR-003: Apply all actions (encrypt/hash/pseudonymize/tokenize)
3. **Consistent Pseudonymization** - FR-005: Track across document
4. **Policy Inheritance** - FR-008: Base policy + overrides (P3, defer)

## Decision 1: Key Reference Implementation

**Question**: How to implement `env://` and `file://` key references?

**Decision**: Create `KeyRef` struct with URI parsing and resolution

**Rationale**:
- Simple URI scheme parsing (split on `://`)
- Environment: `std::env::var()`
- File: `std::fs::read()`
- Return `Result<Vec<u8>, KeyRefError>` for key bytes

**Schema**:
```rust
pub struct KeyRef {
    scheme: KeyRefScheme,
    path: String,
}

pub enum KeyRefScheme {
    Env,   // env://VAR_NAME
    File,  // file:///path/to/key
}
```

## Decision 2: Protection Executor Architecture

**Question**: How to connect Policy to veil-crypto for full protection?

**Decision**: Create `PolicyExecutor` that orchestrates detection + protection

**Rationale**:
- Single entry point: `executor.process(content, policy)`
- Delegates to appropriate veil-crypto functions based on ProtectionAction
- Maintains consistency context for pseudonymization
- Returns protected content + audit trail

**Interface**:
```rust
pub struct PolicyExecutor {
    crypto_config: Option<CryptoConfig>,
    vault: Option<Arc<dyn TokenVault>>,
    pseudonym_cache: HashMap<String, String>,
}

impl PolicyExecutor {
    pub fn process(&mut self, content: &str, policy: &Policy) -> Result<ProcessResult, PolicyError>;
}
```

## Decision 3: Consistency Tracking

**Question**: How to track consistent pseudonymization?

**Decision**: Per-executor HashMap cache, keyed by original value

**Rationale**:
- Simple in-memory cache
- Executor lifetime = consistency scope
- Can be cleared between documents or kept for session
- Same approach used in veil-crypto pseudonymization

## Decision 4: Integration with Existing Crates

**Integration Map**:
```
Policy → DetectionRule → veil-detect (filter findings)
       → ProtectionRule → action switch:
           - Redact → veil-redact
           - Mask → veil-redact
           - Hash → veil-crypto::hash
           - Encrypt → veil-crypto::encrypt
           - Pseudonymize → veil-crypto::pseudonymize
           - Tokenize → veil-crypto::tokenize
```

## Decision 5: Error Handling for Key Resolution

**Question**: What happens when key reference fails?

**Decision**: Fail fast with clear error message

**Rationale**:
- Security: Don't silently skip encryption
- Clear error: "Key not found: env://VEIL_KEY"
- Validate key references at policy load time (optional)
- Fail at protection time if key missing

## Dependencies

| Crate | Already In | Purpose |
|-------|------------|---------|
| serde_yaml | ✅ Yes | YAML parsing |
| veil-detect | ✅ Yes | Finding filtering |
| veil-redact | ✅ Yes | Redaction |
| veil-crypto | ❌ Add | Encryption, hashing, pseudonymization, tokenization |

## Implementation Phases

1. **Phase 1**: Add KeyRef struct and resolution
2. **Phase 2**: Create PolicyExecutor with protection dispatch
3. **Phase 3**: Add consistency tracking for pseudonymization
4. **Phase 4**: Integration tests with full pipeline

## Deferred Features

- **FR-008 Policy Inheritance**: Complex, defer to future iteration
- **Policy hot-reload (FR-012)**: Requires file watcher, defer
