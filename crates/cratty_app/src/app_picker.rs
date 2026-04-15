use iced::advanced::widget::{operate, operation::focusable};
use iced::widget::{operation, scrollable};
use iced::Task;
use std::collections::HashSet;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::code_editor::CodeLanguage;
use crate::file_picker::FilePickerState;
use crate::lsp::{self, HoverInfo, LspRequestKind, LspStatus};
use crate::message::Message;
use crate::pane::{Pane, PaneContent};
use crate::strip_view::strip_scroll_id;
use crate::{Cratty, PendingLspInstall, PendingLspRequest, Toast, ToastKind, WorkspaceLspServer, WorkspaceLspState};
use cratty_core::{PaneId, WorkspaceId};

impl Cratty {
    pub fn open_file_picker(&mut self) -> Task<Message> {
        let root = self.workspaces.get(self.active_ws).and_then(|w| w.root.clone());
        if let Some(root) = root {
            self.file_picker = Some(FilePickerState::open_for(&root));
            return operate(focusable::focus(crate::file_picker_view::input_id()));
        }
        Task::none()
    }

    pub fn file_picker_input(&mut self, q: String) -> Task<Message> {
        if let Some(fp) = &mut self.file_picker {
            fp.query = q;
            fp.refilter();
        }
        Task::none()
    }

    pub fn file_picker_move(&mut self, d: i32) -> Task<Message> {
        if let Some(fp) = &mut self.file_picker {
            fp.move_selection(d);
        }
        Task::none()
    }

    pub fn file_picker_open(&mut self) -> Task<Message> {
        let abs = match self.file_picker.as_ref().and_then(|fp| fp.selected_abs_path()) {
            Some(p) => p,
            None => return Task::none(),
        };
        self.file_picker = None;
        self.open_editor_for(abs)
    }

    pub fn file_picker_select(&mut self, idx: usize) -> Task<Message> {
        if let Some(fp) = &mut self.file_picker {
            fp.select(idx);
            return operate(focusable::focus(crate::file_picker_view::input_id()));
        }
        Task::none()
    }

    pub fn file_picker_close(&mut self) -> Task<Message> {
        self.file_picker = None;
        Task::none()
    }

    pub fn focus_editor_pane(&self, pane_id: PaneId) -> Task<Message> {
        match self.panes.get(&pane_id).map(|pane| &pane.content) {
            Some(PaneContent::Editor(_)) => {
                operate(focusable::focus(crate::editor_widget::editor_id(pane_id)))
            }
            _ => Task::none(),
        }
    }

    pub fn focus_focused_editor_pane(&self) -> Task<Message> {
        match self.workspaces.get(self.active_ws).and_then(|ws| ws.focused_pane()) {
            Some(pid) => self.focus_editor_pane(pid),
            None => Task::none(),
        }
    }

    pub fn handle_code_action(&mut self, pid: PaneId, action: iced::widget::text_editor::Action) -> Task<Message> {
        if let Some(pane) = self.panes.get_mut(&pid) {
            if let PaneContent::Editor(code) = &mut pane.content {
                let is_edit = action.is_edit();
                code.content.perform(action);
                if is_edit {
                    code.dirty = true;
                    code.lsp.version += 1;
                    code.lsp.pending_sync_at =
                        Some(Instant::now() + Duration::from_millis(lsp::CHANGE_DEBOUNCE_MS));
                }
                let cursor = code.content.cursor().position;
                code.lsp.cursor_line = cursor.line;
                code.lsp.cursor_column = cursor.column;
                code.lsp.pending_hover_at =
                    Some(Instant::now() + Duration::from_millis(lsp::HOVER_DEBOUNCE_MS));
                code.lsp.hover = None;
            }
        }
        Task::none()
    }

    pub fn save_focused_file(&mut self) -> Task<Message> {
        let pid = match self.workspaces.get(self.active_ws).and_then(|ws| ws.focused_pane()) {
            Some(pid) => pid,
            None => return Task::none(),
        };
        let Some(workspace_id) = self.pane_workspace_id(pid) else {
            return Task::none();
        };

        let (language, uri, text) = match self.panes.get_mut(&pid) {
            Some(Pane { content: PaneContent::Editor(code), .. }) => {
                if let Err(err) = std::fs::write(&code.path, code.content.text()) {
                    tracing::warn!("save failed: {err}");
                    return Task::none();
                }
                code.dirty = false;
                (
                    code.language,
                    lsp::path_to_uri(&code.path),
                    code.content.text(),
                )
            }
            _ => return Task::none(),
        };

        if let Some(session) = self.workspace_session(workspace_id, language) {
            session.notify("textDocument/didSave", lsp::did_save_params(&uri, text));
        }

        Task::none()
    }

