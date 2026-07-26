import type { JSX } from 'react';
import { BrowserRouter } from 'react-router-dom';
import PlatformShell from './PlatformShell';
import { desktopPlatform } from './platform/desktop';
import { useSettings } from './hooks';
import { SettingsProvider } from './context';
import { FirstRunSetup } from './components';

export default function App(): JSX.Element {
  const settingsState = useSettings();
  const needsSetup = !settingsState.isLoading && !settingsState.serverUrlConfigured;
  return (
    <BrowserRouter>
      <SettingsProvider value={settingsState}>
        {needsSetup ? <FirstRunSetup /> : <PlatformShell platform={desktopPlatform} />}
      </SettingsProvider>
    </BrowserRouter>
  );
}
