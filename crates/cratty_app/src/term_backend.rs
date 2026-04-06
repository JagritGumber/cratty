use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use alacritty_terminal::event::{Event as TermEvent, EventListener, Notify, OnResize, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, EventLoopSender, Notifier};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::{self, Term};
use alacritty_terminal::tty;

/// Event proxy that sends terminal events to an mpsc channel.
#[derive(Clone)]
pub struct EventProxy(pub std::sync::mpsc::Sender<TermEvent>);

impl EventListener for EventProxy {
    fn send_event(&self, event: TermEvent) {
        let _ = self.0.send(event);
    }
}

/// Terminal dimensions for alacritty_terminal.
#[derive(Debug, Clone, Copy)]
pub struct TermSize {
    pub cols: u16,
    pub rows: u16,
}

impl Dimensions for TermSize {
    fn total_lines(&self) -> usize { self.rows as usize }
    fn screen_lines(&self) -> usize { self.rows as usize }
    fn columns(&self) -> usize { self.cols as usize }
}

/// Wraps alacritty_terminal PTY + terminal emulator.
pub struct TermBackend {
    pub term: Arc<FairMutex<Term<EventProxy>>>,
    pub sender: EventLoopSender,
    notifier: Notifier,
    pub event_rx: std::sync::mpsc::Receiver<TermEvent>,
    size: TermSize,
    pub pending_resize: Arc<Mutex<Option<(u16, u16)>>>,
}

impl TermBackend {
    pub fn new(
        shell: String, args: Vec<String>, cwd: Option<PathBuf>,
        cols: u16, rows: u16, cell_w: f32, cell_h: f32,
    ) -> anyhow::Result<Self> {
        let size = TermSize { cols, rows };
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let event_proxy = EventProxy(event_tx);

        let config = term::Config::default();
        let term = Term::new(config, &size, event_proxy.clone());
        let term = Arc::new(FairMutex::new(term));

        let pty_config = tty::Options {
            shell: Some(tty::Shell::new(shell, args)),
            working_directory: cwd,
            ..Default::default()
        };

        let window_size = WindowSize {
            num_lines: rows,
            num_cols: cols,
            cell_width: cell_w as u16,
            cell_height: cell_h as u16,
        };

        let pty = tty::new(&pty_config, window_size, 0u64)?;
        let event_loop = EventLoop::new(
            term.clone(), event_proxy, pty, false, false,
        )?;
        let sender = event_loop.channel();
        let notifier = Notifier(sender.clone());
        let _join = event_loop.spawn();

        let pending_resize = Arc::new(Mutex::new(None));
        Ok(Self { term, sender, notifier, event_rx, size, pending_resize })
    }

    pub fn write(&self, data: &[u8]) {
        let _ = self.notifier.notify(data.to_vec());
    }

    pub fn resize(&mut self, cols: u16, rows: u16, cell_w: f32, cell_h: f32) {
        self.size = TermSize { cols, rows };
        let window_size = WindowSize {
            num_lines: rows, num_cols: cols,
            cell_width: cell_w as u16, cell_height: cell_h as u16,
        };
        let _ = self.notifier.on_resize(window_size);
        self.term.lock().resize(self.size);
    }

    pub fn apply_pending_resize(&mut self, cell_w: f32, cell_h: f32) {
        let pending = self.pending_resize.lock().unwrap().take();
        if let Some((cols, rows)) = pending {
            if cols != self.size.cols || rows != self.size.rows {
                self.resize(cols, rows, cell_w, cell_h);
            }
        }
    }

    pub fn drain_events(&self) -> Vec<TermEvent> {
        let mut events = Vec::new();
        while let Ok(ev) = self.event_rx.try_recv() {
            events.push(ev);
        }
        events
    }
}
