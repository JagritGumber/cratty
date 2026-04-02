use iced::alignment;
use iced::keyboard;
use iced::widget::{
    button, column, container, mouse_area, row, stack, text, text_input, Space,
};
use iced::window;
use iced::{event, Color, Element, Font, Length, Subscription, Task, Theme};

// Phosphor Bold icon font — embedded at compile time.
const PHOSPHOR_BOLD_BYTES: &[u8] = include_bytes!("../resources/fonts/Phosphor-Bold.ttf");
const PHOSPHOR: Font = Font::with_name("Phosphor-Bold");

// Phosphor Bold codepoints.
const ICO_MINUS: char = '\u{E32A}';        // minimize
const ICO_SQUARE: char = '\u{E45E}';       // maximize
const ICO_X: char = '\u{E4F6}';            // close
const ICO_PLUS: char = '\u{E3D4}';         // new tab
const ICO_DOTS_THREE_V: char = '\u{E208}'; // kebab menu

// Colors.
const BG_TITLEBAR: Color = Color::from_rgb(0.07, 0.07, 0.07);
const BG_TERMINAL: Color = Color::from_rgb(0.094, 0.094, 0.094); // #181818
const FG_ACTIVE: Color = Color::WHITE;
const FG_INACTIVE: Color = Color::from_rgb(0.55, 0.55, 0.55);
const FG_DIM: Color = Color::from_rgb(0.4, 0.4, 0.4);
const FG_MUTED: Color = Color::from_rgb(0.25, 0.25, 0.25);
const BG_MENU: Color = Color::from_rgb(0.12, 0.12, 0.12);
const BG_MENU_HOVER: Color = Color::from_rgb(0.18, 0.18, 0.18);
const BG_HOVER_SUBTLE: Color = Color::from_rgb(0.3, 0.3, 0.3);
const TITLEBAR_H: f32 = 36.0;

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

// ── Data ────────────────────────────────────────────────────────────────────

struct Tab {
    id: u64,
    title: String,
    custom_title: Option<String>,
    term: iced_term::Terminal,
}

impl Tab {
    fn display_title(&self) -> &str {
        self.custom_title.as_deref().unwrap_or(&self.title)
    }
}

struct RenameState {
    tab_idx: usize,
    input: String,
}

struct Cratty {
    tabs: Vec<Tab>,
    active_tab: usize,
    next_id: u64,
    last_size: Option<iced::Size>,
    renaming: Option<RenameState>,
    tab_menu_open: Option<usize>,
}

#[derive(Debug, Clone)]
enum Message {
    TermEvent(iced_term::Event),
    NewTab,
    DuplicateTab(usize),
    CloseTab(usize),
    SwitchTab(usize),
    DragWindow,
    Minimize,
    Maximize,
    CloseWindow,
    ToggleTabMenu(usize),
    CloseTabMenu,
    StartRename(usize),
    RenameInput(String),
    ConfirmRename,
    CancelRename,
    KeyPressed(keyboard::Key),
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Get the oldest window id then run an action on it.
fn with_window<F, T>(f: F) -> Task<T>
where
    F: Fn(window::Id) -> Task<T> + Send + 'static,
    T: Send + 'static,
{
    window::oldest().and_then(move |id| f(id))
}

/// Small icon button used in tabs (close ✕, kebab ⋮).
fn icon_btn(icon: char, size: f32, fg: Color, msg: Message) -> Element<'static, Message> {
    button(
        container(text(icon).size(size).font(PHOSPHOR).color(fg))
            .center_x(18)
            .center_y(18),
    )
    .on_press(msg)
    .width(18)
    .height(18)
    .padding(0)
    .style(|_, status| button::Style {
        background: match status {
            button::Status::Hovered => Some(iced::Background::Color(BG_HOVER_SUBTLE)),
            _ => None,
        },
        border: iced::Border { radius: 3.0.into(), ..Default::default() },
        ..Default::default()
    })
    .into()
}

/// Window chrome button (minimize / maximize / close).
fn win_btn(icon: char, msg: Message, fg: Color, hover_bg: Color) -> Element<'static, Message> {
    button(
        container(text(icon).size(14).font(PHOSPHOR).color(fg))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(msg)
    .width(46)
    .height(TITLEBAR_H)
    .padding(0)
    .style(move |_, status| button::Style {
        background: match status {
            button::Status::Hovered => Some(iced::Background::Color(hover_bg)),
            _ => None,
        },
        ..Default::default()
    })
    .into()
}

