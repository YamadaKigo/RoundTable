use crate::app::*;
use crate::grid::*;
use crate::indicator;
use crate::theme::*;
use crate::ui::menu::*;
use eframe::egui;

impl StockApp {
    pub(crate) fn pane_menu_count(&self) -> usize {
        if self.pane_menu_tab == 0 {
            1
        } else {
            indicator::builtin_names().len()
        }
    }

    pub(crate) fn pane_menu_rect(&self, count: usize, window_bottom: f32) -> Option<egui::Rect> {
        if count == 0 {
            return None;
        }
        Some(MenuPanel::left(self.pane_menu_anchor, PANE_MENU_W, count, window_bottom).rect)
    }

    pub(crate) fn set_pane_indicator(&mut self, pane: usize, index: usize) {
        let Some(mut ind) = indicator::builtin_indicator(index) else {
            return;
        };

        ind.set_palette(self.current_palette());

        let name = ind.name().to_owned();
        let script_source = indicator::script_source(&name);
        let script = crate::script::ScriptProgram::parse(&script_source).ok();
        self.panes[pane].indicator = ind;
        self.panes[pane].dirty = true;
        self.panes[pane].script = script;
        self.panes[pane].script_source = script_source;
        self.panes[pane].script_error = None;

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
        let count = self.pane_menu_count();
        let Some(panel) = self.pane_menu_rect(count, ui.max_rect().max.y) else {
            return;
        };
        let painter = ui.painter().clone();
        let menu = MenuPanel {
            rect: panel,
            rows: count,
        };
        let round_table = self.pane_menu_tab == 0;
        let clicked = menu.ui_rows(self, ui, &painter, |app, painter, j, row| {
            let label = if round_table {
                "组建圆桌"
            } else {
                names[j]
            };
            app.text(
                painter,
                egui::pos2(text_left_x(row.min.x, panel.min.x), row.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                theme().menu_popup_text,
            );
        });

        if let Some(index) = clicked {
            if !round_table {
                self.set_pane_indicator(pane, index);
            }
            self.pane_menu = None;
            return;
        }

        let outside_click = ui.ctx().input(|i| {
            i.pointer.primary_clicked()
                && i.pointer.interact_pos().is_some_and(|p| {
                    !panel.expand(grid_line(1)).contains(p) && !self.pane_menu_bar.contains(p)
                })
        });
        if outside_click {
            self.pane_menu = None;
        }
    }
}
