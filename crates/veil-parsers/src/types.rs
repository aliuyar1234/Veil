//! Core parsing-related types.
//!
//! The actual type definitions live in the `veil-types` crate so downstream
//! crates (e.g. detection) can depend on the shared data model without pulling
//! in parser implementations.

pub use veil_types::*;
