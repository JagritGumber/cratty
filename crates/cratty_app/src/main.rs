use iced::keyboard;
use iced::widget::{column, container, mouse_area, stack, Space};
use iced::window;
use iced::{event, Element, Length, Subscription, Task, Theme};
use std::path::PathBuf;

mod home;
mod menu;
mod message;
mod style;
mod terminal;
mod titlebar;
mod widgets;

use message::{Message, RenameState, Tab};
use style::*;
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
    tabs: Vec<Tab>,
    active_tab: usize,
    next_id: u64,
    last_size: Option<iced::Size>,
    renaming: Option<RenameState>,
    tab_menu_open: Option<u64>,
    color_submenu_open: bool,
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
        let app = Self {
            tabs: vec![],
            active_tab: 0,
            next_id: 0,
            last_size: None,
            renaming: None,
            tab_menu_open: None,
            color_submenu_open: false,
        };
        (app, Task::none())
    }

    fn create_tab(&mut self, insert_at: usize, cwd: Option<PathBuf>) -> Task<Message> {
        let id = self.next_id;
        self.next_id += 1;
        match terminal::new_terminal(id, cwd) {
            Ok(mut term) => {
                if let Some(size) = self.last_size {
                    term.handle(iced_term::Command::ProxyToBackend(
                        iced_term::BackendCommand::Resize(Some(size), None),
                    ));
                }
                let focus = iced_term::TerminalView::focus::<Message>(term.widget_id().clone());
                let tab = Tab {
                    id, title: "Terminal".into(), custom_title: None, color: None, term,
                };
                let pos = insert_at.min(self.tabs.len());
                self.tabs.insert(pos, tab);
                self.active_tab = pos;
                focus
            }
            Err(_) => Task::none(),
        }
    }

    fn focus_active_terminal(&self) -> Task<Message> {
        self.tabs.get(self.active_tab).map_or(Task::none(), |tab| {
            iced_term::TerminalView::focus::<Message>(tab.term.widget_id().clone())
        })
    }

    fn close_tab(&mut self, idx: usize) -> Task<Message> {
        if idx >= self.tabs.len() {
            return Task::none();
        }
        let removed_id = self.tabs[idx].id;
        self.tabs.remove(idx);
        self.dismiss_menu();
        if self.renaming.as_ref().is_some_and(|r| r.tab_id == removed_id) {
            self.renaming = None;
        }
        if self.tabs.is_empty() {
            return Task::none();
        }
        self.active_tab = self.active_tab.min(self.tabs.len() - 1);
        self.focus_active_terminal()
    }

    fn dismiss_menu(&mut self) {
        self.tab_menu_open = None;
        self.color_submenu_open = false;
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TermEvent(iced_term::Event::BackendCall(term_id, cmd)) => {
                if let iced_term::BackendCommand::Resize(Some(layout), Some(_)) = &cmd {
                    self.last_size = Some(*layout);
                }
                let action = self.tabs.iter_mut()
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
                    _ => {}
                }
                Task::none()
            }
            Message::NewTab => {
                self.dismiss_menu();
                self.create_tab(self.tabs.len(), None)
            }
            Message::DuplicateTab(idx) => {
                self.dismiss_menu();
                let cwd = self.tabs.get(idx).and_then(|t| terminal::extract_cwd(&t.title));
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
                if self.tab_menu_open == Some(tab_id) {
                    self.dismiss_menu();
                } else {
                    self.dismiss_menu();
                    self.tab_menu_open = Some(tab_id);
                }
                Task::none()
            }
            Message::CloseTabMenu => { self.dismiss_menu(); Task::none() }
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
                if let Some(s) = &mut self.renaming { s.input = val; }
                Task::none()
            }
            Message::ConfirmRename => {
                if let Some(state) = self.renaming.take() {
                    if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == state.tab_id) {
                        let trimmed = state.input.trim();
                        tab.custom_title =
                            if trimmed.is_empty() { None } else { Some(trimmed.into()) };
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
            Message::OpenColorSubmenu => { self.color_submenu_open = true; Task::none() }
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

    fn view(&self) -> Element<'_, Message> {
        let bar = titlebar::view_titlebar(&self.tabs, self.active_tab, &self.renaming);

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
            home::view_home()
        };

        let main_content: Element<Message> = column![bar, terminal_view]
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        if let Some(menu_tab_id) = self.tab_menu_open {
            if let Some(menu_idx) = self.tabs.iter().position(|t| t.id == menu_tab_id) {
                let menu_x = self.tab_menu_x_offset(menu_idx);
                return self.view_menu_overlay(main_content, menu_idx, menu_x);
            }
        }
        main_content
    }

    fn view_menu_overlay<'a>(
        &'a self, main_content: Element<'a, Message>, menu_idx: usize, menu_x: f32,
    ) -> Element<'a, Message> {
        let menu_overlay: Element<Message> = container(
            menu::view_tab_menu(menu_idx, self.color_submenu_open),
        )
        .padding(iced::Padding { top: TITLEBAR_H, left: menu_x, right: 0.0, bottom: 0.0 })
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        let scrim: Element<Message> = mouse_area(
            container(Space::new()).width(Length::Fill).height(Length::Fill),
        )
        .on_press(Message::CloseTabMenu)
        .into();

        let mut layers: Vec<Element<Message>> = vec![main_content, scrim, menu_overlay];

        if self.color_submenu_open {
            if let Some(tab) = self.tabs.get(menu_idx) {
                let color_panel: Element<Message> = container(menu::view_color_panel(tab))
                    .padding(iced::Padding {
                        top: TITLEBAR_H + 60.0, left: menu_x + 140.0, right: 0.0, bottom: 0.0,
                    })
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();
                layers.push(color_panel);
            }
        }

        stack(layers).width(Length::Fill).height(Length::Fill).into()
    }

    fn tab_menu_x_offset(&self, idx: usize) -> f32 {
        let mut x = TAB_ROW_LEFT;
        for (i, tab) in self.tabs.iter().enumerate() {
            if i == idx { break; }
            let is_renaming = self.renaming.as_ref().is_some_and(|r| r.tab_id == tab.id);
            x += tab.estimated_width(is_renaming) + TAB_ROW_SPACING;
        }
        x
    }

    fn theme(&self) -> Theme { Theme::Dark }

    fn subscription(&self) -> Subscription<Message> {
        let term_subs = self.tabs.iter()
            .map(|tab| tab.term.subscription().map(Message::TermEvent));
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
                            return Some(Message::NewTab);
                        }
                    }
                }
            }
            None
        });
        Subscription::batch(term_subs.chain(std::iter::once(key_sub)))
    }
}
