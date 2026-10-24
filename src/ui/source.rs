use crate::application::app::*;
use crate::domain::indicator;
use crate::domain::layout::*;
use crate::engine::kit;
use crate::ui::editor::*;

pub(crate) struct SourceView {
    pub(crate) pane: usize,

    pub(crate) editor: TextEditor,

    pub(crate) last_text: String,

    pub(crate) user_path: Option<std::path::PathBuf>,
}

impl SourceView {
    pub(crate) fn new() -> Self {
        Self {
            pane: 0,
            editor: TextEditor::new(),
            last_text: String::new(),
            user_path: None,
        }
    }
}

pub(crate) const EDITOR_ID: &str = "rt_source_editor";

impl AppState {
    pub(crate) fn source_title(&self) -> String {
        let name = self.chart_world.panes[self.workbench.source.pane]
            .indicator
            .name();
        let star = if self.workbench.source.editor.modified {
            " *"
        } else {
            ""
        };
        format!("Indicator Source - {name}{star}")
    }

    pub(crate) fn reload_source(&mut self, pane: usize) {
        let src = indicator::script_source(self.chart_world.panes[pane].indicator.name());
        let name = self.chart_world.panes[pane].indicator.name().to_owned();
        self.workbench.source.pane = pane;
        self.workbench.source.editor.load(&src);
        self.workbench.source.last_text = src.clone();
        self.workbench.source.user_path = Some(indicator::user_script_path(pane, &name));
        self.apply_script_source(src);
        self.shell.status_msg = format!("Indicator source: {name}");
    }

    fn open_pane_source(&mut self, pane: usize) {
        let name = self.chart_world.panes[pane].indicator.name().to_owned();
        let user_path = indicator::user_script_path(pane, &name);
        let src =
            std::fs::read_to_string(&user_path).unwrap_or_else(|_| indicator::script_source(&name));
        self.workbench.source.pane = pane;
        self.workbench.source.editor.load(&src);
        self.workbench.source.last_text = src;
        self.workbench.source.user_path = Some(user_path);
        self.apply_script_source(self.workbench.source.last_text.clone());
        self.shell.status_msg = format!("Indicator source: {name}");
    }

    pub(crate) fn revert_unsaved_source_if_needed(&mut self) -> bool {
        if !self.workbench.source.editor.modified {
            return false;
        }
        let pane = self.workbench.source.pane;
        self.reload_source(pane);
        self.workbench.source.editor.modified = false;
        true
    }

    pub(crate) fn ui_source_body(&mut self, ui: &mut kit::Ui, rect: kit::Rect) -> EditorCmd {
        let mut editor = std::mem::take(&mut self.workbench.source.editor);
        let cmd = editor.ui(self, ui, rect, kit::Id::new(EDITOR_ID));
        let text = editor.buf.to_text();
        let changed = text != self.workbench.source.last_text;
        self.workbench.source.editor = editor;
        if changed {
            self.workbench.source.last_text = text.clone();
            self.apply_script_source(text);
        }
        cmd
    }

    pub(crate) fn apply_script_source(&mut self, text: String) {
        let pane = self.workbench.source.pane;
        match crate::domain::script::ScriptProgram::parse(&text) {
            Ok(script) => {
                self.chart_world.panes[pane].script = Some(script);
                self.chart_world.panes[pane].script_source = text;
                self.chart_world.panes[pane].script_error = None;
                self.chart_world.panes[pane].dirty = true;
            }
            Err(error) => {
                self.chart_world.panes[pane].script_error = Some(error.clone());
                self.shell.status_msg = format!("Indicator source error: {error}");
            }
        }
    }

    pub(crate) fn run_editor_cmd(&mut self, cmd: EditorCmd) {
        match cmd {
            EditorCmd::None => {}
            EditorCmd::New => self.new_source(),
            EditorCmd::Save => self.save_source(),
            EditorCmd::Reload => {
                let pane = self.workbench.source.pane;
                self.reload_source(pane);
                let name = self.chart_world.panes[pane].indicator.name().to_owned();
                self.shell.status_msg = format!("Restored the original indicator source: {name}");
            }
            EditorCmd::Close => self.request_win(WinCmd::CloseSource),

            _ => {}
        }
    }

