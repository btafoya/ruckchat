//! Plugin SDK for native RuckChat plugins.
//!
//! Plugins are native Rust dynamic libraries (`cdylib`) that implement the
//! [`Plugin`] trait and export an entry point via the [`declare_plugin!`]
//! macro. They run in-process and are loaded by the server at startup from a
//! configured plugin directory.
//!
//! # Example
//!
//! ```rust,ignore
//! use ruckchat_plugin_sdk::{Plugin, PluginInfo, declare_plugin};
//!
//! #[derive(Default)]
//! struct HelloPlugin;
//!
//! impl Plugin for HelloPlugin {
//!     fn info(&self) -> PluginInfo {
//!         PluginInfo::new("hello", "1.0.0")
//!     }
//!
//!     fn on_command(
//!         &mut self,
//!         _host: &dyn HostApi,
//!         _command: PluginCommand,
//!     ) -> CommandResponse {
//!         CommandResponse::Ephemeral {
//!             content: "Hello from the plugin!".into(),
//!         }
//!     }
//!
//!     // ... other required methods
//! }
//!
//! declare_plugin!(HelloPlugin);
//! ```

pub use ruckchat_domain::{Channel, ConversationType, Message, User};
pub use ruckchat_id::{ChannelId, MessageId, UserId};

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Current plugin API version.
///
/// Plugins must return this exact value or the server will refuse to load
/// them. Bumping this number signals a breaking change in the plugin
/// interface.
pub const API_VERSION: u32 = 1;

/// Metadata describing a plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Human-readable plugin name.
    pub name: String,
    /// Plugin version, e.g. `1.0.0`.
    pub version: String,
    /// SDK API version the plugin targets.
    pub api_version: u32,
}

impl PluginInfo {
    /// Creates plugin info, setting [`API_VERSION`] automatically.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            api_version: API_VERSION,
        }
    }
}

/// Log level for host logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    /// Debug-level log.
    Debug,
    /// Informational log.
    Info,
    /// Warning log.
    Warn,
    /// Error log.
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Debug => write!(f, "debug"),
            Self::Info => write!(f, "info"),
            Self::Warn => write!(f, "warn"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Events delivered to plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginEvent {
    /// A message was created in a conversation.
    MessageReceived {
        /// The new message.
        message: Message,
    },
    /// A message was updated.
    MessageUpdated {
        /// The updated message.
        message: Message,
    },
    /// A message was deleted.
    MessageDeleted {
        /// The deleted message.
        message: Message,
    },
    /// A notification is queued for a user.
    Notification {
        /// Target user.
        user_id: UserId,
        /// Notification title.
        title: String,
        /// Notification body.
        body: String,
    },
}

/// A slash-command invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    /// Plugin name.
    pub plugin: String,
    /// Command name.
    pub command: String,
    /// Positional arguments.
    pub args: Vec<String>,
    /// Conversation the command was invoked in.
    pub conversation_id: Uuid,
    /// Conversation kind.
    pub conversation_type: ConversationType,
    /// User that invoked the command.
    pub user_id: UserId,
}

/// Response from a plugin command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommandResponse {
    /// Post a message to the conversation.
    Message {
        /// Message content.
        content: String,
        /// Optional parent message for a thread reply.
        parent_id: Option<MessageId>,
    },
    /// Show an ephemeral message only to the caller.
    Ephemeral {
        /// Ephemeral content.
        content: String,
    },
    /// Return an error to the caller.
    Error {
        /// Error message.
        message: String,
    },
}

/// Request to send a message through the host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    /// Author of the message, often the invoking user or a plugin user.
    pub author_id: UserId,
    /// Target conversation.
    pub conversation_id: Uuid,
    /// Conversation kind.
    pub conversation_type: ConversationType,
    /// Message content.
    pub content: String,
    /// Optional parent message for a thread reply.
    pub parent_id: Option<MessageId>,
}

/// Host-provided API for plugins.
///
/// The server implements this trait and passes a reference to plugin hooks.
/// Calls are synchronous on the server's async runtime; the host
/// implementation blocks on the underlying async services.
pub trait HostApi: Send + Sync {
    /// Logs a message through the server's logging infrastructure.
    fn log(&self, level: LogLevel, message: &str);

    /// Returns the plugin's configuration as a JSON value.
    fn get_config(&self) -> serde_json::Value;

    /// Loads a user by id.
    fn get_user(&self, user_id: UserId) -> Result<Option<User>, String>;

    /// Loads a channel by id.
    fn get_channel(&self, channel_id: ChannelId) -> Result<Option<Channel>, String>;

    /// Sends a message to a conversation as the plugin.
    fn send_message(&self, request: SendMessageRequest) -> Result<Message, String>;

    /// Emits a plugin event back into the host.
    fn emit_event(&self, event: PluginEvent) -> Result<(), String>;
}

/// Trait implemented by every RuckChat plugin.
pub trait Plugin: Send + Sync {
    /// Returns plugin metadata.
    fn info(&self) -> PluginInfo;

    /// Called once after the plugin is loaded with its configuration.
    fn initialize(&mut self, config: serde_json::Value);

    /// Called when a server event the plugin subscribed to occurs.
    fn on_event(&mut self, host: &dyn HostApi, event: PluginEvent);

    /// Called when a user invokes a plugin command.
    fn on_command(&mut self, host: &dyn HostApi, command: PluginCommand) -> CommandResponse;

    /// Called before the plugin is unloaded.
    fn shutdown(&mut self);
}

