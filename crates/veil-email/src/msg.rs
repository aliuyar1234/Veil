//! MSG file parsing implementation using msg_parser.

use msg_parser::Outlook;

use crate::error::{EmailParseError, Result};
use crate::html::convert_html_to_text;
use crate::quotes::detect_quotes;
use crate::types::{
    EmailAddress, EmailAttachment, EmailBodyPart, EmailFormat, EmailHeader, EmailHeaderValue,
    EmailMessage, EmailParseOptions,
};

/// Parse an MSG file from bytes.
pub fn parse_msg(data: &[u8], options: &EmailParseOptions) -> Result<EmailMessage> {
    // Parse using msg_parser
    let outlook = Outlook::from_slice(data)
        .map_err(|e| EmailParseError::MsgError(format!("Failed to parse MSG file: {}", e)))?;

    let mut message = EmailMessage::new(EmailFormat::Msg);
    message.size_bytes = data.len();

    // Extract headers
    message.headers = extract_msg_headers(&outlook, options);

    // Extract body parts
    message.body_parts = extract_msg_body(&outlook, options)?;

    // Extract attachment metadata
    if options.extract_attachments {
        message.attachments = extract_msg_attachments(&outlook, options);
    }

    Ok(message)
}

/// Extract headers from parsed MSG.
fn extract_msg_headers(outlook: &Outlook, options: &EmailParseOptions) -> Vec<EmailHeader> {
    let mut headers = Vec::new();

    // From/Sender - sender is a Person struct (not Option)
    let sender = &outlook.sender;
    if !sender.email.is_empty() || !sender.name.is_empty() {
        let address = EmailAddress {
            display_name: if sender.name.is_empty() {
                None
            } else {
                Some(sender.name.clone())
            },
            address: sender.email.clone(),
        };
        headers.push(EmailHeader {
            name: "From".to_string(),
            value: EmailHeaderValue::Address(address),
            raw_value: if options.include_raw_headers {
                Some(format!("{} <{}>", sender.name, sender.email))
            } else {
                None
            },
        });
    }

    // To recipients - to is Vec<Person>
    if !outlook.to.is_empty() {
        let addresses: Vec<EmailAddress> = outlook
            .to
            .iter()
            .map(|p| EmailAddress {
                display_name: if p.name.is_empty() {
                    None
                } else {
                    Some(p.name.clone())
                },
                address: p.email.clone(),
            })
            .collect();
        headers.push(EmailHeader {
            name: "To".to_string(),
            value: EmailHeaderValue::AddressList(addresses),
            raw_value: None,
        });
    }

    // CC recipients - cc is Vec<Person>
    if !outlook.cc.is_empty() {
        let addresses: Vec<EmailAddress> = outlook
            .cc
            .iter()
            .map(|p| EmailAddress {
                display_name: if p.name.is_empty() {
                    None
                } else {
                    Some(p.name.clone())
                },
                address: p.email.clone(),
            })
            .collect();
        headers.push(EmailHeader {
            name: "Cc".to_string(),
            value: EmailHeaderValue::AddressList(addresses),
            raw_value: None,
        });
    }

    // BCC - bcc is a String, parse it
    if !outlook.bcc.is_empty() {
        // BCC is stored as a string, create a single address from it
        headers.push(EmailHeader {
            name: "Bcc".to_string(),
            value: EmailHeaderValue::Text(outlook.bcc.clone()),
            raw_value: None,
        });
    }

    // Subject - subject is a String
    if !outlook.subject.is_empty() {
        headers.push(EmailHeader {
            name: "Subject".to_string(),
            value: EmailHeaderValue::Text(outlook.subject.clone()),
            raw_value: None,
        });
    }

    // Date - from headers struct (String, not Option)
    if !outlook.headers.date.is_empty() {
        headers.push(EmailHeader {
            name: "Date".to_string(),
            value: EmailHeaderValue::DateTime(outlook.headers.date.clone()),
            raw_value: None,
        });
    }

    // Message-ID - from headers struct (String, not Option)
    if !outlook.headers.message_id.is_empty() {
        headers.push(EmailHeader {
            name: "Message-ID".to_string(),
            value: EmailHeaderValue::Text(outlook.headers.message_id.clone()),
            raw_value: None,
        });
    }

    // Content-Type from headers (String, not Option)
    if !outlook.headers.content_type.is_empty() {
        headers.push(EmailHeader {
            name: "Content-Type".to_string(),
            value: EmailHeaderValue::Text(outlook.headers.content_type.clone()),
            raw_value: None,
        });
    }

    // Reply-To from headers (String, not Option)
    if !outlook.headers.reply_to.is_empty() {
        headers.push(EmailHeader {
            name: "Reply-To".to_string(),
            value: EmailHeaderValue::Text(outlook.headers.reply_to.clone()),
            raw_value: None,
        });
    }

    headers
}

/// Extract body parts from parsed MSG.
fn extract_msg_body(outlook: &Outlook, options: &EmailParseOptions) -> Result<Vec<EmailBodyPart>> {
    let mut body_parts = Vec::new();

    // Get body - body is a String (not Option)
    if !outlook.body.is_empty() {
        let content = outlook.body.clone();
        let is_quoted = if options.detect_quotes {
            // Check if majority of content is quoted
            let detected = detect_quotes(&content);
            detected.iter().filter(|s| s.1).count() > detected.len() / 2
        } else {
            false
        };

        // Check if body looks like HTML
        let content_type = if content.trim_start().starts_with('<')
            && (content.contains("<html") || content.contains("<body") || content.contains("<p>"))
        {
            if options.convert_html {
                // Convert HTML to plain text
                let plain_content = convert_html_to_text(&content);
                body_parts.push(EmailBodyPart {
                    content_type: "text/html".to_string(),
                    charset: "UTF-8".to_string(),
                    content: plain_content,
                    is_quoted,
                    transfer_encoding: None,
                });
                return Ok(body_parts);
            }
            "text/html"
        } else {
            "text/plain"
        };

        body_parts.push(EmailBodyPart {
            content_type: content_type.to_string(),
            charset: "UTF-8".to_string(),
            content,
            is_quoted,
            transfer_encoding: None,
        });
    }

    Ok(body_parts)
}

/// Extract attachment metadata from parsed MSG.
fn extract_msg_attachments(outlook: &Outlook, options: &EmailParseOptions) -> Vec<EmailAttachment> {
    outlook
        .attachments
        .iter()
        .filter_map(|att| {
            // payload is a String containing the attachment data
            let size = att.payload.len();

            // Skip attachments larger than max size
            if size > options.max_attachment_size {
                return None;
            }

            Some(EmailAttachment {
                filename: if att.file_name.is_empty() {
                    None
                } else {
                    Some(att.file_name.clone())
                },
                content_type: if att.mime_tag.is_empty() {
                    "application/octet-stream".to_string()
                } else {
                    att.mime_tag.clone()
                },
                size_bytes: size,
                content_id: None, // msg_parser doesn't expose content_id
                inline: false,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_msg_invalid_data() {
        let invalid_data = b"not a valid MSG file";
        let options = EmailParseOptions::default();
        let result = parse_msg(invalid_data, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_msg_empty_data() {
        let empty_data = &[];
        let options = EmailParseOptions::default();
        let result = parse_msg(empty_data, &options);
        assert!(result.is_err());
    }

    // Note: Real MSG file tests would require actual MSG test fixtures
}