    pub(crate) fn new_source(&mut self) {
        self.workbench.source.editor.load("");
        self.workbench.source.editor.modified = true;
        self.workbench.source.last_text.clear();
        self.apply_script_source(String::new());
        self.shell.status_msg = "New empty file".to_owned();
    }

    pub(crate) fn save_source(&mut self) {
        let name = self.chart_world.panes[self.workbench.source.pane]
            .indicator
            .name()
            .to_owned();
        let path = self
            .workbench
            .source
            .user_path
            .clone()
            .unwrap_or_else(|| indicator::user_script_path(self.workbench.source.pane, &name));
        if let Some(dir) = path.parent() {
            if let Err(error) = std::fs::create_dir_all(dir) {
                self.shell.status_msg = format!(
                    "Save failed: cannot create {}: {}",
                    dir.display(),
                    plain_text(&error.to_string())
                );
                return;
            }
        }
        let text = self.workbench.source.editor.buf.to_text();
        self.apply_script_source(text.clone());
        match write_source(&path, &text) {
            Ok(()) => {
                self.workbench.source.editor.modified = false;
                self.workbench.source.last_text = text;
                self.shell.status_msg = format!("Saved {}", file_name(&path.to_string_lossy()));
            }
            Err(e) => {
                self.shell.status_msg = format!("Save failed: {}", plain_text(&e.to_string()));
            }
        }
    }
}

impl StockApp {
    pub(crate) fn open_source_window(&mut self, pane: usize) {
        if !self.win_mgr.source_open()
            && !self
                .win_mgr
                .can_spawn_window(self.state.chart_ui.work_rect, kit::vec2(SRC_W, SRC_H))
        {
            let text = "No room left to open another window".to_owned();
            self.state.shell.status_msg = text.clone();
            self.state.shell.tooltip = Some((
                kit::Rect::from_min_size(
                    self.state.chart_ui.work_rect.center(),
                    kit::vec2(LINE_W, LINE_W),
                ),
                text,
            ));
            return;
        }
        if self.state.workbench.source.pane != pane && self.state.workbench.source.editor.modified {
            self.state.revert_unsaved_source_if_needed();
        }
        if self.state.workbench.source.pane != pane || !self.win_mgr.source_open() {
            self.state.open_pane_source(pane);
        }
        if self
            .win_mgr
            .show_source(self.state.chart_ui.work_rect)
            .is_none()
        {
            let text = "No room left to open another window".to_owned();
            self.state.shell.status_msg = text.clone();
            self.state.shell.tooltip = Some((
                kit::Rect::from_min_size(
                    self.state.chart_ui.work_rect.center(),
                    kit::vec2(LINE_W, LINE_W),
                ),
                text,
            ));
            return;
        }
        self.state.workbench.debug_windows.source_open = true;
        let name = self.state.chart_world.panes[pane]
            .indicator
            .name()
            .to_owned();
        self.state.shell.status_msg = format!("Indicator source: {name}");
    }

    pub(crate) fn close_source(&mut self) {
        let restored = self.state.revert_unsaved_source_if_needed();
        self.win_mgr.hide_source();
        self.state.workbench.debug_windows.source_open = false;
        self.state.shell.status_msg = if restored {
            "Indicator editor closed; unsaved changes were reverted".to_owned()
        } else {
            "Indicator editor closed".to_owned()
        };
    }
}

fn file_name(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_owned()
}

pub(crate) fn write_source(path: &std::path::Path, text: &str) -> std::io::Result<()> {
    let crlf = match std::fs::read(path) {
        Ok(bytes) => bytes.windows(2).any(|w| w == b"\r\n"),
        Err(_) => true,
    };
    let out = if crlf {
        text.replace('\n', "\r\n")
    } else {
        text.to_owned()
    };
    std::fs::write(path, out)
}

fn plain_text(s: &str) -> String {
    s.chars()
        .filter(|c| crate::ui::textbuf::char_allowed(*c))
        .collect()
}
