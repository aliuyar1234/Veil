//! Integration tests for streaming PII detection.

use std::io::Cursor;
use veil_stream::{ProcessingMode, StreamConfig, StreamEvent, StreamProcessor};

#[test]
fn test_stream_with_email() {
    let processor = StreamProcessor::default();
    let data = "Please contact us at support@example.com for assistance.";
    let reader = Cursor::new(data);

    let mut events = Vec::new();
    let result = processor
        .process(reader, |event| {
            events.push(event);
        })
        .unwrap();

    // Should have at least one finding (email)
    assert!(!result.findings.is_empty());

    // Should have line processed and end of stream events
    assert!(events
        .iter()
        .any(|e| matches!(e, StreamEvent::LineProcessed { .. })));
    assert!(events
        .iter()
        .any(|e| matches!(e, StreamEvent::EndOfStream { .. })));
}

#[test]
fn test_stream_multiline() {
    let processor = StreamProcessor::default();
    let data = "Line 1: user@example.com\nLine 2: another@test.org\nLine 3: no PII here";
    let reader = Cursor::new(data);

    let mut line_events = 0;
    let mut findings_count = 0;

    processor
        .process(reader, |event| {
            if let StreamEvent::LineProcessed { findings, .. } = event {
                line_events += 1;
                findings_count += findings.len();
            }
        })
        .unwrap();

    assert_eq!(line_events, 3);
    assert!(findings_count >= 2); // At least 2 emails
}

#[test]
fn test_stream_chunk_mode() {
    let config = StreamConfig {
        mode: ProcessingMode::ChunkByChunk,
        chunk_size: 50, // Small chunks
        ..Default::default()
    };

    let processor = StreamProcessor::new(config);
    let data = "This is a longer text with email@example.com and more content after it to ensure multiple chunks.";
    let reader = Cursor::new(data);

    let mut chunk_events = 0;
    let result = processor
        .process(reader, |event| {
            if let StreamEvent::ChunkProcessed { .. } = event {
                chunk_events += 1;
            }
        })
        .unwrap();

    assert!(chunk_events > 0);
    assert_eq!(result.total_bytes as usize, data.len());
}

#[test]
fn test_stream_boundary_handling() {
    // Test that PII split across chunk boundaries is still detected
    let config = StreamConfig {
        mode: ProcessingMode::ChunkByChunk,
        chunk_size: 15, // Very small to force splitting
        ..Default::default()
    };

    let processor = StreamProcessor::new(config);
    // Email is 19 chars, will span multiple chunks
    let data = "Contact: test@example.com please";
    let reader = Cursor::new(data);

    let result = processor.process_all(reader).unwrap();

    // Should still find the email despite chunk boundaries
    assert!(!result.findings.is_empty());
}

#[test]
fn test_stream_large_input() {
    let processor = StreamProcessor::default();

    // Create a large input with multiple PII instances
    let mut data = String::new();
    for i in 0..1000 {
        data.push_str(&format!("Line {}: user{}@example.com\n", i, i));
    }

    let reader = Cursor::new(data);
    let result = processor.process_all(reader).unwrap();

    assert_eq!(result.items_processed, 1000);
    assert!(result.findings.len() >= 1000); // Should find emails
}

#[test]
fn test_stream_no_pii() {
    let processor = StreamProcessor::default();
    let data = "This is just regular text without any sensitive information.";
    let reader = Cursor::new(data);

    let result = processor.process_all(reader).unwrap();

    assert_eq!(result.findings.len(), 0);
    assert!(result.total_bytes > 0);
}

#[test]
fn test_stream_mixed_content() {
    let processor = StreamProcessor::default();
    let data = r#"
User Profile:
Name: John Doe
Email: john.doe@example.com
Phone: +1-555-0123
Address: 123 Main St

Notes: Regular text here.
"#;
    let reader = Cursor::new(data);

    let result = processor.process_all(reader).unwrap();

    // Should find email and possibly phone
    assert!(!result.findings.is_empty());

    // Check statistics
    assert!(!result.stats_by_category.is_empty());
}

#[test]
fn test_stream_event_content() {
    let config = StreamConfig {
        include_content: true,
        ..Default::default()
    };

    let processor = StreamProcessor::new(config);
    let data = "test@example.com";
    let reader = Cursor::new(data);

    let mut has_content = false;
    processor
        .process(reader, |event| {
            if event.content().is_some() {
                has_content = true;
            }
        })
        .unwrap();

    assert!(has_content);
}

#[test]
fn test_stream_without_content() {
    let config = StreamConfig {
        include_content: false,
        ..Default::default()
    };

    let processor = StreamProcessor::new(config);
    let data = "test@example.com";
    let reader = Cursor::new(data);

    processor
        .process(reader, |event| {
            assert!(event.content().is_none());
        })
        .unwrap();
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

    // Should only detect emails
    for finding in &result.findings {
        assert_eq!(finding.category, PiiCategory::Email);
    }
}

#[test]
fn test_empty_stream() {
    let processor = StreamProcessor::default();
    let reader = Cursor::new("");

    let mut end_stream_seen = false;
    let result = processor
        .process(reader, |event| {
            if let StreamEvent::EndOfStream { .. } = event {
                end_stream_seen = true;
            }
        })
        .unwrap();

    assert!(end_stream_seen);
    assert_eq!(result.total_bytes, 0);
    assert_eq!(result.findings.len(), 0);
}

#[test]
fn test_result_statistics() {
    use veil_detect::PiiCategory;

    let processor = StreamProcessor::default();
    let data = "Email: a@b.com and another@test.org";
    let reader = Cursor::new(data);

    let mut result = processor.process_all(reader).unwrap();

    result.finalize();

    // Should have statistics
    assert!(!result.stats_by_category.is_empty());

    // Email should be in stats
    assert!(result
        .stats_by_category
        .iter()
        .any(|s| s.category == PiiCategory::Email));
}

#[test]
fn test_chunk_size_configuration() {
    let config = StreamConfig {
        chunk_size: 1024,
        mode: ProcessingMode::ChunkByChunk,
        ..Default::default()
    };

    let processor = StreamProcessor::new(config);

    // Create data larger than chunk size
    let data = "x".repeat(5000);
    let reader = Cursor::new(data);

    let mut chunk_count = 0;
    processor
        .process(reader, |event| {
            if let StreamEvent::ChunkProcessed { .. } = event {
                chunk_count += 1;
            }
        })
        .unwrap();

    // Should have multiple chunks
    assert!(chunk_count > 1);
}

#[test]
fn test_line_numbers() {
    let processor = StreamProcessor::default();
    let data = "Line 1\nLine 2\nLine 3";
    let reader = Cursor::new(data);

    let mut line_numbers = Vec::new();
    processor
        .process(reader, |event| {
            if let StreamEvent::LineProcessed { line_number, .. } = event {
                line_numbers.push(line_number);
            }
        })
        .unwrap();

    assert_eq!(line_numbers, vec![1, 2, 3]);
}

#[test]
fn test_byte_offsets() {
    let processor = StreamProcessor::default();
    let data = "12345\n67890\nABCDE";
    let reader = Cursor::new(data);

    let mut offsets = Vec::new();
    processor
        .process(reader, |event| {
            if let StreamEvent::LineProcessed { offset, .. } = event {
                offsets.push(offset);
            }
        })
        .unwrap();

    // First line at offset 0, second at 6, third at 12
    assert_eq!(offsets, vec![0, 6, 12]);
}
