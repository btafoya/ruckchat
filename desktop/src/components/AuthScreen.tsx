import type { JSX } from 'react';
import { Navigate } from 'react-router-dom';
import { AuthForm } from './AuthForm';
import { useSessionContext } from '../context';

export function AuthScreen(): JSX.Element {
  const { session, isLoading, error, login, register } = useSessionContext();

  if (isLoading) {
    return <div className="flex h-screen items-center justify-center bg-gray-900 text-white">Loading...</div>;
  }

  if (session) {
    return <Navigate to="/" replace />;
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-900 px-4">
      <AuthForm mode="login" onLogin={login} onRegister={register} error={error} isLoading={isLoading} />
    </div>
  );
}
