// ===========================================================================
//  模块 ui::editor：可编辑文本编辑器（全自绘）
// ===========================================================================
//  一块「菜单栏 + 按钮栏 + 行号区 + 代码区」的多行文本编辑器：光标、选区、滚动、中文输入法全在这
//  一份里。自绘浮动窗里的「源码编辑」用它，以后别的窗口要编辑文本也直接用这一份。
//
//  铁律：不许用 egui::TextEdit（连它的外观一起不要）。全 painter 手绘 - 半格都不许有；
//  坐标一律走 grid 那套（cells / grid_line / text_left_x / text_right_x），字号 16px 点阵字。
//  语法着色：关键字蓝、注释绿、字符串品红，其余代码文字（变量名 / 数字 / 标点等）走默认
//  文字色 - 颜色都在 theme 里，切皮肤跟着变（见 code_runs / paint_code_line）。
//  输入法：只有拿到焦点的那个控件才消费 IME 事件（egui 的 owns_ime_events），
//  候选串（Preedit）画在光标处并加 1px 下划线，上屏（Commit）才真正插进缓冲区；
//  候选窗位置通过 PlatformOutput::ime（IMEOutput）告诉系统 - 不设它，拼音框会弹在屏幕左上角。
//
//  【设计逻辑】UI 里每一个控件区域的外圈都必须有横竖线划分（线一律 1px、落在网格线内，
//  绝不叠成 2px）：标题栏 / 菜单栏 / 按钮栏 / 正文区从上往下依次排，相邻两段共用中间那条
//  1px 网格线 - 各段自己往里让开这条线画底色，分隔线统一留到最后再画一遍，谁都盖不掉
//  （见 paint_separators）。全工程的控件区分割都照这条来。
//  弹出层（菜单栏的下拉面板等）属于**当前窗口的最上层**：它必须排在窗口里所有内容之后画，
//  才盖得住底下的控件、不会透出别的元素；菜单栏一律左对齐，摆法和主窗口的那条完全一致
//  （共用 ui::menu::ui_menu_bar / ui_menu_popup）。
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
        items: &[
            MenuNode::act("保存"),
            MenuNode::act("编译"),
            MenuNode::act("还原原始源码"),
            MenuNode::act("关闭窗口"),
        ],
    },
    MenuTab {
        title: "编辑",
        items: &[
            MenuNode::act("撤销"),
            MenuNode::act("剪切"),
            MenuNode::act("复制"),
            MenuNode::act("粘贴"),
            MenuNode::act("全选"),
            MenuNode::sub(
                "查找",
                &[MenuNode::act("查找下一个")],
            ),
        ],
    },
    MenuTab {
        title: "查看",
        items: &[MenuNode::act("回到开头"), MenuNode::act("跳到末尾")],
    },
];

