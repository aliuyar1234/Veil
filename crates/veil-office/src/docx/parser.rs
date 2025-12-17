//! DOCX file parsing implementation.

use std::io::{Read, Seek};
use std::time::Instant;

use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use crate::error::{OfficeError, Result};
use crate::metadata::{parse_app_xml, parse_core_xml, OfficeMetadata};
use veil_parsers::{DocumentMetadata, FileFormat, ParseResult, Position, TableCell, TextSegment};

/// Document section types in Word documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Some variants are for future expansion
pub enum DocxSection {
    /// Main document body.
    Body,
    /// Document header.
    Header,
    /// Document footer.
    Footer,
    /// Footnote.
    Footnote,
    /// Endnote.
    Endnote,
    /// Table content.
    Table,
    /// Text box.
    TextBox,
    /// Comment.
    Comment,
}

impl DocxSection {
    fn as_str(&self) -> &'static str {
        match self {
            DocxSection::Body => "body",
            DocxSection::Header => "header",
            DocxSection::Footer => "footer",
            DocxSection::Footnote => "footnote",
            DocxSection::Endnote => "endnote",
            DocxSection::Table => "table",
            DocxSection::TextBox => "textbox",
            DocxSection::Comment => "comment",
        }
    }
}

/// Parse a DOCX file and extract all text content.
pub fn parse_docx<R: Read + Seek>(reader: R) -> Result<ParseResult> {
    let start = Instant::now();

    let mut archive = ZipArchive::new(reader)?;

    // Verify this is a valid DOCX
    if archive.by_name("word/document.xml").is_err() {
        return Err(OfficeError::NotOfficeOpenXml(
            "word/document.xml".to_string(),
        ));
    }

    let mut segments = Vec::new();
    let mut total_chars = 0;

    // Extract main document content
    let (doc_segments, chars) = extract_document_xml(&mut archive)?;
    segments.extend(doc_segments);
    total_chars += chars;

    // Extract headers
    for i in 1..=3 {
        // Word typically has up to 3 headers
        let header_name = format!("word/header{}.xml", i);
        if let Ok((header_segments, chars)) =
            extract_header_footer_xml(&mut archive, &header_name, DocxSection::Header)
        {
            segments.extend(header_segments);
            total_chars += chars;
        }
    }

    // Extract footers
    for i in 1..=3 {
        let footer_name = format!("word/footer{}.xml", i);
        if let Ok((footer_segments, chars)) =
            extract_header_footer_xml(&mut archive, &footer_name, DocxSection::Footer)
        {
            segments.extend(footer_segments);
            total_chars += chars;
        }
    }

    // Extract metadata
    let metadata = extract_metadata(&mut archive);

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
            format: FileFormat::Docx,
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

/// Extract text from word/document.xml.
fn extract_document_xml<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<(Vec<TextSegment>, usize)> {
    let file = archive.by_name("word/document.xml")?;
    let mut reader = Reader::from_reader(std::io::BufReader::new(file));

    let mut segments = Vec::new();
    let mut total_chars = 0;
    let mut paragraph_num = 0;
    let mut in_paragraph = false;
    let mut in_table = false;
    let mut table_row = 0;
    let mut table_col = 0;
    let mut current_text = String::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"p" => {
                        // Paragraph start
                        in_paragraph = true;
                        paragraph_num += 1;
                        current_text.clear();
                    }
                    b"tbl" => {
                        // Table start
                        in_table = true;
                        table_row = 0;
                    }
                    b"tr" => {
                        // Table row
                        table_row += 1;
                        table_col = 0;
                    }
                    b"tc" => {
                        // Table cell
                        table_col += 1;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_paragraph {
                    let text = e.unescape().unwrap_or_default();
                    current_text.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"p" => {
                        // Paragraph end - emit segment if we have content
                        if !current_text.is_empty() {
                            let section = if in_table {
                                DocxSection::Table
                            } else {
                                DocxSection::Body
                            };

                            total_chars += current_text.len();

                            let position = if in_table {
                                Position::Docx {
                                    section: section.as_str().to_string(),
                                    paragraph: paragraph_num,
                                    char_offset: 0,
                                    char_length: current_text.len(),
                                    table_cell: Some(TableCell {
                                        table_index: 1, // Simplified
                                        row: table_row,
                                        column: table_col,
                                    }),
                                }
                            } else {
                                Position::Docx {
                                    section: section.as_str().to_string(),
                                    paragraph: paragraph_num,
                                    char_offset: 0,
                                    char_length: current_text.len(),
                                    table_cell: None,
                                }
                            };

                            segments.push(TextSegment {
                                content: current_text.clone().into(),
                                position,
                            });
                        }
                        in_paragraph = false;
                        current_text.clear();
                    }
                    b"tbl" => {
                        in_table = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OfficeError::XmlError(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Ok((segments, total_chars))
}

/// Extract text from header/footer XML files.
fn extract_header_footer_xml<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
    section: DocxSection,
) -> Result<(Vec<TextSegment>, usize)> {
    let file = match archive.by_name(path) {
        Ok(f) => f,
        Err(_) => return Ok((Vec::new(), 0)),
    };

    let mut reader = Reader::from_reader(std::io::BufReader::new(file));

    let mut segments = Vec::new();
    let mut total_chars = 0;
    let mut paragraph_num = 0;
    let mut in_paragraph = false;
    let mut current_text = String::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.local_name().as_ref() == b"p" {
                    in_paragraph = true;
                    paragraph_num += 1;
                    current_text.clear();
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_paragraph {
                    let text = e.unescape().unwrap_or_default();
                    current_text.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                if e.local_name().as_ref() == b"p" {
                    if !current_text.is_empty() {
                        total_chars += current_text.len();

                        segments.push(TextSegment {
                            content: current_text.clone().into(),
                            position: Position::Docx {
                                section: section.as_str().to_string(),
                                paragraph: paragraph_num,
                                char_offset: 0,
                                char_length: current_text.len(),
                                table_cell: None,
                            },
                        });
                    }
                    in_paragraph = false;
                    current_text.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok((segments, total_chars))
}

/// Extract metadata from docProps files.
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
