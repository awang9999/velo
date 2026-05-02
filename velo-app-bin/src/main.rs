use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::io::stdout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Prepare the terminal.
    let mut stdout = stdout();
    enable_raw_mode().expect("Failed to enable raw mode");
    execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))
        .expect("Failed to enter alternate screen");

    let backend = CrosstermBackend::new(&stdout);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

    // Calculate string
    let mut buf = "Hello Velo".to_string();
    let mut status_line = "Welcome to Velo!".to_string();

    loop {
        // Parse events
        // Non‑blocking poll – returns immediately if no event is pending.
        if event::poll(std::time::Duration::from_millis(200))? {
            // Read the event that was just polled.
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char(c) => {
                        status_line = format!("Pressed character: {}", c).to_string();
                        buf.push(c);
                    }
                    KeyCode::Enter => {
                        status_line = "Enter pressed".to_string();
                    }
                    KeyCode::Backspace => {
                        status_line = "Backspace pressed".to_string();
                        buf.pop();
                    }
                    KeyCode::Esc => {
                        // Quit key
                        status_line = "Escape pressed".to_string();
                        break;
                    }
                    _ => {}
                },
                Event::Mouse(_) => { /* handle mouse if needed */ }
                Event::Resize(width, height) => {
                    status_line = format!("Window resized: {}x{}", width, height).to_string();
                }
                Event::FocusGained | Event::FocusLost | Event::Paste(_) => todo!(),
            }
        }

        // Render
        terminal
            .draw(|f| {
                let area = f.area();

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
                    .split(area);

                let block = Block::default().title("Velo").borders(Borders::ALL);
                let paragraph = Paragraph::new(buf.clone())
                    .block(block)
                    .wrap(Wrap { trim: false });
                f.render_widget(paragraph, chunks[0]);

                let status_block = Block::default().borders(Borders::NONE);
                let status_paragraph = Paragraph::new(status_line.clone())
                    .block(status_block)
                    .wrap(Wrap { trim: false });
                f.render_widget(status_paragraph, chunks[1]);
            })
            .ok();
    }

    // Cleanup terminal
    disable_raw_mode().ok();
    execute!(&stdout, LeaveAlternateScreen).ok();

    Ok(())
}
