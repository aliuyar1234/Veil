//! Protect functionality for WASM bindings.

use crate::error::WasmError;
use crate::types::{ProtectOptions, ProtectResult, ProtectStats, ProtectStyle};
use crate::utils::{report_progress, validate_format, validate_size};
use veil_redact::RedactionStyle;
use wasm_bindgen::prelude::*;

/// Internal protect implementation.
pub fn protect_impl(data: &[u8], options: JsValue) -> Result<JsValue, JsValue> {
    // Parse options
    let options: ProtectOptions = if options.is_undefined() || options.is_null() {
        ProtectOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options).map_err(|e| {
            WasmError::invalid_config(format!("Invalid protect options: {}", e)).to_js_value()
        })?
    };

    // Validate input
    validate_size(data).map_err(|e| e.to_js_value())?;

    // Detect format
    let format = validate_format(options.filename.as_deref()).map_err(|e| e.to_js_value())?;

    // Perform protection
    let start_time = instant::Instant::now();
    let (protected_data, protected_categories, replacements) =
        perform_protect(data, format, &options).map_err(|e| e.to_js_value())?;
    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Build result
    let stats = ProtectStats::new(replacements, protected_categories, duration_ms);
    let result = ProtectResult::new(protected_data, stats);

    serde_wasm_bindgen::to_value(&result).map_err(|e| {
        WasmError::internal(format!("Failed to serialize result: {}", e)).to_js_value()
    })
}

/// Internal protect with progress implementation.
pub fn protect_with_progress_impl(
    data: &[u8],
    options: JsValue,
    on_progress: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    // Parse options
    let options: ProtectOptions = if options.is_undefined() || options.is_null() {
        ProtectOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options).map_err(|e| {
            WasmError::invalid_config(format!("Invalid protect options: {}", e)).to_js_value()
        })?
    };

    // Validate input
    validate_size(data).map_err(|e| e.to_js_value())?;

    // Detect format
    let format = validate_format(options.filename.as_deref()).map_err(|e| e.to_js_value())?;

    // Report initial progress
    report_progress(on_progress, 0);

    // Perform protection with progress
    let start_time = instant::Instant::now();

    // Scan phase (0-50%)
    report_progress(on_progress, 25);

    let (protected_data, protected_categories, replacements) =
        perform_protect(data, format, &options).map_err(|e| e.to_js_value())?;

    // Protection phase complete (50-100%)
    report_progress(on_progress, 75);
    report_progress(on_progress, 100);

    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Build result
    let stats = ProtectStats::new(replacements, protected_categories, duration_ms);
    let result = ProtectResult::new(protected_data, stats);

    serde_wasm_bindgen::to_value(&result).map_err(|e| {
        WasmError::internal(format!("Failed to serialize result: {}", e)).to_js_value()
    })
}

/// Perform the actual protection using veil-parsers, veil-detect, and veil-redact.
fn perform_protect(
    data: &[u8],
    format: veil_parsers::FileFormat,
    options: &ProtectOptions,
) -> Result<(Vec<u8>, Vec<String>, usize), WasmError> {
    // Parse the document with format set in options
    let parse_options = veil_parsers::ParseOptions {
        format: Some(format),
        ..Default::default()
    };
    let parse_result = veil_parsers::parse_bytes(data, &parse_options)
        .map_err(|e| WasmError::internal(format!("Parse error: {}", e)))?;

    // Use cached detector registry for efficiency
    let registry = crate::registry::get_registry();

    // Detect PII in all segments
    let detected = registry.detect_all(&parse_result.segments);

    // Filter findings by category if specified
    let mut protected_categories = std::collections::HashSet::new();
    let filtered_findings: Vec<_> = detected
        .into_iter()
        .filter(|finding| {
            let category_name = category_to_string(&finding.category);
            if !options.categories.is_empty() && !options.categories.contains(&category_name) {
                return false;
            }
            protected_categories.insert(category_name);
            true
        })
        .collect();

    let replacements = filtered_findings.len();

    // Get full text content
    let text = String::from_utf8_lossy(data).to_string();

    // Convert style
    let redaction_style = match options.style {
        ProtectStyle::Labels => RedactionStyle::Label,
        ProtectStyle::Redact => RedactionStyle::black_bar(),
        ProtectStyle::Mask => RedactionStyle::mask(veil_redact::MaskingRule::default()),
    };

    // Perform redaction
    let redaction_result =
        veil_redact::redact_with_style(&text, &filtered_findings, redaction_style);
    let protected_data = redaction_result.text.into_bytes();

    Ok((
        protected_data,
        protected_categories.into_iter().collect(),
        replacements,
    ))
}