/// 编辑器按钮栏上那一排按钮（图标 + 悬停便签 + 动作），从左往右：还原 / 保存 / 编译 /
/// 剪切 / 复制 / 粘贴 / 撤销 / 查找。便签就是鼠标悬停时显示的那句按钮含义。
pub(crate) const EDITOR_BUTTONS: [(IconKind, &str, EditorCmd); 8] = [
    (IconKind::New, "还原原始源码", EditorCmd::Reload),
    (IconKind::Save, "保存到源文件", EditorCmd::Save),
    (IconKind::Compile, "编译（cargo check）", EditorCmd::Compile),
    (IconKind::Cut, "剪切选中", EditorCmd::Cut),
    (IconKind::Copy, "复制选中", EditorCmd::Copy),
    (IconKind::Paste, "粘贴", EditorCmd::Paste),
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
    /// 正文区一屏能画几行：按窗口高度每帧现算（默认就是自然高度下的一屏行数）。
    pub(crate) rows: usize,
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
            rows: SRC_VISIBLE_LINES,
            preedit: String::new(),
            wheel_acc: 0.0,
            menu: MenuState::default(),
            focused: false,
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
        self.selecting = false;
        self.modified = false;
    }

    /// 滚动上限：总行数减一屏行数。
    pub(crate) fn max_scroll(&self) -> usize {
        self.buf.line_count().saturating_sub(self.rows)
    }

    /// 光标跑到屏幕外了就把它拉回来（键盘翻页 / 打字之后自动跟随）。
    fn ensure_caret_visible(&mut self) {
        let l = self.buf.cursor.0;
        if l < self.scroll {
            self.scroll = l;
        } else if l >= self.scroll + self.rows {
            self.scroll = l + 1 - self.rows;
        }
        self.scroll = self.scroll.min(self.max_scroll());
    }

    /// 画整块编辑器并处理交互：菜单栏 -> 按钮栏 -> 正文区 -> 分隔线 -> 下拉面板（本窗最上层）。
    /// 返回宿主还要去做的动作（保存 / 编译 / 还原 / 关闭）。
    pub(crate) fn ui(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        panel: egui::Rect,
        id: egui::Id,
    ) -> EditorCmd {
        let mut cmd = EditorCmd::None;
        // 正文区一屏几行：按窗口高度现算（行高 SRC_ROW_PITCH = 2 格），窗口改大改小都跟着走。
        self.rows = (((panel.max.y - LINE_W - (panel.min.y + SRC_BODY_TOP)) / SRC_ROW_PITCH).floor()
            as usize)
            .max(1);
        let menu_band = grid_band(panel, SRC_TITLE_ROWS, SRC_MENU_ROWS);
        let bar_band = grid_band(panel, SRC_TITLE_ROWS + SRC_MENU_ROWS, SRC_BAR_ROWS);
        // 菜单栏走 ui::menu 那一份（左对齐、相邻两项共用网格线、下沿那条 1px 线），
        // 和主窗口的菜单栏是同一套实现、同一个摆法。
        let (open, state) = ui_menu_bar(app, ui, menu_band, &EDITOR_TABS, self.menu.clone());
        self.menu = state;
        self.ui_buttonbar(app, ui, bar_band, &mut cmd);
        self.ui_body(app, ui, panel, id, &mut cmd);
        // 各段之间那条 1px 分隔线排在内容之后画：铁律 - 每个控件区的外圈都得有落在网格线上的线，
        // 而且谁都盖不掉（内容先画、线后画，永远只有 1px）。
        self.paint_separators(&ui.painter().clone(), panel);
        // 下拉面板：本窗口的最上层（盖住菜单栏 / 按钮栏 / 正文 / 分隔线），挂在菜单项下沿、
        // 左对齐、整块夹在这块窗口里 - 和主窗口菜单栏是同一份实现（ui::menu::ui_menu_popup）。
        if let Some((idx, anchor)) = open {
            let items = EDITOR_TABS[idx].items;
            let mut menu = self.menu.clone();
            if let Some(label) = ui_menu_popup(app, ui, anchor, items, &mut menu, panel, &[menu_band])
            {
                cmd = EditorCmd::of(label);
            }
            self.menu = menu;
        }
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
        let caret_before = self.buf.cursor;
        let out = match cmd {
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
        };
        // 这些动作可能把光标挪到屏幕外（撤销 / 查找 / 回开头 / 跳末尾）：只在光标真动了的时候
        // 把视图拉回来。光标没动就一律不碰 scroll - 编辑和滚动互不干扰。
        if self.buf.cursor != caret_before {
            self.ensure_caret_visible();
        }
        out
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
        // 代码区铺到窗口右边缘为止：右边是「1px 线格 + 8px 滚动条 + 最右那条边框线」。
        let bar_line = panel.max.x - 2.0 * LINE_W - SRC_BAR_W;
        // 文字的吸附原点 = 这块窗口自己的网格（面板左边那条线）。
        let code_x = text_left_x(gutter_line, panel.min.x);
        let code_w = (bar_line - GRID - code_x).max(0.0);
        let visible = self.rows;

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
            // 点进来的时候要一次 egui 的键盘焦点（要了才能收按键 / 输入法事件）。
            ui.memory_mut(|m| m.request_focus(id));
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
        // egui 的键盘焦点：只在「真的丢了」的时候补要一次，绝不能每帧都 request_focus。
        // 铁律：Memory::request_focus 内部会顺手 interrupt_ime()，每帧都调就变成每帧
        // set_ime_allowed(false)/(true) 重置一次输入法 - 表相就是「输入法弹出来了，拼音却
        // 永远上不了屏」。所以打字期间（有候选串）一次都不许打断，焦点只认 self.focused。
        if !self.focused {
            ui.memory_mut(|m| m.surrender_focus(id));
        } else if self.preedit.is_empty() && !ui.memory(|m| m.has_focus(id)) {
            ui.memory_mut(|m| m.request_focus(id));
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
            let caret_before = self.buf.cursor;
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
            // 只在光标真的动了的时候把视图拉回来；光标停着不动的帧（比如只是滚轮往下看）
            // 一律不碰 scroll - 编辑和滚动互不干扰。
            if self.buf.cursor != caret_before {
                self.ensure_caret_visible();
            }
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
                egui::pos2(text_right_x(gutter_line, panel.min.x), y),
                egui::Align2::RIGHT_CENTER,
                &format!("{}", idx + 1),
                theme().cross,
            );
            paint_code_line(app, &text_painter, code_x, y, self.buf.line(idx), code_w);
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

        // ---- 滚动条：1 格宽的槽 + 按内容比例算高度的把手（鼠标滚轮和拖拽把手都能滚）----
        // 行程两端都压在相邻的边框线上：顶到最上时把手的上沿和按钮栏下沿那条共用线重合，
        // 滑到最下时把手底边和下边框线重合 - 上下一样齐。铁律：线不许叠成 2px，也不许差一行。
        let track = egui::Rect::from_min_max(
            egui::pos2(bar_line + LINE_W, body_top),
            egui::pos2(panel.max.x - LINE_W, body_bottom),
        );
        let travel_min = body_top;
        let travel_max = panel.max.y;
        let total = self.buf.line_count().max(1) as f32;
        let track_h = track.height();
        let handle_h = (track_h * visible as f32 / total)
            .round()
            .clamp(SRC_BAR_W, track_h.max(SRC_BAR_W));
        let max_scroll = self.max_scroll();
        let travel = (travel_max - travel_min - handle_h).max(1.0);
        // 某个滚动位置对应的把手矩形。
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

        // 拖拽：按住把手上下拖 = 直接滚到那个位置（和滚轮同一套范围，互相不干扰）。
        // 热区 id 固定挂在编辑器自己的 id 上 - 把手每帧都在动，用「按矩形算」的 id 会一帧一个样，
        // 按下的那一半和松手的那一半对不上，拖拽直接就丢了（见 grid::interact_rect 的说明）。
        let bar = ui.interact(handle, id.with("vscroll"), egui::Sense::click_and_drag());
        if bar.hovered() || bar.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
        if bar.dragged() {
            if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
                // 拿鼠标当把手的中心：抓住把手哪一处，滚动都跟手，不会跳。
                let t = ((p.y - handle_h / 2.0 - travel_min) / travel).clamp(0.0, 1.0);
                self.scroll = (t * max_scroll as f32).round() as usize;
            }
        }
        // 拖完按新的滚动位置重算把手：这一帧画出来的把手就跟着鼠标走，不多拖一帧。
        let handle = handle_at(self.scroll);
        // 槽：只铺槽底色，左右两侧不画线（左边紧挨着代码区那条竖线，再画一条就叠成 2px 了）。
        painter.rect_filled(track, egui::CornerRadius::ZERO, theme().scroll_track);
        // 把手：把手底色（比槽亮一档，滚到哪儿一眼看得出来）+ 上下两条 1px 线（1px 的线叠不出
        // 2px），左右不画线。
        painter.rect_filled(handle, egui::CornerRadius::ZERO, theme().scroll_thumb);
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

        // 代码区和滚动条列之间那条竖线：两段共用的边界线，永远 1px。
        vline_hard(&painter, bar_line, body_top, body_bottom, theme().ctrl_border);
    }
}