/// Dropdown menu item.
fn menu_item(label: &str, msg: Message) -> Element<'_, Message> {
    button(text(label).size(12).color(FG_ACTIVE))
        .on_press(msg)
        .padding([6, 16])
        .width(Length::Fill)
        .style(|_, status| button::Style {
            background: Some(iced::Background::Color(match status {
                button::Status::Hovered => BG_MENU_HOVER,
                _ => BG_MENU,
            })),
            ..Default::default()
        })
        .into()
}

/// Tab button style.
fn tab_style(active: bool) -> button::Style {
    let (bg, fg, border_color) = if active {
        (BG_TERMINAL, FG_ACTIVE, Color::TRANSPARENT)
    } else {
        (BG_TITLEBAR, FG_INACTIVE, Color::TRANSPARENT)
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: fg,
        border: iced::Border {
            color: border_color,
            width: 0.0,
            radius: iced::border::Radius::new(4.0).bottom(0.0),
        },
        ..Default::default()
    }
}

/// Rename text input style.
fn rename_input_style(_: &Theme, _: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1)),
        border: iced::Border {
            color: BG_HOVER_SUBTLE,
            width: 1.0,
            radius: 3.0.into(),
        },
        icon: FG_DIM,
        placeholder: FG_DIM,
        value: FG_ACTIVE,
        selection: Color::from_rgb(0.3, 0.3, 0.5),
    }
}

// ── App ─────────────────────────────────────────────────────────────────────

impl Cratty {
    fn new() -> (Self, Task<Message>) {
        match new_terminal(0) {
            Ok(t) => {
                let focus = iced_term::TerminalView::focus::<Message>(t.widget_id().clone());
                let app = Self {
                    tabs: vec![Tab { id: 0, title: "Terminal".into(), custom_title: None, term: t }],
                    active_tab: 0,
                    next_id: 1,
                    last_size: None,
                    renaming: None,
                    tab_menu_open: None,
                };
                (app, focus)
            }
            Err(e) => {
                tracing::error!("Failed to create terminal: {e}");
                let app = Self {
                    tabs: vec![],
                    active_tab: 0,
                    next_id: 1,
                    last_size: None,
                    renaming: None,
                    tab_menu_open: None,
                };
                (app, Task::none())
            }
        }
    }

    /// Create a new tab, insert it at `insert_at`, switch to it.
    fn create_tab(&mut self, insert_at: usize) -> Task<Message> {
        let id = self.next_id;
        self.next_id += 1;
        match new_terminal(id) {
            Ok(mut term) => {
                if let Some(size) = self.last_size {
                    term.handle(iced_term::Command::ProxyToBackend(
                        iced_term::BackendCommand::Resize(Some(size), None),
                    ));
                }
                let focus = iced_term::TerminalView::focus::<Message>(term.widget_id().clone());
                let tab = Tab { id, title: "Terminal".into(), custom_title: None, term };
                let pos = insert_at.min(self.tabs.len());
                self.tabs.insert(pos, tab);
                self.active_tab = pos;
                focus
            }
            Err(_) => Task::none(),
        }
    }

