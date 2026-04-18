# Implementation Plan: velo

## Overview

This plan implements Velo, a high-performance post-modern text editor written in Rust. The implementation follows a strict layered architecture across six crates, with a Tokio-based async runtime, dynamic native plugin system, Emacs-inspired major/minor mode system, tree-sitter syntax highlighting, and a two-layer configuration system (TOML + optional Rust config crate).

## Tasks

- [x] 1. Set up Cargo workspace and foundation types
  - [x] 1.1 Create Cargo workspace with six crates
    - Create workspace `Cargo.toml` with members: `velo-types`, `velo-core`, `velo-plugin`, `velo-app`, `velo-tui`, `velo-gui`
    - Create directory structure for all six crates
    - Configure dependency graph: `velo-tui`/`velo-gui` → `velo-app` → `velo-core`/`velo-plugin` → `velo-types`
    - _Requirements: 1.1, 1.3_
  
  - [x] 1.2 Implement foundation types in velo-types
    - Create `velo-types/src/lib.rs` with zero external dependencies
    - Implement `Position { line: usize, column: usize }`
    - Implement `Range { start: Position, end: Position }` with validation constructor
    - Implement `Selection { anchor: Position, head: Position }`
    - Implement `VeloError` enum with variants for all error cases
    - Implement `EditorEvent` enum with all variants (BufferOpened, BufferClosed, BufferModified, BufferSaved, CursorMoved, SelectionChanged, VeloStarted, VeloShutdown, KeyPressed)
    - _Requirements: 1.2, 1.6, 4.1, 4.2, 4.3, 12.1_
  
  - [ ]* 1.3 Write property test for Range validity invariant
    - **Property 1: Range Validity Invariant**
    - **Validates: Requirements 4.4, 4.5**
    - Generate arbitrary Position values and verify Range constructor enforces `start <= end` invariant
    - Test that invalid ranges return VeloError

- [x] 2. Implement Buffer and EditorState in velo-core
  - [x] 2.1 Set up velo-core crate dependencies
    - Add dependencies: `ropey`, `tree-sitter`, `tokio` (with features: `sync`, `rt-multi-thread`), `serde`, `toml`
    - Add dependency on `velo-types`
    - _Requirements: 1.4, 2.1_
  
  - [x] 2.2 Implement Buffer structure
    - Create `Buffer` struct with fields: `rope: ropey::Rope`, `file_path: Option<PathBuf>`, `is_dirty: bool`, `cursor: Position`, `selections: Vec<Selection>`, `major_mode: Box<dyn MajorMode>`, `minor_modes: Vec<Box<dyn MinorMode>>`, `syntax_tree: Option<tree_sitter::Tree>`
    - Implement buffer modification methods that set `is_dirty = true`
    - Implement buffer save method that sets `is_dirty = false`
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 2.10_
  
  - [x] 2.3 Implement MajorMode and MinorMode traits
    - Define `MajorMode` trait with methods: `name()`, `file_patterns()`, `grammar()`, `indent_style()`, `keybindings()`
    - Define `MinorMode` trait with methods: `name()`, `keybindings()`, `on_activate()`, `on_deactivate()`
    - Create plain-text fallback major mode implementation
    - _Requirements: 6.1, 6.4_
  
  - [x] 2.4 Implement MajorModeRegistry
    - Create `MajorModeRegistry` struct with registration and file-type detection logic
    - Implement `register()` method for adding new major modes
    - Implement file extension pattern matching for mode assignment
    - _Requirements: 6.2, 6.3, 6.8_
  
  - [ ]* 2.5 Write property test for file-type major mode assignment
    - **Property 4: File-Type Major Mode Assignment**
    - **Validates: Requirements 6.2, 6.3**
    - Generate arbitrary file paths and registered mode patterns
    - Verify correct mode is assigned or fallback is used
  
  - [x] 2.6 Implement EditorState structure
    - Create `EditorState` struct with fields: `buffers: Vec<Buffer>`, `active_buffer_id: usize`, `config: VeloConfig`, `major_mode_registry: MajorModeRegistry`, `plugin_registry: PluginRegistry`
    - Implement buffer open/close operations that maintain valid `active_buffer_id`
    - Implement safe active buffer access with error handling
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_
  
  - [ ]* 2.7 Write property test for active_buffer_id validity
    - **Property 12: active_buffer_id Validity**
    - **Validates: Requirements 3.6, 3.7**
    - Generate arbitrary sequences of buffer open/close operations
    - Verify `active_buffer_id` is always valid when buffers exist

