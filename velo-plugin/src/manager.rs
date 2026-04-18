use libloading::Library;
use std::error::Error;
use std::path::PathBuf;

/// Default directory where plugins are stored.
pub const PLUGIN_DIR: &str = "~/.config/velo/plugins/";

/// Simple plugin manager – loads plugins from a known directory.
/// The implementation is intentionally lightweight; it can be expanded later.
pub struct PluginManager {
    /// Directory where plugins are stored (e.g. `~/.config/velo/plugins/`).
    plugin_dir: PathBuf,
}

impl PluginManager {
    /// Create a new manager pointing at `plugin_dir`.
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self { plugin_dir }
    }

    /// Load all plugins from the directory and return a registry.
    pub fn load_plugins(&self) -> Result<crate::registry::PluginRegistry, Box<dyn Error>> {
        let mut registry = crate::registry::PluginRegistry::new();

        // Very naïve implementation: load plugins with extensions .so, .dll, .dylib.
        for entry in std::fs::read_dir(&self.plugin_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ["so", "dll", "dylib"].contains(&ext) {
                    unsafe {
                        let lib = Library::new(&path)?;
                        // Load plugin init symbol
                        let func_res: Result<
                            libloading::Symbol<
                                unsafe extern "C" fn() -> *mut dyn crate::plugin::Plugin,
                            >,
                            _,
                        > = lib.get(b"velo_plugin_init");
                        let func = match func_res {
                            Ok(f) => f,
                            Err(e) => {
                                eprintln!(
                                    "Failed to load symbol velo_plugin_init from {:?}: {}",
                                    path, e
                                );
                                continue;
                            }
                        };
                        let boxed_raw = func();
                        let mut boxed: Box<dyn crate::plugin::Plugin> = Box::from_raw(boxed_raw);
                        // Call on_load with dummy state
                        boxed.on_load(&mut () as &mut dyn std::any::Any);
                        registry.register(boxed);
                    }
                }
            }
        }

        Ok(registry)
    }
}
