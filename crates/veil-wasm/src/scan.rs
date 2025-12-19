//! Scan functionality for WASM bindings.

use crate::error::WasmError;
use crate::types::{Finding, ScanOptions, ScanResult, ScanStats};
use crate::utils::{report_progress, validate_format, validate_size};
use wasm_bindgen::prelude::*;
use web_time::Instant;

/// Internal scan implementation.
pub fn scan_impl(data: &[u8], options: JsValue) -> Result<JsValue, JsValue> {
    // Parse options
    let options: ScanOptions = if options.is_undefined() || options.is_null() {
        ScanOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options).map_err(|e| {
            WasmError::invalid_config(format!("Invalid scan options: {}", e)).to_js_value()
        })?
    };

    // Validate input
    validate_size(data).map_err(|e| e.to_js_value())?;

    // Detect format
    let format = validate_format(options.filename.as_deref()).map_err(|e| e.to_js_value())?;

    // Perform scan
    let start_time = Instant::now();
    let result = perform_scan(data, format, &options).map_err(|e| e.to_js_value())?;
    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Build result
    let mut stats = ScanStats::new(data.len(), duration_ms);
    for finding in &result {
        stats.add_category(&finding.category);
    }

    let scan_result = ScanResult::new(result, stats);

    serde_wasm_bindgen::to_value(&scan_result).map_err(|e| {
        WasmError::internal(format!("Failed to serialize result: {}", e)).to_js_value()
    })
}

/// Internal scan with progress implementation.
pub fn scan_with_progress_impl(
    data: &[u8],
    options: JsValue,
    on_progress: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    // Parse options
    let options: ScanOptions = if options.is_undefined() || options.is_null() {
        ScanOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options).map_err(|e| {
            WasmError::invalid_config(format!("Invalid scan options: {}", e)).to_js_value()
        })?
    };

    // Validate input
    validate_size(data).map_err(|e| e.to_js_value())?;

    // Detect format
    let format = validate_format(options.filename.as_deref()).map_err(|e| e.to_js_value())?;

    // Report initial progress
    report_progress(on_progress, 0);

    // Perform scan with progress
    let start_time = Instant::now();

    // Parse phase (0-30%)
    report_progress(on_progress, 10);
    let result = perform_scan(data, format, &options).map_err(|e| e.to_js_value())?;
    report_progress(on_progress, 30);

    // Detection complete
    report_progress(on_progress, 100);

    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Build result
    let mut stats = ScanStats::new(data.len(), duration_ms);
    for finding in &result {
        stats.add_category(&finding.category);
    }

    let scan_result = ScanResult::new(result, stats);

    serde_wasm_bindgen::to_value(&scan_result).map_err(|e| {
        WasmError::internal(format!("Failed to serialize result: {}", e)).to_js_value()
    })
}

/// Perform the actual scan using veil-parsers and veil-detect.
fn perform_scan(
    data: &[u8],
    format: veil_parsers::FileFormat,
    options: &ScanOptions,
) -> Result<Vec<Finding>, WasmError> {
    // Validate PII exposure acknowledgment if include_values is requested
    let include_values = if options.include_values {
        if !options.acknowledge_exposure {
            return Err(WasmError::invalid_config(
                "includeValues requires acknowledgeExposure to be true. \
                 Set {includeValues: true, acknowledgeExposure: true} to confirm \
                 you understand the security implications of including PII values.",
            ));
        }
        true
    } else {
        false
    };

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

    // Convert findings and apply filters
    let mut findings = Vec::new();

    for finding in detected {
        // Apply category filter
        let category_name = category_to_string(&finding.category);
        if !options.categories.is_empty() && !options.categories.contains(&category_name) {
            continue;
        }

        // Apply confidence filter
        let confidence = finding.confidence as f64;
        if let Some(min_conf) = options.min_confidence {
            if confidence < min_conf {
                continue;
            }
        }

        // Get byte offset from segment position
        let segment = &parse_result.segments[finding.segment_index];
        let segment_offset = get_byte_offset(&segment.position);
        let start = segment_offset + finding.start;
        let end = segment_offset + finding.end;

        // Only include PII value if explicitly requested and acknowledged
        // Use .as_str() to get the actual value (Display is intentionally redacted)
        let value = if include_values {
            Some(finding.matched_text.as_str().to_string())
        } else {
            None
        };

        findings.push(Finding::new(category_name, value, start, end, confidence));
    }

    Ok(findings)
}

/// Get byte offset from a Position.
fn get_byte_offset(position: &veil_parsers::Position) -> usize {
    match position {
        veil_parsers::Position::Text { byte_offset, .. } => *byte_offset,
        veil_parsers::Position::Html { byte_offset, .. } => *byte_offset,
        veil_parsers::Position::Pdf { byte_offset, .. } => *byte_offset,
        // CSV and JSON don't have byte offsets, use 0
        veil_parsers::Position::Csv { .. } => 0,
        veil_parsers::Position::Json { .. } => 0,
        // Office formats don't have byte offsets, use 0
        veil_parsers::Position::Docx { .. } => 0,
        veil_parsers::Position::Xlsx { .. } => 0,
        veil_parsers::Position::Pptx { .. } => 0,
        veil_parsers::Position::OfficeMetadata { .. } => 0,
        // Email formats have byte offsets within the field
        veil_parsers::Position::Email { byte_offset, .. } => *byte_offset,
    }
}

