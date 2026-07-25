import type { JSX } from 'react';
import PlatformShell from '../../desktop/src/PlatformShell';
import { webPlatform } from '../../desktop/src/platform/web';
import { useSettings } from '../../desktop/src/hooks';
import { SettingsProvider } from '../../desktop/src/context';

export default function App(): JSX.Element {
  const settingsState = useSettings();
  return (
    <SettingsProvider value={settingsState}>
      <PlatformShell platform={webPlatform} />
    </SettingsProvider>
  );
}
