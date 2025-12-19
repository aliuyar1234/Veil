//! Stream processor implementation.

use std::io::{BufRead, BufReader, Read};

use veil_detect::{DetectorRegistry, Finding};

use crate::error::StreamError;
use crate::types::{ProcessingMode, StreamConfig, StreamEvent, StreamResult};

/// A streaming PII detector that processes data incrementally.
///
/// The processor reads data from a `Read` trait and emits events as PII is detected.
/// It handles partial matches across chunk boundaries and supports both line-by-line
/// and chunk-by-chunk processing modes.
pub struct StreamProcessor {
    config: StreamConfig,
    registry: DetectorRegistry,
}

impl StreamProcessor {
    /// Create a new stream processor with the given configuration.
    pub fn new(config: StreamConfig) -> Self {
        Self {
            config,
            registry: DetectorRegistry::default(),
        }
    }

    /// Create a new stream processor with a custom detector registry.
    pub fn with_registry(config: StreamConfig, registry: DetectorRegistry) -> Self {
        Self { config, registry }
    }

    /// Process data from a reader, calling the event handler for each event.
    ///
    /// The event handler receives `StreamEvent` instances as processing progresses.
    /// Processing continues even if non-fatal errors occur.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::io::Cursor;
    /// use veil_stream::{StreamProcessor, StreamConfig};
    ///
    /// let data = "Email: test@example.com";
    /// let reader = Cursor::new(data);
    /// let processor = StreamProcessor::new(StreamConfig::default());
    ///
    /// processor.process(reader, |event| {
    ///     println!("Event: {:?}", event);
    /// })?;
    /// ```
    pub fn process<R, F>(
        &self,
        reader: R,
        mut event_handler: F,
    ) -> Result<StreamResult, StreamError>
    where
        R: Read,
        F: FnMut(StreamEvent),
    {
        match self.config.mode {
            ProcessingMode::LineByLine => self.process_lines(reader, &mut event_handler),
            ProcessingMode::ChunkByChunk => self.process_chunks(reader, &mut event_handler),
        }
    }

    /// Process data line by line.
    fn process_lines<R, F>(
        &self,
        reader: R,
        event_handler: &mut F,
    ) -> Result<StreamResult, StreamError>
    where
        R: Read,
        F: FnMut(StreamEvent),
    {
        let mut result = StreamResult::new();
        let mut buffered = BufReader::with_capacity(self.config.chunk_size, reader);
        let mut line_number = 0;
        let mut offset: u64 = 0;
        let mut line_buffer = String::new();

        loop {
            line_buffer.clear();
            let bytes_read = buffered.read_line(&mut line_buffer)?;

            if bytes_read == 0 {
                break; // End of stream
            }

            line_number += 1;

            // Detect PII in this line
            let findings = self.detect_in_text(&line_buffer);
            let has_findings = !findings.is_empty();

            // Emit line processed event
            let event = StreamEvent::LineProcessed {
                line_number,
                offset,
                findings: findings.clone(),
                content: if self.config.include_content {
                    Some(line_buffer.clone())
                } else {
                    None
                },
            };
            event_handler(event);

            // Update result
            if has_findings {
                result.findings.extend(findings);
            }
            result.total_bytes += bytes_read as u64;
            result.items_processed += 1;

            offset += bytes_read as u64;
        }

        // Finalize and emit end of stream event
        let total_findings = result.findings.len();
        result.finalize();

        event_handler(StreamEvent::EndOfStream {
            total_bytes: offset,
            total_findings,
        });

        Ok(result)
    }

