use std::any::Any;
use velo_types::EditorEvent;

/// The core trait that every plugin must implement.
///
/// The trait is intentionally object‑safe so that plugins can be stored in a
/// `Vec<Box<dyn Plugin>>`.
///
/// The `on_event` method receives an `EditorEvent` and a mutable reference to
/// the editor state.  The state is passed as `&mut dyn Any` to avoid a
/// dependency cycle between the core and the plugin crate.
///
/// In a real implementation the state would be a concrete type and the
/// trait would be generic.  For this exercise the simplified version is
/// sufficient.
pub trait Plugin: PluginLifecycle + Send + Sync {
    /// Name of the plugin.
    fn name(&self) -> &str;

    /// Called for every event emitted by the editor.
    fn on_event(&mut self, event: &EditorEvent, state: &mut dyn Any);
}

/// Optional lifecycle hooks.
/// Plugins may implement these if they need to run code on load/unload.
pub trait PluginLifecycle {
    /// Called once when the plugin is loaded.
    fn on_load(&mut self, _state: &mut dyn Any) {}

    /// Called once when the plugin is unloaded.
    fn on_unload(&mut self, _state: &mut dyn Any) {}
}
