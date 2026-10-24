use crate::application::app::*;
use crate::domain::layout::*;
use crate::engine::font::{char_advance, text_advance};
use crate::engine::kit;
use crate::engine::theme::*;
use crate::ui::editor::apply_edit_events;
use crate::ui::icon::*;
use crate::ui::textbuf::*;

#[derive(Clone, Copy)]
pub(crate) struct TextInputStyle {
    pub(crate) bg: kit::Color32,

    pub(crate) text: kit::Color32,

    pub(crate) edge: Option<kit::Color32>,
}

#[derive(Clone)]
pub(crate) struct TextInput {
    pub(crate) buf: TextBuffer,

    pub(crate) preedit: String,

    pub(crate) max_chars: usize,

    selecting: bool,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new(0)
    }
}

pub(crate) fn col_at(line: &str, dx: f32) -> usize {
    let mut w = 0.0f32;
    for (i, ch) in line.chars().enumerate() {
        let cw = char_advance(ch);
        if dx < w + cw / 2.0 {
            return i;
        }
        w += cw;
    }
    line.chars().count()
}

impl TextInput {
    pub(crate) fn new(max_chars: usize) -> Self {
        let mut buf = TextBuffer::from_text("");
        buf.move_doc_end();
        Self {
            buf,
            preedit: String::new(),
            max_chars,
            selecting: false,
        }
    }

    pub(crate) fn set_text(&mut self, text: &str) {
        self.buf = TextBuffer::from_text(text);
        self.buf.move_doc_end();
        self.preedit.clear();
        self.selecting = false;
        self.join_lines();
        self.limit_chars(self.max_chars);
    }

    pub(crate) fn join_lines(&mut self) -> bool {
        if self.buf.lines.len() <= 1 {
            return false;
        }
        self.buf.lines = vec![self.buf.lines.join("")];
        self.buf.mark_all_dirty();
        self.buf.sel_anchor = None;
        self.buf.move_doc_end();
        true
    }

    pub(crate) fn text(&self) -> String {
        self.buf.to_text()
    }

    pub(crate) fn limit_chars(&mut self, max: usize) -> bool {
        if max == 0 {
            return false;
        }
        let total: usize = self.buf.lines.iter().map(|l| l.chars().count()).sum();
        if total <= max {
            return false;
        }
        let mut left = max;
        let mut kept: Vec<String> = Vec::new();
        for line in &self.buf.lines {
            if left == 0 {
                break;
            }
            let n = line.chars().count();
            if n <= left {
                left -= n;
                kept.push(line.clone());
            } else {
                kept.push(line.chars().take(left).collect());
                left = 0;
            }
        }
        if kept.is_empty() {
            kept.push(String::new());
        }
        self.buf.lines = kept;
        self.buf.mark_all_dirty();
        self.buf.clamp();
        true
    }

