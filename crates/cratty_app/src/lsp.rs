use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Instant;

use serde_json::{Value, json};

use crate::code_editor::CodeLanguage;
use cratty_core::AppConfig;

pub const CHANGE_DEBOUNCE_MS: u64 = 250;
pub const HOVER_DEBOUNCE_MS: u64 = 350;

#[derive(Debug, Clone)]
pub enum LspStatus {
    Disabled,
    Starting,
    Installing(String),
    Ready,
    MissingServer(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct LspDiagnostic {
    pub line: usize,
    pub end_line: usize,
    pub severity: Option<u64>,
    pub message: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HoverInfo {
    pub contents: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct EditorLspState {
    pub diagnostics: Vec<LspDiagnostic>,
    pub hover: Option<HoverInfo>,
    pub version: i32,
    pub pending_sync_at: Option<Instant>,
    pub pending_hover_at: Option<Instant>,
    pub cursor_line: usize,
    pub cursor_column: usize,
}

impl Default for EditorLspState {
    fn default() -> Self {
        Self {
            diagnostics: Vec::new(),
            hover: None,
            version: 1,
            pending_sync_at: None,
            pending_hover_at: None,
            cursor_line: 0,
            cursor_column: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LspRequestKind {
    Initialize,
    Hover { line: usize, column: usize },
}

#[derive(Debug)]
pub enum LspEvent {
    Notification { method: String, params: Value },
    Response { id: u64, result: Option<Value>, error: Option<Value> },
    Stderr(String),
    Exited,
}

enum LspCommand {
    Notification { method: String, params: Value },
    Request { id: u64, method: String, params: Value },
    Shutdown { id: u64 },
}

pub struct LspSession {
    pub rx: mpsc::Receiver<LspEvent>,
    tx: mpsc::Sender<LspCommand>,
    child: Arc<Mutex<Child>>,
}

impl LspSession {
    pub fn start(command: &str, args: &[&str], cwd: &Path) -> anyhow::Result<Self> {
        let mut child = Command::new(command)
            .args(args)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("missing stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("missing stdout"))?;
        let stderr = child.stderr.take().ok_or_else(|| anyhow::anyhow!("missing stderr"))?;

        let child = Arc::new(Mutex::new(child));
        let (tx, rx_cmd) = mpsc::channel();
        let (tx_evt, rx) = mpsc::channel();

        spawn_writer(stdin, rx_cmd);
        spawn_reader(stdout, tx_evt.clone());
        spawn_stderr(stderr, tx_evt.clone());

        let child_for_wait = child.clone();
        thread::spawn(move || {
            let _ = child_for_wait.lock().ok().and_then(|mut c| c.wait().ok());
            let _ = tx_evt.send(LspEvent::Exited);
        });

        Ok(Self { rx, tx, child })
    }

    pub fn notify(&self, method: &str, params: Value) {
        let _ = self.tx.send(LspCommand::Notification {
            method: method.to_string(),
            params,
        });
    }

    pub fn request(&self, id: u64, method: &str, params: Value) {
        let _ = self.tx.send(LspCommand::Request {
            id,
            method: method.to_string(),
            params,
        });
    }

    pub fn shutdown(&self, id: u64) {
        let _ = self.tx.send(LspCommand::Shutdown { id });
    }

    pub fn kill(&self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
        }
    }
}

fn spawn_writer(mut stdin: ChildStdin, rx_cmd: mpsc::Receiver<LspCommand>) {
    thread::spawn(move || {
        while let Ok(command) = rx_cmd.recv() {
            let payload = match command {
                LspCommand::Notification { method, params } => json!({
                    "jsonrpc": "2.0",
                    "method": method,
                    "params": params,
                }),
                LspCommand::Request { id, method, params } => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "method": method,
                    "params": params,
                }),
                LspCommand::Shutdown { id } => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "method": "shutdown",
                    "params": Value::Null,
                }),
            };

            if write_message(&mut stdin, &payload).is_err() {
                break;
            }
        }
    });
}

fn spawn_reader(stdout: impl Read + Send + 'static, tx_evt: mpsc::Sender<LspEvent>) {
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        while let Ok(Some(message)) = read_message(&mut reader) {
            if let Some(method) = message.get("method").and_then(Value::as_str) {
                let params = message.get("params").cloned().unwrap_or(Value::Null);
                let _ = tx_evt.send(LspEvent::Notification {
                    method: method.to_string(),
                    params,
                });
            } else if let Some(id) = message.get("id").and_then(Value::as_u64) {
                let _ = tx_evt.send(LspEvent::Response {
                    id,
                    result: message.get("result").cloned(),
                    error: message.get("error").cloned(),
                });
            }
        }
        let _ = tx_evt.send(LspEvent::Exited);
    });
}

fn spawn_stderr(stderr: impl Read + Send + 'static, tx_evt: mpsc::Sender<LspEvent>) {
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            tracing::debug!("lsp stderr: {line}");
            let _ = tx_evt.send(LspEvent::Stderr(line));
        }
    });
}

