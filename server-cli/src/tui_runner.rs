use crate::{cmd, logging::LOG, Message};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::Duration,
};
use tracing::{debug, error, warn};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

pub struct Tui {
    pub msg_r: mpsc::Receiver<Message>,
    background: Option<std::thread::JoinHandle<()>>,
    basic: bool,
    running: Arc<AtomicBool>,
}

impl Tui {
    fn handle_events(input: &mut String, msg_s: &mut mpsc::Sender<Message>) {
        use crossterm::event::*;
        if let Event::Key(event) = read().unwrap() {
            match event.code {
                KeyCode::Char('c') => {
                    if event.modifiers.contains(KeyModifiers::CONTROL) {
                        msg_s.send(Message::Quit).unwrap()
                    } else {
                        input.push('c');
                    }
                },
                KeyCode::Char(c) => input.push(c),
                KeyCode::Backspace => {
                    input.pop();
                },
                KeyCode::Enter => {
                    debug!(?input, "tui mode: command entered");
                    cmd::parse_command(input, msg_s);

                    *input = String::new();
                },
                _ => {},
            }
        }
    }

    pub fn run(basic: bool) -> Self {
        let (mut msg_s, msg_r) = mpsc::channel();
        let running = Arc::new(AtomicBool::new(true));
        let running2 = Arc::clone(&running);

        let background = if basic {
            std::thread::spawn(move || {
                while running2.load(Ordering::Relaxed) {
                    let mut line = String::new();

                    match io::stdin().read_line(&mut line) {
                        Err(e) => {
                            error!(
                                ?e,
                                "couldn't read from stdin, cli commands are disabled now!"
                            );
                            break;
                        },
                        Ok(0) => {
                            //Docker seem to send EOF all the time
                            warn!("EOF received, cli commands are disabled now!");
                            break;
                        },
                        Ok(_) => {
                            debug!(?line, "basic mode: command entered");
                            crate::cmd::parse_command(&line, &mut msg_s);
                        },
                    }
                }
            });

            None
        } else {
            Some(std::thread::spawn(move || {
                // Start the tui
                let mut stdout = io::stdout();
                execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

                enable_raw_mode().unwrap();

                let backend = CrosstermBackend::new(stdout);
                let mut terminal = Terminal::new(backend).unwrap();

                let mut input = String::new();

                if let Err(e) = terminal.clear() {
                    error!(?e, "couldn't clean terminal");
                };

                while running2.load(Ordering::Relaxed) {
                    if let Err(e) = terminal.draw(|f| {
                        let (log_rect, input_rect) = if f.size().height > 6 {
                            let mut log_rect = f.size();
                            log_rect.height -= 3;

                            let mut input_rect = f.size();
                            input_rect.y = input_rect.height - 3;
                            input_rect.height = 3;

                            (log_rect, input_rect)
                        } else {
                            (f.size(), Rect::default())
                        };

                        let block = Block::default().borders(Borders::ALL);

                        let wrap = Wrap {
                            scroll_callback: Some(Box::new(|text_area, lines| {
                                LOG.resize(text_area.height as usize);
                                let len = lines.len() as u16;
                                (len.saturating_sub(text_area.height), 0)
                            })),
                            ..Default::default()
                        };

                        let logger = Paragraph::new(LOG.inner.lock().unwrap().clone())
                            .block(block)
                            .wrap(wrap);
                        f.render_widget(logger, log_rect);

                        let text: Text = input.as_str().into();

                        let block = Block::default().borders(Borders::ALL);
                        let size = block.inner(input_rect);

                        let x = (size.x + text.width() as u16).min(size.width);

                        let input_field = Paragraph::new(text).block(block);
                        f.render_widget(input_field, input_rect);

                        f.set_cursor(x, size.y);
                    }) {
                        warn!(?e, "couldn't draw frame");
                    };
                    if crossterm::event::poll(Duration::from_millis(100)).unwrap() {
                        Self::handle_events(&mut input, &mut msg_s);
                    };
                }
            }))
        };

        Self {
            msg_r,
            background,
            basic,
            running,
        }
    }

    pub fn shutdown(basic: bool) {
        if !basic {
            let mut stdout = io::stdout();
            execute!(stdout, LeaveAlternateScreen, DisableMouseCapture).unwrap();
            disable_raw_mode().unwrap();
        }
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.background.take().map(|m| m.join());
        Tui::shutdown(self.basic);
    }
}
