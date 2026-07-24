# ISSUES2 — WYSIWYG Tiptap Composer with Spell Check

## Source

> Implement WYSIWYG Tiptap (<https://github.com/ueberdosis/tiptap>) in place of current text area with spell check (<https://github.com/farscrl/tiptap-extension-spellchecker>) — open

## Research Summary

### Current state

- The composer uses a plain `textarea` (`desktop/src/components/Composer.tsx:189-196`).
- It supports a markdown preview toggle (`Composer.tsx:151-160`) but no inline formatting.
- Basic `@` mention autocomplete is implemented by manually positioning a dropdown over the textarea (`Composer.tsx:198-212`).
- File attachments are handled outside the editor via a `pendingFiles` list (`Composer.tsx:43`, `164-183`).
- Dependencies are not yet checked for Tiptap packages.

### Gaps

1. **Editor replacement** — replace the textarea with `@tiptap/react` and `@tiptap/starter-kit` (or a custom extension set).
2. **Mention extension** — migrate the existing `@` autocomplete to a Tiptap mention node that stores `user_id` and renders `@display_name`.
3. **Spell check** — integrate the requested `farscrl/tiptap-extension-spellchecker` extension, or verify browser spell-check behavior if Tiptap content-editable supports it.
4. **Markdown parity** — decide whether to keep markdown preview, keep a markdown mode, or rely on Tiptap's WYSIWYG output.
5. **File attachments** — ensure the editor can coexist with the existing attachment UI, or move attachments into the editor as nodes.
6. **Keyboard shortcuts** — preserve `Enter` to send and `Shift+Enter` for newline, which currently rely on textarea key handling (`Composer.tsx:141-149`).

### Affected files

- `desktop/package.json` / `web/package.json` — add Tiptap and spell-check dependencies.
- `desktop/src/components/Composer.tsx` — replace textarea with Tiptap editor.
- `desktop/src/components/MessageItem.tsx` — render Tiptap/mention nodes if stored as structured JSON.
- `desktop/src/hooks/useMessages.ts` — ensure message `content` format agreement with backend.
- `server/openapi.yaml` and message service — clarify whether `content` remains plain text, Markdown, or ProseMirror JSON.

## Open Questions

1. **What should the message `content` format be after the switch?**
   - Continue storing plain text/Markdown and render it through Tiptap on read.
   - Store ProseMirror JSON in the database and render it directly.

2. **Which Tiptap extensions are required for MVP?**
   - Starter kit only (bold, italic, headings, lists, blockquote, code).
   - Starter kit plus mention and spell-check only.
   - Full Slack-like formatting (inline code, code blocks, strikethrough).

3. **How should spell check behave?**
   - Use the third-party `tiptap-extension-spellchecker` with a backend or local dictionary.
   - Rely on browser spell-check via `spellCheck={true}` on the content-editable surface.

4. **Should the existing Markdown preview feature survive?**
   - Keep it as an alternate view.
   - Remove it because WYSIWYG is the preview.

## Decisions

- Message content format after Tiptap: store ProseMirror JSON.
- Spell checking: integrate `tiptap-extension-spellchecker` with a dictionary/backend service.
- Markdown preview: remove; WYSIWYG replaces it.
