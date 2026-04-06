use std::collections::HashMap;
use std::sync::mpsc;

use iced::widget::column;
use iced::{Color, Element, Length, Subscription, Task, Theme};

use cratty_core::{IdGen, PaneId, Workspace, WorkspaceId};

use term_backend::TermBackend;

mod app_actions;
mod app_keys;
mod app_tick;
mod app_update;
mod home;
mod message;
mod pane;
mod sidebar;
mod strip_view;
mod style;
mod term_backend;
mod term_canvas;
mod term_colors;
mod term_widget;
mod terminal;
mod titlebar;
mod widgets;
mod ws_item;
mod ws_menu;

use message::Message;
use pane::Pane;
use widgets::PHOSPHOR_BOLD_BYTES;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "cratty=info".parse().unwrap()))
        .init();

    iced::application(Cratty::new, Cratty::update, Cratty::view)
        .subscription(Cratty::subscription)
        .title("Cratty").theme(Cratty::theme)
        .decorations(false).window_size((1200.0, 800.0))
        .antialiasing(true).font(PHOSPHOR_BOLD_BYTES)
        .run()
}

/// Pending PTY spawn -- workspace/pane created, backend arrives later.
pub struct PendingBackend {
    pub pane_id: PaneId,
    pub rx: mpsc::Receiver<anyhow::Result<TermBackend>>,
}

pub struct Cratty {
    pub workspaces: Vec<Workspace>,
    pub active_ws: usize,
    pub panes: HashMap<PaneId, Pane>,
    pub ws_colors: HashMap<WorkspaceId, Color>,
    pub id_gen: IdGen,
    pub sidebar_collapsed: bool,
    pub renaming_ws: Option<WorkspaceId>,
    pub rename_text: String,
    pub ws_menu_idx: Option<usize>,
    pub pending_backends: Vec<PendingBackend>,
}

pub fn with_window<F, T>(f: F) -> Task<T>
where
    F: Fn(iced::window::Id) -> Task<T> + Send + 'static,
    T: Send + 'static,
{
    iced::window::oldest().and_then(move |id| f(id))
}

impl Cratty {
    fn new() -> (Self, Task<Message>) {
        (Self {
            workspaces: vec![], active_ws: 0,
            panes: HashMap::new(), ws_colors: HashMap::new(),
            id_gen: IdGen::new(), sidebar_collapsed: false,
            renaming_ws: None, rename_text: String::new(), ws_menu_idx: None,
            pending_backends: vec![],
        }, Task::none())
    }

    fn view(&self) -> Element<'_, Message> {
        use iced::widget::row;
        let bar = titlebar::view_titlebar();
        let main_area: Element<Message> = match self.workspaces.get(self.active_ws) {
            Some(ws) if !ws.is_empty() => strip_view::view_strip(&ws.strip, &self.panes),
            _ => home::view_home(),
        };

        let body: Element<Message> = if self.sidebar_collapsed {
            main_area
        } else {
            let sb = sidebar::view_sidebar(
                &self.workspaces, &self.ws_colors, &self.panes,
                self.active_ws, self.renaming_ws, &self.rename_text, self.ws_menu_idx,
            );
            row![sb, main_area].width(Length::Fill).height(Length::Fill).into()
        };

        column![bar, body].width(Length::Fill).height(Length::Fill).into()
    }

    fn theme(&self) -> Theme { Theme::Dark }

    fn subscription(&self) -> Subscription<Message> { app_keys::subscription() }
}
