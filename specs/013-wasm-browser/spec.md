# Feature Specification: WASM Browser Integration

**Feature Branch**: `013-wasm-browser`
**Created**: 2025-12-08
**Status**: Draft
**Input**: WebAssembly build for client-side PII detection in browsers

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scan Document in Browser (Priority: P1)

A user visits a web page, selects a file using a file picker, and receives PII scan results without the file ever leaving their browser. All processing happens client-side via WebAssembly.

**Why this priority**: Zero-knowledge scanning is the core value proposition for sensitive document handling.

**Independent Test**: Load WASM module in browser, select file, verify scan completes with findings displayed.

**Acceptance Scenarios**:

1. **Given** user selects a text file, **When** scan initiated, **Then** findings displayed without network requests.
2. **Given** file with PII, **When** scanned, **Then** findings show category, position, and matched text.
3. **Given** browser DevTools network tab open, **When** scanning, **Then** no outbound requests with file data.

---

### User Story 2 - Protect Document in Browser (Priority: P1)

A user scans a document and clicks "Protect" to download a redacted version. The redaction happens entirely in the browser; the user downloads the protected file.

**Why this priority**: End-to-end client-side protection completes the zero-knowledge workflow.

**Independent Test**: Scan and protect file in browser, download result, verify PII is redacted.

**Acceptance Scenarios**:

1. **Given** scan results displayed, **When** "Protect" clicked, **Then** redacted file available for download.
2. **Given** redacted file downloaded, **When** opened, **Then** PII is replaced per selected style.
3. **Given** large file, **When** protecting, **Then** progress indicator shown during processing.

---

### User Story 3 - Integrate WASM Module in Web App (Priority: P1)

A developer includes the Veil WASM module in their web application. They import the module and call scanning functions from JavaScript.

**Why this priority**: Developer integration is essential for embedding Veil in third-party applications.

**Independent Test**: Import WASM module in JS, call scan function, verify results returned.

**Acceptance Scenarios**:

1. **Given** npm package installed, **When** imported in JS, **Then** scan/protect functions available.
2. **Given** file as ArrayBuffer, **When** passed to scan(), **Then** findings returned as JS object.
3. **Given** TypeScript project, **When** using module, **Then** type definitions are available.

---

### User Story 4 - Handle Large Files Efficiently (Priority: P2)

A user attempts to scan a large file (10MB+). The browser remains responsive and shows progress while processing.

**Why this priority**: Large file handling demonstrates production readiness for real-world use.

**Independent Test**: Select 10MB file, verify browser doesn't freeze, progress shown.

**Acceptance Scenarios**:

1. **Given** 10MB file, **When** scanning, **Then** UI remains responsive with progress updates.
2. **Given** Web Worker available, **When** scanning, **Then** processing happens in worker thread.
3. **Given** progress callbacks, **When** scanning, **Then** percentage complete is reported.

---

### User Story 5 - Work Offline (Priority: P2)

A user loads the web app, then goes offline. They can still scan and protect documents because all code runs locally.

**Why this priority**: Offline capability is essential for air-gapped environments and unreliable networks.

**Independent Test**: Load app, disconnect network, scan file, verify it works.

**Acceptance Scenarios**:

1. **Given** app loaded and cached, **When** offline, **Then** scanning still works.
2. **Given** Service Worker registered, **When** app revisited offline, **Then** app loads from cache.
3. **Given** no network, **When** protecting, **Then** download of redacted file works.

---

### Edge Cases

- What happens when browser doesn't support WASM? System shows error with browser requirements.
- What happens when memory is limited (mobile)? System streams processing where possible, reports error for too-large files.
- What happens with unsupported file types? System reports error before attempting processing.
- What happens if WASM module fails to load? System shows clear error with troubleshooting guidance.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST compile to WebAssembly targeting wasm32-unknown-unknown.
- **FR-002**: System MUST expose scan function callable from JavaScript.
- **FR-003**: System MUST expose protect function callable from JavaScript.
- **FR-004**: System MUST accept file input as ArrayBuffer or Uint8Array.
- **FR-005**: System MUST return results as JavaScript objects (not pointers).
- **FR-006**: System MUST provide TypeScript type definitions.
- **FR-007**: System MUST support Web Worker execution for non-blocking processing.
- **FR-008**: System MUST provide progress callbacks for long operations.
- **FR-009**: System MUST work without any network requests after initial load.
- **FR-010**: System MUST support Service Worker caching for offline use.
- **FR-011**: WASM bundle MUST be under 5MB compressed.
- **FR-012**: System MUST handle files up to 50MB in browser environment.

### Key Entities

- **WasmModule**: The compiled WebAssembly module; exports scan, protect, and utility functions.
- **JsBinding**: JavaScript wrapper for WASM functions; handles data marshaling.
- **ScanResult**: JavaScript object returned from scan; contains findings array.
- **ProtectResult**: JavaScript object returned from protect; contains protected file as Uint8Array.
- **ProgressCallback**: Function called during processing; receives percentage complete.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: WASM bundle size is under 5MB gzipped.
- **SC-002**: Scanning 1MB file completes in under 3 seconds on modern browser.
- **SC-003**: Zero network requests made during scan/protect operations.
- **SC-004**: Module loads and initializes in under 2 seconds.
- **SC-005**: Works in latest versions of Chrome, Firefox, Safari, Edge.
- **SC-006**: TypeScript types compile without errors.

## Assumptions

- The WASM build excludes features requiring filesystem access (recursive directory scanning).
- Memory limits are respected; files over 50MB may fail in memory-constrained browsers.
- The dictionaries and regex patterns are bundled in the WASM module.
- Policy configuration is passed from JavaScript; no file-based policy loading in browser.

## JavaScript API Example

```javascript
import { init, scan, protect } from '@veil/wasm';

// Initialize the module
await init();

// Scan a file
const fileBuffer = await file.arrayBuffer();
const findings = await scan(fileBuffer, {
  filename: 'document.txt',
  onProgress: (percent) => console.log(`${percent}% complete`)
});

console.log(findings);
// [{ type: 'email', value: 'test@example.com', start: 100, end: 116, confidence: 1.0 }]

// Protect a file
const protected = await protect(fileBuffer, {
  style: 'labels',
  onProgress: (percent) => updateProgressBar(percent)
});

// Download the protected file
const blob = new Blob([protected], { type: 'text/plain' });
const url = URL.createObjectURL(blob);
downloadLink.href = url;
```
