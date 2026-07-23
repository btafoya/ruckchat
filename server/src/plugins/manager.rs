//! Plugin manager lifecycle and dispatch.

use crate::plugins::host::ServerHostApi;
use crate::plugins::loader::{LoadedPlugin, load_plugins_in_dir};
use crate::services::events::EventBus;
use ruckchat_domain::{ChannelRepository, MessageRepository, UserRepository};
use ruckchat_plugin_sdk::{CommandResponse, HostApi, PluginCommand, PluginEvent, PluginInfo};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::{error, warn};

/// Dependencies required to build a [`PluginManager`].
#[derive(Clone)]
pub struct PluginManagerDeps {
    /// User repository for the host API.
    pub users: Arc<dyn UserRepository + Send + Sync>,
    /// Channel repository for the host API.
    pub channels: Arc<dyn ChannelRepository + Send + Sync>,
    /// Message repository for the host API.
    pub messages: Arc<dyn MessageRepository + Send + Sync>,
    /// Event bus the host API publishes to.
    pub events: Arc<dyn EventBus + Send + Sync>,
}

/// A loaded plugin with its host handle.
struct ManagedPlugin {
    /// Plugin metadata.
    info: PluginInfo,
    /// Loaded library and instance.
    instance: Mutex<LoadedPlugin>,
    /// Host API handle passed to hooks.
    host: Arc<dyn HostApi + Send + Sync>,
}

/// Manages loaded plugins and dispatches events and commands to them.
pub struct PluginManager {
    plugins: Vec<ManagedPlugin>,
}

/// Error from plugin manager operations.
#[derive(Debug, thiserror::Error)]
pub enum PluginManagerError {
    /// Plugin with the given name is not loaded.
    #[error("plugin not found: {0}")]
    NotFound(String),
    /// Failed to read the plugin directory.
    #[error("failed to read plugin directory: {0}")]
    Io(#[from] std::io::Error),
}

impl PluginManager {
    /// Loads all plugin dynamic libraries found in `dir` and initializes them.
    ///
    /// # Safety
    ///
    /// This loads native dynamic libraries into the server process. Only call
    /// with a directory containing trusted plugin binaries.
    pub fn load_from_dir(dir: &Path, deps: PluginManagerDeps) -> Result<Self, PluginManagerError> {
        let loaded = unsafe { load_plugins_in_dir(dir) };
        let mut plugins = Vec::with_capacity(loaded.len());

        for (info, mut loaded) in loaded {
            let config = load_plugin_config(dir, &info.name);
            unsafe { loaded.instance_mut() }.initialize(config.clone());

            let host: Arc<dyn HostApi + Send + Sync> = Arc::new(ServerHostApi::new(
                config,
                deps.users.clone(),
                deps.channels.clone(),
                deps.messages.clone(),
                deps.events.clone(),
            ));

            plugins.push(ManagedPlugin {
                info,
                instance: Mutex::new(loaded),
                host,
            });
        }

        Ok(Self { plugins })
    }

    /// Creates an empty manager with no plugins.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Returns metadata for all loaded plugins.
    #[must_use]
    pub fn list_plugins(&self) -> Vec<&PluginInfo> {
        self.plugins.iter().map(|p| &p.info).collect()
    }

    /// Dispatches an event to every loaded plugin.
    pub fn dispatch_event(&self, event: PluginEvent) {
        for plugin in &self.plugins {
            let Ok(mut guard) = plugin.instance.lock() else {
                warn!(plugin = %plugin.info.name, "plugin instance mutex poisoned");
                continue;
            };
            unsafe { guard.instance_mut() }.on_event(&*plugin.host, event.clone());
        }
    }

    /// Dispatches a command to the plugin named in `command.plugin`.
    ///
    /// # Errors
    ///
    /// Returns [`PluginManagerError::NotFound`] when the plugin is not loaded.
    pub fn dispatch_command(
        &self,
        command: PluginCommand,
    ) -> Result<CommandResponse, PluginManagerError> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.info.name == command.plugin)
            .ok_or_else(|| PluginManagerError::NotFound(command.plugin.clone()))?;

        let Ok(mut guard) = plugin.instance.lock() else {
            error!(plugin = %plugin.info.name, "plugin instance mutex poisoned");
            return Ok(CommandResponse::Error {
                message: "plugin is unavailable".into(),
            });
        };

        let response = unsafe { guard.instance_mut() }.on_command(&*plugin.host, command);
        Ok(response)
    }
}

