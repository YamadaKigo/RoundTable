use crate::domain::layout::LINE_W;
use crate::engine::kit;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum FillPattern {
    Horizontal,
    #[cfg(test)]
    Vertical,
    #[cfg(test)]
    Dot,
}

pub(crate) const TITLE_FILL: FillPattern = FillPattern::Horizontal;

pub(crate) fn paint_fill(
    painter: &kit::Painter,
    rect: kit::Rect,
    pattern: FillPattern,
    color: kit::Color32,
) {
    if !rect.is_positive() {
        return;
    }
    let painter = painter.with_clip_rect(rect);
    #[cfg(test)]
    let x0 = rect.min.x.round() as i32;
    #[cfg(test)]
    let x1 = rect.max.x.round() as i32;
    let y0 = rect.min.y.round() as i32;
    let y1 = rect.max.y.round() as i32;
    let mut mesh = kit::Mesh::default();
    match pattern {
        FillPattern::Horizontal => {
            for y in (y0..y1).step_by(2) {
                mesh.add_colored_rect(
                    kit::Rect::from_min_size(
                        kit::pos2(rect.min.x, y as f32),
                        kit::vec2(rect.width(), LINE_W),
                    ),
                    color,
                );
            }
        }
        #[cfg(test)]
        FillPattern::Vertical => {
            for x in (x0..x1).step_by(2) {
                mesh.add_colored_rect(
                    kit::Rect::from_min_size(
                        kit::pos2(x as f32, rect.min.y),
                        kit::vec2(LINE_W, rect.height()),
                    ),
                    color,
                );
            }
        }
        #[cfg(test)]
        FillPattern::Dot => {
            for y in (y0..y1).step_by(2) {
                for x in (x0..x1).step_by(2) {
                    mesh.add_colored_rect(
                        kit::Rect::from_min_size(
                            kit::pos2(x as f32, y as f32),
                            kit::vec2(LINE_W, LINE_W),
                        ),
                        color,
                    );
                }
            }
        }
    }
    if !mesh.is_empty() {
        painter.add(kit::Shape::mesh(mesh));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fill_templates_emit_pixel_marks_inside_the_target() {
        let area = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(16.0, 16.0));
        for pattern in [
            FillPattern::Horizontal,
            FillPattern::Vertical,
            FillPattern::Dot,
        ] {
            let ctx = kit::Context::default();
            let raw = kit::RawInput {
                screen_rect: Some(area),
                ..Default::default()
            };
            let mut output = ctx.run_ui(raw, |ui| {
                paint_fill(ui.painter(), area, pattern, kit::Color32::RED);
            });
            assert!(!output.shapes.is_empty(), "{pattern:?} 没有画出底纹");
            output.textures_delta.clear();
        }
    }
}
