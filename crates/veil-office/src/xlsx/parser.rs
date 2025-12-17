//! XLSX file parsing implementation using calamine.

use std::io::{BufRead, Read, Seek};
use std::time::Instant;

use calamine::{open_workbook_auto_from_rs, Reader as CalamineReader, Sheets};

use crate::error::Result;
use veil_parsers::{DocumentMetadata, FileFormat, ParseResult, Position, TextSegment};

use super::cell_ref::CellReference;

/// Parse an XLSX file and extract all text content.
pub fn parse_xlsx<R: Read + Seek + BufRead + Clone>(reader: R) -> Result<ParseResult> {
    let start = Instant::now();

    // Open workbook with calamine
    let mut workbook: Sheets<R> = open_workbook_auto_from_rs(reader)?;

    let mut segments = Vec::new();
    let mut total_chars = 0;

    // Get all sheet names first
    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

    // Process each sheet
    for sheet_name in &sheet_names {
        if let Ok(range) = workbook.worksheet_range(sheet_name) {
            // Determine if this is a hidden sheet (calamine doesn't expose this directly,
            // but we process all sheets anyway per spec)
            let is_hidden = false; // TODO: Detect hidden sheets if calamine exposes this

            for (row_idx, row) in range.rows().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    // Extract cell value as text
                    let cell_text = cell_to_string(cell);

                    if !cell_text.is_empty() {
                        let cell_ref = CellReference::new(
                            sheet_name.clone(),
                            (row_idx + 1) as u32, // 1-based row
                            col_idx as u32,       // 0-based column
                        );

                        total_chars += cell_text.len();

                        segments.push(TextSegment {
                            content: cell_text.into(),
                            position: Position::Xlsx {
                                sheet: sheet_name.clone(),
                                row: row_idx + 1, // 1-based
                                column: col_idx,  // 0-based
                                column_letter: cell_ref.column_as_letter(),
                                cell_ref: cell_ref.to_full_reference(),
                                hidden_sheet: is_hidden,
                            },
                        });
                    }
                }
            }
        }
    }

    // TODO: Extract metadata from docProps/core.xml and docProps/app.xml
    // This requires re-opening the file as a ZIP archive since calamine
    // doesn't expose the metadata extraction directly

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
        warnings: Vec::new(),
        total_chars,
        duration_ms,
    })
}

/// Convert a calamine DataType cell value to a string.
///
/// For formulas, this returns the calculated/display value, not the formula itself.
pub fn cell_to_string(cell: &calamine::Data) -> String {
    use calamine::Data;
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            // Format floats nicely (remove trailing zeros)
            format!("{}", f)
        }
        Data::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        Data::Error(e) => format!("#ERROR: {:?}", e),
        Data::DateTime(dt) => {
            // Convert Excel datetime serial number to ISO format
            // Excel stores dates as days since 1899-12-30
            format!("{}", dt)
        }
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::Data;

    #[test]
    fn test_cell_to_string_empty() {
        assert_eq!(cell_to_string(&Data::Empty), "");
    }

    #[test]
    fn test_cell_to_string_string() {
        assert_eq!(cell_to_string(&Data::String("Hello".to_string())), "Hello");
    }

    #[test]
    fn test_cell_to_string_int() {
        assert_eq!(cell_to_string(&Data::Int(42)), "42");
    }

    #[test]
    fn test_cell_to_string_float() {
        assert_eq!(cell_to_string(&Data::Float(2.5)), "2.5");
    }

    #[test]
    fn test_cell_to_string_bool() {
        assert_eq!(cell_to_string(&Data::Bool(true)), "TRUE");
        assert_eq!(cell_to_string(&Data::Bool(false)), "FALSE");
    }
}
