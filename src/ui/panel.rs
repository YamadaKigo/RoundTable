use crate::application::app::*;
use crate::domain::layout::*;
use crate::engine::kit;
use crate::engine::theme::*;
use crate::ui::button::*;
use crate::ui::colorpanel::*;
use crate::ui::icon::*;

pub(crate) const CHART_PANE_W: f32 = cells(CHART_COLS_DEFAULT);

pub(crate) const PANEL_W: f32 = cells(PANEL_COLS);

pub(crate) const PANE_MIN_W: f32 = cells(PANE_MIN_COLS);

pub(crate) const SPLITTER_W: f32 = cells(SPLITTER_COLS);

pub(crate) const PANEL_TITLE: &str = "Panel";

pub(crate) const PANEL_BTN_COUNT: i32 = 10;

pub(crate) const PANEL_CHILD_ROWS: i32 = WIN_TITLE_ROWS + 1 + (PANEL_BTN_COUNT - 1) * 2 + 2 + 1;
pub(crate) const PANEL_CHILD_H: f32 = cells(PANEL_CHILD_ROWS);
const _: () = assert!(PANEL_BTN_COUNT == 10 && PANEL_CHILD_ROWS == 24);

pub(crate) const KLINE_ROWS: i32 = 50;
pub(crate) const VOL_ROWS: i32 = 21;
pub(crate) const IND_ROWS: i32 = 21;
pub(crate) const KLINE_H_DEFAULT: f32 = cells(KLINE_ROWS);
pub(crate) const VOL_H_DEFAULT: f32 = cells(VOL_ROWS);
pub(crate) const IND_H_DEFAULT: f32 = cells(IND_ROWS);

pub(crate) const HSPLIT_H: f32 = cells(1);
pub(crate) const HSPLIT_STEP: f32 = PITCH;

#[derive(Clone, Copy, PartialEq)]
#[cfg(test)]
pub(crate) enum LeftSplit {
    Volume,
    Indicator,
}

#[cfg(test)]
pub(crate) const LEFT_FIXED_H: f32 = 2.0 * HSPLIT_H - 4.0 * LINE_W;

const _: () = assert!(KLINE_ROWS == 50 && VOL_ROWS == 21 && IND_ROWS == 21);
const _: () = assert!((KLINE_H_DEFAULT as i32 - 1) % 9 == 0);
const _: () = assert!((VOL_H_DEFAULT as i32 - 1) % 9 == 0);
const _: () = assert!((IND_H_DEFAULT as i32 - 1) % 9 == 0);
const _: () = assert!(KLINE_ROWS + 1 + VOL_ROWS + 1 + IND_ROWS == WORK_ROWS);
const _: () = assert!((HSPLIT_H as i32 - 1) % 9 == 0 && HSPLIT_STEP as i32 % 9 == 0);

const _: () = assert!((CHART_PANE_W as i32 - 1) % 9 == 0);
const _: () = assert!((PANEL_W as i32 - 1) % 9 == 0);
const _: () = assert!((PANEL_CHILD_H as i32 - 1) % 9 == 0);
const _: () = assert!((PANE_MIN_W as i32 - 1) % 9 == 0);
const _: () = assert!((SPLITTER_W as i32 - 1) % 9 == 0);
const _: () = assert!((BTN_H as i32 - 1) % 9 == 0);
const _: () = assert!((BTN_MAX_W as i32 - 1) % 9 == 0);
const _: () = assert!((BTN_STEP as i32) % 9 == 0);
const _: () = assert!((ICON_W as i32) % 8 == 0);
const _: () = assert!((ICON_W as i32 + ICON_GAP as i32) % 9 == 0);

const _: () =
    assert!(CHART_PANE_W as i32 + SPLITTER_W as i32 + PANEL_W as i32 - 2 == WINDOW_W as i32);

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PanelAction {
    NewRandomWalk,

    OpenUpColor,

    OpenDownColor,

    TogglePointerTrail,

    ToggleAsciiBold,

    CycleFrameRate,

    PrintState,

    FollowSystemColors,

    AddChart,

    OpenConsole,
}

pub(crate) struct PanelButton {
    pub(crate) button: Button,
    pub(crate) action: PanelAction,
}

impl AppState {
    #[cfg(test)]
    pub(crate) fn work_columns(work: kit::Rect, split_x: f32) -> [kit::Rect; 3] {
        let x0 = work.min.x;
        let y0 = work.min.y;
        let y1 = work.max.y;

        let bar_x = x0 + split_x - LINE_W;
        let left = kit::Rect::from_min_max(kit::pos2(x0, y0), kit::pos2(bar_x + LINE_W, y1));
        let splitter =
            kit::Rect::from_min_max(kit::pos2(bar_x, y0), kit::pos2(bar_x + SPLITTER_W, y1));

        let panel = kit::Rect::from_min_max(
            kit::pos2(bar_x + SPLITTER_W - LINE_W, y0),
            kit::pos2(work.max.x, y1),
        );
        [left, splitter, panel]
    }

