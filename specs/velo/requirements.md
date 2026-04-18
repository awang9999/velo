# Requirements Document

## Introduction

Velo is a high-performance, post-modern, all-purpose text editor written in Rust. It is structured as a six-crate Cargo workspace with a strict layered architecture that separates core editing logic from UI concerns. Velo supports both a Terminal UI (TUI) and a future Graphical UI (GUI) over the same core. The system is multi-threaded by design, uses Tokio as its async runtime, and provides a dynamic native plugin system, an Emacs-inspired major/minor mode system, tree-sitter-based incremental syntax highlighting, and a two-layer configuration system (declarative TOML + optional Rust config crate).

---

## Glossary

- **Workspace**: The Cargo workspace named `velo` containing all crates.
- **Buffer**: The in-memory representation of an open file or unsaved document, backed by a `ropey::Rope`.
- **EditorState**: The single source of truth for all mutable editor state, owned by `velo-core`.
- **Command**: The only mechanism for mutating `EditorState`; every user action is modeled as a `Command`.
- **MajorMode**: A per-buffer mode that defines syntax highlighting grammar, indentation rules, and a base keybinding layer for a file type.
- **MinorMode**: A stackable, orthogonal per-buffer or global mode implemented as a plugin that adds or overrides behavior without replacing the major mode.
- **MajorModeRegistry**: The registry of all known major modes (built-in and plugin-provided), used for file-type detection.
- **PluginRegistry**: The registry of all loaded plugins, held inside `EditorState`.
- **Plugin**: A native shared library (`.so`/`.dylib`/`.dll`) loaded at startup via `libloading`, implementing the `Plugin` trait.
- **EditorEvent**: A typed event produced after each command execution, dispatched to all registered plugins.
- **RenderState**: A lightweight snapshot of `EditorState` produced after each command and shared with the UI via `Arc<tokio::sync::RwLock<RenderState>>`.
- **VeloConfig**: The fully-merged in-memory configuration object owned by `EditorState`.
- **Config_Watcher**: The `notify`-based file watcher in `velo-app` that monitors `config.toml` and plugin TOML files.
- **Config_Merger**: The component that applies configuration layers in the defined merge order.
- **Config_Parser**: The component that parses `config.toml` into a `VeloConfig`.
- **Theme_Loader**: The component that resolves a theme name to a TOML theme definition.
- **Plugin_Manager**: The component in `velo-plugin` responsible for plugin discovery, loading, registration, and lifecycle management.
- **Keybinding_Resolver**: The component in `velo-app` that maps raw input events to `Command` instances using the layered keybinding system.
- **InputEvent**: A raw keystroke or mouse event sent from the UI to `velo-app`.
- **BackgroundTask**: A unit of heavy work (file I/O, search, indexing) dispatched to a Tokio worker task.
- **Position**: A `(line: usize, column: usize)` coordinate within a buffer.
- **Range**: A pair of `Position` values `{ start, end }` where `start <= end`.
- **Selection**: A `{ anchor: Position, head: Position }` pair representing a cursor selection.

---

## Requirements

### Requirement 1: Workspace Structure and Crate Layering

**User Story:** As a contributor, I want the project to be organized as a strictly layered six-crate Cargo workspace, so that each crate has a single well-defined responsibility and UI concerns are fully decoupled from core logic.

#### Acceptance Criteria

1. THE Workspace SHALL contain exactly six crates: `velo-types`, `velo-core`, `velo-plugin`, `velo-app`, `velo-tui`, and `velo-gui`.
2. THE `velo-types` crate SHALL have zero external dependencies.
3. THE dependency graph SHALL be acyclic and SHALL only permit dependencies in the direction: `velo-tui`/`velo-gui` → `velo-app` → `velo-core`/`velo-plugin` → `velo-types`.
4. THE `velo-core` crate SHALL have no dependency on `velo-tui` or `velo-gui`.
5. THE `velo-plugin` crate SHALL have no dependency on `velo-tui`, `velo-gui`, or `velo-app`.
6. THE `velo-types` crate SHALL define the shared primitives `Position`, `Range`, `Selection`, `EditorEvent`, and `VeloError`.

