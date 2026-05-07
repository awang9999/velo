use crossterm::event::{self, Event as CEvent, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, Clear, ClearType, EnterAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::io::stdout;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::spawn;
use tokio::sync::{mpsc, RwLock};

pub struct Tui {
    pub buf: Arc<RwLock<String>>,
    pub status: Arc<RwLock<String>>,
    pub quit: Arc<RwLock<bool>>,
    pub tx: mpsc::Sender<()>,
}

#[derive(Clone, Copy, PartialEq)]
enum SequenceState {
    Idle,
    AfterSuperX,
}

impl Tui {
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        let status = Arc::new(RwLock::new("Welcome to Velo!".to_string()));
        let buf = Arc::new(RwLock::new("Hello Velo".to_string()));
        let quit = Arc::new(RwLock::new(false));
        let (tx, rx) = mpsc::channel(1);

        let status_for_task = Arc::clone(&status);
        let buf_for_task = Arc::clone(&buf);
        let quit_for_task = Arc::clone(&quit);
        let tx_for_task = tx.clone();

        spawn(async move {
            let _ = tx_for_task.send(()).await;
            let mut seq_state = SequenceState::Idle;
            let mut last_event = Instant::now();

            loop {
                if *quit_for_task.read().await {
                    break;
                }

                let ev = match event::read() {
                    Ok(e) => e,
                    Err(e) => {
                        let mut status_write = status_for_task.write().await;
                        *status_write = format!("Error reading event: {}", e);
                        let _ = tx_for_task.send(()).await;
                        continue;
                    }
                };

                match ev {
                    CEvent::Key(key) => {
                        // Reset state after 3 seconds of inactivity
                        if last_event.elapsed() > Duration::from_secs(3) {
                            seq_state = SequenceState::Idle;
                            let mut status_write = status_for_task.write().await;
                            *status_write = "None".to_string();
                        }
                        last_event = Instant::now();

                        // FIX: We pass dependencies in and .await the result
                        // The functions now return the NEXT state
                        seq_state = match seq_state {
                            SequenceState::Idle => {
                                process_idle_key_event(
                                    key.modifiers,
                                    key.code,
                                    &status_for_task,
                                    &buf_for_task,
                                )
                                .await
                            }
                            SequenceState::AfterSuperX => {
                                process_super_x_key_event(
                                    key.modifiers,
                                    key.code,
                                    &status_for_task,
                                    &quit_for_task,
                                    &tx_for_task,
                                )
                                .await
                            }
                        };

                        let _ = tx_for_task.send(()).await;
                    }
                    CEvent::Resize(width, height) => {
                        let mut status_write = status_for_task.write().await;
                        *status_write = format!("Window resized: {}x{}", width, height);
                        let _ = tx_for_task.send(()).await;
                    }
                    _ => {}
                }
            }
        });

        (
            Self {
                buf,
                status,
                quit,
                tx,
            },
            rx,
        )
    }

    pub fn start_render_loop(&self, mut rx: mpsc::Receiver<()>) -> tokio::task::JoinHandle<()> {
        let mut stdout = stdout();
        enable_raw_mode().expect("Failed to enable raw mode");
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))
            .expect("Failed to enter alternate screen");

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

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
                }

                let status_text = status_handle.read().await;
                let buf_text = buf_handle.read().await;

                terminal
                    .draw(|f| {
                        let area = f.area();
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
                            .split(area);

                        let block = Block::default().title("Velo").borders(Borders::ALL);
                        let paragraph = Paragraph::new(buf_text.as_str())
                            .block(block)
                            .wrap(Wrap { trim: false });
                        f.render_widget(paragraph, chunks[0]);

                        let status_paragraph = Paragraph::new(status_text.as_str())
                            .block(Block::default().borders(Borders::NONE))
                            .wrap(Wrap { trim: false });
                        f.render_widget(status_paragraph, chunks[1]);
                    })
                    .ok();
            }
        })
    }
}

// --- Logic Helpers ---
// These are now top-level functions that take what they need and return the next state.

async fn process_idle_key_event(
    modifiers: KeyModifiers,
    code: KeyCode,
    status: &Arc<RwLock<String>>,
    buf: &Arc<RwLock<String>>,
) -> SequenceState {
    match (modifiers, code) {
        (KeyModifiers::CONTROL, KeyCode::Char('x')) => {
            let mut s = status.write().await;
            *s = "C-x".to_string();
            SequenceState::AfterSuperX // Transition to next state
        }
        (_, KeyCode::Char(c)) => {
            let mut s = status.write().await;
            *s = format!("Pressed character: {}", c);
            let mut b = buf.write().await;
            b.push(c);
            SequenceState::Idle
        }
        (_, KeyCode::Enter) => {
            let mut s = status.write().await;
            *s = "Enter pressed".to_string();
            SequenceState::Idle
        }
        (_, KeyCode::Backspace) => {
            let mut s = status.write().await;
            *s = "Backspace pressed".to_string();
            let mut b = buf.write().await;
            b.pop();
            SequenceState::Idle
        }
        _ => SequenceState::Idle,
    }
}

async fn process_super_x_key_event(
    modifiers: KeyModifiers,
    code: KeyCode,
    status: &Arc<RwLock<String>>,
    quit: &Arc<RwLock<bool>>,
    tx: &mpsc::Sender<()>,
) -> SequenceState {
    match (modifiers, code) {
        (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
            let mut s = status.write().await;
            *s = "C-x C-g".to_string();
            SequenceState::Idle // Transition back to Idle
        }
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            let mut q = quit.write().await;
            *q = true;
            let _ = tx.send(()).await;
            SequenceState::Idle
        }
        _ => SequenceState::AfterSuperX,
    }
}
