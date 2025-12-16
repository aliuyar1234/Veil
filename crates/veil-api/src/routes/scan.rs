//! Scan endpoint handlers.

use axum::{
    extract::{Multipart, Query, State},
    Json,
};
use std::time::Instant;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::models::{Finding, PositionInfo, ScanOptions, ScanResponse, ScanStats};
use crate::security::sanitize_filename;
use crate::AppState;

/// POST /api/v1/scan - Scan a file for PII.
pub async fn scan_file(
    State(state): State<AppState>,
    Query(options): Query<ScanOptions>,
    mut multipart: Multipart,
) -> ApiResult<Json<ScanResponse>> {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4().to_string();

    // Extract file from multipart
    let (filename, data) = extract_file(&mut multipart, state.config.max_body_size).await?;

    // Detect format and parse
    let format = veil_parsers::detect_format(&data, filename.as_deref());
    let format_str = format!("{:?}", format).to_lowercase();

    // Parse the document
    let parse_options = veil_parsers::ParseOptions {
        format: Some(format),
        ..Default::default()
    };

    let parse_result = veil_parsers::parse_bytes(&data, &parse_options)?;

    // Detect PII
    let registry = veil_detect::DetectorRegistry::default();
    let detections = registry.detect_all(&parse_result.segments);

    // Convert to findings
    let mut findings = Vec::new();
    let mut stats = ScanStats::new(data.len(), 0);

    for detection in detections {
        let category = detection.category.as_str().to_string();

        // Apply filters
        if !options.categories.is_empty() && !options.categories.contains(&category) {
            continue;
        }
        if let Some(min_conf) = options.min_confidence {
            if (detection.confidence as f64) < min_conf {
                continue;
            }
        }

        // Get position info
        let segment = &parse_result.segments[detection.segment_index];
        let position = convert_position(&segment.position);

        // Build context if requested
        let context = if options.include_context {
            let content = &segment.content;
            let start = detection.start.saturating_sub(options.context_chars);
            let end = (detection.end + options.context_chars).min(content.len());
            Some(content[start..end].to_string())
        } else {
            None
        };

        stats.add_finding(&category);

        findings.push(Finding {
            category,
            value: detection.matched_text.clone(),
            confidence: detection.confidence as f64,
            start: detection.start,
            end: detection.end,
            context,
            position,
        });
    }

    stats.duration_ms = start_time.elapsed().as_millis() as u64;

    Ok(Json(ScanResponse::success(
        request_id, format_str, findings, stats,
    )))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_stats() {
        let mut stats = ScanStats::new(1000, 50);
        stats.add_finding("email");
        stats.add_finding("email");
        stats.add_finding("phone");

        assert_eq!(stats.total_findings, 3);
        assert_eq!(stats.category_counts.get("email"), Some(&2));
        assert_eq!(stats.category_counts.get("phone"), Some(&1));
    }
}