---

### Requirement 2: Buffer Data Model

**User Story:** As a developer, I want each open file to be represented by a well-defined `Buffer` structure, so that all editing operations have a consistent, predictable data model to operate on.

#### Acceptance Criteria

1. THE Buffer SHALL contain a `ropey::Rope` field holding the text content.
2. THE Buffer SHALL contain a `file_path: Option<PathBuf>` field that is `None` for unsaved buffers.
3. THE Buffer SHALL contain an `is_dirty: bool` field that is `true` when the buffer has been modified since the last save.
4. THE Buffer SHALL contain a `cursor: Position` field representing the primary cursor location.
5. THE Buffer SHALL contain a `selections: Vec<Selection>` field for multi-cursor and selection ranges.
6. THE Buffer SHALL contain a `major_mode: Box<dyn MajorMode>` field holding the active major mode.
7. THE Buffer SHALL contain a `minor_modes: Vec<Box<dyn MinorMode>>` field holding all active minor modes.
8. THE Buffer SHALL contain a `syntax_tree: Option<tree_sitter::Tree>` field for the incremental parse tree.
9. WHEN a buffer is modified, THE Buffer SHALL set `is_dirty` to `true`.
10. WHEN a buffer is saved, THE Buffer SHALL set `is_dirty` to `false`.

---

### Requirement 3: EditorState as Single Source of Truth

**User Story:** As a developer, I want `EditorState` to be the single authoritative source of all mutable editor state, so that state management is predictable and auditable.

#### Acceptance Criteria

1. THE EditorState SHALL contain a `buffers: Vec<Buffer>` field holding all open buffers.
2. THE EditorState SHALL contain an `active_buffer_id: usize` field identifying the currently active buffer.
3. THE EditorState SHALL contain a `config: VeloConfig` field holding the merged configuration.
4. THE EditorState SHALL contain a `major_mode_registry: MajorModeRegistry` field.
5. THE EditorState SHALL contain a `plugin_registry: PluginRegistry` field.
6. WHEN a buffer is opened, THE EditorState SHALL set `active_buffer_id` to a valid index into the `buffers` Vec.
7. IF `active_buffer_id` is accessed and no buffers are open, THEN THE EditorState SHALL return an appropriate error rather than panicking.

---

### Requirement 4: Position and Range Invariants

**User Story:** As a developer, I want `Position` and `Range` types to enforce structural invariants, so that cursor and selection logic is free of invalid-state bugs.

#### Acceptance Criteria

1. THE Position SHALL be defined as `{ line: usize, column: usize }`.
2. THE Range SHALL be defined as `{ start: Position, end: Position }`.
3. THE Selection SHALL be defined as `{ anchor: Position, head: Position }`.
4. WHEN a Range is constructed, THE Range SHALL satisfy `start.line < end.line`, OR `start.line == end.line AND start.column <= end.column`.
5. IF a Range is constructed with `start > end`, THEN THE system SHALL return a `VeloError` rather than storing an invalid Range.

---

### Requirement 5: Command System and Unidirectional Data Flow

**User Story:** As a developer, I want all `EditorState` mutations to go through the `Command` trait, so that undo/redo is straightforward and data flow is unidirectional.

#### Acceptance Criteria

1. THE Command trait SHALL define `fn execute(&self, state: &mut EditorState) -> Result<(), VeloError>`.
2. THE Command trait SHALL define `fn undo(&self, state: &mut EditorState) -> Result<(), VeloError>` as an optional operation.
3. THE Command trait SHALL define `fn name(&self) -> &str`.
4. WHEN a Command is executed, THE EditorState SHALL be mutated only via `Command::execute`.
5. WHEN `Command::undo` is called after `Command::execute`, THE EditorState SHALL be restored to its state prior to the `execute` call.
6. THE system SHALL provide built-in commands including at minimum: `InsertChar`, `DeleteChar`, `MoveCursor`, `SaveFile`, and `OpenFile`.

