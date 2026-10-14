// ===========================================================================
//  模块 ui::source：指标源码编辑窗
// ===========================================================================
//  点 Pane 标题栏上的「代码」打开：主窗口里一块完全自绘的浮动窗口（走 ui::floatwin 那套统一
//  外壳：底色 + 标题栏 + 标题栏下沿那条线 + 1px 外框 + 按格拖动 + 关闭键），画在每帧最上层。
//  窗口里装的是 ui::editor 那块可编辑文本编辑器（菜单栏 + 按钮栏 + 行号区 + 代码区 + 光标 +
//  选区 + 中文输入法），打开时灌进该指标自己的源码（indicator.source()）。
//   - 「保存」把缓冲区写回 indicator.path()（file!() 给的源文件路径），写完不重编译、也不重读
//     源码（免得把用户正在编辑的内容覆盖回去）；
//   - 「编译」先把缓冲区写盘，再让 cargo check 在后台线程里过一遍，结果回状态栏；
//   - 「还原」/ 关窗 / 换指标都重新灌一次原始源码，所以编辑内容不会跨指标残留。
//
//  窗口尺寸 / 各段几何全部来自 grid.rs 的 SRC_* 常量（和编辑器共用一份口径）。
use crate::app::*;
use crate::grid::*;
use crate::ui::editor::*;
use crate::ui::floatwin::*;
use eframe::egui;

/// 源码编辑窗的全部跨帧状态：窗口外壳 + 编辑的是哪个 Pane + 编辑器本体 + 后台编译的共享状态。
/// egui 是即时模式，这些东西不存字段就跨不过一帧。
#[derive(Clone)]
pub(crate) struct SourceView {
    /// 窗口外壳：全工程统一的浮动窗口那一套（位置按 9px 格走、标题栏下沿那条线也在里面）。
    pub(crate) win: FloatWindow,
    /// 编辑的是哪个 Pane 的指标（换指标 / 再点「代码」都是换这一份源码）。
    pub(crate) pane: usize,
    /// 编辑器本体（缓冲区 + 光标 + 选区 + 滚动 + 输入法候选串）。
    pub(crate) editor: TextEditor,
    /// 后台编译的共享状态：Clone 只克隆 Arc，界面和后台线程看的是同一份。
    pub(crate) compile: CompileSlot,
}

impl SourceView {
    pub(crate) fn new() -> Self {
        Self {
            win: FloatWindow::new(SRC_DEFAULT_X, SRC_DEFAULT_Y, SRC_W, SRC_H),
            pane: 0,
            editor: TextEditor::new(),
            compile: CompileSlot::default(),
        }
    }

}

// ===========================================================================
//  一、后台编译：共享状态
// ===========================================================================

/// 后台编译的槽位：界面每帧看一眼，后台线程跑完把结果写进来。
/// 用 Arc<Mutex> 是为了能跨线程又能跟着 SourceView 一起克隆（进程里只有这一处编译）。
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

/// 一次编译的状态。
#[derive(Default)]
pub(crate) enum CompileState {
    /// 没在编译。
    #[default]
    Idle,
    /// 后台线程正在跑。
    Running,
    /// 跑完了：Ok = 编译成功，Err = 第一条错误（已经过滤成只有中文和 ASCII）。
    Done(Result<(), String>),
}

/// 编辑器窗口的焦点 id：egui 的键盘焦点 / IME 事件归属都按它判定。
pub(crate) const EDITOR_ID: &str = "rt_source_editor";

impl StockApp {
    /// 打开源码编辑窗：把该 Pane 当前指标的 source() 灌进编辑器，滚动位置归零。
    /// 已经开着同一个 Pane 就只把它提到最前，不重新灌源码 - 用户手里那份改动（哪怕还没保存）
    /// 不能因为多点了一下「代码」就没了。真要重新来一份走 reload_source（菜单里的「还原原始源码」）。
    pub(crate) fn open_source(&mut self, pane: usize) {
        if self.source.win.open && self.source.pane == pane {
            let name = self.panes[pane].indicator.name().to_owned();
            self.status_msg = format!("源码：{name}");
            return;
        }
        self.reload_source(pane);
    }

    /// 重新灌一份源码：打开窗口、换 Pane、换了指标、以及「还原原始源码」都走这儿。
    /// 换了指标必须走它 - 缓冲区和文件路径得一起换成新指标那一份，不然保存会把旧指标的文字
    /// 写进新指标的文件里。
    pub(crate) fn reload_source(&mut self, pane: usize) {
        let src = self.panes[pane].indicator.source();
        let name = self.panes[pane].indicator.name().to_owned();
        self.source.win.open = true;
        self.source.pane = pane;
        self.source.editor.load(src);
        self.source.win.drag = false;
        self.status_msg = format!("源码：{name}");
    }

