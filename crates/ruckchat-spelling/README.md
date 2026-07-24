# ruckchat-spelling

Embedded English (en-US) spell-checking engine for RuckChat.

The crate embeds LibreOffice Hunspell `.aff` and `.dic` files at compile time
and exposes a small `Send + Sync` `SpellingEngine` that can:

- `check(text, max_suggestions)` — scan a string and report misspellings with
  optional suggestions.
- `suggest(word, max)` — return suggestions for a single word.
- `language()` — return the configured language tag (`en-US`).

The engine is backed by the pure-Rust [`spellbook`](https://crates.io/crates/spellbook)
Hunspell implementation, which avoids a C++ compiler dependency and lets the
dictionaries be embedded as strings.