---

### Requirement 6: Emacs-Inspired Major/Minor Mode System

**User Story:** As a user, I want a per-buffer major mode and stackable minor modes, so that editor behavior is composable and file-type-aware without global modal editing.

#### Acceptance Criteria

1. THE MajorMode trait SHALL define `fn name(&self) -> &str`, `fn file_patterns(&self) -> &[&str]`, `fn grammar(&self) -> Option<&tree_sitter::Language>`, `fn indent_style(&self) -> IndentStyle`, and `fn keybindings(&self) -> KeybindingMap`.
2. WHEN a file is opened, THE MajorModeRegistry SHALL assign the major mode whose `file_patterns` matches the file extension.
3. IF no registered major mode pattern matches the file extension, THEN THE Buffer SHALL use a plain-text fallback major mode.
4. THE MinorMode trait SHALL extend the Plugin trait and define `fn name(&self) -> &str`, `fn keybindings(&self) -> KeybindingMap`, `fn on_activate(&mut self, buffer_id: usize, state: &mut EditorState)`, and `fn on_deactivate(&mut self, buffer_id: usize, state: &mut EditorState)`.
5. WHEN a MinorMode is activated on a buffer, THE Buffer SHALL add it to the `minor_modes` Vec and call `on_activate`.
6. WHEN a MinorMode is deactivated on a buffer, THE Buffer SHALL remove it from the `minor_modes` Vec and call `on_deactivate`.
7. THE system SHALL support zero or more minor modes active simultaneously on a single buffer.
8. WHEN a plugin calls `MajorModeRegistry::register()`, THE MajorModeRegistry SHALL make the new major mode available for file-type detection on subsequent buffer opens.

---

### Requirement 7: Keybinding Resolution

**User Story:** As a user, I want keybindings to be resolved in a well-defined priority order, so that minor mode overrides, major mode defaults, and global defaults compose predictably.

#### Acceptance Criteria

1. WHEN a keystroke is received, THE Keybinding_Resolver SHALL resolve it by checking active minor modes first, then the active major mode, then global defaults from `VeloConfig`.
2. WHEN the same key is bound in both a minor mode and the active major mode, THE Keybinding_Resolver SHALL use the minor mode binding.
3. WHEN the same key is bound in multiple active minor modes, THE Keybinding_Resolver SHALL use the binding from the most recently activated minor mode.
4. WHEN a key has no binding in any minor mode or the major mode, THE Keybinding_Resolver SHALL use the global default binding from `VeloConfig`.
5. IF a key has no binding at any layer, THEN THE Keybinding_Resolver SHALL produce no command and SHALL NOT raise an error.
6. WHEN a major-mode-specific keymap override is defined in `config.toml` under `[keymaps.<mode_name>]`, THE Keybinding_Resolver SHALL apply it as part of the major mode layer.

---

### Requirement 8: tree-sitter Incremental Syntax Highlighting

**User Story:** As a user, I want syntax highlighting powered by tree-sitter from day one, so that highlighting is accurate, incremental, and error-tolerant even in large files.

#### Acceptance Criteria

1. THE `velo-core` crate SHALL integrate `tree-sitter` for syntax highlighting.
2. WHEN a buffer is opened with a major mode that provides a `tree_sitter::Language`, THE Buffer SHALL parse the full content and store the result in `syntax_tree`.
3. WHEN a buffer is modified, THE Buffer SHALL incrementally update `syntax_tree` using tree-sitter's incremental re-parse API, re-parsing only the affected region.
4. WHEN a buffer is opened with a major mode that provides no grammar, THE Buffer SHALL set `syntax_tree` to `None`.
5. THE RenderState snapshot SHALL include sufficient syntax highlight information derived from `syntax_tree` for the UI to render highlighted text.

