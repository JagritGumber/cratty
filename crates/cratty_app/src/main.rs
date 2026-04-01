use iced::alignment;
use iced::widget::{button, column, container, mouse_area, row, text, Space};
use iced::window;
use iced::{Color, Element, Font, Length, Subscription, Task, Theme};

// Phosphor Bold icon font — embedded at compile time.
const PHOSPHOR_BOLD_BYTES: &[u8] =
    include_bytes!("../resources/fonts/Phosphor-Bold.ttf");
const PHOSPHOR: Font = Font::with_name("Phosphor-Bold");

// Phosphor Bold codepoints.
const ICO_MINUS: char = '\u{E32A}';   // minimize
const ICO_SQUARE: char = '\u{E45E}';  // maximize
const ICO_X: char = '\u{E4F6}';       // close window
const ICO_PLUS: char = '\u{E3D4}';    // new tab

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

struct Tab {
    id: u64,
    title: String,
    term: iced_term::Terminal,
}

struct Cratty {
    tabs: Vec<Tab>,
    active_tab: usize,
    next_id: u64,
    last_size: Option<iced::Size>,
}

#[derive(Debug, Clone)]
enum Message {
    TermEvent(iced_term::Event),
    NewTab,
    CloseTab(usize),
    SwitchTab(usize),
    DragWindow,
    Minimize,
    Maximize,
    CloseWindow,
}

const BG_TITLEBAR: Color = Color::from_rgb(0.07, 0.07, 0.07);
const BG_TERMINAL: Color = Color::from_rgb(0.094, 0.094, 0.094); // #181818 — matches iced_term default
const BG_INACTIVE_TAB: Color = Color::from_rgb(0.07, 0.07, 0.07); // same as titlebar
const FG_ACTIVE: Color = Color::WHITE;
const FG_INACTIVE: Color = Color::from_rgb(0.55, 0.55, 0.55);
const FG_DIM: Color = Color::from_rgb(0.4, 0.4, 0.4);
const BORDER_ACTIVE: Color = Color::from_rgb(0.25, 0.25, 0.25);
const TITLEBAR_H: f32 = 36.0;

/// Helper: get oldest window id then run an action on it.
fn with_window<F, T>(f: F) -> Task<T>
where
    F: Fn(window::Id) -> Task<T> + Send + 'static,
    T: Send + 'static,
{
    window::oldest().and_then(move |id| f(id))
}

