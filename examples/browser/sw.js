// Veil Service Worker for Offline Support
const CACHE_NAME = 'veil-wasm-v1';

// Files to cache for offline use
const CACHE_FILES = [
    './',
    './index.html',
    './main.js',
    './pkg/veil_wasm.js',
    './pkg/veil_wasm_bg.wasm'
];

// Install event - cache files
self.addEventListener('install', event => {
    console.log('[SW] Installing Service Worker');
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then(cache => {
                console.log('[SW] Caching app shell');
                return cache.addAll(CACHE_FILES);
            })
            .then(() => {
                console.log('[SW] Install complete');
                return self.skipWaiting();
            })
            .catch(err => {
                console.error('[SW] Cache failed:', err);
            })
    );
});

// Activate event - clean old caches
self.addEventListener('activate', event => {
    console.log('[SW] Activating Service Worker');
    event.waitUntil(
        caches.keys().then(cacheNames => {
            return Promise.all(
                cacheNames
                    .filter(name => name !== CACHE_NAME)
                    .map(name => {
                        console.log('[SW] Removing old cache:', name);
                        return caches.delete(name);
                    })
            );
        }).then(() => {
            console.log('[SW] Claiming clients');
            return self.clients.claim();
        })
    );
});

// Fetch event - serve from cache, fallback to network
self.addEventListener('fetch', event => {
    // Skip non-GET requests
    if (event.request.method !== 'GET') {
        return;
    }

    event.respondWith(
        caches.match(event.request)
            .then(cachedResponse => {
                if (cachedResponse) {
                    console.log('[SW] Serving from cache:', event.request.url);
                    return cachedResponse;
                }

                console.log('[SW] Fetching from network:', event.request.url);
                return fetch(event.request)
                    .then(response => {
                        // Don't cache non-successful responses
                        if (!response || response.status !== 200) {
                            return response;
                        }

                        // Cache WASM and JS files
                        if (event.request.url.endsWith('.wasm') ||
                            event.request.url.endsWith('.js') ||
                            event.request.url.endsWith('.html')) {
                            const responseToCache = response.clone();
                            caches.open(CACHE_NAME)
                                .then(cache => {
                                    cache.put(event.request, responseToCache);
                                });
                        }

                        return response;
                    })
                    .catch(err => {
                        console.error('[SW] Fetch failed:', err);
                        // Return offline page if available
                        return caches.match('./index.html');
                    });
            })
    );
});
