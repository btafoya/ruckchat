//! Dynamic library loader for native plugins.

use libloading::{Library, Symbol};
use ruckchat_plugin_sdk::{API_VERSION, PluginExports, PluginInfo, PluginInstance};
use std::path::Path;
use tracing::{error, info};

/// A plugin loaded from a dynamic library.
///
/// The instance pointer and destroy function are owned here; dropping this
/// value shuts the plugin down and frees the plugin-allocated memory using the
/// plugin's own destroy function.
pub struct LoadedPlugin {
    info: PluginInfo,
    instance: *mut PluginInstance,
    destroy: unsafe extern "C" fn(*mut PluginInstance),
    /// Loaded library handle. `None` in unit tests that bypass dynamic
    /// loading.
    _library: Option<Library>,
}

// The plugin instance is allocated by the plugin and protected by a mutex in
// the manager, so it is safe to move the handle across threads.
unsafe impl Send for LoadedPlugin {}

impl LoadedPlugin {
    /// Returns plugin metadata.
    #[must_use]
    pub fn info(&self) -> &PluginInfo {
        &self.info
    }

    /// Returns an immutable reference to the plugin instance.
    ///
    /// # Safety
    ///
    /// Callers must ensure exclusive access; the manager coordinates this
    /// through a mutex.
    #[must_use]
    pub unsafe fn instance(&self) -> &PluginInstance {
        // SAFETY: the manager holds this handle behind a mutex and only calls
        // this method while the mutex is locked.
        unsafe { &*self.instance }
    }

    /// Returns a mutable reference to the plugin instance.
    ///
    /// # Safety
    ///
    /// Callers must ensure exclusive access; the manager coordinates this
    /// through a mutex.
    #[must_use]
    pub unsafe fn instance_mut(&mut self) -> &mut PluginInstance {
        // SAFETY: the manager holds this handle behind a mutex and only calls
        // this method while the mutex is locked.
        unsafe { &mut *self.instance }
    }
}

impl Drop for LoadedPlugin {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: the instance pointer came from the plugin's create
            // function and has not been freed yet.
            (*self.instance).shutdown();
            (self.destroy)(self.instance);
        }
    }
}

/// Error loading a plugin.
#[derive(Debug, thiserror::Error)]
pub enum PluginLoadError {
    /// Failed to open the dynamic library.
    #[error("failed to open plugin library: {0}")]
    LibraryOpen(String),
    /// The entry symbol is missing.
    #[error("plugin entry symbol missing: {0}")]
    MissingEntry(String),
    /// The plugin API version is incompatible.
    #[error("incompatible plugin API version: expected {expected}, got {actual}")]
    IncompatibleApiVersion {
        /// Expected API version.
        expected: u32,
        /// Actual API version reported by the plugin.
        actual: u32,
    },
    /// The plugin create function returned null.
    #[error("plugin create returned a null instance")]
    NullInstance,
}

impl From<libloading::Error> for PluginLoadError {
    fn from(err: libloading::Error) -> Self {
        Self::LibraryOpen(err.to_string())
    }
}

/// Loads a single plugin dynamic library.
///
/// # Safety
///
/// Loading dynamic libraries is unsafe because the loaded code runs in the
/// server process. Only load plugins from trusted sources.
pub unsafe fn load_plugin(path: &Path) -> Result<LoadedPlugin, PluginLoadError> {
    // SAFETY: caller guarantees the library is trusted.
    let library = unsafe { Library::new(path)? };
    let entry: Symbol<'_, extern "C" fn() -> *const PluginExports> =
        // SAFETY: the symbol name is the contract exposed by the plugin SDK.
        unsafe { library.get(b"ruckchat_plugin_entry\0")? };

    let exports = entry();
    if exports.is_null() {
        return Err(PluginLoadError::MissingEntry(
            "ruckchat_plugin_entry returned null".into(),
        ));
    }

    // SAFETY: exports is a valid pointer returned by the plugin entry.
    let exports = unsafe { &*exports };
    if exports.api_version != API_VERSION {
        return Err(PluginLoadError::IncompatibleApiVersion {
            expected: API_VERSION,
            actual: exports.api_version,
        });
    }

    // SAFETY: create is the function pointer supplied by the plugin entry.
    let instance = unsafe { (exports.create)() };
    if instance.is_null() {
        return Err(PluginLoadError::NullInstance);
    }

    // SAFETY: instance was just created and is non-null.
    let info = unsafe { (*instance).info() };

    info!(
        plugin = %info.name,
        version = %info.version,
        path = %path.display(),
        "loaded plugin"
    );

    Ok(LoadedPlugin {
        info,
        instance,
        destroy: exports.destroy,
        _library: Some(library),
    })
}

/// Loads every plugin dynamic library found in `dir`.
///
/// # Safety
///
/// Loading dynamic libraries is unsafe because the loaded code runs in the
/// server process. Only load plugins from trusted sources.
pub unsafe fn load_plugins_in_dir(dir: &Path) -> Vec<(PluginInfo, LoadedPlugin)> {
    let mut loaded = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return loaded;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !matches!(ext, "so" | "dll" | "dylib") {
            continue;
        }

        // SAFETY: the whole directory load is covered by the caller's safety
        // contract; individual files are validated by load_plugin.
        match unsafe { load_plugin(&path) } {
            Ok(plugin) => {
                let info = plugin.info().clone();
                loaded.push((info, plugin));
            }
            Err(err) => {
                error!(path = %path.display(), %err, "failed to load plugin");
            }
        }
    }

    loaded
}

#[cfg(test)]
impl LoadedPlugin {
    /// Creates a loaded-plugin handle from a boxed plugin implementation.
    ///
    /// This bypasses dynamic loading and is intended for unit tests.
    pub fn from_boxed_plugin(plugin: Box<dyn ruckchat_plugin_sdk::Plugin>) -> Self {
        use ruckchat_plugin_sdk::PluginInstance;

        extern "C" fn test_destroy(instance: *mut PluginInstance) {
            if !instance.is_null() {
                // SAFETY: this pointer came from the matching Box::into_raw in
                // this test helper.
                unsafe {
                    let _ = Box::from_raw(instance);
                }
            }
        }

        let info = plugin.info();
        let instance = Box::into_raw(Box::new(PluginInstance::new(plugin)));
        Self {
            info,
            instance,
            destroy: test_destroy,
            _library: None,
        }
    }
}
