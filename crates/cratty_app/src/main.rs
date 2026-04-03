use std::collections::HashMap;
use std::path::PathBuf;

use iced::keyboard;
use iced::widget::{column, container, mouse_area, stack, Space};
use iced::window;
use iced::{event, Color, Element, Length, Subscription, Task, Theme};

use cratty_core::{IdGen, PaneId, Workspace, WorkspaceId};

mod home;
mod menu;
mod message;
mod pane;
mod style;
mod terminal;
mod titlebar;
mod widgets;

use message::{Message, RenameState};
use pane::Pane;
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
    workspaces: Vec<Workspace>,
    active_ws: usize,
    panes: HashMap<PaneId, Pane>,
    ws_colors: HashMap<WorkspaceId, Color>,
    id_gen: IdGen,
    last_size: Option<iced::Size>,
    renaming: Option<RenameState>,
    tab_menu_open: Option<WorkspaceId>,
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
        (Self {
            workspaces: vec![],
            active_ws: 0,
            panes: HashMap::new(),
            ws_colors: HashMap::new(),
            id_gen: IdGen::new(),
            last_size: None,
            renaming: None,
            tab_menu_open: None,
            color_submenu_open: false,
        }, Task::none())
    }

    fn create_workspace(&mut self, cwd: Option<PathBuf>) -> Task<Message> {
        let ws_id = self.id_gen.next_workspace();
        let pane_id = self.id_gen.next_pane();
        let term_id = pane_id.0;

        match terminal::new_terminal(term_id, cwd) {
            Ok(mut term) => {
                if let Some(size) = self.last_size {
                    term.handle(iced_term::Command::ProxyToBackend(
                        iced_term::BackendCommand::Resize(Some(size), None),
                    ));
                }
                let focus = iced_term::TerminalView::focus::<Message>(term.widget_id().clone());
                self.panes.insert(pane_id, Pane::new(pane_id, term_id, term));

                let mut ws = Workspace::new(ws_id, format!("Terminal {}", ws_id.0));
                ws.strip.push(pane_id);
                self.workspaces.push(ws);
                self.active_ws = self.workspaces.len() - 1;
                focus
            }
            Err(_) => Task::none(),
        }
    }

    fn focused_pane(&self) -> Option<&Pane> {
        let ws = self.workspaces.get(self.active_ws)?;
        let pane_id = ws.focused_pane()?;
        self.panes.get(&pane_id)
    }

    fn focus_active_terminal(&self) -> Task<Message> {
        self.focused_pane().map_or(Task::none(), |pane| {
            iced_term::TerminalView::focus::<Message>(pane.terminal.widget_id().clone())
        })
    }

    fn close_workspace(&mut self, idx: usize) -> Task<Message> {
        if idx >= self.workspaces.len() {
            return Task::none();
        }
        let ws = self.workspaces.remove(idx);
        for pane_id in &ws.strip.panes {
            self.panes.remove(pane_id);
        }
        self.ws_colors.remove(&ws.id);
        self.dismiss_menu();
        if let Some(r) = &self.renaming {
            if r.workspace_id == ws.id { self.renaming = None; }
        }
        if self.workspaces.is_empty() {
            return Task::none();
        }
        self.active_ws = self.active_ws.min(self.workspaces.len() - 1);
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
                let action = self.panes.values_mut()
                    .find(|p| p.term_id == term_id)
                    .map(|p| p.terminal.handle(iced_term::Command::ProxyToBackend(cmd)));
                match action {
                    Some(iced_term::actions::Action::Shutdown) => {
                        // Find and remove the pane, close workspace if empty
                        if let Some(pid) = self.panes.values()
                            .find(|p| p.term_id == term_id).map(|p| p.id)
                        {
                            self.panes.remove(&pid);
                            for ws in &mut self.workspaces {
                                ws.strip.remove(pid);
                            }
                            // Remove empty workspaces
                            if let Some(idx) = self.workspaces.iter()
                                .position(|ws| ws.is_empty())
                            {
                                let ws = self.workspaces.remove(idx);
                                self.ws_colors.remove(&ws.id);
                                if self.active_ws >= self.workspaces.len()
                                    && !self.workspaces.is_empty()
                                {
                                    self.active_ws = self.workspaces.len() - 1;
                                }
                            }
                        }
                        return self.focus_active_terminal();
                    }
                    Some(iced_term::actions::Action::ChangeTitle(title)) => {
                        if let Some(p) = self.panes.values_mut()
                            .find(|p| p.term_id == term_id)
                        {
                            p.title = title;
                        }
                    }
                    _ => {}
                }
                Task::none()
            }
            Message::NewWorkspace => {
                self.dismiss_menu();
                self.create_workspace(None)
            }
            Message::DuplicateWorkspace(idx) => {
                self.dismiss_menu();
                let cwd = self.workspaces.get(idx)
                    .and_then(|ws| ws.focused_pane())
                    .and_then(|pid| self.panes.get(&pid))
                    .and_then(|p| terminal::extract_cwd(&p.title));
                self.create_workspace(cwd)
            }
            Message::CloseWorkspace(idx) => self.close_workspace(idx),
            Message::SwitchWorkspace(idx) => {
                if self.renaming.is_some() || idx >= self.workspaces.len() {
                    return Task::none();
                }
                self.dismiss_menu();
                self.active_ws = idx;
                self.focus_active_terminal()
            }
            Message::ToggleTabMenu(ws_id) => {
                if self.tab_menu_open == Some(ws_id) {
                    self.dismiss_menu();
                } else {
                    self.dismiss_menu();
                    self.tab_menu_open = Some(ws_id);
                }
                Task::none()
            }
            Message::CloseTabMenu => { self.dismiss_menu(); Task::none() }
            Message::StartRename(idx) => {
                self.dismiss_menu();
                if let Some(ws) = self.workspaces.get(idx) {
                    self.renaming = Some(RenameState {
                        workspace_id: ws.id,
                        input: ws.name.clone(),
                    });
                    iced::widget::operation::focus_next()
                } else { Task::none() }
            }
            Message::RenameInput(val) => {
                if let Some(s) = &mut self.renaming { s.input = val; }
                Task::none()
            }
            Message::ConfirmRename => {
                if let Some(state) = self.renaming.take() {
                    if let Some(ws) = self.workspaces.iter_mut()
                        .find(|w| w.id == state.workspace_id)
                    {
                        let trimmed = state.input.trim();
                        if !trimmed.is_empty() { ws.name = trimmed.into(); }
                    }
                }
                self.focus_active_terminal()
            }
            Message::SetWorkspaceColor(ws_id, color) => {
                match color {
                    Some(c) => { self.ws_colors.insert(ws_id, c); }
                    None => { self.ws_colors.remove(&ws_id); }
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
        let bar = titlebar::view_titlebar(
            &self.workspaces, &self.ws_colors, self.active_ws, &self.renaming,
        );
        let terminal_view: Element<Message> = if let Some(pane) = self.focused_pane() {
            iced::widget::keyed_column(std::iter::once((
                pane.term_id,
                container(
                    iced_term::TerminalView::show(&pane.terminal).map(Message::TermEvent),
                )
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
            .width(Length::Fill).height(Length::Fill).into();

        if let Some(menu_ws_id) = self.tab_menu_open {
            if let Some(menu_idx) = self.workspaces.iter().position(|w| w.id == menu_ws_id) {
                let menu_x = titlebar::tab_menu_x_offset(
                    &self.workspaces, &self.renaming, menu_idx,
                );
                return self.view_menu_overlay(main_content, menu_idx, menu_x);
            }
        }
        main_content
    }

    fn view_menu_overlay<'a>(
        &'a self, main_content: Element<'a, Message>, idx: usize, menu_x: f32,
    ) -> Element<'a, Message> {
        let ws = &self.workspaces[idx];
        let menu_overlay: Element<Message> = container(
            menu::view_tab_menu(idx, self.color_submenu_open),
        )
        .padding(iced::Padding { top: TITLEBAR_H, left: menu_x, right: 0.0, bottom: 0.0 })
        .width(Length::Fill).height(Length::Fill).into();

        let scrim: Element<Message> = mouse_area(
            container(Space::new()).width(Length::Fill).height(Length::Fill),
        ).on_press(Message::CloseTabMenu).into();

        let mut layers: Vec<Element<Message>> = vec![main_content, scrim, menu_overlay];
        if self.color_submenu_open {
            let color = self.ws_colors.get(&ws.id).copied();
            let panel: Element<Message> = container(
                menu::view_color_panel(ws.id, color),
            )
            .padding(iced::Padding {
                top: TITLEBAR_H + 60.0, left: menu_x + 140.0, right: 0.0, bottom: 0.0,
            })
            .width(Length::Fill).height(Length::Fill).into();
            layers.push(panel);
        }
        stack(layers).width(Length::Fill).height(Length::Fill).into()
    }

    fn theme(&self) -> Theme { Theme::Dark }

    fn subscription(&self) -> Subscription<Message> {
        let term_subs = self.panes.values()
            .map(|p| p.terminal.subscription().map(Message::TermEvent));
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
                    }
                }
            }
            None
        });
        Subscription::batch(term_subs.chain(std::iter::once(key_sub)))
    }
}
