use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    pub keys: Vec<String>,
    pub action: String,
    #[serde(default)]
    pub context: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct KeybindingSet {
    pub bindings: Vec<Keybinding>,
}

impl KeybindingSet {
    pub fn defaults() -> Self {
        Self {
            bindings: vec![
                Keybinding {
                    keys: vec!["ctrl+shift+t".into()],
                    action: "new_tab".into(),
                    context: None,
                },
                Keybinding {
                    keys: vec!["ctrl+shift+w".into()],
                    action: "close_tab".into(),
                    context: None,
                },
                Keybinding {
                    keys: vec!["ctrl+shift+d".into()],
                    action: "split_horizontal".into(),
                    context: None,
                },
                Keybinding {
                    keys: vec!["ctrl+shift+e".into()],
                    action: "split_vertical".into(),
                    context: None,
                },
                Keybinding {
                    keys: vec!["ctrl+shift+f".into()],
                    action: "search".into(),
                    context: Some("terminal".into()),
                },
                Keybinding {
                    keys: vec!["ctrl+tab".into()],
                    action: "next_tab".into(),
                    context: None,
                },
                Keybinding {
                    keys: vec!["ctrl+shift+tab".into()],
                    action: "prev_tab".into(),
                    context: None,
                },
            ],
        }
    }
}
