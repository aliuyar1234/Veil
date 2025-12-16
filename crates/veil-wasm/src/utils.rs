//! Utility functions for WASM bindings.

use crate::error::WasmError;

/// Maximum file size in bytes (50 MB).
pub const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;

/// Validate input data size.
pub fn validate_size(data: &[u8]) -> Result<(), WasmError> {
    if data.is_empty() {
        return Err(WasmError::invalid_input("Input data is empty"));
    }

    if data.len() > MAX_FILE_SIZE {
        return Err(WasmError::file_too_large(data.len(), MAX_FILE_SIZE));
    }

    Ok(())
}

/// Detect file format from filename extension.
pub fn detect_format(filename: Option<&str>) -> Option<veil_parsers::FileFormat> {
    let filename = filename?;
    let ext = filename.rsplit('.').next()?.to_lowercase();

    match ext.as_str() {
        "txt" | "text" => Some(veil_parsers::FileFormat::Text),
        "csv" => Some(veil_parsers::FileFormat::Csv),
        "json" => Some(veil_parsers::FileFormat::Json),
        "html" | "htm" => Some(veil_parsers::FileFormat::Html),
        _ => None,
    }
}

/// Validate that a format is supported.
pub fn validate_format(filename: Option<&str>) -> Result<veil_parsers::FileFormat, WasmError> {
    match filename {
        Some(name) => detect_format(Some(name)).ok_or_else(|| {
            let ext = name.rsplit('.').next().unwrap_or("unknown");
            WasmError::unsupported_format(format!(
                "Unsupported file format: .{}. Supported: txt, csv, json, html",
                ext
            ))
        }),
        None => {
            // Default to text format if no filename provided
            Ok(veil_parsers::FileFormat::Text)
        }
    }
}

/// Report progress to JavaScript callback.
pub fn report_progress(on_progress: &js_sys::Function, percent: u32) {
    let _ = on_progress.call1(
        &wasm_bindgen::JsValue::NULL,
        &wasm_bindgen::JsValue::from(percent),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_txt() {
        assert_eq!(
            detect_format(Some("document.txt")),
            Some(veil_parsers::FileFormat::Text)
        );
    }

    #[test]
    fn test_detect_format_csv() {
        assert_eq!(
            detect_format(Some("data.csv")),
            Some(veil_parsers::FileFormat::Csv)
        );
    }

    #[test]
    fn test_detect_format_json() {
        assert_eq!(
            detect_format(Some("config.json")),
            Some(veil_parsers::FileFormat::Json)
        );
    }

    #[test]
    fn test_detect_format_html() {
        assert_eq!(
            detect_format(Some("page.html")),
            Some(veil_parsers::FileFormat::Html)
        );
        assert_eq!(
            detect_format(Some("page.htm")),
            Some(veil_parsers::FileFormat::Html)
        );
    }

    #[test]
    fn test_detect_format_unknown() {
        assert_eq!(detect_format(Some("file.pdf")), None);
    }

    #[test]
    fn test_detect_format_none() {
        assert_eq!(detect_format(None), None);
    }

    #[test]
    fn test_validate_size_empty() {
        let result = validate_size(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_size_ok() {
        let result = validate_size(b"hello");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_format_text_variants() {
        assert!(validate_format(Some("doc.txt")).is_ok());
        assert!(validate_format(Some("doc.text")).is_ok());
        assert!(validate_format(Some("DOC.TXT")).is_ok());
        assert!(validate_format(Some("DOC.TEXT")).is_ok());
    }

    #[test]
    fn test_validate_format_html_variants() {
        assert!(validate_format(Some("page.html")).is_ok());
        assert!(validate_format(Some("page.htm")).is_ok());
        assert!(validate_format(Some("PAGE.HTML")).is_ok());
    }

    #[test]
    fn test_validate_format_unsupported() {
        let result = validate_format(Some("file.pdf"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("pdf"));
    }

    #[test]
    fn test_validate_format_no_extension() {
        let result = validate_format(Some("noextension"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_format_none_defaults_to_text() {
        let result = validate_format(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), veil_parsers::FileFormat::Text);
    }

    #[test]
    fn test_detect_format_case_insensitive() {
        assert_eq!(
            detect_format(Some("FILE.CSV")),
            Some(veil_parsers::FileFormat::Csv)
        );
        assert_eq!(
            detect_format(Some("FILE.JSON")),
            Some(veil_parsers::FileFormat::Json)
        );
    }

    #[test]
    fn test_detect_format_with_path() {
        assert_eq!(
            detect_format(Some("/path/to/file.json")),
            Some(veil_parsers::FileFormat::Json)
        );
        assert_eq!(
            detect_format(Some("C:\\Users\\file.csv")),
            Some(veil_parsers::FileFormat::Csv)
        );
    }

    #[test]
    fn test_max_file_size_constant() {
        // 50 MB
        assert_eq!(MAX_FILE_SIZE, 50 * 1024 * 1024);
    }
}
