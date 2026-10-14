// ===========================================================================
//  模块 ui::editor：可编辑文本编辑器（全自绘）
// ===========================================================================
//  一块「菜单栏 + 按钮栏 + 行号区 + 代码区」的多行文本编辑器：光标、选区、滚动、中文输入法全在这
//  一份里。自绘浮动窗里的「源码编辑」用它，以后别的窗口要编辑文本也直接用这一份。
//
//  铁律：不许用 egui::TextEdit（连它的外观一起不要）。全 painter 手绘 - 半格都不许有；
//  坐标一律走 grid 那套（cells / grid_line / text_left_x / text_right_x），字号 16px 点阵字。
//  输入法：只有拿到焦点的那个控件才消费 IME 事件（egui 的 owns_ime_events），
//  候选串（Preedit）画在光标处并加 1px 下划线，上屏（Commit）才真正插进缓冲区；
//  候选窗位置通过 PlatformOutput::ime（IMEOutput）告诉系统 - 不设它，拼音框会弹在屏幕左上角。
//
//  【设计逻辑】UI 里每一个控件区域的外圈都必须有横竖线划分（线一律 1px、落在网格线内，
//  绝不叠成 2px）：标题栏 / 菜单栏 / 按钮栏 / 正文区从上往下依次排，相邻两段共用中间那条
//  1px 网格线 - 各段自己往里让开这条线画底色，分隔线统一留到最后再画一遍，谁都盖不掉
//  （见 paint_separators）。全工程的控件区分割都照这条来。
//
//  状态全部存在 TextEditor 的字段里（egui 是即时模式）：缓冲区、滚动行、候选串、打开的菜单、
//  是不是正在划选、最近的查找词、有没有改过。
use crate::app::*;
use crate::font::{char_advance, text_advance};
use crate::grid::*;
use crate::theme::*;
use crate::ui::button::*;
use crate::ui::icon::*;
use crate::ui::menu::*;
use crate::ui::textbuf::*;
use eframe::egui;

// ===========================================================================
//  一、菜单栏 / 按钮栏的内容
// ===========================================================================

/// 编辑器菜单栏：三个菜单（顺序就是从左往右的顺序）。条目名字全工程唯一，
/// 点中了按名字反查动作（见 EditorCmd::of）。菜单栏 / 下拉面板的画法走 ui::menu 那一套，
/// 和主窗口的菜单栏长得一模一样（左对齐、相邻两项共用网格线、下沿那条 1px 线）。
pub(crate) const EDITOR_TABS: [MenuTab; 3] = [
    MenuTab {
        title: "文件",
        items: &["保存", "编译", "还原原始源码", "关闭窗口"],
    },
    MenuTab {
        title: "编辑",
        items: &["撤销", "剪切", "复制", "粘贴", "删除选中", "全选", "查找下一个"],
    },
    MenuTab {
        title: "查看",
        items: &["回到开头", "跳到末尾"],
    },
];

/// 编辑器按钮栏上那一排按钮（图标 + 悬停便签 + 动作），从左往右：新建 / 保存 / 编译 /
/// 剪切 / 复制 / 粘贴 / 删除 / 撤销 / 查找。便签就是鼠标悬停时显示的那句按钮含义。
pub(crate) const EDITOR_BUTTONS: [(IconKind, &str, EditorCmd); 9] = [
    (IconKind::New, "还原原始源码", EditorCmd::Reload),
    (IconKind::Save, "保存到源文件", EditorCmd::Save),
    (IconKind::Compile, "编译（cargo check）", EditorCmd::Compile),
    (IconKind::Cut, "剪切选中", EditorCmd::Cut),
    (IconKind::Copy, "复制选中", EditorCmd::Copy),
    (IconKind::Paste, "粘贴", EditorCmd::Paste),
    (IconKind::Cross, "删除选中", EditorCmd::Delete),
    (IconKind::Undo, "撤销", EditorCmd::Undo),
    (IconKind::Find, "查找下一个", EditorCmd::Find),
];

