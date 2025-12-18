//! Fuzz target for PDF parser.
//!
//! Tests PDF text extraction with arbitrary byte inputs to find
//! panics, memory safety issues, and infinite loops.
//!
//! Run with: `cargo +nightly fuzz run pdf_parser`

#![no_main]

use libfuzzer_sys::fuzz_target;
use veil_parsers::PdfParser;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as PDF
    // The parser should gracefully handle malformed input
    let _ = PdfParser::extract_text_from_bytes(data);
});
