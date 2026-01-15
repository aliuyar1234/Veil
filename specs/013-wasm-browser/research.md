# Research: WASM Browser Integration

**Feature**: 013-wasm-browser
**Date**: 2025-12-09

## 1. WASM Tooling: wasm-bindgen vs wasm-pack

**Decision**: Use wasm-pack as the primary build tool

**Rationale**:
- wasm-pack (v0.13+) provides complete build pipeline including JS binding generation
- wasm-bindgen (v0.2.90+) is used internally by wasm-pack
- wasm-pack handles TypeScript generation, npm package creation, and optimization

**Alternatives Considered**:
- Direct wasm-bindgen: More control but requires manual build orchestration
- Trunk: Better for full web apps, overkill for library-only WASM module

**Configuration**:
```toml
# Cargo.toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2.90"
wasm-bindgen-futures = "0.4.40"
js-sys = "0.3.68"
web-sys = { version = "0.3.68", features = ["console", "Worker", "Window"] }

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

## 2. Bundle Size Optimization

**Decision**: Apply multi-layer optimization targeting <5MB gzipped

**Rationale**:
- Spec requires <5MB bundle size
- Multiple techniques compound: LTO + wasm-opt + gzip achieves 10-20x reduction

**Optimization Stack**:
1. **Cargo profile**: `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true`
2. **wasm-opt**: Use `-Oz` flag for size optimization (integrated via wasm-pack)
3. **Compression**: Serve with gzip/brotli (handled by web server)

**Expected Results**:
- Unoptimized: ~2MB
- After wasm-opt: ~800KB
- After LTO + strip: ~400-500KB
- With gzip: ~100-150KB

**Alternatives Considered**:
- wee_alloc (minimal allocator): Adds ~1KB savings but deprecated; standard allocator preferred
- Code splitting: Not needed for single-module library; adds complexity

## 3. Web Worker Integration

**Decision**: Support optional Web Worker execution with main-thread fallback

**Rationale**:
- Large files (10MB+) require non-blocking processing per spec
- Web Workers prevent UI freezing
- Main thread fallback ensures compatibility when Workers unavailable

**Pattern**:
```javascript
// Worker mode (preferred for large files)
const worker = new Worker(new URL('./veil-worker.js', import.meta.url));
worker.postMessage({ type: 'scan', data: fileBuffer });
worker.onmessage = (e) => handleResult(e.data);

// Main thread mode (fallback or small files)
const result = await scan(fileBuffer);
```

**Alternatives Considered**:
- SharedWorker: Overkill for single-page use case
- Comlink: Adds dependency; raw postMessage sufficient for simple API

## 4. Progress Callback Implementation

**Decision**: Use JS Function callbacks passed to WASM

**Rationale**:
- Spec requires progress updates during long operations
- Direct callback invocation is simpler than event-based patterns
- Works in both main thread and Web Worker contexts

**Pattern**:
```rust
use wasm_bindgen::prelude::*;
use js_sys::Function;

#[wasm_bindgen]
pub fn scan_with_progress(
    data: &[u8],
    on_progress: &Function,
) -> Result<JsValue, JsValue> {
    for (i, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
        // Process chunk...

        let progress = ((i + 1) * 100 / total_chunks) as u32;
        let _ = on_progress.call1(&JsValue::NULL, &JsValue::from(progress));
    }
    // Return results...
}
```

**Alternatives Considered**:
- Custom events: More complex, no advantage for single callback
- Observable/Stream pattern: Adds JS dependency; overkill for progress percentage

## 5. Memory Management

**Decision**: Use stack-based processing with pre-allocated buffers

**Rationale**:
- Avoid allocation in tight loops per constitution (Performance principle)
- 50MB max file size is manageable with pre-allocation
- Standard allocator is sufficient; no custom allocator needed

**Pattern**:
```rust
// Accept references, allocate only for output
#[wasm_bindgen]
pub fn scan(input: &[u8]) -> Result<JsValue, JsValue> {
    let mut findings = Vec::with_capacity(expected_findings);
    // Process without intermediate allocations...
    serde_wasm_bindgen::to_value(&findings)
}
```

**Memory Budget**:
- WASM module: ~1MB
- Input buffer: up to 50MB
- Processing overhead: ~10MB
- Total: <100MB for largest files

## 6. TypeScript Type Generation

**Decision**: Use automatic wasm-pack generation with serde-wasm-bindgen for complex types

**Rationale**:
- wasm-pack generates .d.ts automatically from #[wasm_bindgen]
- serde-wasm-bindgen handles complex nested types (ScanResult, Finding)
- Zero manual TypeScript maintenance

**Type Mapping**:
| Rust | TypeScript |
|------|------------|
| i32, u32, f64 | number |
| bool | boolean |
| String, &str | string |
| Vec<T> | T[] |
| Option<T> | T \| null |
| Result<T, E> | T (throws on Err) |
| Custom struct | Generated interface |

**Pattern**:
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Finding {
    pub category: String,
    pub value: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f64,
}

#[wasm_bindgen]
pub fn scan(data: &[u8]) -> Result<JsValue, JsValue> {
    let findings: Vec<Finding> = detect_pii(data)?;
    serde_wasm_bindgen::to_value(&findings)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
```

## 7. Service Worker Caching for Offline Use

**Decision**: Provide example Service Worker with precaching strategy

**Rationale**:
- Spec requires offline capability after initial load
- WASM binaries are immutable and ideal for long-term caching
- Workbox simplifies cache management

**Pattern**:
```javascript
// sw.js - Precache WASM module
const CACHE_NAME = 'veil-wasm-v1';
const WASM_FILES = [
  '/pkg/veil_wasm_bg.wasm',
  '/pkg/veil_wasm.js'
];

self.addEventListener('install', event => {
  event.waitUntil(
    caches.open(CACHE_NAME).then(cache => cache.addAll(WASM_FILES))
  );
});

self.addEventListener('fetch', event => {
  if (event.request.url.endsWith('.wasm') ||
      event.request.url.endsWith('.js')) {
    event.respondWith(
      caches.match(event.request)
        .then(response => response || fetch(event.request))
    );
  }
});
```

**Cache Invalidation**: Version-based cache naming (`veil-wasm-v1`, `veil-wasm-v2`)

## 8. Async Function Bindings

**Decision**: Use wasm-bindgen-futures for async operations

**Rationale**:
- Scan and protect operations may be long-running
- Async allows yielding to browser during processing
- Matches JavaScript async/await expectations

**Pattern**:
```rust
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
pub async fn scan_async(data: Vec<u8>) -> Result<JsValue, JsValue> {
    // Async processing with yield points...
}
```

## Summary of Technology Decisions

| Component | Choice | Version |
|-----------|--------|---------|
| Build tool | wasm-pack | 0.13+ |
| JS bindings | wasm-bindgen | 0.2.90+ |
| Async support | wasm-bindgen-futures | 0.4.40+ |
| Web APIs | web-sys | 0.3.68+ |
| JS types | js-sys | 0.3.68+ |
| Serialization | serde-wasm-bindgen | 0.6+ |
| Optimization | wasm-opt (via wasm-pack) | -Oz |

## Open Questions Resolved

1. **Q: How to handle browser compatibility?**
   A: Target wasm32-unknown-unknown; works in all modern browsers (Chrome 57+, Firefox 52+, Safari 11+, Edge 16+)

2. **Q: How to handle memory limits on mobile?**
   A: Implement streaming for large files; report error for files exceeding memory budget

3. **Q: How to handle unsupported file types?**
   A: Validate file type before processing; return typed error to JavaScript
