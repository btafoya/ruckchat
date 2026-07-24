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
}

export function AuthForm({ mode, onLogin, onRegister, error, isLoading }: AuthFormProps): JSX.Element {
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
    <div className="flex w-full max-w-md flex-col gap-6 rounded-lg bg-gray-800 p-8 shadow-lg">
      <h2 className="text-center text-2xl font-bold text-white">
        {formMode === 'login' ? 'Sign in to RuckChat' : 'Create your account'}
      </h2>
      {error && (
        <div role="alert" className="rounded-md bg-red-900/50 p-3 text-sm text-red-100">
          {error}
        </div>
      )}
      <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
        <label className="flex flex-col gap-1 text-sm text-gray-300">
          Email
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            required
            className="rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-white focus:border-green-500 focus:outline-none"
          />
        </label>
        <label className="flex flex-col gap-1 text-sm text-gray-300">
          Password
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            minLength={10}
            className="rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-white focus:border-green-500 focus:outline-none"
          />
        </label>
        {formMode === 'register' && (
          <>
            <label className="flex flex-col gap-1 text-sm text-gray-300">
              Display name
              <input
                type="text"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                required
                className="rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-white focus:border-green-500 focus:outline-none"
              />
            </label>
            <label className="flex flex-col gap-1 text-sm text-gray-300">
              Organization name
              <input
                type="text"
                value={organizationName}
                onChange={(e) => setOrganizationName(e.target.value)}
                required
                className="rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-white focus:border-green-500 focus:outline-none"
              />
            </label>
            <label className="flex flex-col gap-1 text-sm text-gray-300">
              Organization slug
              <input
                type="text"
                value={organizationSlug}
                onChange={(e) => setOrganizationSlug(e.target.value)}
                required
                className="rounded-md border border-gray-600 bg-gray-900 px-3 py-2 text-white focus:border-green-500 focus:outline-none"
              />
            </label>
          </>
        )}
        <button
          type="submit"
          disabled={isLoading}
          className="rounded-md bg-green-600 px-4 py-2 font-semibold text-white hover:bg-green-500 disabled:opacity-50"
        >
          {isLoading ? 'Please wait...' : formMode === 'login' ? 'Sign in' : 'Create account'}
        </button>
      </form>
      <button
        type="button"
        onClick={() => setFormMode((m) => (m === 'login' ? 'register' : 'login'))}
        className="text-sm text-green-400 hover:text-green-300"
      >
        {formMode === 'login'
          ? "Don't have an account? Create one"
          : 'Already have an account? Sign in'}
      </button>
    </div>
  );
}
