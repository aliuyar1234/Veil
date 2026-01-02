//! Email parsing library for Veil.
//!
//! This crate provides parsing for email messages in EML and MSG formats,
//! extracting text content suitable for PII detection.
//!
//! # Supported Formats
//!
//! - **EML**: RFC 5322 email format (`.eml` files)
//! - **MSG**: Microsoft Outlook message format (`.msg` files)
//!
//! # Example
//!
//! ```no_run
//! use veil_email::{parse_email, EmailParseOptions};
//!
//! let eml_data = std::fs::read("message.eml").unwrap();
//! let options = EmailParseOptions::default();
//! let message = parse_email(&eml_data, &options).unwrap();
//!
//! println!("Subject: {:?}", message.subject());
//! println!("From: {:?}", message.from());
//!
//! // Convert to veil-parsers format for PII detection
//! let segments = message.to_text_segments();
//! ```

pub mod convert;
pub mod eml;
pub mod error;
pub mod html;
pub mod msg;
pub mod quotes;
pub mod types;

pub use error::{EmailParseError, Result};
pub use types::{
    EmailAddress, EmailAttachment, EmailBodyPart, EmailFormat, EmailHeader, EmailHeaderValue,
    EmailMessage, EmailParseOptions,
};

use veil_types::{FileFormat, ParseResult};

/// Parse an email from bytes.
///
/// Automatically detects the format (EML or MSG) based on content.
pub fn parse_email(data: &[u8], options: &EmailParseOptions) -> Result<EmailMessage> {
    // Detect format
    let format = detect_email_format(data);

    match format {
        EmailFormat::Eml => eml::parse_eml(data, options),
        EmailFormat::Msg => msg::parse_msg(data, options),
    }
}

/// Parse an email and return a ParseResult for veil-parsers integration.
pub fn parse_email_to_result(data: &[u8], options: &EmailParseOptions) -> Result<ParseResult> {
    let start = std::time::Instant::now();
    let message = parse_email(data, options)?;
    let duration_ms = start.elapsed().as_millis() as u64;
    Ok(message.to_parse_result(duration_ms))
}

/// Detect the email format from file content.
fn detect_email_format(data: &[u8]) -> EmailFormat {
    // Check for MSG magic bytes (OLE Compound Document)
    // D0 CF 11 E0 A1 B1 1A E1
    if data.len() >= 8
        && data[0] == 0xD0
        && data[1] == 0xCF
        && data[2] == 0x11
        && data[3] == 0xE0
        && data[4] == 0xA1
        && data[5] == 0xB1
        && data[6] == 0x1A
        && data[7] == 0xE1
    {
        return EmailFormat::Msg;
    }

    // Default to EML (text-based format)
    EmailFormat::Eml
}

/// Detect email format from file extension.
pub fn detect_format_from_extension(filename: &str) -> Option<EmailFormat> {
    let ext = filename.rsplit('.').next()?.to_lowercase();
    match ext.as_str() {
        "eml" => Some(EmailFormat::Eml),
        "msg" => Some(EmailFormat::Msg),
        _ => None,
    }
}

/// Convert EmailFormat to veil-parsers FileFormat.
pub fn to_file_format(format: EmailFormat) -> FileFormat {
    match format {
        EmailFormat::Eml => FileFormat::Eml,
        EmailFormat::Msg => FileFormat::Msg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_eml_format() {
        let eml = b"From: test@example.com\r\nSubject: Test\r\n\r\nBody";
        assert_eq!(detect_email_format(eml), EmailFormat::Eml);
    }

    #[test]
    fn test_detect_msg_format() {
        // OLE Compound Document magic bytes
        let msg = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0x00, 0x00];
        assert_eq!(detect_email_format(&msg), EmailFormat::Msg);
    }

    #[test]
    fn test_detect_format_from_extension() {
        assert_eq!(
            detect_format_from_extension("message.eml"),
            Some(EmailFormat::Eml)
        );
        assert_eq!(
            detect_format_from_extension("message.MSG"),
            Some(EmailFormat::Msg)
        );
        assert_eq!(detect_format_from_extension("document.pdf"), None);
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
        let message = parse_email(eml, &options).unwrap();

        assert_eq!(message.subject(), Some("Test email"));
        assert!(message.from().is_some());
        assert!(message.to().is_some());
    }

    #[test]
    fn test_parse_to_result() {
        let eml = b"From: test@example.com\r\n\
                    Subject: Test\r\n\
                    \r\n\
                    Body text\r\n";

        let options = EmailParseOptions::default();
        let result = parse_email_to_result(eml, &options).unwrap();

        assert_eq!(result.metadata.format, FileFormat::Eml);
        assert!(!result.segments.is_empty());
    }
}
