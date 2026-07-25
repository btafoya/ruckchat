import { renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { useRealtimeStore } from './useRealtimeStore';
import type { MessagesState } from './useMessages';
import type { PresenceState } from './usePresence';
import type { TypingState } from './useTyping';
import type { ReadState } from './useReadState';

// These payloads are raw JSON strings, decoded at runtime like the real
// WebSocket message handler does - not TypeScript object literals - so this
// test catches drift between the client's `ServerEvent` type literals
// (desktop/src/api/events.ts) and the server's actual
// `#[serde(tag = "type", rename_all = "snake_case")]` wire format
// (server/src/services/events.rs), which a compile-time-checked object
// literal would silently miss. See ADR-006.
function decodePayload(rawEnvelope: string): unknown {
  return JSON.parse(rawEnvelope).payload;
}

function mockMessages(): MessagesState {
  return {
    messages: [],
    isLoading: false,
    isLoadingOlder: false,
    isLoadingNewer: false,
    error: null,
    hasMoreOlder: false,
    hasMoreNewer: false,
    unreadIds: new Set(),
    lastAppendedId: null,
    refresh: vi.fn(),
    loadOlder: vi.fn(),
    loadNewer: vi.fn(),
    jumpToMessage: vi.fn(),
    sendMessage: vi.fn(),
    retryMessage: vi.fn(),
    loadThreadReplies: vi.fn(),
    loadOlderReplies: vi.fn(),
    loadNewerReplies: vi.fn(),
    threadReplies: [],
    threadRepliesLoading: false,
    threadHasMoreOlder: false,
    threadHasMoreNewer: false,
    reactions: {},
    addReaction: vi.fn(),
    removeReaction: vi.fn(),
    appendMessage: vi.fn(),
    updateMessage: vi.fn(),
    removeMessage: vi.fn(),
    markRead: vi.fn(),
    editingMessage: null,
    startEdit: vi.fn(),
    cancelEdit: vi.fn(),
    saveEdit: vi.fn(),
  };
}

function mockPresence(): PresenceState {
  return { presence: {}, setUserPresence: vi.fn() };
}

function mockTyping(): TypingState {
  return { typingUsers: {}, addTypingUser: vi.fn(), removeTypingUser: vi.fn() };
}

function mockReadState(): ReadState {
  return {
    counts: {},
    total: 0,
    increment: vi.fn(),
    markRead: vi.fn(),
    applyRemoteRead: vi.fn(),
    refresh: vi.fn(),
  };
}

describe('useRealtimeStore', () => {
  it('dispatches a real message_created wire payload to appendMessage', () => {
    const messages = mockMessages();
    const { result } = renderHook(() =>
      useRealtimeStore(messages, mockPresence(), mockTyping(), mockReadState()),
    );

    const payload = decodePayload(
      '{"type":"message.created","id":"e1","timestamp":"2026-01-01T00:00:00Z","payload":{"type":"message_created","message":{"id":"msg-1","conversation_id":"conv-1"}}}',
    );

    // @ts-expect-error - runtime-decoded payload, not a typed ServerEvent literal
    result.current.onEvent(payload);

    expect(messages.appendMessage).toHaveBeenCalledWith({ id: 'msg-1', conversation_id: 'conv-1' });
  });

  it('dispatches a real presence wire payload to setUserPresence', () => {
    const presence = mockPresence();
    const { result } = renderHook(() =>
      useRealtimeStore(mockMessages(), presence, mockTyping(), mockReadState()),
    );

    const payload = decodePayload(
      '{"type":"presence.updated","id":"e2","timestamp":"2026-01-01T00:00:00Z","payload":{"type":"presence","user_id":"user-1","status":"online"}}',
    );

    // @ts-expect-error - runtime-decoded payload, not a typed ServerEvent literal
    result.current.onEvent(payload);

    expect(presence.setUserPresence).toHaveBeenCalledWith('user-1', 'online');
  });

  it('dispatches a real read_state_updated wire payload to applyRemoteRead and markRead', () => {
    const messages = mockMessages();
    const readState = mockReadState();
    const { result } = renderHook(() =>
      useRealtimeStore(messages, mockPresence(), mockTyping(), readState),
    );

    const payload = decodePayload(
      '{"type":"read_state.updated","id":"e3","timestamp":"2026-01-01T00:00:00Z","payload":{"type":"read_state_updated","conversation_id":"conv-1","message_ids":["msg-1","msg-2"]}}',
    );

    // @ts-expect-error - runtime-decoded payload, not a typed ServerEvent literal
    result.current.onEvent(payload);

    expect(readState.applyRemoteRead).toHaveBeenCalledWith('conv-1');
    expect(messages.markRead).toHaveBeenCalledWith('msg-1');
    expect(messages.markRead).toHaveBeenCalledWith('msg-2');
  });
});
