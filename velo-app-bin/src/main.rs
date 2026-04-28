use crossterm::execute;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use std::env;
use std::fs::read_to_string;
use std::io::stdout;
use std::path::PathBuf;
use std::process::exit;

use velo_core::RenderState;
use velo_core::buffer::Buffer;
use velo_tui::Tui;
use velo_types::Position;

#[tokio::main]
async fn main() {
    // Optional file argument
    let path_opt = env::args().nth(1);
    let mut initial_text = String::new();

    if let Some(path) = path_opt {
        let p = PathBuf::from(&path);
        match read_to_string(&p) {
            Ok(txt) => initial_text = txt,
            Err(e) => {
                eprintln!("failed to read {}: {}", path, e);
                exit(1);
            }
        }
    }

    // Create UI + core
    let mut tui = Tui::new();

    // If a file was provided, load it into the first buffer
    if !initial_text.is_empty() {
        let mut buf = Buffer::from_text(&initial_text);
        buf.cursor = Position::new(0, 0);
        tui.app_mut().state.open_buffer(buf);
        // Write initial snapshot
        let snapshot = RenderState::from_state(&tui.app().state);
        let mut w = tui.render_state.write().await;
        *w = snapshot;
    }

    // Start rendering
    let _render_handle = tui.start_render_loop();

    // Run the event loop
    tui.app_mut().run().await;

    // Cleanup terminal
    disable_raw_mode().ok();
    execute!(stdout(), LeaveAlternateScreen).ok();
}
