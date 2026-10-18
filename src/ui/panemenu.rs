use crate::app::*;
use crate::grid::*;
use crate::indicator;
use crate::theme::*;
use crate::ui::menu::*;
use eframe::egui;

impl StockApp {
    pub(crate) fn pane_menu_rect(&self, count: usize, window_bottom: f32) -> Option<egui::Rect> {
        if count == 0 {
            return None;
        }
        Some(MenuPanel::right(self.pane_menu_anchor, PANE_MENU_W, count, window_bottom).rect)
    }

    pub(crate) fn set_pane_indicator(&mut self, pane: usize, index: usize) {
        let mut list = indicator::builtin();
        if index >= list.len() {
            return;
        }
        let mut ind = list.swap_remove(index);

        ind.set_palette(self.current_palette());

        if ind.supports_overlay() {
            ind.set_overlay_state(self.overlay);
        }
        let ov_ok = ind.supports_overlay();
        let name = ind.name().to_owned();
        self.panes[pane].indicator = ind;
        self.panes[pane].dirty = true;

        if !ov_ok {
            self.overlay_menu = None;
        }
        self.status_msg = format!("Pane{} 换成：{}", pane + 1, name);

        if self.win_mgr.source_open() && self.source.pane == pane {
            self.reload_source(pane);
        }
    }

    pub(crate) fn ui_pane_menu(&mut self, ui: &mut egui::Ui) {
        let Some(pane) = self.pane_menu else {
            return;
        };

        let names = indicator::builtin_names();
        let Some(panel) = self.pane_menu_rect(names.len(), ui.max_rect().max.y) else {
            return;
        };
        let painter = ui.painter().clone();
        let menu = MenuPanel {
            rect: panel,
            rows: names.len(),
        };
        let clicked = menu.ui_rows(self, ui, &painter, |app, painter, j, row| {
            app.text(
                painter,
                egui::pos2(text_left_x(row.min.x, panel.min.x), row.center().y),
                egui::Align2::LEFT_CENTER,
                &names[j],
                theme().text,
            );
        });

        if let Some(index) = clicked {
            self.set_pane_indicator(pane, index);
            self.pane_menu = None;
            return;
        }

        if pointer_left(ui, &[panel, self.pane_menu_bar]) {
            self.pane_menu = None;
        }
    }
}
