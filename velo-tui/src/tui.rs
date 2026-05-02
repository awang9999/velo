use std::io::stdout;
use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{read, Event as CEvent};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, Clear, ClearType, EnterAlternateScreen};

use ratatui::backend::CrosstermBackend;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::RwLock;
use tokio::{spawn, time::sleep};

use velo_app::App;
use velo_core::EditorState;
use velo_core::RenderState;
use velo_types::EditorEvent;

/// The main UI struct.
/// `app` owns the shared editor state.
/// `input_tx` is used to send raw input events back to the core.
/// `render_state` is shared with the core for snapshotting.
pub struct Tui {
    pub app: App,
    input_tx: Sender<EditorEvent>,
    pub render_state: Arc<RwLock<RenderState>>,
    pub status: Arc<RwLock<String>>,
}

impl Tui {
    /// Create a new TUI instance.
    /// A bounded channel is created for sending input events to the core.
    /// The channel receiver is handed to the `App` constructor.
    /// An async task is spawned to poll the terminal and forward events.
    pub fn new() -> Self {
        // Channel used to send `EditorEvent` values from the UI to the core.
        let (input_tx, input_rx) = channel::<EditorEvent>(100);

        // Shared render state published by the core
        let render_state = Arc::new(RwLock::new(RenderState::from_state(&EditorState::new())));

        // Create the application shell with the receiver side of the channel.
        let app = App::new_with_input_rx_and_render_state(input_rx, render_state.clone());

        let status = Arc::new(RwLock::new(String::new()));

        // Clone the sender so the polling task can own it.
        let input_tx_clone = input_tx.clone();

        // Spawn an async task that continuously polls for terminal events.
        spawn(async move {
            loop {
                match read() {
                    Ok(CEvent::Key(key_event)) => {
                        let key_str = format!("{:?}", key_event.code);
                        let ev = EditorEvent::KeyPressed {
                            key: key_str,
                            modifiers: Vec::new(),
                        };
                        if input_tx_clone.send(ev).await.is_err() {
                            break;
                        }
                    }
                    Ok(CEvent::Mouse(_)) | Ok(CEvent::Resize(_, _)) | Err(_) => {}
                    Ok(crossterm::event::Event::FocusGained)
                    | Ok(crossterm::event::Event::FocusLost)
                    | Ok(crossterm::event::Event::Paste(_)) => todo!(),
                }
                sleep(Duration::from_millis(10)).await;
            }
        });

        Self {
            app,
            input_tx,
            render_state,
            status,
        }
    }

    /// Return a reference to the shared `App` instance.
    pub fn app(&self) -> &App {
        &self.app
    }

    /// Return a mutable reference to the shared `App` instance.
    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }

    /// Start the render loop.
    /// The render loop reads snapshots from the shared `render_state` and renders them.
    pub fn start_render_loop(&self) -> tokio::task::JoinHandle<()> {
        let render_state_clone = self.render_state.clone();
        // Prepare the terminal.
        let mut stdout = stdout();
        enable_raw_mode().expect("Failed to enable raw mode");
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))
            .expect("Failed to enter alternate screen");
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

        let status_clone = self.status.clone();

        spawn(async move {
            loop {
                let snapshot = render_state_clone.read().await;

                let status_guard = status_clone.read().await; // acquire the lock
                let status_text = (*status_guard).clone(); // clone the inner String

                terminal
                    .draw(|f| {
                        let area = f.area();

                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
                            .split(area);

                        let block = Block::default().title("Velo").borders(Borders::ALL);
                        let paragraph = Paragraph::new::<&str>(snapshot.buffer_text.as_ref())
                            .block(block)
                            .wrap(Wrap { trim: false });
                        f.render_widget(paragraph, chunks[0]);

                        // Bottom status line
                        let status_block = Block::default().borders(Borders::NONE);
                        let status_paragraph = Paragraph::new(status_text)
                            .block(status_block)
                            .wrap(Wrap { trim: false });
                        f.render_widget(status_paragraph, chunks[1]);
                    })
                    .ok();

                sleep(Duration::from_millis(50)).await;
            }
        })
    }
}