    /// Process data in fixed-size chunks.
    fn process_chunks<R, F>(
        &self,
        mut reader: R,
        event_handler: &mut F,
    ) -> Result<StreamResult, StreamError>
    where
        R: Read,
        F: FnMut(StreamEvent),
    {
        let mut result = StreamResult::new();
        let mut offset: u64 = 0;
        let mut buffer = vec![0u8; self.config.chunk_size];
        let mut overflow_buffer = Vec::new();

        loop {
            let bytes_read = reader.read(&mut buffer)?;

            if bytes_read == 0 && overflow_buffer.is_empty() {
                break; // End of stream
            }

            // Combine overflow from previous chunk with new data
            let mut chunk_data = Vec::new();
            chunk_data.extend_from_slice(&overflow_buffer);
            chunk_data.extend_from_slice(&buffer[..bytes_read]);

            overflow_buffer.clear();

            if chunk_data.is_empty() {
                break;
            }

            // Find the last complete boundary (newline or space)
            // to avoid splitting words/lines across chunks
            let split_point = if bytes_read > 0 {
                // Not at end of stream, find boundary
                self.find_split_point(&chunk_data)
            } else {
                // End of stream, process everything
                chunk_data.len()
            };

            let (to_process, to_overflow) = chunk_data.split_at(split_point);

            // Save overflow for next iteration
            overflow_buffer.extend_from_slice(to_overflow);

            // Check buffer size
            if overflow_buffer.len() > self.config.max_buffer_size {
                return Err(StreamError::BufferOverflow(
                    overflow_buffer.len(),
                    self.config.max_buffer_size,
                ));
            }

            if to_process.is_empty() {
                continue;
            }

            // Convert to string and detect PII
            match std::str::from_utf8(to_process) {
                Ok(text) => {
                    let findings = self.detect_in_text(text);
                    let has_findings = !findings.is_empty();

                    // Emit chunk processed event
                    let event = StreamEvent::ChunkProcessed {
                        offset,
                        bytes_processed: to_process.len(),
                        findings: findings.clone(),
                        content: if self.config.include_content {
                            Some(text.to_string())
                        } else {
                            None
                        },
                    };
                    event_handler(event);

                    // Update result
                    if has_findings {
                        result.findings.extend(findings);
                    }
                    result.total_bytes += to_process.len() as u64;
                    result.items_processed += 1;

                    offset += to_process.len() as u64;
                }
                Err(e) => {
                    // Emit error event but continue processing
                    event_handler(StreamEvent::Error {
                        message: format!("UTF-8 decode error: {}", e),
                        offset,
                    });
                    result.error_count += 1;

                    // Skip this chunk
                    offset += to_process.len() as u64;
                }
            }

            if bytes_read == 0 {
                break;
            }
        }

        // Process any remaining overflow data at end of stream
        if !overflow_buffer.is_empty() {
            match std::str::from_utf8(&overflow_buffer) {
                Ok(text) => {
                    let findings = self.detect_in_text(text);
                    let has_findings = !findings.is_empty();

                    let event = StreamEvent::ChunkProcessed {
                        offset,
                        bytes_processed: overflow_buffer.len(),
                        findings: findings.clone(),
                        content: if self.config.include_content {
                            Some(text.to_string())
                        } else {
                            None
                        },
                    };
                    event_handler(event);

                    if has_findings {
                        result.findings.extend(findings);
                    }
                    result.total_bytes += overflow_buffer.len() as u64;
                    result.items_processed += 1;
                }
                Err(e) => {
                    event_handler(StreamEvent::Error {
                        message: format!("UTF-8 decode error in final chunk: {}", e),
                        offset,
                    });
                    result.error_count += 1;
                }
            }
        }

        // Finalize and emit end of stream event
        let total_findings = result.findings.len();
        result.finalize();

        event_handler(StreamEvent::EndOfStream {
            total_bytes: result.total_bytes,
            total_findings,
        });

        Ok(result)
    }

    /// Find a good split point in the data to avoid breaking words/lines.
    ///
    /// Searches backwards from the end for a newline or whitespace character.
    /// If none found, returns the full length (will be handled by overflow buffer).
    fn find_split_point(&self, data: &[u8]) -> usize {
        if data.is_empty() {
            return 0;
        }

        // Search backwards for newline or space
        for i in (0..data.len()).rev() {
            if data[i] == b'\n' || data[i] == b'\r' || data[i] == b' ' || data[i] == b'\t' {
                return i + 1; // Include the delimiter
            }
        }

        // No good split point found, but check if we're at a reasonable size
        // If chunk is very small, just process it all
        if data.len() < 100 {
            return data.len();
        }

        // Otherwise, take most of the data and overflow the rest
        data.len() * 3 / 4
    }

    /// Detect PII in text using the configured registry.
    fn detect_in_text(&self, text: &str) -> Vec<Finding> {
        use veil_parsers::Position;
        use veil_parsers::TextSegment;

        let segment = TextSegment {
            content: text.to_string().into(),
            position: Position::Text {
                line: 1,
                column: 1,
                byte_offset: 0,
                byte_length: text.len(),
            },
        };
        let findings = self.registry.detect_all(&[segment]);

        // Filter by configured categories if specified
        if self.config.categories.is_empty() {
            findings
        } else {
            findings
                .into_iter()
                .filter(|f| self.config.categories.contains(&f.category))
                .collect()
        }
    }

    /// Process a stream and collect all results without event callbacks.
    ///
    /// This is a convenience method for simple use cases where you just want
    /// the final results without handling individual events.
    pub fn process_all<R>(&self, reader: R) -> Result<StreamResult, StreamError>
    where
        R: Read,
    {
        self.process(reader, |_| {})
    }

