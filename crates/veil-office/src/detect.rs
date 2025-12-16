//! Office format detection utilities.

use std::io::{Read, Seek};
use zip::ZipArchive;

use crate::error::{OfficeError, Result};

/// Detected Office document type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficeType {
    /// Word document (.docx)
    Docx,
    /// Excel spreadsheet (.xlsx)
    Xlsx,
    /// PowerPoint presentation (.pptx)
    Pptx,
}

impl OfficeType {
    /// Get the file extension for this Office type.
    pub fn extension(&self) -> &'static str {
        match self {
            OfficeType::Docx => "docx",
            OfficeType::Xlsx => "xlsx",
            OfficeType::Pptx => "pptx",
        }
    }

    /// Get a human-readable name for this Office type.
    pub fn name(&self) -> &'static str {
        match self {
            OfficeType::Docx => "Word Document",
            OfficeType::Xlsx => "Excel Spreadsheet",
            OfficeType::Pptx => "PowerPoint Presentation",
        }
    }
}

/// OLE2 compound file magic bytes (legacy Office formats).
const OLE2_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// ZIP file magic bytes.
const ZIP_MAGIC: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];

/// Check if a file is a legacy OLE2 format (.doc, .xls, .ppt).
pub fn is_legacy_format(data: &[u8]) -> bool {
    data.len() >= 8 && data[0..8] == OLE2_MAGIC
}

/// Check if data starts with ZIP magic bytes.
pub fn is_zip_file(data: &[u8]) -> bool {
    data.len() >= 4 && data[0..4] == ZIP_MAGIC
}

/// Detect the Office document type from a ZIP archive.
///
/// Returns the detected type or an error if the file is not a valid Office document.
pub fn detect_office_type<R: Read + Seek>(reader: R) -> Result<OfficeType> {
    let mut archive = ZipArchive::new(reader)?;

    // Check for [Content_Types].xml (required for all OOXML)
    if archive.by_name("[Content_Types].xml").is_err() {
        return Err(OfficeError::NotOfficeOpenXml(
            "[Content_Types].xml".to_string(),
        ));
    }

    // Check for format-specific indicators
    let has_word = archive.by_name("word/document.xml").is_ok();
    let has_excel = archive.by_name("xl/workbook.xml").is_ok();
    let has_pptx = archive.by_name("ppt/presentation.xml").is_ok();

    match (has_word, has_excel, has_pptx) {
        (true, false, false) => Ok(OfficeType::Docx),
        (false, true, false) => Ok(OfficeType::Xlsx),
        (false, false, true) => Ok(OfficeType::Pptx),
        (false, false, false) => Err(OfficeError::NotOfficeOpenXml(
            "word/document.xml, xl/workbook.xml, or ppt/presentation.xml".to_string(),
        )),
        _ => Err(OfficeError::UnsupportedFormat(
            "hybrid Office document with multiple content types".to_string(),
        )),
    }
}

/// Check if an Office document is encrypted.
///
/// Encrypted OOXML documents have a different structure with EncryptedPackage.
pub fn is_encrypted<R: Read + Seek>(reader: R) -> Result<bool> {
    let mut archive = match ZipArchive::new(reader) {
        Ok(a) => a,
        Err(_) => return Ok(false), // Not a valid ZIP, let other code handle the error
    };

    // Check for encryption indicators
    let has_encrypted_package = archive.by_name("EncryptedPackage").is_ok();
    let has_encryption_info = archive.by_name("EncryptionInfo").is_ok();

    Ok(has_encrypted_package || has_encryption_info)
}

/// Detect legacy format type from filename extension.
pub fn detect_legacy_type(filename: &str) -> Option<&'static str> {
    let lower = filename.to_lowercase();
    if lower.ends_with(".doc") {
        Some("DOC (Word 97-2003)")
    } else if lower.ends_with(".xls") {
        Some("XLS (Excel 97-2003)")
    } else if lower.ends_with(".ppt") {
        Some("PPT (PowerPoint 97-2003)")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_legacy_format() {
        // OLE2 magic bytes
        let ole2_data = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0x00, 0x00];
        assert!(is_legacy_format(&ole2_data));

        // ZIP magic bytes (not legacy)
        let zip_data = [0x50, 0x4B, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00];
        assert!(!is_legacy_format(&zip_data));

        // Random data
        let random_data = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        assert!(!is_legacy_format(&random_data));
    }

    #[test]
    fn test_is_zip_file() {
        let zip_data = [0x50, 0x4B, 0x03, 0x04, 0x00, 0x00];
        assert!(is_zip_file(&zip_data));

        let not_zip = [0x00, 0x01, 0x02, 0x03];
        assert!(!is_zip_file(&not_zip));
    }

    #[test]
    fn test_detect_legacy_type() {
        assert_eq!(detect_legacy_type("report.doc"), Some("DOC (Word 97-2003)"));
        assert_eq!(detect_legacy_type("data.xls"), Some("XLS (Excel 97-2003)"));
        assert_eq!(
            detect_legacy_type("slides.ppt"),
            Some("PPT (PowerPoint 97-2003)")
        );
        assert_eq!(detect_legacy_type("document.docx"), None);
        assert_eq!(detect_legacy_type("spreadsheet.xlsx"), None);
    }

    #[test]
    fn test_office_type_metadata() {
        assert_eq!(OfficeType::Docx.extension(), "docx");
        assert_eq!(OfficeType::Xlsx.extension(), "xlsx");
        assert_eq!(OfficeType::Pptx.extension(), "pptx");

        assert_eq!(OfficeType::Docx.name(), "Word Document");
        assert_eq!(OfficeType::Xlsx.name(), "Excel Spreadsheet");
        assert_eq!(OfficeType::Pptx.name(), "PowerPoint Presentation");
    }
}
