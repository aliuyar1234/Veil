//! Streaming parser implementations.
//!
//! Provides iterator-based parsing for large files without loading
//! everything into memory at once.

use std::io::{BufRead, BufReader, Read};

use crate::error::ParseError;
use crate::types::{FileFormat, ParseOptions, Position, TextSegment};

/// A streaming segment iterator for text files.
pub struct TextStream<R: Read> {
    reader: BufReader<R>,
    line_num: usize,
    byte_offset: usize,
    encoding: String,
    buffer: String,
}

impl<R: Read> TextStream<R> {
    /// Create a new text stream from a reader.
    pub fn new(reader: R, options: &ParseOptions) -> Self {
        let encoding = options
            .encoding
            .clone()
            .unwrap_or_else(|| "UTF-8".to_string());

        Self {
            reader: BufReader::new(reader),
            line_num: 0,
            byte_offset: 0,
            encoding,
            buffer: String::new(),
        }
    }

    /// Get the detected encoding.
    pub fn encoding(&self) -> &str {
        &self.encoding
    }
}

impl<R: Read> Iterator for TextStream<R> {
    type Item = Result<TextSegment, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();

        match self.reader.read_line(&mut self.buffer) {
            Ok(0) => None, // EOF
            Ok(bytes_read) => {
                self.line_num += 1;

                // Trim the trailing newline but keep track of original length
                let content = self.buffer.trim_end_matches(&['\r', '\n'][..]).to_string();
                let content_len = content.len();

                let segment = TextSegment {
                    content: content.into(),
                    position: Position::Text {
                        line: self.line_num,
                        column: 1,
                        byte_offset: self.byte_offset,
                        byte_length: content_len,
                    },
                };

                self.byte_offset += bytes_read;
                Some(Ok(segment))
            }
            Err(e) => Some(Err(ParseError::Io(e))),
        }
    }
}

/// A streaming segment iterator for CSV files.
pub struct CsvStream<R: Read> {
    reader: csv::Reader<R>,
    headers: Vec<String>,
    current_record: Option<csv::StringRecord>,
    row_num: usize,
    col_idx: usize,
    #[allow(dead_code)]
    has_headers: bool,
}

impl<R: Read> CsvStream<R> {
    /// Create a new CSV stream from a reader.
    pub fn new(reader: R, options: &ParseOptions) -> Result<Self, ParseError> {
        let delimiter = options.csv_delimiter.unwrap_or(b',');
        let has_headers = options.csv_has_headers.unwrap_or(true);

        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(has_headers)
            .flexible(true)
            .from_reader(reader);

        let headers: Vec<String> = if has_headers {
            csv_reader
                .headers()
                .map(|h| h.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(Self {
            reader: csv_reader,
            headers,
            current_record: None,
            row_num: if has_headers { 1 } else { 0 },
            col_idx: 0,
            has_headers,
        })
    }

    /// Get the CSV headers.
    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    fn advance_record(&mut self) -> Result<bool, ParseError> {
        match self.reader.records().next() {
            Some(Ok(record)) => {
                self.current_record = Some(record);
                self.row_num += 1;
                self.col_idx = 0;
                Ok(true)
            }
            Some(Err(e)) => Err(ParseError::CsvError {
                row: self.row_num + 1,
                message: e.to_string(),
            }),
            None => {
                self.current_record = None;
                Ok(false)
            }
        }
    }
}

impl<R: Read> Iterator for CsvStream<R> {
    type Item = Result<TextSegment, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        // If no current record or exhausted current record, get next
        if self.current_record.is_none()
            || self
                .current_record
                .as_ref()
                .map(|r| self.col_idx >= r.len())
                .unwrap_or(true)
        {
            match self.advance_record() {
                Ok(true) => {}
                Ok(false) => return None,
                Err(e) => return Some(Err(e)),
            }
        }

        // Get current field
        let record = self.current_record.as_ref()?;
        let field = record.get(self.col_idx)?;
        let header = self.headers.get(self.col_idx).cloned();

        let segment = TextSegment {
            content: field.to_string().into(),
            position: Position::Csv {
                row: self.row_num,
                column: self.col_idx,
                header,
            },
        };

        self.col_idx += 1;
        Some(Ok(segment))
    }
}

/// Streaming parser that yields segments without loading entire file.
pub enum ParseStream<R: Read> {
    /// Text file streaming parser.
    Text(TextStream<R>),
    /// CSV file streaming parser.
    Csv(CsvStream<R>),
}

impl<R: Read> ParseStream<R> {
    /// Create a streaming parser for the given format.
    ///
    /// Returns `None` if the format doesn't support streaming.
    pub fn new(reader: R, format: FileFormat, options: &ParseOptions) -> Result<Option<Self>, ParseError> {
        match format {
            FileFormat::Text => Ok(Some(Self::Text(TextStream::new(reader, options)))),
            FileFormat::Csv => Ok(Some(Self::Csv(CsvStream::new(reader, options)?))),
            // JSON, HTML, PDF require full parsing
            _ => Ok(None),
        }
    }

    /// Check if a format supports streaming.
    pub fn supports_streaming(format: FileFormat) -> bool {
        matches!(format, FileFormat::Text | FileFormat::Csv)
    }
}

impl<R: Read> Iterator for ParseStream<R> {
    type Item = Result<TextSegment, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Text(stream) => stream.next(),
            Self::Csv(stream) => stream.next(),
        }
    }
}

