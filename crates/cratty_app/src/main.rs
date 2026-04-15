use std::{collections::{HashMap, HashSet}, sync::mpsc, time::Instant};
use iced::widget::{column, container, row, text};
use iced::{Color, Element, Length, Subscription, Task, Theme};
use cratty_core::{AppConfig, FontMetrics, IdGen, PaneId, Workspace, WorkspaceId};
use term_backend::TermBackend;
mod app_actions; mod app_keys; mod app_persist; mod app_picker; mod app_tick; mod app_update; mod app_ws_ops;
mod clipboard; mod clipboard_ops; mod code_editor; mod editor_widget; mod file_picker; mod file_picker_view;
mod home; mod layers; mod message;
mod lsp;
mod pane; mod quake; mod sidebar; mod strip_view; mod style;
mod term_backend; mod term_canvas; mod term_colors; mod term_cursor;
mod term_decor; mod term_palette; mod term_input; mod term_scroll;
mod term_widget; mod terminal; mod titlebar; mod widgets; mod ws_item; mod ws_menu;
use file_picker::FilePickerState;
use message::Message; use pane::Pane; use widgets::PHOSPHOR_BOLD_BYTES;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "cratty=info".parse().unwrap()))
        .init();
    alacritty_terminal::tty::setup_env();
    iced::application(Cratty::new, Cratty::update, Cratty::view)
        .subscription(Cratty::subscription)
        .title("Cratty").theme(Cratty::theme).window(window_settings())
        .antialiasing(true).font(PHOSPHOR_BOLD_BYTES)
        .run()
}

fn window_settings() -> iced::window::Settings {
    iced::window::Settings {
        size: iced::Size::new(1200.0, 800.0), decorations: false,
        transparent: false, ..Default::default()
    }
}

pub struct PendingBackend { pub pane_id: PaneId, pub rx: mpsc::Receiver<anyhow::Result<TermBackend>> }
pub struct PendingLspInstall {
    pub workspace_id: WorkspaceId,
    pub language: code_editor::CodeLanguage,
    pub rx: mpsc::Receiver<anyhow::Result<()>>,
}

pub struct WorkspaceLspServer {
    pub session: lsp::LspSession,
    pub status: lsp::LspStatus,
    pub last_stderr: Option<String>,
}

pub struct WorkspaceLspState {
    pub servers: HashMap<code_editor::CodeLanguage, WorkspaceLspServer>,
    pub open_docs: HashMap<code_editor::CodeLanguage, HashSet<String>>,
    pub statuses: HashMap<code_editor::CodeLanguage, lsp::LspStatus>,
}

impl WorkspaceLspState {
    fn new() -> Self {
        Self {
            servers: HashMap::new(),
            open_docs: HashMap::new(),
            statuses: HashMap::new(),
        }
    }
}

pub struct PendingLspRequest {
    pub workspace_id: WorkspaceId,
    pub language: code_editor::CodeLanguage,
    pub pane_id: PaneId,
    pub kind: lsp::LspRequestKind,
}

pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub expires_at: Instant,
}

pub enum ToastKind {
    Info,
    Success,
    Error,
}

pub struct Cratty {
    pub config: AppConfig, pub metrics: FontMetrics,
    pub workspaces: Vec<Workspace>, pub active_ws: usize,
    pub panes: HashMap<PaneId, Pane>, pub ws_colors: HashMap<WorkspaceId, Color>,
    pub id_gen: IdGen, pub sidebar_collapsed: bool,
    pub renaming_ws: Option<WorkspaceId>, pub rename_text: String,
    pub ws_menu_idx: Option<usize>, pub color_submenu: bool,
    pub pending_backends: Vec<PendingBackend>, pub viewport_w: f32,
    pub quake_hidden: bool, pub file_picker: Option<FilePickerState>,
    pub pending_lsp_installs: Vec<PendingLspInstall>, pub toasts: Vec<Toast>,
    pub workspace_lsp: HashMap<WorkspaceId, WorkspaceLspState>,
    pub lsp_pending: HashMap<u64, PendingLspRequest>,
    pub lsp_next_request_id: u64,
    _quake: Option<quake::QuakeHotkey>,
}

pub fn with_window<F, T>(f: F) -> Task<T>
where F: Fn(iced::window::Id) -> Task<T> + Send + 'static, T: Send + 'static {
    iced::window::oldest().and_then(move |id| f(id))
}

impl Cratty {
    fn new() -> (Self, Task<Message>) {
        let config = AppConfig::load(&AppConfig::config_path()).unwrap_or_default();
        let metrics = FontMetrics::from_size(config.font_size);
        let qk = if config.quake_mode.enabled { quake::QuakeHotkey::register(&config.quake_mode.hotkey) } else { None };
        let mut app = Self {
            config, metrics, workspaces: vec![], active_ws: 0,
            panes: HashMap::new(), ws_colors: HashMap::new(),
            id_gen: IdGen::new(), sidebar_collapsed: false,
            renaming_ws: None, rename_text: String::new(),
            ws_menu_idx: None, color_submenu: false,
            pending_backends: vec![], viewport_w: 1200.0,
            quake_hidden: false, file_picker: None,
            pending_lsp_installs: vec![], toasts: vec![],
            workspace_lsp: HashMap::new(), lsp_pending: HashMap::new(), lsp_next_request_id: 1,
            _quake: qk,
        };
        app.restore_layout();
        (app, Task::none())
    }

