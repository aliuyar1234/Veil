//! Core types for email parsing.

use serde::{Deserialize, Serialize};

/// Supported email file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmailFormat {
    /// RFC 5322 email format (.eml files).
    Eml,
    /// Microsoft Outlook MSG format (.msg files).
    Msg,
}

impl EmailFormat {
    /// Get the typical file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            EmailFormat::Eml => "eml",
            EmailFormat::Msg => "msg",
        }
    }
}

/// Options for configuring email parsing behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailParseOptions {
    /// Convert HTML body to plain text (default: true).
    pub convert_html: bool,
    /// Prefer plain text over HTML when both are available (default: true).
    pub prefer_plain_text: bool,
    /// Extract attachment metadata (default: true).
    pub extract_attachments: bool,
    /// Detect quoted/replied content in body (default: true).
    pub detect_quotes: bool,
    /// Maximum attachment size to process in bytes (default: 100MB).
    pub max_attachment_size: usize,
    /// Include raw header values in output (default: false).
    pub include_raw_headers: bool,
}

impl Default for EmailParseOptions {
    fn default() -> Self {
        Self {
            convert_html: true,
            prefer_plain_text: true,
            extract_attachments: true,
            detect_quotes: true,
            max_attachment_size: 100 * 1024 * 1024, // 100MB
            include_raw_headers: false,
        }
    }
}

/// A parsed email message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    /// Email headers (From, To, Subject, etc.).
    pub headers: Vec<EmailHeader>,
    /// Body parts (text and HTML content).
    pub body_parts: Vec<EmailBodyPart>,
    /// Attachment metadata (without content).
    pub attachments: Vec<EmailAttachment>,
    /// Original email format.
    pub format: EmailFormat,
    /// Size of the original email in bytes.
    pub size_bytes: usize,
}

impl EmailMessage {
    /// Create a new empty email message.
    pub fn new(format: EmailFormat) -> Self {
        Self {
            headers: Vec::new(),
            body_parts: Vec::new(),
            attachments: Vec::new(),
            format,
            size_bytes: 0,
        }
    }

    /// Get the From header value.
    pub fn from(&self) -> Option<&EmailHeader> {
        self.headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case("from"))
    }

    /// Get the To header value.
    pub fn to(&self) -> Option<&EmailHeader> {
        self.headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case("to"))
    }

    /// Get the Subject header value.
    pub fn subject(&self) -> Option<&str> {
        self.headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case("subject"))
            .and_then(|h| match &h.value {
                EmailHeaderValue::Text(s) => Some(s.as_str()),
                EmailHeaderValue::Unstructured(s) => Some(s.as_str()),
                _ => None,
            })
    }

    /// Get the Date header value.
    pub fn date(&self) -> Option<&str> {
        self.headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case("date"))
            .and_then(|h| match &h.value {
                EmailHeaderValue::DateTime(s) => Some(s.as_str()),
                EmailHeaderValue::Unstructured(s) => Some(s.as_str()),
                _ => None,
            })
    }

    /// Get the plain text body content.
    pub fn plain_text_body(&self) -> Option<&str> {
        self.body_parts
            .iter()
            .find(|p| p.content_type.starts_with("text/plain"))
            .map(|p| p.content.as_str())
    }

    /// Get the HTML body content.
    pub fn html_body(&self) -> Option<&str> {
        self.body_parts
            .iter()
            .find(|p| p.content_type.starts_with("text/html"))
            .map(|p| p.content.as_str())
    }
}

/// An email header with parsed value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailHeader {
    /// Header name (e.g., "From", "To", "Subject").
    pub name: String,
    /// Parsed header value.
    pub value: EmailHeaderValue,
    /// Raw header value (if requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_value: Option<String>,
}

/// Parsed email header value types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EmailHeaderValue {
    /// Single email address (e.g., From header).
    Address(EmailAddress),
    /// List of email addresses (e.g., To, CC headers).
    AddressList(Vec<EmailAddress>),
    /// Plain text value (e.g., Subject).
    Text(String),
    /// Date/time value (e.g., Date header).
    DateTime(String),
    /// Unstructured/unparsed value.
    Unstructured(String),
}

/// An email address with optional display name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    /// Display name (e.g., "John Doe").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Email address (e.g., "john@example.com").
    pub address: String,
}

impl EmailAddress {
    /// Create a new email address without display name.
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            display_name: None,
            address: address.into(),
        }
    }

    /// Create a new email address with display name.
    pub fn with_name(display_name: impl Into<String>, address: impl Into<String>) -> Self {
        Self {
            display_name: Some(display_name.into()),
            address: address.into(),
        }
    }
}

impl std::fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.display_name {
            Some(name) => write!(f, "{} <{}>", name, self.address),
            None => write!(f, "{}", self.address),
        }
    }
}

/// A body part of an email message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailBodyPart {
    /// MIME content type (e.g., "text/plain", "text/html").
    pub content_type: String,
    /// Character encoding (e.g., "UTF-8", "ISO-8859-1").
    pub charset: String,
    /// Decoded text content.
    pub content: String,
    /// Whether this part is quoted/replied content.
    pub is_quoted: bool,
    /// Original transfer encoding (e.g., "base64", "quoted-printable").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer_encoding: Option<String>,
}

impl EmailBodyPart {
    /// Create a new plain text body part.
    pub fn plain_text(content: impl Into<String>) -> Self {
        Self {
            content_type: "text/plain".to_string(),
            charset: "UTF-8".to_string(),
            content: content.into(),
            is_quoted: false,
            transfer_encoding: None,
        }
    }

    /// Create a new HTML body part.
    pub fn html(content: impl Into<String>) -> Self {
        Self {
            content_type: "text/html".to_string(),
            charset: "UTF-8".to_string(),
            content: content.into(),
            is_quoted: false,
            transfer_encoding: None,
        }
    }
}

/// Metadata about an email attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    /// Attachment filename.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// MIME content type.
    pub content_type: String,
    /// Size in bytes.
    pub size_bytes: usize,
    /// Content-ID for inline attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
    /// Whether this is an inline attachment (e.g., embedded image).
    pub inline: bool,
}

impl EmailAttachment {
    /// Create a new attachment metadata.
    pub fn new(content_type: impl Into<String>, size_bytes: usize) -> Self {
        Self {
            filename: None,
            content_type: content_type.into(),
            size_bytes,
            content_id: None,
            inline: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_address_display() {
        let addr = EmailAddress::new("test@example.com");
        assert_eq!(addr.to_string(), "test@example.com");

        let addr_with_name = EmailAddress::with_name("John Doe", "john@example.com");
        assert_eq!(addr_with_name.to_string(), "John Doe <john@example.com>");
    }

    #[test]
    fn test_email_format_extension() {
        assert_eq!(EmailFormat::Eml.extension(), "eml");
        assert_eq!(EmailFormat::Msg.extension(), "msg");
    }

    #[test]
    fn test_default_parse_options() {
        let opts = EmailParseOptions::default();
        assert!(opts.convert_html);
        assert!(opts.prefer_plain_text);
        assert!(opts.extract_attachments);
        assert!(opts.detect_quotes);
    }

    #[test]
    fn test_email_body_part_constructors() {
        let plain = EmailBodyPart::plain_text("Hello world");
        assert_eq!(plain.content_type, "text/plain");
        assert_eq!(plain.content, "Hello world");

        let html = EmailBodyPart::html("<p>Hello</p>");
        assert_eq!(html.content_type, "text/html");
    }
}
