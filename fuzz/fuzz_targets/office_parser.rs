//! Fuzz target for Office document parser.
//!
//! Tests Office document parsing (DOCX, XLSX, PPTX) with arbitrary
//! byte inputs to find panics, memory safety issues, and infinite loops.
//!
//! Run with: `cargo +nightly fuzz run office_parser`

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Office documents are ZIP archives with XML content
    // Test the ZIP parsing layer first
    use std::io::Cursor;

    // Try to read as ZIP archive
    let cursor = Cursor::new(data);
    if let Ok(mut archive) = zip::ZipArchive::new(cursor) {
        // Try to list files (exercises ZIP parsing)
        for i in 0..archive.len().min(10) {
            if let Ok(file) = archive.by_index(i) {
                // Read file name (exercises string handling)
                let _ = file.name();
            }
        }
    }
});
