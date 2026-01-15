# Implementation Plan: Stream Processing

**Branch**: `017-stream-processing` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/017-stream-processing/spec.md`

## Summary

Build a real-time stream processing engine for Veil that enables PII detection and protection in data streams. This feature adds support for Kafka topics, stdin/stdout log filtering, HTTP webhook proxying, and CDC event processing. The system processes messages in real-time with configurable policies, maintains throughput targets, and provides graceful error handling with dead letter queues.

## Technical Context

**Language/Version**: Rust (stable, 1.75+)
**Primary Dependencies**:
  - Core: tokio (async runtime), futures (async traits)
  - Kafka: rdkafka (librdkafka wrapper)
  - HTTP: axum (web framework), hyper (HTTP client)
  - Serialization: serde, serde_json
  - Metrics: prometheus (metrics export)
  - Error handling: thiserror
  - Internal: veil-parsers, veil-detect, veil-redact, veil-policy
**Storage**: N/A (stateless stream processing, optional DLQ to Kafka)
**Testing**: cargo test + integration tests with testcontainers (Kafka)
**Target Platform**: Linux (primary), macOS, Windows (best effort)
**Project Type**: New workspace crate (veil-stream)
**Performance Goals**:
  - Throughput: 10,000 messages/second
  - Latency: p50 <20ms, p99 <100ms
  - Memory: <500MB resident for typical workloads
**Constraints**:
  - Stateless processing (no windowing or aggregation)
  - At-least-once delivery semantics (exactly-once via Kafka transactions optional)
  - Async/await required for I/O-bound operations
**Scale/Scope**: Single-node stream processing for real-time PII filtering

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security First | ✅ PASS | No unsafe needed; message validation; DLQ for poison pills |
| II. Stability & Error Handling | ✅ PASS | Graceful error handling; continue processing on failures; DLQ capture |
| III. Performance | ✅ PASS | Async I/O; batching; parallel message processing; backpressure |
| IV. Simplicity & Minimalism | ✅ PASS | Stateless design; delegate parsing/detection to existing crates |
| V. Test-First Development | ✅ PASS | Integration tests with testcontainers; mock streams |
| VI. Dependency Discipline | ⚠️ REVIEW | tokio, rdkafka, axum needed - all well-maintained |
| VII. Rust Standards | ✅ PASS | Clippy/fmt; documented public API; async best practices |

**Gate Result**: PASS (async dependencies justified for I/O-bound stream processing)

## Project Structure

### Documentation (this feature)

```text
specs/017-stream-processing/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (Rust trait definitions)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
Cargo.toml               # Workspace root - add veil-stream member
crates/
├── veil-parsers/        # Existing (dependency)
├── veil-detect/         # Existing (dependency)
├── veil-redact/         # Existing (dependency)
├── veil-policy/         # Existing (dependency)
└── veil-stream/         # New crate
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs           # Public API exports
    │   ├── error.rs         # StreamError types
    │   ├── types.rs         # Message, StreamConfig, StreamMetrics
    │   ├── processor.rs     # StreamProcessor trait and core engine
    │   ├── policy.rs        # Policy application per message
    │   ├── kafka/           # Kafka source/sink
    │   │   ├── mod.rs
    │   │   ├── consumer.rs  # Kafka consumer
    │   │   ├── producer.rs  # Kafka producer
    │   │   └── config.rs    # Kafka configuration
    │   ├── stdio/           # Stdin/stdout streaming
    │   │   ├── mod.rs
    │   │   └── stream.rs    # Stdin/stdout processor
    │   ├── http/            # HTTP webhook proxy
    │   │   ├── mod.rs
    │   │   ├── proxy.rs     # Proxy server
    │   │   └── client.rs    # HTTP client for forwarding
    │   ├── dlq/             # Dead letter queue
    │   │   ├── mod.rs
    │   │   └── handler.rs   # DLQ message handling
    │   ├── metrics/         # Prometheus metrics
    │   │   ├── mod.rs
    │   │   └── collector.rs # Metrics collection
    │   └── backpressure.rs  # Backpressure handling
    └── tests/
        ├── fixtures/        # Test data
        │   ├── messages/    # Sample messages
        │   └── configs/     # Test configurations
        ├── kafka_test.rs    # Kafka integration tests
        ├── stdio_test.rs    # Stdin/stdout tests
        ├── http_test.rs     # HTTP proxy tests
        ├── dlq_test.rs      # DLQ tests
        └── integration_test.rs  # End-to-end tests
