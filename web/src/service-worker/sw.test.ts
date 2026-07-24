import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const CURRENT_CACHE = 'ruckchat-assets-__BUILD_HASH__';

interface FakeEvent {
  waitUntil: (promise: Promise<unknown>) => void;
}

async function loadServiceWorker(existingCacheNames: string[]) {
  const handlers: Record<string, (event: FakeEvent) => void> = {};
  const claim = vi.fn().mockResolvedValue(undefined);
  const matchAll = vi.fn().mockResolvedValue([]);
  const cacheDelete = vi.fn().mockResolvedValue(true);
  const skipWaiting = vi.fn();

  vi.stubGlobal('self', {
    addEventListener: (type: string, handler: (event: FakeEvent) => void) => {
      handlers[type] = handler;
    },
    clients: { claim, matchAll },
    skipWaiting,
  });
  vi.stubGlobal('caches', {
    keys: vi.fn().mockResolvedValue(existingCacheNames),
    delete: cacheDelete,
  });

  vi.resetModules();
  await import('../../public/sw.js');

  async function fire(type: string) {
    let captured: Promise<unknown> | undefined;
    handlers[type]?.({
      waitUntil: (promise) => {
        captured = promise;
      },
    });
    await captured;
  }

  return { fire, claim, matchAll, cacheDelete, skipWaiting };
}

describe('sw.js', () => {
  beforeEach(() => {
    vi.resetModules();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('does not force-reload open tabs on a first-ever install', async () => {
    const { fire, matchAll } = await loadServiceWorker([]);

    await fire('install');
    await fire('activate');

    expect(matchAll).not.toHaveBeenCalled();
  });

  it('force-reloads already-open tabs when an older cache existed', async () => {
    const navigate = vi.fn().mockResolvedValue(undefined);
    const { fire, matchAll } = await loadServiceWorker(['ruckchat-assets-old-build']);
    matchAll.mockResolvedValue([{ url: 'https://example.test/channel/1', navigate }]);

    await fire('install');
    await fire('activate');

    expect(matchAll).toHaveBeenCalledWith({ type: 'window' });
    expect(navigate).toHaveBeenCalledWith('https://example.test/channel/1');
  });

  it('does not reload open tabs when the current build is already cached', async () => {
    const { fire, matchAll } = await loadServiceWorker([CURRENT_CACHE]);

    await fire('install');
    await fire('activate');

    expect(matchAll).not.toHaveBeenCalled();
  });
});