/// Convert PiiCategory to string.
fn category_to_string(category: &veil_detect::PiiCategory) -> String {
    category.as_str().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protect_text_with_email() {
        let data = b"Contact us at test@example.com for more info.";
        let options = ProtectOptions {
            style: ProtectStyle::Labels,
            ..Default::default()
        };

        let (protected, categories, count) =
            perform_protect(data, veil_parsers::FileFormat::Text, &options).unwrap();

        assert_eq!(count, 1);
        assert!(categories.contains(&"email".to_string()));
        let protected_str = String::from_utf8(protected).unwrap();
        assert!(protected_str.contains("[EMAIL]"));
        assert!(!protected_str.contains("test@example.com"));
    }

    #[test]
    fn test_protect_with_redact_style() {
        let data = b"Email: test@example.com";
        let options = ProtectOptions {
            style: ProtectStyle::Redact,
            ..Default::default()
        };

        let (protected, _, _) =
            perform_protect(data, veil_parsers::FileFormat::Text, &options).unwrap();

        let protected_str = String::from_utf8(protected).unwrap();
        assert!(protected_str.contains('█'));
        assert!(!protected_str.contains("test@example.com"));
    }

    #[test]
    fn test_protect_with_category_filter() {
        let data = b"Email: test@example.com, Phone: +43 123 456789";
        let options = ProtectOptions {
            categories: vec!["email".to_string()],
            style: ProtectStyle::Labels,
            ..Default::default()
        };

        let (protected, categories, _) =
            perform_protect(data, veil_parsers::FileFormat::Text, &options).unwrap();

        let protected_str = String::from_utf8(protected).unwrap();
        assert!(protected_str.contains("[EMAIL]"));
        // Phone should not be protected
        assert!(protected_str.contains("+43"));
        assert_eq!(categories.len(), 1);
    }

    #[test]
    fn test_protect_no_pii() {
        let data = b"Hello world! No PII here.";
        let options = ProtectOptions::default();

        let (protected, categories, count) =
            perform_protect(data, veil_parsers::FileFormat::Text, &options).unwrap();

        assert_eq!(count, 0);
        assert!(categories.is_empty());
        assert_eq!(protected, data.to_vec());
    }

    #[test]
    fn test_protect_multiple_emails() {
        let data = b"Contact: a@test.com or b@test.com";
        let options = ProtectOptions {
            style: ProtectStyle::Labels,
            ..Default::default()
        };

        let (protected, _, count) =
            perform_protect(data, veil_parsers::FileFormat::Text, &options).unwrap();

        let protected_str = String::from_utf8(protected).unwrap();
        assert_eq!(count, 2);
        assert!(!protected_str.contains("a@test.com"));
        assert!(!protected_str.contains("b@test.com"));
    }

    #[test]
    fn test_protect_csv_format() {
        let data = b"name,email\nJohn,john@example.com";
        let options = ProtectOptions {
            style: ProtectStyle::Labels,
            ..Default::default()
        };

        let (protected, categories, count) =
            perform_protect(data, veil_parsers::FileFormat::Csv, &options).unwrap();

        // CSV format detection works - it finds the email
        assert!(count > 0);
        assert!(categories.contains(&"email".to_string()));
        // Verify we got valid output
        assert!(!protected.is_empty());
    }

    #[test]
    fn test_protect_json_format() {
        let data = br#"{"email": "test@example.com"}"#;
        let options = ProtectOptions {
            style: ProtectStyle::Labels,
            ..Default::default()
        };

        let (protected, _, _) =
            perform_protect(data, veil_parsers::FileFormat::Json, &options).unwrap();

        let protected_str = String::from_utf8(protected).unwrap();
        assert!(!protected_str.contains("test@example.com"));
    }

    #[test]
    fn test_protect_html_format() {
        let data = b"<p>Email: test@example.com</p>";
        let options = ProtectOptions {
            style: ProtectStyle::Labels,
            ..Default::default()
        };

        let (protected, _, _) =
            perform_protect(data, veil_parsers::FileFormat::Html, &options).unwrap();

        let protected_str = String::from_utf8(protected).unwrap();
        assert!(!protected_str.contains("test@example.com"));
    }

    #[test]
    fn test_protect_with_mask_style() {
        let data = b"Email: test@example.com";
        let options = ProtectOptions {
            style: ProtectStyle::Mask,
            ..Default::default()
        };

        let (protected, _, _) =
            perform_protect(data, veil_parsers::FileFormat::Text, &options).unwrap();

        let protected_str = String::from_utf8(protected).unwrap();
        assert!(!protected_str.contains("test@example.com"));
        // Mask style typically uses asterisks
        assert!(protected_str.contains('*'));
    }

    #[test]
    fn test_protect_returns_valid_utf8() {
        let data = b"Contact: test@example.com";
        let options = ProtectOptions::default();

        let (protected, _, _) =
            perform_protect(data, veil_parsers::FileFormat::Text, &options).unwrap();

        // Protected data should be valid UTF-8
        assert!(String::from_utf8(protected).is_ok());
    }

    #[test]
    fn test_protect_with_empty_category_filter() {
        let data = b"Email: test@example.com";
        let options = ProtectOptions {
            categories: vec![],  // Empty means all categories
            style: ProtectStyle::Labels,
            ..Default::default()
        };

        let (protected, _, count) =
            perform_protect(data, veil_parsers::FileFormat::Text, &options).unwrap();

        let protected_str = String::from_utf8(protected).unwrap();
        assert!(count > 0);
        assert!(!protected_str.contains("test@example.com"));
    }

    #[test]
    fn test_protect_with_nonexistent_category() {
        let data = b"Email: test@example.com";
        let options = ProtectOptions {
            categories: vec!["nonexistent".to_string()],
            ..Default::default()
        };

        let (protected, categories, count) =
            perform_protect(data, veil_parsers::FileFormat::Text, &options).unwrap();

        // Nothing should be protected
        assert_eq!(count, 0);
        assert!(categories.is_empty());
        assert_eq!(protected, data.to_vec());
    }
}
