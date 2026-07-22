import { act, renderHook } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { useSettings } from './useSettings';
import { DEFAULT_API_URL } from '../config';

describe('useSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('returns defaults when no settings are stored', () => {
    const { result } = renderHook(() => useSettings());
    expect(result.current.apiUrl).toBe(DEFAULT_API_URL);
    expect(result.current.notificationsEnabled).toBe(true);
  });

  it('persists api url changes to localStorage', () => {
    const { result } = renderHook(() => useSettings());

    act(() => {
      result.current.setApiUrl('http://example.com:8080');
    });

    expect(result.current.apiUrl).toBe('http://example.com:8080');
    const stored = JSON.parse(localStorage.getItem('ruckchat_settings') ?? '{}');
    expect(stored.apiUrl).toBe('http://example.com:8080');
  });

  it('falls back to default for empty api urls', () => {
    const { result } = renderHook(() => useSettings());

    act(() => {
      result.current.setApiUrl('');
    });

    expect(result.current.apiUrl).toBe(DEFAULT_API_URL);
  });

  it('restores stored settings on mount', () => {
    localStorage.setItem(
      'ruckchat_settings',
      JSON.stringify({ apiUrl: 'http://restored.test', notificationsEnabled: false }),
    );

    const { result } = renderHook(() => useSettings());
    expect(result.current.apiUrl).toBe('http://restored.test');
    expect(result.current.notificationsEnabled).toBe(false);
  });

  it('ignores corrupted storage', () => {
    localStorage.setItem('ruckchat_settings', 'not-json');
    const { result } = renderHook(() => useSettings());
    expect(result.current.apiUrl).toBe(DEFAULT_API_URL);
  });
});