    fn view(&self) -> Element<'_, Message> {
        let mut view_layers = vec![self.view_base()];
        if let Some(p) = self.view_popups() { view_layers.push(p); }
        if let Some(t) = self.view_toasts() { view_layers.push(t); }
        layers::compose(view_layers)
    }

    fn view_base(&self) -> Element<'_, Message> {
        use iced::widget::row;
        let content: Element<Message> = match self.workspaces.get(self.active_ws) {
            Some(ws) if !ws.is_empty() =>
                strip_view::view_strip(&ws.strip, &self.panes, self.viewport_w, self.metrics),
            _ => home::view_home(),
        };
        let body: Element<Message> = if self.sidebar_collapsed { content } else {
            let sb = sidebar::view_sidebar(&self.workspaces, &self.ws_colors,
                &self.panes, self.active_ws, self.renaming_ws,
                &self.rename_text, self.ws_menu_idx);
            row![sb, content].width(Length::Fill).height(Length::Fill).into()
        };
        let mut layout = column![titlebar::view_titlebar(), body]
            .width(Length::Fill)
            .height(Length::Fill);
        if let Some(status) = self.view_workspace_lsp_bar() {
            layout = layout.push(status);
        }
        layout.into()
    }

    fn view_popups(&self) -> Option<Element<'_, Message>> {
        if let Some(fp) = &self.file_picker { return Some(file_picker_view::view(fp)); }
        self.menu_overlay()
    }
    fn view_toasts(&self) -> Option<Element<'_, Message>> {
        if self.toasts.is_empty() {
            None
        } else {
            Some(crate::widgets::view_toasts(&self.toasts))
        }
    }
    fn theme(&self) -> Theme { style::cratty_theme() }
    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![app_keys::input_subscription(), quake::subscription()];
        if self.needs_tick() {
            subs.push(app_keys::tick_subscription());
        }
        Subscription::batch(subs)
    }

    fn needs_tick(&self) -> bool {
        !self.pending_backends.is_empty()
            || self.workspaces.iter().any(|ws| ws.strip.viewport.is_animating())
            || self.workspaces.iter().any(|ws| ws.strip.any_width_animating())
            || self.panes.values().any(|pane| pane.terminal().is_some())
            || !self.pending_lsp_installs.is_empty()
            || self.workspace_lsp.values().any(|state| !state.servers.is_empty())
            || self.panes.values().any(|pane| match &pane.content {
                pane::PaneContent::Editor(code) =>
                    code.lsp.pending_sync_at.is_some() || code.lsp.pending_hover_at.is_some(),
                _ => false,
            })
    }

    fn view_workspace_lsp_bar(&self) -> Option<Element<'_, Message>> {
        use crate::style::{BG_TITLEBAR, FG_ACTIVE, FG_DIM, FG_INACTIVE};

        let workspace = self.workspaces.get(self.active_ws)?;
        let state = self.workspace_lsp.get(&workspace.id)?;

        let mut active = 0usize;
        let mut installing = 0usize;
        let mut failed = 0usize;
        let mut failed_details = Vec::new();
        for status in state.statuses.values() {
            match status {
                lsp::LspStatus::Ready => active += 1,
                lsp::LspStatus::Installing(_) | lsp::LspStatus::Starting => installing += 1,
                lsp::LspStatus::MissingServer(message) | lsp::LspStatus::Failed(message) => {
                    failed += 1;
                    failed_details.push(message.clone());
                }
                lsp::LspStatus::Disabled => {}
            }
        }

        let mut file_counts = HashMap::new();
        for pane_id in &workspace.strip.panes {
            let Some(pane::Pane { content: pane::PaneContent::Editor(code), .. }) = self.panes.get(pane_id) else {
                continue;
            };
            file_counts.insert(code.path.clone(), lsp::summarize_diagnostics(&code.lsp.diagnostics));
        }
        let (errors, warnings) = file_counts.values().fold((0usize, 0usize), |acc, counts| {
            (acc.0 + counts.0, acc.1 + counts.1)
        });

        let mut parts = vec![format!("LSP {active} active")];
        if installing > 0 {
            parts.push(format!("{installing} installing"));
        }
        if failed > 0 {
            parts.push(format!("{failed} failed"));
        }
        parts.push(format!("{errors}E {warnings}W"));

        let failure_text = failed_details.into_iter().next().map(|detail| {
            let detail = detail.replace('\n', " ");
            if detail.chars().count() > 120 {
                let truncated: String = detail.chars().take(117).collect();
                format!("{truncated}...")
            } else {
                detail
            }
        });

        Some(
            container(
                row![
                    text(&workspace.name).size(11).color(FG_ACTIVE),
                    text(parts.join("  •  ")).size(11).color(FG_INACTIVE),
                    container(text("")).width(Length::Fill),
                    text(failure_text.unwrap_or_default()).size(11).color(Color::from_rgb(0.878, 0.424, 0.459)),
                ]
                .align_y(iced::alignment::Vertical::Center)
                .spacing(12),
            )
            .width(Length::Fill)
            .padding([7, 12])
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(BG_TITLEBAR)),
                border: iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.07),
                    width: 1.0,
                    ..Default::default()
                },
                text_color: Some(FG_DIM),
                ..Default::default()
            })
            .into(),
        )
    }
}
