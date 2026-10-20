use crate::app::*;
use crate::font::{char_advance, text_advance};
use crate::grid::*;
use crate::theme::*;
use crate::ui::button::*;
use crate::ui::contextmenu::*;
use crate::ui::icon::CursorKind;
use crate::ui::menu::*;
use crate::ui::textbuf::*;
use eframe::egui;
use std::collections::BTreeSet;

pub(crate) const EDITOR_TABS: [MenuTab; 3] = [
    MenuTab {
        title: "文件",
        items: &[
            MenuNode::act("保存"),
            MenuNode::act("还原原始指标代码"),
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
            MenuNode::sub("查找", &[MenuNode::act("查找下一个")]),
        ],
    },
    MenuTab {
        title: "查看",
        items: &[MenuNode::act("回到开头"), MenuNode::act("跳到末尾")],
    },
];

pub(crate) const EDITOR_BUTTONS: [(&str, &str, EditorCmd); 7] = [
    ("新建", "新建空白文件", EditorCmd::New),
    ("保存", "保存指标代码", EditorCmd::Save),
    ("剪切", "剪切选中", EditorCmd::Cut),
    ("复制", "复制选中", EditorCmd::Copy),
    ("粘贴", "粘贴", EditorCmd::Paste),
    ("撤销", "撤销", EditorCmd::Undo),
    ("查找", "查找下一个", EditorCmd::Find),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditorCmd {
    None,

    New,

    Save,

    Reload,

    Close,
    Cut,
    Copy,
    Paste,
    Undo,
    SelectAll,

    Find,
    DocHome,
    DocEnd,
}

impl EditorCmd {
    fn of(label: &str) -> EditorCmd {
        match label {
            "保存" => EditorCmd::Save,
            "还原原始指标代码" => EditorCmd::Reload,
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

pub(crate) struct EditOutcome {
    pub(crate) changed: bool,

    pub(crate) save: bool,

    pub(crate) copy: Option<String>,
}

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

#[derive(Clone)]
pub(crate) struct TextEditor {
    pub(crate) buf: TextBuffer,

    pub(crate) scroll: usize,

    pub(crate) scroll_grab: Option<egui::Vec2>,

    pub(crate) rows: usize,

    pub(crate) preedit: String,

    pub(crate) wheel_acc: f32,

    pub(crate) menu: MenuState,

    pub(crate) focused: bool,

    pub(crate) selecting: bool,

    pub(crate) find_term: String,

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
            scroll_grab: None,
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

    pub(crate) fn load(&mut self, text: &str) {
        self.buf = TextBuffer::from_text(text);
        self.scroll = 0;
        self.preedit.clear();
        self.wheel_acc = 0.0;
        self.selecting = false;
        self.modified = false;
    }

    pub(crate) fn max_scroll(&self) -> usize {
        self.buf.line_count().saturating_sub(self.rows)
    }

    fn ensure_caret_visible(&mut self) {
        let l = self.buf.cursor.0;
        if l < self.scroll {
            self.scroll = l;
        } else if l >= self.scroll + self.rows {
            self.scroll = l + 1 - self.rows;
        }
        self.scroll = self.scroll.min(self.max_scroll());
    }

    pub(crate) fn ui(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        panel: egui::Rect,
        id: egui::Id,
    ) -> EditorCmd {
        let mut cmd = EditorCmd::None;

        self.rows =
            (((panel.max.y - LINE_W - (panel.min.y + SRC_BODY_TOP)) / SRC_ROW_PITCH).floor()
                as usize)
                .max(1);
        let menu_band = grid_band(panel, SRC_TITLE_ROWS, SRC_MENU_ROWS);
        let bar_band = grid_band(panel, SRC_TITLE_ROWS + SRC_MENU_ROWS, SRC_BAR_ROWS);

        let (open, state) = ui_menu_bar(app, ui, menu_band, &EDITOR_TABS, self.menu.clone());
        self.menu = state;
        self.ui_buttonbar(app, ui, bar_band, &mut cmd);
        self.ui_body(app, ui, panel, id, &mut cmd);

        self.paint_separators(&ui.painter().clone(), panel);

        if let Some((idx, anchor)) = open {
            let items = EDITOR_TABS[idx].items;
            let mut menu = self.menu.clone();
            if let Some(label) =
                ui_menu_popup(app, ui, anchor, items, &mut menu, panel, &[menu_band])
            {
                cmd = EditorCmd::of(label);
            }
            self.menu = menu;
        }
        self.finish_cmd(ui, cmd)
    }

    fn paint_separators(&self, painter: &egui::Painter, panel: egui::Rect) {
        for rows in [SRC_TITLE_ROWS + SRC_MENU_ROWS, SRC_HEAD_ROWS] {
            let y = panel.min.y + grid_line(rows);
            hline_hard(painter, panel.min.x, panel.max.x, y, theme().editor_border);
        }
    }

    fn finish_cmd(&mut self, ui: &egui::Ui, cmd: EditorCmd) -> EditorCmd {
        let caret_before = self.buf.cursor;
        let out = match cmd {
            EditorCmd::None => EditorCmd::None,
            EditorCmd::New | EditorCmd::Save | EditorCmd::Reload | EditorCmd::Close => cmd,
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
                if let Some(text) = crate::platform::clipboard_text() {
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

        if self.buf.cursor != caret_before {
            self.ensure_caret_visible();
        }
        out
    }

    pub(crate) fn apply_context_action(&mut self, ui: &egui::Ui, action: ContextAction) {
        let cmd = match action {
            ContextAction::Cut => EditorCmd::Cut,
            ContextAction::Copy => EditorCmd::Copy,
            ContextAction::Paste => EditorCmd::Paste,
            ContextAction::SelectAll => EditorCmd::SelectAll,
            ContextAction::Undo => EditorCmd::Undo,
            _ => return,
        };
        let _ = self.finish_cmd(ui, cmd);
    }

    fn ui_buttonbar(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        band: egui::Rect,
        cmd: &mut EditorCmd,
    ) {
        let painter = ui.painter().clone();

        painter.rect_filled(
            egui::Rect::from_min_max(egui::pos2(band.min.x, band.min.y + LINE_W), band.max),
            egui::CornerRadius::ZERO,
            theme().panel_bg,
        );
        let mut x = band.min.x + grid_line(1);
        for (label, tip, action) in EDITOR_BUTTONS {
            let label = label.to_owned();
            let content = ButtonContent::Text(label.clone());
            let w = app.button_width(&content);
            if x + w > band.max.x {
                break;
            }
            let rect =
                egui::Rect::from_min_size(egui::pos2(x, band.min.y), egui::vec2(w, band.height()));
            if app.draw_button(ui, &painter, rect, &content, false, tip) {
                *cmd = action;
            }
            x += w - LINE_W;
        }
    }

    fn pos_at(&self, p: egui::Pos2, code_x: f32, body_top: f32) -> (usize, usize) {
        let row = ((p.y - body_top) / SRC_ROW_PITCH).floor().max(0.0) as usize;
        let line = (self.scroll + row).min(self.buf.line_count().saturating_sub(1));
        let x = (p.x - code_x).max(0.0);
        let mut w = 0.0f32;
        let mut col = 0usize;
        for ch in self.buf.line(line).chars() {
            let cw = char_advance(ch);

            if x < w + cw * 0.5 {
                break;
            }
            w += cw;
            col += 1;
        }
        (line, col)
    }

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

        let bar_line = panel.max.x - 2.0 * LINE_W - SRC_BAR_W;

        let code_x = text_left_x(gutter_line, panel.min.x);
        let code_w = (bar_line - GRID - code_x).max(0.0);
        let visible = self.rows;

        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(panel.min.x + LINE_W, body_top),
                egui::pos2(panel.max.x - LINE_W, body_bottom),
            ),
            egui::CornerRadius::ZERO,
            theme().editor_bg,
        );
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(panel.min.x + LINE_W, body_top + LINE_W),
                egui::pos2(gutter_line, body_bottom),
            ),
            egui::CornerRadius::ZERO,
            theme().editor_gutter_bg,
        );
        vline_hard(
            &painter,
            gutter_line,
            body_top,
            body_bottom,
            theme().editor_border,
        );

        let hovered = !app.win_silent
            && ui
                .input(|i| i.pointer.hover_pos())
                .is_some_and(|p| panel.contains(p));
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

        let body = egui::Rect::from_min_max(
            egui::pos2(panel.min.x + LINE_W, body_top),
            egui::pos2(panel.max.x - LINE_W, body_bottom),
        );

        let window_active = !app.win_silent;
        if !window_active {
            self.focused = false;
            ui.memory_mut(|m| m.surrender_focus(id));
        }
        let resp = ui.interact(body, id, egui::Sense::click_and_drag());
        let text_area = egui::Rect::from_min_max(
            egui::pos2(panel.min.x + LINE_W, body_top),
            egui::pos2(bar_line, body_bottom),
        );
        if ui
            .input(|i| i.pointer.hover_pos())
            .is_some_and(|p| text_area.contains(p))
        {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
            app.pointer.text_hover = true;
            app.pointer.hint = CursorKind::Text;
        }
        let primary_clicked_in_body = ui.input(|i| {
            i.pointer.primary_clicked()
                && i.pointer
                    .interact_pos()
                    .or_else(|| i.pointer.latest_pos())
                    .is_some_and(|p| body.contains(p))
        });
        if primary_clicked_in_body || resp.drag_started() {
            self.focused = true;

            ui.memory_mut(|m| m.request_focus(id));
        }

        if self.focused {
            let outside_click = ui.ctx().input(|i| {
                i.pointer.primary_clicked()
                    && i.pointer.interact_pos().is_some_and(|p| !panel.contains(p))
            });
            if outside_click {
                self.focused = false;
            }
        }

        if !self.focused {
            ui.memory_mut(|m| m.surrender_focus(id));
        } else if self.preedit.is_empty() && !ui.memory(|m| m.has_focus(id)) {
            ui.memory_mut(|m| m.request_focus(id));
        }
        let focused = self.focused;

        let (pointer_pos, press_pos, primary_clicked) = ui.input(|i| {
            let pointer_pos = i.pointer.interact_pos().or_else(|| i.pointer.latest_pos());
            (
                pointer_pos,
                i.pointer.press_origin(),
                i.pointer.primary_clicked() && pointer_pos.is_some_and(|p| body.contains(p)),
            )
        });
        let clamp_to_text = |p: egui::Pos2| {
            egui::pos2(
                p.x.clamp(text_area.min.x, text_area.max.x - LINE_W),
                p.y.clamp(text_area.min.y, text_area.max.y - LINE_W),
            )
        };
        if let Some(p) = pointer_pos {
            if resp.secondary_clicked() {
                self.focused = true;
                ui.memory_mut(|m| m.request_focus(id));
                let pos = self.pos_at(clamp_to_text(p), code_x, body_top);
                if !self.buf.selection_contains(pos) {
                    self.buf.set_cursor(pos, false);
                }
                app.context_menu.open_editor(p);
            } else if resp.drag_started() {
                self.selecting = true;

                let start = self.pos_at(clamp_to_text(press_pos.unwrap_or(p)), code_x, body_top);
                self.buf.set_cursor(start, false);
                self.buf.sel_anchor = Some(start);
                let now = self.pos_at(clamp_to_text(p), code_x, body_top);
                self.buf.set_cursor(now, true);
            } else if self.selecting && ui.input(|i| i.pointer.primary_down()) {
                let at = clamp_to_text(p);
                app.pointer.pinned = Some(at);
                let pos = self.pos_at(at, code_x, body_top);
                self.buf.set_cursor(pos, true);
            } else if primary_clicked {
                let pos = self.pos_at(clamp_to_text(p), code_x, body_top);
                self.buf.set_cursor(pos, false);
            }
        }
        if !ui.input(|i| i.pointer.primary_down()) {
            self.selecting = false;
        }

        if focused && window_active {
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
            let out = apply_edit_events(
                &mut self.buf,
                &mut self.preedit,
                &events,
                true,
                &char_allowed,
            );
            if out.changed {
                self.modified = true;
            }
            if let Some(text) = out.copy {
                ui.ctx().copy_text(text);
            }
            if out.save {
                *cmd = EditorCmd::Save;
            }

            if self.buf.cursor != caret_before {
                self.ensure_caret_visible();
            }
        } else {
            self.preedit.clear();
        }

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

            let y = body_top + k as f32 * SRC_ROW_PITCH + LINE_W + SRC_LINE_H / 2.0;
            app.text(
                &painter,
                egui::pos2(text_right_x(gutter_line, panel.min.x), y),
                egui::Align2::RIGHT_CENTER,
                &format!("{}", idx + 1),
                theme().editor_line_number,
            );
        }
        let user_names = code_user_names(&self.buf.lines);
        for k in 0..visible {
            let idx = self.scroll + k;
            if idx >= self.buf.line_count() {
                break;
            }
            let y = body_top + k as f32 * SRC_ROW_PITCH + LINE_W + SRC_LINE_H / 2.0;
            paint_code_line(
                app,
                &text_painter,
                code_x,
                y,
                self.buf.line(idx),
                code_w,
                &user_names,
            );
        }

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
                        theme().editor_text,
                    );
                    hline_hard(
                        &text_painter,
                        x,
                        x + pw,
                        top + SRC_LINE_H - LINE_W,
                        theme().editor_text,
                    );
                    x += pw;
                }

                let caret = text_caret_rect(x, top, SRC_LINE_H);

                text_painter.rect_filled(caret, egui::CornerRadius::ZERO, theme().editor_text);
                caret_rect = Some(caret);
            }
        }

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

        let total = self.buf.line_count().max(1) as f32;
        let max_scroll = self.max_scroll();
        vertical_scrollbar(
            app,
            ui,
            &painter,
            id,
            panel,
            body_top,
            body_bottom,
            visible,
            total as usize,
            max_scroll,
            &mut self.scroll,
            &mut self.scroll_grab,
        );

        if self.selecting
            && ui.input(|i| i.pointer.primary_down())
            && let Some(p) =
                ui.input(|i| i.pointer.interact_pos().or_else(|| i.pointer.latest_pos()))
        {
            app.pointer.pinned = Some(clamp_to_text(p));
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SynKind {
    Plain,

    UserName,

    Comment,
}

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

pub(crate) fn code_user_names(lines: &[String]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in lines {
        let chars: Vec<char> = line.chars().collect();
        let n = chars.len();
        let mut i = 0;
        let mut in_string = false;
        let mut escaped = false;
        while i < n {
            let ch = chars[i];
            if in_string {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_string = false;
                }
                i += 1;
                continue;
            }
            if ch == '"' {
                in_string = true;
                i += 1;
                continue;
            }
            if ch == '/' && chars.get(i + 1) == Some(&'/') {
                break;
            }
            if ch == '#' {
                let color_end = i + 7;
                let color = color_end <= n
                    && chars[i + 1..color_end]
                        .iter()
                        .all(|digit| digit.is_ascii_hexdigit())
                    && chars
                        .get(color_end)
                        .is_none_or(|next| !next.is_ascii_alphanumeric() && *next != '_');
                i = if color { color_end } else { n };
                continue;
            }
            if ch.is_ascii_alphabetic() || ch == '_' {
                let start = i;
                i += 1;
                while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let mut next = i;
                while next < n && chars[next].is_whitespace() {
                    next += 1;
                }
                if chars.get(next) == Some(&':') || chars.get(next) == Some(&'=') {
                    names.insert(chars[start..i].iter().collect());
                }
                continue;
            }
            i += 1;
        }
    }
    names
}