/// Create a streaming parser from a reader with format detection.
///
/// Reads the first 1KB to detect format, then returns a streaming iterator.
/// Returns `None` if the format doesn't support streaming.
pub fn stream_reader<R: Read>(
    mut reader: R,
    options: &ParseOptions,
) -> Result<Option<ParseStream<impl Read>>, ParseError> {
    use std::io::Cursor;

    // Read first 1KB for format detection
    let mut header = vec![0u8; 1024];
    let n = reader.read(&mut header)?;
    header.truncate(n);

    let format = options
        .format
        .unwrap_or_else(|| crate::detect::detect_format(&header, None));

    if !ParseStream::<R>::supports_streaming(format) {
        return Ok(None);
    }

    // Chain the header bytes with the rest of the reader
    let chained = Cursor::new(header).chain(reader);
    ParseStream::new(chained, format, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_text_stream() {
        let data = "Line 1\nLine 2\nLine 3\n";
        let reader = Cursor::new(data);
        let stream = TextStream::new(reader, &ParseOptions::default());

        let segments: Vec<_> = stream.filter_map(|r| r.ok()).collect();

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].content.as_str(), "Line 1");
        assert_eq!(segments[1].content.as_str(), "Line 2");
        assert_eq!(segments[2].content.as_str(), "Line 3");
    }

    #[test]
    fn test_text_stream_positions() {
        let data = "Hello\nWorld\n";
        let reader = Cursor::new(data);
        let stream = TextStream::new(reader, &ParseOptions::default());

        let segments: Vec<_> = stream.filter_map(|r| r.ok()).collect();

        if let Position::Text {
            line, byte_offset, ..
        } = &segments[0].position
        {
            assert_eq!(*line, 1);
            assert_eq!(*byte_offset, 0);
        }

        if let Position::Text {
            line, byte_offset, ..
        } = &segments[1].position
        {
            assert_eq!(*line, 2);
            assert_eq!(*byte_offset, 6); // "Hello\n" = 6 bytes
        }
    }

    #[test]
    fn test_csv_stream() {
        let data = "name,email\nJohn,john@test.com\nJane,jane@test.com\n";
        let reader = Cursor::new(data);
        let stream = CsvStream::new(reader, &ParseOptions::default()).unwrap();

        let segments: Vec<_> = stream.filter_map(|r| r.ok()).collect();

        // 2 rows * 2 columns = 4 segments
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].content.as_str(), "John");
        assert_eq!(segments[1].content.as_str(), "john@test.com");
        assert_eq!(segments[2].content.as_str(), "Jane");
        assert_eq!(segments[3].content.as_str(), "jane@test.com");
    }

    #[test]
    fn test_csv_stream_headers() {
        let data = "name,email\nJohn,john@test.com\n";
        let reader = Cursor::new(data);
        let stream = CsvStream::new(reader, &ParseOptions::default()).unwrap();

        let segments: Vec<_> = stream.filter_map(|r| r.ok()).collect();

        if let Position::Csv { header, .. } = &segments[1].position {
            assert_eq!(header.as_deref(), Some("email"));
        }
    }

    #[test]
    fn test_parse_stream_text() {
        let data = "Line 1\nLine 2\n";
        let reader = Cursor::new(data);
        let options = ParseOptions {
            format: Some(FileFormat::Text),
            ..Default::default()
        };

        let stream = ParseStream::new(reader, FileFormat::Text, &options)
            .unwrap()
            .unwrap();
        let segments: Vec<_> = stream.filter_map(|r| r.ok()).collect();

        assert_eq!(segments.len(), 2);
    }

    #[test]
    fn test_parse_stream_csv() {
        let data = "a,b\n1,2\n";
        let reader = Cursor::new(data);
        let options = ParseOptions::default();

        let stream = ParseStream::new(reader, FileFormat::Csv, &options)
            .unwrap()
            .unwrap();
        let segments: Vec<_> = stream.filter_map(|r| r.ok()).collect();

        assert_eq!(segments.len(), 2);
    }

    #[test]
    fn test_supports_streaming() {
        assert!(ParseStream::<&[u8]>::supports_streaming(FileFormat::Text));
        assert!(ParseStream::<&[u8]>::supports_streaming(FileFormat::Csv));
        assert!(!ParseStream::<&[u8]>::supports_streaming(FileFormat::Json));
        assert!(!ParseStream::<&[u8]>::supports_streaming(FileFormat::Html));
        assert!(!ParseStream::<&[u8]>::supports_streaming(FileFormat::Pdf));
    }

    #[test]
    fn test_stream_reader_with_detection() {
        let data = "Line 1\nLine 2\n";
        let reader = Cursor::new(data);
        let options = ParseOptions::default();

        let stream = stream_reader(reader, &options).unwrap().unwrap();
        let segments: Vec<_> = stream.filter_map(|r| r.ok()).collect();

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].content.as_str(), "Line 1");
    }

    #[test]
    fn test_empty_text_stream() {
        let data = "";
        let reader = Cursor::new(data);
        let stream = TextStream::new(reader, &ParseOptions::default());

        let segments: Vec<_> = stream.filter_map(|r| r.ok()).collect();
        assert_eq!(segments.len(), 0);
    }

    #[test]
    fn test_single_line_no_newline() {
        let data = "Single line";
        let reader = Cursor::new(data);
        let stream = TextStream::new(reader, &ParseOptions::default());

        let segments: Vec<_> = stream.filter_map(|r| r.ok()).collect();

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].content.as_str(), "Single line");
    }
}
