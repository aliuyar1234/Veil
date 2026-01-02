//! # veil-audit
//!
//! Audit logging with hash chain for PII operations.

mod checksum;
mod encrypted;
mod entry;
mod error;
mod logger;
mod operation;
mod summary;

pub use checksum::{
    calculate_checksum, verify_chain, verify_chain_from_env, verify_chain_with_key,
};
pub use encrypted::{EncryptedAuditLogger, EncryptionConfig};
pub use entry::{AuditEntry, AuditOutcome, AuditParameters};
pub use error::AuditError;
pub use logger::{AuditFilter, AuditLogger};
pub use operation::AuditOperation;
pub use summary::{FindingsSummary, RedactionsSummary};
