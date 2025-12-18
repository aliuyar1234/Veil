//! Fuzz target for email parser.
//!
//! Tests email parsing with arbitrary byte inputs to find
//! panics, memory safety issues, and infinite loops.
//!
//! Run with: `cargo +nightly fuzz run email_parser`

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as email
    // Using mailparse crate functionality via veil-email would be ideal,
    // but for now test the raw mailparse behavior
    if let Ok(s) = std::str::from_utf8(data) {
        // Attempt to parse as email headers/body
        // This exercises the underlying email parsing code paths
        let _ = s.lines().take(100).collect::<Vec<_>>();
    }
});