/// 一行源码里一个着色片段的种类。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SynKind {
    /// 普通代码：变量名 / 数字 / 标点 / 运算符等，一律走默认文字色。
    Plain,
    /// 关键字：`let` / `fn` / `impl` 这些（用户点名的蓝色）。
    Keyword,
    /// 注释：`//` 到行尾（绿色）。
    Comment,
    /// 字符串：一对引号之间（品红色）。
    String,
}

/// Rust 关键字 + 内置类型名：这些画成关键字蓝，其余标识符（变量名 / 函数名等）走默认文字色。
const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
    "type", "union", "unsafe", "use", "where", "while", "abstract", "become", "box", "do",
    "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "bool", "char",
    "str", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64",
    "u128", "usize",
];

/// 是不是 Rust 关键字 / 内置类型名。
fn is_rust_keyword(word: &str) -> bool {
    RUST_KEYWORDS.contains(&word)
}

/// 往 runs 末尾追加一段；和上一段同色就并进去，免得切出一堆碎片。
fn push_run(runs: &mut Vec<(String, SynKind)>, text: String, kind: SynKind) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = runs.last_mut() {
        if last.1 == kind {
            last.0.push_str(&text);
            return;
        }
    }
    runs.push((text, kind));
}

/// 把一段「代码」（非字符串、非注释）切成片段：关键字算 Keyword（蓝色），
/// 变量名 / 数字 / 标点 / 运算符 / 空白一律算 Plain（默认文字色）。
fn push_code(runs: &mut Vec<(String, SynKind)>, code: &str) {
    let chars: Vec<char> = code.chars().collect();
    let n = chars.len();
    let mut i = 0;
    while i < n {
        let start = i;
        if chars[i].is_alphabetic() || chars[i] == '_' {
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let kind = if is_rust_keyword(&word) { SynKind::Keyword } else { SynKind::Plain };
            push_run(runs, word, kind);
        } else {
            while i < n && !(chars[i].is_alphabetic() || chars[i] == '_') {
                i += 1;
            }
            push_run(runs, chars[start..i].iter().collect(), SynKind::Plain);
        }
    }
}