#[cfg(test)]
pub(crate) fn code_runs(line: &str) -> Vec<(String, SynKind)> {
    code_runs_with_names(line, &BTreeSet::new())
}

pub(crate) fn code_runs_with_names(
    line: &str,
    user_names: &BTreeSet<String>,
) -> Vec<(String, SynKind)> {
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut runs: Vec<(String, SynKind)> = Vec::new();
    let mut code = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    while i < n {
        let ch = chars[i];
        if !in_string && ch == '/' && chars.get(i + 1) == Some(&'/') {
            push_run(&mut runs, std::mem::take(&mut code), SynKind::Plain);
            code.clear();
            push_run(&mut runs, chars[i..].iter().collect(), SynKind::Comment);
            return runs;
        }
        if !in_string && ch == '#' {
            let color_end = i + 7;
            let color = color_end <= n
                && chars[i + 1..color_end]
                    .iter()
                    .all(|digit| digit.is_ascii_hexdigit())
                && chars
                    .get(color_end)
                    .is_none_or(|next| !next.is_ascii_alphanumeric() && *next != '_');
            if color {
                code.extend(chars[i..color_end].iter());
                i = color_end;
                continue;
            }
            push_run(&mut runs, std::mem::take(&mut code), SynKind::Plain);
            push_run(&mut runs, chars[i..].iter().collect(), SynKind::Comment);
            return runs;
        }
        if !in_string && (ch.is_ascii_alphabetic() || ch == '_') {
            let start = i;
            i += 1;
            while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let ident: String = chars[start..i].iter().collect();
            let kind = if user_names.contains(&ident) {
                SynKind::UserName
            } else {
                SynKind::Plain
            };
            push_run(&mut runs, std::mem::take(&mut code), SynKind::Plain);
            push_run(&mut runs, ident, kind);
            continue;
        }
        code.push(ch);
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
        }
        i += 1;
    }
    push_run(&mut runs, code, SynKind::Plain);
    runs
}