fn write_message(writer: &mut impl Write, payload: &Value) -> anyhow::Result<()> {
    let text = payload.to_string();
    let header = format!("Content-Length: {}\r\n\r\n", text.len());
    writer.write_all(header.as_bytes())?;
    writer.write_all(text.as_bytes())?;
    writer.flush()?;
    Ok(())
}

fn read_message(reader: &mut BufReader<impl Read>) -> anyhow::Result<Option<Value>> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            return Ok(None);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            content_length = value.trim().parse::<usize>().ok();
        }
    }

    let Some(content_length) = content_length else {
        return Ok(None);
    };
    let mut buf = vec![0; content_length];
    reader.read_exact(&mut buf)?;
    let message = serde_json::from_slice::<Value>(&buf)?;
    Ok(Some(message))
}

pub struct LanguageServerConfig {
    pub language_id: &'static str,
    pub command: &'static str,
    pub args: &'static [&'static str],
}

pub struct ResolvedLanguageServerConfig {
    pub command: String,
    pub args: Vec<String>,
}

pub fn server_config(language: CodeLanguage) -> Option<LanguageServerConfig> {
    match language {
        CodeLanguage::Rust => Some(LanguageServerConfig {
            language_id: "rust",
            command: "rust-analyzer",
            args: &[],
        }),
        CodeLanguage::TypeScript | CodeLanguage::JavaScript => Some(LanguageServerConfig {
            language_id: "typescript",
            command: "typescript-language-server",
            args: &["--stdio"],
        }),
        CodeLanguage::Json => Some(LanguageServerConfig {
            language_id: "json",
            command: "vscode-json-language-server",
            args: &["--stdio"],
        }),
        CodeLanguage::Yaml => Some(LanguageServerConfig {
            language_id: "yaml",
            command: "yaml-language-server",
            args: &["--stdio"],
        }),
        CodeLanguage::Toml => Some(LanguageServerConfig {
            language_id: "toml",
            command: "taplo",
            args: &["lsp", "stdio"],
        }),
        CodeLanguage::Markdown => Some(LanguageServerConfig {
            language_id: "markdown",
            command: "marksman",
            args: &["server"],
        }),
        CodeLanguage::Shell => Some(LanguageServerConfig {
            language_id: "shellscript",
            command: "bash-language-server",
            args: &["start"],
        }),
        CodeLanguage::PlainText => None,
    }
}

pub fn resolved_server_config(language: CodeLanguage) -> Option<ResolvedLanguageServerConfig> {
    let config = server_config(language)?;
    let command = match language {
        CodeLanguage::Toml => managed_taplo_path()
            .filter(|path| path.exists())
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| config.command.to_string()),
        _ => config.command.to_string(),
    };

    Some(ResolvedLanguageServerConfig {
        command,
        args: config.args.iter().map(|arg| (*arg).to_string()).collect(),
    })
}

#[derive(Debug, Clone)]
pub struct InstallPlan {
    pub label: &'static str,
    pub strategy: InstallStrategy,
}

