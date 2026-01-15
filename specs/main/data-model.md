# Data Model: Veil MVP

**Date**: 2025-12-15 | **Plan**: specs/main/plan.md

## Entity Overview

```text
┌─────────────────────────────────────────────────────────────────────┐
│                           PARSING (001)                              │
│  Document ──parse──▶ ParseResult ──contains──▶ [TextSegment]        │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          DETECTION (002)                             │
│  TextSegment ──detect──▶ [Finding]                                  │
│  DetectorRegistry ──contains──▶ [Detector]                          │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          REDACTION (003)                             │
│  (Text, [Finding]) ──redact──▶ RedactionResult                      │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                            POLICY (009)                              │
│  Policy ──contains──▶ [DetectionRule] + [ProtectionRule]            │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                            AUDIT (011)                               │
│  AuditLogger ──append──▶ AuditLog ──contains──▶ [AuditEntry]        │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 001: Parsing Entities

### Document

The input file to be parsed.

```rust
pub struct Document {
    /// File path (if from filesystem)
    pub path: Option<PathBuf>,

    /// Detected or specified format
    pub format: DocumentFormat,

    /// Original encoding (if detected)
    pub encoding: String,

    /// File size in bytes
    pub size: u64,
}

pub enum DocumentFormat {
    PlainText,
    Csv { delimiter: char, has_headers: bool },
    Json,
    Html,
}
```

### TextSegment

A piece of extracted text with position metadata.

```rust
pub struct TextSegment {
    /// The extracted text content
    pub content: String,

    /// Position in the original document
    pub position: Position,

    /// Original byte offset (for reconstruction)
    pub byte_offset: usize,

    /// Original byte length
    pub byte_length: usize,
}

pub enum Position {
    /// Plain text: line number (1-indexed), character offset in line
    Text { line: usize, column: usize },

    /// CSV: row number (0-indexed), column index, optional column name
    Csv { row: usize, col: usize, header: Option<String> },

    /// JSON: path notation (e.g., "$.users[0].email")
    Json { path: String },

    /// HTML: approximate character offset (after tag stripping)
    Html { offset: usize },
}
```

### ParseResult

The output of parsing a document.

```rust
pub struct ParseResult {
    /// Source document metadata
    pub document: Document,

    /// Extracted text segments
    pub segments: Vec<TextSegment>,

    /// Warnings encountered during parsing
    pub warnings: Vec<ParseWarning>,
}

pub struct ParseWarning {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}
```

---

## 002: Detection Entities

### Finding

A detected PII instance.

```rust
pub struct Finding {
    /// The matched text
    pub matched_text: String,

    /// PII category (e.g., "email", "iban", "phone")
    pub category: PiiCategory,

    /// Start position in the segment's content
    pub start: usize,

    /// End position (exclusive) in the segment's content
    pub end: usize,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Validation status
    pub validation: ValidationStatus,

    /// Reference back to source segment
    pub segment_index: usize,
}

pub enum PiiCategory {
    Email,
    Iban,
    Phone,
    CreditCard,
    SvnrAt,  // Austrian social security
    SvnrDe,  // German social security
    TaxId,
    Ipv4,
    Ipv6,
    MacAddress,
    Custom(String),
}

pub enum ValidationStatus {
    /// Pattern matched and validation passed
    Valid,
    /// Pattern matched but validation failed (e.g., wrong checksum)
    Invalid { reason: String },
    /// Pattern matched, no validation available
    Unvalidated,
}
```

### Detector

A pattern matcher for a specific PII type.

```rust
pub trait Detector: Send + Sync {
    /// Unique name for this detector
    fn name(&self) -> &str;

    /// PII category this detector finds
    fn category(&self) -> PiiCategory;

    /// Find all matches in the given text
    fn detect(&self, text: &str) -> Vec<Match>;

    /// Validate a potential match (checksum, format, etc.)
    fn validate(&self, matched: &str) -> ValidationStatus;

    /// Base confidence score for this detector
    fn base_confidence(&self) -> f32;
}

pub struct Match {
    pub start: usize,
    pub end: usize,
    pub text: String,
}
```

### DetectorRegistry

Manages all available detectors.

```rust
pub struct DetectorRegistry {
    detectors: HashMap<String, Box<dyn Detector>>,
    enabled: HashSet<String>,
}

