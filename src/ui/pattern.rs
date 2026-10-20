use crate::grid::LINE_W;
use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub(crate) enum FillPattern {
    Horizontal,
    Vertical,
    Dot,
}

pub(crate) const TITLE_FILL: FillPattern = FillPattern::Horizontal;

pub(crate) fn paint_fill(
    painter: &egui::Painter,
    rect: egui::Rect,
    pattern: FillPattern,
    color: egui::Color32,
) {
    if !rect.is_positive() {
        return;
    }
    let painter = painter.with_clip_rect(rect);
    let x0 = rect.min.x.round() as i32;
    let x1 = rect.max.x.round() as i32;
    let y0 = rect.min.y.round() as i32;
    let y1 = rect.max.y.round() as i32;
    match pattern {
        FillPattern::Horizontal => {
            for y in (y0..y1).step_by(2) {
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(rect.min.x, y as f32),
                        egui::vec2(rect.width(), LINE_W),
                    ),
                    egui::CornerRadius::ZERO,
                    color,
                );
            }
        }
        FillPattern::Vertical => {
            for x in (x0..x1).step_by(2) {
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(x as f32, rect.min.y),
                        egui::vec2(LINE_W, rect.height()),
                    ),
                    egui::CornerRadius::ZERO,
                    color,
                );
            }
        }
        FillPattern::Dot => {
            for y in (y0..y1).step_by(2) {
                for x in (x0..x1).step_by(2) {
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(x as f32, y as f32),
                            egui::vec2(LINE_W, LINE_W),
                        ),
                        egui::CornerRadius::ZERO,
                        color,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fill_templates_emit_pixel_marks_inside_the_target() {
        let area = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(16.0, 16.0));
        for pattern in [
            FillPattern::Horizontal,
            FillPattern::Vertical,
            FillPattern::Dot,
        ] {
            let ctx = egui::Context::default();
            let raw = egui::RawInput {
                screen_rect: Some(area),
                ..Default::default()
            };
            let mut output = ctx.run_ui(raw, |ui| {
                paint_fill(ui.painter(), area, pattern, egui::Color32::RED);
            });
            assert!(!output.shapes.is_empty(), "{pattern:?} 没有画出底纹");
            output.textures_delta.clear();
        }
    }
}
