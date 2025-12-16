//! Error types for email parsing.

use thiserror::Error;

/// Errors that can occur during email parsing.
#[derive(Error, Debug)]
pub enum EmailParseError {
    /// Invalid or malformed email format.
    #[error("Invalid email format: {0}")]
    InvalidFormat(String),

    /// Failed to parse email headers.
    #[error("Failed to parse headers: {0}")]
    HeaderParse(String),

    /// Failed to parse email body.
    #[error("Failed to parse body: {0}")]
    BodyParse(String),

    /// Character encoding error.
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// MIME parsing error.
    #[error("MIME error: {0}")]
    MimeError(String),

    /// Attachment processing error.
    #[error("Attachment error: {0}")]
    AttachmentError(String),

    /// MSG format parsing error.
    #[error("MSG format error: {0}")]
    MsgError(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Mailparse library error.
    #[error("Mailparse error: {0}")]
    Mailparse(String),
}

impl From<mailparse::MailParseError> for EmailParseError {
    fn from(err: mailparse::MailParseError) -> Self {
        EmailParseError::Mailparse(err.to_string())
    }
}

/// Result type for email parsing operations.
pub type Result<T> = std::result::Result<T, EmailParseError>;
