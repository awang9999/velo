// Graphical UI implementation (future)

use velo_app::App;

use std::sync::Arc;
use tokio::sync::mpsc::channel;
use tokio::sync::RwLock;

use velo_core::{EditorState, RenderState};
use velo_types::EditorEvent;

pub struct Gui {
    pub app: App,
}

impl Gui {
    pub fn new() -> Self {
        // 1️⃣  Dummy input channel
        let (_tx, input_rx) = channel::<EditorEvent>(1);

        // 2️⃣  Dummy render‑state
        let render_state = Arc::new(RwLock::new(RenderState::from_state(&EditorState::new())));

        // 3️⃣  Create the App
        let app = App::new_with_input_rx_and_render_state(input_rx, render_state);

        Self { app }
    }
}

impl Default for Gui {
    fn default() -> Self {
        Self::new()
    }
}
