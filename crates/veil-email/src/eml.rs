//! EML file parsing implementation using mailparse.

use mailparse::body::Body;
use mailparse::{parse_mail, MailHeaderMap, ParsedMail};

use crate::error::{EmailParseError, Result};
use crate::html::convert_html_to_text;
use crate::types::{
    EmailAddress, EmailAttachment, EmailBodyPart, EmailFormat, EmailHeader, EmailHeaderValue,
    EmailMessage, EmailParseOptions,
};

/// Parse an EML file from bytes.
pub fn parse_eml(data: &[u8], options: &EmailParseOptions) -> Result<EmailMessage> {
    let parsed = parse_mail(data)?;

    let mut message = EmailMessage::new(EmailFormat::Eml);
    message.size_bytes = data.len();

    // Extract headers
    message.headers = extract_headers(&parsed, options);

    // Extract body parts
    message.body_parts = extract_body_parts(&parsed, options)?;

    // Extract attachment metadata
    if options.extract_attachments {
        message.attachments = extract_attachments(&parsed, options);
    }

    Ok(message)
}

/// Extract headers from parsed email.
fn extract_headers(parsed: &ParsedMail, options: &EmailParseOptions) -> Vec<EmailHeader> {
    let mut headers = Vec::new();

    for header in parsed.headers.iter() {
        let name = header.get_key();
        let raw_value = header.get_value();

        let value = parse_header_value(&name, &raw_value);

        headers.push(EmailHeader {
            name,
            value,
            raw_value: if options.include_raw_headers {
                Some(raw_value)
            } else {
                None
            },
        });
    }

    headers
}

/// Parse a header value into the appropriate type.
fn parse_header_value(name: &str, value: &str) -> EmailHeaderValue {
    let name_lower = name.to_lowercase();

    match name_lower.as_str() {
        "from" | "sender" | "reply-to" => {
            // Single address header
            if let Some(addr) = parse_email_address(value) {
                EmailHeaderValue::Address(addr)
            } else {
                EmailHeaderValue::Unstructured(value.to_string())
            }
        }
        "to" | "cc" | "bcc" => {
            // Address list header
            let addresses = parse_address_list(value);
            if addresses.is_empty() {
                EmailHeaderValue::Unstructured(value.to_string())
            } else {
                EmailHeaderValue::AddressList(addresses)
            }
        }
        "date" => EmailHeaderValue::DateTime(value.to_string()),
        "subject" => EmailHeaderValue::Text(value.to_string()),
        "message-id" | "in-reply-to" | "references" => EmailHeaderValue::Text(value.to_string()),
        _ => EmailHeaderValue::Unstructured(value.to_string()),
    }
}

/// Parse a single email address from a header value.
fn parse_email_address(value: &str) -> Option<EmailAddress> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    // Check for "Display Name <email@example.com>" format
    if let Some(angle_start) = value.rfind('<') {
        if let Some(angle_end) = value.rfind('>') {
            if angle_start < angle_end {
                let address = value[angle_start + 1..angle_end].trim().to_string();
                let display_name = value[..angle_start].trim();

                // Remove quotes from display name
                let display_name = display_name.trim_matches('"').trim();

                if display_name.is_empty() {
                    return Some(EmailAddress::new(address));
                } else {
                    return Some(EmailAddress::with_name(display_name, address));
                }
            }
        }
    }

    // Plain email address
    if value.contains('@') {
        Some(EmailAddress::new(value.to_string()))
    } else {
        None
    }
}

