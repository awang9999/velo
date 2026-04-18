// Major and Minor Mode system
// Requirements 6.1, 6.4

use std::collections::HashMap;

/// Indentation style for a major mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentStyle {
    Spaces(usize),
    Tabs,
}

/// Keybinding map: key combination -> command name
pub type KeybindingMap = HashMap<String, String>;

/// MajorMode trait - exactly one per buffer, defines file-type-specific behavior
/// Requirement 6.1
pub trait MajorMode: Send + Sync {
    /// Returns the name of this major mode (e.g., "rust", "markdown")
    fn name(&self) -> &str;

    /// Returns file patterns this mode applies to (e.g., ["*.rs"])
    fn file_patterns(&self) -> &[&str];

    /// Returns the tree-sitter grammar for syntax highlighting, if available
    fn grammar(&self) -> Option<&tree_sitter::Language> {
        None
    }

    /// Returns the indentation style for this mode
    fn indent_style(&self) -> IndentStyle;

    /// Returns the keybinding map for this mode
    fn keybindings(&self) -> KeybindingMap {
        HashMap::new()
    }
}

/// MinorMode trait - zero or more per buffer, stackable behavior layers
/// Requirement 6.4
pub trait MinorMode: Send + Sync {
    /// Returns the name of this minor mode
    fn name(&self) -> &str;

    /// Returns the keybinding map for this mode (merged on top of major mode)
    fn keybindings(&self) -> KeybindingMap {
        HashMap::new()
    }

    /// Called when the minor mode is activated on a buffer
    fn on_activate(&mut self, buffer_id: usize, state: &mut crate::EditorState);

    /// Called when the minor mode is deactivated from a buffer
    fn on_deactivate(&mut self, buffer_id: usize, state: &mut crate::EditorState);
}

/// Plain-text fallback major mode implementation
/// Requirement 6.3
#[derive(Debug)]
pub struct PlainTextMode;

impl MajorMode for PlainTextMode {
    fn name(&self) -> &str {
        "plain-text"
    }

    fn file_patterns(&self) -> &[&str] {
        &["*.txt"]
    }

    fn grammar(&self) -> Option<&tree_sitter::Language> {
        None
    }

    fn indent_style(&self) -> IndentStyle {
        IndentStyle::Spaces(4)
    }

    fn keybindings(&self) -> KeybindingMap {
        HashMap::new()
    }
}

impl PlainTextMode {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlainTextMode {
    fn default() -> Self {
        Self::new()
    }
}

/// MajorModeRegistry - manages all known major modes and assigns modes based on file patterns
/// Requirements 6.2, 6.3, 6.8
pub struct MajorModeRegistry {
    modes: Vec<MajorModeInfo>,
}

/// Information about a registered major mode
struct MajorModeInfo {
    name: String,
    file_patterns: Vec<String>,
}

impl MajorModeRegistry {
    /// Create a new registry with the plain-text fallback mode
    pub fn new() -> Self {
        let mut registry = Self {
            modes: Vec::new(),
        };
        // Register the plain-text fallback mode
        registry.register_info("plain-text", vec!["*.txt".to_string()]);
        registry
    }

    /// Register a new major mode by extracting its information
    /// Requirement 6.8
    pub fn register(&mut self, mode: Box<dyn MajorMode>) {
        let name = mode.name().to_string();
        let patterns = mode.file_patterns().iter().map(|s| s.to_string()).collect();
        self.register_info(&name, patterns);
    }

    /// Register mode information directly
    fn register_info(&mut self, name: &str, file_patterns: Vec<String>) {
        self.modes.push(MajorModeInfo {
            name: name.to_string(),
            file_patterns,
        });
    }

    /// Get the name of the major mode that should be used for a file
    /// Returns the first mode whose file_patterns matches the file path,
    /// or "plain-text" if no pattern matches
    /// Requirements 6.2, 6.3
    pub fn mode_name_for_file(&self, file_path: &str) -> &str {
        // Try to match against registered modes (excluding plain-text fallback)
        for mode_info in &self.modes {
            if mode_info.name == "plain-text" {
                continue;
            }
            
            for pattern in &mode_info.file_patterns {
                if Self::matches_pattern(file_path, pattern) {
                    return &mode_info.name;
                }
            }
        }
        
        // Fallback to plain-text mode
        "plain-text"
    }

