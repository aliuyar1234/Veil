# veil-wasm

WebAssembly bindings for Veil PII detection and protection.

## Features

- **Zero-knowledge scanning**: Files never leave the browser
- **Client-side protection**: Redact PII without server upload
- **TypeScript support**: Full type definitions included
- **Progress callbacks**: Track long-running operations
- **Offline capable**: Works without network after initial load

## Installation

```bash
npm install @veil/wasm
```

## Quick Start

```javascript
import init, { scan, protect } from '@veil/wasm';

// Initialize the WASM module
await init();

// Scan a file for PII
const fileBuffer = await file.arrayBuffer();
const result = scan(new Uint8Array(fileBuffer), {
  filename: 'document.txt'
});

console.log(result.findings);
// [{ category: 'email', value: 'test@example.com', start: 0, end: 16, confidence: 1.0 }]

// Protect (redact) the file
const protected = protect(new Uint8Array(fileBuffer), {
  filename: 'document.txt',
  style: 'labels'  // or 'redact', 'mask'
});

// Download the protected file
const blob = new Blob([protected.data], { type: 'text/plain' });
```

## API Reference

### `init()`

Initialize the WASM module. Must be called before using other functions.

### `scan(data, options?)`

Scan data for PII.

**Parameters:**
- `data: Uint8Array` - File contents
- `options?: ScanOptions`
  - `filename?: string` - Used for format detection
  - `categories?: string[]` - Filter to specific categories
  - `minConfidence?: number` - Minimum confidence threshold (0-1)

**Returns:** `ScanResult`
- `findings: Finding[]` - Detected PII instances
- `stats: ScanStats` - Processing statistics

### `scan_with_progress(data, options, onProgress)`

Same as `scan()` but with progress callback.

**Parameters:**
- `onProgress: (percent: number) => void` - Called with 0-100

### `protect(data, options)`

Redact PII from data.

**Parameters:**
- `data: Uint8Array` - File contents
- `options: ProtectOptions`
  - `filename?: string` - Used for format detection
  - `style: 'labels' | 'redact' | 'mask'` - Redaction style
  - `categories?: string[]` - Filter to specific categories

**Returns:** `ProtectResult`
- `data: Uint8Array` - Protected file contents
- `stats: ProtectStats` - Processing statistics

### `protect_with_progress(data, options, onProgress)`

Same as `protect()` but with progress callback.

### `supported_formats()`

Returns array of supported file extensions: `['txt', 'csv', 'json', 'html']`

### `available_categories()`

Returns array of detectable PII categories: `['email', 'iban', 'phone', 'credit_card']`

## Types

```typescript
interface Finding {
  category: string;
  value: string;
  start: number;
  end: number;
  confidence: number;
}

interface ScanResult {
  findings: Finding[];
  stats: ScanStats;
}

interface ScanStats {
  bytesProcessed: number;
  durationMs: number;
  categoryCounts: Record<string, number>;
}

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

## Building from Source

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web
wasm-pack build --target web crates/veil-wasm

# Output in crates/veil-wasm/pkg/
```

## Browser Support

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## Size

- Uncompressed: ~800KB
- Gzipped: ~150KB

## License

MIT OR Apache-2.0