/// Convert PiiCategory to string.
fn category_to_string(category: &veil_detect::PiiCategory) -> String {
    category.as_str().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_text_with_email_no_value_by_default() {
        let data = b"Contact us at test@example.com for more info.";
        let options = ScanOptions::default();

        let result = perform_scan(data, veil_parsers::FileFormat::Text, &options).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category, "email");
        // Value should be None by default for security
        assert_eq!(result[0].value, None);
    }

    #[test]
    fn test_scan_with_include_values_and_acknowledgment() {
        let data = b"Contact us at test@example.com for more info.";
        let options = ScanOptions {
            include_values: true,
            acknowledge_exposure: true,
            ..Default::default()
        };

        let result = perform_scan(data, veil_parsers::FileFormat::Text, &options).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category, "email");
        // Value should be included when acknowledged
        assert_eq!(result[0].value, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_scan_with_include_values_without_acknowledgment_fails() {
        let data = b"Contact us at test@example.com for more info.";
        let options = ScanOptions {
            include_values: true,
            acknowledge_exposure: false,
            ..Default::default()
        };

        let result = perform_scan(data, veil_parsers::FileFormat::Text, &options);

        // Should fail without acknowledgment
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_with_category_filter() {
        let data = b"Email: test@example.com, Phone: +43 123 456789";
        let options = ScanOptions {
            categories: vec!["email".to_string()],
            ..Default::default()
        };

        let result = perform_scan(data, veil_parsers::FileFormat::Text, &options).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category, "email");
    }

    #[test]
    fn test_scan_with_confidence_filter() {
        let data = b"Contact: test@example.com";
        let options = ScanOptions {
            min_confidence: Some(0.5),
            ..Default::default()
        };

        let result = perform_scan(data, veil_parsers::FileFormat::Text, &options).unwrap();

        // Email should pass 0.5 confidence threshold
        assert!(!result.is_empty());
    }

    #[test]
    fn test_scan_no_pii() {
        let data = b"Hello world! This is a test without any PII.";
        let options = ScanOptions::default();

        let result = perform_scan(data, veil_parsers::FileFormat::Text, &options).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_scan_multiple_pii_types() {
        let data = b"Email: test@example.com, Phone: +43 123 456789";
        let options = ScanOptions::default();

        let result = perform_scan(data, veil_parsers::FileFormat::Text, &options).unwrap();

        // Should find both email and phone
        assert!(result.len() >= 2);
        let categories: Vec<_> = result.iter().map(|f| f.category.as_str()).collect();
        assert!(categories.contains(&"email"));
        assert!(categories.contains(&"phone"));
    }

    #[test]
    fn test_scan_csv_format() {
        let data = b"name,email\nJohn,john@example.com\nJane,jane@test.org";
        let options = ScanOptions::default();

        let result = perform_scan(data, veil_parsers::FileFormat::Csv, &options).unwrap();

        assert!(result.len() >= 2);
    }

    #[test]
    fn test_scan_json_format() {
        let data = br#"{"user": {"email": "test@example.com"}}"#;
        let options = ScanOptions::default();

        let result = perform_scan(data, veil_parsers::FileFormat::Json, &options).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category, "email");
    }

    #[test]
    fn test_scan_html_format() {
        let data = b"<html><body>Contact: test@example.com</body></html>";
        let options = ScanOptions::default();

        let result = perform_scan(data, veil_parsers::FileFormat::Html, &options).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category, "email");
    }

    #[test]
    fn test_scan_with_very_high_confidence_filter() {
        let data = b"Contact: test@example.com";
        let options = ScanOptions {
            min_confidence: Some(0.99999),
            ..Default::default()
        };

        let result = perform_scan(data, veil_parsers::FileFormat::Text, &options).unwrap();

        // High confidence may filter out some results
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_scan_with_empty_category_filter() {
        let data = b"Email: test@example.com";
        let options = ScanOptions {
            categories: vec![], // Empty means all categories
            ..Default::default()
        };

        let result = perform_scan(data, veil_parsers::FileFormat::Text, &options).unwrap();

        assert!(!result.is_empty());
    }

    #[test]
    fn test_scan_with_nonexistent_category_filter() {
        let data = b"Email: test@example.com";
        let options = ScanOptions {
            categories: vec!["nonexistent_category".to_string()],
            ..Default::default()
        };

        let result = perform_scan(data, veil_parsers::FileFormat::Text, &options).unwrap();

        // No findings should match nonexistent category
        assert!(result.is_empty());
    }

    #[test]
    fn test_scan_finding_positions() {
        let data = b"Email: test@example.com here";
        let options = ScanOptions::default();

        let result = perform_scan(data, veil_parsers::FileFormat::Text, &options).unwrap();

        assert_eq!(result.len(), 1);
        // Verify the positions are within bounds
        assert!(result[0].start < result[0].end);
        assert!(result[0].end <= data.len());
    }
}
