//! Fuzz target for YAML policy parser.
//!
//! Tests policy YAML parsing with arbitrary string inputs to find
//! panics, stack overflows, and parsing issues.
//!
//! Run with: `cargo +nightly fuzz run policy_yaml`

#![no_main]

use libfuzzer_sys::fuzz_target;
use veil_policy::Policy;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as YAML policy
    if let Ok(s) = std::str::from_utf8(data) {
        // Policy::from_yaml should handle malformed YAML gracefully
        let _ = Policy::from_yaml(s);
    }
});