/// Loads a plugin's TOML configuration file if it exists.
fn load_plugin_config(dir: &Path, name: &str) -> serde_json::Value {
    let path = dir.join(format!("{name}.toml"));
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => return serde_json::Value::Object(Default::default()),
    };

    toml::from_str(&content)
        .inspect_err(|err| {
            warn!(path = %path.display(), %err, "failed to parse plugin config; using empty config");
        })
        .unwrap_or_else(|_| serde_json::Value::Object(Default::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruckchat_domain::{ConversationType, Message};
    use ruckchat_id::UserId;
    use ruckchat_plugin_sdk::{
        Channel, ChannelId, CommandResponse, HostApi, LogLevel, Plugin, PluginCommand, PluginEvent,
        PluginInfo, SendMessageRequest, User,
    };
    use serde_json::json;
    use std::sync::Arc;
    use uuid::Uuid;

    struct NullHost;

    impl HostApi for NullHost {
        fn log(&self, _level: LogLevel, _message: &str) {}

        fn get_config(&self) -> serde_json::Value {
            json!({})
        }

        fn get_user(&self, _user_id: UserId) -> Result<Option<User>, String> {
            Ok(None)
        }

        fn get_channel(&self, _channel_id: ChannelId) -> Result<Option<Channel>, String> {
            Ok(None)
        }

        fn send_message(&self, _request: SendMessageRequest) -> Result<Message, String> {
            unimplemented!()
        }

        fn emit_event(&self, _event: PluginEvent) -> Result<(), String> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct EchoPlugin {
        events: Arc<std::sync::Mutex<Vec<PluginEvent>>>,
    }

    impl Plugin for EchoPlugin {
        fn info(&self) -> PluginInfo {
            PluginInfo::new("echo", "1.0.0")
        }

        fn initialize(&mut self, _config: serde_json::Value) {}

        fn on_event(&mut self, _host: &dyn HostApi, event: PluginEvent) {
            self.events.lock().unwrap().push(event);
        }

        fn on_command(&mut self, _host: &dyn HostApi, command: PluginCommand) -> CommandResponse {
            CommandResponse::Ephemeral {
                content: format!("echo {}", command.args.join(" ")),
            }
        }

        fn shutdown(&mut self) {}
    }

    fn manager_with_plugin(plugin: Box<dyn ruckchat_plugin_sdk::Plugin>) -> PluginManager {
        let info = plugin.info();
        let loaded = LoadedPlugin::from_boxed_plugin(plugin);
        let host: Arc<dyn HostApi + Send + Sync> = Arc::new(NullHost);
        PluginManager {
            plugins: vec![ManagedPlugin {
                info,
                instance: Mutex::new(loaded),
                host,
            }],
        }
    }

    #[test]
    fn dispatch_event_delivers_to_plugin() {
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let plugin = Box::new(EchoPlugin {
            events: events.clone(),
        });
        let manager = manager_with_plugin(plugin);

        let message = Message::new(
            Uuid::new_v4(),
            ConversationType::Channel,
            UserId::new(),
            "hi",
            None,
        )
        .unwrap();
        manager.dispatch_event(PluginEvent::MessageReceived {
            message: message.clone(),
        });

        let received = events.lock().unwrap();
        assert_eq!(received.len(), 1);
        assert!(matches!(
            &received[0],
            PluginEvent::MessageReceived { message: m } if m.id == message.id
        ));
    }

    #[test]
    fn dispatch_command_returns_plugin_response() {
        let manager = manager_with_plugin(Box::new(EchoPlugin::default()));
        let response = manager
            .dispatch_command(PluginCommand {
                plugin: "echo".into(),
                command: "say".into(),
                args: vec!["hello".into(), "world".into()],
                conversation_id: Uuid::new_v4(),
                conversation_type: ConversationType::Channel,
                user_id: UserId::new(),
            })
            .unwrap();

        assert!(
            matches!(response, CommandResponse::Ephemeral { content } if content == "echo hello world")
        );
    }

    #[test]
    fn dispatch_command_returns_error_for_missing_plugin() {
        let manager = PluginManager::empty();
        let err = manager
            .dispatch_command(PluginCommand {
                plugin: "missing".into(),
                command: "x".into(),
                args: vec![],
                conversation_id: Uuid::new_v4(),
                conversation_type: ConversationType::Channel,
                user_id: UserId::new(),
            })
            .unwrap_err();

        assert!(matches!(err, PluginManagerError::NotFound(name) if name == "missing"));
    }
}
