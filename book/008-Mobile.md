# 008 - Mobile

## Mobile Client

The mobile client is a Flutter application for Android and iOS. It provides the same core messaging features as the desktop client with a touch-first interface.

## Technology Stack

| Layer | Technology |
|-------|------------|
| Framework | Flutter |
| Language | Dart |
| State management | Riverpod |
| HTTP | `package:http` |
| WebSocket | `package:web_socket_channel` |
| Local storage | `shared_preferences` |
| JSON serialization | `json_serializable` |

## Navigation Structure

1. **Organization Switcher** — shown when the user belongs to multiple organizations.
2. **Conversation List** — channels and DMs grouped by section.
3. **Chat View** — message history, composer, and attachments.
4. **Thread View** — replies to a parent message.
5. **Profile / Settings** — account, notifications, and organization settings.

## Screen Layout

- Bottom navigation on phones: Conversations, Notifications, Profile.
- Tablets use a two-pane layout similar to the desktop client.
- Messages render in a scrollable list with date separators.
- Composer anchors to the bottom of the screen and grows up to a max height.

## Real-Time Behavior

- WebSocket connection is maintained while the app is in the foreground.
- When backgrounded, the connection is closed and the app relies on push notifications for new messages.
- On resume, the app reconnects and fetches missed history.

## Push Notifications

- v1 uses local notifications while the app is in the foreground.
- Background push notifications are a post-MVP feature because they require Firebase Cloud Messaging (Android) and Apple Push Notification service (iOS), which introduce external dependencies.

## Offline Behavior

- Drafts are saved to `shared_preferences`.
- Failed sends remain in the composer with a retry button.
- Read state is reconciled on reconnect.

## Platform-Specific Behavior

- Android: respect system dark mode, use system back gesture.
- iOS: use Cupertino-style navigation when it matches the system better; support dynamic type.

## Build and Release

- Android: `flutter build apk` and `flutter build appbundle`.
- iOS: `flutter build ios` and `flutter build ipa`.
- Release artifacts are signed through platform-specific CI steps.

## Security

- Session cookie stored securely by the HTTP client.
- No cleartext passwords in logs or local storage.
- Biometric lock is a post-MVP feature.
