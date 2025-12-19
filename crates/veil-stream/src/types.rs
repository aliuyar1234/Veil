//! Core types for stream processing.

use serde::{Deserialize, Serialize};
use veil_detect::{Finding, PiiCategory};

/// Configuration for stream processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Size of chunks to read from the stream (in bytes).
    /// Default: 8192 bytes (8KB)
    pub chunk_size: usize,

    /// Maximum buffer size for handling partial matches across boundaries.
    /// Default: 65536 bytes (64KB)
    pub max_buffer_size: usize,

    /// Processing mode: line-by-line or chunk-by-chunk.
    pub mode: ProcessingMode,

    /// Categories of PII to detect. If empty, all categories are detected.
    pub categories: Vec<PiiCategory>,

    /// Whether to include the original content in events.
    pub include_content: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            chunk_size: 8192,
            max_buffer_size: 65536,
            mode: ProcessingMode::LineByLine,
            categories: Vec::new(),
            include_content: false,
        }
    }
}

/// Processing mode for stream data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessingMode {
    /// Process data line by line (for text logs).
    LineByLine,
    /// Process data in fixed-size chunks.
    ChunkByChunk,
}

/// Event emitted during stream processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    /// A chunk of data was processed.
    ChunkProcessed {
        /// Byte offset in the stream where this chunk started.
        offset: u64,
        /// Number of bytes processed in this chunk.
        bytes_processed: usize,
        /// PII findings in this chunk.
        findings: Vec<Finding>,
        /// Original content (if include_content is true).
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
    },

    /// A line of data was processed (only in LineByLine mode).
    LineProcessed {
        /// Line number (1-indexed).
        line_number: usize,
        /// Byte offset in the stream where this line started.
        offset: u64,
        /// PII findings in this line.
        findings: Vec<Finding>,
        /// Original line content (if include_content is true).
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
    },

    /// End of stream reached.
    EndOfStream {
        /// Total bytes processed.
        total_bytes: u64,
        /// Total number of findings across the entire stream.
        total_findings: usize,
    },

    /// An error occurred during processing (non-fatal).
    Error {
        /// Error message.
        message: String,
        /// Byte offset where the error occurred.
        offset: u64,
    },
}

impl StreamEvent {
    /// Get the findings from this event, if any.
    pub fn findings(&self) -> &[Finding] {
        match self {
            StreamEvent::ChunkProcessed { findings, .. } => findings,
            StreamEvent::LineProcessed { findings, .. } => findings,
            _ => &[],
        }
    }

    /// Check if this event contains any PII findings.
    pub fn has_findings(&self) -> bool {
        !self.findings().is_empty()
    }

    /// Get the content from this event, if available.
    pub fn content(&self) -> Option<&str> {
        match self {
            StreamEvent::ChunkProcessed { content, .. } => content.as_deref(),
            StreamEvent::LineProcessed { content, .. } => content.as_deref(),
            _ => None,
        }
    }
}

/// Result of processing an entire stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamResult {
    /// Total number of bytes processed.
    pub total_bytes: u64,

    /// Total number of chunks or lines processed.
    pub items_processed: usize,

    /// All findings across the entire stream.
    pub findings: Vec<Finding>,

    /// Number of errors encountered (non-fatal).
    pub error_count: usize,

    /// Processing statistics by category.
    pub stats_by_category: Vec<CategoryStats>,
}

impl StreamResult {
    /// Create a new empty result.
    pub fn new() -> Self {
        Self {
            total_bytes: 0,
            items_processed: 0,
            findings: Vec::new(),
            error_count: 0,
            stats_by_category: Vec::new(),
        }
    }

    /// Add findings from a processing event.
    pub fn add_findings(&mut self, findings: Vec<Finding>, bytes: usize) {
        self.total_bytes += bytes as u64;
        self.items_processed += 1;
        self.findings.extend(findings);
    }

    /// Finalize the result by computing statistics.
    pub fn finalize(&mut self) {
        use std::collections::HashMap;

        let mut category_counts: HashMap<PiiCategory, usize> = HashMap::new();
        for finding in &self.findings {
            *category_counts.entry(finding.category.clone()).or_insert(0) += 1;
        }

        self.stats_by_category = category_counts
            .into_iter()
            .map(|(category, count)| CategoryStats { category, count })
            .collect();

        // Sort by count descending
        self.stats_by_category.sort_by(|a, b| b.count.cmp(&a.count));
    }
}

impl Default for StreamResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a specific PII category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    /// The PII category.
    pub category: PiiCategory,
    /// Number of findings for this category.
    pub count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StreamConfig::default();
        assert_eq!(config.chunk_size, 8192);
        assert_eq!(config.max_buffer_size, 65536);
        assert_eq!(config.mode, ProcessingMode::LineByLine);
        assert!(!config.include_content);
    }

    #[test]
    fn test_stream_event_findings() {
        let event = StreamEvent::ChunkProcessed {
            offset: 0,
            bytes_processed: 100,
            findings: vec![],
            content: None,
        };
        assert!(!event.has_findings());
        assert_eq!(event.findings().len(), 0);
    }

    #[test]
    fn test_stream_result_accumulation() {
        let mut result = StreamResult::new();
        result.add_findings(vec![], 100);
        result.add_findings(vec![], 200);

        assert_eq!(result.total_bytes, 300);
        assert_eq!(result.items_processed, 2);
    }
}
