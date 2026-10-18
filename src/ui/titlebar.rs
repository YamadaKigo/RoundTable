use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::menu::*;
use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum SysButton {
    Minimize,
    Maximize,
    Close,
}

impl SysButton {
    pub(crate) fn paint_symbol(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        symbol: egui::Color32,
        face: egui::Color32,
    ) {
        self.paint_symbol_max(painter, rect, symbol, face, false)
    }

    pub(crate) fn paint_symbol_max(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        symbol: egui::Color32,
        _face: egui::Color32,
        maximized: bool,
    ) {
        let rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x.round(), rect.min.y.round()),
            rect.size(),
        );

        let cx = rect.min.x + CELL_CENTER;
        let cy = rect.min.y + CELL_CENTER;

        match self {
            SysButton::Minimize => {
                let bar = egui::Rect::from_min_size(
                    egui::pos2(cx - GRID / 2.0, cy),
                    egui::vec2(GRID, LINE_W),
                );
                painter.rect_filled(bar, egui::CornerRadius::ZERO, symbol);
            }
            SysButton::Close => {
                paint_cross(painter, egui::pos2(cx, cy), 4, symbol);
            }
            SysButton::Maximize => {
                if maximized {
                    paint_restore_symbol(painter, egui::pos2(cx, cy), symbol);
                } else {
                    let square = egui::Rect::from_min_size(
                        egui::pos2(cx - 4.0, cy - 4.0),
                        egui::vec2(9.0, 9.0),
                    );
                    border_hard(painter, square, symbol);
                }
            }
        }
    }
}

pub(crate) fn paint_restore_symbol(
    painter: &egui::Painter,
    center: egui::Pos2,
    symbol: egui::Color32,
) {
    let cx = center.x.round() as i32;
    let cy = center.y.round() as i32;

    const R: i32 = 4;
    for dx in -R..=R {
        let dy = R - dx.abs();
        for y in [cy - dy, cy + dy] {
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2((cx + dx) as f32, y as f32),
                    egui::vec2(LINE_W, LINE_W),
                ),
                egui::CornerRadius::ZERO,
                symbol,
            );
        }
    }
}

pub(crate) fn close_key_face(hot: bool) -> egui::Color32 {
    if hot {
        theme().close_hover
    } else {
        theme().btn_face
    }
}

pub(crate) fn close_key_symbol(hot: bool) -> egui::Color32 {
    if hot {
        theme().text_white
    } else {
        theme().text
    }
}

pub(crate) fn title_text_slot(app: &mut StockApp, title: egui::Rect, text: &str) -> egui::Rect {
    let w = tab_width(app, text).min(title.width());
    egui::Rect::from_min_size(title.min, egui::vec2(w, title.height()))
}

pub(crate) fn paint_title_text_edge(painter: &egui::Painter, slot: egui::Rect) {
    vline_hard(
        painter,
        slot.max.x - LINE_W,
        slot.min.y + LINE_W,
        slot.max.y - LINE_W,
        theme().ctrl_border,
    );
}

pub(crate) fn sys_button_rects(bar: egui::Rect) -> [egui::Rect; 3] {
    let mut rects = [egui::Rect::NOTHING; 3];

    let mut right = bar.max.x;
    for i in (0..3).rev() {
        rects[i] = egui::Rect::from_min_size(
            egui::pos2(right - SYS_BTN_W, bar.min.y),
            egui::vec2(SYS_BTN_W, SYS_BTN_H),
        );

        right = rects[i].min.x + LINE_W;
    }
    rects
}

impl StockApp {
    pub(crate) fn ui_titlebar(&mut self, ui: &mut egui::Ui, bar: egui::Rect) {
        let ctx = ui.ctx().clone();
        let painter = ui.painter().clone();

        let name_slot =
            egui::Rect::from_min_size(bar.min, egui::vec2(tab_width(self, APP_NAME), bar.height()));
        self.text(
            &painter,
            name_slot.center(),
            egui::Align2::CENTER_CENTER,
            APP_NAME,
            theme().text,
        );

        let buttons = sys_button_rects(bar);

        let kinds = [SysButton::Minimize, SysButton::Close];
        let slots = [1usize, 2];
        for (i, kind) in kinds.iter().enumerate() {
            let rect = buttons[slots[i]];
            let resp = ui.interact(rect, egui::Id::new(("rt_sys_btn", i)), egui::Sense::click());
            let hovered = resp.hovered();

            let (face, symbol) = match kind {
                SysButton::Close => (close_key_face(hovered), close_key_symbol(hovered)),

                SysButton::Minimize | SysButton::Maximize => (
                    if hovered {
                        theme().btn_hover
                    } else {
                        theme().btn_face
                    },
                    theme().text,
                ),
            };

            painter.rect_filled(rect, egui::CornerRadius::ZERO, face);
            border_hard(&painter, rect, theme().ctrl_border);
            kind.paint_symbol(&painter, rect, symbol, face);

            if resp.clicked() {
                match kind {
                    SysButton::Minimize => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                    SysButton::Maximize => {}
                    SysButton::Close => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
        }
    }
}
