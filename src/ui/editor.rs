use crate::application::app::*;
use crate::domain::layout::*;
use crate::engine::font::{char_advance, text_advance};
use crate::engine::kit;
use crate::engine::theme::*;
use crate::ui::button::*;
use crate::ui::contextmenu::*;
use crate::ui::icon::CursorKind;
use crate::ui::menu::*;
use crate::ui::textbuf::*;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const EDITOR_TABS: [MenuTab; 3] = [
    MenuTab {
        title: "File",
        items: &[
            MenuNode::act("Save"),
            MenuNode::act("Restore Original Source"),
            MenuNode::act("Close Window"),
        ],
    },
    MenuTab {
        title: "Edit",
        items: &[
            MenuNode::act("Undo"),
            MenuNode::act("Cut"),
            MenuNode::act("Copy"),
            MenuNode::act("Paste"),
            MenuNode::act("Select All"),
            MenuNode::sub("Find", &[MenuNode::act("Find Next")]),
        ],
    },
    MenuTab {
        title: "View",
        items: &[MenuNode::act("Go to Start"), MenuNode::act("Go to End")],
    },
];

pub(crate) const EDITOR_BUTTONS: [(&str, &str, EditorCmd); 7] = [
    ("New", "Create an empty file", EditorCmd::New),
    ("Save", "SaveIndicator Source", EditorCmd::Save),
    ("Cut", "Cut selection", EditorCmd::Cut),
    ("Copy", "Copy selection", EditorCmd::Copy),
    ("Paste", "Paste", EditorCmd::Paste),
    ("Undo", "Undo", EditorCmd::Undo),
    ("Find", "Find Next", EditorCmd::Find),
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
            "Save" => EditorCmd::Save,
            "Restore Original Source" => EditorCmd::Reload,
            "Close Window" => EditorCmd::Close,
            "Undo" => EditorCmd::Undo,
            "Cut" => EditorCmd::Cut,
            "Copy" => EditorCmd::Copy,
            "Paste" => EditorCmd::Paste,
            "Select All" => EditorCmd::SelectAll,
            "Find Next" => EditorCmd::Find,
            "Go to Start" => EditorCmd::DocHome,
            "Go to End" => EditorCmd::DocEnd,
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
    events: &[kit::Event],
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
            kit::Event::Text(text) => {
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
            kit::Event::Paste(text) => {
                buf.insert_text(text, filter);
                *preedit = String::new();
                out.changed = true;
            }
            kit::Event::Ime(ime) => match ime {
                kit::ImeEvent::Preedit { text, .. } => {
                    *preedit = text.clone();
                }
                kit::ImeEvent::Commit(text) => {
                    buf.insert_text(text, filter);
                    preedit.clear();
                    out.changed = true;
                }
                #[cfg(test)]
                kit::ImeEvent::DeleteSurrounding {
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
            },
            kit::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => {
                let ctrl = modifiers.ctrl || modifiers.command;
                match key {
                    kit::Key::Backspace => {
                        buf.backspace();
                        out.changed = true;
                    }
                    kit::Key::Delete => {
                        buf.delete();
                        out.changed = true;
                    }
                    kit::Key::Enter => {
                        if multiline {
                            buf.insert_newline();
                            out.changed = true;
                        }
                    }
                    kit::Key::Tab => {
                        if !ctrl {
                            buf.insert_text(&" ".repeat(TAB_SPACES), filter);
                            out.changed = true;
                        }
                    }
                    kit::Key::ArrowLeft => {
                        if ctrl {
                            buf.move_home_or_end(false, modifiers.shift);
                        } else {
                            buf.move_horizontal(true, modifiers.shift);
                        }
                    }
                    kit::Key::ArrowRight => {
                        if ctrl {
                            buf.move_home_or_end(true, modifiers.shift);
                        } else {
                            buf.move_horizontal(false, modifiers.shift);
                        }
                    }
                    kit::Key::ArrowUp => buf.move_vertical_select(false, modifiers.shift),
                    kit::Key::ArrowDown => buf.move_vertical_select(true, modifiers.shift),
                    kit::Key::Home => {
                        if ctrl {
                            buf.move_doc_edge(false, modifiers.shift);
                        } else {
                            buf.move_home_or_end(false, modifiers.shift);
                        }
                    }
                    kit::Key::End => {
                        if ctrl {
                            buf.move_doc_edge(true, modifiers.shift);
                        } else {
                            buf.move_home_or_end(true, modifiers.shift);
                        }
                    }
                    kit::Key::A if ctrl => buf.select_all(),
                    kit::Key::C if ctrl => {
                        out.copy = Some(buf.selected_text());
                    }
                    kit::Key::X if ctrl => {
                        let text = buf.cut();
                        if !text.is_empty() {
                            out.changed = true;
                            out.copy = Some(text);
                        }
                    }
                    kit::Key::Z if ctrl => {
                        let changed = if modifiers.shift {
                            buf.redo()
                        } else {
                            buf.undo()
                        };
                        if changed {
                            out.changed = true;
                        }
                    }
                    kit::Key::Y if ctrl => {
                        if buf.redo() {
                            out.changed = true;
                        }
                    }
                    kit::Key::S if ctrl => out.save = true,
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

    pub(crate) scroll_grab: Option<kit::Vec2>,

    pub(crate) rows: usize,

    pub(crate) preedit: String,

    pub(crate) wheel_acc: f32,

    pub(crate) menu: MenuState,

    pub(crate) focused: bool,

    pub(crate) selecting: bool,

    pub(crate) drag_overflow: f32,

    keep_selection_on_release: bool,

    pub(crate) find_term: String,

    pub(crate) modified: bool,

    layout: TextLayoutCache,
}

#[derive(Clone, Default)]
struct TextLayoutCache {
    initialized: bool,
    advances: Vec<Vec<f32>>,
    runs: Vec<Vec<(String, SynKind)>>,
    line_numbers: Vec<String>,
    line_names: Vec<BTreeSet<String>>,
    name_counts: BTreeMap<String, usize>,
    user_names: BTreeSet<String>,
    #[cfg(test)]
    rebuilt_lines: usize,
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
            drag_overflow: 0.0,
            keep_selection_on_release: false,
            find_term: String::new(),
            modified: false,
            layout: TextLayoutCache::default(),
        }
    }

    pub(crate) fn load(&mut self, text: &str) {
        self.buf = TextBuffer::from_text(text);
        self.scroll = 0;
        self.preedit.clear();
        self.wheel_acc = 0.0;
        self.selecting = false;
        self.drag_overflow = 0.0;
        self.keep_selection_on_release = false;
        self.modified = false;
        self.layout = TextLayoutCache::default();
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
        self.ensure_layout();
    }

    fn ensure_layout(&mut self) {
        let change = self.buf.take_line_change();
        if !self.layout.initialized {
            self.rebuild_layout();
            return;
        }
        let Some(change) = change else {
            if self.layout.line_names.len() != self.buf.lines.len() {
                self.rebuild_layout();
            }
            return;
        };
        let (start, old_len, new_len) = match change {
            LineChange::Range {
                start,
                old_len,
                new_len,
            } => (start, old_len, new_len),
            LineChange::Suffix { start } => (
                start,
                self.layout.line_names.len().saturating_sub(start),
                self.buf.lines.len().saturating_sub(start),
            ),
        };
        let old_end = start.saturating_add(old_len);
        let new_end = start.saturating_add(new_len);
        if start > self.layout.line_names.len()
            || old_end > self.layout.line_names.len()
            || new_end > self.buf.lines.len()
            || self.layout.line_names.len() - old_len + new_len != self.buf.lines.len()
        {
            self.rebuild_layout();
            return;
        }

        let mut names_changed = false;
        let removed: Vec<BTreeSet<String>> = self.layout.line_names.drain(start..old_end).collect();
        for names in removed {
            for name in names {
                if let Some(count) = self.layout.name_counts.get_mut(&name) {
                    *count -= 1;
                    if *count == 0 {
                        self.layout.name_counts.remove(&name);
                        self.layout.user_names.remove(&name);
                        names_changed = true;
                    }
                }
            }
        }

        let inserted_names: Vec<BTreeSet<String>> = self.buf.lines[start..new_end]
            .iter()
            .map(|line| {
                let mut names = BTreeSet::new();
                collect_line_user_names(line, &mut names);
                names
            })
            .collect();
        for names in &inserted_names {
            for name in names {
                let count = self.layout.name_counts.entry(name.clone()).or_default();
                *count += 1;
                if *count == 1 {
                    self.layout.user_names.insert(name.clone());
                    names_changed = true;
                }
            }
        }
        self.layout.line_names.splice(start..start, inserted_names);

        let mut advances = Vec::with_capacity(new_len);
        let mut runs = Vec::with_capacity(new_len);
        for line in &self.buf.lines[start..new_end] {
            let mut widths = Vec::with_capacity(line.chars().count() + 1);
            widths.push(0.0);
            let mut advance = 0.0;
            for ch in line.chars() {
                advance += char_advance(ch);
                widths.push(advance);
            }
            advances.push(widths);
            runs.push(code_runs_with_names(line, &self.layout.user_names));
        }
        self.layout.advances.splice(start..old_end, advances);
        if names_changed {
            self.layout.runs = self
                .buf
                .lines
                .iter()
                .map(|line| code_runs_with_names(line, &self.layout.user_names))
                .collect();
        } else {
            self.layout.runs.splice(start..old_end, runs);
        }

        self.layout.line_numbers.truncate(self.buf.lines.len());
        self.layout.line_numbers.resize_with(self.buf.lines.len(), {
            let mut next = self.layout.line_numbers.len() + 1;
            move || {
                let value = next.to_string();
                next += 1;
                value
            }
        });
        for line in start..self.buf.lines.len() {
            self.layout.line_numbers[line] = (line + 1).to_string();
        }
        #[cfg(test)]
        {
            self.layout.rebuilt_lines = new_len;
        }
    }

    fn rebuild_layout(&mut self) {
        self.layout = TextLayoutCache::default();
        self.layout.initialized = true;
        self.layout.line_names.reserve(self.buf.lines.len());
        for line in &self.buf.lines {
            let mut names = BTreeSet::new();
            collect_line_user_names(line, &mut names);
            for name in &names {
                *self.layout.name_counts.entry(name.clone()).or_default() += 1;
            }
            self.layout.user_names.extend(names.iter().cloned());
            self.layout.line_names.push(names);
        }
        for (index, line) in self.buf.lines.iter().enumerate() {
            self.layout.line_numbers.push((index + 1).to_string());

            let mut widths = Vec::with_capacity(line.chars().count() + 1);
            widths.push(0.0);
            let mut advance = 0.0;
            for ch in line.chars() {
                advance += char_advance(ch);
                widths.push(advance);
            }
            self.layout.advances.push(widths);
            self.layout
                .runs
                .push(code_runs_with_names(line, &self.layout.user_names));
        }
        #[cfg(test)]
        {
            self.layout.rebuilt_lines = self.buf.lines.len();
        }
    }

    fn advance_at(&self, line: usize, column: usize) -> f32 {
        self.layout
            .advances
            .get(line)
            .and_then(|widths| widths.get(column))
            .copied()
            .unwrap_or(0.0)
    }

    pub(crate) fn ui(
        &mut self,
        app: &mut AppState,
        ui: &mut kit::Ui,
        panel: kit::Rect,
        id: kit::Id,
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
        let menu_open = self.menu.open.is_some();
        if menu_open {
            ui.scope(|ui| {
                ui.disable();
                self.ui_buttonbar(app, ui, bar_band, &mut cmd);
            });
        } else {
            self.ui_buttonbar(app, ui, bar_band, &mut cmd);
        }
        self.ui_body(app, ui, panel, id, menu_open, &mut cmd);

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

    fn paint_separators(&self, painter: &kit::Painter, panel: kit::Rect) {
        for rows in [SRC_TITLE_ROWS + SRC_MENU_ROWS, SRC_HEAD_ROWS] {
            let y = panel.min.y + grid_line(rows);
            hline_hard(painter, panel.min.x, panel.max.x, y, theme().editor_border);
        }
    }

    fn finish_cmd(&mut self, ui: &kit::Ui, cmd: EditorCmd) -> EditorCmd {
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

    pub(crate) fn apply_context_action(&mut self, ui: &kit::Ui, action: ContextAction) {
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
        app: &mut AppState,
        ui: &mut kit::Ui,
        band: kit::Rect,
        cmd: &mut EditorCmd,
    ) {
        let painter = ui.painter().clone();

        painter.rect_filled(
            kit::Rect::from_min_max(kit::pos2(band.min.x, band.min.y + LINE_W), band.max),
            kit::CornerRadius::ZERO,
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
                kit::Rect::from_min_size(kit::pos2(x, band.min.y), kit::vec2(w, band.height()));
            if app.draw_button(ui, &painter, rect, &content, false, tip) {
                *cmd = action;
            }
            x += w - LINE_W;
        }
    }

    fn pos_at(&self, p: kit::Pos2, code_x: f32, body_top: f32) -> (usize, usize) {
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
        app: &mut AppState,
        ui: &mut kit::Ui,
        panel: kit::Rect,
        id: kit::Id,
        menu_open: bool,
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
            kit::Rect::from_min_max(
                kit::pos2(panel.min.x + LINE_W, body_top),
                kit::pos2(panel.max.x - LINE_W, body_bottom),
            ),
            kit::CornerRadius::ZERO,
            theme().editor_bg,
        );
        painter.rect_filled(
            kit::Rect::from_min_max(
                kit::pos2(panel.min.x + LINE_W, body_top + LINE_W),
                kit::pos2(gutter_line, body_bottom),
            ),
            kit::CornerRadius::ZERO,
            theme().editor_gutter_bg,
        );
        vline_hard(
            &painter,
            gutter_line,
            body_top,
            body_bottom,
            theme().editor_border,
        );

        let hovered = !menu_open
            && !app.shell.win_silent
            && ui
                .input(|i| i.pointer.hover_pos())
                .is_some_and(|p| panel.contains(p));
        if hovered {
            let mut acc = self.wheel_acc;
            ui.input(|i| {
                for event in &i.events {
                    if let kit::Event::MouseWheel { unit, delta, .. } = event {
                        acc += match unit {
                            kit::MouseWheelUnit::Line => delta.y,
                            kit::MouseWheelUnit::Point => delta.y / WHEEL_POINTS_PER_NOTCH,
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

        let text_area = kit::Rect::from_min_max(
            kit::pos2(panel.min.x + LINE_W, body_top),
            kit::pos2(bar_line, body_bottom),
        );

        let window_active = !app.shell.win_silent;
        let resp = if menu_open {
            ui.interact(text_area, id, kit::Sense::hover())
        } else {
            ui.interact(text_area, id, kit::Sense::click_and_drag())
        };
        if ui
            .input(|i| i.pointer.hover_pos())
            .filter(|_| window_active && !menu_open)
            .is_some_and(|p| text_area.contains(p))
        {
            ui.ctx().set_cursor_icon(kit::CursorIcon::Text);
            app.pointer.text_hover = true;
            app.pointer.hint = CursorKind::Text;
        }
        let primary_pressed_in_body = resp.pressed() && !menu_open;
        let primary_clicked_in_body = resp.clicked() && !menu_open;
        if primary_pressed_in_body || resp.drag_started() {
            if primary_pressed_in_body {
                self.keep_selection_on_release = !self.focused && self.buf.selection().is_some();
            } else {
                self.keep_selection_on_release = false;
            }
            self.focused = true;

            ui.memory_mut(|m| m.request_focus(id));
        }

        let blur_press = ui.ctx().input(|i| {
            i.pointer.primary_pressed()
                && i.pointer.interact_pos().is_none_or(|p| !panel.contains(p))
        });
        if blur_press {
            self.focused = false;
            self.selecting = false;
            self.drag_overflow = 0.0;
            self.keep_selection_on_release = false;
            self.buf.sel_anchor = None;
        }

        if !self.focused {
            ui.memory_mut(|m| m.surrender_focus(id));
            self.selecting = false;
        } else if self.preedit.is_empty() && !ui.memory(|m| m.has_focus(id)) {
            ui.memory_mut(|m| m.request_focus(id));
        }
        let focused = self.focused;

        let (pointer_pos, press_pos, shift, double_clicked) = ui.input(|i| {
            let pointer_pos = app
                .pointer
                .live
                .or_else(|| i.pointer.interact_pos())
                .or_else(|| i.pointer.latest_pos());
            (
                pointer_pos,
                i.pointer.press_origin(),
                i.modifiers.shift,
                i.pointer.button_double_clicked(kit::PointerButton::Primary),
            )
        });
        let clamp_to_text = |p: kit::Pos2| {
            kit::pos2(
                p.x.clamp(text_area.min.x, text_area.max.x - LINE_W),
                p.y.clamp(text_area.min.y, text_area.max.y - LINE_W),
            )
        };
        if !menu_open && let Some(p) = pointer_pos {
            if resp.secondary_clicked() {
                self.focused = true;
                ui.memory_mut(|m| m.request_focus(id));
                let pos = self.pos_at(clamp_to_text(p), code_x, body_top);
                if !self.buf.selection_contains(pos) {
                    self.buf.set_cursor(pos, false);
                }
                app.shell.context_menu.open_editor(p);
            } else if resp.drag_started() {
                self.selecting = true;
                let start = self.pos_at(clamp_to_text(press_pos.unwrap_or(p)), code_x, body_top);
                self.buf.set_cursor(start, false);
                self.buf.sel_anchor = Some(start);
                let now = self.pos_at(clamp_to_text(p), code_x, body_top);
                self.buf.set_cursor(now, true);
                self.drag_overflow = selection_overflow(p.y, text_area);
                app.pointer.pinned = None;
                app.reset_caret_blink(ui.ctx());
            } else if self.selecting && ui.input(|i| i.pointer.primary_down()) {
                app.pointer.text_selecting = true;
                let pointer_moved = ui.input(|i| {
                    i.events.iter().any(|event| {
                        matches!(
                            event,
                            kit::Event::PointerMoved(_) | kit::Event::MouseMoved(_)
                        )
                    })
                });
                if pointer_moved {
                    self.drag_overflow = selection_overflow(p.y, text_area);
                }
                let rows = (self.drag_overflow.abs() / SRC_ROW_PITCH).ceil() as usize;
                if rows > 0 {
                    if self.drag_overflow < 0.0 {
                        self.scroll = self.scroll.saturating_sub(rows.min(4));
                    } else {
                        self.scroll = (self.scroll + rows.min(4)).min(self.max_scroll());
                    }
                }
                let pos = self.pos_at(clamp_to_text(p), code_x, body_top);
                self.buf.set_cursor(pos, true);
                app.pointer.pinned = None;
                app.reset_caret_blink(ui.ctx());
            } else if primary_clicked_in_body {
                let pos = self.pos_at(clamp_to_text(p), code_x, body_top);
                if double_clicked {
                    self.buf.select_word(pos);
                } else if shift {
                    let anchor = self.buf.sel_anchor.unwrap_or(self.buf.cursor);
                    self.buf.sel_anchor = Some(anchor);
                    self.buf.set_cursor(pos, true);
                } else if self.keep_selection_on_release || self.buf.selection_contains(pos) {
                    // Refocusing by clicking inside an existing selection must not collapse it.
                } else {
                    self.buf.set_cursor(pos, false);
                }
                self.keep_selection_on_release = false;
                app.reset_caret_blink(ui.ctx());
            }
        }
        if !ui.input(|i| i.pointer.primary_down()) {
            self.selecting = false;
            self.drag_overflow = 0.0;
        }

        if focused && window_active {
            app.note_caret();
            let caret_before = self.buf.cursor;
            let out = ui.input(|input| {
                self.buf
                    .begin_undo_group((input.time * 2.0).floor().max(0.0) as u64);
                apply_edit_events(
                    &mut self.buf,
                    &mut self.preedit,
                    &input.events,
                    true,
                    &char_allowed,
                )
            });
            if out.changed {
                self.modified = true;
            }
            let keyboard_input = ui.input(|i| {
                i.events.iter().any(|event| {
                    matches!(
                        event,
                        kit::Event::Key { pressed: true, .. }
                            | kit::Event::Text(_)
                            | kit::Event::Paste(_)
                            | kit::Event::Ime(_)
                    )
                })
            });
            if keyboard_input {
                app.reset_caret_blink(ui.ctx());
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
        self.ensure_layout();

        let text_painter = painter.with_clip_rect(kit::Rect::from_min_max(
            kit::pos2(code_x, body_top),
            kit::pos2(bar_line, body_bottom),
        ));

        if let Some((start, end)) = self.buf.selection() {
            for k in 0..visible {
                let idx = self.scroll + k;
                if idx < start.0 || idx > end.0 {
                    continue;
                }
                let columns = self.buf.line_chars(idx);
                let c0 = if idx == start.0 { start.1 } else { 0 }.min(columns);
                let c1 = if idx == end.0 { end.1 } else { columns }.min(columns);
                let x0 = code_x + self.advance_at(idx, c0);
                let x1 = code_x + self.advance_at(idx, c1);
                let x1 = x1.min(code_x + code_w);
                if x1 <= x0 {
                    continue;
                }
                painter.rect_filled(
                    kit::Rect::from_min_size(
                        kit::pos2(x0, body_top + k as f32 * SRC_ROW_PITCH + LINE_W),
                        kit::vec2(x1 - x0, SRC_LINE_H),
                    ),
                    kit::CornerRadius::ZERO,
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
                kit::pos2(text_right_x(gutter_line, panel.min.x), y),
                kit::Align2::RIGHT_CENTER,
                &self.layout.line_numbers[idx],
                theme().editor_line_number,
            );
        }
        for k in 0..visible {
            let idx = self.scroll + k;
            if idx >= self.buf.line_count() {
                break;
            }
            let y = body_top + k as f32 * SRC_ROW_PITCH + LINE_W + SRC_LINE_H / 2.0;
            paint_code_runs(
                app,
                &text_painter,
                code_x,
                y,
                code_w,
                &self.layout.runs[idx],
            );
        }

        let mut caret_rect: Option<kit::Rect> = None;
        if focused {
            let (cl, cc) = self.buf.cursor;
            if cl >= self.scroll && cl < self.scroll + visible {
                let k = cl - self.scroll;
                let top = body_top + k as f32 * SRC_ROW_PITCH + LINE_W;
                let mid = top + SRC_LINE_H / 2.0;
                let mut x = (code_x + self.advance_at(cl, cc)).round();
                if !self.preedit.is_empty() {
                    let pw = text_advance(&self.preedit);
                    app.text(
                        &text_painter,
                        kit::pos2(x, mid),
                        kit::Align2::LEFT_CENTER,
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
                if !self.preedit.is_empty() || app.caret_visible(ui.ctx()) {
                    text_painter.with_layer(kit::PaintLayer::Dynamic, || {
                        text_painter.rect_filled(
                            caret,
                            kit::CornerRadius::ZERO,
                            theme().editor_text,
                        );
                    });
                }
                caret_rect = Some(caret);
            }
        }

        if focused {
            if let Some(bar) = caret_rect {
                let to_global = ui
                    .ctx()
                    .layer_transform_to_global(ui.layer_id())
                    .unwrap_or_default();
                let area = kit::Rect::from_min_max(
                    kit::pos2(code_x, body_top),
                    kit::pos2(bar_line, body_bottom),
                );
                ui.output_mut(|o| {
                    o.ime = Some(kit::output::IMEOutput {
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
    }
}

fn selection_overflow(y: f32, text_area: kit::Rect) -> f32 {
    if y < text_area.min.y {
        y - text_area.min.y
    } else if y > text_area.max.y - LINE_W {
        y - (text_area.max.y - LINE_W)
    } else {
        0.0
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

#[cfg(test)]
pub(crate) fn code_user_names(lines: &[String]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in lines {
        collect_line_user_names(line, &mut names);
    }
    names
}

fn collect_line_user_names(line: &str, names: &mut BTreeSet<String>) {
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

pub(crate) fn syn_color(kind: SynKind) -> kit::Color32 {
    match kind {
        SynKind::Plain => theme().editor_text,
        SynKind::UserName => theme().syn_keyword,
        SynKind::Comment => theme().syn_comment,
    }
}

pub(crate) fn paint_code_runs(
    app: &mut AppState,
    painter: &kit::Painter,
    code_x: f32,
    y: f32,
    code_w: f32,
    runs: &[(String, SynKind)],
) {
    let end = code_x + code_w;
    let mut x = code_x;
    for (text, kind) in runs {
        if x >= end {
            break;
        }
        let color = syn_color(*kind);
        let w = text_advance(text);
        if x + w > end {
            let shown = fit_line(text, end - x);
            if !shown.is_empty() {
                app.text(
                    painter,
                    kit::pos2(x, y),
                    kit::Align2::LEFT_CENTER,
                    &shown,
                    color,
                );
            }
            break;
        }
        app.text(
            painter,
            kit::pos2(x, y),
            kit::Align2::LEFT_CENTER,
            text,
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

pub(crate) fn scrollbar_track(panel: kit::Rect) -> kit::Rect {
    let bar_line = panel.max.x - 2.0 * LINE_W - SRC_BAR_W;
    kit::Rect::from_min_max(
        kit::pos2(bar_line + LINE_W, panel.min.y + SRC_BODY_TOP),
        kit::pos2(panel.max.x - LINE_W, panel.max.y - LINE_W),
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "hard-edge primitive keeps geometry arguments explicit"
)]
pub(crate) fn vertical_scrollbar(
    app: &mut AppState,
    ui: &mut kit::Ui,
    painter: &kit::Painter,
    id: kit::Id,
    panel: kit::Rect,
    body_top: f32,
    body_bottom: f32,
    visible: usize,
    total: usize,
    max_scroll: usize,
    scroll: &mut usize,
    scroll_grab: &mut Option<kit::Vec2>,
) {
    let bar_line = panel.max.x - 2.0 * LINE_W - SRC_BAR_W;
    let track = kit::Rect::from_min_max(
        kit::pos2(bar_line + LINE_W, body_top),
        kit::pos2(panel.max.x - LINE_W, body_bottom),
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
        kit::Rect::from_min_size(
            kit::pos2(track.min.x, (body_top + travel * t).round()),
            kit::vec2(track.width(), handle_h),
        )
    };
    let handle = handle_at(*scroll);

    let bar = ui.interact(handle, id.with("vscroll"), kit::Sense::click_and_drag());
    let track_sense = if bar.is_pointer_button_down_on() {
        kit::Sense::hover()
    } else {
        kit::Sense::click_and_drag()
    };
    let track_resp = ui.interact(track, id.with("vscroll_track"), track_sense);
    if bar.hovered() || bar.dragged() || track_resp.hovered() {
        ui.ctx().set_cursor_icon(kit::CursorIcon::ResizeVertical);
        app.pointer.hint = CursorKind::Slider;
    }

    let (down, press, pointer_event) = ui.input(|i| {
        (
            i.pointer.primary_down(),
            i.pointer.press_origin(),
            i.pointer.interact_pos(),
        )
    });

    let pointer = app.pointer.live.or(pointer_event);
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
                kit::vec2(0.0, handle_h / 2.0)
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

    painter.rect_filled(track, kit::CornerRadius::ZERO, theme().scroll_track);
    painter.rect_filled(handle, kit::CornerRadius::ZERO, theme().scroll_thumb);
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

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn layout_rebuilds_only_the_changed_line() {
        let mut editor = TextEditor::new();
        editor.load("alpha\nbeta\ngamma");
        editor.ensure_layout();
        assert_eq!(editor.layout.rebuilt_lines, 3);

        editor.buf.set_cursor((1, 4), false);
        editor.buf.insert_char('!');
        editor.ensure_layout();
        assert_eq!(editor.layout.rebuilt_lines, 1);
        assert_eq!(editor.layout.line_numbers, ["1", "2", "3"]);
        assert_eq!(editor.buf.line(1), "beta!");
    }

    #[test]
    fn inserted_line_only_rebuilds_the_split_line() {
        let mut editor = TextEditor::new();
        editor.load("alpha\nbeta\ngamma");
        editor.ensure_layout();
        editor.buf.set_cursor((1, 2), false);
        editor.buf.insert_newline();
        editor.ensure_layout();
        assert_eq!(editor.layout.rebuilt_lines, 2);
        assert_eq!(editor.layout.line_numbers, ["1", "2", "3", "4"]);
    }
}
