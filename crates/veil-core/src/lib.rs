//! # veil-core
//!
//! Core types for the Veil PII detection toolkit.
//!
//! This crate provides shared types used across the Veil workspace,
//! including `SensitiveString` for secure handling of PII data.

mod constants;
mod sensitive;

pub use constants::*;
pub use sensitive::SensitiveString;
