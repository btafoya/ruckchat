import { useEffect, type JSX, type ReactNode } from 'react';
import { useSettings } from '../hooks';

interface ThemeProviderProps {
  children: ReactNode;
}

export function ThemeProvider({ children }: ThemeProviderProps): JSX.Element {
  const { resolvedTheme } = useSettings();

  useEffect(() => {
    const root = document.documentElement;
    root.classList.remove('light', 'dark');
    root.classList.add(resolvedTheme);

    const meta = document.querySelector('meta[name="theme-color"]');
    if (meta) {
      meta.setAttribute('content', resolvedTheme === 'dark' ? '#111827' : '#ffffff');
    }
  }, [resolvedTheme]);

  return <>{children}</>;
}
