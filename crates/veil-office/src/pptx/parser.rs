//! PPTX file parsing implementation.

use std::io::{Read, Seek};
use std::time::Instant;

use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use crate::error::{OfficeError, Result};
use crate::metadata::{parse_app_xml, parse_core_xml, OfficeMetadata};
use crate::security::{is_encrypted, validate_archive, SecurityLimits};
use veil_types::{DocumentMetadata, FileFormat, ParseResult, ParseWarning, Position, TextSegment};

/// Element types in PowerPoint presentations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Some variants are for future expansion
pub enum PptxElement {
    /// Slide title.
    Title,
    /// Slide body text.
    Body,
    /// Speaker notes.
    Note,
    /// Shape text.
    Shape,
    /// Table text.
    Table,
}

impl PptxElement {
    fn as_str(&self) -> &'static str {
        match self {
            PptxElement::Title => "title",
            PptxElement::Body => "body",
            PptxElement::Note => "note",
            PptxElement::Shape => "shape",
            PptxElement::Table => "table",
        }
    }
}

/// Parse a PPTX file and extract all text content.
pub fn parse_pptx<R: Read + Seek>(reader: R) -> Result<ParseResult> {
    let start = Instant::now();

    let mut archive = ZipArchive::new(reader)?;
    validate_archive(&mut archive, &SecurityLimits::default())?;
    if is_encrypted(&mut archive) {
        return Err(OfficeError::Encrypted);
    }

    // Verify this is a valid PPTX
    if archive.by_name("ppt/presentation.xml").is_err() {
        return Err(OfficeError::NotOfficeOpenXml(
            "ppt/presentation.xml".to_string(),
        ));
    }

    let mut segments = Vec::new();
    let mut total_chars = 0;
    let mut warnings = Vec::new();

    // Find all slide files
    let slide_files: Vec<String> = (1..=1000)
        .map(|i| format!("ppt/slides/slide{}.xml", i))
        .take_while(|path| archive.by_name(path).is_ok())
        .collect();

    // Process each slide
    for (slide_idx, slide_path) in slide_files.iter().enumerate() {
        let slide_num = slide_idx + 1;

        // Extract slide content
        match extract_slide_xml(&mut archive, slide_path, slide_num) {
            Ok((slide_segments, chars)) => {
                segments.extend(slide_segments);
                total_chars += chars;
            }
            Err(_) => warnings.push(crate::warnings::xml_parse_warning(slide_path)),
        }

        // Extract speaker notes
        let notes_path = format!("ppt/notesSlides/notesSlide{}.xml", slide_num);
        if let Ok((notes_segments, chars)) =
            extract_notes_xml(&mut archive, &notes_path, slide_num, &mut warnings)
        {
            segments.extend(notes_segments);
            total_chars += chars;
        }
    }

    // Extract metadata
    let metadata = extract_metadata(&mut archive, &mut warnings);

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
            format: FileFormat::Pptx,
            encoding: "UTF-8".to_string(),
            size_bytes: None,
            filename: None,
            encoding_lossy: false,
        },
        segments,
        warnings,
        total_chars,
        duration_ms,
    })
}

