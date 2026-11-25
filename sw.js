// Ten Cells offline cache. Online: always loads the newest upload. Offline: plays the last copy it saw.
const C = 'ten-cells';
const FILES = ['./', 'index.html', 'manifest.webmanifest', 'icon180.png', 'icon512.png'];
self.addEventListener('install', e => { self.skipWaiting(); e.waitUntil(caches.open(C).then(c => c.addAll(FILES))); });
self.addEventListener('activate', e => e.waitUntil(self.clients.claim()));
self.addEventListener('fetch', e => {
  const u = new URL(e.request.url);
  if (e.request.method !== 'GET' || u.origin !== location.origin || u.pathname.endsWith('version.json')) return; // the update check always asks the network
  e.respondWith(fetch(e.request).then(res => { if (res.ok) { const copy = res.clone(); caches.open(C).then(c => c.put(e.request, copy)); } return res; })
    .catch(() => caches.match(e.request, { ignoreSearch: true }).then(m => m || caches.match('index.html'))));
});
