use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, Clear, ClearType, EnterAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tokio::sync::mpsc;

/// All the state that can change during the app's lifetime.
struct EditorState {
    buf: String,
    status: String,
    quit: bool,
    seq_state: SequenceState,
    last_event_time: Instant,
}

#[derive(Clone, Copy, PartialEq)]
enum SequenceState {
    Idle,
    AfterSuperX,
}

pub struct Tui {
    pub state: Arc<Mutex<EditorState>>,
    pub tx: mpsc::Sender<()>,
}

impl Tui {
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        let (tx, rx) = mpsc::channel(1);

        let state = Arc::new(Mutex::new(EditorState {
            buf: "Hello Velo".to_string(),
            status: "Welcome to Velo!".to_string(),
            quit: false,
            seq_state: SequenceState::Idle,
            last_event_time: Instant::now(),
        }));

        let state_for_input = Arc::clone(&state);
        let tx_for_input = tx.clone();

        // --- 1. SYNCHRONOUS INPUT THREAD ---
        // We use tokio::task::spawn_blocking instead of tokio::spawn because event::read() blocks.
        tokio::task::spawn_blocking(move || {
            loop {
                // Blocking call: this thread sleeps until a key is pressed
                match event::read() {
                    Ok(CEvent::Key(key)) => {
                        let mut s = state_for_input.lock().unwrap();

                        // Handle timeout (reset state if user waits > 3s)
                        if s.last_event_time.elapsed() > Duration::from_secs(3) {
                            s.seq_state = SequenceState::Idle;
                            s.status = "None".to_string();
                        }
                        s.last_event_time = Instant::now();

                        // Run the state machine synchronously
                        process_key_event(&mut s, key);

                        // Signal the render loop to wake up
                        // blocking_send is used because we are in a synchronous thread
                        let _ = tx_for_input.blocking_send(());

                        if s.quit {
                            break;
                        }
                    }
                    Ok(CEvent::Resize(w, h)) => {
                        let mut s = state_for_input.lock().unwrap();
                        s.status = format!("Window resized: {}x{}", w, h);
                        let _ = tx_for_input.blocking_send(());
                    }
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error reading event: {}", e);
                    }
                }
            }
        });

        (Self { state, tx }, rx)
    }

    pub fn start_render_loop(&self, mut rx: mpsc::Receiver<()>) -> tokio::task::JoinHandle<()> {
        // Set up terminal for Raw Input
        let mut stdout = stdout();
        enable_raw_mode().expect("Failed to enable raw mode");
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))
            .expect("Failed to enter alternate screen");

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

        let state_handle = Arc::clone(&self.state);

        tokio::spawn(async move {
            // 1. Define the drawing logic so we can reuse it
            let draw_frame = |terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
                              state_handle: &Arc<Mutex<EditorState>>| {
                let (buf_text, status_text, _) = {
                    let s = state_handle.lock().unwrap();
                    (s.buf.clone(), s.status.clone(), s.quit)
                };

                terminal
                    .draw(|f| {
                        let area = f.area();
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
                            .split(area);

                        f.render_widget(
                            Paragraph::new(buf_text.as_str())
                                .block(Block::default().title("Velo").borders(Borders::ALL))
                                .wrap(Wrap { trim: false }),
                            chunks[0],
                        );

                        f.render_widget(
                            Paragraph::new(status_text.as_str())
                                .block(Block::default().borders(Borders::NONE)),
                            chunks[1],
                        );
                    })
                    .ok();
            };

            // 2. DRAW IMMEDIATELY ON STARTUP
            draw_frame(&mut terminal, &state_handle);

            // 3. Now enter the loop to wait for updates
            loop {
                if rx.recv().await.is_none() {
                    break;
                }

                // Check for quit
                if state_handle.lock().unwrap().quit {
                    break;
                }

                // Draw again
                draw_frame(&mut terminal, &state_handle);
            }
        })
    }
}

// --- 2. SYNCHRONOUS STATE MACHINE ---
// This is a "pure" logic function. No async, no Arc, no Mutexes inside.
// It just takes a mutable reference to the state and the key.
fn process_key_event(s: &mut EditorState, key: KeyEvent) {
    match s.seq_state {
        SequenceState::Idle => match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('x')) => {
                s.status = "C-x".to_string();
                s.seq_state = SequenceState::AfterSuperX;
            }
            (_, KeyCode::Char(c)) => {
                s.status = format!("Pressed character: {}", c);
                s.buf.push(c);
            }
            (_, KeyCode::Enter) => {
                s.status = "Enter pressed".to_string();
            }
            (_, KeyCode::Backspace) => {
                s.status = "Backspace pressed".to_string();
                s.buf.pop();
            }
            _ => {}
        },
        SequenceState::AfterSuperX => {
            match (key.modifiers, key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
                    s.status = "C-x C-g".to_string();
                    s.seq_state = SequenceState::Idle;
                }
                (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                    s.status = "C-x C-s - file saved".to_string();
                    s.seq_state = SequenceState::Idle;
                }
                (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
                    s.status = "C-x C-f - open file".to_string();
                    s.seq_state = SequenceState::Idle;
                }
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                    s.quit = true;
                }
                _ => {
                    // Any other key resets the chord
                    s.seq_state = SequenceState::Idle;
                }
            }
        }
    }
}
