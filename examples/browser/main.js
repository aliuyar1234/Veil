// Veil WASM Demo
// This demo shows how to use the Veil WASM module for client-side PII detection

// Import the WASM module (adjust path based on your build output)
import init, { scan, protect, scan_with_progress, protect_with_progress, supported_formats, available_categories } from './pkg/veil_wasm.js';

// State
let selectedFile = null;
let fileBuffer = null;
let scanResults = null;

// DOM elements
const fileInput = document.getElementById('fileInput');
const fileInfo = document.getElementById('fileInfo');
const scanBtn = document.getElementById('scanBtn');
const protectBtn = document.getElementById('protectBtn');
const styleSelect = document.getElementById('styleSelect');
const scanProgress = document.getElementById('scanProgress');
const scanProgressBar = document.getElementById('scanProgressBar');
const protectProgress = document.getElementById('protectProgress');
const protectProgressBar = document.getElementById('protectProgressBar');
const scanResults_el = document.getElementById('scanResults');
const protectResults = document.getElementById('protectResults');
const statusEl = document.getElementById('status');

// Initialize WASM module
async function initialize() {
    try {
        await init();
        console.log('WASM module initialized');
        console.log('Supported formats:', supported_formats());
        console.log('Available categories:', available_categories());
    } catch (error) {
        console.error('Failed to initialize WASM:', error);
        scanResults_el.innerHTML = `<p style="color: red;">Failed to load WASM module: ${error.message}</p>`;
    }
}

// File selection handler
fileInput.addEventListener('change', async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    selectedFile = file;
    fileBuffer = await file.arrayBuffer();

    fileInfo.innerHTML = `
        <p><strong>File:</strong> ${file.name}</p>
        <p><strong>Size:</strong> ${formatBytes(file.size)}</p>
        <p><strong>Type:</strong> ${file.type || 'unknown'}</p>
    `;

    scanBtn.disabled = false;
    protectBtn.disabled = true;
    scanResults_el.innerHTML = '';
    protectResults.innerHTML = '';
    scanResults = null;
});

// Scan button handler
scanBtn.addEventListener('click', async () => {
    if (!fileBuffer) return;

    scanBtn.disabled = true;
    scanProgress.classList.add('active');
    scanResults_el.innerHTML = '<p>Scanning...</p>';

    try {
        const data = new Uint8Array(fileBuffer);
        const options = {
            filename: selectedFile.name
        };

        // Use progress callback for large files
        const result = await new Promise((resolve, reject) => {
            try {
                const res = scan_with_progress(data, options, (percent) => {
                    scanProgressBar.style.width = `${percent}%`;
                });
                resolve(res);
            } catch (err) {
                reject(err);
            }
        });

        scanResults = result;
        displayScanResults(result);
        protectBtn.disabled = false;

    } catch (error) {
        console.error('Scan error:', error);
        scanResults_el.innerHTML = `<p style="color: red;">Scan failed: ${error.message || error}</p>`;
    } finally {
        scanBtn.disabled = false;
        scanProgress.classList.remove('active');
    }
});

// Protect button handler
protectBtn.addEventListener('click', async () => {
    if (!fileBuffer) return;

    protectBtn.disabled = true;
    protectProgress.classList.add('active');
    protectResults.innerHTML = '<p>Protecting...</p>';

    try {
        const data = new Uint8Array(fileBuffer);
        const options = {
            filename: selectedFile.name,
            style: styleSelect.value
        };

        // Use progress callback
        const result = await new Promise((resolve, reject) => {
            try {
                const res = protect_with_progress(data, options, (percent) => {
                    protectProgressBar.style.width = `${percent}%`;
                });
                resolve(res);
            } catch (err) {
                reject(err);
            }
        });

        // Create download
        const blob = new Blob([result.data], { type: 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `protected_${selectedFile.name}`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);

        protectResults.innerHTML = `
            <p style="color: green;">File protected successfully!</p>
            <p class="stats">
                Replacements: ${result.stats.replacements}<br>
                Categories: ${result.stats.protectedCategories.join(', ') || 'none'}<br>
                Time: ${result.stats.durationMs}ms
            </p>
        `;

    } catch (error) {
        console.error('Protect error:', error);
        protectResults.innerHTML = `<p style="color: red;">Protection failed: ${error.message || error}</p>`;
    } finally {
        protectBtn.disabled = false;
        protectProgress.classList.remove('active');
    }
});

// Display scan results
function displayScanResults(result) {
    const { findings, stats } = result;

    let html = `
        <p class="stats">
            Processed: ${formatBytes(stats.bytesProcessed)}<br>
            Time: ${stats.durationMs}ms<br>
            Findings: ${findings.length}
        </p>
    `;

    if (findings.length === 0) {
        html += '<p style="color: green;">No PII detected!</p>';
    } else {
        html += '<h3>Findings:</h3>';
        for (const finding of findings) {
            html += `
                <div class="finding">
                    <span class="category">${finding.category.toUpperCase()}</span>
                    <span class="value">${escapeHtml(finding.value)}</span>
                    <span class="stats">(confidence: ${(finding.confidence * 100).toFixed(0)}%, position: ${finding.start}-${finding.end})</span>
                </div>
            `;
        }
    }

    scanResults_el.innerHTML = html;
}

// Utility functions
function formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Service Worker registration for offline support
if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('sw.js')
        .then(reg => {
            console.log('Service Worker registered');
        })
        .catch(err => {
            console.log('Service Worker registration failed:', err);
        });
}

// Online/offline status
function updateOnlineStatus() {
    if (navigator.onLine) {
        statusEl.className = 'status online';
        statusEl.textContent = 'Online';
    } else {
        statusEl.className = 'status offline';
        statusEl.textContent = 'Offline - Working from cache';
    }
}

window.addEventListener('online', updateOnlineStatus);
window.addEventListener('offline', updateOnlineStatus);
updateOnlineStatus();

// Initialize on load
initialize();