- [x] 3. Implement Command system
  - [x] 3.1 Define Command trait
    - Create `Command` trait with methods: `execute(&self, state: &mut EditorState)`, `undo(&self, state: &mut EditorState)`, `name(&self) -> &str`
    - _Requirements: 5.1, 5.2, 5.3_
  
  - [x] 3.2 Implement built-in commands
    - Implement `InsertChar` command with execute and undo
    - Implement `DeleteChar` command with execute and undo
    - Implement `MoveCursor` command with execute and undo
    - Implement `SaveFile` command
    - Implement `OpenFile` command
    - Ensure all EditorState mutations go through Command::execute
    - _Requirements: 5.4, 5.6_
  
  - [ ]* 3.3 Write property test for command execute/undo round trip
    - **Property 2: Command Execute/Undo Round Trip**
    - **Validates: Requirements 5.5**
    - Generate arbitrary commands and EditorState instances
    - Verify execute followed by undo restores original state

- [x] 4. Implement tree-sitter integration
  - [x] 4.1 Add tree-sitter support to Buffer
    - Integrate tree-sitter parsing on buffer open for modes with grammar
    - Implement incremental re-parse on buffer modification
    - Store parse tree in `syntax_tree` field
    - Handle buffers without grammar (set `syntax_tree = None`)
    - _Requirements: 8.1, 8.2, 8.3, 8.4_
  
  - [x] 4.2 Create RenderState snapshot with syntax highlighting
    - Define `RenderState` struct with syntax highlight information derived from `syntax_tree`
    - Implement snapshot generation from EditorState
    - _Requirements: 8.5_

- [x] 5. Checkpoint - Ensure core data structures are working
  - Ensure all tests pass, ask the user if questions arise.

- [~] 6. Implement plugin system in velo-plugin
  - [X] 6.1 Set up velo-plugin crate
    - Add dependencies: `libloading`, `velo-types`
    - Create plugin directory path constant: `~/.config/velo/plugins/`
    - _Requirements: 1.5, 10.1_
  
  - [X] 6.2 Define Plugin trait
    - Create `Plugin` trait with methods: `name()`, `on_event(&mut self, event: &EditorEvent, state: &mut EditorState)`
    - Define plugin lifecycle hooks: `on_load()`, `on_unload()`
    - _Requirements: 11.1, 11.2_
  
  - [x] 6.3 Implement PluginRegistry
    - Create `PluginRegistry` struct holding `Vec<Box<dyn Plugin>>`
    - Implement `dispatch(event: &EditorEvent)` method that calls `on_event` on all plugins
    - Add panic recovery for plugin `on_event` calls
    - _Requirements: 12.2, 12.3, 12.4_
  
  - [ ]* 6.4 Write property test for plugin event dispatch completeness
    - **Property 6: Plugin Event Dispatch Completeness**
    - **Validates: Requirements 12.2, 12.3**
    - Generate arbitrary plugin sets and events
    - Verify every plugin receives every dispatched event
  
  - [x] 6.5 Implement Plugin Manager with libloading
    - Implement plugin discovery: scan `~/.config/velo/plugins/` directory
    - Implement plugin loading: `dlopen` each `.so`/`.dylib`/`.dll` file
    - Resolve `velo_plugin_init` symbol and call it to get `Box<dyn Plugin>`
    - Register returned plugin in PluginRegistry
    - Handle missing `velo_plugin_init` symbol gracefully with error logging
    - Implement plugin lifecycle: call `on_load()` once at startup, `on_unload()` at shutdown
    - _Requirements: 10.2, 10.3, 10.4, 10.5, 10.6, 10.7, 11.4_
  
  - [ ]* 6.6 Write property test for plugin loading registration
    - **Property 10: Plugin Loading Registration**
    - **Validates: Requirements 10.3, 10.4**
    - Test that valid plugin libraries result in exactly one registered plugin
  
  - [ ]* 6.7 Write property test for plugin lifecycle ordering
    - **Property 11: Plugin Lifecycle Ordering**
    - **Validates: Requirements 10.6, 10.7, 11.4**
    - Verify `on_load` called once before any `on_event`, `on_unload` called once after all events
  
  - [~] 6.8 Implement plugin installation command
    - Implement `:plugin install <name>` command handler
    - Download plugin binary and write to `~/.config/velo/plugins/`
    - Notify user that restart is required
    - _Requirements: 10.8, 10.9, 10.10_

