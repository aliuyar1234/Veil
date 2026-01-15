# Data Model: Secure Scan Response

## Entity Changes

### Finding (API Response)

**Before**:
```
Finding {
    category: String        # Required
    value: String           # Required - THE PROBLEM
    confidence: f64         # Required
    start: usize           # Required
    end: usize             # Required
    context: Option<String> # Optional
    position: Option<PositionInfo>
}
```

**After**:
```
Finding {
    category: String        # Required
    value: Option<String>   # Optional - omitted by default
    confidence: f64         # Required
    start: usize           # Required
    end: usize             # Required
    context: Option<String> # Optional
    position: Option<PositionInfo>
}
```

### ScanOptions (API Request)

**Before**:
```
ScanOptions {
    categories: Vec<String>
    min_confidence: Option<f64>
    include_context: bool
    context_chars: usize
}
```

**After**:
```
ScanOptions {
    categories: Vec<String>
    min_confidence: Option<f64>
    include_context: bool
    context_chars: usize
    include_values: bool    # NEW - default false
}
```

### ScanOptions (WASM)

**Before**:
```typescript
interface ScanOptions {
    filename?: string
    categories?: string[]
    minConfidence?: number
}
```

**After**:
```typescript
interface ScanOptions {
    filename?: string
    categories?: string[]
    minConfidence?: number
    includeValues?: boolean       # NEW - default false
    acknowledgeExposure?: boolean # NEW - required if includeValues=true
}
```

### FindingOutput (CLI)

**Before**:
```
FindingOutput {
    category: String
    text: String        # THE PROBLEM
    position: String
    confidence: f32
}
```

**After**:
```
FindingOutput {
    category: String
    text: Option<String>  # Omitted by default
    position: String
    confidence: f32
}
```

### ScanArgs (CLI)

**Before**:
```
ScanArgs {
    paths: Vec<PathBuf>
    recursive: bool
    policy: Option<PathBuf>
    detect: Option<Vec<String>>
    fail_on_findings: bool
}
```

**After**:
```
ScanArgs {
    paths: Vec<PathBuf>
    recursive: bool
    policy: Option<PathBuf>
    detect: Option<Vec<String>>
    fail_on_findings: bool
    include_values: bool    # NEW - requires confirmation
}
```

## Validation Rules

### API Header Validation
- If `include_values=true` in query params:
  - Header `X-Acknowledge-PII-Exposure` MUST be present
  - Header value MUST equal `accepted` (case-insensitive)
  - If missing or wrong value: HTTP 400 response

### CLI Flag Validation
- If `--include-values` flag is present:
  - If stdin is TTY: prompt for confirmation
  - If stdin is not TTY: require `--yes` flag
  - Without confirmation: exit with error code 1

### WASM Option Validation
- If `includeValues: true`:
  - `acknowledgeExposure` MUST also be `true`
  - If `acknowledgeExposure` is false/missing: throw WasmError