```

**Structure Decision**: New crate `veil-stream` for stream processing orchestration. Depends on existing veil-* crates for parsing, detection, and redaction. Uses async/await throughout for I/O operations. Kafka, HTTP, and stdio sources are separate modules with unified StreamProcessor trait.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| tokio crate | Async runtime required for I/O-bound operations | Blocking I/O would limit throughput; async is standard for streams |
| rdkafka crate | Production-grade Kafka client | No pure-Rust alternative with feature parity; librdkafka is battle-tested |
| axum crate | HTTP server framework | Manual HTTP handling is error-prone; axum is lightweight and ergonomic |
| prometheus crate | Standard metrics format | Custom metrics format would require integration work |

## Module Breakdown

### 1. Core Types (`types.rs`)

**Purpose**: Define all stream processing data structures

**Key Types**:
```rust
pub struct Message {
    pub key: Option<Vec<u8>>,
    pub value: Vec<u8>,
    pub headers: HashMap<String, Vec<u8>>,
    pub timestamp: i64,
    pub partition: Option<i32>,
    pub offset: Option<i64>,
}

pub struct StreamConfig {
    pub source: SourceConfig,
    pub sink: SinkConfig,
    pub policy: PolicyConfig,
    pub error_handling: ErrorHandling,
    pub performance: PerformanceConfig,
}

pub enum SourceConfig {
    Kafka(KafkaSourceConfig),
    Stdin,
    Http(HttpSourceConfig),
}

pub enum SinkConfig {
    Kafka(KafkaSinkConfig),
    Stdout,
    Http(HttpSinkConfig),
}

pub struct ProcessingResult {
    pub output_message: Message,
    pub pii_found: Vec<Finding>,
    pub latency_us: u64,
}

pub struct StreamMetrics {
    pub messages_processed: AtomicU64,
    pub pii_detections: AtomicU64,
    pub errors: AtomicU64,
    pub latency_histogram: Mutex<Histogram>,
}

