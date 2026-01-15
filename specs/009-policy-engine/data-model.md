# Data Model: Policy Engine

**Feature**: 009-policy-engine
**Date**: 2025-12-15

## Entity Relationship Overview

```text
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│     Policy      │────▶│  DetectionRule  │     │  ProtectionRule │
│─────────────────│     │─────────────────│     │─────────────────│
│ version         │     │ types           │     │ types           │
│ name            │     │ confidence      │     │ action          │
│ locale          │     │ enabled         │     │ style           │
│ detection[]     │     └─────────────────┘     │ consistent      │
│ protection[]    │────────────────────────────▶│ key_ref         │
└─────────────────┘                             └────────┬────────┘
                                                         │
                                                         ▼
                                                ┌─────────────────┐
                                                │     KeyRef      │
                                                │─────────────────│
                                                │ scheme          │
                                                │ path            │
                                                │ resolve()       │
                                                └─────────────────┘
```

## Existing Types (in veil-policy)

### Policy (schema.rs)

```rust
/// A complete policy definition (EXISTS).
pub struct Policy {
    pub version: String,
    pub name: String,
    pub locale: Option<Locale>,
    pub detection: Vec<DetectionRule>,
    pub protection: Vec<ProtectionRule>,
}
```

### DetectionRule (rules.rs)

```rust
/// A rule for filtering detection results (EXISTS).
pub struct DetectionRule {
    pub types: Vec<PiiCategory>,
    pub confidence_threshold: f32,
    pub enabled: bool,
}
```

### ProtectionRule (rules.rs)

```rust
/// A rule for applying protection (EXISTS - NEEDS EXTENSION).
pub struct ProtectionRule {
    pub types: Vec<PiiCategory>,
    pub action: ProtectionAction,
    pub style: Option<String>,
    pub consistent: bool,
    // ADD: pub key_ref: Option<KeyRef>,
}
```

### ProtectionAction (rules.rs)

```rust
/// Action to take for protection (EXISTS).
pub enum ProtectionAction {
    Redact,
    Mask,
    Hash,
    Pseudonymize,
    Encrypt,
    Tokenize,
}
```

## New Types

### KeyRef

Key reference for encryption operations.

```rust
/// Reference to an encryption key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct KeyRef {
    /// URI scheme (env, file).
    scheme: KeyRefScheme,
    /// Path or variable name.
    path: String,
}

/// Key reference schemes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyRefScheme {
    /// Environment variable: env://VAR_NAME
    Env,
    /// File path: file:///path/to/key
    File,
}

impl KeyRef {
    /// Resolve the key reference to bytes.
    pub fn resolve(&self) -> Result<Vec<u8>, KeyRefError>;
}
```

**Validation Rules**:
- URI must start with `env://` or `file://`
- For `env://`: Variable name must be non-empty
- For `file://`: Path must be absolute

### KeyRefError

```rust
/// Errors resolving key references.
#[derive(Debug, Error)]
pub enum KeyRefError {
    #[error("Invalid key reference format: {0}")]
    InvalidFormat(String),

    #[error("Unknown scheme: {0}")]
    UnknownScheme(String),

    #[error("Environment variable not found: {0}")]
    EnvNotFound(String),

    #[error("Key file not found: {0}")]
    FileNotFound(String),

    #[error("Failed to read key file: {0}")]
    FileReadError(String),
}
```

### PolicyExecutor

High-level executor for policy-driven protection.

```rust
/// Executor for applying policies to content.
pub struct PolicyExecutor {
    /// Encryption key (resolved from policy).
    encryption_key: Option<Vec<u8>>,
    /// Token vault for tokenization.
    vault: Option<Arc<dyn TokenVault>>,
    /// Cache for consistent pseudonymization.
    pseudonym_cache: HashMap<String, String>,
    /// Locale for pseudonymization.
    locale: String,
}

impl PolicyExecutor {
    /// Create executor from policy.
    pub fn from_policy(policy: &Policy) -> Result<Self, PolicyError>;

    /// Process content with policy.
    pub fn process(&mut self, content: &str, policy: &Policy)
        -> Result<ProcessResult, PolicyError>;

    /// Apply protection to a single finding.
    pub fn protect_finding(&mut self, finding: &Finding, rule: &ProtectionRule)
        -> Result<ProtectedValue, PolicyError>;

    /// Clear pseudonym cache (for new document).
    pub fn clear_cache(&mut self);
}
```

### ProcessResult

Result of policy processing.

```rust
/// Result of processing content with a policy.
#[derive(Debug)]
pub struct ProcessResult {
    /// Protected content.
    pub content: String,
    /// Original findings that were processed.
    pub findings: Vec<Finding>,
    /// Protection actions applied.
    pub actions: Vec<AppliedAction>,
    /// Processing statistics.
    pub stats: ProcessStats,
}

/// A protection action that was applied.
#[derive(Debug)]
pub struct AppliedAction {
    /// Finding that was protected.
    pub finding_index: usize,
    /// Action that was applied.
    pub action: ProtectionAction,
    /// Original value.
    pub original: String,
    /// Protected value.
    pub protected: String,
}

/// Processing statistics.
#[derive(Debug, Default)]
pub struct ProcessStats {
    pub findings_detected: usize,
    pub findings_filtered: usize,
    pub findings_protected: usize,
    pub duration_ms: u64,
}
```

### ProtectedValue

Result of protecting a single value.

```rust
/// A protected value with metadata.
#[derive(Debug)]
pub struct ProtectedValue {
    /// The protected string.
    pub value: String,
    /// Action that was applied.
    pub action: ProtectionAction,
    /// Whether this was a cached result (consistent mode).
    pub cached: bool,
}
```

## Module Organization

```text
crates/veil-policy/src/
├── lib.rs           # Public exports
├── schema.rs        # Policy struct (EXISTS)
├── rules.rs         # DetectionRule, ProtectionRule (EXISTS - EXTEND)
├── loader.rs        # YAML loading (EXISTS)
├── validation.rs    # Policy validation (EXISTS)
├── defaults.rs      # Default policy (EXISTS)
├── locale.rs        # Locale handling (EXISTS)
├── error.rs         # PolicyError (EXISTS - EXTEND)
├── apply.rs         # apply_policy_to_findings (EXISTS)
├── keyref.rs        # NEW: KeyRef and resolution
├── executor.rs      # NEW: PolicyExecutor
└── protect.rs       # NEW: Protection dispatch
```

## State Transitions

### Policy Loading Flow

```text
YAML String ──parse──▶ Policy (unvalidated)
                            │
                            ▼
                     validate_policy()
                            │
               ┌────────────┴────────────┐
               ▼                         ▼
    PolicyValidationResult       PolicyExecutor::from_policy()
        (errors/warnings)               │
                                        ▼
                              Ready to process content
```

### Protection Flow

```text
Content + Policy ──detect──▶ Vec<Finding>
                                  │
                                  ▼
                    apply_policy_to_findings()
                                  │
                                  ▼
                         Filtered Findings
                                  │
                                  ▼
              ┌───────────────────┴───────────────────┐
              ▼                                       ▼
        For each Finding                        Build result
              │
              ▼
    match protection_rule.action {
        Redact => veil_redact
        Mask => veil_redact
        Hash => veil_crypto::hash
        Encrypt => veil_crypto::encrypt
        Pseudonymize => veil_crypto::pseudonymize
        Tokenize => veil_crypto::tokenize
    }
```