    pub fn open_editor_for(&mut self, abs: std::path::PathBuf) -> Task<Message> {
        let pane_id = self.id_gen.next_pane();
        let src = std::fs::read_to_string(&abs).unwrap_or_default();
        self.panes.insert(pane_id, Pane::new_code(pane_id, abs, src));

        if let Some(ws) = self.workspaces.get_mut(self.active_ws) {
            ws.strip.push(pane_id);
            let x = crate::app_update::compute_scroll_x(&ws.strip, self.viewport_w);
            ws.strip.viewport = cratty_core::ViewOffset::Static(x);
            self.start_lsp_for_pane(pane_id);
            return Task::batch([
                operation::scroll_to(strip_scroll_id(), scrollable::AbsoluteOffset { x, y: 0.0 }),
                self.focus_editor_pane(pane_id),
            ]);
        }

        Task::none()
    }

    pub fn start_lsp_for_pane(&mut self, pane_id: PaneId) {
        let Some(workspace_id) = self.pane_workspace_id(pane_id) else {
            return;
        };
        let Some((language, path)) = self.editor_language_and_path(pane_id) else {
            return;
        };
        let Some(server) = lsp::resolved_server_config(language) else {
            return;
        };

        self.track_workspace_documents(workspace_id, language);

        if self.workspace_session(workspace_id, language).is_some() {
            if self.workspace_status(workspace_id, language).is_some_and(|status| matches!(status, LspStatus::Ready)) {
                self.open_document_for_pane(pane_id);
            }
            return;
        }

        let workspace_root = self.workspace_root_by_id(workspace_id);
        let root = lsp::root_for(&path, workspace_root.as_ref());
        let server_command = server.command.clone();
        let server_args: Vec<&str> = server.args.iter().map(String::as_str).collect();
        match lsp::LspSession::start(&server_command, &server_args, &root) {
            Ok(session) => {
                let request_id = self.next_lsp_request_id();
                session.request(request_id, "initialize", lsp::initialize_params(&root));
                self.lsp_pending.insert(request_id, PendingLspRequest {
                    workspace_id,
                    language,
                    pane_id,
                    kind: LspRequestKind::Initialize,
                });
                let state = self.workspace_lsp.entry(workspace_id).or_insert_with(WorkspaceLspState::new);
                state.servers.insert(language, WorkspaceLspServer {
                    session,
                    status: LspStatus::Starting,
                    last_stderr: None,
                });
                state.statuses.insert(language, LspStatus::Starting);
            }
            Err(err) => {
                let missing_server = lsp::is_missing_program_error(&err);
                let install_queued = if missing_server {
                    self.queue_lsp_install(workspace_id, language)
                } else {
                    false
                };
                let status = if missing_server {
                    if install_queued {
                        let server_name = lsp::server_config(language)
                            .map(|cfg| cfg.command.to_string())
                            .unwrap_or_else(|| language.label().to_string());
                        LspStatus::Installing(server_name)
                    } else {
                        let server_name = lsp::server_config(language)
                            .map(|cfg| cfg.command.to_string())
                            .unwrap_or_else(|| language.label().to_string());
                        LspStatus::MissingServer(server_name)
                    }
                } else {
                    LspStatus::Failed(err.to_string())
                };
                let error_text = match &status {
                    LspStatus::MissingServer(message)
                    | LspStatus::Failed(message)
                    | LspStatus::Installing(message) => Some(message.clone()),
                    _ => None,
                };
                self.workspace_lsp
                    .entry(workspace_id)
                    .or_insert_with(WorkspaceLspState::new)
                    .statuses
                    .insert(language, status);
                if !install_queued {
                    if let Some(message) = error_text {
                        self.push_toast(
                            format!("{} LSP failed: {message}", language.label()),
                            ToastKind::Error,
                        );
                    }
                }
            }
        }
    }

    fn next_lsp_request_id(&mut self) -> u64 {
        let id = self.lsp_next_request_id;
        self.lsp_next_request_id += 1;
        id
    }

