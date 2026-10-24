use crate::application::app::*;
use crate::domain::indicator;
use crate::domain::layout::*;
use crate::engine::kit;
use crate::engine::theme::*;
use crate::ui::menu::*;

impl AppState {
    pub(crate) fn pane_menu_count(&self) -> usize {
        match self.shell.pane_menu_tab {
            0 | 1 => 1,
            _ => indicator::builtin_names().len(),
        }
    }

    pub(crate) fn pane_menu_rect(&self, count: usize, window_bottom: f32) -> Option<kit::Rect> {
        if count == 0 {
            return None;
        }
        Some(
            MenuPanel::left(
                self.shell.pane_menu_anchor,
                PANE_MENU_W,
                count,
                window_bottom,
            )
            .rect,
        )
    }

    pub(crate) fn set_pane_indicator(&mut self, pane: usize, index: usize) {
        let Some(mut ind) = indicator::builtin_indicator(index) else {
            return;
        };

        ind.set_palette(self.current_palette());

        let name = ind.name().to_owned();
        let script_source = indicator::script_source(&name);
        let script = crate::domain::script::ScriptProgram::parse(&script_source).ok();
        self.chart_world.panes[pane].indicator = ind;
        self.chart_world.panes[pane].dirty = true;
        self.chart_world.panes[pane].script = script;
        self.chart_world.panes[pane].script_source = script_source;
        self.chart_world.panes[pane].script_error = None;

        self.shell.status_msg = format!("Pane {} changed to: {}", pane + 1, name);

        if self.workbench.debug_windows.source_open && self.workbench.source.pane == pane {
            self.reload_source(pane);
        }
    }

    pub(crate) fn ui_pane_menu(&mut self, ui: &mut kit::Ui) {
        let Some(pane) = self.shell.pane_menu else {
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
        let tab = self.shell.pane_menu_tab;
        let clicked = menu.ui_rows(self, ui, &painter, |app, painter, j, row| {
            let label = match tab {
                0 => "Build RoundTable",
                1 => "Edit Chart Source",
                _ => names[j],
            };
            app.text(
                painter,
                kit::pos2(text_left_x(row.min.x, panel.min.x), row.center().y),
                kit::Align2::LEFT_CENTER,
                label,
                theme().menu_popup_text,
            );
        });

        if let Some(index) = clicked {
            match tab {
                1 => self.request_win(WinCmd::OpenSource(pane)),
                2 => self.set_pane_indicator(pane, index),
                _ => {}
            }
            self.shell.pane_menu = None;
            return;
        }

        let outside_click = ui.ctx().input(|i| {
            i.pointer.primary_pressed()
                && i.pointer.interact_pos().is_some_and(|p| {
                    !panel.expand(grid_line(1)).contains(p) && !self.shell.pane_menu_bar.contains(p)
                })
        });
        if outside_click {
            self.shell.pane_menu = None;
        }
    }
}