    /// Assign a major mode based on file extension pattern matching
    /// Returns a new instance of the appropriate mode
    /// Requirements 6.2, 6.3
    pub fn mode_for_file(&self, _file_path: &str) -> Box<dyn MajorMode> {
        // For now, always return plain-text mode
        // In a full implementation, this would use a factory pattern
        // to create instances of the appropriate mode type
        Box::new(PlainTextMode::new())
    }

    /// Check if a file path matches a glob-style pattern
    /// Supports simple patterns like "*.rs", "*.txt", etc.
    fn matches_pattern(file_path: &str, pattern: &str) -> bool {
        // Simple glob matching for patterns like "*.ext"
        if pattern.starts_with("*.") {
            let extension = &pattern[2..];
            file_path.ends_with(&format!(".{}", extension))
        } else if pattern.starts_with("*") {
            let suffix = &pattern[1..];
            file_path.ends_with(suffix)
        } else {
            // Exact match
            file_path == pattern
        }
    }

    /// Get all registered mode names
    pub fn registered_modes(&self) -> Vec<&str> {
        self.modes.iter().map(|m| m.name.as_str()).collect()
    }
}

impl Default for MajorModeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EditorState;

    // Test MajorMode trait implementation with PlainTextMode
    #[test]
    fn test_plain_text_mode_name() {
        let mode = PlainTextMode::new();
        assert_eq!(mode.name(), "plain-text");
    }

