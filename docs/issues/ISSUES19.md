# ISSUES19 — Remove Markdown Preview from Composer

## Source

> Remove preview - this is now a wysiwyg and no longer needed — open

## Research Summary

### Current state

- `Composer.tsx` uses Tiptap and stores ProseMirror JSON as the message content.
- The component still maintains `showPreview` state and renders a preview mode (`desktop/src/components/Composer.tsx:74`).
- The preview toggle and rendering code are leftovers from the previous plain-text composer with Markdown preview.
- The user is no longer typing raw Markdown; Tiptap renders formatting live, making a separate preview redundant.

### Gaps

1. **Preview state and UI still present** — remove `showPreview`, `setShowPreview`, and any preview rendering branch.
2. **Preview-specific rendering code** — `MessageContent` may be used inside the composer for preview; verify it is not needed.
3. **Toolbar layout** — removing the preview toggle may leave an empty slot or change the toolbar composition.
4. **Tests** — `Composer.test.tsx` may assert preview behavior that must be removed or updated.

### Affected files

- `desktop/src/components/Composer.tsx` — remove preview state, toggle button, and conditional rendering.
- `desktop/src/components/Composer.test.tsx` — remove or update preview-related tests.
- `desktop/src/components/MessageContent.tsx` — used for message rendering; still needed, but ensure no composer-only preview logic.

## Open Questions

1. **Should any raw-source/debug view remain?**
   - No, keep the composer surface purely WYSIWYG.
   - Yes, keep a hidden developer-only JSON inspector.

2. **What happens to the toolbar space?**
   - Remove the button and let the formatting toolbar flow naturally.
   - Replace it with a send shortcut hint or attachment button.

## Decisions

- Remove the `showPreview` state, the Preview/Edit toggle button, the preview rendering branch, the Escape-to-close preview handler, and all preview-related tests. The composer is purely WYSIWYG via Tiptap.

## Status

Complete.