    /// Focus the active tab's terminal.
    fn focus_active_terminal(&self) -> Task<Message> {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            iced_term::TerminalView::focus::<Message>(tab.term.widget_id().clone())
        } else {
            Task::none()
        }
    }

    /// Close a tab by index, exit if none left.
    fn close_tab(&mut self, idx: usize) -> Task<Message> {
        if idx >= self.tabs.len() {
            return Task::none();
        }
        self.tabs.remove(idx);
        if self.tabs.is_empty() {
            return with_window(window::close);
        }
        self.active_tab = self.active_tab.min(self.tabs.len() - 1);
        Task::none()
    }

    fn dismiss_menu(&mut self) {
        self.tab_menu_open = None;
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TermEvent(iced_term::Event::BackendCall(term_id, cmd)) => {
                if let iced_term::BackendCommand::Resize(Some(layout), Some(_)) = &cmd {
                    self.last_size = Some(*layout);
                }
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == term_id) {
                    match tab.term.handle(iced_term::Command::ProxyToBackend(cmd)) {
                        iced_term::actions::Action::Shutdown => {
                            let idx = self.tabs.iter().position(|t| t.id == term_id).unwrap();
                            return self.close_tab(idx);
                        }
                        iced_term::actions::Action::ChangeTitle(title) => {
                            tab.title = title;
                        }
                        iced_term::actions::Action::Ignore => {}
                    }
                }
                Task::none()
            }

            Message::NewTab => {
                self.dismiss_menu();
                self.create_tab(self.tabs.len())
            }

            Message::DuplicateTab(idx) => {
                self.dismiss_menu();
                self.create_tab(idx + 1)
            }

            Message::CloseTab(idx) => {
                self.dismiss_menu();
                self.renaming = None;
                self.close_tab(idx)
            }

            Message::SwitchTab(idx) => {
                if self.renaming.is_some() || idx >= self.tabs.len() {
                    return Task::none();
                }
                self.dismiss_menu();
                self.active_tab = idx;
                self.focus_active_terminal()
            }

            Message::ToggleTabMenu(idx) => {
                self.tab_menu_open = if self.tab_menu_open == Some(idx) { None } else { Some(idx) };
                Task::none()
            }

            Message::CloseTabMenu => {
                self.dismiss_menu();
                Task::none()
            }

            Message::StartRename(idx) => {
                self.dismiss_menu();
                if idx >= self.tabs.len() {
                    return Task::none();
                }
                self.renaming = Some(RenameState {
                    tab_idx: idx,
                    input: self.tabs[idx].custom_title.clone().unwrap_or_default(),
                });
                iced::widget::operation::focus_next()
            }

            Message::RenameInput(val) => {
                if let Some(state) = &mut self.renaming {
                    state.input = val;
                }
                Task::none()
            }

            Message::ConfirmRename => {
                if let Some(state) = self.renaming.take() {
                    if let Some(tab) = self.tabs.get_mut(state.tab_idx) {
                        let trimmed = state.input.trim();
                        tab.custom_title = if trimmed.is_empty() { None } else { Some(trimmed.to_string()) };
                    }
                }
                self.focus_active_terminal()
            }

            Message::CancelRename => {
                self.renaming = None;
                self.focus_active_terminal()
            }

            Message::KeyPressed(key) => {
                if key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                    if self.renaming.is_some() {
                        return self.update(Message::CancelRename);
                    }
                    self.dismiss_menu();
                }
                Task::none()
            }

            Message::DragWindow => with_window(window::drag),
            Message::Minimize => with_window(|id| window::minimize(id, true)),
            Message::Maximize => with_window(window::toggle_maximize),
            Message::CloseWindow => with_window(window::close),
        }
    }

    // ── View ────────────────────────────────────────────────────────────────

    fn view(&self) -> Element<'_, Message> {
        let titlebar = self.view_titlebar();

        let terminal_view: Element<Message> = if let Some(tab) = self.tabs.get(self.active_tab) {
            iced::widget::keyed_column(std::iter::once((
                tab.id,
                container(iced_term::TerminalView::show(&tab.term).map(Message::TermEvent))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
            )))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            container(text("No terminal open").size(16))
                .center(Length::Fill)
                .into()
        };

        let main_content: Element<Message> = column![titlebar, terminal_view]
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        if let Some(menu_idx) = self.tab_menu_open {
            let menu_overlay: Element<Message> = container(self.view_tab_menu(menu_idx))
                .padding(iced::Padding {
                    top: TITLEBAR_H,
                    left: self.tab_menu_x_offset(menu_idx),
                    right: 0.0,
                    bottom: 0.0,
                })
                .width(Length::Fill)
                .height(Length::Fill)
                .into();

            let scrim: Element<Message> = mouse_area(
                container(Space::new()).width(Length::Fill).height(Length::Fill),
            )
            .on_press(Message::CloseTabMenu)
            .into();

            stack![main_content, scrim, menu_overlay]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            main_content
        }
    }

    fn tab_menu_x_offset(&self, idx: usize) -> f32 {
        let tab_width: f32 = 120.0;
        let spacing: f32 = 2.0;
        let left_pad: f32 = 8.0;
        left_pad + (idx as f32) * (tab_width + spacing)
    }

    fn view_titlebar(&self) -> Element<'_, Message> {
        let mut tabs_items: Vec<Element<Message>> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(idx, tab)| self.view_tab(idx, tab))
            .collect();

        tabs_items.push(
            button(
                container(text(ICO_PLUS).size(14).font(PHOSPHOR).color(FG_DIM))
                    .center_x(Length::Shrink)
                    .center_y(Length::Shrink),
            )
            .on_press(Message::NewTab)
            .padding([4, 8])
            .style(|_, status| button::Style {
                background: match status {
                    button::Status::Hovered => Some(iced::Background::Color(BG_MENU_HOVER)),
                    _ => None,
                },
                border: iced::Border { radius: 4.0.into(), ..Default::default() },
                ..Default::default()
            })
            .into(),
        );

        let tabs = row(tabs_items)
            .spacing(2)
            .padding(iced::Padding { top: 4.0, right: 8.0, bottom: 0.0, left: 8.0 })
            .align_y(alignment::Vertical::Bottom);

        let controls = row![
            win_btn(ICO_MINUS, Message::Minimize, FG_DIM, Color::from_rgb(0.2, 0.2, 0.2)),
            win_btn(ICO_SQUARE, Message::Maximize, FG_DIM, Color::from_rgb(0.2, 0.2, 0.2)),
            win_btn(ICO_X, Message::CloseWindow, FG_DIM, Color::from_rgb(0.7, 0.15, 0.15)),
        ]
        .spacing(0);

        let bar = row![tabs, Space::new().width(Length::Fill), controls]
            .align_y(alignment::Vertical::Center)
            .height(TITLEBAR_H);

        mouse_area(
            container(bar)
                .width(Length::Fill)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(BG_TITLEBAR)),
                    ..Default::default()
                }),
        )
        .on_press(Message::DragWindow)
        .into()
    }

    fn view_tab(&self, idx: usize, tab: &Tab) -> Element<'_, Message> {
        let active = idx == self.active_tab;
        let is_renaming = self.renaming.as_ref().is_some_and(|r| r.tab_idx == idx);
        let icon_fg = if active { FG_DIM } else { FG_MUTED };

        let tab_content: Element<Message> = if is_renaming {
            let input_val = self.renaming.as_ref().unwrap().input.clone();
            text_input("Tab name (empty to reset)", &input_val)
                .on_input(Message::RenameInput)
                .on_submit(Message::ConfirmRename)
                .size(11)
                .width(120)
                .padding([2, 4])
                .style(rename_input_style)
                .into()
        } else {
            let display = tab.display_title();
            let label = if display.len() > 20 {
                format!("{}...", &display[..17])
            } else {
                display.to_string()
            };
            text(label)
                .size(11)
                .color(if active { FG_ACTIVE } else { FG_INACTIVE })
                .into()
        };

        let tab_row = row![
            tab_content,
            icon_btn(ICO_DOTS_THREE_V, 14.0, icon_fg, Message::ToggleTabMenu(idx)),
            icon_btn(ICO_X, 10.0, icon_fg, Message::CloseTab(idx)),
        ]
        .spacing(4)
        .align_y(alignment::Vertical::Center);

        button(tab_row)
            .on_press(Message::SwitchTab(idx))
            .padding([5, 10])
            .style(move |_, _| tab_style(active))
            .into()
    }

    fn view_tab_menu(&self, idx: usize) -> Element<'_, Message> {
        container(
            column![
                menu_item("Rename", Message::StartRename(idx)),
                menu_item("Duplicate", Message::DuplicateTab(idx)),
                menu_item("Close", Message::CloseTab(idx)),
            ]
            .width(120),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(BG_MENU)),
            border: iced::Border {
                color: FG_MUTED,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        let term_subs = self
            .tabs
            .iter()
            .map(|tab| tab.term.subscription().map(Message::TermEvent));

        let key_sub = event::listen_with(|evt, _status, _window| {
            if let iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = evt {
                Some(Message::KeyPressed(key))
            } else {
                None
            }
        });

        Subscription::batch(term_subs.chain(std::iter::once(key_sub)))
    }
}

// ── Terminal ────────────────────────────────────────────────────────────────

fn new_terminal(id: u64) -> std::io::Result<iced_term::Terminal> {
    let (shell, args) = default_shell();
    iced_term::Terminal::new(
        id,
        iced_term::settings::Settings {
            font: iced_term::settings::FontSettings { size: 14.0, ..Default::default() },
            theme: iced_term::settings::ThemeSettings::default(),
            backend: iced_term::settings::BackendSettings {
                program: shell,
                args,
                ..Default::default()
            },
        },
    )
}

fn default_shell() -> (String, Vec<String>) {
    #[cfg(windows)]
    {
        if which("pwsh.exe") {
            ("pwsh.exe".into(), vec!["-NoLogo".into()])
        } else if which("powershell.exe") {
            ("powershell.exe".into(), vec!["-NoLogo".into()])
        } else {
            ("cmd.exe".into(), vec![])
        }
    }
    #[cfg(not(windows))]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
        (shell, vec!["-l".into()])
    }
}

#[cfg(windows)]
fn which(name: &str) -> bool {
    std::process::Command::new("where")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
