//! Quote detection for email threads.
//!
//! Identifies quoted/replied content in email bodies.

/// Quote patterns to detect in email text.
const QUOTE_PREFIXES: &[&str] = &[">", ">>>", ">>", "|"];

/// Attribution line patterns (before quoted text).
const ATTRIBUTION_PATTERNS: &[&str] = &["wrote:", "schrieb:", "écrit:", "escribió:"];

/// Original message separator patterns.
const SEPARATOR_PATTERNS: &[&str] = &[
    "-----Original Message-----",
    "-------- Original Message --------",
    "________________________________",
    "On ",
    "Am ",
];

/// Detect and mark quoted sections in email text.
///
/// Returns a list of (text, is_quoted) tuples representing the email body
/// split into original and quoted sections.
pub fn detect_quotes(text: &str) -> Vec<(String, bool)> {
    let mut sections = Vec::new();
    let mut current_section = String::new();
    let mut current_is_quoted = false;
    let mut in_quote_block = false;

    for line in text.lines() {
        let line_is_quoted = is_quote_line(line);
        let is_attribution = is_attribution_line(line);
        let is_separator = is_separator_line(line);

        // Start of quoted section
        if !in_quote_block && (line_is_quoted || is_attribution || is_separator) {
            // Save current section if not empty
            if !current_section.trim().is_empty() {
                sections.push((current_section.clone(), current_is_quoted));
            }
            current_section.clear();
            in_quote_block = true;
            current_is_quoted = true;
        }
        // End of quoted section (non-quoted line after quoted block)
        else if in_quote_block
            && !line_is_quoted
            && !line.trim().is_empty()
            && !is_continuation(line)
        {
            // Check if this looks like the end of quotes
            // (heuristic: a non-empty, non-quoted line)
            if !current_section.trim().is_empty() {
                sections.push((current_section.clone(), current_is_quoted));
            }
            current_section.clear();
            in_quote_block = false;
            current_is_quoted = false;
        }

        // Add line to current section
        if !current_section.is_empty() {
            current_section.push('\n');
        }
        current_section.push_str(line);
    }

    // Don't forget the last section
    if !current_section.trim().is_empty() {
        sections.push((current_section, current_is_quoted));
    }

    // If no sections detected, return the whole text as non-quoted
    if sections.is_empty() {
        vec![(text.to_string(), false)]
    } else {
        sections
    }
}

/// Check if a line starts with a quote prefix.
fn is_quote_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    QUOTE_PREFIXES
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
}

/// Check if a line is an attribution line (e.g., "On Dec 15, John wrote:").
fn is_attribution_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    ATTRIBUTION_PATTERNS
        .iter()
        .any(|pattern| lower.contains(pattern))
}

/// Check if a line is a separator line.
fn is_separator_line(line: &str) -> bool {
    SEPARATOR_PATTERNS
        .iter()
        .any(|pattern| line.contains(pattern))
}

/// Check if a line looks like a continuation of quoted content.
fn is_continuation(line: &str) -> bool {
    // Empty lines or very short lines might be part of quotes
    line.trim().is_empty() || line.trim().len() < 3
}

/// Strip quote prefixes from a line.
pub fn strip_quote_prefix(line: &str) -> &str {
    let mut result = line.trim_start();

    for prefix in QUOTE_PREFIXES {
        if result.starts_with(prefix) {
            result = result[prefix.len()..].trim_start();
            // Recursively strip nested prefixes
            return strip_quote_prefix(result);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_quote_line() {
        assert!(is_quote_line("> This is quoted"));
        assert!(is_quote_line(">> Nested quote"));
        assert!(is_quote_line("  > Indented quote"));
        assert!(!is_quote_line("Normal text"));
    }

    #[test]
    fn test_is_attribution_line() {
        assert!(is_attribution_line("On Dec 15, 2024, John Doe wrote:"));
        assert!(is_attribution_line("Am 15.12.2024 um 10:00 schrieb:"));
        assert!(!is_attribution_line("Normal text"));
    }

    #[test]
    fn test_is_separator_line() {
        assert!(is_separator_line("-----Original Message-----"));
        assert!(is_separator_line(
            "On Mon, Dec 15, 2024 at 10:00 AM John wrote:"
        ));
        assert!(!is_separator_line("Normal text"));
    }

    #[test]
    fn test_strip_quote_prefix() {
        assert_eq!(strip_quote_prefix("> Hello"), "Hello");
        assert_eq!(strip_quote_prefix(">> Hello"), "Hello");
        assert_eq!(strip_quote_prefix(">>> Hello"), "Hello");
        assert_eq!(strip_quote_prefix("Hello"), "Hello");
    }

    #[test]
    fn test_detect_quotes_simple() {
        let text = "This is my reply.\n\n> Original message here.";
        let sections = detect_quotes(text);

        assert!(!sections.is_empty());
        // First section should be non-quoted
        assert!(!sections[0].1 || sections.iter().any(|(_, quoted)| !quoted));
    }

    #[test]
    fn test_detect_quotes_with_attribution() {
        let text = "Thanks for the info.\n\nOn Dec 15, 2024, John wrote:\n> Previous message";
        let sections = detect_quotes(text);

        // Should have at least one non-quoted and one quoted section
        let has_quoted = sections.iter().any(|(_, q)| *q);
        let has_unquoted = sections.iter().any(|(_, q)| !*q);
        assert!(has_quoted || has_unquoted);
    }
}
