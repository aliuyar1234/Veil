//! WebAssembly bindings for Veil PII detection and protection.
//!
//! This crate provides browser-compatible WASM functions for scanning
//! and protecting documents containing personally identifiable information (PII).
//!
//! # Example (JavaScript)
//!
//! ```javascript
//! import init, { scan, protect } from '@veil/wasm';
//!
//! await init();
//!
//! const fileBuffer = await file.arrayBuffer();
//! const findings = await scan(new Uint8Array(fileBuffer), { filename: 'doc.txt' });
//! console.log(findings);
//! ```

mod error;
mod protect;
mod registry;
mod scan;
mod types;
mod utils;

pub use error::{ErrorCode, WasmError};
pub use types::{
    Finding, ProtectOptions, ProtectResult, ProtectStats, ProtectStyle, ScanOptions, ScanResult,
    ScanStats,
};

use wasm_bindgen::prelude::*;

/// Initialize the WASM module.
///
/// Must be called before using scan() or protect().
/// This is typically handled automatically by the JS loader.
#[wasm_bindgen]
pub fn init() -> Result<(), JsValue> {
    Ok(())
}

/// Scan data for PII.
///
/// # Arguments
///
/// * `data` - File contents as Uint8Array
/// * `options` - Optional scan configuration (filename, categories, minConfidence)
///
/// # Returns
///
/// ScanResult containing findings array and statistics.
#[wasm_bindgen]
pub fn scan(data: &[u8], options: JsValue) -> Result<JsValue, JsValue> {
    scan::scan_impl(data, options)
}

/// Scan data for PII with progress callback.
///
/// # Arguments
///
/// * `data` - File contents as Uint8Array
/// * `options` - Optional scan configuration
/// * `on_progress` - Callback function receiving progress percentage (0-100)
#[wasm_bindgen]
pub fn scan_with_progress(
    data: &[u8],
    options: JsValue,
    on_progress: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    scan::scan_with_progress_impl(data, options, on_progress)
}

/// Protect data by redacting PII.
///
/// # Arguments
///
/// * `data` - File contents as Uint8Array
/// * `options` - Protection configuration (filename, style, categories)
///
/// # Returns
///
/// ProtectResult containing redacted data and statistics.
#[wasm_bindgen]
pub fn protect(data: &[u8], options: JsValue) -> Result<JsValue, JsValue> {
    protect::protect_impl(data, options)
}

/// Protect data by redacting PII with progress callback.
///
/// # Arguments
///
/// * `data` - File contents as Uint8Array
/// * `options` - Protection configuration
/// * `on_progress` - Callback function receiving progress percentage (0-100)
#[wasm_bindgen]
pub fn protect_with_progress(
    data: &[u8],
    options: JsValue,
    on_progress: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    protect::protect_with_progress_impl(data, options, on_progress)
}

/// Get supported file formats.
///
/// Returns an array of supported file extensions.
#[wasm_bindgen]
pub fn supported_formats() -> JsValue {
    let formats = vec!["txt", "csv", "json", "html"];
    serde_wasm_bindgen::to_value(&formats).unwrap_or(JsValue::NULL)
}

/// Get available PII categories.
///
/// Returns an array of detectable PII category names.
#[wasm_bindgen]
pub fn available_categories() -> JsValue {
    let categories = vec!["email", "iban", "phone", "credit_card"];
    serde_wasm_bindgen::to_value(&categories).unwrap_or(JsValue::NULL)
}
