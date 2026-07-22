import { useCallback, useEffect, useMemo, useState } from 'react';
import { createApi, isUnauthorizedError } from '../api';
import type { LoginRequest, RegisterRequest, User } from '../api';
import { useSettings } from './useSettings';

const TOKEN_KEY = 'ruckchat_session_token';

export interface Session {
  token: string;
  user: User;
}

export interface SessionState {
  session: Session | null;
  isLoading: boolean;
  error: string | null;
  login: (request: LoginRequest) => Promise<void>;
  register: (request: RegisterRequest) => Promise<void>;
  logout: () => Promise<void>;
}

export function useSession(): SessionState {
  const [session, setSession] = useState<Session | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);

  useEffect(() => {
    let cancelled = false;

    async function restore() {
      const token = localStorage.getItem(TOKEN_KEY);
      if (!token) {
        setIsLoading(false);
        return;
      }

      try {
        const user = await api.auth.getProfile(token);
        if (!cancelled) {
          setSession({ token, user });
        }
      } catch (err) {
        if (isUnauthorizedError(err)) {
          localStorage.removeItem(TOKEN_KEY);
        }
      } finally {
        if (!cancelled) {
          setIsLoading(false);
        }
      }
    }

    void restore();
    return () => {
      cancelled = true;
    };
  }, []);

  const login = useCallback(
    async (request: LoginRequest) => {
      setIsLoading(true);
      setError(null);
      try {
        const response = await api.auth.login(request);
        localStorage.setItem(TOKEN_KEY, response.token);
        setSession({ token: response.token, user: response.user });
      } catch (err) {
        if (err instanceof Error) {
          setError(err.message);
        } else {
          setError('Login failed');
        }
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [api],
  );

  const register = useCallback(
    async (request: RegisterRequest) => {
      setIsLoading(true);
      setError(null);
      try {
        const response = await api.auth.register(request);
        // The register endpoint returns user and organization, not a token.
        // Log in with the same credentials to establish a session.
        const loginResponse = await api.auth.login({
          email: request.email,
          password: request.password,
        });
        localStorage.setItem(TOKEN_KEY, loginResponse.token);
        setSession({ token: loginResponse.token, user: loginResponse.user });
      } catch (err) {
        if (err instanceof Error) {
          setError(err.message);
        } else {
          setError('Registration failed');
        }
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [api],
  );

  const logout = useCallback(async () => {
    if (!session) {
      return;
    }
    setIsLoading(true);
    try {
      await api.auth.logout(session.token);
    } catch {
      // Ignore logout failures; clear local session regardless.
    } finally {
      localStorage.removeItem(TOKEN_KEY);
      setSession(null);
      setIsLoading(false);
    }
  }, [api, session]);

  return {
    session,
    isLoading,
    error,
    login,
    register,
    logout,
  };
}
