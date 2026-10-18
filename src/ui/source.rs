use crate::app::*;
use crate::ui::editor::*;
use eframe::egui;

pub(crate) struct SourceView {
    pub(crate) pane: usize,

    pub(crate) editor: TextEditor,

    pub(crate) compile: CompileSlot,
}

impl SourceView {
    pub(crate) fn new() -> Self {
        Self {
            pane: 0,
            editor: TextEditor::new(),
            compile: CompileSlot::default(),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct CompileSlot {
    pub(crate) state: std::sync::Arc<std::sync::Mutex<CompileState>>,
}

impl CompileSlot {
    fn take_done(&self) -> Option<Result<(), String>> {
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        match &*st {
            CompileState::Done(_) => match std::mem::take(&mut *st) {
                CompileState::Done(res) => Some(res),
                _ => None,
            },
            _ => None,
        }
    }

    fn is_running(&self) -> bool {
        matches!(
            &*self.state.lock().unwrap_or_else(|e| e.into_inner()),
            CompileState::Running
        )
    }
}

#[derive(Default)]
pub(crate) enum CompileState {
    #[default]
    Idle,

    Running,

    Done(Result<(), String>),
}

pub(crate) const EDITOR_ID: &str = "rt_source_editor";

impl StockApp {
    pub(crate) fn open_source_window(&mut self, pane: usize) {
        if self.source.pane != pane || !self.win_mgr.source_open() {
            self.reload_source(pane);
        }
        self.win_mgr.show_source(self.work_rect);
        let name = self.panes[pane].indicator.name().to_owned();
        self.status_msg = format!("源码：{name}");
    }

    pub(crate) fn source_title(&self) -> String {
        let name = self.panes[self.source.pane].indicator.name();
        let star = if self.source.editor.modified {
            " *"
        } else {
            ""
        };
        format!("源码 - {name}{star}")
    }

    pub(crate) fn reload_source(&mut self, pane: usize) {
        let src = self.panes[pane].indicator.source();
        let name = self.panes[pane].indicator.name().to_owned();
        self.source.pane = pane;
        self.source.editor.load(src);
        self.status_msg = format!("源码：{name}");
    }

    pub(crate) fn close_source(&mut self) {
        self.win_mgr.hide_source();
        self.status_msg = "源码编辑窗：已关闭".to_owned();
    }

    pub(crate) fn ui_source_body(&mut self, ui: &mut egui::Ui, rect: egui::Rect) -> EditorCmd {
        let mut editor = std::mem::take(&mut self.source.editor);
        let cmd = editor.ui(self, ui, rect, egui::Id::new(EDITOR_ID));
        self.source.editor = editor;
        cmd
    }

    pub(crate) fn run_editor_cmd(&mut self, cmd: EditorCmd) {
        match cmd {
            EditorCmd::None => {}
            EditorCmd::Save => self.save_source(),
            EditorCmd::Compile => self.compile_source(),
            EditorCmd::Reload => {
                let pane = self.source.pane;
                self.reload_source(pane);
                let name = self.panes[pane].indicator.name().to_owned();
                self.status_msg = format!("已还原成原始源码：{name}");
            }
            EditorCmd::Close => self.close_source(),

            _ => {}
        }
    }

    pub(crate) fn save_source(&mut self) {
        let raw = self.panes[self.source.pane].indicator.path();
        let path = source_path(raw);
        let text = self.source.editor.buf.to_text();
        match write_source(&path, &text) {
            Ok(()) => {
                self.source.editor.modified = false;
                self.status_msg = format!("已保存 {}", file_name(raw));
            }
            Err(e) => {
                self.status_msg = format!("保存失败：{}", plain_text(&e.to_string()));
            }
        }
    }

    pub(crate) fn compile_source(&mut self) {
        if self.source.compile.is_running() {
            self.status_msg = "已经在编译了".to_owned();
            return;
        }
        self.save_source();
        self.status_msg = "编译中".to_owned();
        let slot = self.source.compile.clone();
        *slot.state.lock().unwrap_or_else(|e| e.into_inner()) = CompileState::Running;
        std::thread::spawn(move || {
            let result = run_cargo_check();
            if let Ok(mut st) = slot.state.lock() {
                *st = CompileState::Done(result);
            }
        });
    }

    pub(crate) fn poll_compile(&mut self) {
        if let Some(res) = self.source.compile.take_done() {
            self.status_msg = match res {
                Ok(()) => "编译成功".to_owned(),
                Err(e) => format!("编译失败：{e}"),
            };
        }
    }
}

fn source_path(path: &str) -> std::path::PathBuf {
    let p = std::path::Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(p)
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

fn run_cargo_check() -> Result<(), String> {
    let root = env!("CARGO_MANIFEST_DIR");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = std::process::Command::new(&cargo)
        .args(["check", "--quiet", "--message-format=short"])
        .current_dir(root)
        .output();
    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stderr);
            Err(first_error_line(&text))
        }
        Err(e) => Err(plain_text(&format!("起不了 cargo：{e}"))),
    }
}

fn first_error_line(text: &str) -> String {
    let pick = text
        .lines()
        .map(|l| l.trim())
        .find(|l| l.contains("error") && !l.is_empty())
        .or_else(|| text.lines().map(|l| l.trim()).find(|l| !l.is_empty()));
    plain_text(pick.unwrap_or("编译失败"))
}
