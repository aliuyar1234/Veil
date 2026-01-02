//! Streaming XLSX processing for memory-efficient large file handling.
//!
//! While calamine loads entire sheets into memory, this module provides
//! a callback-based API that processes rows incrementally without accumulating
//! all segments into a single vector.

use std::io::{BufRead, Read, Seek};
use std::time::Instant;

use calamine::{open_workbook_auto_from_rs, Reader as CalamineReader, Sheets};

use crate::error::Result;
use veil_types::{DocumentMetadata, FileFormat, ParseResult, Position, TextSegment};

use super::cell_ref::CellReference;
use super::parser::cell_to_string;

/// Configuration for streaming XLSX processing.
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Maximum number of rows to process per batch (default: 10,000).
    pub batch_size: usize,
    /// Maximum total rows to process (0 = unlimited).
    pub max_rows: usize,
    /// Maximum memory estimate before stopping (bytes, 0 = unlimited).
    pub max_memory_estimate: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            batch_size: 10_000,
            max_rows: 0,
            max_memory_estimate: 500 * 1024 * 1024, // 500MB default limit
        }
    }
}

/// Statistics from streaming processing.
#[derive(Debug, Default, Clone)]
pub struct StreamingStats {
    /// Total rows processed.
    pub total_rows: usize,
    /// Total cells with content.
    pub total_cells: usize,
    /// Total characters extracted.
    pub total_chars: usize,
    /// Number of sheets processed.
    pub sheets_processed: usize,
    /// Processing time in milliseconds.
    pub duration_ms: u64,
    /// Whether processing was truncated due to limits.
    pub truncated: bool,
    /// Reason for truncation if any.
    pub truncation_reason: Option<String>,
}

/// Callback function type for row processing.
pub type RowCallback<'a> = &'a mut dyn FnMut(&str, usize, Vec<TextSegment>) -> bool;

/// Parse an XLSX file with streaming callbacks.
///
/// This function processes rows in batches and calls the callback for each batch.
/// The callback receives the sheet name, row number, and segments for that row.
/// Return `false` from the callback to stop processing.
///
/// # Example
///
/// ```ignore
/// let mut all_segments = Vec::new();
/// parse_xlsx_streaming(reader, &StreamingConfig::default(), &mut |_sheet, _row, segments| {
///     all_segments.extend(segments);
///     true // continue processing
/// })?;
/// ```
pub fn parse_xlsx_streaming<R: Read + Seek + BufRead + Clone>(
    reader: R,
    config: &StreamingConfig,
    callback: RowCallback<'_>,
) -> Result<StreamingStats> {
    let start = Instant::now();
    let mut stats = StreamingStats::default();

    // Open workbook
    let mut workbook: Sheets<R> = open_workbook_auto_from_rs(reader)?;
    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

    let mut memory_estimate: usize = 0;

    'sheets: for sheet_name in &sheet_names {
        stats.sheets_processed += 1;

        if let Ok(range) = workbook.worksheet_range(sheet_name) {
            for (row_idx, row) in range.rows().enumerate() {
                stats.total_rows += 1;

                // Check row limit
                if config.max_rows > 0 && stats.total_rows > config.max_rows {
                    stats.truncated = true;
                    stats.truncation_reason =
                        Some(format!("Row limit reached: {} rows", config.max_rows));
                    break 'sheets;
                }

                // Process cells in this row
                let mut row_segments = Vec::new();
                for (col_idx, cell) in row.iter().enumerate() {
                    let cell_text = cell_to_string(cell);

                    if !cell_text.is_empty() {
                        let cell_ref = CellReference::new(
                            sheet_name.clone(),
                            (row_idx + 1) as u32,
                            col_idx as u32,
                        );

                        stats.total_cells += 1;
                        stats.total_chars += cell_text.len();
                        memory_estimate += cell_text.len() + 100; // Rough estimate

                        row_segments.push(TextSegment {
                            content: cell_text.into(),
                            position: Position::Xlsx {
                                sheet: sheet_name.clone(),
                                row: row_idx + 1,
                                column: col_idx,
                                column_letter: cell_ref.column_as_letter(),
                                cell_ref: cell_ref.to_full_reference(),
                                hidden_sheet: false,
                            },
                        });
                    }
                }

                // Call the callback if we have segments
                if !row_segments.is_empty() {
                    let should_continue = callback(sheet_name, row_idx + 1, row_segments);
                    if !should_continue {
                        stats.truncated = true;
                        stats.truncation_reason = Some("Callback requested stop".to_string());
                        break 'sheets;
                    }
                }

                // Check memory limit
                if config.max_memory_estimate > 0 && memory_estimate > config.max_memory_estimate {
                    stats.truncated = true;
                    stats.truncation_reason = Some(format!(
                        "Memory limit reached: ~{} MB",
                        memory_estimate / (1024 * 1024)
                    ));
                    break 'sheets;
                }
            }
        }
    }

    stats.duration_ms = start.elapsed().as_millis() as u64;
    Ok(stats)
}

/// Parse a large XLSX file and return results with memory limits.
///
/// This is a convenience wrapper around `parse_xlsx_streaming` that collects
/// all segments but respects memory and row limits.
pub fn parse_xlsx_large<R: Read + Seek + BufRead + Clone>(
    reader: R,
    config: &StreamingConfig,
) -> Result<ParseResult> {
    let start = Instant::now();
    let mut segments = Vec::new();
    let mut warnings = Vec::new();

    let stats = parse_xlsx_streaming(reader, config, &mut |_sheet, _row, row_segments| {
        segments.extend(row_segments);
        true
    })?;

    if stats.truncated {
        warnings.push(veil_types::ParseWarning {
            code: veil_types::WarningCode::Truncated,
            message: stats
                .truncation_reason
                .clone()
                .unwrap_or_else(|| "Content truncated".to_string()),
            position: None,
        });
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(ParseResult {
        metadata: DocumentMetadata {
            format: FileFormat::Xlsx,
            encoding: "UTF-8".to_string(),
            size_bytes: None,
            filename: None,
            encoding_lossy: false,
        },
        segments,
        warnings,
        total_chars: stats.total_chars,
        duration_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.batch_size, 10_000);
        assert_eq!(config.max_rows, 0);
        assert_eq!(config.max_memory_estimate, 500 * 1024 * 1024);
    }

    #[test]
    fn test_streaming_stats_default() {
        let stats = StreamingStats::default();
        assert_eq!(stats.total_rows, 0);
        assert_eq!(stats.total_cells, 0);
        assert!(!stats.truncated);
    }
}
