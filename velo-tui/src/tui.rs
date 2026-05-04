use std::io::stdout;
use std::sync::Arc;
// use std::time::Duration;

use crossterm::event::{self, Event as CEvent, KeyCode};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, Clear, ClearType, EnterAlternateScreen};

use ratatui::backend::CrosstermBackend;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tokio::spawn;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, Duration}; // Add this import

// use velo_app::App;
// use velo_core::EditorState;
// use velo_core::RenderState;
// use velo_types::EditorEvent;

/// The main UI struct.
/// `app` owns the shared editor state.
/// `input_tx` is used to send raw input events back to the core.
/// `render_state` is shared with the core for snapshotting.
pub struct Tui {
    // pub app: App,
    // input_tx: Sender<EditorEvent>,
    // pub render_state: Arc<RwLock<RenderState>>,
    pub buf: Arc<RwLock<String>>,
    pub status: Arc<RwLock<String>>,
    pub quit: Arc<RwLock<bool>>,
    pub tx: mpsc::Sender<()>,
}

impl Tui {
    /// Create a new TUI instance.
    /// A bounded channel is created for sending input events to the core.
    /// The channel receiver is handed to the `App` constructor.
    /// An async task is spawned to poll the terminal and forward events.
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        // Channel used to send `EditorEvent` values from the UI to the core.
        // let (input_tx, input_rx) = channel::<EditorEvent>(100);

        // Shared render state published by the core
        // let render_state = Arc::new(RwLock::new(RenderState::from_state(&EditorState::new())));

        // Create the application shell with the receiver side of the channel.
        // let app = App::new_with_input_rx_and_render_state(input_rx, render_state.clone());

        // Clone the sender so the polling task can own it.
        // let input_tx_clone = input_tx.clone();

        let status = Arc::new(RwLock::new("Welcome to Velo!".to_string()));
        let buf = Arc::new(RwLock::new("Hello Velo".to_string()));
        let quit = Arc::new(RwLock::new(false));
        let (tx, rx) = mpsc::channel(1);

        let status_for_task = Arc::clone(&status);
        let buf_for_task = Arc::clone(&buf);
        let quit_for_task = Arc::clone(&quit);
        let tx_for_task = tx.clone();

        // Spawn an async task that continuously polls for terminal events.
        spawn(async move {
            // Send first event to render channel
            let _ = tx_for_task.send(()).await;

            loop {
                // 1. Try to read the event first
                let ev = match event::read() {
                    Ok(e) => e,
                    Err(e) => {
                        let mut status_write = status_for_task.write().await;
                        *status_write = format!("Error reading event: {}", e);
                        let _ = tx_for_task.send(()).await;
                        continue; // Skip this iteration and try again
                    }
                };

                // Parse events
                // Read the event that was just polled.
                match ev {
                    CEvent::Key(key) => match key.code {
                        KeyCode::Char(c) => {
                            let mut status_write = status_for_task.write().await;
                            *status_write = format!("Pressed character: {}", c);
                            let mut buf_write = buf_for_task.write().await;
                            buf_write.push(c);

                            let _ = tx_for_task.send(()).await;
                        }
                        KeyCode::Enter => {
                            let mut status_write = status_for_task.write().await;
                            *status_write = "Enter pressed".to_string();
                            let _ = tx_for_task.send(()).await;
                        }
                        KeyCode::Backspace => {
                            let mut status_write = status_for_task.write().await;
                            *status_write = "Backspace pressed".to_string();
                            let mut buf_write = buf_for_task.write().await;
                            buf_write.pop();
                            let _ = tx_for_task.send(()).await;
                        }
                        KeyCode::Esc => {
                            // Quit key
                            let mut quit_write = quit_for_task.write().await;
                            *quit_write = true;
                            let _ = tx_for_task.send(()).await;
                            break;
                        }
                        _ => {}
                    },
                    CEvent::Mouse(_) => { /* handle mouse if needed */ }
                    CEvent::Resize(width, height) => {
                        let mut status_write = status_for_task.write().await;
                        *status_write = format!("Window resized: {}x{}", width, height);
                        let _ = tx_for_task.send(()).await;
                    }
                    CEvent::FocusGained | CEvent::FocusLost | CEvent::Paste(_) => todo!(),
                }
            }
        });

        (
            Self {
                // app,
                // input_tx,
                // render_state,
                buf,
                status,
                quit,
                tx,
            },
            rx,
        )
    }

    // /// Return a reference to the shared `App` instance.
    // pub fn app(&self) -> &App {
    //     &self.app
    // }

    // /// Return a mutable reference to the shared `App` instance.
    // pub fn app_mut(&mut self) -> &mut App {
    //     &mut self.app
    // }

    /// Start the render loop.
    /// The render loop reads snapshots from the shared `render_state` and renders them.
    pub fn start_render_loop(&self, mut rx: mpsc::Receiver<()>) -> tokio::task::JoinHandle<()> {
        // let render_state_clone = self.render_state.clone();
        // Prepare the terminal.
        let mut stdout = stdout();
        enable_raw_mode().expect("Failed to enable raw mode");
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))
            .expect("Failed to enter alternate screen");
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

        // let status_clone = self.status.clone();
        let status_handle = Arc::clone(&self.status);
        let buf_handle = Arc::clone(&self.buf);
        let quit_handle = Arc::clone(&self.quit);

        spawn(async move {
            loop {
                if rx.recv().await.is_none() {
                    break;
                }

                if *quit_handle.read().await {
                    break;
                };

                // let snapshot = render_state_clone.read().await;

                let status_text = status_handle.read().await;
                let buf_text = buf_handle.read().await;

                terminal
                    .draw(|f| {
                        let area = f.area();

                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
                            .split(area);

                        // Focused buffer text
                        let block = Block::default().title("Velo").borders(Borders::ALL);

                        let paragraph = Paragraph::new(buf_text.as_str())
                            .block(block)
                            .wrap(Wrap { trim: false });
                        f.render_widget(paragraph, chunks[0]);

                        // Status line
                        let status_block = Block::default().borders(Borders::NONE);
                        let status_paragraph = Paragraph::new(status_text.as_str())
                            .block(status_block)
                            .wrap(Wrap { trim: false });
                        f.render_widget(status_paragraph, chunks[1]);
                    })
                    .ok();

                sleep(Duration::from_millis(33)).await;
            }
        })
    }
}
