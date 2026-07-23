import type { JSX } from 'react';
import { BrowserRouter } from 'react-router-dom';
import PlatformShell from './PlatformShell';
import { desktopPlatform } from './platform/desktop';

export default function App(): JSX.Element {
  return (
    <BrowserRouter>
      <PlatformShell platform={desktopPlatform} />
    </BrowserRouter>
  );
}