---

### Requirement 9: Concurrency Model and Channel Topology

**User Story:** As a developer, I want the UI thread to never be blocked by core operations, so that the editor remains responsive under all workloads.

#### Acceptance Criteria

1. THE system SHALL use Tokio as the async runtime throughout `velo-app` and `velo-core`.
2. THE `input_tx / input_rx` channel (UI → App) SHALL be a bounded `tokio::sync::mpsc` channel carrying `InputEvent` values.
3. THE `cmd_tx / cmd_rx` channel (App → Core) SHALL be a bounded `tokio::sync::mpsc` channel carrying `Box<dyn Command>` values.
4. THE `task_tx / task_rx` channel (Core → Workers) SHALL be a bounded `tokio::sync::mpsc` channel carrying `BackgroundTask` values.
5. THE `result_tx / result_rx` channel (Workers → Core) SHALL be a bounded `tokio::sync::mpsc` channel carrying `TaskResult` values.
6. WHEN the Core task falls behind processing commands, THE bounded `cmd_tx` channel SHALL apply backpressure to the App task rather than unboundedly queuing commands.
7. THE UI thread SHALL NOT hold a lock on `EditorState` at any time.
8. WHEN a command is applied, THE Core SHALL produce a `RenderState` snapshot and publish it via `Arc<tokio::sync::RwLock<RenderState>>`.
9. WHEN the UI acquires the `RenderState` read lock to render, THE Core task SHALL be able to continue processing commands concurrently.
10. WHEN heavy work is required (file I/O, regex search, indexing), THE Core SHALL dispatch it as a `BackgroundTask` via `tokio::spawn` rather than blocking the Core task.

---

### Requirement 10: Dynamic Native Plugin System

**User Story:** As a user, I want to install and use native Rust plugins without recompiling Velo, so that I can extend the editor with new functionality.

#### Acceptance Criteria

1. THE Plugin_Manager SHALL support only Rust-compiled native shared libraries (`.so` on Linux, `.dylib` on macOS, `.dll` on Windows).
2. WHEN Velo starts, THE Plugin_Manager SHALL scan `~/.config/velo/plugins/` and attempt to load each shared library found there.
3. WHEN a shared library is loaded, THE Plugin_Manager SHALL resolve the `velo_plugin_init` symbol and call it to obtain a `Box<dyn Plugin>`.
4. WHEN `velo_plugin_init` is resolved successfully, THE Plugin_Manager SHALL register the returned `Box<dyn Plugin>` in the `PluginRegistry`.
5. IF a shared library does not export `velo_plugin_init`, THEN THE Plugin_Manager SHALL log an error and skip that library without crashing.
6. WHEN a plugin is loaded, THE Plugin_Manager SHALL call `on_load()` exactly once, providing read access to `VeloConfig`.
7. WHEN Velo shuts down, THE Plugin_Manager SHALL call `on_unload()` on each loaded plugin.
8. WHEN the user issues `:plugin install <name>`, THE Plugin_Manager SHALL download the plugin binary and write it to `~/.config/velo/plugins/`.
9. WHEN a plugin is installed, THE system SHALL notify the user that a restart is required to activate the plugin.
10. WHEN Velo restarts after a plugin install, THE Plugin_Manager SHALL load the newly installed plugin as part of the normal startup scan.

---

### Requirement 11: Plugin Trait and Lifecycle

**User Story:** As a plugin author, I want a well-defined `Plugin` trait and lifecycle, so that I can write plugins that integrate cleanly with the editor.

#### Acceptance Criteria

