# ISSUES2 — WYSIWYG Tiptap Composer with Spell Check

## Source

> Implement WYSIWYG Tiptap (<https://github.com/ueberdosis/tiptap>) in place of current text area with spell check (<https://github.com/farscrl/tiptap-extension-spellchecker>) — open

## Research Summary

### Current state

- The composer has been rewritten with Tiptap:
  - `@tiptap/react`, `@tiptap/starter-kit`, `@tiptap/extension-mention`,
    `@tiptap/extension-placeholder`, `@tiptap/suggestion`, `tippy.js`.
- Message content is now stored as ProseMirror JSON.
- `@display_name` mentions are first-class Tiptap `mention` nodes with
  `id` (user_id) and `label` (display_name) attributes.
- `MessageContent.tsx` renders ProseMirror JSON including styled mention nodes,
  inline marks (bold, italic, strike, code, link), and block nodes (paragraph,
  lists, blockquote, code block).
- Browser spell-check is enabled on the contenteditable surface via
  `spellcheck="true"`.

### Gaps (resolved)

1. **farscrl spell-checker integration** — wired into `Composer.tsx` via
   `SpellcheckerExtension.configure({ proofreader })`.
2. **Server-side spelling API** — `POST /api/v1/spelling/check`,
   `POST /api/v1/spelling/suggest`, and `GET /api/v1/spelling/languages` are
   implemented and rate-limited per user.
3. **Dictionary/backend proofreader** — `crates/ruckchat-spelling` embeds the
   `en-US` Hunspell dictionary via the pure-Rust `spellbook` crate; the
   frontend `SpellingProofreader` calls the REST endpoints.

### Decisions

- Message content format: ProseMirror JSON (already implemented).
- Markdown preview: removed; WYSIWYG is the preview.
- Spell checking: integrate `@farscrl/tiptap-extension-spellchecker` with a
  server-side Hunspell-based API embedded in the Rust server.
- Dictionaries: embed LibreOffice en-US Hunspell `.aff` / `.dic` files in the
  new `ruckchat-spelling` crate via `include_str!`.
- Engine: use the pure-Rust `spellbook` crate instead of `hunspell-sys`,
  avoiding a C++ compiler dependency in the build and Docker images (see
  `docs/ADR-014-Spell-Checker.md`).

## Proposed implementation

1. **New crate `crates/ruckchat-spelling`**
   - Wrap the pure-Rust `spellbook` Hunspell implementation.
   - Embed LibreOffice `en-US.aff` / `en-US.dic` dictionaries at compile time.
   - Provide `SpellingEngine`, `Send + Sync` via an internal `Arc<Dictionary>`.
   - APIs:
     - `SpellingEngine::embedded_en_us()` / `SpellingEngine::new(aff: &str, dic: &str, language)`
     - `check(text: &str, max_suggestions: usize) -> Vec<Misspelling>`
     - `suggest(word: &str, max: usize) -> Vec<String>`
     - `language() -> &'static str`

2. **Server changes**
   - Add `spelling_enabled` and `spelling_default_language` to
     `ruckchat_domain::ServerSettings`, the SQLx repository, and
     `ruckchat_config::ServerSettingsOverride`.
   - Add `server/src/services/spelling.rs` with input validation, per-user
     token-bucket rate limiting (10 req/s burst, 100 req/min), and
     `check`/`suggest`/`languages`.
   - Add `server/src/handlers/spelling.rs` for:
     - `POST /api/v1/spelling/check`
     - `POST /api/v1/spelling/suggest`
     - `GET /api/v1/spelling/languages`
   - Wire routes in `server/src/handlers/mod.rs`.
   - Add `pub spelling: Option<SpellingService>` to `AppState` and initialize
     it from the embedded dictionary.

3. **Frontend changes**
   - Add `@farscrl/tiptap-extension-spellchecker` to `desktop/package.json` and
     `web/package.json`.
   - Create `desktop/src/spelling/SpellingProofreader.ts` implementing
     `IProofreaderInterface`:
     - `proofreadText` calls `POST /api/v1/spelling/check` and returns
       `{ offset, length, word }[]`.
     - `getSuggestions` calls `POST /api/v1/spelling/suggest` and returns
       suggestions; cache results for 1 minute keyed by normalized word.
     - `normalizeTextForLanguage` lowercases and strips diacritics.
   - Wire the proofreader into `Composer.tsx` via
     `SpellcheckerExtension.configure({ proofreader })`.

4. **API / docs**
   - Update `server/openapi.yaml` with `SpellingCheckRequest`,
     `SpellingCheckResponse`, `SpellingSuggestRequest`,
     `SpellingSuggestResponse`, `SpellingLanguageList`, and the three endpoints.
   - Update `book/019-Web-UI.md` to document the spell-checker feature.
   - Add or update an ADR covering the decision to embed Hunspell instead of a
     separate container.

## Affected files

- `crates/ruckchat-spelling/Cargo.toml`
- `crates/ruckchat-spelling/src/lib.rs`
- `crates/ruckchat-spelling/src/engine.rs`
- `crates/ruckchat-spelling/src/dictionary.rs`
- `crates/ruckchat-spelling/dictionaries/en-US.aff`
- `crates/ruckchat-spelling/dictionaries/en-US.dic`
- `server/Cargo.toml` — add `ruckchat-spelling` dependency.
- `server/src/services/spelling.rs`
- `server/src/handlers/spelling.rs`
- `server/src/handlers/mod.rs`
- `server/src/state.rs`
- `server/src/services/server_settings.rs`
- `server/src/repositories/server_settings.rs`
- `crates/ruckchat-domain/src/server_settings.rs`
- `crates/ruckchat-config/src/lib.rs`
- `server/openapi.yaml`
- `desktop/package.json`
- `desktop/src/spelling/SpellingProofreader.ts`
- `desktop/src/components/Composer.tsx`
- `desktop/src/api/schema.ts`
- `book/019-Web-UI.md`
- `docs/ADR-*.md`

## Status

✅ Complete — Tiptap composer, mentions, and the `@farscrl` spell-checker
integration backed by the embedded `ruckchat-spelling` Hunspell engine are all
implemented, tested, and documented.