- [~] 7. Implement configuration system
  - [~] 7.1 Define VeloConfig structure in velo-core
    - Create `VeloConfig` struct with fields: `editor: EditorSettings`, `theme: ThemeConfig`, `keymaps: KeymapConfig`, `ui: UiConfig`, `plugins: HashMap<String, toml::Value>`
    - Define nested config structures: `EditorSettings`, `ThemeConfig`, `KeymapConfig`, `UiConfig`
    - _Requirements: 13.2, 13.3, 13.4, 13.5, 13.6, 13.7_
  
  - [~] 7.2 Implement TOML config parser
    - Implement parser for `~/.config/velo/config.toml`
    - Parse `[editor]` section with: `tab_width`, `line_numbers`, `soft_wrap`, `scroll_off`
    - Parse `[theme]` section with `name` field
    - Parse `[keymaps]` section for global keybinding overrides
    - Parse `[keymaps.<mode_name>]` subsections for major-mode-specific overrides
    - Parse `[plugins]` section with `enabled` array
    - Parse `[plugins.<name>]` subsections for per-plugin config
    - Handle missing config.toml with built-in defaults
    - Handle syntax errors with descriptive error and fallback to defaults
    - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8, 13.9_
  
  - [ ]* 7.3 Write property test for TOML config round trip
    - **Property 8: TOML Config Round Trip**
    - **Validates: Requirements 13.1, 13.8, 13.9**
    - Generate arbitrary VeloConfig instances
    - Verify serialize to TOML then parse produces equivalent config
  
  - [~] 7.4 Implement config merge system
    - Implement Config_Merger that applies layers in order: (1) built-in defaults, (2) config.toml, (3) per-plugin TOML files, (4) Rust config crate
    - Parse per-plugin `~/.config/velo/plugins/<name>.toml` files
    - Merge per-plugin TOML over `[plugins.<name>]` sections
    - _Requirements: 15.1, 15.2_
  
  - [ ]* 7.5 Write property test for config merge order precedence
    - **Property 7: Config Merge Order Precedence**
    - **Validates: Requirements 15.1, 15.2, 15.3**
    - Generate config values at multiple layers
    - Verify final value matches highest-precedence layer
  
  - [~] 7.6 Implement Rust config crate support
    - Create public `velo-config-api` crate with `VeloUserConfig` trait
    - Define `VeloUserConfig` trait with `apply(&self, config: &mut VeloConfig)` method
    - Implement detection of `~/.config/velo/config/` directory
    - Implement `cargo build --release` invocation with caching (only rebuild when src/lib.rs changes)
    - Implement `dlopen` of compiled config crate
    - Resolve `velo_user_config_init() -> Box<dyn VeloUserConfig>` symbol
    - Call `apply()` on returned config object
    - Handle compilation failures gracefully with error logging
    - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5, 14.6, 14.7, 15.3_
  
  - [~] 7.7 Implement hot reload with notify crate
    - Add `notify` crate dependency to velo-app
    - Create Config_Watcher that watches `~/.config/velo/config.toml` and `~/.config/velo/plugins/*.toml`
    - On file change, re-parse and re-merge TOML configuration
    - Send `Command::ReloadConfig` to Core
    - Implement `Command::ReloadConfig` to update `EditorState::config`
    - Document that Rust config crate is NOT hot-reloaded (restart required)
    - _Requirements: 16.1, 16.2, 16.3, 16.4, 16.5_
  
  - [ ]* 7.8 Write property test for hot reload consistency
    - **Property 9: Hot Reload Consistency**
    - **Validates: Requirements 16.2, 16.3**
    - Simulate config.toml changes
    - Verify VeloConfig reflects updated values after ReloadConfig
  
  - [~] 7.9 Implement theme system
    - Define TOML theme file format with `[colors]`, `[syntax]`, `[ui]` sections
    - Implement Theme_Loader that resolves theme name to TOML file
    - Support built-in themes and plugin-provided themes
    - Fallback to default theme if configured theme not found
    - Support per-frontend UI layout configuration
    - _Requirements: 17.1, 17.2, 17.3, 17.4, 17.5_

- [~] 8. Checkpoint - Ensure configuration system is working
  - Ensure all tests pass, ask the user if questions arise.