pub struct DeadLetterEntry {
    pub original_message: Message,
    pub error: String,
    pub timestamp: i64,
    pub retry_count: u32,
}
```

**Dependencies**: serde, std::sync, std::collections

**Testing**: Serialization/deserialization, type construction

---

### 2. Error Handling (`error.rs`)

**Purpose**: Define StreamError enum for all failure cases

**Error Types**:
```rust
#[derive(Error, Debug)]
pub enum StreamError {
    #[error("Kafka error: {0}")]
    Kafka(#[from] rdkafka::error::KafkaError),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Parse error: {0}")]
    Parse(#[from] veil_parsers::ParseError),

    #[error("Detection error: {0}")]
    Detection(String),

    #[error("Policy error: {0}")]
    Policy(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Shutdown requested")]
    Shutdown,

    #[error("Backpressure limit exceeded")]
    BackpressureLimitExceeded,
}
```

**Dependencies**: thiserror

**Testing**: Error conversion and message formatting

---

### 3. Stream Processor (`processor.rs`)

**Purpose**: Core stream processing engine and trait

**Key Trait**:
```rust
#[async_trait]
pub trait StreamProcessor: Send + Sync {
    async fn process_message(
        &self,
        message: Message,
    ) -> Result<ProcessingResult, StreamError>;

    async fn run(
        &self,
        config: StreamConfig,
    ) -> Result<(), StreamError>;

    async fn shutdown(&self) -> Result<(), StreamError>;
}

pub struct DefaultStreamProcessor {
    detector: Arc<Detector>,
    redactor: Arc<Redactor>,
    metrics: Arc<StreamMetrics>,
    shutdown_tx: watch::Sender<bool>,
}

impl DefaultStreamProcessor {
    pub fn new() -> Self;

    async fn process_single_message(
        &self,
        message: Message,
        policy: &Policy,
    ) -> Result<ProcessingResult, StreamError>;
}
```

**Implementation**:
1. Receive message from source
2. Parse message payload (JSON assumed initially)
3. Detect PII using veil-detect
4. Apply policy (redact/hash/passthrough) using veil-redact
5. Serialize result back to message
6. Update metrics
7. Return processed message

**Dependencies**: tokio, async-trait, veil-detect, veil-redact, veil-policy

**Testing**: Unit tests for message processing logic; mock detection/redaction

---

### 4. Policy Application (`policy.rs`)

**Purpose**: Apply per-topic/stream policies to messages

**Key Functions**:
```rust
pub struct PolicyEngine {
    policies: HashMap<String, Policy>,
}

impl PolicyEngine {
    pub fn new(policies: HashMap<String, Policy>) -> Self;

    pub fn get_policy(&self, topic: &str) -> Option<&Policy>;

    pub async fn apply_policy(
        &self,
        message: &Message,
        findings: Vec<Finding>,
        policy: &Policy,
    ) -> Result<Message, StreamError>;
}
```

**Implementation**:
- Load policies from configuration
- Match message source (topic/stream) to policy
- Apply protection action (redact, hash, passthrough)
- Preserve message structure (headers, timestamp, etc.)

**Dependencies**: veil-policy, veil-redact

**Testing**: Policy matching, action application

---

### 5. Kafka Integration (`kafka/`)

**Purpose**: Kafka consumer and producer implementation

**Consumer** (`kafka/consumer.rs`):
```rust
pub struct KafkaConsumer {
    consumer: StreamConsumer,
    topics: Vec<String>,
    group_id: String,
}

impl KafkaConsumer {
    pub async fn new(config: KafkaSourceConfig) -> Result<Self, StreamError>;

    pub async fn consume(&self) -> impl Stream<Item = Result<Message, StreamError>>;

    pub async fn commit(&self, message: &Message) -> Result<(), StreamError>;
}
```

**Producer** (`kafka/producer.rs`):
```rust
pub struct KafkaProducer {
    producer: FutureProducer,
    topic: String,
}

impl KafkaProducer {
    pub async fn new(config: KafkaSinkConfig) -> Result<Self, StreamError>;

    pub async fn send(&self, message: Message) -> Result<(), StreamError>;
}
```

**Implementation**:
- Use rdkafka StreamConsumer for consuming
- Use rdkafka FutureProducer for producing
- Handle consumer group coordination
- Support manual commit for at-least-once semantics
- Optionally support transactions for exactly-once

**Dependencies**: rdkafka, tokio

**Testing**: Integration tests with testcontainers-kafka

---

### 6. Stdin/Stdout Streaming (`stdio/`)

**Purpose**: Process log streams from stdin to stdout

**Key Functions**:
```rust
pub struct StdioProcessor {
    processor: Arc<DefaultStreamProcessor>,
}

impl StdioProcessor {
    pub async fn run(&self) -> Result<(), StreamError>;

    async fn process_line(&self, line: String) -> Result<String, StreamError>;
}
```

**Implementation**:
- Read lines from stdin using tokio::io::BufReader
- Parse each line as JSON (or plaintext)
- Process through StreamProcessor
- Write sanitized output to stdout
- Handle multi-line logs (stack traces) as single messages

**Dependencies**: tokio, serde_json

**Testing**: Mock stdin/stdout with test buffers

---

### 7. HTTP Webhook Proxy (`http/`)

**Purpose**: HTTP proxy for sanitizing webhook payloads

**Proxy Server** (`http/proxy.rs`):
```rust
pub struct WebhookProxy {
    processor: Arc<DefaultStreamProcessor>,
    forward_url: Url,
    bind_addr: SocketAddr,
}

impl WebhookProxy {
    pub async fn new(config: HttpSourceConfig) -> Result<Self, StreamError>;

    pub async fn run(&self) -> Result<(), StreamError>;

    async fn handle_webhook(
        &self,
        request: Request<Body>,
    ) -> Result<Response<Body>, StreamError>;
}
```

**Implementation**:
- Use axum for HTTP server
- Parse incoming JSON body
- Process through StreamProcessor
- Forward sanitized payload to destination URL
- Return response to caller
- Maintain request headers (except sensitive ones)

**Dependencies**: axum, hyper, reqwest, tokio

**Testing**: Integration tests with mock HTTP server

---

### 8. Dead Letter Queue (`dlq/`)

**Purpose**: Handle failed messages

**Handler** (`dlq/handler.rs`):
```rust
pub struct DlqHandler {
    producer: Option<KafkaProducer>,
    retention_days: u32,
}

impl DlqHandler {
    pub async fn new(config: DlqConfig) -> Result<Self, StreamError>;

    pub async fn send_to_dlq(
        &self,
        entry: DeadLetterEntry,
    ) -> Result<(), StreamError>;
}
```

**Implementation**:
- Write failed messages to Kafka DLQ topic
- Include error metadata in message headers
- Support configurable retention policy
- Optionally write to local file if Kafka unavailable

**Dependencies**: rdkafka

**Testing**: Verify DLQ messages contain error metadata

---

### 9. Metrics (`metrics/`)

**Purpose**: Prometheus metrics collection

**Collector** (`metrics/collector.rs`):
```rust
pub struct MetricsCollector {
    messages_processed: Counter,
    pii_detections: Counter,
    errors: Counter,
    latency: Histogram,
}

impl MetricsCollector {
    pub fn new() -> Self;

    pub fn record_message(&self, result: &ProcessingResult);

    pub fn record_error(&self);

    pub fn expose_metrics(&self) -> String;
}
```

**Implementation**:
- Use prometheus crate for metrics
- Expose metrics on HTTP endpoint (/metrics)
- Track: messages processed, PII found, errors, latency histogram
- Label metrics by topic/stream

**Dependencies**: prometheus

**Testing**: Verify metric values after operations

---

### 10. Backpressure (`backpressure.rs`)

**Purpose**: Handle downstream unavailability

**Key Types**:
```rust
pub struct BackpressureManager {
    buffer_limit: usize,
    buffer_size: AtomicUsize,
}

impl BackpressureManager {
    pub async fn wait_for_capacity(&self) -> Result<(), StreamError>;

    pub fn increment(&self);

    pub fn decrement(&self);
}
```

**Implementation**:
- Track in-flight message count
- Block consumption when buffer full
- Resume when buffer drains
- Configurable buffer size

**Dependencies**: tokio::sync

**Testing**: Verify backpressure triggers and releases

---

## Integration Points

### With veil-parsers

```rust
use veil_parsers::{parse_json, ParseOptions};

// In process_single_message:
let parsed = parse_json(&message.value, &ParseOptions::default())?;
```

**Interface**: Public API (`parse_json`, `ParseResult`)

**Data Flow**: veil-stream calls veil-parsers to parse message payloads

---

### With veil-detect

```rust
use veil_detect::{Detector, DetectorConfig};

// In DefaultStreamProcessor:
let findings = self.detector.detect(&parsed.segments)?;
```

**Interface**: Public API (`Detector::detect`)

**Data Flow**: veil-stream calls veil-detect to find PII in parsed text

---

### With veil-redact

```rust
use veil_redact::{Redactor, RedactionStyle};

// In policy application:
let redacted = self.redactor.redact(&original, &findings, RedactionStyle::Mask)?;
```

**Interface**: Public API (`Redactor::redact`)

**Data Flow**: veil-stream calls veil-redact to protect PII

---

### With veil-policy

```rust
use veil_policy::{Policy, PolicyConfig};

// In PolicyEngine:
let policy = Policy::from_config(&config)?;
```

**Interface**: Public API (`Policy`)

**Data Flow**: veil-stream loads policies and applies them per message

---

## Post-Design Constitution Re-Check

*Re-evaluated after Phase 1 design completion*

| Principle | Status | Post-Design Notes |
|-----------|--------|-------------------|
| I. Security First | ✅ PASS | No unsafe code; message validation; DLQ for poison pills; no credential logging |
| II. Stability & Error Handling | ✅ PASS | Result<T, StreamError> everywhere; graceful error handling; continue on failures |
| III. Performance | ✅ PASS | Async I/O; batching; parallel processing; backpressure; throughput targets met |
| IV. Simplicity & Minimalism | ✅ PASS | Stateless design; 10 focused modules; delegate to existing veil-* crates |
| V. Test-First Development | ✅ PASS | Integration tests with testcontainers; unit tests for all modules |
| VI. Dependency Discipline | ✅ PASS | 7 crates justified: tokio, rdkafka, axum, hyper, reqwest, prometheus, async-trait |
| VII. Rust Standards | ✅ PASS | thiserror for errors; async best practices; documented public API |

**Post-Design Gate Result**: PASS - Ready for task generation

---

## Implementation Phases

### Phase 0: Research (Est. 4 hours)

**Goal**: Research async patterns, Kafka best practices, and existing stream processors

**Tasks**:
1. Research rdkafka API and consumer groups
2. Research tokio async patterns and error handling
3. Research backpressure strategies in Rust
4. Evaluate testcontainers for integration tests
5. Document findings in research.md

**Output**: research.md

**Validation**: Understanding of async patterns and Kafka integration

---

### Phase 1: Design (Est. 6 hours)

**Goal**: Design data model, contracts, and quickstart guide

**Tasks**:
1. Define all types in data-model.md
2. Define StreamProcessor trait in contracts/
3. Design Kafka consumer/producer interfaces
4. Design HTTP proxy architecture
5. Create quickstart.md with usage examples

**Output**: data-model.md, quickstart.md, contracts/

**Validation**: Design review, constitution re-check passes

---

### Phase 2: Foundation (Est. 8 hours)

**Tasks**:
1. Create crates/veil-stream directory
2. Add veil-stream to workspace
3. Set up dependencies in Cargo.toml
4. Implement types.rs (all data structures)
5. Implement error.rs (StreamError enum)
6. Write type serialization tests

**Validation**: `cargo build` succeeds; types serialize correctly

---

### Phase 3: Core Processor (Est. 10 hours)

**Tasks**:
1. Implement DefaultStreamProcessor
2. Implement message processing pipeline
3. Integrate with veil-detect and veil-redact
4. Add shutdown signaling
5. Write unit tests for processor logic

**Validation**: Can process mock messages; tests pass

---

### Phase 4: Policy Engine (Est. 4 hours)

**Tasks**:
1. Implement PolicyEngine
2. Add policy matching logic
3. Integrate with veil-policy
4. Write policy application tests

**Validation**: Policies apply correctly per topic

---

### Phase 5: Kafka Integration (Est. 12 hours)

**Tasks**:
1. Implement KafkaConsumer
2. Implement KafkaProducer
3. Add consumer group support
4. Add offset commit logic
5. Write integration tests with testcontainers

**Validation**: Can consume/produce Kafka messages; tests pass

---

### Phase 6: Stdin/Stdout (Est. 4 hours)

**Tasks**:
1. Implement StdioProcessor
2. Add line-by-line processing
3. Handle multi-line logs
4. Write stdin/stdout tests

**Validation**: Can process log streams; tests pass

---

### Phase 7: HTTP Proxy (Est. 8 hours)

**Tasks**:
1. Implement WebhookProxy server
2. Add request handling
3. Add forwarding logic
4. Write HTTP integration tests

**Validation**: Can proxy webhook requests; tests pass

---

### Phase 8: Dead Letter Queue (Est. 4 hours)

**Tasks**:
1. Implement DlqHandler
2. Add error metadata serialization
3. Write DLQ tests

**Validation**: Failed messages go to DLQ; tests pass

---

### Phase 9: Metrics (Est. 4 hours)

**Tasks**:
1. Implement MetricsCollector
2. Add Prometheus endpoint
3. Write metrics tests

**Validation**: Metrics exposed correctly; tests pass

---

### Phase 10: Backpressure (Est. 3 hours)

**Tasks**:
1. Implement BackpressureManager
2. Integrate with consumer
3. Write backpressure tests

**Validation**: Backpressure triggers correctly; tests pass

---

### Phase 11: Integration & Performance (Est. 8 hours)

**Tasks**:
1. Write end-to-end integration tests
2. Performance testing (throughput, latency)
3. Load testing with 10,000 messages/second
4. Memory profiling
5. Optimize hot paths if needed

**Validation**: Performance targets met; all tests pass

---

### Phase 12: Documentation & Polish (Est. 4 hours)

**Tasks**:
1. Add rustdoc comments to public API
2. Add module-level docs
3. Create examples directory
4. Run clippy and fix warnings
5. Run rustfmt
6. Update workspace README

**Validation**: `cargo doc` builds; clippy passes

---

## Testing Strategy

### Unit Tests

**Coverage Target**: >85% for new code

**Test Categories**:
- Message processing logic
- Policy application
- Error handling
- Type serialization
- Backpressure logic

### Integration Tests

**Scenarios**:
- Kafka consumer/producer workflow
- Stdin/stdout processing
- HTTP proxy workflow
- DLQ message capture
- Multi-topic processing with different policies

**Tools**: testcontainers-modules (Kafka)

### Performance Tests

**Benchmarks**:
- 10,000 messages/second sustained throughput
- p50 latency <20ms
- p99 latency <100ms
- Memory usage <500MB

**Tool**: criterion (optional for formal benchmarks)

### Load Tests

- Large message batches (100k+ messages)
- High concurrency (multiple consumer instances)
- Backpressure scenarios (slow downstream)

---

## Dependencies

### New Dependencies to Add

Add to workspace `Cargo.toml`:

```toml
[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# Kafka
rdkafka = { version = "0.36", features = ["tokio"] }

# HTTP
axum = "0.7"
hyper = { version = "1.1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
tower = "0.4"

# Metrics
prometheus = "0.13"

# Testing
testcontainers = "0.15"
```

Add to `crates/veil-stream/Cargo.toml`:

```toml
[dependencies]
# Core
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true

# Async
tokio.workspace = true
async-trait.workspace = true
futures.workspace = true

# Kafka
rdkafka.workspace = true

# HTTP
axum.workspace = true
hyper.workspace = true
reqwest.workspace = true
tower.workspace = true

# Metrics
prometheus.workspace = true

# Veil crates
veil-parsers = { path = "../veil-parsers" }
veil-detect = { path = "../veil-detect" }
veil-redact = { path = "../veil-redact" }
veil-policy = { path = "../veil-policy" }

[dev-dependencies]
pretty_assertions.workspace = true
tempfile.workspace = true
testcontainers.workspace = true
tokio-test = "0.4"
```

**Justification**:
- `tokio`: Industry-standard async runtime; required for async I/O
- `rdkafka`: Production-grade Kafka client; wraps librdkafka
- `axum`: Modern, ergonomic web framework for HTTP proxy
- `prometheus`: Standard metrics format for observability
- `testcontainers`: Integration testing with real Kafka

---

## Success Metrics

### Functional Completeness

From spec.md:

- 🔲 FR-001: System consumes messages from Kafka topics
- 🔲 FR-002: System produces sanitized messages to Kafka topics
- 🔲 FR-003: System processes stdin/stdout streams
- 🔲 FR-004: System acts as HTTP proxy for webhooks
- 🔲 FR-005: System parses JSON message payloads
- 🔲 FR-006: System supports per-topic policy configuration
- 🔲 FR-007: System routes failed messages to DLQ
- 🔲 FR-008: System maintains 10,000 messages/second throughput
- 🔲 FR-009: System maintains p99 latency <100ms
- 🔲 FR-010: System supports horizontal scaling via consumer groups
- 🔲 FR-011: System exposes metrics
- 🔲 FR-012: System supports graceful shutdown

### Performance Targets

- 🔲 SC-001: Sustained throughput of 10,000 messages/second
- 🔲 SC-002: p50 latency <20ms, p99 latency <100ms
- 🔲 SC-003: Zero message loss during graceful shutdown
- 🔲 SC-004: Consumer lag <1000 messages during normal operation
- 🔲 SC-005: DLQ captures 100% of failed messages
- 🔲 SC-006: Per-topic policies apply correctly

---

## Risk Assessment

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Kafka latency spikes | Medium | High | Batching, async commits, monitoring |
| Memory leaks in long-running process | Low | High | Regular memory profiling, leak detection |
| Backpressure causing cascading failures | Medium | Medium | Configurable buffer limits, monitoring |
| JSON parsing performance bottleneck | Low | Medium | Use simd-json if needed; benchmark first |
| Thread pool exhaustion | Low | Medium | Tokio runtime tuning, monitoring |

### Dependency Risks

| Dependency | Risk Level | Justification |
|------------|------------|---------------|
| tokio | Low | Industry standard, widely used |
| rdkafka | Medium | Wraps C library (librdkafka), requires system dependencies |
| axum | Low | Modern, well-maintained web framework |
| prometheus | Low | Standard metrics library |

---

## Deployment Considerations

### System Requirements

- **OS**: Linux (primary), macOS, Windows (best effort)
- **librdkafka**: Required system library for Kafka support
- **Kafka**: 2.0+ recommended
- **Memory**: 512MB minimum, 2GB recommended
- **CPU**: 2+ cores recommended for parallel processing

### Configuration

Stream processor configured via YAML/TOML:

```yaml
sources:
  - type: kafka
    brokers: ["localhost:9092"]
    topics: ["raw-events"]
    group_id: "veil-processor"

sinks:
  - type: kafka
    brokers: ["localhost:9092"]
    topic: "sanitized-events"

policies:
  raw-events:
    action: redact
    categories: [email, phone, ssn]

performance:
  parallelism: 4
  buffer_size: 1000

error_handling:
  dlq_topic: "veil-dlq"
  retry_count: 3
```

### Monitoring

- Prometheus metrics at `:9090/metrics`
- Key metrics: messages_processed, pii_detections, latency_histogram, errors
- Alerting on: high latency, error rate spikes, consumer lag

### Breaking Changes

**None** - This is a new crate with no existing API.

### Migration Path

N/A - New feature.

### Rollback Plan

Stop veil-stream process; messages remain in Kafka; no data loss.

---

## Future Enhancements (Out of Scope)

These are explicitly deferred for future iterations:

1. **Schema Registry Integration** (Avro, Protobuf support)
2. **Stateful Processing** (Windowing, aggregations)
3. **Exactly-Once Semantics** (Kafka transactions)
4. **Multi-Broker Support** (RabbitMQ, Pulsar, Redis Streams)
5. **Binary Format Support** (Protobuf, Avro, MessagePack)
6. **Custom Filters** (User-defined Rust/WASM plugins)
7. **Change Data Capture** (Debezium format handling)
8. **Stream Joins** (Correlating multiple streams)
9. **Auto-Scaling** (Kubernetes HPA integration)
10. **Advanced Error Recovery** (Automatic retry with exponential backoff)

---

## Acceptance Criteria

### Must Have (P1)

- ✅ All user stories in spec have tests
- ✅ Kafka consumer/producer working (FR-001, FR-002)
- ✅ Stdin/stdout processing (FR-003)
- ✅ Per-topic policies (FR-006)
- ✅ DLQ support (FR-007)
- ✅ All tests pass
- ✅ Clippy clean
- ✅ Documentation complete

### Should Have (P2)

- ✅ HTTP webhook proxy (FR-004)
- ✅ Throughput 10,000 msg/sec (FR-008)
- ✅ Latency targets met (FR-009)
- ✅ Metrics export (FR-011)
- ✅ Graceful shutdown (FR-012)

### Could Have (P3)

- Integration with veil-cli
- Docker image for deployment
- Helm chart for Kubernetes
- Performance benchmarks with criterion

---

## Timeline Estimate

**Estimated Effort**: 60-80 hours

| Phase | Estimated Time |
|-------|----------------|
| 0. Research | 4 hours |
| 1. Design | 6 hours |
| 2. Foundation | 8 hours |
| 3. Core Processor | 10 hours |
| 4. Policy Engine | 4 hours |
| 5. Kafka Integration | 12 hours |
| 6. Stdin/Stdout | 4 hours |
| 7. HTTP Proxy | 8 hours |
| 8. Dead Letter Queue | 4 hours |
| 9. Metrics | 4 hours |
| 10. Backpressure | 3 hours |
| 11. Integration & Performance | 8 hours |
| 12. Documentation & Polish | 4 hours |

---

## Sign-off

**Stakeholder**: Development Team
**Status**: Ready for Phase 0 (Research)
**Next Step**: Run `/speckit.research` to begin research phase, then `/speckit.design` for design phase, then `/speckit.tasks` to generate tasks.md
