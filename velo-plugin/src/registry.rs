use std::any::Any;
use std::panic::AssertUnwindSafe;

use crate::plugin::Plugin;
use velo_types::EditorEvent;

/// Registry of loaded plugins.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// Create a new, empty registry.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a new plugin instance.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Dispatch an event to all registered plugins.
    /// Panics inside a plugin are caught and ignored so the editor continues running.
    pub fn dispatch(&mut self, event: &EditorEvent, state: &mut dyn Any) {
        for plugin in &mut self.plugins {
            let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
                plugin.on_event(event, state);
            }));
        }
    }
    /// Call on_unload on all plugins, passing state.
    pub fn unload_all(&mut self, state: &mut dyn Any) {
        for plugin in &mut self.plugins {
            plugin.on_unload(state);
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