- [~] 9. Implement concurrency model in velo-app
  - [~] 9.1 Set up velo-app crate with Tokio
    - Add dependencies: `tokio` (with features: `sync`, `rt-multi-thread`, `macros`), `velo-core`, `velo-plugin`, `velo-types`
    - _Requirements: 9.1_
  
  - [~] 9.2 Define channel topology
    - Create `input_tx / input_rx` bounded mpsc channel for `InputEvent` (UI → App)
    - Create `cmd_tx / cmd_rx` bounded mpsc channel for `Box<dyn Command>` (App → Core)
    - Create `task_tx / task_rx` bounded mpsc channel for `BackgroundTask` (Core → Workers)
    - Create `result_tx / result_rx` bounded mpsc channel for `TaskResult` (Workers → Core)
    - _Requirements: 9.2, 9.3, 9.4, 9.5_
  
  - [~] 9.3 Implement RenderState sharing with Arc<RwLock>
    - Create `Arc<tokio::sync::RwLock<RenderState>>` for UI access
    - Ensure UI never holds lock on EditorState
    - Implement RenderState snapshot generation after each command
    - Publish snapshot to Arc<RwLock> before processing next command
    - _Requirements: 9.7, 9.8_
  
  - [ ]* 9.4 Write property test for RenderState published after every command
    - **Property 13: RenderState Published After Every Command**
    - **Validates: Requirements 9.8**
    - Execute arbitrary command sequences
    - Verify RenderState snapshot is written before next command begins
  
  - [~] 9.5 Implement Core task event loop
    - Create Core task that owns EditorState
    - Receive commands from `cmd_rx` channel
    - Execute commands via `Command::execute`
    - Dispatch EditorEvents to PluginRegistry
    - Produce and publish RenderState snapshots
    - Dispatch BackgroundTasks to worker pool via `task_tx`
    - Receive TaskResults from `result_rx`
    - _Requirements: 9.6, 9.10_
  
  - [~] 9.6 Implement keybinding resolver
    - Create Keybinding_Resolver in velo-app
    - Implement resolution order: active minor modes (most recent first) → major mode → global defaults
    - Handle same key in multiple layers by using highest-priority binding
    - Handle unbound keys gracefully (no command, no error)
    - Apply major-mode-specific keymap overrides from config.toml
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_
  
  - [ ]* 9.7 Write property test for keybinding resolution priority
    - **Property 3: Keybinding Resolution Priority**
    - **Validates: Requirements 7.1, 7.2, 7.3, 7.4**
    - Generate arbitrary keybinding configurations across layers
    - Verify resolved command matches highest-priority layer
  
  - [~] 9.8 Implement App task event loop
    - Create App task that receives InputEvents from UI
    - Translate InputEvents to Commands via Keybinding_Resolver
    - Send Commands to Core via `cmd_tx` channel
    - Handle backpressure from bounded `cmd_tx` channel
    - _Requirements: 9.6_

- [~] 10. Implement minor mode system
  - [~] 10.1 Implement minor mode activation/deactivation
    - Add methods to Buffer for activating/deactivating minor modes
    - Call `on_activate` when minor mode is added to buffer
    - Call `on_deactivate` when minor mode is removed from buffer
    - Support multiple simultaneous minor modes per buffer
    - _Requirements: 6.5, 6.6, 6.7_
  
  - [ ]* 10.2 Write property test for minor mode activate/deactivate round trip
    - **Property 5: Minor Mode Activate/Deactivate Round Trip**
    - **Validates: Requirements 6.5, 6.6**
    - Activate then immediately deactivate arbitrary minor modes
    - Verify buffer minor_modes Vec returns to original state

- [~] 11. Implement velo-tui frontend
  - [~] 11.1 Set up velo-tui crate
    - Add dependencies: `ratatui`, `crossterm`, `tokio`, `velo-app`, `velo-types`
    - _Requirements: 18.1_
  
  - [~] 11.2 Implement input polling with crossterm
    - Poll raw input events using crossterm
    - Convert to InputEvent values
    - Send to velo-app via `input_tx` channel without blocking render loop
    - _Requirements: 18.2, 18.5_
  
  - [~] 11.3 Implement render loop with ratatui
    - Acquire `Arc<RwLock<RenderState>>` read lock
    - Render latest RenderState snapshot
    - Release lock immediately
    - Decouple render loop from command processing tick
    - _Requirements: 18.3, 18.4_

- [ ]* 12. Write property test for MajorModeRegistry registration availability
  - **Property 14: MajorModeRegistry Registration Availability**
  - **Validates: Requirements 6.8, 12.1**
  - Register arbitrary major modes
  - Verify file-type lookup returns registered mode for matching patterns

- [~] 13. Create velo-gui stub crate
  - [~] 13.1 Create velo-gui crate structure
    - Create `velo-gui/Cargo.toml` with dependencies on `velo-app` and `velo-types`
    - Create stub `velo-gui/src/lib.rs` with TODO comments for future GUI implementation
    - Document that GUI frontend will follow same contract as velo-tui
    - _Requirements: 1.1_

- [~] 14. Integration and wiring
  - [~] 14.1 Create main binary in velo-app
    - Implement `main()` function that initializes Tokio runtime
    - Initialize EditorState with default config
    - Load plugins from `~/.config/velo/plugins/`
    - Parse and merge configuration layers
    - Spawn Core task, App task, and worker pool
    - Initialize velo-tui frontend
    - Wire all channels together
    - _Requirements: All_
  
  - [~] 14.2 Implement graceful shutdown
    - Handle shutdown signal (Ctrl+C)
    - Call `on_unload()` on all plugins
    - Close all channels
    - Wait for all tasks to complete
    - _Requirements: 10.7_

- [~] 15. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional property-based tests and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation at logical breakpoints
- Property tests validate universal correctness properties from the design document
- All 18 requirements are covered across implementation tasks
- All 14 correctness properties have corresponding property test tasks
