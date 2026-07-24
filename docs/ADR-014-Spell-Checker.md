# ADR-014: Spell Checker

## Status

Accepted — implemented in Phase 2 (ISSUES2).

## Context

The Tiptap composer needed inline spell checking beyond the browser's native
`spellcheck` attribute, which is inconsistent across browsers and cannot be
styled or backed by a shared server-side dictionary. ISSUES2 specified
integrating `@farscrl/tiptap-extension-spellchecker` on the frontend with a
server-side Hunspell-based API, embedding LibreOffice `en-US` dictionaries
directly in the server binary rather than running a separate dictionary
service or container.

We needed to decide how to implement the Hunspell engine in Rust, how to
expose it over HTTP without becoming a spam/DoS vector, and how operators
toggle the feature.

## Decision

- **Engine crate**: `crates/ruckchat-spelling` wraps
  [`spellbook`](https://crates.io/crates/spellbook), a pure-Rust Hunspell-format
  implementation, instead of `hunspell-sys` as originally scoped in ISSUES2.
  This avoids a C++ compiler dependency in the build and Docker images while
  reading the same `.aff`/`.dic` dictionary format. LibreOffice's `en-US`
  dictionary files are embedded at compile time with `include_str!`.
- **Service-layer rate limiting**: `server/src/services/spelling.rs` wraps the
  engine with a per-user token-bucket limiter (10 requests/second burst,
  100/minute) and input length caps (10,000 bytes of text, 100-byte words),
  independent of any HTTP-level rate limiting.
- **REST surface**: `POST /api/v1/spelling/check`, `POST
  /api/v1/spelling/suggest`, and `GET /api/v1/spelling/languages`, all
  requiring authentication.
- **Server settings gate**: `spelling_enabled` (default `true`) and
  `spelling_default_language` (default `en-US`) are added to the existing
  database-backed `server_settings` table from ADR-013, with the same YAML
  override precedence. When disabled, the endpoints return empty results
  rather than an error, so the composer extension degrades silently instead of
  surfacing failures to users.
- **Frontend integration**: `desktop/src/spelling/SpellingProofreader.ts`
  implements the extension's `IProofreaderInterface`, calling the REST
  endpoints and caching per-word suggestions for one minute.

## Consequences

### Positive

- No additional runtime dependency (container, sidecar, or C++ toolchain) is
  needed to ship spell checking.
- Rate limiting is enforced once, in the service layer, protecting the
  CPU-bound dictionary lookups regardless of which handler or future client
  calls them.
- The `spelling_enabled` flag lets operators disable the feature without a
  restart, consistent with other server settings.

### Negative

- `spellbook` only supports one embedded dictionary (`en-US`) today; adding
  languages means bundling more `.aff`/`.dic` pairs and extending
  `SpellingEngine` to hold multiple dictionaries, not just swapping a config
  value.
- The in-memory per-user rate-limit map in `SpellingService` is unbounded and
  never evicts entries for users who stop connecting; acceptable at current
  scale, but a future concern for very large, long-lived deployments.

## Implementation

- `crates/ruckchat-spelling/src/lib.rs` — `SpellingEngine`, embedded
  dictionaries, misspelling detection and suggestion lookup.
- `server/src/services/spelling.rs` — rate limiting, validation, and the
  `SpellingService` used by handlers.
- `server/src/handlers/spelling.rs` — REST handlers for check/suggest/languages.
- `server/src/state.rs` — `AppState.spelling: Option<SpellingService>`,
  initialized from the embedded dictionary at startup.
- `crates/ruckchat-domain/src/server_settings.rs`,
  `crates/ruckchat-config/src/lib.rs`,
  `server/src/repositories/server_settings.rs`,
  `server/src/services/server_settings.rs` — `spelling_enabled` and
  `spelling_default_language` settings and YAML overrides.
- `server/tests/spelling.rs` — integration tests for all three endpoints,
  including the disabled-via-settings path.
- `server/openapi.yaml` — `Spelling` tag, three paths, and request/response
  schemas.
- `desktop/src/spelling/SpellingProofreader.ts` — `IProofreaderInterface`
  implementation.
- `desktop/src/components/Composer.tsx` — wires
  `SpellcheckerExtension.configure({ proofreader })` into the Tiptap editor.
- `desktop/src/styles/theme.css` — `.spell-error` and `#suggestions-box`
  styling using existing theme tokens.
- `desktop/src/api/spelling.ts` — REST client used by the proofreader.

## Related

- `docs/issues/ISSUES2.md`
- `docs/issues/WORKFLOW.md`
- `book/019-Web-UI.md`
- `docs/ADR-013-Web-UI-Admin-Panel.md`