    pub(crate) fn ui(
        &mut self,
        app: &mut AppState,
        ui: &mut kit::Ui,
        rect: kit::Rect,
        id: kit::Id,
        filter: &dyn Fn(char) -> bool,
        style: TextInputStyle,
    ) -> bool {
        let painter = ui.painter().clone();
        let inner = rect.shrink(LINE_W);

        let text_x = text_left_x(rect.min.x, rect.min.x);
        let mid_y = inner.center().y;

        let resp = ui.interact(rect, id, kit::Sense::click_and_drag());
        if resp.hovered() {
            ui.ctx().set_cursor_icon(kit::CursorIcon::Text);
            app.pointer.text_hover = true;
            app.pointer.hint = CursorKind::Text;
        }
        let outside_press = ui.input(|i| {
            i.pointer.primary_pressed()
                && i.pointer
                    .interact_pos()
                    .or_else(|| i.pointer.latest_pos())
                    .is_some_and(|p| !rect.contains(p))
        });
        if resp.clicked() || resp.drag_started() || resp.pressed() {
            ui.memory_mut(|m| m.request_focus(id));
        } else if outside_press {
            ui.memory_mut(|m| m.surrender_focus(id));
        }
        let focused = ui.memory(|m| m.has_focus(id));
        if focused && !app.shell.win_silent {
            app.note_caret();
        }

        let (pointer, press, shift, double_clicked) = ui.input(|i| {
            (
                i.pointer.interact_pos().or_else(|| i.pointer.latest_pos()),
                i.pointer.press_origin(),
                i.modifiers.shift,
                i.pointer.button_double_clicked(kit::PointerButton::Primary),
            )
        });
        if focused {
            if resp.drag_started()
                && let Some(start) = press
            {
                app.pointer.text_selecting = true;
                let col = col_at(self.buf.line(0), start.x - text_x);
                self.buf.set_cursor((0, col), false);
                self.buf.sel_anchor = Some((0, col));
                self.selecting = true;
            }
            if self.selecting
                && ui.input(|i| i.pointer.primary_down())
                && let Some(p) = pointer
            {
                app.pointer.text_selecting = true;
                let x = p.x.clamp(inner.min.x, inner.max.x);
                let col = col_at(self.buf.line(0), x - text_x);
                self.buf.set_cursor((0, col), true);
                app.pointer.pinned = None;
            } else if resp.clicked()
                && let Some(p) = pointer
            {
                let col = col_at(self.buf.line(0), p.x - text_x);
                if double_clicked {
                    self.buf.select_word((0, col));
                } else if shift {
                    let anchor = self.buf.sel_anchor.unwrap_or(self.buf.cursor);
                    self.buf.sel_anchor = Some(anchor);
                    self.buf.set_cursor((0, col), true);
                } else {
                    self.buf.set_cursor((0, col), false);
                }
            }
        }
        if !ui.input(|i| i.pointer.primary_down()) {
            self.selecting = false;
        }

        let mut changed = false;
        if focused {
            let out = ui.input(|input| {
                self.buf
                    .begin_undo_group((input.time * 2.0).floor().max(0.0) as u64);
                apply_edit_events(
                    &mut self.buf,
                    &mut self.preedit,
                    &input.events,
                    false,
                    filter,
                )
            });
            changed = out.changed;
            if let Some(text) = out.copy {
                ui.ctx().copy_text(text);
            }
            if self.join_lines() {
                changed = true;
            }
            if self.limit_chars(self.max_chars) {
                changed = true;
            }
        } else {
            self.preedit.clear();
            self.selecting = false;
            self.buf.sel_anchor = None;
        }
        if focused && (resp.clicked() || resp.drag_started() || changed) {
            app.reset_caret_blink(ui.ctx());
        }

        painter.rect_filled(inner, kit::CornerRadius::ZERO, style.bg);
        if let Some(edge) = style.edge {
            border_hard(&painter, rect, edge);
        }
        let text_painter = painter.with_clip_rect(inner);
        let line = self.buf.line(0).to_owned();
        if let Some((start, end)) = self.buf.selection() {
            let x0 = text_x + line.chars().take(start.1).map(char_advance).sum::<f32>();
            let x1 = text_x + line.chars().take(end.1).map(char_advance).sum::<f32>();
            painter.rect_filled(
                kit::Rect::from_min_size(
                    kit::pos2(x0, inner.min.y),
                    kit::vec2(x1 - x0, inner.height()),
                ),
                kit::CornerRadius::ZERO,
                theme().sel_bg,
            );
        }
        app.text(
            &text_painter,
            kit::pos2(text_x, mid_y),
            kit::Align2::LEFT_CENTER,
            &line,
            style.text,
        );

        let mut caret: Option<kit::Rect> = None;
        if focused {
            let mut x = (text_x
                + line
                    .chars()
                    .take(self.buf.cursor.1)
                    .map(char_advance)
                    .sum::<f32>())
            .round();
            if !self.preedit.is_empty() {
                let pw = text_advance(&self.preedit);
                app.text(
                    &text_painter,
                    kit::pos2(x, mid_y),
                    kit::Align2::LEFT_CENTER,
                    &self.preedit,
                    theme().input_caret,
                );
                hline_hard(
                    &text_painter,
                    x,
                    x + pw,
                    inner.max.y - LINE_W,
                    theme().input_caret,
                );
                x += pw;
            }
            let bar = text_caret_rect(x, inner.min.y, inner.height());
            if !self.preedit.is_empty() || app.caret_visible(ui.ctx()) {
                text_painter.with_layer(kit::PaintLayer::Dynamic, || {
                    text_painter.rect_filled(bar, kit::CornerRadius::ZERO, theme().input_caret);
                });
            }
            caret = Some(bar);
        }

        if let Some(bar) = caret {
            let to_global = ui
                .ctx()
                .layer_transform_to_global(ui.layer_id())
                .unwrap_or_default();
            ui.output_mut(|o| {
                o.ime = Some(kit::output::IMEOutput {
                    rect: to_global * rect,
                    cursor_rect: to_global * bar,
                    should_interrupt_composition: false,
                });
            });
        }

        changed
    }
}
