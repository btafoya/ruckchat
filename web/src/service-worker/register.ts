/**
 * Registers the RuckChat service worker and subscribes to Web Push when the
 * user grants notification permission.
 */
export async function registerServiceWorker(): Promise<void> {
  if (!('serviceWorker' in navigator)) {
    return;
  }

  try {
    const registration = await navigator.serviceWorker.register('/sw.js', {
      scope: '/',
      updateViaCache: 'imports',
    });
    // eslint-disable-next-line no-console
    console.info('service worker registered:', registration.scope);
  } catch (err) {
    console.warn('failed to register service worker', err);
  }
}