impl DetectorRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, detector: Box<dyn Detector>);
    pub fn enable(&mut self, name: &str);
    pub fn disable(&mut self, name: &str);
    pub fn detect_all(&self, segments: &[TextSegment]) -> Vec<Finding>;
}
```

---

## 003: Redaction Entities

### RedactionStyle

The type of redaction to apply.

```rust
pub enum RedactionStyle {
    /// Replace with category label: [EMAIL], [IBAN]
    Label,

    /// Replace with solid characters: ████████
    BlackBar { char: char },

    /// Partial masking with rules
    Mask(MaskingRule),

    /// Custom replacement text
    Custom { text: String },
}

pub struct MaskingRule {
    /// Number of characters to show at start
    pub show_first: usize,

    /// Number of characters to show at end
    pub show_last: usize,

    /// Mask character
    pub mask_char: char,

    /// Preserve certain characters (e.g., '@' in email)
    pub preserve: Vec<char>,
}
```

### RedactionConfig

Settings for the redaction engine.

```rust
pub struct RedactionConfig {
    /// Default style for all categories
    pub default_style: RedactionStyle,

    /// Per-category style overrides
    pub category_styles: HashMap<PiiCategory, RedactionStyle>,
}
```

### RedactionResult

Output of redaction operation.

```rust
pub struct RedactionResult {
    /// The redacted text
    pub text: String,

    /// List of applied redactions
    pub redactions: Vec<AppliedRedaction>,

    /// Position mapping for downstream use
    pub position_map: PositionMap,
}

pub struct AppliedRedaction {
    /// Original text that was redacted
    pub original: String,

    /// Replacement text
    pub replacement: String,

    /// Original position (start, end)
    pub original_position: (usize, usize),

    /// New position after redaction
    pub new_position: (usize, usize),

    /// PII category
    pub category: PiiCategory,
}

pub struct PositionMap {
    /// Maps original positions to redacted positions
    entries: Vec<PositionMapEntry>,
}

pub struct PositionMapEntry {
    pub original_start: usize,
    pub original_end: usize,
    pub redacted_start: usize,
    pub redacted_end: usize,
}
```

---

## 009: Policy Entities

### Policy

A complete policy definition.

```rust
pub struct Policy {
    /// Policy format version (semver)
    pub version: String,

    /// Human-readable name
    pub name: String,

    /// Optional locale for region-specific detection
    pub locale: Option<Locale>,

    /// Rules for filtering detection results
    pub detection: Vec<DetectionRule>,

    /// Rules for applying protection
    pub protection: Vec<ProtectionRule>,
}

pub enum Locale {
    DeAt,  // Austria
    DeDe,  // Germany
    DeCh,  // Switzerland
    En,    // English (international)
}
```

### DetectionRule

A rule for filtering findings.

```rust
pub struct DetectionRule {
    /// PII types this rule applies to
    pub types: Vec<PiiCategory>,

    /// Minimum confidence threshold
    pub confidence_threshold: f32,

    /// Whether this rule is enabled
    pub enabled: bool,
}
```

### ProtectionRule

A rule for applying protection.

```rust
pub struct ProtectionRule {
    /// PII types this rule applies to
    pub types: Vec<PiiCategory>,

    /// Protection action to apply
    pub action: ProtectionAction,

    /// Style options for the action
    pub style: Option<RedactionStyle>,

    /// Consistency flag for pseudonymization
    pub consistent: bool,

    /// Key reference for encryption
    pub key_ref: Option<KeyReference>,
}

pub enum ProtectionAction {
    Redact,
    Mask,
    Hash,
    Pseudonymize,
    Encrypt,
    Tokenize,
}

pub enum KeyReference {
    /// Read from environment variable: env://VAR_NAME
    Env(String),
    /// Read from file: file:///path/to/key
    File(PathBuf),
}
```

### PolicyValidationResult

Outcome of policy validation.

```rust
pub struct PolicyValidationResult {
    pub valid: bool,
    pub errors: Vec<PolicyError>,
    pub warnings: Vec<PolicyWarning>,
}

pub struct PolicyError {
    pub code: String,
    pub message: String,
    pub location: Option<String>,  // YAML path
}

