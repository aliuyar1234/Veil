//! # veil-policy
//!
//! YAML-based policy engine for PII detection and protection.
//!
//! ## Features
//!
//! - YAML-based policy configuration
//! - Detection rule filtering by PII type and confidence
//! - Protection actions: redact, mask, hash, encrypt, pseudonymize, tokenize
//! - Key references for external encryption keys (env://, file://)
//! - Consistent pseudonymization across documents
//!
//! ## Example
//!
//! ```rust,ignore
//! use veil_policy::{load_policy, PolicyExecutor};
//!
//! let policy = load_policy("policy.yaml")?;
//! let mut executor = PolicyExecutor::from_policy(&policy)?;
//! let result = executor.process(content, findings, &policy)?;
//! ```

mod apply;
mod defaults;
mod error;
mod executor;
mod keyref;
mod loader;
mod locale;
mod protect;
mod rules;
mod schema;
mod validation;

pub use apply::{apply_policy_to_findings, get_redaction_config};
pub use defaults::default_policy;
pub use error::PolicyError;
pub use executor::{AppliedAction, PolicyExecutor, ProcessResult, ProcessStats};
pub use keyref::{KeyRef, KeyRefError, KeyRefScheme};
pub use loader::{load_policy, parse_policy, validate_policy};
pub use locale::Locale;
pub use protect::{protect_value, ProtectedValue, ProtectionContext};
pub use rules::{DetectionRule, ProtectionAction, ProtectionRule};
pub use schema::Policy;
pub use validation::PolicyValidationResult;
