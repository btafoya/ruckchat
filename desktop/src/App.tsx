import type { JSX } from 'react';
import { BrowserRouter } from 'react-router-dom';
import PlatformShell from './PlatformShell';
import { desktopPlatform } from './platform/desktop';
import { useSettings } from './hooks';
import { SettingsProvider } from './context';

export default function App(): JSX.Element {
  const settingsState = useSettings();
  return (
    <BrowserRouter>
      <SettingsProvider value={settingsState}>
        <PlatformShell platform={desktopPlatform} />
      </SettingsProvider>
    </BrowserRouter>
  );
}
