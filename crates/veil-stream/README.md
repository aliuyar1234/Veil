# veil-stream

Real-time streaming PII detection and processing for the Veil privacy framework.

## Overview

`veil-stream` provides a memory-efficient streaming interface for detecting PII in data streams. It processes data incrementally using the Rust `Read` trait, without loading entire files into memory.

## Features

- **Stream Processing**: Process data from any `Read` source (files, stdin, network, etc.)
- **Event-Based API**: Receive callbacks as PII is detected
- **Boundary Handling**: Correctly handles partial matches across chunk boundaries
- **Flexible Modes**: Line-by-line or chunk-by-chunk processing
- **Memory Efficient**: Configurable chunk sizes and buffer limits
- **Category Filtering**: Filter detection by specific PII categories

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
veil-stream = { path = "../veil-stream" }
```

## Usage

### Basic Example

```rust
use std::io::Cursor;
use veil_stream::{StreamProcessor, StreamConfig};

let data = "Email: test@example.com\nPhone: 555-1234";
let reader = Cursor::new(data);

let processor = StreamProcessor::default();
let result = processor.process_all(reader)?;

println!("Found {} PII instances", result.findings.len());
```

### With Event Callbacks

```rust
use veil_stream::{StreamProcessor, StreamEvent};

processor.process(reader, |event| {
    match event {
        StreamEvent::LineProcessed { line_number, findings, .. } => {
            println!("Line {}: {} findings", line_number, findings.len());
        }
        StreamEvent::EndOfStream { total_bytes, total_findings } => {
            println!("Done: {} bytes, {} findings", total_bytes, total_findings);
        }
        _ => {}
    }
})?;
```

### Chunk-by-Chunk Processing

```rust
use veil_stream::{StreamConfig, ProcessingMode, StreamProcessor};

let mut config = StreamConfig::default();
config.mode = ProcessingMode::ChunkByChunk;
config.chunk_size = 8192;

let processor = StreamProcessor::new(config);
let result = processor.process_all(reader)?;
```

### Category Filtering

```rust
use veil_detect::PiiCategory;
use veil_stream::StreamConfig;

let mut config = StreamConfig::default();
config.categories = vec![PiiCategory::Email, PiiCategory::Phone];

let processor = StreamProcessor::new(config);
```

## Processing Modes

### Line-by-Line

Best for text logs and line-oriented data:

```rust
config.mode = ProcessingMode::LineByLine;
```

- Emits `LineProcessed` events
- Respects line boundaries
- Suitable for log files and structured text

### Chunk-by-Chunk

Best for binary data or non-line-oriented streams:

```rust
config.mode = ProcessingMode::ChunkByChunk;
config.chunk_size = 4096;
```

- Emits `ChunkProcessed` events
- Fixed-size chunks with smart boundary detection
- Handles partial matches across chunks

## Configuration

```rust
let config = StreamConfig {
    chunk_size: 8192,              // Read buffer size
    max_buffer_size: 65536,        // Max overflow buffer
    mode: ProcessingMode::LineByLine,
    categories: vec![],            // Empty = all categories
    include_content: true,         // Include original content in events
};
```

## Examples

Run the examples:

```bash
# Basic streaming example
cargo run --example basic_streaming

# Filter stdin to stdout (redacting PII)
echo "Email: test@example.com" | cargo run --example stdin_filter
```

## Events

The processor emits these events:

- **`LineProcessed`**: A line was processed (line-by-line mode)
  - Contains: line number, offset, findings, optional content

- **`ChunkProcessed`**: A chunk was processed (chunk-by-chunk mode)
  - Contains: offset, bytes processed, findings, optional content

- **`EndOfStream`**: Stream processing completed
  - Contains: total bytes, total findings

- **`Error`**: Non-fatal error occurred
  - Contains: error message, offset

## Results

The `StreamResult` provides:

- **Total bytes processed**
- **Number of items processed** (lines or chunks)
- **All findings** across the stream
- **Statistics by category**
- **Error count**

## Performance

- **Chunk Size**: Larger chunks = fewer syscalls but more memory
- **Buffer Size**: Maximum memory for boundary handling
- **Line Mode**: Slower for files with very long lines
- **Chunk Mode**: Faster but requires careful boundary handling

## Architecture

```
┌─────────────┐
│   Reader    │ (stdin, file, network, etc.)
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│ StreamProcessor │
│  ┌───────────┐  │
│  │  Chunking │  │ (read in configurable chunks)
│  └─────┬─────┘  │
│        │        │
│  ┌─────▼─────┐  │
│  │ Boundary  │  │ (handle partial matches)
│  │ Detection │  │
│  └─────┬─────┘  │
│        │        │
│  ┌─────▼─────┐  │
│  │   Detect  │  │ (veil-detect integration)
│  │    PII    │  │
│  └─────┬─────┘  │
│        │        │
│  ┌─────▼─────┐  │
│  │   Events  │  │ (emit to callback)
│  └───────────┘  │
└─────────────────┘
       │
       ▼
   Callback
   (user code)
```

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test streaming_tests

# Run with output
cargo test -- --nocapture
```

## License

MIT OR Apache-2.0
