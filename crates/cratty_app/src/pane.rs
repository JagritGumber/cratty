use std::path::PathBuf;

use cratty_core::PaneId;
use iced::widget::text_editor;

use crate::code_editor::CodeLanguage;
use crate::lsp::EditorLspState;
use crate::term_backend::TermBackend;

pub enum PaneContent {
    Loading,
    Terminal(TermBackend),
    Editor(CodePane),
}

pub struct CodePane {
    pub path: PathBuf,
    pub content: text_editor::Content,
    pub dirty: bool,
    pub language: CodeLanguage,
    pub lsp: EditorLspState,
}

pub struct Pane {
    pub id: PaneId,
    pub content: PaneContent,
    pub title: String,
    pub custom_title: Option<String>,
    pub cwd: Option<PathBuf>,
}

impl Pane {
    pub fn new_loading(id: PaneId) -> Self {
        Self { id, content: PaneContent::Loading, title: "Loading...".into(),
            custom_title: None, cwd: None }
    }

    pub fn new_code(id: PaneId, path: PathBuf, src: String) -> Self {
        let title = path.file_name().and_then(|n| n.to_str())
            .unwrap_or("Code").to_string();
        let cwd = path.parent().map(PathBuf::from);
        let language = CodeLanguage::detect(&path);
        Self {
            id, title, custom_title: None, cwd,
            content: PaneContent::Editor(CodePane {
                path,
                content: text_editor::Content::with_text(&src),
                dirty: false,
                language,
                lsp: EditorLspState::default(),
            }),
        }
    }

    pub fn display_title(&self) -> &str {
        self.custom_title.as_deref().unwrap_or(&self.title)
    }

    pub fn terminal(&self) -> Option<&TermBackend> {
        match &self.content { PaneContent::Terminal(b) => Some(b), _ => None }
    }

    pub fn terminal_mut(&mut self) -> Option<&mut TermBackend> {
        match &mut self.content { PaneContent::Terminal(b) => Some(b), _ => None }
    }
}