/// 编辑器要做的动作。缓冲区内部的那些（剪切 / 复制 / 粘贴 / 撤销 / 查找 / 全选）编辑器当场就做了；
/// 只有要碰文件、要碰别的状态的四个（保存 / 编译 / 还原 / 关闭）才交回宿主去做。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditorCmd {
    None,
    /// 把缓冲区写回文件。
    Save,
    /// 编译（先写盘再让 rustc / cargo 过一遍）。
    Compile,
    /// 丢掉编辑，重新读回指标自己的源码。
    Reload,
    /// 关掉这块窗口。
    Close,
    Cut,
    Copy,
    Paste,
    /// 删除选中的文本。
    Delete,
    Undo,
    SelectAll,
    /// 查找下一个（没选区就用上一次的词）。
    Find,
    DocHome,
    DocEnd,
}

impl EditorCmd {
    /// 按菜单条目名字反查动作（条目名字全工程唯一，所以直接比名字）。
    fn of(label: &str) -> EditorCmd {
        match label {
            "保存" => EditorCmd::Save,
            "编译" => EditorCmd::Compile,
            "还原原始源码" => EditorCmd::Reload,
            "关闭窗口" => EditorCmd::Close,
            "撤销" => EditorCmd::Undo,
            "剪切" => EditorCmd::Cut,
            "复制" => EditorCmd::Copy,
            "粘贴" => EditorCmd::Paste,
            "删除选中" => EditorCmd::Delete,
            "全选" => EditorCmd::SelectAll,
            "查找下一个" => EditorCmd::Find,
            "回到开头" => EditorCmd::DocHome,
            "跳到末尾" => EditorCmd::DocEnd,
            _ => EditorCmd::None,
        }
    }
}

// ===========================================================================
//  二、键盘 / 输入法事件 -> 缓冲区（单行输入框和多行编辑器共用这一份）
// ===========================================================================

/// 处理完这一帧的事件之后，宿主需要知道的事。
pub(crate) struct EditOutcome {
    /// 内容改过没有（置脏 / 标题上那个星号用它）。
    pub(crate) changed: bool,
    /// 按了 Ctrl+S。
    pub(crate) save: bool,
    /// 要写进系统剪贴板的文本（复制 / 剪切）。
    pub(crate) copy: Option<String>,
}

/// 把这一帧的键盘 / 输入法事件喂给缓冲区。multiline = false 时不认回车（单行输入框）。
/// filter 决定哪些字符收得下（源码编辑器用 char_allowed，Hex 输入框只收 16 进制数字）。
/// 候选串（IME Preedit）存在 preedit 里：空串 = 没有候选，宿主据此在光标处画下划线。
pub(crate) fn apply_edit_events(
    buf: &mut TextBuffer,
    preedit: &mut String,
    events: &[egui::Event],
    multiline: bool,
    filter: &dyn Fn(char) -> bool,
) -> EditOutcome {
    let mut out = EditOutcome {
        changed: false,
        save: false,
        copy: None,
    };
    for event in events {
        match event {
            egui::Event::Text(text) => {
                for ch in text.chars() {
                    if ch.is_control() {
                        continue;
                    }
                    if filter(ch) {
                        buf.insert_char(ch);
                        out.changed = true;
                    }
                }
            }
            egui::Event::Paste(text) => {
                buf.insert_text(text, filter);
                *preedit = String::new();
                out.changed = true;
            }
            egui::Event::Copy => {
                out.copy = Some(buf.selected_text());
            }
            egui::Event::Cut => {
                let text = buf.cut();
                if !text.is_empty() {
                    out.changed = true;
                    out.copy = Some(text);
                }
            }
            egui::Event::Ime(ime) => match ime {
                egui::ImeEvent::Preedit { text, .. } => {
                    *preedit = text.clone();
                }
                egui::ImeEvent::Commit(text) => {
                    buf.insert_text(text, filter);
                    preedit.clear();
                    out.changed = true;
                }
                egui::ImeEvent::DeleteSurrounding {
                    before_chars,
                    after_chars,
                } => {
                    // 输入法要删光标周围的字符：按字符数删（列本来就是字符索引）。
                    for _ in 0..*before_chars {
                        buf.backspace();
                    }
                    for _ in 0..*after_chars {
                        buf.delete();
                    }
                    out.changed = true;
                }
                _ => {}
            },
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => {
                let ctrl = modifiers.ctrl || modifiers.command;
                match key {
                    egui::Key::Backspace => {
                        buf.backspace();
                        out.changed = true;
                    }
                    egui::Key::Delete => {
                        buf.delete();
                        out.changed = true;
                    }
                    egui::Key::Enter => {
                        if multiline {
                            buf.insert_newline();
                            out.changed = true;
                        }
                    }
                    egui::Key::Tab => {
                        if !ctrl {
                            buf.insert_text(&" ".repeat(TAB_SPACES), filter);
                            out.changed = true;
                        }
                    }
                    egui::Key::ArrowLeft => {
                        if ctrl {
                            buf.move_home();
                        } else {
                            buf.move_left();
                        }
                    }
                    egui::Key::ArrowRight => {
                        if ctrl {
                            buf.move_end();
                        } else {
                            buf.move_right();
                        }
                    }
                    egui::Key::ArrowUp => buf.move_vertical(false),
                    egui::Key::ArrowDown => buf.move_vertical(true),
                    egui::Key::Home => {
                        if ctrl {
                            buf.move_doc_home();
                        } else {
                            buf.move_home();
                        }
                    }
                    egui::Key::End => {
                        if ctrl {
                            buf.move_doc_end();
                        } else {
                            buf.move_end();
                        }
                    }
                    egui::Key::A if ctrl => buf.select_all(),
                    egui::Key::C if ctrl => {
                        out.copy = Some(buf.selected_text());
                    }
                    egui::Key::X if ctrl => {
                        let text = buf.cut();
                        if !text.is_empty() {
                            out.changed = true;
                            out.copy = Some(text);
                        }
                    }
                    egui::Key::Z if ctrl => {
                        if buf.undo() {
                            out.changed = true;
                        }
                    }
                    egui::Key::S if ctrl => out.save = true,
                    _ => {}
                }
            }
            _ => {}
        }
    }
    out
}