1. THE Plugin trait SHALL define `fn name(&self) -> &str` and `fn on_event(&mut self, event: &EditorEvent, state: &mut EditorState)`.
2. WHEN `on_event` is called, THE Plugin SHALL receive a mutable reference to `EditorState`, allowing it to modify buffers and other state.
3. WHEN a plugin needs to perform async work (e.g., network calls), THE Plugin SHALL dispatch a `BackgroundTask` through a channel handle provided at `on_load` time rather than blocking the Core task.
4. THE Plugin lifecycle SHALL proceed in order: Discovery → Loading → Registration → Initialization (`on_load`) → Event Dispatch (`on_event`) → Teardown (`on_unload`).

---

### Requirement 12: EditorEvent Taxonomy and Dispatch

**User Story:** As a plugin author, I want a comprehensive set of typed editor events, so that my plugin can react to all relevant editor state changes.

#### Acceptance Criteria

1. THE `velo-types` crate SHALL define the `EditorEvent` enum with at minimum the following variants: `BufferOpened { buffer_id, path }`, `BufferClosed { buffer_id }`, `BufferModified { buffer_id, change }`, `BufferSaved { buffer_id, path }`, `CursorMoved { buffer_id, new_pos }`, `SelectionChanged { buffer_id, selection }`, `VeloStarted`, `VeloShutdown`, and `KeyPressed { key, modifiers }`.
2. WHEN a command is applied to `EditorState`, THE Core SHALL produce the appropriate `EditorEvent` and dispatch it to all registered plugins via `PluginRegistry::dispatch`.
3. WHEN `PluginRegistry::dispatch` is called, THE PluginRegistry SHALL call `on_event` on every registered plugin with the event.
4. WHEN a plugin's `on_event` raises a panic, THE system SHALL recover gracefully and continue dispatching to remaining plugins.

---

### Requirement 13: Declarative TOML Configuration

**User Story:** As a user, I want to configure Velo through a `config.toml` file, so that I can customize keybindings, theme, editor settings, and plugins without writing code.

#### Acceptance Criteria

1. THE Config_Parser SHALL parse `~/.config/velo/config.toml` into a `VeloConfig` at startup.
2. THE `config.toml` SHALL support an `[editor]` section with fields: `tab_width`, `line_numbers`, `soft_wrap`, and `scroll_off`.
3. THE `config.toml` SHALL support a `[theme]` section with a `name` field referencing a built-in or plugin-provided theme.
4. THE `config.toml` SHALL support a `[keymaps]` section for global keybinding overrides in the format `"key_combo" = "command_name"`.
5. THE `config.toml` SHALL support `[keymaps.<mode_name>]` subsections for major-mode-specific keybinding overrides.
6. THE `config.toml` SHALL support a `[plugins]` section with an `enabled` array listing plugin names.
7. THE `config.toml` SHALL support `[plugins.<name>]` subsections for inline per-plugin configuration.
8. IF `config.toml` is absent at startup, THEN THE system SHALL use built-in defaults and SHALL NOT crash.
9. IF `config.toml` contains a syntax error, THEN THE Config_Parser SHALL report a descriptive error and fall back to built-in defaults.

---

### Requirement 14: Scriptable Rust Config Crate

**User Story:** As a power user, I want to write a Rust config crate for programmatic configuration, so that I can use conditional logic, computed values, and full language expressiveness to configure Velo.

#### Acceptance Criteria

1. WHEN `~/.config/velo/config/` exists at startup, THE system SHALL attempt to compile it using `cargo build --release`.
2. WHEN the Rust config crate source has not changed since the last build, THE system SHALL use the cached compiled artifact without recompiling.
3. WHEN the Rust config crate compiles successfully, THE system SHALL `dlopen` the resulting shared library and resolve `fn velo_user_config_init() -> Box<dyn VeloUserConfig>`.
4. WHEN `velo_user_config_init` is resolved, THE system SHALL call `apply(&mut VeloConfig)` on the returned object, allowing it to override any value set by `config.toml`.
5. IF the Rust config crate fails to compile, THEN THE system SHALL log the compiler error and proceed with the `config.toml` values.
6. THE `VeloUserConfig` trait SHALL be defined in a public `velo-config-api` crate that both plugin authors and config crate authors depend on.
7. WHEN the Rust config crate's `src/lib.rs` changes, THE system SHALL require a restart to apply the new config (hot reload is not supported for the Rust config crate).

