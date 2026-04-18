// Application shell and event loop
// Requirement 9

use velo_core::EditorState;

pub struct App {
    pub state: EditorState,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