    #[test]
    fn test_plain_text_mode_file_patterns() {
        let mode = PlainTextMode::new();
        let patterns = mode.file_patterns();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0], "*.txt");
    }

    #[test]
    fn test_plain_text_mode_grammar() {
        let mode = PlainTextMode::new();
        assert!(mode.grammar().is_none());
    }

    #[test]
    fn test_plain_text_mode_indent_style() {
        let mode = PlainTextMode::new();
        assert_eq!(mode.indent_style(), IndentStyle::Spaces(4));
    }

    #[test]
    fn test_plain_text_mode_keybindings() {
        let mode = PlainTextMode::new();
        let bindings = mode.keybindings();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_plain_text_mode_default() {
        let mode = PlainTextMode::default();
        assert_eq!(mode.name(), "plain-text");
    }

    // Test IndentStyle enum
    #[test]
    fn test_indent_style_spaces() {
        let style = IndentStyle::Spaces(4);
        assert_eq!(style, IndentStyle::Spaces(4));
    }

    #[test]
    fn test_indent_style_tabs() {
        let style = IndentStyle::Tabs;
        assert_eq!(style, IndentStyle::Tabs);
    }

    // Test custom MajorMode implementation
    struct TestMajorMode;

    impl MajorMode for TestMajorMode {
        fn name(&self) -> &str {
            "test-mode"
        }

        fn file_patterns(&self) -> &[&str] {
            &["*.test"]
        }

        fn indent_style(&self) -> IndentStyle {
            IndentStyle::Tabs
        }

        fn keybindings(&self) -> KeybindingMap {
            let mut map = HashMap::new();
            map.insert("ctrl+t".to_string(), "test_command".to_string());
            map
        }
    }

    #[test]
    fn test_custom_major_mode() {
        let mode = TestMajorMode;
        assert_eq!(mode.name(), "test-mode");
        assert_eq!(mode.file_patterns(), &["*.test"]);
        assert_eq!(mode.indent_style(), IndentStyle::Tabs);
        assert!(mode.grammar().is_none());
        
        let bindings = mode.keybindings();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings.get("ctrl+t"), Some(&"test_command".to_string()));
    }

    // Test custom MinorMode implementation
    struct TestMinorMode {
        activated: bool,
    }

    impl MinorMode for TestMinorMode {
        fn name(&self) -> &str {
            "test-minor"
        }

        fn keybindings(&self) -> KeybindingMap {
            let mut map = HashMap::new();
            map.insert("ctrl+m".to_string(), "minor_command".to_string());
            map
        }

        fn on_activate(&mut self, _buffer_id: usize, _state: &mut EditorState) {
            self.activated = true;
        }

        fn on_deactivate(&mut self, _buffer_id: usize, _state: &mut EditorState) {
            self.activated = false;
        }
    }

    #[test]
    fn test_custom_minor_mode() {
        let mut mode = TestMinorMode { activated: false };
        assert_eq!(mode.name(), "test-minor");
        
        let bindings = mode.keybindings();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings.get("ctrl+m"), Some(&"minor_command".to_string()));
        
        let mut state = EditorState::new();
        
        // Test activation
        assert!(!mode.activated);
        mode.on_activate(0, &mut state);
        assert!(mode.activated);
        
        // Test deactivation
        mode.on_deactivate(0, &mut state);
        assert!(!mode.activated);
    }

    // Test that MajorMode and MinorMode are Send + Sync
    #[test]
    fn test_major_mode_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PlainTextMode>();
    }

    #[test]
    fn test_minor_mode_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        struct DummyMinor;
        impl MinorMode for DummyMinor {
            fn name(&self) -> &str { "dummy" }
            fn on_activate(&mut self, _: usize, _: &mut EditorState) {}
            fn on_deactivate(&mut self, _: usize, _: &mut EditorState) {}
        }
        assert_send_sync::<DummyMinor>();
    }

    // Test MajorModeRegistry
    #[test]
    fn test_major_mode_registry_new() {
        let registry = MajorModeRegistry::new();
        // Registry should have at least the plain-text fallback mode
        let modes = registry.registered_modes();
        assert!(modes.contains(&"plain-text"));
    }

    #[test]
    fn test_major_mode_registry_default() {
        let registry = MajorModeRegistry::default();
        let modes = registry.registered_modes();
        assert!(modes.contains(&"plain-text"));
    }

    #[test]
    fn test_major_mode_registry_register() {
        let mut registry = MajorModeRegistry::new();
        let initial_count = registry.registered_modes().len();
        
        registry.register(Box::new(TestMajorMode));
        
        assert_eq!(registry.registered_modes().len(), initial_count + 1);
        assert!(registry.registered_modes().contains(&"test-mode"));
    }

    #[test]
    fn test_major_mode_registry_mode_name_for_file_fallback() {
        let registry = MajorModeRegistry::new();
        
        // Unknown file extension should return plain-text mode
        let mode_name = registry.mode_name_for_file("unknown.xyz");
        assert_eq!(mode_name, "plain-text");
    }

    #[test]
    fn test_major_mode_registry_mode_name_for_file_txt() {
        let registry = MajorModeRegistry::new();
        
        // .txt files should match plain-text mode
        let mode_name = registry.mode_name_for_file("readme.txt");
        assert_eq!(mode_name, "plain-text");
    }

    #[test]
    fn test_major_mode_registry_mode_name_for_file_custom() {
        let mut registry = MajorModeRegistry::new();
        registry.register(Box::new(TestMajorMode));
        
        // .test files should match test-mode
        let mode_name = registry.mode_name_for_file("example.test");
        assert_eq!(mode_name, "test-mode");
    }

    #[test]
    fn test_major_mode_registry_mode_for_file_fallback() {
        let registry = MajorModeRegistry::new();
        
        // Unknown file extension should return plain-text mode
        let mode = registry.mode_for_file("unknown.xyz");
        assert_eq!(mode.name(), "plain-text");
    }

    #[test]
    fn test_major_mode_registry_mode_for_file_txt() {
        let registry = MajorModeRegistry::new();
        
        // .txt files should match plain-text mode
        let mode = registry.mode_for_file("readme.txt");
        assert_eq!(mode.name(), "plain-text");
    }

    #[test]
    fn test_matches_pattern_wildcard_extension() {
        assert!(MajorModeRegistry::matches_pattern("test.rs", "*.rs"));
        assert!(MajorModeRegistry::matches_pattern("main.rs", "*.rs"));
        assert!(!MajorModeRegistry::matches_pattern("test.txt", "*.rs"));
    }

    #[test]
    fn test_matches_pattern_wildcard_suffix() {
        assert!(MajorModeRegistry::matches_pattern("Makefile", "*file"));
        assert!(MajorModeRegistry::matches_pattern("Dockerfile", "*file"));
        assert!(!MajorModeRegistry::matches_pattern("test.rs", "*file"));
    }

    #[test]
    fn test_matches_pattern_exact() {
        assert!(MajorModeRegistry::matches_pattern("Makefile", "Makefile"));
        assert!(!MajorModeRegistry::matches_pattern("makefile", "Makefile"));
    }

    #[test]
    fn test_matches_pattern_with_path() {
        assert!(MajorModeRegistry::matches_pattern("src/main.rs", "*.rs"));
        assert!(MajorModeRegistry::matches_pattern("/home/user/test.txt", "*.txt"));
    }
}
