/* eslint-disable no-restricted-globals */

const CACHE_NAME = 'ruckchat-assets-__BUILD_HASH__';
const STATIC_PATHS =
  /^\/(?:assets\/(?:index-[^/]+\.(?:js|css)|[^/]+\.(?:js|css|png|svg|woff2)))?$|^\/(?:icons\/[^/]+|manifest\.json|favicon\.ico|index\.html)?$/;

// Set during install: true when an older cache (from a previous deploy)
// existed, false on a genuine first-ever install. Used in activate to decide
// whether already-open tabs are running stale JS and need a forced reload -
// their in-memory app code has no way to notice the update on its own.
let isUpdate = false;

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((names) => {
        isUpdate = names.length > 0 && !names.includes(CACHE_NAME);
        return Promise.all(
          names
            .filter((name) => name !== CACHE_NAME)
            .map((name) => caches.delete(name)),
        );
      })
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    self.clients.claim().then(() => {
      if (!isUpdate) {
        return undefined;
      }
      return self.clients.matchAll({ type: 'window' }).then((clientList) =>
        Promise.all(clientList.map((client) => client.navigate(client.url))),
      );
    }),
  );
});

self.addEventListener('push', (event) => {
  if (!event.data) {
    return;
  }

  let payload;
  try {
    payload = event.data.json();
  } catch {
    payload = { title: 'RuckChat', body: event.data.text() };
  }

  const title = payload.title ?? 'RuckChat';
  const options = {
    body: payload.body ?? '',
    icon: payload.icon ?? '/icons/icon-192x192.png',
    badge: payload.badge ?? '/icons/icon-192x192.png',
    data: payload.data ?? {},
  };

  event.waitUntil(self.registration.showNotification(title, options));
});

self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  const url = event.notification.data?.url ?? '/';
  event.waitUntil(
    self.clients
      .matchAll({ type: 'window', includeUncontrolled: true })
      .then((clientList) => {
        for (const client of clientList) {
          if (client.url === url && 'focus' in client) {
            return client.focus();
          }
        }
        if (self.clients.openWindow) {
          return self.clients.openWindow(url);
        }
      }),
  );
});

self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') {
    return;
  }
  const requestUrl = new URL(event.request.url);
  if (
    requestUrl.origin !== self.location.origin ||
    !STATIC_PATHS.test(requestUrl.pathname)
  ) {
    return;
  }

  event.respondWith(
    caches.match(event.request).then((cached) => {
      if (cached) {
        return cached;
      }
      return fetch(event.request).then((response) => {
        const clone = response.clone();
        caches.open(CACHE_NAME).then((cache) => cache.put(event.request, clone));
        return response;
      });
    }),
  );
});