    #[cfg(test)]
    pub(crate) fn child_areas(work: kit::Rect) -> [kit::Rect; 2] {
        let x0 = work.min.x;
        let y0 = work.min.y;
        let cols = ((work.width() - LINE_W) / PITCH).floor() as i32;
        let bar_x = x0 + cells((cols - PANEL_COLS).max(PANE_MIN_COLS)) - LINE_W;
        let chart =
            kit::Rect::from_min_max(kit::pos2(x0, y0), kit::pos2(bar_x + LINE_W, work.max.y));
        let panel = kit::Rect::from_min_max(
            kit::pos2(bar_x, y0),
            kit::pos2(work.max.x, y0 + PANEL_CHILD_H),
        );
        [chart, panel]
    }

    pub(crate) fn panel_min_size(&self) -> kit::Vec2 {
        let w = self
            .shell
            .panel_buttons
            .iter()
            .fold(0.0f32, |acc, b| acc.max(b.button.w));
        kit::vec2(snap_cells_up(w + 2.0 * grid_line(1)), PANEL_CHILD_H)
    }

    #[cfg(test)]
    pub(crate) fn clamp_split(&mut self, work: kit::Rect) {
        self.chart_ui.split_x =
            snap_cells_up(self.chart_ui.split_x).clamp(PANE_MIN_W, Self::split_max_x(work));
    }

    #[cfg(test)]
    pub(crate) fn split_max_x(work: kit::Rect) -> f32 {
        let cols = ((work.width() - LINE_W) / PITCH).floor() as i32;
        cells((cols - SPLITTER_COLS - PANE_MIN_COLS).max(PANE_MIN_COLS))
    }

    #[cfg(test)]
    pub(crate) fn left_rows(work: kit::Rect, kline_h: f32, vol_h: f32) -> [kit::Rect; 5] {
        let x0 = work.min.x;
        let w = work.width();
        let kline = kit::Rect::from_min_size(kit::pos2(x0, work.min.y), kit::vec2(w, kline_h));
        let bar1 =
            kit::Rect::from_min_size(kit::pos2(x0, kline.max.y - LINE_W), kit::vec2(w, HSPLIT_H));
        let vol = kit::Rect::from_min_size(kit::pos2(x0, bar1.max.y - LINE_W), kit::vec2(w, vol_h));
        let bar2 =
            kit::Rect::from_min_size(kit::pos2(x0, vol.max.y - LINE_W), kit::vec2(w, HSPLIT_H));
        let ind = kit::Rect::from_min_max(
            kit::pos2(x0, bar2.max.y - LINE_W),
            kit::pos2(x0 + w, work.max.y),
        );
        [kline, bar1, vol, bar2, ind]
    }

    #[cfg(test)]
    pub(crate) fn clamp_left_heights(&mut self, work: kit::Rect) {
        let budget = (work.height() - LEFT_FIXED_H).max(3.0 * PANE_H_MIN);
        let min = PANE_H_MIN;
        let ind_max = (budget - 2.0 * min).max(min);
        self.chart_ui.ind_h = snap_cells_up(self.chart_ui.ind_h).clamp(min, ind_max);
        let vol_max = (budget - min - self.chart_ui.ind_h).max(min);
        self.chart_ui.vol_h = snap_cells_up(self.chart_ui.vol_h).clamp(min, vol_max);
        self.chart_ui.kline_h = budget - self.chart_ui.vol_h - self.chart_ui.ind_h;
    }

