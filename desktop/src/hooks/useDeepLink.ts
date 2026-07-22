import { useEffect } from 'react';
import { getCurrent } from '@tauri-apps/plugin-deep-link';

export function useDeepLink(): void {
  useEffect(() => {
    let cancelled = false;
    async function check() {
      try {
        const urls = await getCurrent();
        if (!cancelled && urls && urls.length > 0) {
          // eslint-disable-next-line no-console
          console.info('deep-link url:', urls[0]);
        }
      } catch {
        // ignore deep link failures
      }
    }
    void check();
    return () => {
      cancelled = true;
    };
  }, []);
}
