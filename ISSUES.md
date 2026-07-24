# Issues found while testing

## Overall UI

- This is meant to be a slack replcement - some features such as mention @ are missing - open
- Needs a light theme with light/dark toggle - open
- Implement WYSIWYG Tiptap (<https://github.com/ueberdosis/tiptap>) in place of current text area with spell check (<https://github.com/farscrl/tiptap-extension-spellchecker>) - open

## Chat UI

- If the user belongs to one organization it should take them directly to the #general channel when logging in - open
- No method for creating channels (CRUD - Also public and private channels with user invite CRUD for private channels) - Modal like <https://github.com/block/buzz/blob/main/docs/assets/screenshots/create-channel.png> - open
- Direct messages is missing all functionality. Fully complete the UI allowing users to message others in their organization in the same fashion slack does - open

## Admin UI

- Server Admin UI and Organization Admin UI both have no link to return to the chat UI - open
- Organization Admin UI is incomplete - open
- The user editor is just a line editor in the user list when it should be a modal with the ability to manage all aspects of the user account - open
- Add to site setting allow registration checkbox option defaulting to on and add logic to allow/deny user registrations based on the setting - open