/// Parse a list of email addresses from a header value.
fn parse_address_list(value: &str) -> Vec<EmailAddress> {
    let mut addresses = Vec::new();

    // Split by comma, but respect quoted strings and angle brackets
    let mut current = String::new();
    let mut in_quotes = false;
    let mut in_angles: i32 = 0;

    for ch in value.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            '<' => {
                in_angles += 1;
                current.push(ch);
            }
            '>' => {
                in_angles = in_angles.saturating_sub(1);
                current.push(ch);
            }
            ',' if !in_quotes && in_angles == 0 => {
                if let Some(addr) = parse_email_address(&current) {
                    addresses.push(addr);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    // Don't forget the last address
    if let Some(addr) = parse_email_address(&current) {
        addresses.push(addr);
    }

    addresses
}

/// Extract body parts from parsed email.
fn extract_body_parts(
    parsed: &ParsedMail,
    options: &EmailParseOptions,
) -> Result<Vec<EmailBodyPart>> {
    let mut parts = Vec::new();
    extract_body_parts_recursive(parsed, options, &mut parts)?;
    Ok(parts)
}

/// Recursively extract body parts from MIME structure.
fn extract_body_parts_recursive(
    parsed: &ParsedMail,
    options: &EmailParseOptions,
    parts: &mut Vec<EmailBodyPart>,
) -> Result<()> {
    let content_type = parsed.ctype.mimetype.to_lowercase();

    if content_type.starts_with("multipart/") {
        // Process subparts
        for subpart in &parsed.subparts {
            extract_body_parts_recursive(subpart, options, parts)?;
        }
    } else if content_type.starts_with("text/plain") {
        // Plain text part
        let body = parsed
            .get_body()
            .map_err(|e| EmailParseError::BodyParse(e.to_string()))?;
        let charset = parsed
            .ctype
            .params
            .get("charset")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "UTF-8".to_string());

        parts.push(EmailBodyPart {
            content_type: "text/plain".to_string(),
            charset,
            content: body,
            is_quoted: false,
            transfer_encoding: get_transfer_encoding(parsed),
        });
    } else if content_type.starts_with("text/html") {
        // HTML part
        let body = parsed
            .get_body()
            .map_err(|e| EmailParseError::BodyParse(e.to_string()))?;
        let charset = parsed
            .ctype
            .params
            .get("charset")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "UTF-8".to_string());

        let content = if options.convert_html {
            convert_html_to_text(&body)
        } else {
            body
        };

        parts.push(EmailBodyPart {
            content_type: "text/html".to_string(),
            charset,
            content,
            is_quoted: false,
            transfer_encoding: get_transfer_encoding(parsed),
        });
    }
    // Skip other content types (attachments, etc.)

    Ok(())
}

/// Get the transfer encoding from a parsed mail part.
fn get_transfer_encoding(parsed: &ParsedMail) -> Option<String> {
    parsed.headers.get_first_value("Content-Transfer-Encoding")
}

/// Extract attachment metadata from parsed email.
fn extract_attachments(parsed: &ParsedMail, options: &EmailParseOptions) -> Vec<EmailAttachment> {
    let mut attachments = Vec::new();
    extract_attachments_recursive(parsed, options, &mut attachments);
    attachments
}

/// Recursively extract attachments from MIME structure.
fn extract_attachments_recursive(
    parsed: &ParsedMail,
    options: &EmailParseOptions,
    attachments: &mut Vec<EmailAttachment>,
) {
    let content_type = parsed.ctype.mimetype.to_lowercase();

    // Check if this is an attachment
    let content_disposition = parsed
        .headers
        .get_first_value("Content-Disposition")
        .unwrap_or_default()
        .to_lowercase();

    let is_attachment = content_disposition.starts_with("attachment")
        || (content_disposition.starts_with("inline") && !content_type.starts_with("text/"));

    if is_attachment {
        // Get filename
        let filename = get_attachment_filename(parsed);

        // Get size without decoding full attachment bodies (avoid OOM/CPU for huge base64 parts).
        let size_bytes = estimate_attachment_size_bytes(parsed);

        // Check size limit
        if size_bytes <= options.max_attachment_size {
            let content_id = parsed
                .headers
                .get_first_value("Content-ID")
                .map(|s| s.trim_matches(|c| c == '<' || c == '>').to_string());

            let inline = content_disposition.starts_with("inline");

            attachments.push(EmailAttachment {
                filename,
                content_type: parsed.ctype.mimetype.clone(),
                size_bytes,
                content_id,
                inline,
            });
        }
    }

    // Process subparts
    for subpart in &parsed.subparts {
        extract_attachments_recursive(subpart, options, attachments);
    }
}

/// Estimate attachment size in bytes without decoding the attachment payload.
///
/// Prefers a declared `size=` parameter in `Content-Disposition` when present,
/// otherwise computes an upper bound based on the transfer encoding and raw body length.
fn estimate_attachment_size_bytes(parsed: &ParsedMail) -> usize {
    if let Some(disposition) = parsed.headers.get_first_value("Content-Disposition") {
        if let Some(size_str) = extract_param(&disposition, "size") {
            if let Ok(size) = size_str.trim().parse::<usize>() {
                return size;
            }
        }
    }

    approximate_decoded_body_size(parsed)
}