pub(crate) fn syn_color(kind: SynKind) -> egui::Color32 {
    match kind {
        SynKind::Plain => theme().editor_text,
        SynKind::UserName => theme().syn_keyword,
        SynKind::Comment => theme().syn_comment,
    }
}

pub(crate) fn paint_code_line(
    app: &mut StockApp,
    painter: &egui::Painter,
    code_x: f32,
    y: f32,
    line: &str,
    code_w: f32,
    user_names: &BTreeSet<String>,
) {
    let end = code_x + code_w;
    let mut x = code_x;
    for (text, kind) in code_runs_with_names(line, user_names) {
        if x >= end {
            break;
        }
        let color = syn_color(kind);
        let w = text_advance(&text);
        if x + w > end {
            let shown = fit_line(&text, end - x);
            if !shown.is_empty() {
                app.text(
                    painter,
                    egui::pos2(x, y),
                    egui::Align2::LEFT_CENTER,
                    &shown,
                    color,
                );
            }
            break;
        }
        app.text(
            painter,
            egui::pos2(x, y),
            egui::Align2::LEFT_CENTER,
            &text,
            color,
        );
        x += w;
    }
}

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

pub(crate) fn scrollbar_track(panel: egui::Rect) -> egui::Rect {
    let bar_line = panel.max.x - 2.0 * LINE_W - SRC_BAR_W;
    egui::Rect::from_min_max(
        egui::pos2(bar_line + LINE_W, panel.min.y + SRC_BODY_TOP),
        egui::pos2(panel.max.x - LINE_W, panel.max.y - LINE_W),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn vertical_scrollbar(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    id: egui::Id,
    panel: egui::Rect,
    body_top: f32,
    body_bottom: f32,
    visible: usize,
    total: usize,
    max_scroll: usize,
    scroll: &mut usize,
    scroll_grab: &mut Option<egui::Vec2>,
) {
    let bar_line = panel.max.x - 2.0 * LINE_W - SRC_BAR_W;
    let track = egui::Rect::from_min_max(
        egui::pos2(bar_line + LINE_W, body_top),
        egui::pos2(panel.max.x - LINE_W, body_bottom),
    );
    let total = total.max(1) as f32;
    let track_h = track.height();
    let handle_h = (track_h * visible as f32 / total)
        .round()
        .clamp(SRC_BAR_W, track_h.max(SRC_BAR_W));
    let travel = (track_h - handle_h).max(1.0);

    let handle_at = |value: usize| {
        let t = if max_scroll == 0 {
            0.0
        } else {
            value as f32 / max_scroll as f32
        };
        egui::Rect::from_min_size(
            egui::pos2(track.min.x, (body_top + travel * t).round()),
            egui::vec2(track.width(), handle_h),
        )
    };
    let handle = handle_at(*scroll);

    let track_resp = ui.interact(
        track,
        id.with("vscroll_track"),
        egui::Sense::click_and_drag(),
    );
    let bar = ui.interact(handle, id.with("vscroll"), egui::Sense::click_and_drag());
    if bar.hovered() || bar.dragged() || track_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        app.pointer.hint = CursorKind::Slider;
    }

    let (down, press, egui_pointer) = ui.input(|i| {
        (
            i.pointer.primary_down(),
            i.pointer.press_origin(),
            i.pointer.interact_pos(),
        )
    });

    let pointer = app.pointer.live.or(egui_pointer);
    let press = app.pointer.live.or(press);
    if !down {
        *scroll_grab = None;
    }
    if down
        && scroll_grab.is_none()
        && (bar.is_pointer_button_down_on() || track_resp.is_pointer_button_down_on())
    {
        if let Some(p0) = press {
            *scroll_grab = Some(if handle.contains(p0) {
                p0 - handle.min
            } else {
                egui::vec2(0.0, handle_h / 2.0)
            });
        }
    }
    if let (true, Some(grab), Some(p)) = (down, *scroll_grab, pointer) {
        let top = p.y - grab.y;
        let want = (top - body_top) / travel;
        let t = want.clamp(0.0, 1.0);
        *scroll = (t * max_scroll as f32).round() as usize;
        if app.pointer.system && want != t {
            let anchored = handle_at(*scroll);
            *scroll_grab = Some(p - anchored.min);
        }
    }

    let handle = handle_at(*scroll);

    if !app.pointer.system
        && let Some(grab) = *scroll_grab
    {
        app.pointer.pinned = Some(handle.min + grab);
    }

    painter.rect_filled(track, egui::CornerRadius::ZERO, theme().scroll_track);
    painter.rect_filled(handle, egui::CornerRadius::ZERO, theme().scroll_thumb);
    hline_hard(
        painter,
        handle.min.x,
        handle.max.x,
        handle.min.y,
        theme().scroll_border,
    );
    if handle.max.y < body_bottom {
        hline_hard(
            painter,
            handle.min.x,
            handle.max.x,
            handle.max.y - LINE_W,
            theme().scroll_border,
        );
    }
    vline_hard(
        painter,
        bar_line,
        body_top,
        body_bottom,
        theme().scroll_border,
    );
}