    #[cfg(test)]
    pub(crate) fn ui_hsplitter(
        &mut self,
        ui: &mut kit::Ui,
        bar: kit::Rect,
        work: kit::Rect,
        which: LeftSplit,
    ) {
        let chart_dragging = self
            .chart_world
            .panes
            .iter()
            .any(|pane| pane.view.pan.is_some());
        let resp = ui.allocate_rect(
            bar,
            if chart_dragging {
                kit::Sense::hover()
            } else {
                kit::Sense::drag()
            },
        );
        if !chart_dragging && (resp.hovered() || resp.dragged()) {
            ui.ctx().set_cursor_icon(kit::CursorIcon::ResizeVertical);

            self.pointer.hint = CursorKind::ResizeV;
        }
        if resp.drag_started() {
            let grab = ui
                .input(|i| i.pointer.latest_pos())
                .map(|p| p.y - bar.min.y);
            match which {
                LeftSplit::Volume => self.chart_ui.hsplit_grab = grab,
                LeftSplit::Indicator => self.chart_ui.vsplit_grab = grab,
            }
        }
        if resp.dragged() {
            let grab = match which {
                LeftSplit::Volume => self.chart_ui.hsplit_grab,
                LeftSplit::Indicator => self.chart_ui.vsplit_grab,
            };
            if let (Some(grab), Some(pointer)) = (grab, ui.input(|i| i.pointer.latest_pos())) {
                let want = match which {
                    LeftSplit::Volume => self.chart_ui.vol_h - (pointer.y - grab - bar.min.y),
                    LeftSplit::Indicator => self.chart_ui.ind_h - (pointer.y - grab - bar.min.y),
                };
                let want = snap_cells_round(want);
                match which {
                    LeftSplit::Volume => self.chart_ui.vol_h = want,
                    LeftSplit::Indicator => self.chart_ui.ind_h = want,
                }
                self.clamp_left_heights(work);
            }
        }
        if resp.drag_stopped() {
            match which {
                LeftSplit::Volume => self.chart_ui.hsplit_grab = None,
                LeftSplit::Indicator => self.chart_ui.vsplit_grab = None,
            }
        }

        let painter = ui.painter().clone();
        painter.rect_filled(bar, kit::CornerRadius::ZERO, theme().splitter);
        hline_hard(
            &painter,
            bar.min.x,
            bar.max.x,
            bar.min.y,
            theme().panel_border,
        );
        hline_hard(
            &painter,
            bar.min.x,
            bar.max.x,
            bar.max.y - LINE_W,
            theme().panel_border,
        );
    }

    pub(crate) fn ui_panel(&mut self, ui: &mut kit::Ui, body: kit::Rect) {
        let painter = ui.painter().clone();

        painter.rect_filled(body, kit::CornerRadius::ZERO, theme().panel_bg);

        let mut rects = Vec::with_capacity(self.shell.panel_buttons.len());
        let x = body.min.x + grid_line(1);
        let mut y = body.min.y + grid_line(1);
        for button in &self.shell.panel_buttons {
            rects.push(kit::Rect::from_min_size(
                kit::pos2(x, y),
                kit::vec2(button.button.w, BTN_H),
            ));
            y += BTN_STEP;
        }

        let pointer = (!self.shell.win_silent)
            .then(|| ui.input(|i| i.pointer.hover_pos()))
            .flatten();
        let down = !self.shell.win_silent && ui.input(|i| i.pointer.primary_down());
        let hot = rects
            .iter()
            .position(|r| pointer.is_some_and(|p| r.contains(p)));
        self.shell.panel_press = if down { hot } else { None };

        for (i, rect) in rects.iter().enumerate() {
            let content = self.shell.panel_buttons[i].button.content.clone();
            let tip = self.shell.panel_buttons[i].button.tip.clone();
            let action = self.shell.panel_buttons[i].action;
            let pressed = self.shell.panel_press == Some(i);

            if self.draw_button(ui, &painter, *rect, &content, pressed, &tip) {
                self.run_action(action);
            }
        }
    }

    pub(crate) fn build_panel_buttons(&mut self) {
        let frame_rate = self.stats.frame_rate_cap.label();
        let items: [(ButtonContent, &str, PanelAction); PANEL_BTN_COUNT as usize] = [
            (
                ButtonContent::Text("Regenerate".to_owned()),
                "Generate new market data with another seed",
                PanelAction::NewRandomWalk,
            ),
            (
                ButtonContent::Text("Up Color".to_owned()),
                "Open the palette and change the rising candle color",
                PanelAction::OpenUpColor,
            ),
            (
                ButtonContent::Text("Down Color".to_owned()),
                "Open the palette and change the falling candle color",
                PanelAction::OpenDownColor,
            ),
            (
                ButtonContent::Text("Pointer Trail".to_owned()),
                "Toggle the pointer debug overlay: recent path, speed, acceleration, and coordinates",
                PanelAction::TogglePointerTrail,
            ),
            (
                ButtonContent::Text(self.ascii_bold_button_label()),
                "Toggle ASCII font weight: regular / bold",
                PanelAction::ToggleAsciiBold,
            ),
            (
                ButtonContent::Text(frame_rate),
                "Cycle the frame-rate cap: 60 / 120 / 240 / Unlimited",
                PanelAction::CycleFrameRate,
            ),
            (
                ButtonContent::Text("Follow Theme".to_owned()),
                "Make candle colors follow the active theme",
                PanelAction::FollowSystemColors,
            ),
            (
                ButtonContent::Text("New Chart".to_owned()),
                "Open a chart window from the first free slot",
                PanelAction::AddChart,
            ),
            (
                ButtonContent::Text("Debug Console".to_owned()),
                "Open the debug console with status, logs, and command input",
                PanelAction::OpenConsole,
            ),
            (
                ButtonContent::Text("Print State".to_owned()),
                "Print chart, window, and theme state to the status bar",
                PanelAction::PrintState,
            ),
        ];
        self.shell.panel_buttons = items
            .into_iter()
            .map(|(content, tip, action)| {
                let mut button = Button::new(content).with_tip(tip);
                button.measure(self);
                PanelButton { button, action }
            })
            .collect();
    }

