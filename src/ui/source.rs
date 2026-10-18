use crate::app::*;
use crate::indicator;
use crate::ui::editor::*;
use eframe::egui;

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

impl StockApp {
    pub(crate) fn open_source_window(&mut self, pane: usize) {
        if self.source.pane != pane || !self.win_mgr.source_open() {
            self.open_pane_source(pane);
        }
        self.win_mgr.show_source(self.work_rect);
        let name = self.panes[pane].indicator.name().to_owned();
        self.status_msg = format!("指标代码：{name}");
    }

    pub(crate) fn source_title(&self) -> String {
        let name = self.panes[self.source.pane].indicator.name();
        let star = if self.source.editor.modified {
            " *"
        } else {
            ""
        };
        format!("指标代码 - {name}{star}")
    }

    pub(crate) fn reload_source(&mut self, pane: usize) {
        let src = indicator::script_source(self.panes[pane].indicator.name());
        let name = self.panes[pane].indicator.name().to_owned();
        self.source.pane = pane;
        self.source.editor.load(&src);
        self.source.last_text = src;
        self.source.user_path = Some(indicator::user_script_path(pane, &name));
        self.status_msg = format!("指标代码：{name}");
    }

    fn open_pane_source(&mut self, pane: usize) {
        let name = self.panes[pane].indicator.name().to_owned();
        let user_path = indicator::user_script_path(pane, &name);
        let src =
            std::fs::read_to_string(&user_path).unwrap_or_else(|_| indicator::script_source(&name));
        self.source.pane = pane;
        self.source.editor.load(&src);
        self.source.last_text = src;
        self.source.user_path = Some(user_path);
        self.apply_script_source(self.source.last_text.clone());
        self.status_msg = format!("指标代码：{name}");
    }

    pub(crate) fn close_source(&mut self) {
        self.win_mgr.hide_source();
        self.status_msg = "指标代码编辑窗：已关闭".to_owned();
    }

    pub(crate) fn ui_source_body(&mut self, ui: &mut egui::Ui, rect: egui::Rect) -> EditorCmd {
        let mut editor = std::mem::take(&mut self.source.editor);
        let cmd = editor.ui(self, ui, rect, egui::Id::new(EDITOR_ID));
        let text = editor.buf.to_text();
        let changed = text != self.source.last_text;
        self.source.editor = editor;
        if changed {
            self.source.last_text = text.clone();
            self.apply_script_source(text);
        }
        cmd
    }

    pub(crate) fn apply_script_source(&mut self, text: String) {
        let pane = self.source.pane;
        match crate::script::ScriptProgram::parse(&text) {
            Ok(script) => {
                self.panes[pane].script = Some(script);
                self.panes[pane].script_source = text;
                self.panes[pane].script_error = None;
                self.panes[pane].dirty = true;
            }
            Err(error) => {
                self.panes[pane].script_error = Some(error.clone());
                self.status_msg = format!("指标代码错误：{error}");
            }
        }
    }

    pub(crate) fn run_editor_cmd(&mut self, cmd: EditorCmd) {
        match cmd {
            EditorCmd::None => {}
            EditorCmd::Save => self.save_source(),
            EditorCmd::Reload => {
                let pane = self.source.pane;
                self.reload_source(pane);
                let name = self.panes[pane].indicator.name().to_owned();
                self.status_msg = format!("已还原成原始指标代码：{name}");
            }
            EditorCmd::Close => self.close_source(),

            _ => {}
        }
    }

    pub(crate) fn save_source(&mut self) {
        let name = self.panes[self.source.pane].indicator.name().to_owned();
        let path = self
            .source
            .user_path
            .clone()
            .unwrap_or_else(|| indicator::user_script_path(self.source.pane, &name));
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let text = self.source.editor.buf.to_text();
        self.apply_script_source(text.clone());
        match write_source(&path, &text) {
            Ok(()) => {
                self.source.editor.modified = false;
                self.source.last_text = text;
                self.status_msg = format!("已保存 {}", file_name(&path.to_string_lossy()));
            }
            Err(e) => {
                self.status_msg = format!("保存失败：{}", plain_text(&e.to_string()));
            }
        }
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
