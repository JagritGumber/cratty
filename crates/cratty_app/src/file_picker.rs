use std::path::{Path, PathBuf};

pub struct FilePickerState {
    pub root: PathBuf,
    pub query: String,
    pub all_files: Vec<PathBuf>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub preview: FilePreview,
}

pub struct FilePreview {
    pub title: String,
    pub body: String,
}

impl FilePickerState {
    pub fn open_for(root: &Path) -> Self {
        let all_files: Vec<PathBuf> = ignore::WalkBuilder::new(root)
            .hidden(true).git_ignore(true).git_exclude(true).git_global(true)
            .build()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().map_or(false, |t| t.is_file()))
            .map(|e| e.path().strip_prefix(root).unwrap_or(e.path()).to_path_buf())
            .take(10_000)
            .collect();
        let mut s = Self {
            root: root.to_path_buf(),
            query: String::new(),
            all_files, filtered: vec![], selected: 0,
            preview: FilePreview { title: String::new(), body: String::new() },
        };
        s.refilter();
        s
    }

    pub fn refilter(&mut self) {
        let q = self.query.to_lowercase();
        self.filtered = self.all_files.iter().enumerate()
            .filter(|(_, p)| q.is_empty() || p.to_string_lossy().to_lowercase().contains(&q))
            .map(|(i, _)| i).take(300).collect();
        self.selected = 0;
        self.refresh_preview();
    }

    pub fn move_selection(&mut self, delta: i32) {
        if self.filtered.is_empty() { return; }
        let len = self.filtered.len() as i32;
        let mut n = self.selected as i32 + delta;
        if n < 0 { n = 0; }
        if n >= len { n = len - 1; }
        self.selected = n as usize;
        self.refresh_preview();
    }

    pub fn select(&mut self, index: usize) {
        if index >= self.filtered.len() {
            return;
        }
        self.selected = index;
        self.refresh_preview();
    }

    pub fn selected_abs_path(&self) -> Option<PathBuf> {
        let idx = *self.filtered.get(self.selected)?;
        let rel = self.all_files.get(idx)?;
        Some(self.root.join(rel))
    }

    fn refresh_preview(&mut self) {
        let Some(abs) = self.selected_abs_path() else {
            self.preview.title = "No file selected".into();
            self.preview.body = "Type to search for a file.".into();
            return;
        };
        let rel = abs.strip_prefix(&self.root).unwrap_or(&abs);
        self.preview.title = rel.to_string_lossy().into_owned();

        match std::fs::read(&abs) {
            Ok(bytes) => {
                if bytes.iter().take(4096).any(|b| *b == 0) {
                    self.preview.body = "[binary file]".into();
                    return;
                }
                let text = String::from_utf8_lossy(&bytes);
                let mut preview = String::new();
                for line in text.lines().take(120) {
                    if !preview.is_empty() {
                        preview.push('\n');
                    }
                    preview.push_str(line);
                    if preview.len() > 12_000 {
                        break;
                    }
                }
                if preview.is_empty() {
                    preview.push_str("[empty file]");
                }
                self.preview.body = preview;
            }
            Err(err) => {
                self.preview.body = format!("[preview unavailable: {err}]");
            }
        }
    }
}