    pub fn flush_lsp_work(&mut self) {
        let now = Instant::now();
        let pane_ids: Vec<_> = self.panes.keys().copied().collect();
        for pane_id in pane_ids {
            let Some(workspace_id) = self.pane_workspace_id(pane_id) else {
                continue;
            };

            let mut change_request: Option<(CodeLanguage, String, i32, String)> = None;
            let mut hover_request: Option<(CodeLanguage, String, usize, usize, usize)> = None;

            {
                let Some(Pane { content: PaneContent::Editor(code), .. }) = self.panes.get_mut(&pane_id) else {
                    continue;
                };

                if let Some(at) = code.lsp.pending_sync_at {
                    if at <= now {
                        change_request = Some((
                            code.language,
                            lsp::path_to_uri(&code.path),
                            code.lsp.version,
                            code.content.text(),
                        ));
                        code.lsp.pending_sync_at = None;
                    }
                }

                if let Some(at) = code.lsp.pending_hover_at {
                    if at <= now {
                        let line = code.lsp.cursor_line;
                        let column = code.lsp.cursor_column;
                        let line_text = code.content.line(line)
                            .map(|line| line.text.into_owned())
                            .unwrap_or_default();
                        hover_request = Some((
                            code.language,
                            lsp::path_to_uri(&code.path),
                            line,
                            column,
                            lsp::utf16_col(&line_text, column),
                        ));
                        code.lsp.pending_hover_at = None;
                    }
                }
            }

            if let Some((language, uri, version, text)) = change_request {
                if let Some(session) = self.workspace_session(workspace_id, language) {
                    session.notify("textDocument/didChange", lsp::did_change_params(&uri, version, text));
                }
            }

            if let Some((language, uri, line, column, utf16_col)) = hover_request {
                let request_id = self.next_lsp_request_id();
                if let Some(session) = self.workspace_session(workspace_id, language) {
                    session.request(request_id, "textDocument/hover", lsp::hover_params(&uri, line, utf16_col));
                    self.lsp_pending.insert(request_id, PendingLspRequest {
                        workspace_id,
                        language,
                        pane_id,
                        kind: LspRequestKind::Hover { line, column },
                    });
                }
            }
        }
    }

    pub fn process_lsp_events(&mut self) {
        let workspace_ids: Vec<_> = self.workspace_lsp.keys().copied().collect();
        let mut dead = Vec::new();

        for workspace_id in workspace_ids {
            let languages = self.workspace_lsp.get(&workspace_id)
                .map(|state| state.servers.keys().copied().collect::<Vec<_>>())
                .unwrap_or_default();

            for language in languages {
                let mut events = Vec::new();
                loop {
                    let recv = {
                        let Some(session) = self.workspace_session(workspace_id, language) else {
                            break;
                        };
                        session.rx.try_recv()
                    };
                    match recv {
                        Ok(event) => events.push(event),
                        Err(std::sync::mpsc::TryRecvError::Empty) => break,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            dead.push((workspace_id, language));
                            break;
                        }
                    }
                }
                for event in events {
                    self.handle_lsp_event(workspace_id, language, event, &mut dead);
                }
            }
        }