#[derive(Debug, Clone)]
pub enum InstallStrategy {
    Command {
        program: &'static str,
        args: &'static [&'static str],
    },
    #[cfg(windows)]
    ManagedTaploBinary,
}

pub fn install_plan(language: CodeLanguage) -> Option<InstallPlan> {
    match language {
        CodeLanguage::Rust => Some(InstallPlan {
            label: "rust-analyzer",
            strategy: InstallStrategy::Command {
                program: "rustup",
                args: &["component", "add", "rust-analyzer", "rust-src"],
            },
        }),
        CodeLanguage::TypeScript | CodeLanguage::JavaScript => Some(InstallPlan {
            label: "TypeScript language server",
            strategy: InstallStrategy::Command {
                program: "npm",
                args: &["install", "-g", "typescript-language-server", "typescript"],
            },
        }),
        CodeLanguage::Json => Some(InstallPlan {
            label: "JSON language server",
            strategy: InstallStrategy::Command {
                program: "npm",
                args: &["install", "-g", "vscode-langservers-extracted"],
            },
        }),
        CodeLanguage::Yaml => Some(InstallPlan {
            label: "YAML language server",
            strategy: InstallStrategy::Command {
                program: "npm",
                args: &["install", "-g", "yaml-language-server"],
            },
        }),
        CodeLanguage::Toml => Some(InstallPlan {
            label: "Taplo",
            #[cfg(windows)]
            strategy: InstallStrategy::ManagedTaploBinary,
            #[cfg(not(windows))]
            strategy: InstallStrategy::Command {
                program: "cargo",
                args: &["install", "taplo-cli", "--locked", "--features", "lsp"],
            },
        }),
        CodeLanguage::Shell => Some(InstallPlan {
            label: "bash-language-server",
            strategy: InstallStrategy::Command {
                program: "npm",
                args: &["install", "-g", "bash-language-server"],
            },
        }),
        CodeLanguage::Markdown => None,
        CodeLanguage::PlainText => None,
    }
}

pub fn is_missing_program_error(err: &anyhow::Error) -> bool {
    if let Some(io_err) = err.downcast_ref::<io::Error>() {
        return io_err.kind() == io::ErrorKind::NotFound;
    }

    let text = err.to_string().to_ascii_lowercase();
    text.contains("os error 2")
        || text.contains("program not found")
        || text.contains("not found")
        || text.contains("the system cannot find the file specified")
}

pub fn managed_tools_dir() -> PathBuf {
    AppConfig::config_dir().join("tools")
}

pub fn managed_taplo_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        Some(managed_tools_dir().join("taplo").join("taplo.exe"))
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
pub fn install_taplo_binary() -> anyhow::Result<()> {
    let install_dir = managed_tools_dir().join("taplo");
    std::fs::create_dir_all(&install_dir)?;

    let zip_path = install_dir.join("taplo-windows-x86_64.zip");
    let extract_dir = install_dir.join("_extract");
    if extract_dir.exists() {
        std::fs::remove_dir_all(&extract_dir)?;
    }
    std::fs::create_dir_all(&extract_dir)?;

    let script = format!(
        concat!(
            "$ErrorActionPreference='Stop';",
            "$ProgressPreference='SilentlyContinue';",
            "Invoke-WebRequest -Uri 'https://github.com/tamasfe/taplo/releases/latest/download/taplo-windows-x86_64.zip' -OutFile '{}';",
            "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force;"
        ),
        ps_single_quote(&zip_path),
        ps_single_quote(&zip_path),
        ps_single_quote(&extract_dir),
    );

    let status = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Taplo download exited with {status}"));
    }

    let extracted = find_file_named(&extract_dir, "taplo.exe")
        .ok_or_else(|| anyhow::anyhow!("taplo.exe not found in downloaded archive"))?;
    let final_path = install_dir.join("taplo.exe");
    std::fs::copy(&extracted, &final_path)?;

    let _ = std::fs::remove_file(&zip_path);
    let _ = std::fs::remove_dir_all(&extract_dir);
    Ok(())
}

