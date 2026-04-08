use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static CACHED_SHELL: OnceLock<(String, Vec<String>)> = OnceLock::new();

pub fn default_shell() -> (String, Vec<String>) {
    CACHED_SHELL.get_or_init(detect_shell).clone()
}

fn detect_shell() -> (String, Vec<String>) {
    #[cfg(windows)]
    {
        if which("powershell.exe") {
            ("powershell.exe".into(), vec!["-NoLogo".into()])
        } else if which("pwsh.exe") {
            ("pwsh.exe".into(), vec!["-NoLogo".into()])
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

#[allow(dead_code)]
pub fn extract_cwd(title: &str) -> Option<PathBuf> {
    let candidates: Vec<&str> = vec![
        title.trim(),
        title.strip_prefix("Administrator: ").unwrap_or("").trim(),
    ];
    let embedded = extract_embedded_path(title);
    for candidate in candidates.into_iter().chain(embedded.as_deref()) {
        if candidate.is_empty() { continue; }
        let path = Path::new(candidate);
        if path.is_absolute() && path.is_dir() {
            return Some(path.to_path_buf());
        }
    }
    None
}

#[allow(dead_code)]
fn extract_embedded_path(title: &str) -> Option<String> {
    if let Some(idx) = title.find(":\\") {
        if idx > 0 {
            let start = idx - 1;
            let candidate = title[start..].trim_end_matches(&[' ', '>', ']', ')'][..]);
            return Some(candidate.to_string());
        }
    }
    if let Some(idx) = title.find('/') {
        let candidate = title[idx..].split_whitespace().next()?;
        return Some(candidate.to_string());
    }
    None
}

#[cfg(windows)]
fn which(name: &str) -> bool {
    let path_var = std::env::var("PATH").unwrap_or_default();
    path_var.split(';').any(|dir| Path::new(dir).join(name).exists())
}