// ===========================================================================
//  三、编辑器本体
// ===========================================================================

/// 一块可编辑文本编辑器的全部跨帧状态。
#[derive(Clone)]
pub(crate) struct TextEditor {
    /// 文本缓冲区（行 + 光标 + 选区 + 撤销栈）。
    pub(crate) buf: TextBuffer,
    /// 第一行可见行的行号（整数行滚动）。
    pub(crate) scroll: usize,
    /// 输入法候选串：画在光标处、底下加一根 1px 下划线。
    pub(crate) preedit: String,
    /// 滚轮攒下来的零头（触控板那种按点上报的增量）。
    pub(crate) wheel_acc: f32,
    /// 菜单栏的状态（哪一项开着）。菜单栏 / 下拉面板的画法走 ui::menu。
    pub(crate) menu: MenuState,
    /// 焦点：点正文区拿焦点、点面板外面还回去。egui 的键盘焦点我们也照样申请
    /// （egui_wants_keyboard_input 要为真，按键才不被系统吞掉），但「收不收键盘事件」
    /// 只认这一个标志 - 不依赖 egui 内部焦点哪天被谁踢掉，打字永远跟手。
    pub(crate) focused: bool,
    /// 正在拖滚动条的把手：记下鼠标相对把手上沿的偏移，拖的时候把手不跳。
    pub(crate) bar_grab: Option<f32>,
    /// 这次鼠标拖拽是不是在划选区。
    pub(crate) selecting: bool,
    /// 上一次查找用的词（没选区时「查找下一个」接着用它）。
    pub(crate) find_term: String,
    /// 内容被改过没有（标题上标个星号）。
    pub(crate) modified: bool,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEditor {
    pub(crate) fn new() -> Self {
        Self {
            buf: TextBuffer::default(),
            scroll: 0,
            preedit: String::new(),
            wheel_acc: 0.0,
            menu: MenuState::default(),
            focused: false,
            bar_grab: None,
            selecting: false,
            find_term: String::new(),
            modified: false,
        }
    }

    /// 整份换成一段新文本（打开窗口 / 还原原始源码）：滚动、候选串、修改标记一起归零。
    pub(crate) fn load(&mut self, text: &str) {
        self.buf = TextBuffer::from_text(text);
        self.scroll = 0;
        self.preedit.clear();
        self.wheel_acc = 0.0;
        self.bar_grab = None;
        self.selecting = false;
        self.modified = false;
    }

    /// 滚动上限：总行数减一屏行数。
    pub(crate) fn max_scroll(&self) -> usize {
        self.buf.line_count().saturating_sub(SRC_VISIBLE_LINES)
    }

    /// 光标跑到屏幕外了就把它拉回来（键盘翻页 / 打字之后自动跟随）。
    fn ensure_caret_visible(&mut self) {
        let l = self.buf.cursor.0;
        if l < self.scroll {
            self.scroll = l;
        } else if l >= self.scroll + SRC_VISIBLE_LINES {
            self.scroll = l + 1 - SRC_VISIBLE_LINES;
        }
        self.scroll = self.scroll.min(self.max_scroll());
    }

    /// 画整块编辑器并处理交互：菜单栏 -> 按钮栏 -> 正文区 -> 下拉面板（最后一层）。
    /// 返回宿主还要去做的动作（保存 / 编译 / 还原 / 关闭）。
    pub(crate) fn ui(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        panel: egui::Rect,
        id: egui::Id,
    ) -> EditorCmd {
        let mut cmd = EditorCmd::None;
        let menu_band = grid_band(panel, SRC_TITLE_ROWS, SRC_MENU_ROWS);
        let bar_band = grid_band(panel, SRC_TITLE_ROWS + SRC_MENU_ROWS, SRC_BAR_ROWS);
        let dropdown = self.ui_menubar(app, ui, menu_band, id);
        self.ui_buttonbar(app, ui, bar_band, &mut cmd);
        self.ui_body(app, ui, panel, id, &mut cmd);
        if let Some(anchor) = dropdown {
            self.ui_dropdown(app, ui, anchor, &mut cmd);
        }
        // 各段之间那条 1px 分隔线最后画一遍：铁律 - 每个控件区的外圈都得有落在网格线上的线，
        // 而且谁都盖不掉（内容先画、线后画，永远只有 1px）。
        self.paint_separators(&ui.painter().clone(), panel);
        self.finish_cmd(ui, cmd)
    }

    /// 各段之间的 1px 分隔线：菜单栏下沿 + 按钮栏下沿（标题栏下沿那条由浮动窗外壳补画）。
    /// 排在所有内容之后画，控件底色盖不掉它们。
    fn paint_separators(&self, painter: &egui::Painter, panel: egui::Rect) {
        for rows in [SRC_TITLE_ROWS + SRC_MENU_ROWS, SRC_HEAD_ROWS] {
            let y = panel.min.y + grid_line(rows);
            hline_hard(painter, panel.min.x, panel.max.x, y, theme().ctrl_border);
        }
    }

    /// 缓冲区内部的动作用不着宿主，当场做掉；只有碰文件 / 碰宿主状态的才往外传。
    fn finish_cmd(&mut self, ui: &egui::Ui, cmd: EditorCmd) -> EditorCmd {
        match cmd {
            EditorCmd::None => EditorCmd::None,
            EditorCmd::Save | EditorCmd::Compile | EditorCmd::Reload | EditorCmd::Close => cmd,
            EditorCmd::Cut => {
                let text = self.buf.cut();
                if !text.is_empty() {
                    ui.ctx().copy_text(text);
                    self.modified = true;
                }
                EditorCmd::None
            }
            EditorCmd::Copy => {
                let text = self.buf.selected_text();
                if !text.is_empty() {
                    ui.ctx().copy_text(text);
                }
                EditorCmd::None
            }
            EditorCmd::Paste => {
                // 剪贴板由我们自己读（egui 只提供「写」，读要走系统 API，见 win32::clipboard_text）。
                if let Some(text) = crate::win32::clipboard_text() {
                    self.buf.insert_text(&text, &char_allowed);
                    self.modified = true;
                }
                EditorCmd::None
            }
            EditorCmd::Delete => {
                if self.buf.delete_selection() {
                    self.modified = true;
                }
                EditorCmd::None
            }
            EditorCmd::Undo => {
                if self.buf.undo() {
                    self.modified = true;
                }
                EditorCmd::None
            }
            EditorCmd::SelectAll => {
                self.buf.select_all();
                EditorCmd::None
            }
            EditorCmd::Find => {
                // 有选区就拿选区当查找词（并记下来），没有就用上一次的词，往下找。
                let term = if self.buf.has_selection() {
                    let t = self.buf.selected_text();
                    self.find_term = t.clone();
                    t
                } else {
                    self.find_term.clone()
                };
                if !term.is_empty() {
                    self.buf.find_next(&term);
                }
                EditorCmd::None
            }
            EditorCmd::DocHome => {
                self.buf.move_doc_home();
                EditorCmd::None
            }
            EditorCmd::DocEnd => {
                self.buf.move_doc_end();
                EditorCmd::None
            }
        }
    }

    /// 菜单栏：走 ui::menu 那一套（左对齐、相邻两项共用网格线、下沿那条 1px 线在最后画，
    /// 点菜单栏和下拉面板以外的地方自动收起）。返回下拉面板的锚点矩形（没开着就是 None）。
    fn ui_menubar(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        band: egui::Rect,
        _id: egui::Id,
    ) -> Option<egui::Rect> {
        let (open, state) = ui_menu_bar(app, ui, band, &EDITOR_TABS, self.menu);
        self.menu = state;
        open.map(|(_, anchor)| anchor)
    }

    /// 菜单下拉面板：走 ui::menu 那一套（铺底 -> 逐条画 -> 1px 硬边压在最后）。
    /// 点中的条目按名字反查动作。
    fn ui_dropdown(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        anchor: egui::Rect,
        cmd: &mut EditorCmd,
    ) {
        let Some(open) = self.menu.open else {
            return;
        };
        let items = EDITOR_TABS[open].items;
        let panel = MenuPanel::right(anchor, MENU_PANEL_W, items.len(), ui.max_rect().max.y);
        let state = self.menu;
        let (clicked, state) = ui_dropdown(app, ui, panel, items, state);
        self.menu = state;
        if let Some(j) = clicked {
            *cmd = EditorCmd::of(items[j]);
        }
    }

    /// 按钮栏：一排只有图标的方按钮（各 cells(2) 见方，相邻共用中间那条网格线），
    /// 外观走全工程通用的 draw_button；悬停时弹便签说明按钮含义。
    fn ui_buttonbar(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        band: egui::Rect,
        cmd: &mut EditorCmd,
    ) {
        let painter = ui.painter().clone();
        // 底色让开上沿那条共用网格线（菜单栏下沿已经把它画成边框色了）：从线下面那一行起铺，
        // 那条线才不会被这一段的底色啃掉。
        painter.rect_filled(
            egui::Rect::from_min_max(egui::pos2(band.min.x, band.min.y + LINE_W), band.max),
            egui::CornerRadius::ZERO,
            theme().panel_bg,
        );
        let w = cells(2);
        let mut x = band.min.x + grid_line(1);
        for (icon, tip, action) in EDITOR_BUTTONS {
            if x + w > band.max.x {
                break;
            }
            let rect = egui::Rect::from_min_size(
                egui::pos2(x, band.min.y),
                egui::vec2(w, band.height()),
            );
            let content = ButtonContent::Icon(icon);
            if app.draw_button(ui, &painter, rect, &content, false, tip) {
                *cmd = action;
            }
            x += w - LINE_W;
        }
    }

    /// 把屏幕坐标换成缓冲区位置（行, 字符列）：行按行距反算、列按「逐字累加宽度」反算。
    fn pos_at(&self, p: egui::Pos2, code_x: f32, body_top: f32) -> (usize, usize) {
        let row = ((p.y - body_top) / SRC_ROW_PITCH).floor().max(0.0) as usize;
        let line = (self.scroll + row).min(self.buf.line_count().saturating_sub(1));
        let x = (p.x - code_x).max(0.0);
        let mut w = 0.0f32;
        let mut col = 0usize;
        for ch in self.buf.line(line).chars() {
            let cw = char_advance(ch);
            // 落在某个字的后半截就算在这个字之后（点右半边 = 光标放到它后面）。
            if x < w + cw * 0.5 {
                break;
            }
            w += cw;
            col += 1;
        }
        (line, col)
    }

    /// 正文区：行号区 + 代码区 + 光标 + 选区 + 输入法候选 + 滚动条。
    fn ui_body(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        panel: egui::Rect,
        id: egui::Id,
        cmd: &mut EditorCmd,
    ) {
        let painter = ui.painter().clone();
        let body_top = panel.min.y + SRC_BODY_TOP;
        let body_bottom = panel.max.y - LINE_W;
        let gutter_line = panel.min.x + grid_line(SRC_GUTTER_COLS);
        let bar_line = panel.min.x + grid_line(SRC_COLS - 1);
        let code_x = text_left_x(gutter_line);
        let code_w = (bar_line - GRID - code_x).max(0.0);
        let visible = SRC_VISIBLE_LINES;

        // 行号区底色 + 右边那条竖线。底色让开上沿那条共用网格线（按钮栏下沿那条）。
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(panel.min.x + LINE_W, body_top + LINE_W),
                egui::pos2(gutter_line, body_bottom),
            ),
            egui::CornerRadius::ZERO,
            theme().panel_bg,
        );
        vline_hard(&painter, gutter_line, body_top, body_bottom, theme().ctrl_border);

