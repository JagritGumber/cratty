use std::path::{Path, PathBuf};

pub fn new_terminal(
    id: u64, working_directory: Option<PathBuf>,
) -> std::io::Result<iced_term::Terminal> {
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

pub fn extract_cwd(title: &str) -> Option<PathBuf> {
    let candidates: Vec<&str> = vec![
        title.trim(),
        title.strip_prefix("Administrator: ").unwrap_or("").trim(),
    ];
    let embedded = extract_embedded_path(title);

    for candidate in candidates.into_iter().chain(embedded.as_deref()) {
        if candidate.is_empty() {
            continue;
        }
        let path = Path::new(candidate);
        if path.is_absolute() && path.is_dir() {
            return Some(path.to_path_buf());
        }
    }
    None
}

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