        for (workspace_id, language) in dead {
            let exit_message = self.workspace_lsp.get(&workspace_id)
                .and_then(|state| state.servers.get(&language))
                .and_then(|server| server.last_stderr.clone())
                .filter(|message| !message.trim().is_empty())
                .unwrap_or_else(|| "LSP server exited".into());
            self.push_toast(
                format!("{} LSP exited: {exit_message}", language.label()),
                ToastKind::Error,
            );
            self.shutdown_workspace_language(workspace_id, language, Some(exit_message));
        }
    }

    fn handle_lsp_event(
        &mut self,
        workspace_id: WorkspaceId,
        language: CodeLanguage,
        event: lsp::LspEvent,
        dead: &mut Vec<(WorkspaceId, CodeLanguage)>,
    ) {
        match event {
            lsp::LspEvent::Notification { method, params } => {
                if method == "textDocument/publishDiagnostics" {
                    let Some(uri) = params.get("uri").and_then(serde_json::Value::as_str) else {
                        return;
                    };
                    let diagnostics = lsp::parse_diagnostics(&params);
                    self.apply_diagnostics(workspace_id, uri, diagnostics);
                }
            }
            lsp::LspEvent::Stderr(line) => {
                if let Some(server) = self.workspace_lsp.get_mut(&workspace_id)
                    .and_then(|state| state.servers.get_mut(&language))
                {
                    server.last_stderr = Some(line);
                }
            }
            lsp::LspEvent::Response { id, result, error } => {
                let Some(target) = self.lsp_pending.remove(&id) else {
                    return;
                };
                if target.workspace_id != workspace_id || target.language != language {
                    return;
                }
                match target.kind {
                    LspRequestKind::Initialize => {
                        if let Some(err) = error {
                            let message = err.to_string();
                            self.set_workspace_status(workspace_id, language, LspStatus::Failed(message.clone()));
                            self.push_toast(
                                format!("{} LSP initialize failed: {message}", language.label()),
                                ToastKind::Error,
                            );
                            return;
                        }

                        if let Some(session) = self.workspace_session(workspace_id, language) {
                            session.notify("initialized", serde_json::Value::Object(Default::default()));
                        }
                        self.set_workspace_status(workspace_id, language, LspStatus::Ready);
                        self.track_workspace_documents(workspace_id, language);
                        self.open_workspace_documents(workspace_id, language);
                    }
                    LspRequestKind::Hover { line, column } => {
                        if let Some(Pane { content: PaneContent::Editor(code), .. }) = self.panes.get_mut(&target.pane_id) {
                            code.lsp.hover = result
                                .as_ref()
                                .and_then(lsp::hover_text)
                                .map(|contents| HoverInfo { contents, line, column });
                        }
                    }
                }
            }
            lsp::LspEvent::Exited => dead.push((workspace_id, language)),
        }
    }

    pub fn detach_lsp_for_pane(&mut self, pane_id: PaneId) {
        self.lsp_pending.retain(|_, pending| pending.pane_id != pane_id);

        let Some(workspace_id) = self.pane_workspace_id(pane_id) else {
            return;
        };
        let Some((language, path)) = self.editor_language_and_path(pane_id) else {
            return;
        };
        let uri = lsp::path_to_uri(&path);

        let still_open_elsewhere = self.workspace_editor_panes(workspace_id).into_iter().any(|other_id| {
            other_id != pane_id && self.editor_path(other_id).is_some_and(|other| lsp::path_to_uri(&other) == uri)
        });

        if still_open_elsewhere {
            return;
        }

        if let Some(session) = self.workspace_session(workspace_id, language) {
            session.notify("textDocument/didClose", lsp::did_close_params(&uri));
        }

        if let Some(state) = self.workspace_lsp.get_mut(&workspace_id) {
            if let Some(open_docs) = state.open_docs.get_mut(&language) {
                open_docs.remove(&uri);
            }
        }
    }

    pub fn shutdown_lsp_for_workspace(&mut self, workspace_id: WorkspaceId) {
        self.pending_lsp_installs.retain(|pending| pending.workspace_id != workspace_id);
        self.lsp_pending.retain(|_, pending| pending.workspace_id != workspace_id);
        if let Some(state) = self.workspace_lsp.remove(&workspace_id) {
            for (_, server) in state.servers {
                let request_id = self.next_lsp_request_id();
                server.session.shutdown(request_id);
                server.session.notify("exit", serde_json::Value::Null);
                server.session.kill();
            }
        }
    }

    fn shutdown_workspace_language(
        &mut self,
        workspace_id: WorkspaceId,
        language: CodeLanguage,
        replacement_status: Option<String>,
    ) {
        self.lsp_pending.retain(|_, pending| {
            !(pending.workspace_id == workspace_id && pending.language == language)
        });

        let server = self.workspace_lsp.get_mut(&workspace_id)
            .and_then(|state| state.servers.remove(&language));

        if let Some(server) = server {
            let request_id = self.next_lsp_request_id();
            server.session.shutdown(request_id);
            server.session.notify("exit", serde_json::Value::Null);
            server.session.kill();
        }

        if let Some(state) = self.workspace_lsp.get_mut(&workspace_id) {
            if let Some(message) = replacement_status {
                state.statuses.insert(language, LspStatus::Failed(message));
            } else {
                state.statuses.remove(&language);
            }
        }
    }

    fn queue_lsp_install(&mut self, workspace_id: WorkspaceId, language: CodeLanguage) -> bool {
        if self.pending_lsp_installs.iter().any(|pending| {
            pending.workspace_id == workspace_id && pending.language == language
        }) {
            return true;
        }
        let Some(plan) = lsp::install_plan(language) else {
            self.push_toast(
                format!("No automatic installer available for {}", language.label()),
                ToastKind::Error,
            );
            return false;
        };

        let (tx, rx) = mpsc::channel();
        let label = plan.label.to_string();
        thread::spawn(move || {
            let result = match &plan.strategy {
                lsp::InstallStrategy::Command { program, args } => Command::new(program)
                    .args(args.iter().copied())
                    .status()
                    .map_err(anyhow::Error::from)
                    .and_then(|status| {
                        if status.success() {
                            Ok(())
                        } else {
                            Err(anyhow::anyhow!("{label} installer exited with {status}"))
                        }
                    }),
                #[cfg(windows)]
                lsp::InstallStrategy::ManagedTaploBinary => lsp::install_taplo_binary(),
            };
            let _ = tx.send(result);
        });

        self.set_workspace_status(
            workspace_id,
            language,
            LspStatus::Installing(plan.label.to_string()),
        );
        self.push_toast(format!("Installing {}…", plan.label), ToastKind::Info);
        self.pending_lsp_installs.push(PendingLspInstall {
            workspace_id,
            language,
            rx,
        });
        true
    }

    pub fn poll_pending_lsp_installs(&mut self) {
        let mut finished = Vec::new();
        self.pending_lsp_installs.retain(|pending| match pending.rx.try_recv() {
            Ok(result) => {
                finished.push((pending.workspace_id, pending.language, result));
                false
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => true,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                finished.push((
                    pending.workspace_id,
                    pending.language,
                    Err(anyhow::anyhow!("installer disconnected")),
                ));
                false
            }
        });

        for (workspace_id, language, result) in finished {
            match result {
                Ok(()) => {
                    self.push_toast(format!("Installed {}", language.label()), ToastKind::Success);
                    self.set_workspace_status(workspace_id, language, LspStatus::Starting);
                    self.start_lsp_for_workspace_language(workspace_id, language);
                }
                Err(err) => {
                    self.push_toast(format!("Failed to install {}: {err}", language.label()), ToastKind::Error);
                    self.set_workspace_status(workspace_id, language, LspStatus::Failed(err.to_string()));
                }
            }
        }
    }

    pub fn push_toast(&mut self, message: String, kind: ToastKind) {
        self.toasts.push(Toast {
            message,
            kind,
            expires_at: Instant::now() + Duration::from_secs(5),
        });
    }

    pub fn prune_toasts(&mut self) {
        let now = Instant::now();
        self.toasts.retain(|toast| toast.expires_at > now);
    }

    fn start_lsp_for_workspace_language(&mut self, workspace_id: WorkspaceId, language: CodeLanguage) {
        let Some(pane_id) = self.workspace_editor_panes(workspace_id).into_iter().find(|pid| {
            self.panes.get(pid).is_some_and(|pane| match &pane.content {
                PaneContent::Editor(code) => code.language == language,
                _ => false,
            })
        }) else {
            return;
        };
        self.start_lsp_for_pane(pane_id);
    }

    fn workspace_session(&self, workspace_id: WorkspaceId, language: CodeLanguage) -> Option<&lsp::LspSession> {
        self.workspace_lsp.get(&workspace_id)
            .and_then(|state| state.servers.get(&language))
            .map(|server| &server.session)
    }

    fn workspace_status(&self, workspace_id: WorkspaceId, language: CodeLanguage) -> Option<&LspStatus> {
        self.workspace_lsp.get(&workspace_id)
            .and_then(|state| state.statuses.get(&language))
    }

    fn set_workspace_status(&mut self, workspace_id: WorkspaceId, language: CodeLanguage, status: LspStatus) {
        let state = self.workspace_lsp.entry(workspace_id).or_insert_with(WorkspaceLspState::new);
        if let Some(server) = state.servers.get_mut(&language) {
            server.status = status.clone();
        }
        state.statuses.insert(language, status);
    }

    fn pane_workspace_id(&self, pane_id: PaneId) -> Option<WorkspaceId> {
        self.workspaces.iter()
            .find(|ws| ws.strip.panes.contains(&pane_id))
            .map(|ws| ws.id)
    }

    fn workspace_root_by_id(&self, workspace_id: WorkspaceId) -> Option<std::path::PathBuf> {
        self.workspaces.iter()
            .find(|ws| ws.id == workspace_id)
            .and_then(|ws| ws.root.clone())
    }

    fn workspace_editor_panes(&self, workspace_id: WorkspaceId) -> Vec<PaneId> {
        self.workspaces.iter()
            .find(|ws| ws.id == workspace_id)
            .map(|ws| ws.strip.panes.iter().copied().collect())
            .unwrap_or_default()
    }

    fn editor_language_and_path(&self, pane_id: PaneId) -> Option<(CodeLanguage, std::path::PathBuf)> {
        self.panes.get(&pane_id).and_then(|pane| match &pane.content {
            PaneContent::Editor(code) => Some((code.language, code.path.clone())),
            _ => None,
        })
    }

    fn editor_path(&self, pane_id: PaneId) -> Option<std::path::PathBuf> {
        self.panes.get(&pane_id).and_then(|pane| match &pane.content {
            PaneContent::Editor(code) => Some(code.path.clone()),
            _ => None,
        })
    }

    fn track_workspace_documents(&mut self, workspace_id: WorkspaceId, language: CodeLanguage) {
        let uris: HashSet<_> = self.collect_workspace_documents(workspace_id, language)
            .into_iter()
            .map(|(_, uri, _, _)| uri)
            .collect();
        let state = self.workspace_lsp.entry(workspace_id).or_insert_with(WorkspaceLspState::new);
        state.open_docs.insert(language, uris);
    }

    fn collect_workspace_documents(
        &self,
        workspace_id: WorkspaceId,
        language: CodeLanguage,
    ) -> Vec<(PaneId, String, String, i32)> {
        let mut docs = Vec::new();
        let mut seen = HashSet::new();

        for pane_id in self.workspace_editor_panes(workspace_id) {
            let Some(Pane { content: PaneContent::Editor(code), .. }) = self.panes.get(&pane_id) else {
                continue;
            };
            if code.language != language {
                continue;
            }
            let uri = lsp::path_to_uri(&code.path);
            if !seen.insert(uri.clone()) {
                continue;
            }
            docs.push((pane_id, uri, code.content.text(), code.lsp.version));
        }

        docs
    }

    fn open_workspace_documents(&mut self, workspace_id: WorkspaceId, language: CodeLanguage) {
        let docs = self.collect_workspace_documents(workspace_id, language);
        let Some(session) = self.workspace_session(workspace_id, language) else {
            return;
        };
        let Some(server) = lsp::server_config(language) else {
            return;
        };

        for (_, uri, text, version) in &docs {
            session.notify(
                "textDocument/didOpen",
                lsp::did_open_params(uri, server.language_id, *version, text.clone()),
            );
        }

        let uris = docs.into_iter().map(|(_, uri, _, _)| uri).collect();
        if let Some(state) = self.workspace_lsp.get_mut(&workspace_id) {
            state.open_docs.insert(language, uris);
        }
    }

    fn open_document_for_pane(&mut self, pane_id: PaneId) {
        let Some(workspace_id) = self.pane_workspace_id(pane_id) else {
            return;
        };
        let Some(Pane { content: PaneContent::Editor(code), .. }) = self.panes.get(&pane_id) else {
            return;
        };
        let language = code.language;
        let uri = lsp::path_to_uri(&code.path);
        let text = code.content.text();
        let version = code.lsp.version;

        let already_open = self.workspace_lsp.get(&workspace_id)
            .and_then(|state| state.open_docs.get(&language))
            .is_some_and(|uris| uris.contains(&uri));
        if already_open {
            return;
        }

        let Some(session) = self.workspace_session(workspace_id, language) else {
            return;
        };
        let Some(server) = lsp::server_config(language) else {
            return;
        };
        session.notify(
            "textDocument/didOpen",
            lsp::did_open_params(&uri, server.language_id, version, text),
        );

        if let Some(state) = self.workspace_lsp.get_mut(&workspace_id) {
            state.open_docs.entry(language).or_default().insert(uri);
        }
    }

    fn apply_diagnostics(&mut self, workspace_id: WorkspaceId, uri: &str, diagnostics: Vec<lsp::LspDiagnostic>) {
        for pane_id in self.workspace_editor_panes(workspace_id) {
            let Some(Pane { content: PaneContent::Editor(code), .. }) = self.panes.get_mut(&pane_id) else {
                continue;
            };
            if lsp::path_to_uri(&code.path) == uri {
                code.lsp.diagnostics = diagnostics.clone();
            }
        }
    }
}