        // ---- 滚轮：按整数行滚动（鼠标压在面板里才算数）----
        let hovered = ui.input(|i| i.pointer.hover_pos()).is_some_and(|p| panel.contains(p));
        if hovered {
            let mut acc = self.wheel_acc;
            ui.input(|i| {
                for event in &i.events {
                    if let egui::Event::MouseWheel { unit, delta, .. } = event {
                        acc += match unit {
                            egui::MouseWheelUnit::Line => delta.y,
                            egui::MouseWheelUnit::Point => delta.y / WHEEL_POINTS_PER_NOTCH,
                            egui::MouseWheelUnit::Page => delta.y,
                        };
                    }
                }
            });
            let whole = acc.trunc();
            self.wheel_acc = acc - whole;
            if whole != 0.0 {
                let max = self.max_scroll() as f32;
                self.scroll = (self.scroll as f32 - whole).round().clamp(0.0, max) as usize;
            }
        }
        self.scroll = self.scroll.min(self.max_scroll());

        // ---- 鼠标：点一下定位光标、按住划选；点一下就把焦点收过来 ----
        let body = egui::Rect::from_min_max(
            egui::pos2(panel.min.x + LINE_W, body_top),
            egui::pos2(panel.max.x - LINE_W, body_bottom),
        );
        // 用 interact 而不是 allocate_rect：egui 只认「这一帧登记过的」控件才留住焦点
        // （Focus::end_pass 会把没登记的焦点踢掉 - allocate_rect 生成的是自动 id，不是我们
        // 请求焦点用的那个，所以点完一帧焦点就没了）。
        let resp = ui.interact(body, id, egui::Sense::click_and_drag());
        if resp.hovered() || resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
        }
        if resp.clicked() || resp.drag_started() {
            self.focused = true;
        }
        // 点在面板外面（别的窗口 / 图表上）就把焦点还回去。
        if self.focused {
            let outside_click = ui.ctx().input(|i| {
                i.pointer.primary_clicked()
                    && i.pointer
                        .interact_pos()
                        .is_some_and(|p| !panel.contains(p))
            });
            if outside_click {
                self.focused = false;
            }
        }
        // 键盘焦点照样每帧申请（egui_wants_keyboard_input 要为真，按键才不被系统当成没人在听），
        // 但「收不收键盘事件」只认 self.focused - 不靠 egui 内部焦点跨帧留存。
        if self.focused {
            ui.memory_mut(|m| m.request_focus(id));
        } else {
            ui.memory_mut(|m| m.surrender_focus(id));
        }
        let focused = self.focused;

        if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
            if resp.drag_started() {
                self.selecting = true;
                let pos = self.pos_at(p, code_x, body_top);
                self.buf.set_cursor(pos, false);
                self.buf.sel_anchor = Some(pos);
            } else if self.selecting && resp.dragged() {
                let pos = self.pos_at(p, code_x, body_top);
                self.buf.set_cursor(pos, true);
            } else if resp.clicked() {
                let pos = self.pos_at(p, code_x, body_top);
                self.buf.set_cursor(pos, false);
            }
        }
        if !ui.input(|i| i.pointer.primary_down()) {
            self.selecting = false;
        }

        // ---- 键盘 / 输入法：只有拿到焦点才收 ----
        if focused {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    id,
                    egui::EventFilter {
                        tab: true,
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                    },
                )
            });
            let events = ui.input(|i| i.events.clone());
            let out = apply_edit_events(&mut self.buf, &mut self.preedit, &events, true, &char_allowed);
            if out.changed {
                self.modified = true;
            }
            if let Some(text) = out.copy {
                ui.ctx().copy_text(text);
            }
            if out.save {
                *cmd = EditorCmd::Save;
            }
            self.ensure_caret_visible();
        } else {
            self.preedit.clear();
        }

        // ---- 绘制：选区 -> 行号 + 正文 -> 光标 + 候选串 ----
        let text_painter = painter.with_clip_rect(egui::Rect::from_min_max(
            egui::pos2(code_x, body_top),
            egui::pos2(bar_line, body_bottom),
        ));

        if let Some((start, end)) = self.buf.selection() {
            for k in 0..visible {
                let idx = self.scroll + k;
                if idx < start.0 || idx > end.0 {
                    continue;
                }
                let line = self.buf.line(idx);
                let chars: Vec<char> = line.chars().collect();
                let c0 = if idx == start.0 { start.1 } else { 0 }.min(chars.len());
                let c1 = if idx == end.0 { end.1 } else { chars.len() }.min(chars.len());
                let x0 = code_x + text_advance(&chars[..c0].iter().collect::<String>());
                let x1 = code_x + text_advance(&chars[..c1].iter().collect::<String>());
                let x1 = x1.min(code_x + code_w);
                if x1 <= x0 {
                    continue;
                }
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(x0, body_top + k as f32 * SRC_ROW_PITCH + LINE_W),
                        egui::vec2(x1 - x0, SRC_LINE_H),
                    ),
                    egui::CornerRadius::ZERO,
                    theme().sel_bg,
                );
            }
        }

        for k in 0..visible {
            let idx = self.scroll + k;
            if idx >= self.buf.line_count() {
                break;
            }
            // 一行文字的行盒上沿在行首那条线格往下 1px：文字那一行才踩在内容格起点上。
            let y = body_top + k as f32 * SRC_ROW_PITCH + LINE_W + SRC_LINE_H / 2.0;
            app.text(
                &painter,
                egui::pos2(text_right_x(gutter_line), y),
                egui::Align2::RIGHT_CENTER,
                &format!("{}", idx + 1),
                theme().cross,
            );
            let shown = fit_line(self.buf.line(idx), code_w);
            app.text(
                &text_painter,
                egui::pos2(code_x, y),
                egui::Align2::LEFT_CENTER,
                &shown,
                theme().text,
            );
        }

        // 光标（2px 宽的竖线，整数像素、不闪）+ 输入法候选串（在下划线上）。
        // 顺手把光标那个小条记下来：要给系统报输入法的候选窗口位置（见下面）。
        let mut caret_rect: Option<egui::Rect> = None;
        if focused {
            let (cl, cc) = self.buf.cursor;
            if cl >= self.scroll && cl < self.scroll + visible {
                let k = cl - self.scroll;
                let top = body_top + k as f32 * SRC_ROW_PITCH + LINE_W;
                let mid = top + SRC_LINE_H / 2.0;
                let head: String = self.buf.line(cl).chars().take(cc).collect();
                let mut x = (code_x + text_advance(&head)).round();
                if !self.preedit.is_empty() {
                    let pw = text_advance(&self.preedit);
                    app.text(
                        &text_painter,
                        egui::pos2(x, mid),
                        egui::Align2::LEFT_CENTER,
                        &self.preedit,
                        theme().text,
                    );
                    hline_hard(&text_painter, x, x + pw, top + SRC_LINE_H - LINE_W, theme().text);
                    x += pw;
                }
                let bar = egui::Rect::from_min_size(
                    egui::pos2(x, top),
                    egui::vec2(2.0 * LINE_W, SRC_LINE_H),
                );
                text_painter.rect_filled(bar, egui::CornerRadius::ZERO, theme().text);
                caret_rect = Some(bar);
            }
        }

        // 输入法候选窗贴哪儿：不报的话系统的拼音框会弹到屏幕左上角去。
        // egui 0.36.2 的入口是 PlatformOutput::ime（egui::output::IMEOutput，屏幕坐标），
        // egui-winit 拿它去调 winit 的 set_ime_cursor_area；照 TextEdit::show 里那份写。
        if focused {
            if let Some(bar) = caret_rect {
                let to_global = ui
                    .ctx()
                    .layer_transform_to_global(ui.layer_id())
                    .unwrap_or_default();
                let area = egui::Rect::from_min_max(
                    egui::pos2(code_x, body_top),
                    egui::pos2(bar_line, body_bottom),
                );
                ui.output_mut(|o| {
                    o.ime = Some(egui::output::IMEOutput {
                        purpose: egui::IMEPurpose::Normal,
                        rect: to_global * area,
                        cursor_rect: to_global * bar,
                        should_interrupt_composition: false,
                    });
                });
            }
        }

        // ---- 滚动条：1 格宽的槽 + 按内容比例算高度的把手（鼠标按住可以拖）----
        // 槽底铺满到窗口下边框；把手的行程比槽上下各收一点：贴着上沿时上边框正好压在
        // 按钮栏下沿那条共用线上（重合，仍是 1px），滑到底时下边框也离窗口下边框留一行空档
        // - 铁律：任何一条线都不许和别的线紧挨着堆成 2px。
        let track = egui::Rect::from_min_max(
            egui::pos2(bar_line + LINE_W, body_top),
            egui::pos2(panel.max.x - LINE_W, body_bottom),
        );
        let travel_min = body_top;
        let travel_max = body_bottom - LINE_W;
        let total = self.buf.line_count().max(1) as f32;
        let track_h = track.height();
        let handle_h = (track_h * visible as f32 / total)
            .round()
            .clamp(SRC_BAR_W, track_h.max(SRC_BAR_W));
        let max_scroll = self.max_scroll();
        let travel = (travel_max - travel_min - handle_h).max(1.0);
        // 某个滚动位置对应的把手矩形（交互先按这一帧开头那份 scroll 算，改完再算一次来画）。
        let handle_at = |scroll: usize| {
            let t = if max_scroll == 0 {
                0.0
            } else {
                scroll as f32 / max_scroll as f32
            };
            egui::Rect::from_min_size(
                egui::pos2(track.min.x, (travel_min + travel * t).round()),
                egui::vec2(track.width(), handle_h),
            )
        };
        let handle = handle_at(self.scroll);

        // 拖动：按住把手拖（把手不跳）；点槽 / 拖槽就把把手心挪到鼠标那儿。
        let bar_resp = ui.interact(track, id.with("scrollbar"), egui::Sense::click_and_drag());
        if bar_resp.hovered() || bar_resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
        if bar_resp.drag_started() {
            self.bar_grab = ui
                .input(|i| i.pointer.latest_pos())
                .map(|p| p.y - handle.min.y);
        }
        if bar_resp.clicked() {
            // 点的是槽（没点着把手）：把手心跳过来，接着就能拖。
            self.bar_grab = Some(handle_h / 2.0);
        }
        if bar_resp.dragged() {
            let grab = self.bar_grab.unwrap_or(handle_h / 2.0);
            if let Some(p) = ui.input(|i| i.pointer.latest_pos()) {
                let t = ((p.y - grab - track.min.y) / travel).clamp(0.0, 1.0);
                self.scroll = (t * max_scroll as f32).round() as usize;
            }
        }
        if bar_resp.drag_stopped() {
            self.bar_grab = None;
        }
        let handle = handle_at(self.scroll);

        // 槽：只铺浅底色，左右两侧不画线（左边紧挨着代码区那条竖线，再画一条就叠成 2px 了）。
        painter.rect_filled(track, egui::CornerRadius::ZERO, theme().splitter);
        // 把手：浅底色 + 上下两条 1px 线（1px 的线叠不出 2px），左右不画线。
        painter.rect_filled(handle, egui::CornerRadius::ZERO, theme().btn_face);
        hline_hard(
            &painter,
            handle.min.x,
            handle.max.x,
            handle.min.y,
            theme().ctrl_border,
        );
        hline_hard(
            &painter,
            handle.min.x,
            handle.max.x,
            handle.max.y - LINE_W,
            theme().ctrl_border,
        );
        // 把手中间三根短横线（抓手纹）：不连到左右边框上，落在网格线内。
        if handle_h >= 3.0 * GRID {
            let cx = handle.center().x;
            let cy = grid_snap(handle.center().y);
            let half = GRID / 2.0;
            for k in [-GRID, 0.0, GRID] {
                let y = cy + k;
                if y > handle.min.y + LINE_W && y < handle.max.y - 2.0 * LINE_W {
                    hline_hard(&painter, cx - half, cx + half, y, theme().line);
                }
            }
        }
        // 代码区和滚动条列之间那条竖线：两段共用的边界线，永远 1px。
        vline_hard(&painter, bar_line, body_top, body_bottom, theme().ctrl_border);
    }
}

/// 把一行截断到给定像素宽：放得下就原样返回；放不下就一个字一个字量到放不下的那个为止
///（只截断，不做横向滚动 - 和原来那份只读窗口一个口径）。
pub(crate) fn fit_line(line: &str, max_w: f32) -> String {
    if text_advance(line) <= max_w {
        return line.to_owned();
    }
    let mut out = String::new();
    let mut w = 0.0f32;
    for ch in line.chars() {
        let cw = char_advance(ch);
        if w + cw > max_w {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out
}