#[cfg(windows)]
fn find_file_named(root: &Path, file_name: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case(file_name))
            {
                return Some(path);
            }
        }
    }
    None
}

#[cfg(windows)]
fn ps_single_quote(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

pub fn path_to_uri(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

pub fn initialize_params(root: &Path) -> Value {
    let uri = path_to_uri(root);
    let name = root.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace");
    json!({
        "processId": std::process::id(),
        "clientInfo": { "name": "cratty", "version": "0.1.0" },
        "rootUri": uri,
        "workspaceFolders": [{ "uri": uri, "name": name }],
        "capabilities": {
            "textDocument": {
                "hover": { "dynamicRegistration": false },
                "publishDiagnostics": { "relatedInformation": true },
                "synchronization": {
                    "dynamicRegistration": false,
                    "didSave": true,
                    "willSave": false,
                    "willSaveWaitUntil": false
                }
            }
        }
    })
}

pub fn did_open_params(uri: &str, language_id: &str, version: i32, text: String) -> Value {
    json!({
        "textDocument": {
            "uri": uri,
            "languageId": language_id,
            "version": version,
            "text": text,
        }
    })
}

pub fn did_change_params(uri: &str, version: i32, text: String) -> Value {
    json!({
        "textDocument": {
            "uri": uri,
            "version": version,
        },
        "contentChanges": [{ "text": text }]
    })
}

pub fn did_save_params(uri: &str, text: String) -> Value {
    json!({
        "textDocument": { "uri": uri },
        "text": text,
    })
}

pub fn did_close_params(uri: &str) -> Value {
    json!({
        "textDocument": { "uri": uri },
    })
}

pub fn hover_params(uri: &str, line: usize, character: usize) -> Value {
    json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": character },
    })
}

pub fn parse_diagnostics(params: &Value) -> Vec<LspDiagnostic> {
    params.get("diagnostics")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|diag| {
            let range = diag.get("range")?;
            let start = range.get("start")?;
            let end = range.get("end")?;
            Some(LspDiagnostic {
                line: start.get("line").and_then(Value::as_u64).unwrap_or(0) as usize,
                end_line: end.get("line").and_then(Value::as_u64).unwrap_or(0) as usize,
                severity: diag.get("severity").and_then(Value::as_u64),
                message: diag.get("message").and_then(Value::as_str).unwrap_or("").to_string(),
                source: diag.get("source").and_then(Value::as_str).map(str::to_string),
            })
        })
        .collect()
}

pub fn hover_text(result: &Value) -> Option<String> {
    let contents = result.get("contents")?;
    match contents {
        Value::String(text) => Some(text.clone()),
        Value::Array(items) => {
            let mut out = String::new();
            for item in items {
                if let Some(text) = marked_string_text(item) {
                    if !out.is_empty() {
                        out.push_str("\n\n");
                    }
                    out.push_str(&text);
                }
            }
            if out.is_empty() { None } else { Some(out) }
        }
        Value::Object(map) => {
            if let Some(value) = map.get("value").and_then(Value::as_str) {
                Some(value.to_string())
            } else {
                marked_string_text(contents)
            }
        }
        _ => None,
    }
}

fn marked_string_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Object(map) => {
            if let Some(value) = map.get("value").and_then(Value::as_str) {
                Some(value.to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn utf16_col(line_text: &str, column: usize) -> usize {
    line_text.chars().take(column).map(char::len_utf16).sum()
}

pub fn root_for(path: &Path, workspace_root: Option<&PathBuf>) -> PathBuf {
    workspace_root
        .cloned()
        .or_else(|| path.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn summarize_diagnostics(diags: &[LspDiagnostic]) -> (usize, usize) {
    let mut errors = 0;
    let mut warnings = 0;
    for diag in diags {
        match diag.severity.unwrap_or(1) {
            1 => errors += 1,
            2 => warnings += 1,
            _ => {}
        }
    }
    (errors, warnings)
}
