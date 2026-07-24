import type { IProofreaderInterface, ITextWithPosition } from '@farscrl/tiptap-extension-spellchecker';
import type { RuckChatApi } from '../api';

const SUGGESTION_CACHE_TTL_MS = 60_000;

interface CachedSuggestions {
  suggestions: string[];
  expiresAt: number;
}

/** Calls the server-side Hunspell API to back the Tiptap spellchecker extension. */
export class SpellingProofreader implements IProofreaderInterface {
  private readonly suggestionCache = new Map<string, CachedSuggestions>();

  constructor(
    private readonly api: RuckChatApi,
    private readonly getToken: () => string | undefined,
  ) {}

  normalizeTextForLanguage(text: string): string {
    return text
      .normalize('NFD')
      .replace(/\p{Diacritic}/gu, '')
      .toLowerCase();
  }

  async proofreadText(sentence: string): Promise<ITextWithPosition[]> {
    const token = this.getToken();
    if (!token) {
      return [];
    }
    try {
      const response = await this.api.spelling.check(token, { text: sentence });
      return response.misspellings.map((m) => ({
        offset: m.offset,
        length: m.length,
        word: m.word,
      }));
    } catch {
      return [];
    }
  }

  async getSuggestions(word: string): Promise<string[]> {
    const key = this.normalizeTextForLanguage(word);
    const cached = this.suggestionCache.get(key);
    if (cached && cached.expiresAt > Date.now()) {
      return cached.suggestions;
    }
    const token = this.getToken();
    if (!token) {
      return [];
    }
    try {
      const response = await this.api.spelling.suggest(token, { word });
      this.suggestionCache.set(key, {
        suggestions: response.suggestions,
        expiresAt: Date.now() + SUGGESTION_CACHE_TTL_MS,
      });
      return response.suggestions;
    } catch {
      return [];
    }
  }
}
