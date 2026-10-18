use crate::app::*;
use crate::grid::*;
use crate::platform::client_origin;
use crate::pointer::*;
use crate::theme::*;
use eframe::egui;

pub(crate) const POINTER_TRAIL_S: f64 = 1.20;
pub(crate) const POINTER_TRAIL_MAX: usize = 260;

pub(crate) const POINTER_TRAIL_HOT_S: f64 = 0.18;

impl StockApp {
    pub(crate) fn paint_pointer_debug(&mut self, ctx: &egui::Context, at: egui::Pos2) {
        if !self.pointer.show_trail {
            return;
        }
        let now = ctx.input(|i| i.time);
        let origin = client_origin(self.hwnd);
        let to_client = |screen: egui::Pos2| match origin {
            Some((ox, oy)) => egui::pos2(screen.x - ox as f32, screen.y - oy as f32),
            None => screen,
        };
        let samples: Vec<PointerSample> = self
            .pointer
            .trajectory()
            .iter()
            .rev()
            .take_while(|s| now - s.time <= POINTER_TRAIL_S)
            .take(POINTER_TRAIL_MAX)
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        let velocity = self.pointer.velocity();
        let acceleration = self.pointer.acceleration();
        let x = at.x.round() as i32;
        let y = at.y.round() as i32;
        let lines = [
            format!("x={x} y={y}"),
            format!(
                "v={:.0},{:.0} |v|={:.0} px/s",
                velocity.x,
                velocity.y,
                velocity.length()
            ),
            format!(
                "a={:.0},{:.0} |a|={:.0} px/s2",
                acceleration.x,
                acceleration.y,
                acceleration.length()
            ),
        ];
        let mut text_w = 0.0f32;
        for line in &lines {
            text_w = text_w.max(self.text_width(line));
        }

        let layer = egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("rt_pointer_debug"));
        let painter = ctx.layer_painter(layer);
        let t = theme();

        for pair in samples.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let color = if now - b.time <= POINTER_TRAIL_HOT_S {
                t.pointer_trail_hot
            } else {
                t.pointer_trail
            };
            painter.line_segment(
                [to_client(a.position), to_client(b.position)],
                egui::Stroke::new(LINE_W, color),
            );
        }
        if let Some(first) = samples.first() {
            let p = to_client(first.position);
            painter.rect_filled(
                egui::Rect::from_center_size(p, egui::vec2(3.0, 3.0)),
                egui::CornerRadius::ZERO,
                t.pointer_trail,
            );
        }

        painter.rect_filled(
            egui::Rect::from_center_size(at, egui::vec2(3.0, 3.0)),
            egui::CornerRadius::ZERO,
            t.pointer_trail_hot,
        );

        let pad = grid_line(1);
        let w = snap_cells_up(text_w + 2.0 * pad).max(cells(20));
        let h = cells(TEXT_ROWS * lines.len() as i32);
        let bounds = ctx.input(|i| i.content_rect());
        let mut min = at + egui::vec2(18.0, 18.0);
        if min.x + w > bounds.max.x - LINE_W {
            min.x = at.x - 18.0 - w;
        }
        if min.y + h > bounds.max.y - LINE_W {
            min.y = at.y - 18.0 - h;
        }
        let min_x = bounds.min.x + LINE_W;
        let min_y = bounds.min.y + LINE_W;
        min.x = min.x.clamp(min_x, (bounds.max.x - w).max(min_x));
        min.y = min.y.clamp(min_y, (bounds.max.y - h).max(min_y));
        let badge = egui::Rect::from_min_size(min, egui::vec2(w, h));
        painter.rect_filled(badge, egui::CornerRadius::ZERO, t.menu_bg);
        border_hard(&painter, badge, t.ctrl_border);
        for (i, line) in lines.iter().enumerate() {
            let row = egui::Rect::from_min_size(
                egui::pos2(badge.min.x, badge.min.y + grid_line(i as i32 * TEXT_ROWS)),
                egui::vec2(badge.width(), cells(TEXT_ROWS)),
            );
            self.text(
                &painter,
                egui::pos2(text_left_x(row.min.x, badge.min.x), row.center().y),
                egui::Align2::LEFT_CENTER,
                line,
                t.text,
            );
        }
    }
}
