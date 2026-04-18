// velo-core: Buffer management, EditorState, Commands, File I/O
// Requirements 2, 3, 5, 6, 8, 9

pub mod buffer;
pub mod command;
pub mod config;
pub mod mode;
pub mod render_state;
pub mod state;

pub use buffer::Buffer;
pub use command::{Command, DeleteChar, InsertChar, MoveCursor, OpenFile, SaveFile};
pub use config::VeloConfig;
pub use mode::{
    IndentStyle, KeybindingMap, MajorMode, MajorModeRegistry, MinorMode, PlainTextMode,
};
pub use render_state::RenderState;
pub use state::EditorState;