    pub(crate) fn run_action(&mut self, action: PanelAction) {
        match action {
            PanelAction::NewRandomWalk => {
                self.regenerate_random_walk();
            }

            PanelAction::OpenUpColor => self.request_win(WinCmd::OpenColor(ColorTarget::Up)),
            PanelAction::OpenDownColor => self.request_win(WinCmd::OpenColor(ColorTarget::Down)),
            PanelAction::TogglePointerTrail => {
                self.pointer.show_trail = !self.pointer.show_trail;
                self.shell.status_msg = if self.pointer.show_trail {
                    "Pointer trail: On (path, speed, acceleration, coordinates)".to_owned()
                } else {
                    "Pointer Trail: Off".to_owned()
                };
            }
            PanelAction::ToggleAsciiBold => {
                let bold = self
                    .assets
                    .text_engine
                    .as_ref()
                    .is_some_and(|text| text.ascii_bold);
                let next = !bold;
                if let Some(text) = &mut self.assets.text_engine {
                    text.set_ascii_bold(next);
                }
                self.shell.status_msg = if next {
                    "ASCII Weight: Bold".to_owned()
                } else {
                    "ASCII Weight: Regular".to_owned()
                };
                self.sync_ascii_bold_button();
                self.stats.repaint_requested = true;
            }
            PanelAction::CycleFrameRate => {
                self.stats.frame_rate_cap = self.stats.frame_rate_cap.next();
                self.stats.fps = self
                    .stats
                    .frame_rate_cap
                    .fps()
                    .map_or(0.0, |fps| fps as f32);
                self.shell.status_msg = match self.stats.frame_rate_cap.fps() {
                    Some(fps) => format!("Frame cap: {fps} FPS"),
                    None => "Frame cap: Unlimited".to_owned(),
                };
                self.sync_frame_rate_button();
            }
            PanelAction::FollowSystemColors => self.follow_system_chart_colors(),

            PanelAction::AddChart => self.request_win(WinCmd::AddChart),
            PanelAction::OpenConsole => self.request_win(WinCmd::OpenConsole),
            PanelAction::PrintState => {
                let view = self
                    .active_chart_pane()
                    .map(|pane| &self.chart_world.panes[pane].view);
                self.shell.status_msg = format!(
                    "Charts: {}, windows: {}, theme: {}, zoom: {}, pan: {}",
                    self.workbench.debug_windows.chart_count,
                    self.workbench.debug_windows.window_count,
                    theme().name(),
                    view.map_or(0, |view| view.scale),
                    view.map_or(0, |view| view.offset)
                );
            }
        }
    }

    fn sync_frame_rate_button(&mut self) {
        let content = ButtonContent::Text(self.stats.frame_rate_cap.label());
        let width = self.button_width(&content);
        if let Some(button) = self
            .shell
            .panel_buttons
            .iter_mut()
            .find(|button| button.action == PanelAction::CycleFrameRate)
        {
            button.button.content = content;
            button.button.w = width;
        }
    }

    pub(crate) fn regenerate_random_walk(&mut self) {
        self.chart_world.bar_seed = self
            .chart_world
            .bar_seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.chart_world.regenerate_market();
        for pane in &mut self.chart_world.panes {
            pane.view.offset = 0;
            pane.view.scale = BAR_SCALE_DEFAULT;
        }
        self.chart_world.clear_navigation();
        self.shell.status_msg = format!(
            "Generated new random market data (seed {})",
            self.chart_world.bar_seed
        );
    }

    fn ascii_bold_button_label(&self) -> String {
        if self
            .assets
            .text_engine
            .as_ref()
            .is_some_and(|text| text.ascii_bold)
        {
            "ASCII Regular".to_owned()
        } else {
            "ASCII Bold".to_owned()
        }
    }

    fn sync_ascii_bold_button(&mut self) {
        let content = ButtonContent::Text(self.ascii_bold_button_label());
        let width = self.button_width(&content);
        if let Some(button) = self
            .shell
            .panel_buttons
            .iter_mut()
            .find(|button| button.action == PanelAction::ToggleAsciiBold)
        {
            button.button.content = content;
            button.button.w = width;
        }
    }
}

impl StockApp {
    pub(crate) fn build_panel_buttons(&mut self) {
        self.state.build_panel_buttons();
        self.update_panel_min_size();
    }
}
