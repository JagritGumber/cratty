use iced::alignment;
use iced::keyboard;
use iced::widget::{
    button, column, container, mouse_area, row, stack, text, text_input, Space,
};
use iced::window;
use iced::{event, Color, Element, Font, Length, Subscription, Task, Theme};
use std::path::{Path, PathBuf};

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

// Tab geometry constants for menu positioning.
const TAB_PAD_H: f32 = 10.0;     // horizontal padding inside tab button ([5, 10])
const TAB_ICON_W: f32 = 18.0;    // width of each icon button (dots, close)
const TAB_ICON_GAP: f32 = 4.0;   // spacing between elements in tab row
const TAB_ROW_LEFT: f32 = 8.0;   // left padding of the tabs row
const TAB_ROW_SPACING: f32 = 2.0; // spacing between tab buttons
const TAB_CHAR_W: f32 = 6.5;     // approximate width per character at size 11

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
    color: Option<Color>,
    term: iced_term::Terminal,
}

// Preset tab colors.
const TAB_COLORS: &[(Color, &str)] = &[
    (Color::from_rgb(0.90, 0.30, 0.30), "Red"),
    (Color::from_rgb(0.95, 0.55, 0.25), "Orange"),
    (Color::from_rgb(0.90, 0.80, 0.25), "Yellow"),
    (Color::from_rgb(0.35, 0.75, 0.40), "Green"),
    (Color::from_rgb(0.30, 0.65, 0.90), "Blue"),
    (Color::from_rgb(0.55, 0.40, 0.85), "Purple"),
    (Color::from_rgb(0.85, 0.40, 0.70), "Pink"),
    (Color::from_rgb(0.45, 0.75, 0.75), "Teal"),
];

impl Tab {
    fn display_title(&self) -> &str {
        self.custom_title.as_deref().unwrap_or(&self.title)
    }

    /// Estimate the rendered width of this tab button.
    fn estimated_width(&self, is_renaming: bool) -> f32 {
        let text_w = if is_renaming {
            120.0 // text_input has explicit width(120)
        } else {
            self.display_title().chars().count().min(20) as f32 * TAB_CHAR_W
        };
        // text + gap + dots icon + gap + close icon + horizontal padding both sides
        text_w + TAB_ICON_GAP + TAB_ICON_W + TAB_ICON_GAP + TAB_ICON_W + TAB_PAD_H * 2.0
    }
}

struct RenameState {
    tab_id: u64,
    input: String,
}

