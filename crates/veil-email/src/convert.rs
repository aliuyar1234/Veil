//! Conversion utilities for veil-parsers integration.

use veil_parsers::{
    DocumentMetadata, FileFormat, ParseResult, ParseWarning, Position, TextSegment,
};

use crate::types::{EmailBodyPart, EmailHeader, EmailHeaderValue, EmailMessage};

impl EmailMessage {
    /// Convert the email message to TextSegments for PII detection.
    pub fn to_text_segments(&self) -> Vec<TextSegment> {
        let mut segments = Vec::new();

        // Convert headers to segments
        for (idx, header) in self.headers.iter().enumerate() {
            segments.extend(header.to_text_segments(idx));
        }

        // Convert body parts to segments
        for (idx, part) in self.body_parts.iter().enumerate() {
            segments.extend(part.to_text_segments(idx));
        }

        // Convert attachment filenames to segments
        for attachment in &self.attachments {
            if let Some(filename) = &attachment.filename {
                segments.push(TextSegment {
                    content: filename.clone().into(),
                    position: Position::Email {
                        field: "attachment_filename".to_string(),
                        field_index: None,
                        part_index: None,
                        byte_offset: 0,
                        byte_length: filename.len(),
                    },
                });
            }
        }

        segments
    }

    /// Convert the email message to a ParseResult.
    pub fn to_parse_result(&self, duration_ms: u64) -> ParseResult {
        let segments = self.to_text_segments();
        let total_chars = segments.iter().map(|s| s.content.len()).sum();

        ParseResult {
            metadata: DocumentMetadata {
                format: match self.format {
                    crate::types::EmailFormat::Eml => FileFormat::Eml,
                    crate::types::EmailFormat::Msg => FileFormat::Msg,
                },
                encoding: "UTF-8".to_string(),
                size_bytes: Some(self.size_bytes),
                filename: None,
                encoding_lossy: false,
            },
            segments,
            warnings: Vec::new(),
            total_chars,
            duration_ms,
        }
    }

    /// Convert to ParseResult with warnings.
    pub fn to_parse_result_with_warnings(
        &self,
        duration_ms: u64,
        warnings: Vec<ParseWarning>,
    ) -> ParseResult {
        let mut result = self.to_parse_result(duration_ms);
        result.warnings = warnings;
        result
    }
}

impl EmailHeader {
    /// Convert header to text segments.
    pub fn to_text_segments(&self, field_index: usize) -> Vec<TextSegment> {
        let mut segments = Vec::new();

        match &self.value {
            EmailHeaderValue::Address(addr) => {
                // Add display name if present
                if let Some(name) = &addr.display_name {
                    segments.push(TextSegment {
                        content: name.clone().into(),
                        position: Position::Email {
                            field: format!("{}_name", self.name.to_lowercase()),
                            field_index: Some(field_index),
                            part_index: None,
                            byte_offset: 0,
                            byte_length: name.len(),
                        },
                    });
                }

                // Add email address
                segments.push(TextSegment {
                    content: addr.address.clone().into(),
                    position: Position::Email {
                        field: format!("{}_address", self.name.to_lowercase()),
                        field_index: Some(field_index),
                        part_index: None,
                        byte_offset: 0,
                        byte_length: addr.address.len(),
                    },
                });
            }
            EmailHeaderValue::AddressList(addrs) => {
                for (i, addr) in addrs.iter().enumerate() {
                    // Add display name if present
                    if let Some(name) = &addr.display_name {
                        segments.push(TextSegment {
                            content: name.clone().into(),
                            position: Position::Email {
                                field: format!("{}_name", self.name.to_lowercase()),
                                field_index: Some(field_index),
                                part_index: Some(i),
                                byte_offset: 0,
                                byte_length: name.len(),
                            },
                        });
                    }

                    // Add email address
                    segments.push(TextSegment {
                        content: addr.address.clone().into(),
                        position: Position::Email {
                            field: format!("{}_address", self.name.to_lowercase()),
                            field_index: Some(field_index),
                            part_index: Some(i),
                            byte_offset: 0,
                            byte_length: addr.address.len(),
                        },
                    });
                }
            }
            EmailHeaderValue::Text(text) | EmailHeaderValue::Unstructured(text) => {
                if !text.is_empty() {
                    segments.push(TextSegment {
                        content: text.clone().into(),
                        position: Position::Email {
                            field: self.name.to_lowercase(),
                            field_index: Some(field_index),
                            part_index: None,
                            byte_offset: 0,
                            byte_length: text.len(),
                        },
                    });
                }
            }
            EmailHeaderValue::DateTime(dt) => {
                segments.push(TextSegment {
                    content: dt.clone().into(),
                    position: Position::Email {
                        field: self.name.to_lowercase(),
                        field_index: Some(field_index),
                        part_index: None,
                        byte_offset: 0,
                        byte_length: dt.len(),
                    },
                });
            }
        }

        segments
    }
}

impl EmailBodyPart {
    /// Convert body part to text segments.
    pub fn to_text_segments(&self, part_index: usize) -> Vec<TextSegment> {
        if self.content.is_empty() {
            return Vec::new();
        }

        let field = if self.is_quoted {
            "body_quoted"
        } else {
            "body"
        };

        vec![TextSegment {
            content: self.content.clone().into(),
            position: Position::Email {
                field: field.to_string(),
                field_index: None,
                part_index: Some(part_index),
                byte_offset: 0,
                byte_length: self.content.len(),
            },
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EmailAddress, EmailFormat};

    #[test]
    fn test_header_to_segments_address() {
        let header = EmailHeader {
            name: "From".to_string(),
            value: EmailHeaderValue::Address(EmailAddress::with_name(
                "John Doe",
                "john@example.com",
            )),
            raw_value: None,
        };

        let segments = header.to_text_segments(0);
        assert_eq!(segments.len(), 2);
        assert!(segments.iter().any(|s| s.content.as_str() == "John Doe"));
        assert!(segments.iter().any(|s| s.content.as_str() == "john@example.com"));
    }

    #[test]
    fn test_header_to_segments_text() {
        let header = EmailHeader {
            name: "Subject".to_string(),
            value: EmailHeaderValue::Text("Test Subject".to_string()),
            raw_value: None,
        };

        let segments = header.to_text_segments(0);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].content.as_str(), "Test Subject");
    }

    #[test]
    fn test_body_part_to_segments() {
        let part = EmailBodyPart::plain_text("Hello world");
        let segments = part.to_text_segments(0);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].content.as_str(), "Hello world");
    }

    #[test]
    fn test_message_to_segments() {
        let mut message = EmailMessage::new(EmailFormat::Eml);
        message.headers.push(EmailHeader {
            name: "Subject".to_string(),
            value: EmailHeaderValue::Text("Test".to_string()),
            raw_value: None,
        });
        message
            .body_parts
            .push(EmailBodyPart::plain_text("Body text"));

        let segments = message.to_text_segments();
        assert!(segments.len() >= 2);
    }
}
