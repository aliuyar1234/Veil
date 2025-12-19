//! Redaction helpers for log output.

use std::path::Path;

pub(crate) fn redact_path(path: &Path) -> String {
    redact_text(&path.to_string_lossy())
}

pub(crate) fn redact_text(text: &str) -> String {
    let mut redacted = text.to_string();
    for marker in ["\\Users\\", "\\users\\", "/Users/", "/users/", "/home/"] {
        redacted = redact_after_marker(&redacted, marker);
    }
    redacted
}

fn redact_after_marker(text: &str, marker: &str) -> String {
    let Some(start_idx) = text.find(marker) else {
        return text.to_string();
    };

    let name_start = start_idx + marker.len();
    let mut name_end = text.len();
    for (offset, ch) in text[name_start..].char_indices() {
        if ch == '\\' || ch == '/' {
            name_end = name_start + offset;
            break;
        }
    }

    let mut output = String::with_capacity(text.len());
    output.push_str(&text[..name_start]);
    output.push_str("[REDACTED]");
    output.push_str(&text[name_end..]);
    output
}
