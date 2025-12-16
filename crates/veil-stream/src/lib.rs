//! # veil-stream
//!
//! Real-time streaming PII detection and processing library.
//!
//! This crate provides a streaming interface for detecting PII in data streams,
//! processing data incrementally without loading entire files into memory.
//!
//! ## Features
//!
//! - Process data as a stream using the `Read` trait
//! - Emit events as PII is detected
//! - Handle partial matches across chunk boundaries
//! - Configurable chunk sizes
//! - Memory-efficient processing
//!
//! ## Example
//!
//! ```rust,ignore
//! use veil_stream::{StreamProcessor, StreamConfig};
//! use std::io::Cursor;
//!
//! let data = "Email: test@example.com\nPhone: 555-1234";
//! let reader = Cursor::new(data);
//! let config = StreamConfig::default();
//!
//! let mut processor = StreamProcessor::new(config);
//! processor.process(reader, |event| {
//!     println!("Found PII: {:?}", event);
//! })?;
//! ```

mod error;
mod processor;
mod types;

pub use error::StreamError;
pub use processor::StreamProcessor;
pub use types::{ProcessingMode, StreamConfig, StreamEvent, StreamResult};

/// Result type for stream operations.
pub type Result<T> = std::result::Result<T, StreamError>;
