use crate::app::*;
use crate::grid::*;
use crate::indicator;
use crate::theme::*;
use crate::ui::menu::*;
use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OverlayKind {
    Ma,
    Ema,
    Boll,
}

pub(crate) struct OverlayItem {
    pub(crate) kind: OverlayKind,
    pub(crate) index: usize,
    pub(crate) label: String,
}

impl StockApp {
    pub(crate) fn overlay_items() -> Vec<OverlayItem> {
        let mut items = Vec::with_capacity(13);
        for (i, p) in indicator::MA_PERIODS.iter().enumerate() {
            items.push(OverlayItem {
                kind: OverlayKind::Ma,
                index: i,
                label: format!("MA{p}"),
            });
        }
        for (i, p) in indicator::EMA_PERIODS.iter().enumerate() {
            items.push(OverlayItem {
                kind: OverlayKind::Ema,
                index: i,
                label: format!("EMA{p}"),
            });
        }
        items.push(OverlayItem {
            kind: OverlayKind::Boll,
            index: 0,
            label: "BOLL(20,2)".to_owned(),
        });
        items
    }

    pub(crate) fn overlay_menu_rect(&self, count: usize, window_bottom: f32) -> Option<egui::Rect> {
        if count == 0 {
            return None;
        }
        Some(
            MenuPanel::right(
                self.overlay_menu_anchor,
                OVERLAY_MENU_W,
                count,
                window_bottom,
            )
            .rect,
        )
    }

    pub(crate) fn sync_overlay(&mut self) {
        let ov = self.overlay;
        for pane in &mut self.panes {
            if pane.indicator.supports_overlay() {
                pane.indicator.set_overlay_state(ov);
                pane.dirty = true;
            }
        }
        let (ma, ema) = (ov.ma.count_ones(), ov.ema.count_ones());
        self.status_msg = format!(
            "叠加：MA {ma} 条 / EMA {ema} 条 / BOLL {}",
            if ov.boll { "开" } else { "关" }
        );
    }

    pub(crate) fn ui_overlay_menu(&mut self, ui: &mut egui::Ui) {
        let Some(pane) = self.overlay_menu else {
            return;
        };
        let items = Self::overlay_items();
        let Some(panel) = self.overlay_menu_rect(items.len(), ui.max_rect().max.y) else {
            return;
        };
        let painter = ui.painter().clone();
        let menu = MenuPanel {
            rect: panel,
            rows: items.len(),
        };

        let mut ov = self.panes[pane].indicator.overlay_state();
        let t = theme();
        let clicked = menu.ui_rows(self, ui, &painter, |app, painter, j, row| {
            let item = &items[j];

            let on = match item.kind {
                OverlayKind::Ma => ov.ma_on(item.index),
                OverlayKind::Ema => ov.ema_on(item.index),
                OverlayKind::Boll => ov.boll,
            };
            let line_color = match item.kind {
                OverlayKind::Ma => t.ma[item.index],
                OverlayKind::Ema => t.ema[item.index],
                OverlayKind::Boll => t.boll_mid,
            };
            let mark = egui::Rect::from_min_size(
                egui::pos2(row.min.x + LINE_W, row.center().y - GRID / 2.0),
                egui::vec2(GRID, GRID),
            );
            if on {
                painter.rect_filled(mark, egui::CornerRadius::ZERO, line_color);
            } else {
                border_hard(painter, mark, line_color);
            }

            app.text(
                painter,
                egui::pos2(text_left_x(row.min.x, panel.min.x) + PITCH, row.center().y),
                egui::Align2::LEFT_CENTER,
                &item.label,
                theme().text,
            );
        });

        if let Some(j) = clicked {
            let item = &items[j];
            match item.kind {
                OverlayKind::Ma => ov.toggle_ma(item.index),
                OverlayKind::Ema => ov.toggle_ema(item.index),
                OverlayKind::Boll => ov.toggle_boll(),
            }

            self.overlay = ov;
            self.sync_overlay();
            return;
        }

        if pointer_left(ui, &[panel, self.overlay_menu_bar]) {
            self.overlay_menu = None;
        }
    }
}
