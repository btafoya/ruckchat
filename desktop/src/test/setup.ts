import '@testing-library/jest-dom/vitest';
import { mockTauriPlugins } from './tauriMocks';

mockTauriPlugins();

if (typeof window.HTMLElement.prototype.scrollIntoView === 'undefined') {
  window.HTMLElement.prototype.scrollIntoView = () => {};
}

if (typeof window.matchMedia === 'undefined') {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: (query: string) => ({
      matches: false,
      media: query,
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
    }),
  });
}