pub struct PolicyWarning {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}
```

---

## 011: Audit Entities

### AuditEntry

A single audit log record.

```rust
pub struct AuditEntry {
    /// Unique identifier
    pub id: Uuid,

    /// When the operation occurred
    pub timestamp: DateTime<Utc>,

    /// Type of operation
    pub operation: AuditOperation,

    /// Operation-specific parameters
    pub parameters: AuditParameters,

    /// Operation outcome
    pub outcome: AuditOutcome,

    /// Checksum for tamper detection
    pub checksum: String,

    /// Previous entry's checksum (hash chain)
    pub previous_checksum: Option<String>,
}

pub enum AuditOperation {
    Scan,
    Protect,
    PolicyValidate,
    ReportGenerate,
}

pub struct AuditParameters {
    /// Input file path(s)
    pub input: Vec<PathBuf>,

    /// Output file path (for protect)
    pub output: Option<PathBuf>,

    /// Policy used
    pub policy: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

pub struct AuditOutcome {
    /// Whether operation succeeded
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,

    /// Findings summary (for scan)
    pub findings: Option<FindingsSummary>,

    /// Redactions summary (for protect)
    pub redactions: Option<RedactionsSummary>,
}

pub struct FindingsSummary {
    pub total: usize,
    pub by_category: HashMap<PiiCategory, usize>,
}

pub struct RedactionsSummary {
    pub total: usize,
    pub by_category: HashMap<PiiCategory, usize>,
}
```

### AuditLogger

Service for writing audit entries.

```rust
pub struct AuditLogger {
    /// Path to audit log directory
    log_dir: PathBuf,

    /// Current day's log file
    current_file: Option<File>,

    /// Last entry's checksum for hash chain
    last_checksum: Option<String>,
}

impl AuditLogger {
    pub fn new(log_dir: PathBuf) -> Result<Self, AuditError>;
    pub fn log(&mut self, entry: AuditEntry) -> Result<(), AuditError>;
    pub fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>, AuditError>;
    pub fn export(&self, filter: AuditFilter, format: ExportFormat) -> Result<String, AuditError>;
}

pub struct AuditFilter {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub operations: Option<Vec<AuditOperation>>,
    pub paths: Option<Vec<PathBuf>>,
}

pub enum ExportFormat {
    JsonLines,
    Csv,
    Text,
}
```

---

## Entity Relationships

```text
Document
    │
    └──▶ ParseResult
            │
            ├──▶ TextSegment (1..n)
            │        │
            │        └──▶ Position
            │
            └──▶ ParseWarning (0..n)

TextSegment
    │
    └──▶ Finding (0..n)
            │
            ├──▶ PiiCategory
            └──▶ ValidationStatus

Finding + RedactionConfig
    │
    └──▶ RedactionResult
            │
            ├──▶ AppliedRedaction (1..n)
            └──▶ PositionMap

Policy
    │
    ├──▶ DetectionRule (0..n)
    │        └──▶ PiiCategory (1..n)
    │
    └──▶ ProtectionRule (0..n)
            ├──▶ PiiCategory (1..n)
            ├──▶ ProtectionAction
            └──▶ KeyReference?

AuditLogger
    │
    └──▶ AuditEntry (0..n)
            │
            ├──▶ AuditOperation
            ├──▶ AuditParameters
            └──▶ AuditOutcome
                    │
                    ├──▶ FindingsSummary?
                    └──▶ RedactionsSummary?
```

---

## Serialization Formats

### Finding (JSON)

```json
{
  "matched_text": "john@example.com",
  "category": "email",
  "start": 42,
  "end": 58,
  "confidence": 1.0,
  "validation": "valid",
  "segment_index": 0
}
```

### Policy (YAML)

```yaml
version: "1.0"
name: "GDPR Standard"
locale: "de-AT"

detection:
  - types: [email, phone]
    confidence_threshold: 0.8
    enabled: true

protection:
  - types: [email, phone]
    action: redact
    style: label
  - types: [iban]
    action: mask
```

### AuditEntry (JSON Lines)

```json
{"id":"550e8400-e29b-41d4-a716-446655440000","timestamp":"2025-12-15T10:30:00Z","operation":"scan","parameters":{"input":["doc.txt"]},"outcome":{"success":true,"findings":{"total":5,"by_category":{"email":3,"iban":2}}},"checksum":"abc123","previous_checksum":null}
```
