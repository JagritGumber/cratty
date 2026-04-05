use std::collections::HashMap;
use std::path::PathBuf;

use iced::keyboard;
use iced::widget::column;
use iced::window;
use iced::{event, Color, Element, Length, Subscription, Task, Theme};

use cratty_core::{IdGen, PaneId, Workspace, WorkspaceId};

mod home;
mod message;
mod pane;
mod sidebar;
mod strip_view;
mod style;
mod term_backend;
mod term_canvas;
mod term_widget;
mod terminal;
mod titlebar;
mod widgets;

use message::Message;
use pane::Pane;
use term_backend::TermBackend;
use widgets::PHOSPHOR_BOLD_BYTES;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cratty=info".parse().unwrap()),
        )
        .init();

    iced::application(Cratty::new, Cratty::update, Cratty::view)
        .subscription(Cratty::subscription)
        .title("Cratty")
        .theme(Cratty::theme)
        .decorations(false)
        .window_size((1200.0, 800.0))
        .antialiasing(true)
        .font(PHOSPHOR_BOLD_BYTES)
        .run()
}

struct Cratty {
    workspaces: Vec<Workspace>,
    active_ws: usize,
    panes: HashMap<PaneId, Pane>,
    ws_colors: HashMap<WorkspaceId, Color>,
    id_gen: IdGen,
}

fn with_window<F, T>(f: F) -> Task<T>
where
    F: Fn(window::Id) -> Task<T> + Send + 'static,
    T: Send + 'static,
{
    window::oldest().and_then(move |id| f(id))
}

impl Cratty {
    fn new() -> (Self, Task<Message>) {
        (Self {
            workspaces: vec![],
            active_ws: 0,
            panes: HashMap::new(),
            ws_colors: HashMap::new(),
            id_gen: IdGen::new(),
        }, Task::none())
    }

    fn create_workspace(&mut self, cwd: Option<PathBuf>) -> Task<Message> {
        let ws_id = self.id_gen.next_workspace();
        let pane_id = self.id_gen.next_pane();
        let (shell, args) = terminal::default_shell();

        match TermBackend::new(shell, args, cwd, 80, 24, 8, 16) {
            Ok(backend) => {
                self.panes.insert(pane_id, Pane::new(pane_id, backend));
                let mut ws = Workspace::new(ws_id, format!("Terminal {}", ws_id.0));
                ws.strip.push(pane_id);
                self.workspaces.push(ws);
                self.active_ws = self.workspaces.len() - 1;
                Task::none()
            }
            Err(e) => {
                tracing::error!("Failed to create terminal: {e}");
                Task::none()
            }
        }
    }

    fn add_pane_to_active_workspace(&mut self, cwd: Option<PathBuf>) -> Task<Message> {
        let ws = match self.workspaces.get_mut(self.active_ws) {
            Some(ws) => ws,
            None => return self.create_workspace(cwd),
        };
        let pane_id = self.id_gen.next_pane();
        let (shell, args) = terminal::default_shell();

        match TermBackend::new(shell, args, cwd, 80, 24, 8, 16) {
            Ok(backend) => {
                self.panes.insert(pane_id, Pane::new(pane_id, backend));
                ws.strip.push(pane_id);
                Task::none()
            }
            Err(_) => Task::none(),
        }
    }

    fn close_workspace(&mut self, idx: usize) -> Task<Message> {
        if idx >= self.workspaces.len() {
            return Task::none();
        }
        let active_id = self.workspaces.get(self.active_ws).map(|w| w.id);
        let ws = self.workspaces.remove(idx);
        for pane_id in &ws.strip.panes {
            self.panes.remove(pane_id);
        }
        self.ws_colors.remove(&ws.id);
        if self.workspaces.is_empty() {
            self.active_ws = 0;
            return Task::none();
        }
        self.active_ws = active_id
            .and_then(|id| self.workspaces.iter().position(|w| w.id == id))
            .unwrap_or(self.active_ws.min(self.workspaces.len() - 1));
        Task::none()
    }

