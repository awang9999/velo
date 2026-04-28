// velo/velo-app/src/app.rs
// Add Command trait import to velo-app/src/app.rs

use std::sync::Arc;

use tokio::sync::{mpsc::Receiver, RwLock};

use velo_core::command::{DeleteChar, InsertChar};
use velo_core::Command;
use velo_core::{EditorState, RenderState};
use velo_types::EditorEvent;

/// The core application struct that owns the editor state and receives input events.
pub struct App {
    pub state: EditorState,
    pub input_rx: Receiver<EditorEvent>,
    pub render_state: Arc<RwLock<RenderState>>,
    pub exit_key: String,
}

impl App {
    /// Create a new `App` that talks to the UI via `input_rx` and writes snapshots
    /// to `render_state`.  The default exit key is `Ctrl+Q`.
    pub fn new_with_input_rx_and_render_state(
        input_rx: Receiver<EditorEvent>,
        render_state: Arc<RwLock<RenderState>>,
    ) -> Self {
        Self {
            state: EditorState::new(),
            input_rx,
            render_state,
            exit_key: "Ctrl+Q".to_string(),
        }
    }

    /// Set a custom key sequence for exiting the editor.
    pub fn set_exit_key(&mut self, key: impl Into<String>) {
        self.exit_key = key.into();
    }

    /// Parse a string from crossterm into a simple key type.
    fn parse_key_from_str(key: &str) -> Option<KeyType> {
        match key {
            "Left" => Some(KeyType::Left),
            "Right" => Some(KeyType::Right),
            "Up" => Some(KeyType::Up),
            "Down" => Some(KeyType::Down),
            "Backspace" => Some(KeyType::Backspace),
            _ => {
                if key.starts_with("Char('") && key.ends_with("')") {
                    key.chars().nth(6).map(KeyType::Char)
                } else {
                    None
                }
            }
        }
    }

    /// Main event loop:  receives an event, turns it into a command,
    /// executes it, then writes a fresh snapshot to the render state.
    /// If the exit key is pressed, the loop ends and the program exits.
    pub async fn run(&mut self) {
        while let Some(event) = self.input_rx.recv().await {
            match event {
                EditorEvent::KeyPressed { key, .. } => {
                    // Exit handling
                    if key == self.exit_key {
                        break;
                    }

                    if let Some(k) = Self::parse_key_from_str(&key) {
                        match k {
                            KeyType::Char(ch) => {
                                let cmd = InsertChar::new(ch);
                                let _ = cmd.execute(&mut self.state);
                            }
                            KeyType::Backspace => {
                                if let Some(buf) = self.state.active_buffer_mut().ok() {
                                    if buf.cursor.column > 0 {
                                        buf.cursor.column -= 1;
                                    } else if buf.cursor.line > 0 {
                                        let prev_line = buf.cursor.line - 1;
                                        let prev_len =
                                            buf.line(prev_line).map(|s| s.len()).unwrap_or(0);
                                        buf.cursor.line = prev_line;
                                        buf.cursor.column = prev_len;
                                    }
                                    let _ = DeleteChar::new().execute(&mut self.state);
                                }
                            }
                            KeyType::Left => {
                                if let Some(buf) = self.state.active_buffer_mut().ok() {
                                    if buf.cursor.column > 0 {
                                        buf.cursor.column -= 1;
                                    } else if buf.cursor.line > 0 {
                                        let prev_line = buf.cursor.line - 1;
                                        let prev_len =
                                            buf.line(prev_line).map(|s| s.len()).unwrap_or(0);
                                        buf.cursor.line = prev_line;
                                        buf.cursor.column = prev_len;
                                    }
                                }
                            }
                            KeyType::Right => {
                                if let Some(buf) = self.state.active_buffer_mut().ok() {
                                    let line_len =
                                        buf.line(buf.cursor.line).map(|s| s.len()).unwrap_or(0);
                                    if buf.cursor.column < line_len {
                                        buf.cursor.column += 1;
                                    } else if buf.cursor.line + 1 < buf.line_count() {
                                        buf.cursor.line += 1;
                                        buf.cursor.column = 0;
                                    }
                                }
                            }
                            KeyType::Up => {
                                if let Some(buf) = self.state.active_buffer_mut().ok() {
                                    if buf.cursor.line > 0 {
                                        let new_line = buf.cursor.line - 1;
                                        let line_len =
                                            buf.line(new_line).map(|s| s.len()).unwrap_or(0);
                                        buf.cursor.line = new_line;
                                        buf.cursor.column = buf.cursor.column.min(line_len);
                                    }
                                }
                            }
                            KeyType::Down => {
                                if let Some(buf) = self.state.active_buffer_mut().ok() {
                                    if buf.cursor.line + 1 < buf.line_count() {
                                        let new_line = buf.cursor.line + 1;
                                        let line_len =
                                            buf.line(new_line).map(|s| s.len()).unwrap_or(0);
                                        buf.cursor.line = new_line;
                                        buf.cursor.column = buf.cursor.column.min(line_len);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            // After every command, write a fresh snapshot
            let snapshot = RenderState::from_state(&self.state);
            let mut w = self.render_state.write().await;
            *w = snapshot;
        }
    }
}

/// Simple enum that represents the keys we care about.
#[derive(Debug, Clone, Copy)]
enum KeyType {
    Left,
    Right,
    Up,
    Down,
    Backspace,
    Char(char),
}
