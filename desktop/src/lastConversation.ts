const KEY_PREFIX = 'ruckchat_last_conversation_';

export interface LastConversation {
  type: 'channel' | 'dm';
  id: string;
}

export function getLastConversation(organizationId: string): LastConversation | null {
  try {
    const raw = localStorage.getItem(KEY_PREFIX + organizationId);
    if (!raw) {
      return null;
    }
    const parsed = JSON.parse(raw) as unknown;
    if (
      typeof parsed === 'object' &&
      parsed !== null &&
      ((parsed as LastConversation).type === 'channel' || (parsed as LastConversation).type === 'dm') &&
      typeof (parsed as LastConversation).id === 'string'
    ) {
      return parsed as LastConversation;
    }
  } catch {
    // ignore corrupted storage
  }
  return null;
}

export function setLastConversation(organizationId: string, conversation: LastConversation): void {
  try {
    localStorage.setItem(KEY_PREFIX + organizationId, JSON.stringify(conversation));
  } catch {
    // ignore storage failures
  }
}