struct Cratty {
    tabs: Vec<Tab>,
    active_tab: usize,
    next_id: u64,
    last_size: Option<iced::Size>,
    renaming: Option<RenameState>,
    tab_menu_open: Option<u64>, // tab ID, not index
    color_submenu_open: bool,
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
    ToggleTabMenu(u64),
    CloseTabMenu,
    StartRename(usize),
    RenameInput(String),
    ConfirmRename,
    SetTabColor(u64, Option<Color>),
    OpenColorSubmenu,
    EscapePressed,
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

/// Small icon button used in tabs (close, kebab).
fn icon_btn(icon: char, size: f32, fg: Color, msg: Message) -> Element<'static, Message> {
    button(
        container(text(icon).size(size).font(PHOSPHOR).color(fg))
            .center_x(18)
            .center_y(18),
    )
    .on_press(msg)
    .width(TAB_ICON_W)
    .height(TAB_ICON_W)
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

/// Tab button style with optional color accent.
fn tab_style(active: bool, tab_color: Option<Color>) -> button::Style {
    let (bg, fg) = if active {
        match tab_color {
            // Tint the active tab background with the color (subtle blend)
            Some(c) => {
                let bg = Color::from_rgb(
                    BG_TERMINAL.r * 0.7 + c.r * 0.3,
                    BG_TERMINAL.g * 0.7 + c.g * 0.3,
                    BG_TERMINAL.b * 0.7 + c.b * 0.3,
                );
                (bg, FG_ACTIVE)
            }
            None => (BG_TERMINAL, FG_ACTIVE),
        }
    } else {
        (BG_TITLEBAR, FG_INACTIVE)
    };

    // Colored tabs get a 2px bottom accent line
    let (border_color, border_width) = match tab_color {
        Some(c) => (c, 2.0),
        None => (Color::TRANSPARENT, 0.0),
    };

    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: fg,
        border: iced::Border {
            color: border_color,
            width: border_width,
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
        match new_terminal(0, None) {
            Ok(t) => {
                let focus = iced_term::TerminalView::focus::<Message>(t.widget_id().clone());
                let app = Self {
                    tabs: vec![Tab { id: 0, title: "Terminal".into(), custom_title: None, color: None, term: t }],
                    active_tab: 0,
                    next_id: 1,
                    last_size: None,
                    renaming: None,
                    tab_menu_open: None,
                    color_submenu_open: false,
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
                    color_submenu_open: false,
                };
                (app, Task::none())
            }
        }
    }

    /// Create a new tab, insert it at `insert_at`, switch to it.
    fn create_tab(&mut self, insert_at: usize, cwd: Option<PathBuf>) -> Task<Message> {
        let id = self.next_id;
        self.next_id += 1;
        match new_terminal(id, cwd) {
            Ok(mut term) => {
                if let Some(size) = self.last_size {
                    term.handle(iced_term::Command::ProxyToBackend(
                        iced_term::BackendCommand::Resize(Some(size), None),
                    ));
                }
                let focus = iced_term::TerminalView::focus::<Message>(term.widget_id().clone());
                let tab = Tab { id, title: "Terminal".into(), custom_title: None, color: None, term };
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

    /// Close a tab by index. Clears menu/rename state, refocuses terminal.
    fn close_tab(&mut self, idx: usize) -> Task<Message> {
        if idx >= self.tabs.len() {
            return Task::none();
        }
        let removed_id = self.tabs[idx].id;
        self.tabs.remove(idx);
        self.tab_menu_open = None;
        if self.renaming.as_ref().is_some_and(|r| r.tab_id == removed_id) {
            self.renaming = None;
        }
        if self.tabs.is_empty() {
            return with_window(window::close);
        }
        self.active_tab = self.active_tab.min(self.tabs.len() - 1);
        self.focus_active_terminal()
    }

    fn dismiss_menu(&mut self) {
        self.tab_menu_open = None;
        self.color_submenu_open = false;
    }

    /// Whether any UI overlay (menu, rename) is active — used to gate keyboard subscription.
    fn has_overlay(&self) -> bool {
        self.renaming.is_some() || self.tab_menu_open.is_some()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TermEvent(iced_term::Event::BackendCall(term_id, cmd)) => {
                if let iced_term::BackendCommand::Resize(Some(layout), Some(_)) = &cmd {
                    self.last_size = Some(*layout);
                }
                // Handle the backend command and capture the resulting action.
                // Separate the borrow of `tab` from the post-action dispatch to
                // avoid holding a mutable borrow across `self.close_tab()`.
                let action = self
                    .tabs
                    .iter_mut()
                    .find(|t| t.id == term_id)
                    .map(|tab| tab.term.handle(iced_term::Command::ProxyToBackend(cmd)));

                match action {
                    Some(iced_term::actions::Action::Shutdown) => {
                        if let Some(idx) = self.tabs.iter().position(|t| t.id == term_id) {
                            return self.close_tab(idx);
                        }
                    }
                    Some(iced_term::actions::Action::ChangeTitle(title)) => {
                        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == term_id) {
                            tab.title = title;
                        }
                    }
                    Some(iced_term::actions::Action::Ignore) | None => {}
                }
                Task::none()
            }

            Message::NewTab => {
                self.dismiss_menu();
                self.create_tab(self.tabs.len(), None)
            }

            Message::DuplicateTab(idx) => {
                self.dismiss_menu();
                let cwd = self.tabs.get(idx).and_then(|tab| extract_cwd(&tab.title));
                self.create_tab(idx + 1, cwd)
            }

            Message::CloseTab(idx) => self.close_tab(idx),

            Message::SwitchTab(idx) => {
                if self.renaming.is_some() || idx >= self.tabs.len() {
                    return Task::none();
                }
                self.dismiss_menu();
                self.active_tab = idx;
                self.focus_active_terminal()
            }

            Message::ToggleTabMenu(tab_id) => {
                self.tab_menu_open =
                    if self.tab_menu_open == Some(tab_id) { None } else { Some(tab_id) };
                Task::none()
            }

            Message::CloseTabMenu => {
                self.dismiss_menu();
                Task::none()
            }

            Message::StartRename(idx) => {
                self.dismiss_menu();
                if let Some(tab) = self.tabs.get(idx) {
                    self.renaming = Some(RenameState {
                        tab_id: tab.id,
                        input: tab.custom_title.clone().unwrap_or_default(),
                    });
                    iced::widget::operation::focus_next()
                } else {
                    Task::none()
                }
            }

            Message::RenameInput(val) => {
                if let Some(state) = &mut self.renaming {
                    state.input = val;
                }
                Task::none()
            }

            Message::ConfirmRename => {
                if let Some(state) = self.renaming.take() {
                    if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == state.tab_id) {
                        let trimmed = state.input.trim();
                        tab.custom_title =
                            if trimmed.is_empty() { None } else { Some(trimmed.to_string()) };
                    }
                }
                self.focus_active_terminal()
            }

            Message::SetTabColor(tab_id, color) => {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
                    tab.color = color;
                }
                self.dismiss_menu();
                Task::none()
            }

            Message::OpenColorSubmenu => {
                self.color_submenu_open = true;
                Task::none()
            }

            Message::EscapePressed => {
                if self.renaming.is_some() {
                    self.renaming = None;
                    return self.focus_active_terminal();
                }
                self.dismiss_menu();
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

        if let Some(menu_tab_id) = self.tab_menu_open {
            if let Some(menu_idx) = self.tabs.iter().position(|t| t.id == menu_tab_id) {
                let menu_x = self.tab_menu_x_offset(menu_idx);

                let menu_overlay: Element<Message> = container(self.view_tab_menu(menu_idx))
                    .padding(iced::Padding {
                        top: TITLEBAR_H,
                        left: menu_x,
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

                let mut layers: Vec<Element<Message>> = vec![
                    main_content,
                    scrim,
                    menu_overlay,
                ];

                // Color submenu: nested panel flush to the right of the main menu
                if self.color_submenu_open {
                    // Main menu is 140px wide. Color row is 3rd item (~84px from top).
                    let color_panel: Element<Message> =
                        container(self.view_color_panel(menu_idx))
                            .padding(iced::Padding {
                                top: TITLEBAR_H + 60.0,
                                left: menu_x + 140.0,
                                right: 0.0,
                                bottom: 0.0,
                            })
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into();
                    layers.push(color_panel);
                }

                return stack(layers)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();
            }
        }
        main_content
    }

    /// Compute the x offset for the dropdown menu under tab at `idx`.
    fn tab_menu_x_offset(&self, idx: usize) -> f32 {
        let mut x = TAB_ROW_LEFT;
        for (i, tab) in self.tabs.iter().enumerate() {
            if i == idx {
                break;
            }
            let is_renaming = self.renaming.as_ref().is_some_and(|r| r.tab_id == tab.id);
            x += tab.estimated_width(is_renaming) + TAB_ROW_SPACING;
        }
        x
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
            .spacing(TAB_ROW_SPACING)
            .padding(iced::Padding { top: 4.0, right: 8.0, bottom: 0.0, left: TAB_ROW_LEFT })
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
        let is_renaming = self.renaming.as_ref().is_some_and(|r| r.tab_id == tab.id);
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
            let label: String = if display.chars().count() > 20 {
                format!("{}...", display.chars().take(17).collect::<String>())
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
            icon_btn(ICO_DOTS_THREE_V, 14.0, icon_fg, Message::ToggleTabMenu(tab.id)),
            icon_btn(ICO_X, 10.0, icon_fg, Message::CloseTab(idx)),
        ]
        .spacing(TAB_ICON_GAP)
        .align_y(alignment::Vertical::Center);

        let color = tab.color;
        button(tab_row)
            .on_press(Message::SwitchTab(idx))
            .padding([5, TAB_PAD_H as u16])
            .style(move |_, _| tab_style(active, color))
            .into()
    }

    fn view_tab_menu(&self, idx: usize) -> Element<'_, Message> {
        let separator = || {
            container(Space::new())
                .width(Length::Fill)
                .height(1)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(FG_MUTED)),
                    ..Default::default()
                })
        };

        let color_label = row![
            text("Color").size(12).color(FG_ACTIVE),
            Space::new().width(Length::Fill),
            text("›").size(14).color(FG_DIM),
        ]
        .width(Length::Fill)
        .padding([6, 16]);

        let color_bg = if self.color_submenu_open { BG_MENU_HOVER } else { BG_MENU };
        let color_item: Element<Message> = mouse_area(
            container(color_label)
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(color_bg)),
                    ..Default::default()
                }),
        )
        .on_enter(Message::OpenColorSubmenu)
        .into();

        container(
            column![
                menu_item("Rename", Message::StartRename(idx)),
                menu_item("Duplicate", Message::DuplicateTab(idx)),
                separator(),
                color_item,
                separator(),
                menu_item("Close", Message::CloseTab(idx)),
            ]
            .width(140),
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

    fn view_color_panel(&self, idx: usize) -> Element<'_, Message> {
        let tab_id = self.tabs[idx].id;
        let current_color = self.tabs[idx].color;

        let make_swatch = |color: Color, is_selected: bool, tid: u64| -> Element<'_, Message> {
            button(Space::new().width(18).height(18))
                .on_press(Message::SetTabColor(tid, Some(color)))
                .width(24)
                .height(24)
                .padding(3)
                .style(move |_, status| {
                    let border = if is_selected {
                        iced::Border { color: FG_ACTIVE, width: 2.0, radius: 4.0.into() }
                    } else {
                        match status {
                            button::Status::Hovered => iced::Border {
                                color: FG_DIM,
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            _ => iced::Border { radius: 4.0.into(), ..Default::default() },
                        }
                    };
                    button::Style {
                        background: Some(iced::Background::Color(color)),
                        border,
                        ..Default::default()
                    }
                })
                .into()
        };

        let row1: Vec<Element<Message>> = TAB_COLORS[..4]
            .iter()
            .map(|(c, _)| make_swatch(*c, current_color == Some(*c), tab_id))
            .collect();

        let mut row2: Vec<Element<Message>> = TAB_COLORS[4..]
            .iter()
            .map(|(c, _)| make_swatch(*c, current_color == Some(*c), tab_id))
            .collect();

        // Add a "clear" swatch (dark with red ✕) as the 9th item in row 2
        if current_color.is_some() {
            row2.push(
                button(
                    container(text(ICO_X).size(10).font(PHOSPHOR).color(Color::from_rgb(0.8, 0.25, 0.25)))
                        .center_x(18)
                        .center_y(18),
                )
                .on_press(Message::SetTabColor(tab_id, None))
                .width(24)
                .height(24)
                .padding(0)
                .style(|_, status| button::Style {
                    background: Some(iced::Background::Color(match status {
                        button::Status::Hovered => Color::from_rgb(0.2, 0.2, 0.2),
                        _ => Color::from_rgb(0.12, 0.12, 0.12),
                    })),
                    border: iced::Border {
                        color: Color::from_rgb(0.8, 0.25, 0.25),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .into(),
            );
        }

        container(
            column![
                row(row1).spacing(3),
                row(row2).spacing(3),
            ]
            .spacing(3)
            .padding(6),
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

        // Only intercept Escape when an overlay is active, so terminal apps
        // (vim, htop, etc.) receive Escape normally at all other times.
        if self.has_overlay() {
            let esc_sub = event::listen_with(|evt, _status, _window| {
                if let iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = evt {
                    if key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                        return Some(Message::EscapePressed);
                    }
                }
                None
            });
            Subscription::batch(term_subs.chain(std::iter::once(esc_sub)))
        } else {
            Subscription::batch(term_subs)
        }
    }
}

// ── Terminal ────────────────────────────────────────────────────────────────

fn new_terminal(id: u64, working_directory: Option<PathBuf>) -> std::io::Result<iced_term::Terminal> {
    let (shell, args) = default_shell();
    iced_term::Terminal::new(
        id,
        iced_term::settings::Settings {
            font: iced_term::settings::FontSettings { size: 14.0, ..Default::default() },
            theme: iced_term::settings::ThemeSettings::default(),
            backend: iced_term::settings::BackendSettings {
                program: shell,
                args,
                working_directory,
                ..Default::default()
            },
        },
    )
}

/// Try to extract a valid directory path from a terminal title.
/// Shells set the title in various formats — we try common patterns.
fn extract_cwd(title: &str) -> Option<PathBuf> {
    tracing::debug!("extract_cwd from title: {:?}", title);

    let candidates: Vec<&str> = vec![
        title.trim(),
        // "Administrator: C:\path"
        title.strip_prefix("Administrator: ").unwrap_or("").trim(),
        // "MINGW64:/c/Users/foo" → skip
        // "user@host: ~/projects" → skip (not absolute on Windows)
    ];

    // Also try to find a Windows path (X:\...) or Unix path (/...) anywhere in the title
    // e.g. "PS C:\Users\jagri" → extract "C:\Users\jagri"
    let embedded = extract_embedded_path(title);

    for candidate in candidates.into_iter().chain(embedded.as_deref()) {
        if candidate.is_empty() {
            continue;
        }
        let path = Path::new(candidate);
        if path.is_absolute() && path.is_dir() {
            tracing::info!("Extracted CWD: {:?}", path);
            return Some(path.to_path_buf());
        }
    }

    tracing::debug!("No valid CWD found in title");
    None
}

/// Try to find an embedded absolute path in a string.
/// Handles cases like "PS C:\Users\foo" or "1 | vim - C:\Projects".
fn extract_embedded_path(title: &str) -> Option<String> {
    // Windows: look for X:\ pattern
    if let Some(idx) = title.find(":\\") {
        if idx > 0 {
            let start = idx - 1;
            let candidate = title[start..].trim_end_matches(&[' ', '>', ']', ')'][..]);
            return Some(candidate.to_string());
        }
    }
    // Unix: look for paths starting with /
    if let Some(idx) = title.find('/') {
        let candidate = title[idx..].split_whitespace().next()?;
        return Some(candidate.to_string());
    }
    None
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
