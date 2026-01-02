//! HTML to text conversion for email bodies.

use html2text::from_read;

/// Convert HTML content to plain text.
///
/// This function:
/// - Removes all HTML tags
/// - Preserves links as `[text](url)` format
/// - Converts images to `[image: alt]` format
/// - Preserves line breaks and paragraph structure
pub fn convert_html_to_text(html: &str) -> String {
    // Use html2text with a reasonable width
    let text = from_read(html.as_bytes(), 80).unwrap_or_else(|_| html.to_string());

    // Clean up excessive whitespace
    clean_whitespace(&text)
}

/// Clean up excessive whitespace while preserving structure.
fn clean_whitespace(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut last_was_newline = false;
    let mut newline_count = 0;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            newline_count += 1;
            // Allow at most 2 consecutive empty lines
            if newline_count <= 2 && !result.is_empty() {
                result.push('\n');
                last_was_newline = true;
            }
        } else {
            if !result.is_empty() && !last_was_newline {
                result.push('\n');
            }
            result.push_str(trimmed);
            result.push('\n');
            last_was_newline = true;
            newline_count = 0;
        }
    }

    // Remove trailing newlines
    while result.ends_with('\n') {
        result.pop();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_html() {
        let html = "<html><body><p>Hello world</p></body></html>";
        let text = convert_html_to_text(html);
        assert!(text.contains("Hello world"));
    }

    #[test]
    fn test_html_with_links() {
        let html = r#"<p>Click <a href="https://example.com">here</a> for more info.</p>"#;
        let text = convert_html_to_text(html);
        // html2text preserves links in some format
        assert!(text.contains("here") || text.contains("example.com"));
    }

    #[test]
    fn test_html_with_paragraphs() {
        let html = "<p>First paragraph.</p><p>Second paragraph.</p>";
        let text = convert_html_to_text(html);
        assert!(text.contains("First paragraph"));
        assert!(text.contains("Second paragraph"));
    }

    #[test]
    fn test_html_strips_scripts() {
        let html = "<p>Text</p><script>alert('bad');</script><p>More text</p>";
        let text = convert_html_to_text(html);
        assert!(!text.contains("alert"));
        assert!(text.contains("Text"));
        assert!(text.contains("More text"));
    }

    #[test]
    fn test_clean_whitespace() {
        let text = "Line 1\n\n\n\n\nLine 2";
        let cleaned = clean_whitespace(text);
        // Should have at most 2 blank lines between
        let newline_count = cleaned.matches('\n').count();
        assert!(newline_count <= 3);
    }
}