/// 把一行源码切成若干着色片段：`"..."` 之间算字符串，`//` 到行尾算注释，其余算代码再细分
/// 成「关键字（蓝）/ 普通代码（默认色）」。片段拼起来就是原行，一个字都不丢、也不插字。
pub(crate) fn code_runs(line: &str) -> Vec<(String, SynKind)> {
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut runs: Vec<(String, SynKind)> = Vec::new();
    let mut code = String::new();
    let mut i = 0;
    while i < n {
        if chars[i] == '"' {
            push_code(&mut runs, &code);
            code.clear();
            // 从这个引号一直吃到下一个引号；没有收尾的（半行字符串）就算到行尾。
            let mut s = String::from('"');
            i += 1;
            while i < n {
                s.push(chars[i]);
                let closed = chars[i] == '"';
                i += 1;
                if closed {
                    break;
                }
            }
            push_run(&mut runs, s, SynKind::String);
        } else if chars[i] == '/' && i + 1 < n && chars[i + 1] == '/' {
            push_code(&mut runs, &code);
            code.clear();
            push_run(&mut runs, chars[i..].iter().collect(), SynKind::Comment);
            i = n;
        } else {
            code.push(chars[i]);
            i += 1;
        }
    }
    push_code(&mut runs, &code);
    runs
}

/// 一个片段该用什么颜色：一律从当前皮肤取（见 theme 里的 syn_code / syn_comment / syn_string）。
pub(crate) fn syn_color(kind: SynKind) -> egui::Color32 {
    match kind {
        SynKind::Plain => theme().text,
        SynKind::Keyword => theme().syn_code,
        SynKind::Comment => theme().syn_comment,
        SynKind::String => theme().syn_string,
    }
}
/// 画一行带语法着色的源码：逐段量宽、接着往右排；超出 code_w 的那一段用 fit_line 截断。
/// 落点全在整数像素上，段与段之间不留缝也不重叠（拼起来和不着色时一模一样）。
pub(crate) fn paint_code_line(
    app: &mut StockApp,
    painter: &egui::Painter,
    code_x: f32,
    y: f32,
    line: &str,
    code_w: f32,
) {
    let end = code_x + code_w;
    let mut x = code_x;
    for (text, kind) in code_runs(line) {
        if x >= end {
            break;
        }
        let color = syn_color(kind);
        let w = text_advance(&text);
        if x + w > end {
            let shown = fit_line(&text, end - x);
            if !shown.is_empty() {
                app.text(painter, egui::pos2(x, y), egui::Align2::LEFT_CENTER, &shown, color);
            }
            break;
        }
        app.text(painter, egui::pos2(x, y), egui::Align2::LEFT_CENTER, &text, color);
        x += w;
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
