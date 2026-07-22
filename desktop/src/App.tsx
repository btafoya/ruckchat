import type { JSX } from 'react';
import { Route, Routes, Navigate } from 'react-router-dom';
import { SessionProvider, useSession } from './hooks';
import { AuthScreen, Shell } from './components';

export default function App(): JSX.Element {
  const sessionState = useSession();

  return (
    <SessionProvider value={sessionState}>
      <Routes>
        <Route path="/login" element={<AuthScreen />} />
        <Route path="/*" element={<Shell />}>
          <Route index element={<Navigate to="/org" replace />} />
          <Route path="org" element={<div />} />
          <Route path="org/:organizationId/channel" element={<div />} />
          <Route path="org/:organizationId/channel/:channelId" element={<div />} />
        </Route>
      </Routes>
    </SessionProvider>
  );
}
