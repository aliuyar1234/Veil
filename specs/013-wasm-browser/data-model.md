# Data Model: WASM Browser Integration

**Feature**: 013-wasm-browser
**Date**: 2025-12-09

## Overview

This document defines the data structures exposed via the WASM JavaScript API. All types are
serialized via serde-wasm-bindgen between Rust and JavaScript.

## Core Entities

### Finding

Represents a single PII detection result.

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Finding {
    /// PII category (e.g., "email", "phone", "iban")
    pub category: String,

    /// The matched text value
    pub value: String,

    /// Byte offset of match start in source
    pub start: usize,

    /// Byte offset of match end in source (exclusive)
    pub end: usize,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}
```

**TypeScript**:
```typescript
interface Finding {
  category: string;
  value: string;
  start: number;
  end: number;
  confidence: number;
}
```

**Validation Rules**:
- `category` MUST be non-empty
- `end` MUST be > `start`
- `confidence` MUST be in range [0.0, 1.0]

---

### ScanOptions

Configuration for a scan operation.

```rust
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ScanOptions {
    /// Original filename (used for format detection)
    pub filename: Option<String>,

    /// Categories to detect (empty = all categories)
    pub categories: Vec<String>,

    /// Minimum confidence threshold (default: 0.5)
    pub min_confidence: Option<f64>,
}
```

**TypeScript**:
```typescript
interface ScanOptions {
  filename?: string;
  categories?: string[];
  minConfidence?: number;
}
```

**Validation Rules**:
- `min_confidence` if provided MUST be in range [0.0, 1.0]
- `categories` if empty defaults to all available categories

---

### ScanResult

Result of a scan operation.

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanResult {
    /// All findings from the scan
    pub findings: Vec<Finding>,

    /// Processing statistics
    pub stats: ScanStats,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanStats {
    /// Total bytes processed
    pub bytes_processed: usize,

    /// Processing time in milliseconds
    pub duration_ms: u64,

    /// Number of findings by category
    pub category_counts: HashMap<String, usize>,
}
```

**TypeScript**:
```typescript
interface ScanResult {
  findings: Finding[];
  stats: ScanStats;
}

interface ScanStats {
  bytesProcessed: number;
  durationMs: number;
  categoryCounts: Record<string, number>;
}
```

---

### ProtectOptions

Configuration for a protect operation.

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProtectOptions {
    /// Original filename (used for format detection)
    pub filename: Option<String>,

    /// Protection style
    pub style: ProtectStyle,

    /// Categories to protect (empty = all found categories)
    pub categories: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProtectStyle {
    /// Replace with [CATEGORY] labels
    Labels,

    /// Replace with ████ characters
    Redact,

    /// Replace with *** partial masking
    Mask,
}
```

**TypeScript**:
```typescript
interface ProtectOptions {
  filename?: string;
  style: 'labels' | 'redact' | 'mask';
  categories?: string[];
}
```

**Validation Rules**:
- `style` MUST be one of the defined values
- Invalid style values result in error, not silent fallback

---

### ProtectResult

Result of a protect operation.

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProtectResult {
    /// Protected content as bytes
    pub data: Vec<u8>,

    /// Statistics about the protection
    pub stats: ProtectStats,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProtectStats {
    /// Number of replacements made
    pub replacements: usize,

    /// Categories that were protected
    pub protected_categories: Vec<String>,

    /// Processing time in milliseconds
    pub duration_ms: u64,
}
```

**TypeScript**:
```typescript
interface ProtectResult {
  data: Uint8Array;
  stats: ProtectStats;
}

interface ProtectStats {
  replacements: number;
  protectedCategories: string[];
  durationMs: number;
}
```

---

### WasmError

Error type returned to JavaScript.

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WasmError {
    /// Error code for programmatic handling
    pub code: ErrorCode,

    /// Human-readable error message
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ErrorCode {
    /// Invalid input data
    InvalidInput,

    /// Unsupported file format
    UnsupportedFormat,

    /// File too large for memory
    FileTooLarge,

    /// Invalid configuration
    InvalidConfig,

    /// Internal processing error
    InternalError,
}
```

**TypeScript**:
```typescript
interface WasmError {
  code: 'InvalidInput' | 'UnsupportedFormat' | 'FileTooLarge' | 'InvalidConfig' | 'InternalError';
  message: string;
}
```

---

## Progress Callback

The progress callback signature for long-running operations.

```rust
// Rust: accepts js_sys::Function
pub type ProgressCallback = js_sys::Function;
```

**TypeScript**:
```typescript
type ProgressCallback = (progress: number) => void;
// progress is a number from 0 to 100
```

---

## State Transitions

### Scan Operation Flow

```
┌─────────────┐
│ Input Data  │ (ArrayBuffer/Uint8Array)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Validate    │ → [Error: InvalidInput, UnsupportedFormat, FileTooLarge]
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Parse       │ → progress(0-30%)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Detect PII  │ → progress(30-100%)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ ScanResult  │ (findings + stats)
└─────────────┘
```

### Protect Operation Flow

```
┌─────────────┐
│ Input Data  │ + ScanResult (optional, for incremental protect)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Validate    │ → [Error: InvalidInput, UnsupportedFormat]
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Scan (if    │ → progress(0-50%)
│ not cached) │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Apply       │ → progress(50-100%)
│ Protection  │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│ ProtectResult   │ (data + stats)
└─────────────────┘
```

---

## Entity Relationships

```
ScanOptions ──────┐
                  │
                  ▼
             ┌─────────┐
Input Data ──┤  scan() ├──▶ ScanResult
             └─────────┘         │
                                 │ contains
                                 ▼
                            ┌─────────┐
                            │ Finding │ (0..n)
                            └─────────┘

ProtectOptions ───┐
                  │
                  ▼
             ┌───────────┐
Input Data ──┤ protect() ├──▶ ProtectResult
             └───────────┘
```

---

## Size Limits

| Entity | Limit | Rationale |
|--------|-------|-----------|
| Input file | 50 MB | Browser memory constraints |
| Findings array | 100,000 | Practical limit for UI rendering |
| Category string | 64 chars | Reasonable identifier length |
| Value string | 1,000 chars | Prevents excessive memory for single finding |
