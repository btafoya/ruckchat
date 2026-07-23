import type { JSX } from 'react';
import PlatformShell from '../../desktop/src/PlatformShell';
import { webPlatform } from '../../desktop/src/platform/web';

export default function App(): JSX.Element {
  return <PlatformShell platform={webPlatform} />;
}