/// Extract text from a slide XML file.
fn extract_slide_xml<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
    slide_num: usize,
) -> Result<(Vec<TextSegment>, usize)> {
    let file = archive.by_name(path)?;
    let mut reader = Reader::from_reader(std::io::BufReader::new(file));

    let mut segments = Vec::new();
    let mut total_chars = 0;
    let mut text_idx = 0;
    let mut in_text = false;
    let mut current_text = String::new();
    let mut current_element = PptxElement::Body;

    // Track nesting to detect titles vs body
    let mut in_title_shape = false;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"t" => {
                        // Text element (DrawingML a:t)
                        in_text = true;
                        current_text.clear();
                    }
                    b"ph" => {
                        // Placeholder - check if it's a title
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"type" {
                                let val = String::from_utf8_lossy(&attr.value);
                                if val == "title" || val == "ctrTitle" {
                                    in_title_shape = true;
                                    current_element = PptxElement::Title;
                                }
                            }
                        }
                    }
                    b"txBody" => {
                        // Text body - reset for new shape
                        if !in_title_shape {
                            current_element = PptxElement::Body;
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_text {
                    let text = e.unescape().unwrap_or_default();
                    current_text.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"t" => {
                        if !current_text.is_empty() {
                            text_idx += 1;
                            total_chars += current_text.len();

                            segments.push(TextSegment {
                                content: current_text.clone().into(),
                                position: Position::Pptx {
                                    slide: slide_num,
                                    element: current_element.as_str().to_string(),
                                    text_index: text_idx,
                                    char_offset: 0,
                                    char_length: current_text.len(),
                                },
                            });
                        }
                        in_text = false;
                        current_text.clear();
                    }
                    b"sp" => {
                        // Shape end - reset title flag
                        in_title_shape = false;
                        current_element = PptxElement::Body;
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

/// Extract text from speaker notes XML.
fn extract_notes_xml<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
    slide_num: usize,
    warnings: &mut Vec<ParseWarning>,
) -> Result<(Vec<TextSegment>, usize)> {
    let file = match archive.by_name(path) {
        Ok(f) => f,
        Err(_) => return Ok((Vec::new(), 0)),
    };

    let mut reader = Reader::from_reader(std::io::BufReader::new(file));

    let mut segments = Vec::new();
    let mut total_chars = 0;
    let mut text_idx = 0;
    let mut in_text = false;
    let mut current_text = String::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.local_name().as_ref() == b"t" {
                    in_text = true;
                    current_text.clear();
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_text {
                    let text = e.unescape().unwrap_or_default();
                    current_text.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                if e.local_name().as_ref() == b"t" {
                    if !current_text.is_empty() {
                        text_idx += 1;
                        total_chars += current_text.len();

                        segments.push(TextSegment {
                            content: current_text.clone().into(),
                            position: Position::Pptx {
                                slide: slide_num,
                                element: PptxElement::Note.as_str().to_string(),
                                text_index: text_idx,
                                char_offset: 0,
                                char_length: current_text.len(),
                            },
                        });
                    }
                    in_text = false;
                    current_text.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => {
                warnings.push(crate::warnings::xml_parse_warning(path));
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok((segments, total_chars))
}

/// Extract metadata from docProps files.
fn extract_metadata<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    warnings: &mut Vec<ParseWarning>,
) -> Option<OfficeMetadata> {
    let mut metadata = OfficeMetadata::new();

    // Parse core.xml
    if let Ok(file) = archive.by_name("docProps/core.xml") {
        let (parsed, mut parsed_warnings) =
            parse_core_xml("docProps/core.xml", std::io::BufReader::new(file));
        metadata = parsed;
        warnings.append(&mut parsed_warnings);
    }

    // Parse app.xml
    if let Ok(file) = archive.by_name("docProps/app.xml") {
        warnings.extend(parse_app_xml(
            "docProps/app.xml",
            std::io::BufReader::new(file),
            &mut metadata,
        ));
    }

    if metadata.is_empty() {
        None
    } else {
        Some(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::write::FileOptions;
    use zip::{CompressionMethod, ZipWriter};

    fn build_pptx_zip(entries: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
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
    fn parse_pptx_rejects_zip_bomb() {
        let presentation = br#"<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"></p:presentation>"#.to_vec();
        let bomb = vec![0u8; 2 * 1024 * 1024];

        let zip_bytes = build_pptx_zip(vec![
            ("ppt/presentation.xml", presentation),
            ("ppt/media/bomb.bin", bomb),
        ]);

        let err = parse_pptx(Cursor::new(zip_bytes)).unwrap_err();
        assert!(matches!(err, OfficeError::ZipBomb { .. }));
    }
}
