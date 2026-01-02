//! XLSX file parsing implementation using calamine.

use std::collections::HashMap;
use std::io::{BufRead, Read, Seek, SeekFrom};
use std::time::Instant;

use calamine::{open_workbook_auto_from_rs, Reader as CalamineReader, Sheets};
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use crate::error::{OfficeError, Result};
use crate::metadata::{parse_app_xml, parse_core_xml, OfficeMetadata};
use crate::security::{is_encrypted, validate_archive, SecurityLimits};
use veil_parsers::{DocumentMetadata, FileFormat, ParseResult, Position, TextSegment};

use super::cell_ref::CellReference;

/// Parse an XLSX file and extract all text content.
pub fn parse_xlsx<R: Read + Seek + BufRead + Clone>(reader: R) -> Result<ParseResult> {
    let start = Instant::now();

    // Security validation before passing the reader to calamine.
    // XLSX is a ZIP-based format, so we can enforce ZIP bomb/path traversal limits.
    let (hidden_sheets, metadata) = {
        let mut reader = reader.clone();
        reader.seek(SeekFrom::Start(0))?;
        let mut archive = ZipArchive::new(&mut reader)?;
        validate_archive(&mut archive, &SecurityLimits::default())?;
        if is_encrypted(&mut archive) {
            return Err(OfficeError::Encrypted);
        }

        let hidden_sheets = extract_hidden_sheet_map(&mut archive);
        let metadata = extract_metadata(&mut archive);
        (hidden_sheets, metadata)
    };

    // Open workbook with calamine
    let mut reader = reader;
    reader.seek(SeekFrom::Start(0))?;
    let mut workbook: Sheets<R> = open_workbook_auto_from_rs(reader)?;

    let mut segments = Vec::new();
    let mut total_chars = 0;

    // Get all sheet names first
    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

    // Process each sheet
    for sheet_name in &sheet_names {
        let is_hidden = hidden_sheets.get(sheet_name).copied().unwrap_or(false);

        if let Ok(range) = workbook.worksheet_range(sheet_name) {
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

    // Add metadata as segments
    if let Some(meta) = metadata {
        let meta_segments = meta.to_text_segments();
        for seg in &meta_segments {
            total_chars += seg.content.len();
        }
        segments.extend(meta_segments);
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
        warnings: Vec::new(),
        total_chars,
        duration_ms,
    })
}

fn extract_hidden_sheet_map<R: Read + Seek>(archive: &mut ZipArchive<R>) -> HashMap<String, bool> {
    let mut hidden_sheets = HashMap::new();

    let file = match archive.by_name("xl/workbook.xml") {
        Ok(file) => file,
        Err(_) => return hidden_sheets,
    };

    let mut xml_reader = Reader::from_reader(std::io::BufReader::new(file));
    let mut buf = Vec::new();

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if e.local_name().as_ref() == b"sheet" {
                    let mut name: Option<String> = None;
                    let mut is_hidden = false;

                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"name" => {
                                name =
                                    Some(String::from_utf8_lossy(attr.value.as_ref()).to_string());
                            }
                            b"state" => {
                                let state =
                                    String::from_utf8_lossy(attr.value.as_ref()).to_lowercase();
                                is_hidden = state == "hidden" || state == "veryhidden";
                            }
                            _ => {}
                        }
                    }

                    if let Some(name) = name {
                        hidden_sheets.insert(name, is_hidden);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    hidden_sheets
}

fn extract_metadata<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Option<OfficeMetadata> {
    let mut metadata = OfficeMetadata::new();

    // Parse core.xml
    if let Ok(file) = archive.by_name("docProps/core.xml") {
        metadata = parse_core_xml(std::io::BufReader::new(file));
    }

    // Parse app.xml
    if let Ok(file) = archive.by_name("docProps/app.xml") {
        parse_app_xml(std::io::BufReader::new(file), &mut metadata);
    }

    if metadata.is_empty() {
        None
    } else {
        Some(metadata)
    }
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
    use std::io::{Cursor, Write};
    use zip::write::FileOptions;
    use zip::{CompressionMethod, ZipWriter};

    fn build_xlsx_zip(entries: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buf);

            let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
            for (name, data) in entries {
                zip.start_file(name, options).unwrap();
                zip.write_all(&data).unwrap();
            }
            zip.finish().unwrap();
        }
        buf.into_inner()
    }

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

    #[test]
    fn parse_xlsx_rejects_zip_bomb_before_calamine() {
        let zip_bytes = build_xlsx_zip(vec![
            ("xl/workbook.xml", b"<workbook/>".to_vec()),
            ("xl/media/bomb.bin", vec![0u8; 2 * 1024 * 1024]),
        ]);
        let err = parse_xlsx(Cursor::new(zip_bytes)).unwrap_err();
        assert!(matches!(err, OfficeError::ZipBomb { .. }));
    }

    #[test]
    fn extract_hidden_sheet_map_detects_hidden_sheets() {
        let workbook = br#"
            <workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
                      xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
              <sheets>
                <sheet name="Visible" sheetId="1" r:id="rId1"/>
                <sheet name="Hidden" sheetId="2" r:id="rId2" state="hidden"/>
                <sheet name="VeryHidden" sheetId="3" r:id="rId3" state="veryHidden"/>
              </sheets>
            </workbook>
        "#
        .to_vec();

        let zip_bytes = build_xlsx_zip(vec![("xl/workbook.xml", workbook)]);
        let mut archive = ZipArchive::new(Cursor::new(zip_bytes)).unwrap();

        let hidden = extract_hidden_sheet_map(&mut archive);
        assert_eq!(hidden.get("Visible").copied(), Some(false));
        assert_eq!(hidden.get("Hidden").copied(), Some(true));
        assert_eq!(hidden.get("VeryHidden").copied(), Some(true));
    }

    #[test]
    fn extract_metadata_reads_docprops() {
        let core = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/"
                   xmlns:dcterms="http://purl.org/dc/terms/">
    <dc:title>Test Spreadsheet</dc:title>
    <dc:creator>Jane Doe</dc:creator>
</cp:coreProperties>"#
            .as_bytes()
            .to_vec();

        let app = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
    <Company>Acme Corp</Company>
    <Application>Microsoft Excel</Application>
</Properties>"#
            .as_bytes()
            .to_vec();

        let zip_bytes = build_xlsx_zip(vec![
            ("xl/workbook.xml", b"<workbook/>".to_vec()),
            ("docProps/core.xml", core),
            ("docProps/app.xml", app),
        ]);
        let mut archive = ZipArchive::new(Cursor::new(zip_bytes)).unwrap();

        let metadata = extract_metadata(&mut archive).unwrap();
        assert_eq!(metadata.title.as_deref(), Some("Test Spreadsheet"));
        assert_eq!(metadata.creator.as_deref(), Some("Jane Doe"));
        assert_eq!(metadata.company.as_deref(), Some("Acme Corp"));
        assert_eq!(metadata.application.as_deref(), Some("Microsoft Excel"));
    }
}
