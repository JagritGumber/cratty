use std::path::PathBuf;
use std::sync::Arc;

use alacritty_terminal::event::{Event as TermEvent, EventListener, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, Notifier};
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
    pub notifier: Notifier,
    pub event_rx: std::sync::mpsc::Receiver<TermEvent>,
    size: TermSize,
}

impl TermBackend {
    pub fn new(
        shell: String, args: Vec<String>, cwd: Option<PathBuf>,
        cols: u16, rows: u16, cell_w: u16, cell_h: u16,
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
            cell_width: cell_w,
            cell_height: cell_h,
        };

        let pty = tty::new(&pty_config, window_size, 0u64)?;
        let event_loop = EventLoop::new(
            term.clone(), event_proxy, pty, false, false,
        )?;
        let notifier = Notifier(event_loop.channel());
        let _join = event_loop.spawn();

        Ok(Self { term, notifier, event_rx, size })
    }

    pub fn write(&self, data: &[u8]) {
        let _ = self.notifier.notify(data.to_vec());
    }

    pub fn resize(&mut self, cols: u16, rows: u16, cell_w: u16, cell_h: u16) {
        self.size = TermSize { cols, rows };
        let window_size = WindowSize {
            num_lines: rows, num_cols: cols,
            cell_width: cell_w, cell_height: cell_h,
        };
        let _ = self.notifier.on_resize(window_size);
        self.term.lock().resize(self.size);
    }

    pub fn drain_events(&self) -> Vec<TermEvent> {
        let mut events = Vec::new();
        while let Ok(ev) = self.event_rx.try_recv() {
            events.push(ev);
        }
        events
    }
}