impl Cratty {
    fn new() -> (Self, Task<Message>) {
        match new_terminal(0) {
            Ok(t) => {
                let focus = iced_term::TerminalView::focus::<Message>(t.widget_id().clone());
                (
                    Self {
                        tabs: vec![Tab { id: 0, title: "Terminal".into(), term: t }],
                        active_tab: 0,
                        next_id: 1,
                        last_size: None,
                    },
                    focus,
                )
            }
            Err(e) => {
                tracing::error!("Failed to create terminal: {e}");
                (
                    Self { tabs: vec![], active_tab: 0, next_id: 1, last_size: None },
                    Task::none(),
                )
            }
        }
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
                            self.tabs.retain(|t| t.id != term_id);
                            if self.tabs.is_empty() {
                                return with_window(window::close);
                            }
                            self.active_tab = self.active_tab.min(self.tabs.len() - 1);
                        }
                        iced_term::actions::Action::ChangeTitle(title) => {
                            if let Some(t) = self.tabs.iter_mut().find(|t| t.id == term_id) {
                                t.title = title;
                            }
                        }
                        iced_term::actions::Action::Ignore => {}
                    }
                }
                Task::none()
            }

            Message::NewTab => {
                let id = self.next_id;
                self.next_id += 1;
                match new_terminal(id) {
                    Ok(mut term) => {
                        if let Some(size) = self.last_size {
                            term.handle(iced_term::Command::ProxyToBackend(
                                iced_term::BackendCommand::Resize(Some(size), None),
                            ));
                        }
                        let focus =
                            iced_term::TerminalView::focus::<Message>(term.widget_id().clone());
                        self.tabs.push(Tab { id, title: "Terminal".into(), term });
                        self.active_tab = self.tabs.len() - 1;
                        focus
                    }
                    Err(_) => Task::none(),
                }
            }

            Message::CloseTab(idx) => {
                if idx < self.tabs.len() {
                    self.tabs.remove(idx);
                    if self.tabs.is_empty() {
                        return with_window(window::close);
                    }
                    self.active_tab = self.active_tab.min(self.tabs.len() - 1);
                }
                Task::none()
            }

            Message::SwitchTab(idx) => {
                if idx < self.tabs.len() {
                    self.active_tab = idx;
                    iced_term::TerminalView::focus::<Message>(
                        self.tabs[idx].term.widget_id().clone(),
                    )
                } else {
                    Task::none()
                }
            }

            Message::DragWindow => with_window(window::drag),
            Message::Minimize => with_window(|id| window::minimize(id, true)),
            Message::Maximize => with_window(window::toggle_maximize),
            Message::CloseWindow => with_window(window::close),
        }
    }

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

        column![titlebar, terminal_view]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_titlebar(&self) -> Element<'_, Message> {
        // --- Tabs ---
        let mut tabs_items: Vec<Element<Message>> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(idx, tab)| {
                let active = idx == self.active_tab;
                let label = if tab.title.len() > 20 {
                    format!("{}...", &tab.title[..17])
                } else {
                    tab.title.clone()
                };

                let close = button(
                    container(
                        text(ICO_X)
                            .size(10)
                            .font(PHOSPHOR)
                            .color(if active { FG_DIM } else { Color::from_rgb(0.25, 0.25, 0.25) }),
                    )
                    .center_x(18)
                    .center_y(18),
                )
                .on_press(Message::CloseTab(idx))
                .width(18)
                .height(18)
                .padding(0)
                .style(|_, status| button::Style {
                    background: match status {
                        button::Status::Hovered => Some(iced::Background::Color(
                            Color::from_rgb(0.3, 0.3, 0.3),
                        )),
                        _ => None,
                    },
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

                let tab_row = row![
                    text(label).size(11).color(if active { FG_ACTIVE } else { FG_INACTIVE }),
                    close,
                ]
                .spacing(6)
                .align_y(alignment::Vertical::Center);

                button(tab_row)
                    .on_press(Message::SwitchTab(idx))
                    .padding([5, 12])
                    .style(move |_, _| {
                        if active {
                            button::Style {
                                background: Some(iced::Background::Color(BG_TERMINAL)),
                                text_color: FG_ACTIVE,
                                border: iced::Border {
                                    color: BORDER_ACTIVE,
                                    width: 0.0,
                                    radius: iced::border::Radius::new(4.0).bottom(0.0),
                                },
                                ..Default::default()
                            }
                        } else {
                            button::Style {
                                background: Some(iced::Background::Color(BG_INACTIVE_TAB)),
                                text_color: FG_INACTIVE,
                                border: iced::Border {
                                    color: Color::TRANSPARENT,
                                    width: 0.0,
                                    radius: iced::border::Radius::new(4.0).bottom(0.0),
                                },
                                ..Default::default()
                            }
                        }
                    })
                    .into()
            })
            .collect();

        // + new tab
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
                    button::Status::Hovered => {
                        Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15)))
                    }
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

        // --- Window controls: [minimize] [maximize] [close] ---
        let controls = row![
            win_btn(ICO_MINUS, Message::Minimize, FG_DIM, Color::from_rgb(0.2, 0.2, 0.2)),
            win_btn(ICO_SQUARE, Message::Maximize, FG_DIM, Color::from_rgb(0.2, 0.2, 0.2)),
            win_btn(ICO_X, Message::CloseWindow, FG_DIM, Color::from_rgb(0.7, 0.15, 0.15)),
        ]
        .spacing(0);

        // --- Full bar: draggable area with tabs on left, controls on right ---
        let bar_inner = row![
            tabs,
            Space::new().width(Length::Fill),
            controls,
        ]
        .align_y(alignment::Vertical::Center)
        .height(TITLEBAR_H);

        // Wrap in mouse_area for drag
        mouse_area(
            container(bar_inner)
                .width(Length::Fill)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(BG_TITLEBAR)),
                    ..Default::default()
                }),
        )
        .on_press(Message::DragWindow)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(
            self.tabs
                .iter()
                .map(|tab| tab.term.subscription().map(Message::TermEvent)),
        )
    }
}

/// Window control button (minimize / maximize / close).
fn win_btn(
    icon: char,
    msg: Message,
    fg: Color,
    hover_bg: Color,
) -> Element<'static, Message> {
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

fn new_terminal(id: u64) -> std::io::Result<iced_term::Terminal> {
    let (shell, args) = default_shell();
    iced_term::Terminal::new(
        id,
        iced_term::settings::Settings {
            font: iced_term::settings::FontSettings {
                size: 14.0,
                ..Default::default()
            },
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
