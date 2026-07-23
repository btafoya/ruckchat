//! Server-side plugin loading and management.

pub mod bus;
pub mod host;
pub mod loader;
pub mod manager;

pub use bus::CompositeEventBus;
pub use host::ServerHostApi;
pub use loader::{LoadedPlugin, PluginLoadError, load_plugin, load_plugins_in_dir};
pub use manager::{PluginManager, PluginManagerError};
