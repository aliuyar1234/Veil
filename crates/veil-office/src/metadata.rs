//! Document metadata extraction for Office files.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::io::BufRead;

use veil_parsers::{Position, TextSegment};

/// Dublin Core and extended metadata fields from Office documents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OfficeMetadata {
    /// Document title (dc:title).
    pub title: Option<String>,
    /// Document subject (dc:subject).
    pub subject: Option<String>,
    /// Document creator/author (dc:creator).
    pub creator: Option<String>,
    /// Keywords/tags (cp:keywords).
    pub keywords: Option<String>,
    /// Description (dc:description).
    pub description: Option<String>,
    /// Last modified by (cp:lastModifiedBy).
    pub last_modified_by: Option<String>,
    /// Creation date (dcterms:created).
    pub created: Option<String>,
    /// Last modification date (dcterms:modified).
    pub modified: Option<String>,
    /// Company name (extended property).
    pub company: Option<String>,
    /// Manager name (extended property).
    pub manager: Option<String>,
    /// Application that created the document.
    pub application: Option<String>,
}

impl OfficeMetadata {
    /// Create a new empty metadata instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any metadata fields are populated.
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.subject.is_none()
            && self.creator.is_none()
            && self.keywords.is_none()
            && self.description.is_none()
            && self.last_modified_by.is_none()
            && self.created.is_none()
            && self.modified.is_none()
            && self.company.is_none()
            && self.manager.is_none()
            && self.application.is_none()
    }

    /// Convert metadata fields to TextSegments for PII detection.
    ///
    /// Each non-empty metadata field becomes a TextSegment with
    /// Position::OfficeMetadata indicating the field name.
    pub fn to_text_segments(&self) -> Vec<TextSegment> {
        let mut segments = Vec::new();

        if let Some(ref title) = self.title {
            segments.push(Self::create_segment("title", title));
        }
        if let Some(ref subject) = self.subject {
            segments.push(Self::create_segment("subject", subject));
        }
        if let Some(ref creator) = self.creator {
            segments.push(Self::create_segment("creator", creator));
        }
        if let Some(ref keywords) = self.keywords {
            segments.push(Self::create_segment("keywords", keywords));
        }
        if let Some(ref description) = self.description {
            segments.push(Self::create_segment("description", description));
        }
        if let Some(ref last_modified_by) = self.last_modified_by {
            segments.push(Self::create_segment("last_modified_by", last_modified_by));
        }
        if let Some(ref company) = self.company {
            segments.push(Self::create_segment("company", company));
        }
        if let Some(ref manager) = self.manager {
            segments.push(Self::create_segment("manager", manager));
        }

        segments
    }

    fn create_segment(field: &str, value: &str) -> TextSegment {
        TextSegment {
            content: value.to_string(),
            position: Position::OfficeMetadata {
                field: field.to_string(),
            },
        }
    }
}

/// Parse Dublin Core metadata from docProps/core.xml.
pub fn parse_core_xml<R: BufRead>(reader: R) -> OfficeMetadata {
    let mut metadata = OfficeMetadata::new();
    let mut xml_reader = Reader::from_reader(reader);

    let mut buf = Vec::new();
    let mut current_element: Option<String> = None;

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                current_element = Some(name);
            }
            Ok(Event::Text(ref e)) => {
                if let Some(ref element) = current_element {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if !text.is_empty() {
                        match element.as_str() {
                            "title" => metadata.title = Some(text),
                            "subject" => metadata.subject = Some(text),
                            "creator" => metadata.creator = Some(text),
                            "keywords" => metadata.keywords = Some(text),
                            "description" => metadata.description = Some(text),
                            "lastModifiedBy" => metadata.last_modified_by = Some(text),
                            "created" => metadata.created = Some(text),
                            "modified" => metadata.modified = Some(text),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(_)) => {
                current_element = None;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    metadata
}

/// Parse extended properties from docProps/app.xml.
pub fn parse_app_xml<R: BufRead>(reader: R, metadata: &mut OfficeMetadata) {
    let mut xml_reader = Reader::from_reader(reader);

    let mut buf = Vec::new();
    let mut current_element: Option<String> = None;

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                current_element = Some(name);
            }
            Ok(Event::Text(ref e)) => {
                if let Some(ref element) = current_element {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if !text.is_empty() {
                        match element.as_str() {
                            "Company" => metadata.company = Some(text),
                            "Manager" => metadata.manager = Some(text),
                            "Application" => metadata.application = Some(text),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(_)) => {
                current_element = None;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_core_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/"
                   xmlns:dcterms="http://purl.org/dc/terms/">
    <dc:title>Test Document</dc:title>
    <dc:creator>John Doe</dc:creator>
    <cp:lastModifiedBy>Jane Smith</cp:lastModifiedBy>
    <dcterms:created>2024-01-15T10:30:00Z</dcterms:created>
</cp:coreProperties>"#;

        let metadata = parse_core_xml(Cursor::new(xml));

        assert_eq!(metadata.title, Some("Test Document".to_string()));
        assert_eq!(metadata.creator, Some("John Doe".to_string()));
        assert_eq!(metadata.last_modified_by, Some("Jane Smith".to_string()));
        assert!(metadata.created.is_some());
    }

    #[test]
    fn test_parse_app_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
    <Company>Acme Corp</Company>
    <Manager>Bob Manager</Manager>
    <Application>Microsoft Office Word</Application>
</Properties>"#;

        let mut metadata = OfficeMetadata::new();
        parse_app_xml(Cursor::new(xml), &mut metadata);

        assert_eq!(metadata.company, Some("Acme Corp".to_string()));
        assert_eq!(metadata.manager, Some("Bob Manager".to_string()));
        assert_eq!(
            metadata.application,
            Some("Microsoft Office Word".to_string())
        );
    }

    #[test]
    fn test_metadata_to_segments() {
        let metadata = OfficeMetadata {
            title: Some("My Document".to_string()),
            creator: Some("John Doe".to_string()),
            company: Some("Acme Corp".to_string()),
            ..Default::default()
        };

        let segments = metadata.to_text_segments();

        assert_eq!(segments.len(), 3);
        assert!(segments.iter().any(|s| s.content == "John Doe"));
        assert!(segments.iter().any(|s| s.content == "Acme Corp"));
    }

    #[test]
    fn test_empty_metadata() {
        let metadata = OfficeMetadata::new();
        assert!(metadata.is_empty());

        let segments = metadata.to_text_segments();
        assert!(segments.is_empty());
    }
}
