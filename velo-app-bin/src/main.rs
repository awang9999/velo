use crossterm::execute;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use std::io::stdout;
use velo_tui::Tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tui = Tui::new();

    // 1. Capture the handle returned by start_render_loop
    let render_handle = tui.start_render_loop();

    // 2. WAIT for the render loop to finish.
    // The render loop only finishes if the task is aborted
    // or if you implement a way to break the loop.
    let _ = render_handle.await;

    // 3. Now that the loop has ended, cleanup the terminal
    disable_raw_mode().ok();
    execute!(stdout(), LeaveAlternateScreen).ok(); // Corrected stdout() call

    Ok(())
}
