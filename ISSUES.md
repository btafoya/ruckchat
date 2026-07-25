# Issues found while testing - Alpha 3

## Server

- WHen I submit a message it is duplicated - open
- agent identified error: "Stopping Bash use — even a bare echo hello is now being blocked with the same message: something is wrapping every shell command through an unrequested binary (tokenjuice wrap --source claude-code) that neither I nor your config invoked. This started appearing partway through this session (earlier commands like cargo test, pnpm test, git status --porcelain ran fine and returned real output;  the wrapping/blocking only kicked in on the later calls). I won't try to route around it further — this looks like either a compromised PATH/shell rc file or an external process injecting itself into the session, and it's worth checking on your end before running any more shell commands in this session." - open

## Overall UI both Web and Desktop

- The sidebar needs to be able to collapse to from full to narrow using a collapse icon - open
- The sidebar is missing on mobile - open
- Admin menu items should move to the top bar far left using icons (from <https://fontawesome.com/search?ic=free-collection>) with mouseover tool tips to show the full menu item title - open
- On login if the user only belongs to one organization go straight to that organization when they are going to the 'home' view - basically the org view becomes the home view - open
- The org view should show all unread messages for the logged in user linked to the actual channel/direct message view - open
- The preferred theme doesn't survive logging out and returning; this should be a profile setting stored in the server user profile - open

## Chat UI both Web and Desktop

- Add delete option to message - open
- Remove preview - this is now a wysiwyg and no longer needed - open
- Finish direct messages - clicking on + brings up a modal with no users listed - finish CRUD - open

## Admin UI both Web and Desktop

- Finish CRUD for `Server Admins View` list (Completely missing) - open
