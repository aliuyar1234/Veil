//! Protect endpoint handlers.

use axum::{
    body::Body,
    extract::{Multipart, Query, State},
    http::{header, Response, StatusCode},
};
use std::time::Instant;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::models::{Finding, PositionInfo, ProtectOptions, ScanStats};
use crate::security::{safe_content_disposition, sanitize_filename};
use crate::AppState;

/// POST /api/v1/protect - Protect a file by redacting PII.
pub async fn protect_file(
    State(state): State<AppState>,
    Query(options): Query<ProtectOptions>,
    mut multipart: Multipart,
) -> ApiResult<Response<Body>> {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4().to_string();

    // Extract file from multipart
    let (filename, data) = extract_file(&mut multipart, state.config.max_body_size).await?;

    // Detect format and parse
    let format = veil_parsers::detect_format(&data, filename.as_deref());
    let _format_str = format!("{:?}", format).to_lowercase();

    // Parse the document
    let parse_options = veil_parsers::ParseOptions {
        format: Some(format),
        ..Default::default()
    };

    let parse_result = veil_parsers::parse_bytes(&data, &parse_options)?;

    // Detect PII
    let registry = veil_detect::DetectorRegistry::default();
    let detections = registry.detect_all(&parse_result.segments);

    // Filter detections based on options
    let filtered_detections: Vec<_> = detections
        .into_iter()
        .filter(|d| {
            let category = d.category.as_str();
            if !options.categories.is_empty() && !options.categories.contains(&category.to_string())
            {
                return false;
            }
            if let Some(min_conf) = options.min_confidence {
                if (d.confidence as f64) < min_conf {
                    return false;
                }
            }
            true
        })
        .collect();

    // Determine redaction style
    let style = match options.style {
        crate::models::ProtectionMethod::Labels => veil_redact::RedactionStyle::Label,
        crate::models::ProtectionMethod::Redact => veil_redact::RedactionStyle::Label,
        crate::models::ProtectionMethod::Mask => {
            veil_redact::RedactionStyle::Mask(veil_redact::MaskingRule::default())
        }
        _ => veil_redact::RedactionStyle::Label,
    };

    // Apply redaction
    let original_text = String::from_utf8_lossy(&data);
    let redaction_result =
        veil_redact::redact_with_style(&original_text, &filtered_detections, style);
    let redacted_content = redaction_result.text;

    let protected_count = filtered_detections.len();
    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Build stats
    let mut stats = ScanStats::new(data.len(), duration_ms);
    for d in &filtered_detections {
        stats.add_finding(d.category.as_str());
    }

    // Build findings list if requested
    let _findings: Vec<Finding> = if options.include_findings {
        filtered_detections
            .iter()
            .map(|d| {
                let segment = &parse_result.segments[d.segment_index];
                Finding {
                    category: d.category.as_str().to_string(),
                    value: d.matched_text.clone(),
                    confidence: d.confidence as f64,
                    start: d.start,
                    end: d.end,
                    context: None,
                    position: convert_position(&segment.position),
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    // Return the protected file with sanitized filename
    let safe_filename = filename.unwrap_or_else(|| "document.txt".to_string());
    let output_filename = format!("protected_{}", safe_filename);
    let content_disposition = safe_content_disposition(&output_filename, "attachment");

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .header("X-Request-Id", &request_id)
        .header("X-Protected-Count", protected_count.to_string())
        .header("X-Duration-Ms", duration_ms.to_string())
        .body(Body::from(redacted_content))
        .map_err(|e| ApiError::Internal(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

/// Extract file from multipart form data.
async fn extract_file(
    multipart: &mut Multipart,
    max_size: usize,
) -> ApiResult<(Option<String>, Vec<u8>)> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to read multipart: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            // Sanitize filename to prevent path traversal attacks
            let filename = field.file_name().map(sanitize_filename);

            let data = field
                .bytes()
                .await
                .map_err(|e| ApiError::BadRequest(format!("Failed to read file: {}", e)))?;

            if data.len() > max_size {
                return Err(ApiError::PayloadTooLarge(format!(
                    "File size {} exceeds maximum {}",
                    data.len(),
                    max_size
                )));
            }

            return Ok((filename, data.to_vec()));
        }
    }

    Err(ApiError::BadRequest(
        "No file field in multipart form".to_string(),
    ))
}

/// Convert veil-parsers Position to API PositionInfo.
fn convert_position(position: &veil_parsers::Position) -> Option<PositionInfo> {
    match position {
        veil_parsers::Position::Text { line, column, .. } => Some(PositionInfo::Text {
            line: *line,
            column: *column,
        }),
        veil_parsers::Position::Csv {
            row,
            column,
            header,
            ..
        } => Some(PositionInfo::Csv {
            row: *row,
            column: *column,
            header: header.clone(),
        }),
        veil_parsers::Position::Json { path, .. } => {
            Some(PositionInfo::Json { path: path.clone() })
        }
        veil_parsers::Position::Pdf { page, .. } => Some(PositionInfo::Pdf { page: *page }),
        veil_parsers::Position::Email { field, .. } => Some(PositionInfo::Email {
            field: field.clone(),
        }),
        _ => None,
    }
}
