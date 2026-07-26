import { useEffect, useMemo, useState, type JSX } from 'react';
import { Navigate } from 'react-router-dom';
import { AuthForm } from './AuthForm';
import { createApi } from '../api';
import { useSessionContext, useSettingsContext } from '../context';

export function AuthScreen(): JSX.Element {
  const { session, isLoading, error, login, register } = useSessionContext();
  const { apiUrl, serverUrlConfigured } = useSettingsContext();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [allowRegistration, setAllowRegistration] = useState<boolean | null>(null);

  useEffect(() => {
    let cancelled = false;
    if (!serverUrlConfigured) {
      return;
    }
    api.auth
      .getRegistrationStatus()
      .then((status) => {
        if (!cancelled) {
          setAllowRegistration(status.allow_registration);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setAllowRegistration(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [api, serverUrlConfigured]);

  if (isLoading || allowRegistration === null) {
    return <div className="flex h-screen items-center justify-center bg-bg text-text">Loading...</div>;
  }

  if (session) {
    return <Navigate to="/" replace />;
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-bg px-4">
      <AuthForm
        mode={allowRegistration ? 'login' : 'login'}
        onLogin={login}
        onRegister={register}
        error={error}
        isLoading={isLoading}
        allowRegistration={allowRegistration}
      />
    </div>
  );
}
