//! HTML parser.

use std::time::Instant;

use scraper::{Html, Selector};

use crate::error::ParseError;
use crate::types::{
    DocumentMetadata, FileFormat, ParseOptions, ParseResult, Position, TextSegment,
};

/// Parser for HTML files.
pub struct HtmlParser;

impl HtmlParser {
    /// Create a new HTML parser.
    pub fn new() -> Self {
        Self
    }

    /// Parse bytes into a ParseResult.
    pub fn parse_bytes(
        &self,
        bytes: &[u8],
        _options: &ParseOptions,
    ) -> Result<ParseResult, ParseError> {
        let start = Instant::now();

        let content = String::from_utf8_lossy(bytes);
        let document = Html::parse_document(&content);

        let mut segments = Vec::new();
        let mut byte_offset = 0;

        // Select body or root if no body
        let body_selector = Selector::parse("body").unwrap();
        let root_elements = document.select(&body_selector);

        // Selectors for elements to exclude
        let exclude_tags = ["script", "style", "noscript", "template"];

        for element in root_elements {
            extract_text_from_element(&element, &exclude_tags, &mut segments, &mut byte_offset);
        }

        // If no body found, try the whole document
        if segments.is_empty() {
            let root = document.root_element();
            extract_text_from_element(&root, &exclude_tags, &mut segments, &mut byte_offset);
        }

        let total_chars = segments.iter().map(|s| s.content.len()).sum();

        Ok(ParseResult {
            metadata: DocumentMetadata {
                format: FileFormat::Html,
                encoding: "UTF-8".to_string(),
                size_bytes: Some(bytes.len()),
                filename: None,
                encoding_lossy: false,
            },
            segments,
            warnings: Vec::new(),
            total_chars,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract visible text from an HTML element, excluding certain tags.
fn extract_text_from_element(
    element: &scraper::ElementRef,
    exclude_tags: &[&str],
    segments: &mut Vec<TextSegment>,
    byte_offset: &mut usize,
) {
    for child in element.children() {
        match child.value() {
            scraper::node::Node::Element(el) => {
                let tag_name = el.name();

                // Skip excluded tags
                if exclude_tags.contains(&tag_name) {
                    continue;
                }

                // Recurse into child elements
                if let Some(child_ref) = scraper::ElementRef::wrap(child) {
                    extract_text_from_element(&child_ref, exclude_tags, segments, byte_offset);
                }
            }
            scraper::node::Node::Text(text) => {
                let content = text.trim();
                if !content.is_empty() {
                    let len = content.len();
                    segments.push(TextSegment {
                        content: content.to_string().into(),
                        position: Position::Html {
                            byte_offset: *byte_offset,
                            byte_length: len,
                        },
                    });
                    *byte_offset += len;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let parser = HtmlParser::new();
        let content = b"<html><body><p>Hello World</p></body></html>";
        let result = parser
            .parse_bytes(content, &ParseOptions::default())
            .unwrap();

        assert!(!result.segments.is_empty());
        let all_text: String = result
            .segments
            .iter()
            .map(|s| s.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(all_text.contains("Hello World"));
    }

    #[test]
    fn test_exclude_scripts() {
        let parser = HtmlParser::new();
        let content = b"<html><body><p>Visible</p><script>hidden()</script></body></html>";
        let result = parser
            .parse_bytes(content, &ParseOptions::default())
            .unwrap();

        let all_text: String = result
            .segments
            .iter()
            .map(|s| s.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        assert!(all_text.contains("Visible"));
        assert!(!all_text.contains("hidden"));
    }

    #[test]
    fn test_entity_decoding() {
        let parser = HtmlParser::new();
        let content = b"<html><body><p>A &amp; B</p></body></html>";
        let result = parser
            .parse_bytes(content, &ParseOptions::default())
            .unwrap();

        let all_text: String = result
            .segments
            .iter()
            .map(|s| s.content.as_str())
            .collect::<Vec<_>>()
            .join("");

        // scraper/html5ever automatically decodes entities
        assert!(all_text.contains("A & B") || all_text.contains("A &amp; B"));
    }
}