/// Upper bound for decoded body size based on transfer encoding.
fn approximate_decoded_body_size(parsed: &ParsedMail) -> usize {
    match parsed.get_body_encoded() {
        Body::Base64(body) => {
            // Base64 expands: decoded_len <= ceil(n/4)*3, where n is non-whitespace chars.
            let mut n = 0usize;
            for &b in body.get_raw() {
                if !b.is_ascii_whitespace() {
                    n += 1;
                }
            }
            n.div_ceil(4) * 3
        }
        Body::QuotedPrintable(body) => body.get_raw().len(), // decoded_len <= raw_len
        Body::SevenBit(body) => body.get_raw().len(),
        Body::EightBit(body) => body.get_raw().len(),
        Body::Binary(body) => body.get_raw().len(),
    }
}

/// Get attachment filename from headers.
fn get_attachment_filename(parsed: &ParsedMail) -> Option<String> {
    // Try Content-Disposition filename parameter
    if let Some(disposition) = parsed.headers.get_first_value("Content-Disposition") {
        if let Some(filename) = extract_param(&disposition, "filename") {
            return Some(filename);
        }
    }

    // Try Content-Type name parameter
    parsed.ctype.params.get("name").cloned()
}

/// Extract a parameter value from a header.
fn extract_param(header: &str, param_name: &str) -> Option<String> {
    let search = format!("{}=", param_name);
    if let Some(idx) = header.to_lowercase().find(&search) {
        let start = idx + search.len();
        let rest = &header[start..];

        // Check for quoted value
        if let Some(unquoted) = rest.strip_prefix('"') {
            if let Some(end) = unquoted.find('"') {
                return Some(unquoted[..end].to_string());
            }
        } else {
            // Unquoted value - ends at semicolon or end
            let end = rest.find(';').unwrap_or(rest.len());
            return Some(rest[..end].trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_email_address_simple() {
        let addr = parse_email_address("test@example.com").unwrap();
        assert_eq!(addr.address, "test@example.com");
        assert!(addr.display_name.is_none());
    }

    #[test]
    fn test_parse_email_address_with_name() {
        let addr = parse_email_address("John Doe <john@example.com>").unwrap();
        assert_eq!(addr.address, "john@example.com");
        assert_eq!(addr.display_name.as_deref(), Some("John Doe"));
    }

    #[test]
    fn test_parse_email_address_quoted_name() {
        let addr = parse_email_address("\"Doe, John\" <john@example.com>").unwrap();
        assert_eq!(addr.address, "john@example.com");
        assert_eq!(addr.display_name.as_deref(), Some("Doe, John"));
    }

    #[test]
    fn test_parse_address_list() {
        let list = parse_address_list("john@example.com, Jane Doe <jane@example.com>");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].address, "john@example.com");
        assert_eq!(list[1].address, "jane@example.com");
        assert_eq!(list[1].display_name.as_deref(), Some("Jane Doe"));
    }

    #[test]
    fn test_parse_simple_eml() {
        let eml = b"From: sender@example.com\r\n\
                    To: recipient@example.com\r\n\
                    Subject: Test email\r\n\
                    Date: Mon, 15 Dec 2024 10:00:00 +0000\r\n\
                    \r\n\
                    Hello, this is a test email.\r\n";

        let options = EmailParseOptions::default();
        let message = parse_eml(eml, &options).unwrap();

        assert_eq!(message.subject(), Some("Test email"));
        assert!(message.from().is_some());
        assert!(message.to().is_some());
        assert_eq!(message.body_parts.len(), 1);
        assert!(message.body_parts[0].content.contains("Hello"));
    }

    #[test]
    fn test_extract_param() {
        let header = "attachment; filename=\"report.pdf\"; size=12345";
        assert_eq!(
            extract_param(header, "filename"),
            Some("report.pdf".to_string())
        );

        let header2 = "attachment; filename=report.pdf";
        assert_eq!(
            extract_param(header2, "filename"),
            Some("report.pdf".to_string())
        );
    }
}