---

### Requirement 15: Configuration Merge Order

**User Story:** As a user, I want configuration layers to be applied in a well-defined order, so that I can predict which settings take precedence.

#### Acceptance Criteria

1. THE Config_Merger SHALL apply configuration in the following order, with later layers overriding earlier ones: (1) Velo built-in defaults, (2) `config.toml`, (3) per-plugin `~/.config/velo/plugins/<name>.toml` files, (4) Rust config crate `apply()` call.
2. WHEN a per-plugin `~/.config/velo/plugins/<name>.toml` exists, THE Config_Merger SHALL merge its values over the corresponding `[plugins.<name>]` section in `config.toml`.
3. WHEN the Rust config crate's `apply()` sets a value, THE resulting `VeloConfig` SHALL reflect that value regardless of what `config.toml` specified.

---

### Requirement 16: Hot Reload of TOML Configuration

**User Story:** As a user, I want changes to `config.toml` to take effect without restarting Velo, so that I can iterate on my configuration quickly.

#### Acceptance Criteria

1. THE Config_Watcher SHALL use the `notify` crate to watch `~/.config/velo/config.toml` and all `~/.config/velo/plugins/*.toml` files for changes.
2. WHEN a watched TOML file changes on disk, THE Config_Watcher SHALL re-parse and re-merge the declarative configuration layer and send `Command::ReloadConfig` to the Core.
3. WHEN `Command::ReloadConfig` is executed, THE EditorState SHALL update `VeloConfig` to reflect the new TOML values.
4. WHEN `Command::ReloadConfig` is executed, THE system SHALL NOT require a restart.
5. THE Rust config crate SHALL NOT be hot-reloaded; a restart is required when `~/.config/velo/config/src/lib.rs` changes.

---

### Requirement 17: Theme System

**User Story:** As a user, I want to select and customize themes via named TOML files, so that I can control the visual appearance of the editor.

#### Acceptance Criteria

1. THE Theme_Loader SHALL resolve a theme name from `VeloConfig` to a TOML theme file at startup.
2. THE theme TOML format SHALL support `[colors]` (background, foreground, cursor), `[syntax]` (keyword, string, comment, function, and other tree-sitter scope mappings), and `[ui]` (statusline_bg, statusline_fg, border) sections.
3. WHEN a plugin provides a theme, THE Theme_Loader SHALL make it available by name alongside built-in themes.
4. IF the configured theme name does not resolve to any known theme, THEN THE Theme_Loader SHALL fall back to the built-in default theme and log a warning.
5. THE `[ui]` layout configuration in `config.toml` SHALL be per-frontend, allowing `velo-tui` and `velo-gui` to have independent layout settings.

---

### Requirement 18: Terminal UI Frontend

**User Story:** As a user, I want a fully functional terminal UI, so that I can use Velo in any terminal environment.

#### Acceptance Criteria

1. THE `velo-tui` crate SHALL use the `ratatui` crate for terminal rendering.
2. WHEN the terminal UI starts, THE `velo-tui` SHALL poll raw input events using `crossterm` and send them as `InputEvent` values to `velo-app`.
3. WHEN a new `RenderState` snapshot is available, THE `velo-tui` SHALL render it on the next render tick by acquiring the `Arc<RwLock<RenderState>>` read lock, rendering, and immediately releasing the lock.
4. THE `velo-tui` render loop SHALL be decoupled from the command processing tick, rendering at its own cadence from the latest available snapshot.
5. THE `velo-tui` SHALL send `InputEvent` values to `velo-app` via the bounded `input_tx` channel without blocking the render loop.

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

