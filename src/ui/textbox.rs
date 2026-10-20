use crate::app::*;
use crate::font::{char_advance, text_advance};
use crate::grid::*;
use crate::theme::*;
use crate::ui::editor::apply_edit_events;
use crate::ui::icon::*;
use crate::ui::textbuf::*;
use eframe::egui;

#[derive(Clone, Copy)]
pub(crate) struct TextInputStyle {
    pub(crate) bg: egui::Color32,

    pub(crate) text: egui::Color32,

    pub(crate) edge: Option<egui::Color32>,
}

#[derive(Clone)]
pub(crate) struct TextInput {
    pub(crate) buf: TextBuffer,

    pub(crate) preedit: String,

    pub(crate) max_chars: usize,
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
        }
    }

    pub(crate) fn set_text(&mut self, text: &str) {
        self.buf = TextBuffer::from_text(text);
        self.buf.move_doc_end();
        self.preedit.clear();
        self.join_lines();
        self.limit_chars(self.max_chars);
    }

    pub(crate) fn join_lines(&mut self) -> bool {
        if self.buf.lines.len() <= 1 {
            return false;
        }
        self.buf.lines = vec![self.buf.lines.join("")];
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
        self.buf.clamp();
        true
    }

    pub(crate) fn ui(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        id: egui::Id,
        filter: &dyn Fn(char) -> bool,
        style: TextInputStyle,
    ) -> bool {
        let painter = ui.painter().clone();
        let inner = rect.shrink(LINE_W);

        let text_x = text_left_x(rect.min.x, rect.min.x);
        let mid_y = inner.center().y;

        let resp = ui.interact(rect, id, egui::Sense::click());
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
            app.pointer.text_hover = true;
            app.pointer.hint = CursorKind::Text;
        }
        if resp.clicked() {
            ui.memory_mut(|m| m.request_focus(id));
        } else if ui.input(|i| i.pointer.primary_clicked()) {
            ui.memory_mut(|m| m.surrender_focus(id));
        }
        let focused = ui.memory(|m| m.has_focus(id));

        if focused && resp.clicked() {
            if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
                let col = col_at(self.buf.line(0), p.x - text_x);
                self.buf.set_cursor((0, col), false);
            }
        }

        let mut changed = false;
        if focused {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    id,
                    egui::EventFilter {
                        tab: true,
                        horizontal_arrows: true,
                        vertical_arrows: false,
                        escape: true,
                    },
                )
            });
            let events = ui.input(|i| i.events.clone());
            let out = apply_edit_events(&mut self.buf, &mut self.preedit, &events, false, filter);
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
        }
        if focused && (resp.clicked() || changed) {
            app.reset_caret_blink(ui.ctx());
        }

        painter.rect_filled(inner, egui::CornerRadius::ZERO, style.bg);
        if let Some(edge) = style.edge {
            border_hard(&painter, rect, edge);
        }
        let text_painter = painter.with_clip_rect(inner);
        let line = self.buf.line(0).to_owned();
        if let Some((start, end)) = self.buf.selection() {
            let x0 = text_x + text_advance(&line.chars().take(start.1).collect::<String>());
            let x1 = text_x + text_advance(&line.chars().take(end.1).collect::<String>());
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(x0, inner.min.y),
                    egui::vec2(x1 - x0, inner.height()),
                ),
                egui::CornerRadius::ZERO,
                theme().sel_bg,
            );
        }
        app.text(
            &text_painter,
            egui::pos2(text_x, mid_y),
            egui::Align2::LEFT_CENTER,
            &line,
            style.text,
        );

        let mut caret: Option<egui::Rect> = None;
        if focused {
            let head: String = line.chars().take(self.buf.cursor.1).collect();
            let mut x = (text_x + text_advance(&head)).round();
            if !self.preedit.is_empty() {
                let pw = text_advance(&self.preedit);
                app.text(
                    &text_painter,
                    egui::pos2(x, mid_y),
                    egui::Align2::LEFT_CENTER,
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
            if app.caret_visible(ui.ctx()) {
                text_painter.rect_filled(bar, egui::CornerRadius::ZERO, theme().input_caret);
            }
            caret = Some(bar);
        }

        if let Some(bar) = caret {
            let to_global = ui
                .ctx()
                .layer_transform_to_global(ui.layer_id())
                .unwrap_or_default();
            ui.output_mut(|o| {
                o.ime = Some(egui::output::IMEOutput {
                    purpose: egui::IMEPurpose::Normal,
                    rect: to_global * rect,
                    cursor_rect: to_global * bar,
                    should_interrupt_composition: false,
                });
            });
        }

        changed
    }
}
