//! Plugin system for Velo.
//!
//! Exposes the public API (`Plugin`, `PluginRegistry`, `PluginManager`)
//! and re‑exports the submodules.

pub mod manager;
pub use manager::PLUGIN_DIR;
pub mod plugin;
pub mod registry;

pub use manager::PluginManager;
pub use plugin::{Plugin, PluginLifecycle};
pub use registry::PluginRegistry;