    fn remove_pane(&mut self, pid: PaneId) -> Task<Message> {
        self.panes.remove(&pid);
        let active_id = self.workspaces.get(self.active_ws).map(|w| w.id);
        for ws in &mut self.workspaces {
            ws.strip.remove(pid);
        }
        let empty_ids: Vec<WorkspaceId> = self.workspaces.iter()
            .filter(|ws| ws.is_empty()).map(|ws| ws.id).collect();
        for id in &empty_ids {
            self.ws_colors.remove(id);
        }
        self.workspaces.retain(|ws| !ws.is_empty());
        self.active_ws = active_id
            .and_then(|id| self.workspaces.iter().position(|w| w.id == id))
            .unwrap_or(self.active_ws.min(
                self.workspaces.len().saturating_sub(1),
            ));
        Task::none()
    }

    fn process_terminal_events(&mut self) {
        let pane_ids: Vec<PaneId> = self.panes.keys().copied().collect();
        for pid in pane_ids {
            let events = if let Some(pane) = self.panes.get(&pid) {
                pane.backend.drain_events()
            } else {
                continue;
            };
            for ev in events {
                match ev {
                    alacritty_terminal::event::Event::Title(title) => {
                        if let Some(pane) = self.panes.get_mut(&pid) {
                            pane.title = title;
                        }
                    }
                    alacritty_terminal::event::Event::Exit => {
                        self.panes.remove(&pid);
                        for ws in &mut self.workspaces {
                            ws.strip.remove(pid);
                        }
                    }
                    _ => {}
                }
            }
        }
        // Clean up empty workspaces
        self.workspaces.retain(|ws| !ws.is_empty());
        if self.active_ws >= self.workspaces.len() && !self.workspaces.is_empty() {
            self.active_ws = self.workspaces.len() - 1;
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewWorkspace => self.create_workspace(None),
            Message::CloseWorkspace(idx) => self.close_workspace(idx),
            Message::SwitchWorkspace(idx) => {
                if idx >= self.workspaces.len() { return Task::none(); }
                self.active_ws = idx;
                Task::none()
            }
            Message::NewPane => self.add_pane_to_active_workspace(None),
            Message::ClosePane(pid) => self.remove_pane(pid),
            Message::FocusPaneLeft => {
                if let Some(ws) = self.workspaces.get_mut(self.active_ws) {
                    ws.strip.focus_left();
                }
                Task::none()
            }
            Message::FocusPaneRight => {
                if let Some(ws) = self.workspaces.get_mut(self.active_ws) {
                    ws.strip.focus_right();
                }
                Task::none()
            }
            Message::EscapePressed => Task::none(),
            Message::Tick => {
                self.process_terminal_events();
                Task::none()
            }
            Message::DragWindow => with_window(window::drag),
            Message::Minimize => with_window(|id| window::minimize(id, true)),
            Message::Maximize => with_window(window::toggle_maximize),
            Message::CloseWindow => with_window(window::close),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        use iced::widget::row;
        let bar = titlebar::view_titlebar();

        let main_area: Element<Message> = if let Some(ws) = self.workspaces.get(self.active_ws) {
            if ws.is_empty() {
                home::view_home()
            } else {
                strip_view::view_strip(&ws.strip, &self.panes)
            }
        } else {
            home::view_home()
        };

        let sidebar = sidebar::view_sidebar(
            &self.workspaces, &self.ws_colors, &self.panes, self.active_ws,
        );

        let body: Element<Message> = row![sidebar, main_area]
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        column![bar, body].width(Length::Fill).height(Length::Fill).into()
    }

    fn theme(&self) -> Theme { Theme::Dark }

    fn subscription(&self) -> Subscription<Message> {
        let key_sub = event::listen_with(|evt, _status, _window| {
            if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key, modifiers, ..
            }) = evt {
                if key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                    return Some(Message::EscapePressed);
                }
                if modifiers.control() && modifiers.shift() {
                    if let keyboard::Key::Character(c) = &key {
                        if c.as_str().eq_ignore_ascii_case("t") {
                            return Some(Message::NewWorkspace);
                        }
                        if c.as_str().eq_ignore_ascii_case("n") {
                            return Some(Message::NewPane);
                        }
                    }
                    if let keyboard::Key::Named(named) = &key {
                        match named {
                            keyboard::key::Named::ArrowLeft =>
                                return Some(Message::FocusPaneLeft),
                            keyboard::key::Named::ArrowRight =>
                                return Some(Message::FocusPaneRight),
                            _ => {}
                        }
                    }
                }
            }
            None
        });
        let tick_sub = iced::time::every(std::time::Duration::from_millis(16))
            .map(|_| Message::Tick);
        Subscription::batch([key_sub, tick_sub])
    }
}
