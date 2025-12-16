//! Error types for Office document parsing.

use thiserror::Error;

/// Errors that can occur during Office document parsing.
#[derive(Debug, Error)]
pub enum OfficeError {
    /// The file is not a valid ZIP archive.
    #[error("file is not a valid ZIP archive: {0}")]
    NotZipArchive(String),

    /// The file is a ZIP but not a valid Office Open XML document.
    #[error("file is not a valid Office Open XML document: missing {0}")]
    NotOfficeOpenXml(String),

    /// The document is encrypted and cannot be processed.
    #[error("document is encrypted and cannot be processed without a password")]
    Encrypted,

    /// The file is a legacy Office format (.doc, .xls, .ppt).
    #[error("legacy Office format ({0}) is not supported; please convert to Office Open XML format (.docx, .xlsx, .pptx)")]
    LegacyFormat(String),

    /// The file format is not supported.
    #[error("unsupported Office format: {0}")]
    UnsupportedFormat(String),

    /// XML parsing error.
    #[error("XML parsing error: {0}")]
    XmlError(String),

    /// The document appears to be corrupted.
    #[error("document appears to be corrupted: {0}")]
    Corrupted(String),

    /// The file exceeds the maximum allowed size.
    #[error("file exceeds maximum allowed size of {max_mb}MB (actual: {actual_mb:.1}MB)")]
    FileTooLarge { max_mb: usize, actual_mb: f64 },

    /// Potential ZIP bomb detected (decompressed size greatly exceeds compressed size).
    #[error("potential ZIP bomb detected: compressed {compressed_mb:.1}MB would expand to {uncompressed_mb:.1}MB (ratio {ratio:.0}x exceeds limit of {max_ratio}x)")]
    ZipBomb {
        compressed_mb: f64,
        uncompressed_mb: f64,
        ratio: f64,
        max_ratio: usize,
    },

    /// Malicious path traversal attempt detected.
    #[error("malicious path traversal detected in archive entry: {0}")]
    PathTraversal(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// ZIP extraction error.
    #[error("ZIP extraction error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Calamine (Excel) error.
    #[error("Excel parsing error: {0}")]
    Calamine(#[from] calamine::Error),

    /// Quick-XML error.
    #[error("XML parsing error: {0}")]
    QuickXml(#[from] quick_xml::Error),
}

/// Result type for Office parsing operations.
pub type Result<T> = std::result::Result<T, OfficeError>;

impl From<quick_xml::events::attributes::AttrError> for OfficeError {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        OfficeError::XmlError(err.to_string())
    }
}