    /// 源码编辑窗：主窗口里的一块自绘浮动面板，画在每帧最上层（压在界面所有东西之上）。
    /// 面板状态整份搬到本地变量上改，最后再写回字段 - 中途要调 self.text(...)，那会借走整个 self。
    pub(crate) fn ui_source_window(&mut self, ui: &mut egui::Ui, layout: egui::Rect) {
        // 编译是后台线程跑的：不管窗口开没开，每帧都看一眼结果。
        self.poll_compile();

        let mut sv = self.source.clone();
        if !sv.win.open {
            return;
        }

        let name = self.panes[sv.pane].indicator.name().to_owned();
        let star = if sv.editor.modified { " *" } else { "" };
        let mut cmd = EditorCmd::None;
        match sv.win.begin(self, ui, layout, &format!("源码 - {name}{star}")) {
            FloatShell::Closed => {
                sv.win.open = false;
                sv.win.drag = false;
                self.status_msg = "源码编辑窗：已关闭".to_owned();
            }
            FloatShell::Open(panel) => {
                let painter = ui.painter().clone();
                cmd = sv
                    .editor
                    .ui(self, ui, panel, egui::Id::new(EDITOR_ID));
                // 最外圈那 1px 硬边框和标题栏下沿那条线由外壳补画（排在内容之后）。
                sv.win.end(&painter);
            }
        }
        self.source = sv;
        self.run_editor_cmd(cmd);
    }

    /// 编辑器交回来的动作：碰文件 / 碰宿主状态的那四个在这儿做。
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
            EditorCmd::Close => {
                self.source.win.open = false;
                self.source.win.drag = false;
                self.status_msg = "源码编辑窗：已关闭".to_owned();
            }
            // 其余动作编辑器自己就做完了，不会传上来。
            _ => {}
        }
    }

    /// 保存：把缓冲区写回这个指标自己的源文件（路径由 file!() 给出）。
    /// 写完不重编译、也不重读源码 - 用户接着编辑，缓冲区一个字都不动。
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

    /// 编译：先把缓冲区写盘（编译读的是磁盘上那一份），再在后台线程里跑 `cargo check`。
    /// 结果回状态栏：成功 = 「编译成功」，失败 = 第一条错误的行。
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

    /// 每帧看一眼后台编译：跑完了就把结果写到状态栏。
    fn poll_compile(&mut self) {
        if let Some(res) = self.source.compile.take_done() {
            self.status_msg = match res {
                Ok(()) => "编译成功".to_owned(),
                Err(e) => format!("编译失败：{e}"),
            };
        }
    }
}

/// file!() 给的是「相对仓库根」的路径，而且 Windows 上是反斜杠；进程的工作目录又未必
/// 是仓库根，所以相对路径一律挂到 CARGO_MANIFEST_DIR 上，保证写的永远是那个真文件。
fn source_path(path: &str) -> std::path::PathBuf {
    let p = std::path::Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(p)
    }
}

/// 路径里最后那一段（状态栏上显示文件名的用）；正反斜杠都认。
fn file_name(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_owned()
}

/// 落盘：缓冲区里一律 \n，写出去按这个文件本来那套行尾（仓库里的源文件清一色 CRLF，
/// 别把整个文件的行尾给改了 - 那样每一行都会进 diff）。读不到原文件（新文件）就按 CRLF。
/// 写出去的字节就是这些 - 不带 BOM，不做任何别的加工。
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

/// 把一段可能带花哨字符的文本过滤成界面画得出来的（只有中文和 ASCII）。
fn plain_text(s: &str) -> String {
    s.chars().filter(|c| crate::ui::textbuf::char_allowed(*c)).collect()
}

/// 真编译一遍：在仓库根跑 `cargo check`（只做检查、不产出可执行文件，不会覆盖正在跑的自己）。
/// 起不了 cargo 就返回错误 - 不假装成功。
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

/// 从 cargo 的输出里挑第一条带 error 的行（状态栏只有一行格子，长篇大论放不下）。
fn first_error_line(text: &str) -> String {
    let pick = text
        .lines()
        .map(|l| l.trim())
        .find(|l| l.contains("error") && !l.is_empty())
        .or_else(|| text.lines().map(|l| l.trim()).find(|l| !l.is_empty()));
    plain_text(pick.unwrap_or("编译失败"))
}