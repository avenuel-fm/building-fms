// 잠실점 에비뉴엘 FMS 서비스 워커 (PWA 오프라인 지원)
const CACHE_NAME = 'avenuel-fms-v1';
const STATIC_ASSETS = [
  './',
  './index.html',
  './manifest.json',
  'https://www.gstatic.com/antigravity/web/dev/tailwindcss.min.js',
  'https://cdn.jsdelivr.net/npm/@supabase/supabase-js@2'
];

// 1. 서비스 워커 설치: 정적 자원 사전 캐싱
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      console.log('[FMS ServiceWorker] 정적 자원 사전 캐싱 완료');
      return cache.addAll(STATIC_ASSETS).catch((err) => {
        console.warn('[FMS ServiceWorker] 일부 외부 자원 캐싱 스킵:', err);
      });
    })
  );
  self.skipWaiting();
});

// 2. 서비스 워커 활성화: 구버전 캐시 정리
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) => {
      return Promise.all(
        keys.map((key) => {
          if (key !== CACHE_NAME) {
            console.log('[FMS ServiceWorker] 구버전 캐시 삭제:', key);
            return caches.delete(key);
          }
        })
      );
    })
  );
  self.clients.claim();
});

// 3. 네트워크 요청 가로채기 (Network-First -> Offline Cache Fallback)
self.addEventListener('fetch', (event) => {
  // POST 등 비-GET 요청이나 API 호출은 네트워크 직접 통신
  if (event.request.method !== 'GET') return;
  
  const url = new URL(event.request.url);

  // 로컬 백엔드 API 요청인 경우 네트워크 실패 시 오프라인 응답
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(
      fetch(event.request).catch(() => {
        return new Response(JSON.stringify({ error: 'OFFLINE_MODE', message: '현재 오프라인 상태입니다.' }), {
          headers: { 'Content-Type': 'application/json' }
        });
      })
    );
    return;
  }

  // 앱 쉘 및 정적 파일: Network first with Cache fallback
  event.respondWith(
    fetch(event.request)
      .then((networkResponse) => {
        if (networkResponse && networkResponse.status === 200) {
          const responseToCache = networkResponse.clone();
          caches.open(CACHE_NAME).then((cache) => {
            cache.put(event.request, responseToCache);
          });
        }
        return networkResponse;
      })
      .catch(() => {
        return caches.match(event.request).then((cachedResponse) => {
          if (cachedResponse) {
            return cachedResponse;
          }
          // 탐색 요청(HTML)인 경우 기본 index.html 반환
          if (event.request.mode === 'navigate') {
            return caches.match('./index.html');
          }
        });
      })
  );
});
