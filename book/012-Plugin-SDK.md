# 012 - Plugin SDK

## Purpose

The plugin SDK allows third-party developers to extend RuckChat with native Rust plugins that run in-process. Plugins can react to events, add commands, and integrate with external systems without modifying the server.

## Plugin Model

- Plugins are compiled as native dynamic libraries (`cdylib`).
- The server loads plugins from `PLUGIN_DIR` at startup.
- Each plugin exports a single entry point function that returns a descriptor and a vtable of callbacks.
- Plugins run in the same process as the server; a misbehaving plugin can crash the server, so plugins are loaded only from trusted sources.

## SDK Crate

The `plugins` crate provides:

- `Plugin` trait with lifecycle hooks.
- `Event` and `Command` types.
- Host API for database reads, logging, configuration, and emitting events.
- Macros for declaring plugin entry points.

## Lifecycle

1. **Load** — the server opens the dynamic library and calls the entry point.
2. **Initialize** — the plugin receives configuration and registers event subscriptions and commands.
3. **Run** — the plugin responds to events and commands.
4. **Shutdown** — the plugin is notified before the server exits.

## Hooks

| Hook | Trigger | Use Case |
|------|---------|----------|
| `on_message_received` | A message is created. | Filtering, moderation, external logging. |
| `on_message_sent` | A message is about to be persisted. | Validation, transformation, external routing. |
| `on_command` | A user invokes a plugin command. | Custom slash commands. |
| `on_notification` | A notification is queued. | Custom notification channels. |

## Commands

- Plugins register slash commands scoped to a channel or DM.
- Command format: `/plugin-name command-name arg1 arg2`.
- The plugin receives parsed arguments and may respond with a message or ephemeral UI.

## Configuration

- Each plugin reads configuration from `PLUGIN_DIR/plugin-name.toml`.
- Configuration is passed during initialization and can be reloaded at runtime in post-MVP.

## Host API

Plugins interact with the server through a controlled host API:

- `log(level, message)`
- `get_config() -> PluginConfig`
- `get_user(user_id) -> Option<User>`
- `get_channel(channel_id) -> Option<Channel>`
- `send_message(conversation_id, content, parent_id?)`
- `emit_event(event)`

Plugins do not execute raw SQL or access the database connection pool directly.

## Sandboxing and Security

- v1 does not sandbox plugins. Operators must trust the plugins they install.
- Plugin binaries should be signed in post-MVP releases.
- The server validates plugin API versions and rejects incompatible plugins.

## Distribution

- Plugins are distributed as precompiled binaries for supported platforms.
- A plugin manifest (`plugin.toml`) declares name, version, API version, and supported platforms.
- Community plugins are documented in the project wiki; official plugins live in the `ruckchat-plugins` repository.
