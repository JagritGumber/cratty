use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use iced::{Subscription, Task};
use std::sync::OnceLock;
#[cfg(windows)]
use windows::Win32::{Foundation::{BOOL, HWND, LPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{EnumThreadWindows, IsWindowVisible, SetForegroundWindow,
        ShowWindow, SW_HIDE, SW_SHOW}};
use crate::{message::Message, Cratty};

pub struct QuakeHotkey { _manager: GlobalHotKeyManager }

impl QuakeHotkey {
    pub fn register(hotkey_str: &str) -> Option<Self> {
        let manager = GlobalHotKeyManager::new().ok()?;
        manager.register(parse_hotkey(hotkey_str)?).ok()?;
        tracing::info!("Quake hotkey registered: {hotkey_str}");
        Some(Self { _manager: manager })
    }
}

#[cfg(windows)]
static MAIN_HWND: OnceLock<isize> = OnceLock::new();

#[cfg(windows)]
fn find_main_hwnd() -> Option<HWND> {
    if let Some(h) = MAIN_HWND.get() { return Some(HWND(*h as *mut _)); }
    let mut found: isize = 0;
    unsafe extern "system" fn cb(hwnd: HWND, lp: LPARAM) -> BOOL { unsafe {
        if IsWindowVisible(hwnd).as_bool() { *(lp.0 as *mut isize) = hwnd.0 as isize; BOOL(0) } else { BOOL(1) }
    } }
    unsafe {
        let _ = EnumThreadWindows(GetCurrentThreadId(), Some(cb), LPARAM(&mut found as *mut _ as isize));
        if found == 0 { return None; }
        let _ = MAIN_HWND.set(found);
        Some(HWND(found as *mut _))
    }
}

impl Cratty {
    pub fn handle_toggle_quake(&mut self) -> Task<Message> {
        #[cfg(windows)]
        if let Some(hwnd) = find_main_hwnd() {
            unsafe {
                if self.quake_hidden {
                    let _ = ShowWindow(hwnd, SW_SHOW);
                    let _ = SetForegroundWindow(hwnd);
                    self.quake_hidden = false;
                } else { let _ = ShowWindow(hwnd, SW_HIDE); self.quake_hidden = true; }
            }
        }
        Task::none()
    }
}

pub fn subscription() -> Subscription<Message> {
    Subscription::run(|| {
        iced::stream::channel(10, |mut tx: iced::futures::channel::mpsc::Sender<Message>| async move {
            let rx = GlobalHotKeyEvent::receiver();
            loop {
                while let Ok(event) = rx.try_recv() {
                    if event.state == HotKeyState::Pressed { let _ = tx.try_send(Message::ToggleQuake); }
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        })
    })
}

fn parse_hotkey(s: &str) -> Option<HotKey> {
    let (mut mods, mut key) = (Modifiers::empty(), None);
    for part in s.split('+').map(str::trim) {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" => mods |= Modifiers::ALT,
            "shift" => mods |= Modifiers::SHIFT,
            "super" | "win" | "meta" => mods |= Modifiers::SUPER,
            "`" | "backquote" | "grave" => key = Some(Code::Backquote),
            "f12" => key = Some(Code::F12),
            "f11" => key = Some(Code::F11),
            "space" => key = Some(Code::Space),
            _ => { tracing::warn!("Unknown hotkey part: {part}"); return None; }
        }
    }
    Some(HotKey::new(Some(mods), key?))
}
