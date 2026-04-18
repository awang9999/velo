// EditorEvent taxonomy
// Requirements 12.1, 4.1, 4.2, 4.3

use crate::{Position, Selection};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum EditorEvent {
    // Buffer events
    BufferOpened {
        buffer_id: usize,
        path: Option<PathBuf>,
    },
    BufferClosed {
        buffer_id: usize,
    },
    BufferModified {
        buffer_id: usize,
    },
    BufferSaved {
        buffer_id: usize,
        path: PathBuf,
    },
    
    // Cursor events
    CursorMoved {
        buffer_id: usize,
        new_pos: Position,
    },
    SelectionChanged {
        buffer_id: usize,
        selection: Selection,
    },
    
    // Editor lifecycle events
    VeloStarted,
    VeloShutdown,
    
    // Input events
    KeyPressed {
        key: String,
        modifiers: Vec<String>,
    },
}