---

### Property 1: Range Validity Invariant

*For any* `Range` value stored in the system, `start.line < end.line`, OR `start.line == end.line AND start.column <= end.column`.

**Validates: Requirements 4.4, 4.5**

---

### Property 2: Command Execute/Undo Round Trip

*For any* `Command` that implements `undo`, executing the command on an `EditorState` and then calling `undo` on the same state SHALL produce an `EditorState` equivalent to the original pre-execute state.

**Validates: Requirements 5.5**

---

### Property 3: Keybinding Resolution Priority

*For any* keystroke and any combination of active minor modes, major mode, and global bindings where the same key appears in multiple layers, the resolved command SHALL be the one from the highest-priority layer (most recently activated minor mode > any minor mode > major mode > global defaults).

**Validates: Requirements 7.1, 7.2, 7.3, 7.4**

---

### Property 4: File-Type Major Mode Assignment

*For any* file path and any set of registered major modes, opening the file SHALL assign the major mode whose `file_patterns` matches the file extension, or the plain-text fallback if no pattern matches.

**Validates: Requirements 6.2, 6.3**

---

### Property 5: Minor Mode Activate/Deactivate Round Trip

*For any* buffer and any minor mode, activating then immediately deactivating the minor mode SHALL leave the buffer's `minor_modes` Vec in the same state as before activation.

**Validates: Requirements 6.5, 6.6**

---

### Property 6: Plugin Event Dispatch Completeness

*For any* `EditorEvent` dispatched via `PluginRegistry::dispatch` and any set of registered plugins, every plugin in the registry SHALL have its `on_event` called with that event.

**Validates: Requirements 12.2, 12.3**

---

### Property 7: Config Merge Order Precedence

*For any* configuration key that is set at multiple layers (built-in defaults, `config.toml`, per-plugin TOML, Rust config crate), the final value in `VeloConfig` SHALL equal the value set by the highest-precedence layer that specifies it.

**Validates: Requirements 15.1, 15.2, 15.3**

---

### Property 8: TOML Config Round Trip

*For any* valid `VeloConfig` object, serializing it to TOML and then parsing the result SHALL produce a `VeloConfig` equivalent to the original.

**Validates: Requirements 13.1, 13.8, 13.9**

---

### Property 9: Hot Reload Consistency

*For any* change to `config.toml`, after `Command::ReloadConfig` is processed, the `VeloConfig` in `EditorState` SHALL reflect the values from the updated file.

**Validates: Requirements 16.2, 16.3**

---

### Property 10: Plugin Loading Registration

*For any* valid plugin shared library that exports `velo_plugin_init`, loading it SHALL result in exactly one `Box<dyn Plugin>` being registered in the `PluginRegistry`.

**Validates: Requirements 10.3, 10.4**

---

### Property 11: Plugin Lifecycle Ordering

*For any* loaded plugin, `on_load` SHALL be called exactly once before any `on_event` call, and `on_unload` SHALL be called exactly once after all `on_event` calls during a single Velo session.

**Validates: Requirements 10.6, 10.7, 11.4**

---

### Property 12: active_buffer_id Validity

*For any* sequence of buffer open and close operations, `EditorState::active_buffer_id` SHALL always be a valid index into the `buffers` Vec when at least one buffer is open.

**Validates: Requirements 3.6, 3.7**

---

### Property 13: RenderState Published After Every Command

*For any* command executed against `EditorState`, a new `RenderState` snapshot SHALL be written to the `Arc<RwLock<RenderState>>` before the next command begins processing.

**Validates: Requirements 9.8**

---

### Property 14: MajorModeRegistry Registration Availability

*For any* major mode registered via `MajorModeRegistry::register()`, that mode SHALL be returned by the registry's file-type lookup for any file path matching its `file_patterns`.

**Validates: Requirements 6.8, 12.1**

---
