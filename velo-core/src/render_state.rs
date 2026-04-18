use crate::buffer::Buffer;
use tree_sitter::Tree;
use velo_types::Position;
use velo_types::Selection;

/// RenderState is a lightweight snapshot of the editor state
/// used by the UI for rendering. It contains the current
/// buffer text, a copy of the syntax tree (if any), and
/// minimal cursor/selection information.
pub struct RenderState {
    /// The full buffer text at the time of snapshot
    pub buffer_text: String,
    /// Optional syntax tree for highlighting
    pub syntax_tree: Option<Tree>,
    /// Current cursor position
    pub cursor: Position,
    /// Current selections
    pub selections: Vec<Selection>,
}

impl RenderState {
    /// Create a RenderState from a Buffer.
    /// This takes a snapshot of the buffer’s text and state.
    pub fn from_buffer(buffer: &Buffer) -> Self {
        Self {
            buffer_text: buffer.text(),
            syntax_tree: buffer.syntax_tree.clone(),
            cursor: buffer.cursor,
            selections: buffer.selections.clone(),
        }
    }

    /// Create a RenderState from the active buffer in EditorState.
    /// If there is no active buffer, an empty snapshot is returned.
    pub fn from_state(state: &crate::state::EditorState) -> Self {
        match state.active_buffer() {
            Ok(buffer) => Self::from_buffer(buffer),
            Err(_) => Self {
                buffer_text: String::new(),
                syntax_tree: None,
                cursor: Position::new(0, 0),
                selections: Vec::new(),
            },
        }
    }
}
