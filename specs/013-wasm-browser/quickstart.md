# Quickstart: Veil WASM Browser Integration

This guide shows how to use the Veil WASM module for client-side PII detection and protection.

## Installation

### npm

```bash
npm install @veil/wasm
```

### CDN (ESM)

```html
<script type="module">
  import { init, scan, protect } from 'https://unpkg.com/@veil/wasm@latest';
</script>
```

## Basic Usage

### Initialize the Module

```javascript
import { init, scan, protect } from '@veil/wasm';

// Initialize once on page load
await init();
```

### Scan a File for PII

```javascript
// Get file from input element
const file = document.querySelector('input[type="file"]').files[0];
const buffer = await file.arrayBuffer();

// Scan for PII
const result = await scan(buffer, {
  filename: file.name,
  onProgress: (percent) => {
    console.log(`Scanning: ${percent}%`);
  }
});

// Display findings
console.log(`Found ${result.findings.length} PII items`);
result.findings.forEach(finding => {
  console.log(`${finding.category}: ${finding.value} (${finding.confidence * 100}% confidence)`);
});
```

### Protect a File (Redact PII)

```javascript
const result = await protect(buffer, {
  filename: file.name,
  style: 'labels', // or 'redact', 'mask'
  onProgress: (percent) => {
    progressBar.value = percent;
  }
});

// Download protected file
const blob = new Blob([result.data], { type: file.type });
const url = URL.createObjectURL(blob);

const link = document.createElement('a');
link.href = url;
link.download = `protected_${file.name}`;
link.click();

URL.revokeObjectURL(url);
```

## Advanced Usage

### Filter by Category

```javascript
// Only detect emails and phone numbers
const result = await scan(buffer, {
  categories: ['email', 'phone'],
  minConfidence: 0.8
});
```

### Check Supported Formats

```javascript
import { isSupported, getCategories } from '@veil/wasm';

if (isSupported('document.txt')) {
  // Process file
}

// Get all detection categories
const categories = getCategories();
// ['email', 'phone', 'iban', 'credit_card', ...]
```

### Web Worker (Non-blocking)

For large files, run processing in a Web Worker to keep the UI responsive:

**worker.js:**
```javascript
import { init, scan, protect } from '@veil/wasm';

let initialized = false;

self.onmessage = async (event) => {
  if (!initialized) {
    await init();
    initialized = true;
  }

  const { type, data, options } = event.data;

  try {
    if (type === 'scan') {
      const result = await scan(data, {
        ...options,
        onProgress: (p) => self.postMessage({ type: 'progress', value: p })
      });
      self.postMessage({ type: 'result', value: result });
    } else if (type === 'protect') {
      const result = await protect(data, {
        ...options,
        onProgress: (p) => self.postMessage({ type: 'progress', value: p })
      });
      self.postMessage({ type: 'result', value: result });
    }
  } catch (error) {
    self.postMessage({ type: 'error', value: error.message });
  }
};
```

**main.js:**
```javascript
const worker = new Worker(new URL('./worker.js', import.meta.url), { type: 'module' });

function scanWithWorker(buffer, options) {
  return new Promise((resolve, reject) => {
    worker.onmessage = (event) => {
      if (event.data.type === 'progress') {
        options.onProgress?.(event.data.value);
      } else if (event.data.type === 'result') {
        resolve(event.data.value);
      } else if (event.data.type === 'error') {
        reject(new Error(event.data.value));
      }
    };

    worker.postMessage({ type: 'scan', data: buffer, options });
  });
}
```

## Offline Support (Service Worker)

Cache the WASM module for offline use:

**sw.js:**
```javascript
const CACHE_NAME = 'veil-wasm-v1';
const ASSETS = [
  '/index.html',
  '/app.js',
  '/node_modules/@veil/wasm/veil_wasm.js',
  '/node_modules/@veil/wasm/veil_wasm_bg.wasm'
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(ASSETS))
  );
});

self.addEventListener('fetch', (event) => {
  event.respondWith(
    caches.match(event.request).then((response) => {
      return response || fetch(event.request);
    })
  );
});
```

**Register in main app:**
```javascript
if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('/sw.js');
}
```

## Error Handling

```javascript
import { scan, VeilError } from '@veil/wasm';

try {
  const result = await scan(buffer);
} catch (error) {
  if (error.code === 'FileTooLarge') {
    alert('File exceeds 50MB limit');
  } else if (error.code === 'UnsupportedFormat') {
    alert('This file type is not supported');
  } else if (error.code === 'InvalidInput') {
    alert('Could not read file');
  } else {
    console.error('Unexpected error:', error.message);
  }
}
```

## Complete Example

```html
<!DOCTYPE html>
<html>
<head>
  <title>Veil PII Scanner</title>
</head>
<body>
  <input type="file" id="fileInput" />
  <button id="scanBtn" disabled>Scan</button>
  <button id="protectBtn" disabled>Protect</button>
  <progress id="progress" value="0" max="100"></progress>
  <pre id="results"></pre>

  <script type="module">
    import { init, scan, protect, isSupported } from '@veil/wasm';

    const fileInput = document.getElementById('fileInput');
    const scanBtn = document.getElementById('scanBtn');
    const protectBtn = document.getElementById('protectBtn');
    const progress = document.getElementById('progress');
    const results = document.getElementById('results');

    let currentBuffer = null;
    let currentFilename = null;

    // Initialize on load
    await init();

    fileInput.addEventListener('change', async (e) => {
      const file = e.target.files[0];
      if (!file) return;

      if (!isSupported(file.name)) {
        alert('Unsupported file type');
        return;
      }

      currentBuffer = await file.arrayBuffer();
      currentFilename = file.name;
      scanBtn.disabled = false;
      protectBtn.disabled = true;
      results.textContent = '';
    });

    scanBtn.addEventListener('click', async () => {
      progress.value = 0;
      results.textContent = 'Scanning...';

      try {
        const result = await scan(currentBuffer, {
          filename: currentFilename,
          onProgress: (p) => { progress.value = p; }
        });

        results.textContent = JSON.stringify(result, null, 2);
        protectBtn.disabled = false;
      } catch (error) {
        results.textContent = `Error: ${error.message}`;
      }
    });

    protectBtn.addEventListener('click', async () => {
      progress.value = 0;
      results.textContent = 'Protecting...';

      try {
        const result = await protect(currentBuffer, {
          filename: currentFilename,
          style: 'labels',
          onProgress: (p) => { progress.value = p; }
        });

        // Download
        const blob = new Blob([result.data]);
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `protected_${currentFilename}`;
        a.click();
        URL.revokeObjectURL(url);

        results.textContent = `Protected! ${result.stats.replacements} items redacted.`;
      } catch (error) {
        results.textContent = `Error: ${error.message}`;
      }
    });
  </script>
</body>
</html>
```

## Performance Tips

1. **Use Web Workers** for files >1MB to prevent UI freezing
2. **Pre-initialize** the module during page load, not on first use
3. **Reuse buffers** when scanning and protecting the same file
4. **Filter categories** to scan only what you need
5. **Set minConfidence** higher (0.8+) to reduce false positives

## Browser Support

| Browser | Minimum Version |
|---------|-----------------|
| Chrome  | 57+ |
| Firefox | 52+ |
| Safari  | 11+ |
| Edge    | 16+ |

## Limitations

- Maximum file size: 50MB (browser memory constraint)
- Supported formats: TXT, CSV, JSON (additional formats via other Veil features)
- No filesystem access (browser security sandbox)
- Dictionaries are bundled in WASM (no runtime loading)