/// Opaque container for a plugin instance returned by a dynamic library.
///
/// The server loads a pointer to this type and calls its safe methods. The
/// actual plugin implementation lives inside the dynamic library.
pub struct PluginInstance {
    plugin: Box<dyn Plugin>,
}

impl PluginInstance {
    /// Creates a plugin instance from a boxed plugin implementation.
    #[must_use]
    pub fn new(plugin: Box<dyn Plugin>) -> Self {
        Self { plugin }
    }

    /// Returns plugin metadata.
    #[must_use]
    pub fn info(&self) -> PluginInfo {
        self.plugin.info()
    }

    /// Initializes the plugin with its configuration.
    pub fn initialize(&mut self, config: serde_json::Value) {
        self.plugin.initialize(config);
    }

    /// Dispatches an event to the plugin.
    pub fn on_event(&mut self, host: &dyn HostApi, event: PluginEvent) {
        self.plugin.on_event(host, event);
    }

    /// Dispatches a command to the plugin.
    #[must_use]
    pub fn on_command(&mut self, host: &dyn HostApi, command: PluginCommand) -> CommandResponse {
        self.plugin.on_command(host, command)
    }

    /// Shuts the plugin down.
    pub fn shutdown(&mut self) {
        self.plugin.shutdown();
    }
}

/// Raw exports from a plugin dynamic library.
///
/// Plugins use [`declare_plugin!`] to generate this structure and the
/// `ruckchat_plugin_entry` symbol.
#[repr(C)]
pub struct PluginExports {
    /// SDK API version the plugin targets.
    pub api_version: u32,
    /// Creates a plugin instance.
    pub create: unsafe extern "C" fn() -> *mut PluginInstance,
    /// Destroys a plugin instance previously created by `create`.
    pub destroy: unsafe extern "C" fn(*mut PluginInstance),
}

/// Declares the plugin entry point for a `cdylib` crate.
///
/// The macro generates the `ruckchat_plugin_entry` symbol that the server
/// loader looks up, plus the `create` and `destroy` functions it references.
/// Only one invocation per dynamic library is supported.
///
/// The supplied type must implement [`Default`] and [`Plugin`].
///
/// # Example
///
/// ```rust,ignore
/// use ruckchat_plugin_sdk::{Plugin, PluginInfo, declare_plugin};
///
/// #[derive(Default)]
/// struct HelloPlugin;
///
/// impl Plugin for HelloPlugin {
///     fn info(&self) -> PluginInfo { PluginInfo::new("hello", "1.0.0") }
///     // ...
/// }
///
/// declare_plugin!(HelloPlugin);
/// ```
#[macro_export]
macro_rules! declare_plugin {
    ($plugin:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn ruckchat_plugin_entry() -> *const $crate::PluginExports {
            static EXPORTS: $crate::PluginExports = $crate::PluginExports {
                api_version: $crate::API_VERSION,
                create: __ruckchat_plugin_create,
                destroy: __ruckchat_plugin_destroy,
            };
            &EXPORTS
        }

        unsafe extern "C" fn __ruckchat_plugin_create() -> *mut $crate::PluginInstance {
            let plugin: Box<dyn $crate::Plugin> = Box::new(<$plugin>::default());
            let instance = $crate::PluginInstance::new(plugin);
            Box::into_raw(Box::new(instance))
        }

        unsafe extern "C" fn __ruckchat_plugin_destroy(instance: *mut $crate::PluginInstance) {
            if !instance.is_null() {
                // SAFETY: `instance` is a non-null pointer returned by the matching
                // `Box::into_raw` in `__ruckchat_plugin_create`.
                unsafe {
                    let _ = Box::from_raw(instance);
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Default)]
    struct TestPlugin {
        initialized: bool,
    }

    impl Plugin for TestPlugin {
        fn info(&self) -> PluginInfo {
            PluginInfo::new("test", "0.1.0")
        }

        fn initialize(&mut self, _config: serde_json::Value) {
            self.initialized = true;
        }

        fn on_event(&mut self, _host: &dyn HostApi, _event: PluginEvent) {}

        fn on_command(&mut self, _host: &dyn HostApi, _command: PluginCommand) -> CommandResponse {
            CommandResponse::Ephemeral {
                content: "ok".into(),
            }
        }

        fn shutdown(&mut self) {}
    }

    declare_plugin!(TestPlugin);

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

    #[test]
    fn plugin_entry_exports_current_api_version() {
        unsafe {
            let exports = &*ruckchat_plugin_entry();
            assert_eq!(exports.api_version, API_VERSION);

            let instance = (exports.create)();
            assert!(!instance.is_null());

            let info = (*instance).info();
            assert_eq!(info.name, "test");
            assert_eq!(info.version, "0.1.0");
            assert_eq!(info.api_version, API_VERSION);

            (exports.destroy)(instance);
        }
    }

    #[test]
    fn plugin_instance_round_trip() {
        unsafe {
            let exports = &*ruckchat_plugin_entry();
            let instance = (exports.create)();

            let mut instance = Box::from_raw(instance);
            instance.initialize(json!({"key": "value"}));

            let host = NullHost;
            let response = instance.on_command(
                &host,
                PluginCommand {
                    plugin: "test".into(),
                    command: "ping".into(),
                    args: vec![],
                    conversation_id: Uuid::new_v4(),
                    conversation_type: ConversationType::Channel,
                    user_id: UserId::new(),
                },
            );
            assert!(matches!(response, CommandResponse::Ephemeral { content } if content == "ok"));
            instance.shutdown();
        }
    }
}
