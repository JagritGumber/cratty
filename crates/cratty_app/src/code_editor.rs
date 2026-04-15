use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodeLanguage {
    Rust,
    TypeScript,
    JavaScript,
    Json,
    Toml,
    Yaml,
    Markdown,
    Shell,
    PlainText,
}

impl CodeLanguage {
    pub fn detect(path: &Path) -> Self {
        let ext = path.extension().and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());
        match ext.as_deref() {
            Some("rs") => Self::Rust,
            Some("ts") | Some("tsx") => Self::TypeScript,
            Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => Self::JavaScript,
            Some("json") => Self::Json,
            Some("toml") => Self::Toml,
            Some("yaml") | Some("yml") => Self::Yaml,
            Some("md") | Some("mdx") => Self::Markdown,
            Some("sh") | Some("bash") | Some("zsh") | Some("ps1") | Some("psm1") => Self::Shell,
            _ => Self::PlainText,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::TypeScript => "TypeScript",
            Self::JavaScript => "JavaScript",
            Self::Json => "JSON",
            Self::Toml => "TOML",
            Self::Yaml => "YAML",
            Self::Markdown => "Markdown",
            Self::Shell => "Shell",
            Self::PlainText => "Plain Text",
        }
    }

    pub fn syntax_token(self) -> &'static str {
        match self {
            Self::Rust => "rs",
            Self::TypeScript => "ts",
            Self::JavaScript => "js",
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Markdown => "md",
            Self::Shell => "sh",
            Self::PlainText => "txt",
        }
    }
}
