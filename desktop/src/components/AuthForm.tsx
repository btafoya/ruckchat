import { useState } from 'react';
import type { JSX } from 'react';
import type { LoginRequest, RegisterRequest } from '../api';

type AuthMode = 'login' | 'register';

interface AuthFormProps {
  mode: AuthMode;
  onLogin: (request: LoginRequest) => Promise<void>;
  onRegister: (request: RegisterRequest) => Promise<void>;
  error: string | null;
  isLoading: boolean;
  allowRegistration?: boolean;
}

export function AuthForm({
  mode,
  onLogin,
  onRegister,
  error,
  isLoading,
  allowRegistration = true,
}: AuthFormProps): JSX.Element {
  const [formMode, setFormMode] = useState<AuthMode>(mode);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [organizationName, setOrganizationName] = useState('');
  const [organizationSlug, setOrganizationSlug] = useState('');

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (formMode === 'login') {
      await onLogin({ email, password });
    } else {
      await onRegister({
        email,
        password,
        display_name: displayName,
        organization_name: organizationName,
        organization_slug: organizationSlug,
      });
    }
  };

  return (
    <div className="flex w-full max-w-md flex-col gap-6 rounded-lg bg-surface p-8 shadow-lg">
      <h2 className="text-center text-2xl font-bold text-text">
        {formMode === 'login' ? 'Sign in to RuckChat' : 'Create your account'}
      </h2>
      {error && (
        <div role="alert" className="rounded-md bg-danger-bg p-3 text-sm text-danger">
          {error}
        </div>
      )}
      <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
        <label className="flex flex-col gap-1 text-sm text-text">
          Email
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            required
            className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
          />
        </label>
        <label className="flex flex-col gap-1 text-sm text-text">
          Password
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            minLength={10}
            className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
          />
        </label>
        {formMode === 'register' && (
          <>
            <label className="flex flex-col gap-1 text-sm text-text">
              Display name
              <input
                type="text"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                required
                className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
              />
            </label>
            <label className="flex flex-col gap-1 text-sm text-text">
              Organization name
              <input
                type="text"
                value={organizationName}
                onChange={(e) => setOrganizationName(e.target.value)}
                required
                className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
              />
            </label>
            <label className="flex flex-col gap-1 text-sm text-text">
              Organization slug
              <input
                type="text"
                value={organizationSlug}
                onChange={(e) => setOrganizationSlug(e.target.value)}
                required
                className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
              />
            </label>
          </>
        )}
        <button
          type="submit"
          disabled={isLoading}
          className="rounded-md bg-accent px-4 py-2 font-semibold text-text-inverse hover:bg-accent-hover disabled:opacity-50"
        >
          {isLoading ? 'Please wait...' : formMode === 'login' ? 'Sign in' : 'Create account'}
        </button>
      </form>
      {allowRegistration ? (
        <button
          type="button"
          onClick={() => setFormMode((m) => (m === 'login' ? 'register' : 'login'))}
          className="text-sm text-accent hover:text-accent-hover"
        >
          {formMode === 'login'
            ? "Don't have an account? Create one"
            : 'Already have an account? Sign in'}
        </button>
      ) : (
        <p className="text-sm text-text-muted">New registrations are disabled.</p>
      )}
    </div>
  );
}
