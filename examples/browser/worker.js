// Veil Web Worker for Large File Processing
// This worker runs scanning/protection in a background thread to avoid blocking the UI

import init, { scan, protect, scan_with_progress, protect_with_progress } from './pkg/veil_wasm.js';

let initialized = false;

// Initialize WASM module
async function initialize() {
    if (!initialized) {
        await init();
        initialized = true;
        console.log('[Worker] WASM initialized');
    }
}

// Handle messages from main thread
self.onmessage = async function(event) {
    const { type, id, data, options } = event.data;

    try {
        await initialize();

        let result;

        switch (type) {
            case 'scan':
                result = scan_with_progress(
                    new Uint8Array(data),
                    options,
                    (percent) => {
                        self.postMessage({ type: 'progress', id, percent });
                    }
                );
                break;

            case 'protect':
                result = protect_with_progress(
                    new Uint8Array(data),
                    options,
                    (percent) => {
                        self.postMessage({ type: 'progress', id, percent });
                    }
                );
                break;

            default:
                throw new Error(`Unknown message type: ${type}`);
        }

        self.postMessage({ type: 'result', id, result });

    } catch (error) {
        self.postMessage({
            type: 'error',
            id,
            error: error.message || String(error)
        });
    }
};

// Example usage from main thread:
//
// const worker = new Worker(new URL('./worker.js', import.meta.url), { type: 'module' });
//
// function scanWithWorker(fileBuffer, options) {
//     return new Promise((resolve, reject) => {
//         const id = Math.random().toString(36);
//
//         function handler(event) {
//             if (event.data.id !== id) return;
//
//             switch (event.data.type) {
//                 case 'progress':
//                     console.log(`Progress: ${event.data.percent}%`);
//                     break;
//                 case 'result':
//                     worker.removeEventListener('message', handler);
//                     resolve(event.data.result);
//                     break;
//                 case 'error':
//                     worker.removeEventListener('message', handler);
//                     reject(new Error(event.data.error));
//                     break;
//             }
//         }
//
//         worker.addEventListener('message', handler);
//         worker.postMessage({ type: 'scan', id, data: fileBuffer, options });
//     });
// }