    /// Flush any remaining buffered data.
    ///
    /// This method is called automatically at the end of stream processing,
    /// but can be called manually if needed.
    pub fn flush<F>(&self, mut event_handler: F) -> Result<(), StreamError>
    where
        F: FnMut(StreamEvent),
    {
        // Emit end of stream marker
        event_handler(StreamEvent::EndOfStream {
            total_bytes: 0,
            total_findings: 0,
        });
        Ok(())
    }
}

impl Default for StreamProcessor {
    fn default() -> Self {
        Self::new(StreamConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_process_empty_stream() {
        let processor = StreamProcessor::default();
        let reader = Cursor::new("");
        let result = processor.process_all(reader).unwrap();

        assert_eq!(result.total_bytes, 0);
        assert_eq!(result.findings.len(), 0);
    }

    #[test]
    fn test_process_line_by_line() {
        let processor = StreamProcessor::default();
        let data = "Line 1\nLine 2\nLine 3";
        let reader = Cursor::new(data);

        let mut line_count = 0;
        let result = processor
            .process(reader, |event| {
                if let StreamEvent::LineProcessed { .. } = event {
                    line_count += 1;
                }
            })
            .unwrap();

        assert_eq!(line_count, 3);
        assert!(result.total_bytes > 0);
    }

    #[test]
    fn test_process_chunks() {
        let config = StreamConfig {
            mode: ProcessingMode::ChunkByChunk,
            chunk_size: 10, // Small chunks to test splitting
            ..Default::default()
        };

        let processor = StreamProcessor::new(config);
        let data = "This is a test of chunk processing with some text";
        let reader = Cursor::new(data);

        let mut chunk_count = 0;
        let result = processor
            .process(reader, |event| {
                if let StreamEvent::ChunkProcessed { .. } = event {
                    chunk_count += 1;
                }
            })
            .unwrap();

        assert!(chunk_count > 1); // Should have multiple chunks
        assert_eq!(result.total_bytes as usize, data.len());
    }

    #[test]
    fn test_detect_pii_in_stream() {
        let processor = StreamProcessor::default();
        let data = "Contact: test@example.com for more info";
        let reader = Cursor::new(data);

        let mut found_pii = false;
        processor
            .process(reader, |event| {
                if event.has_findings() {
                    found_pii = true;
                }
            })
            .unwrap();

        assert!(found_pii);
    }

    #[test]
    fn test_split_point_finding() {
        let processor = StreamProcessor::default();

        // Test with newline
        let data = b"Hello\nWorld";
        let split = processor.find_split_point(data);
        assert_eq!(split, 6); // After the newline

        // Test with space
        let data = b"Hello World";
        let split = processor.find_split_point(data);
        assert_eq!(split, 6); // After the space

        // Test with no delimiter but small size
        let data = b"Hi";
        let split = processor.find_split_point(data);
        assert_eq!(split, 2); // Process all
    }

    #[test]
    fn test_category_filtering() {
        use veil_detect::PiiCategory;

        let config = StreamConfig {
            categories: vec![PiiCategory::Email],
            ..Default::default()
        };

        let processor = StreamProcessor::new(config);
        let data = "Email: test@example.com Phone: 555-1234";
        let reader = Cursor::new(data);

        let result = processor.process_all(reader).unwrap();

        // Should only find email, not phone
        assert!(result
            .findings
            .iter()
            .all(|f| f.category == PiiCategory::Email));
    }

    #[test]
    fn test_content_inclusion() {
        let config = StreamConfig {
            include_content: false,
            ..Default::default()
        };

        let processor = StreamProcessor::new(config);
        let data = "Test data";
        let reader = Cursor::new(data);

        processor
            .process(reader, |event| {
                assert!(event.content().is_none());
            })
            .unwrap();
    }

    #[test]
    fn test_error_handling() {
        // Invalid UTF-8 sequence
        let invalid_data: &[u8] = &[0xFF, 0xFE, 0xFD];

        let config = StreamConfig {
            mode: ProcessingMode::ChunkByChunk,
            ..Default::default()
        };
        let processor = StreamProcessor::new(config);

        let reader = Cursor::new(invalid_data);

        let mut error_count = 0;
        let result = processor
            .process(reader, |event| {
                if let StreamEvent::Error { .. } = event {
                    error_count += 1;
                }
            })
            .unwrap();

        assert!(error_count > 0 || result.error_count > 0);
    }
}
