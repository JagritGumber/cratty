use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use iced::{window, Size, Subscription, Task};

use crate::message::Message;
use crate::{with_window, Cratty};

pub struct QuakeHotkey {
    _manager: GlobalHotKeyManager,
}

impl QuakeHotkey {
    pub fn register(hotkey_str: &str) -> Option<Self> {
        let manager = GlobalHotKeyManager::new().ok()?;
        let hotkey = parse_hotkey(hotkey_str)?;
        manager.register(hotkey).ok()?;
        tracing::info!("Quake hotkey registered: {hotkey_str}");
        Some(Self { _manager: manager })
    }
}

impl Cratty {
    /// Toggle the quake dropdown: if hidden, slide into position; if visible, hide.
    pub fn handle_toggle_quake(&mut self) -> Task<Message> {
        let height_pct = self.config.quake_mode.height_percent;
        with_window(move |id| {
            window::is_minimized(id).then(move |minimized| {
                if !minimized.unwrap_or(false) {
                    return window::minimize(id, true);
                }
                window::monitor_size(id).then(move |mon| {
                    let (w, h) = mon.map(|s| (s.width, s.height)).unwrap_or((1280.0, 720.0));
                    let target = Size::new(w, h * height_pct);
                    Task::batch([
                        window::minimize(id, false),
                        window::move_to(id, iced::Point::new(0.0, 0.0)),
                        window::resize(id, target),
                        window::gain_focus(id),
                    ])
                })
            })
        })
    }
}

pub fn subscription() -> Subscription<Message> {
    Subscription::run(|| {
        iced::stream::channel(10, |mut tx: iced::futures::channel::mpsc::Sender<Message>| async move {
            let rx = GlobalHotKeyEvent::receiver();
            loop {
                if rx.try_recv().is_ok() {
                    let _ = tx.try_send(Message::ToggleQuake);
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        })
    })
}

fn parse_hotkey(s: &str) -> Option<HotKey> {
    let parts: Vec<&str> = s.split('+').map(str::trim).collect();
    let mut mods = Modifiers::empty();
    let mut key = None;
    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" => mods |= Modifiers::ALT,
            "shift" => mods |= Modifiers::SHIFT,
            "super" | "win" | "meta" => mods |= Modifiers::SUPER,
            "`" | "backquote" | "grave" => key = Some(Code::Backquote),
            "f12" => key = Some(Code::F12),
            "f11" => key = Some(Code::F11),
            "space" => key = Some(Code::Space),
            _ => {
                tracing::warn!("Unknown hotkey part: {part}");
                return None;
            }
        }
    }
    Some(HotKey::new(Some(mods), key?))
}
