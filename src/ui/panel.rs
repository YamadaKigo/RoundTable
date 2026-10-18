use crate::app::*;
use crate::grid::*;
use crate::pointer::*;
use crate::theme::*;
use crate::ui::button::*;
use crate::ui::colorpanel::*;
use crate::ui::icon::*;
use crate::ui::winmgr::*;
use eframe::egui;

pub(crate) const CHART_PANE_W: f32 = cells(CHART_COLS_DEFAULT);

pub(crate) const PANEL_W: f32 = cells(PANEL_COLS);

pub(crate) const PANE_MIN_W: f32 = cells(PANE_MIN_COLS);

pub(crate) const SPLITTER_W: f32 = cells(SPLITTER_COLS);

pub(crate) const PANEL_TITLE: &str = "控制面板";

pub(crate) const PANEL_BTN_COUNT: i32 = 18;

pub(crate) const PANEL_CHILD_ROWS: i32 = WIN_TITLE_ROWS + 1 + (PANEL_BTN_COUNT - 1) * 2 + 2 + 1;
pub(crate) const PANEL_CHILD_H: f32 = cells(PANEL_CHILD_ROWS);
const _: () = assert!(PANEL_BTN_COUNT == 18 && PANEL_CHILD_ROWS == 40);

pub(crate) const KLINE_ROWS: i32 = 52;
pub(crate) const VOL_ROWS: i32 = 21;
pub(crate) const IND_ROWS: i32 = 21;
pub(crate) const KLINE_H_DEFAULT: f32 = cells(KLINE_ROWS);
pub(crate) const VOL_H_DEFAULT: f32 = cells(VOL_ROWS);
pub(crate) const IND_H_DEFAULT: f32 = cells(IND_ROWS);

pub(crate) const HSPLIT_H: f32 = cells(1);
pub(crate) const HSPLIT_STEP: f32 = PITCH;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum LeftSplit {
    Volume,
    Indicator,
}

pub(crate) const LEFT_FIXED_H: f32 = 2.0 * HSPLIT_H - 4.0 * LINE_W;

const _: () = assert!(KLINE_ROWS == 52 && VOL_ROWS == 21 && IND_ROWS == 21);
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

    PointerSpeedUp,
    PointerSpeedDown,

    PointerAccelUp,
    PointerAccelDown,

    TogglePointerMode,

    TogglePointerTrail,

    ZoomIn,
    ZoomOut,

    ResetView,

    PrintState,

    ToggleGrid,

    ToggleTheme,

    FollowSystemColors,

    AddChart,

    OpenConsole,
}

pub(crate) struct PanelButton {
    pub(crate) button: Button,
    pub(crate) action: PanelAction,
}

impl StockApp {
    #[allow(dead_code)]
    pub(crate) fn work_columns(work: egui::Rect, split_x: f32) -> [egui::Rect; 3] {
        let x0 = work.min.x;
        let y0 = work.min.y;
        let y1 = work.max.y;

        let bar_x = x0 + split_x - LINE_W;
        let left = egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(bar_x + LINE_W, y1));
        let splitter =
            egui::Rect::from_min_max(egui::pos2(bar_x, y0), egui::pos2(bar_x + SPLITTER_W, y1));

        let panel = egui::Rect::from_min_max(
            egui::pos2(bar_x + SPLITTER_W - LINE_W, y0),
            egui::pos2(work.max.x, y1),
        );
        [left, splitter, panel]
    }

    pub(crate) fn child_areas(work: egui::Rect) -> [egui::Rect; 2] {
        let x0 = work.min.x;
        let y0 = work.min.y;
        let cols = ((work.width() - LINE_W) / PITCH).floor() as i32;
        let bar_x = x0 + cells((cols - PANEL_COLS).max(PANE_MIN_COLS)) - LINE_W;
        let chart =
            egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(bar_x + LINE_W, work.max.y));
        let panel = egui::Rect::from_min_max(
            egui::pos2(bar_x, y0),
            egui::pos2(work.max.x, y0 + PANEL_CHILD_H),
        );
        [chart, panel]
    }

    pub(crate) fn panel_min_size(&self) -> egui::Vec2 {
        let w = self
            .panel_buttons
            .iter()
            .fold(0.0f32, |acc, b| acc.max(b.button.w));
        egui::vec2(snap_cells_up(w + 2.0 * grid_line(1)), PANEL_CHILD_H)
    }

    #[allow(dead_code)]
    pub(crate) fn clamp_split(&mut self, work: egui::Rect) {
        self.split_x = snap_cells_up(self.split_x).clamp(PANE_MIN_W, Self::split_max_x(work));
    }

    #[allow(dead_code)]
    pub(crate) fn split_max_x(work: egui::Rect) -> f32 {
        let cols = ((work.width() - LINE_W) / PITCH).floor() as i32;
        cells((cols - SPLITTER_COLS - PANE_MIN_COLS).max(PANE_MIN_COLS))
    }

    #[allow(dead_code)]
    pub(crate) fn left_rows(work: egui::Rect, kline_h: f32, vol_h: f32) -> [egui::Rect; 5] {
        let x0 = work.min.x;
        let w = work.width();
        let kline = egui::Rect::from_min_size(egui::pos2(x0, work.min.y), egui::vec2(w, kline_h));
        let bar1 = egui::Rect::from_min_size(
            egui::pos2(x0, kline.max.y - LINE_W),
            egui::vec2(w, HSPLIT_H),
        );
        let vol =
            egui::Rect::from_min_size(egui::pos2(x0, bar1.max.y - LINE_W), egui::vec2(w, vol_h));
        let bar2 =
            egui::Rect::from_min_size(egui::pos2(x0, vol.max.y - LINE_W), egui::vec2(w, HSPLIT_H));
        let ind = egui::Rect::from_min_max(
            egui::pos2(x0, bar2.max.y - LINE_W),
            egui::pos2(x0 + w, work.max.y),
        );
        [kline, bar1, vol, bar2, ind]
    }

    #[allow(dead_code)]
    pub(crate) fn clamp_left_heights(&mut self, work: egui::Rect) {
        let budget = (work.height() - LEFT_FIXED_H).max(3.0 * PANE_H_MIN);
        let min = PANE_H_MIN;
        let ind_max = (budget - 2.0 * min).max(min);
        self.ind_h = snap_cells_up(self.ind_h).clamp(min, ind_max);
        let vol_max = (budget - min - self.ind_h).max(min);
        self.vol_h = snap_cells_up(self.vol_h).clamp(min, vol_max);
        self.kline_h = budget - self.vol_h - self.ind_h;
    }

    #[allow(dead_code)]
    pub(crate) fn ui_hsplitter(
        &mut self,
        ui: &mut egui::Ui,
        bar: egui::Rect,
        work: egui::Rect,
        which: LeftSplit,
    ) {
        let chart_dragging = self.pan.is_some();
        let resp = ui.allocate_rect(
            bar,
            if chart_dragging {
                egui::Sense::hover()
            } else {
                egui::Sense::drag()
            },
        );
        if !chart_dragging && (resp.hovered() || resp.dragged()) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);

            self.pointer.hint = CursorKind::ResizeV;
        }
        if resp.drag_started() {
            let grab = ui
                .input(|i| i.pointer.latest_pos())
                .map(|p| p.y - bar.min.y);
            match which {
                LeftSplit::Volume => self.hsplit_grab = grab,
                LeftSplit::Indicator => self.vsplit_grab = grab,
            }
        }
        if resp.dragged() {
            let grab = match which {
                LeftSplit::Volume => self.hsplit_grab,
                LeftSplit::Indicator => self.vsplit_grab,
            };
            if let (Some(grab), Some(pointer)) = (grab, ui.input(|i| i.pointer.latest_pos())) {
                let want = match which {
                    LeftSplit::Volume => self.vol_h - (pointer.y - grab - bar.min.y),
                    LeftSplit::Indicator => self.ind_h - (pointer.y - grab - bar.min.y),
                };
                let want = snap_cells_round(want);
                match which {
                    LeftSplit::Volume => self.vol_h = want,
                    LeftSplit::Indicator => self.ind_h = want,
                }
                self.clamp_left_heights(work);
            }
        }
        if resp.drag_stopped() {
            match which {
                LeftSplit::Volume => self.hsplit_grab = None,
                LeftSplit::Indicator => self.vsplit_grab = None,
            }
        }

        let painter = ui.painter().clone();
        painter.rect_filled(bar, egui::CornerRadius::ZERO, theme().splitter);
        hline_hard(
            &painter,
            bar.min.x,
            bar.max.x,
            bar.min.y,
            theme().ctrl_border,
        );
        hline_hard(
            &painter,
            bar.min.x,
            bar.max.x,
            bar.max.y - LINE_W,
            theme().ctrl_border,
        );
    }

    pub(crate) fn ui_panel(&mut self, ui: &mut egui::Ui, body: egui::Rect) {
        let painter = ui.painter().clone();

        painter.rect_filled(body, egui::CornerRadius::ZERO, theme().panel_bg);

        let mut rects = Vec::with_capacity(self.panel_buttons.len());
        let x = body.min.x + grid_line(1);
        let mut y = body.min.y + grid_line(1);
        for button in &self.panel_buttons {
            rects.push(egui::Rect::from_min_size(
                egui::pos2(x, y),
                egui::vec2(button.button.w, BTN_H),
            ));
            y += BTN_STEP;
        }

        let pointer = ui.input(|i| i.pointer.hover_pos());
        let down = ui.input(|i| i.pointer.primary_down());
        let hot = rects
            .iter()
            .position(|r| pointer.is_some_and(|p| r.contains(p)));
        self.panel_press = if down { hot } else { None };

        for (i, rect) in rects.iter().enumerate() {
            let content = self.panel_buttons[i].button.content.clone();
            let action = self.panel_buttons[i].action;
            let pressed = self.panel_press == Some(i);
            let label = self.panel_buttons[i].button.label();

            if self.draw_button(ui, &painter, *rect, &content, pressed, "") {
                self.run_action(action, &label);
            }
        }
    }

    pub(crate) fn build_panel_buttons(&mut self) {
        let items: [(ButtonContent, &str, PanelAction); PANEL_BTN_COUNT as usize] = [
            (
                ButtonContent::Text("重新生成".to_owned()),
                "换一个种子重新生成一段行情",
                PanelAction::NewRandomWalk,
            ),
            (
                ButtonContent::Text("设置涨色".to_owned()),
                "打开调色板，改 K 线的涨色",
                PanelAction::OpenUpColor,
            ),
            (
                ButtonContent::Text("设置跌色".to_owned()),
                "打开调色板，改 K 线的跌色",
                PanelAction::OpenDownColor,
            ),
            (
                ButtonContent::Text("切换鼠标控制".to_owned()),
                "在软件鼠标与系统鼠标之间切换；系统鼠标保持系统指针原样",
                PanelAction::TogglePointerMode,
            ),
            (
                ButtonContent::Text("鼠标速度 +".to_owned()),
                "提高软件指针的基础速度（不写回系统设置）",
                PanelAction::PointerSpeedUp,
            ),
            (
                ButtonContent::Text("鼠标速度 -".to_owned()),
                "降低软件指针的基础速度（不写回系统设置）",
                PanelAction::PointerSpeedDown,
            ),
            (
                ButtonContent::Text("鼠标加速 +".to_owned()),
                "增强快速移动时的指针加速度",
                PanelAction::PointerAccelUp,
            ),
            (
                ButtonContent::Text("鼠标加速 -".to_owned()),
                "减弱快速移动时的指针加速度（减到 0 = 完全线性）",
                PanelAction::PointerAccelDown,
            ),
            (
                ButtonContent::Text("鼠标轨迹".to_owned()),
                "开关指针工程调试叠层：最近一段轨迹、速度、加速度和坐标",
                PanelAction::TogglePointerTrail,
            ),
            (
                ButtonContent::Text("放大".to_owned()),
                "K 线放大一档",
                PanelAction::ZoomIn,
            ),
            (
                ButtonContent::Text("缩小".to_owned()),
                "K 线缩小一档",
                PanelAction::ZoomOut,
            ),
            (
                ButtonContent::Text("重置视图".to_owned()),
                "缩放与平移都回到最初",
                PanelAction::ResetView,
            ),
            (
                ButtonContent::Text("打印当前状态".to_owned()),
                "把当前状态写到状态栏",
                PanelAction::PrintState,
            ),
            (
                ButtonContent::Text("UI工程测试".to_owned()),
                "给每块 UI 元素描一圈边界框（绿 = 四条边都在网格线上 / 橙 = 没对齐）+ 右下角坐标读数",
                PanelAction::ToggleGrid,
            ),
            (
                ButtonContent::Text("切换主题".to_owned()),
                "在浅色 / 深色两套皮肤之间切换",
                PanelAction::ToggleTheme,
            ),
            (
                ButtonContent::Text("图表配色跟随系统".to_owned()),
                "K 线涨跌色重新跟当前皮肤走",
                PanelAction::FollowSystemColors,
            ),
            (
                ButtonContent::Text("新增图表".to_owned()),
                "在窗口区左上角新增一块图表窗（被占了就往右下让；最多 8 块）",
                PanelAction::AddChart,
            ),
            (
                ButtonContent::Text("打开调试控制台".to_owned()),
                "打开调试控制台：运行状态、滚动日志和命令输入",
                PanelAction::OpenConsole,
            ),
        ];
        self.panel_buttons = items
            .into_iter()
            .map(|(content, tip, action)| {
                let mut button = Button::new(content).with_tip(tip);
                button.measure(self);
                PanelButton { button, action }
            })
            .collect();

        let min = self.panel_min_size();
        for w in self.win_mgr.wins.iter_mut() {
            if w.kind == WinKind::Panel {
                w.w.min_size = min;
            }
        }
    }

    pub(crate) fn run_action(&mut self, action: PanelAction, label: &str) {
        match action {
            PanelAction::NewRandomWalk => {
                self.regenerate_random_walk();
            }

            PanelAction::ZoomIn => self.zoom_step(1),
            PanelAction::ZoomOut => self.zoom_step(-1),

            PanelAction::OpenUpColor => self.open_color_panel(ColorTarget::Up),
            PanelAction::OpenDownColor => self.open_color_panel(ColorTarget::Down),
            PanelAction::PointerSpeedUp => {
                let speed = self.pointer.adjust_speed(POINTER_SPEED_STEP);
                self.status_msg = format!("鼠标速度：{speed:.2}x");
            }
            PanelAction::PointerSpeedDown => {
                let speed = self.pointer.adjust_speed(1.0 / POINTER_SPEED_STEP);
                self.status_msg = format!("鼠标速度：{speed:.2}x");
            }
            PanelAction::PointerAccelUp => {
                let accel = self.pointer.adjust_acceleration(POINTER_ACCEL_STEP);
                self.status_msg = format!("鼠标加速度：{accel:.2}");
            }
            PanelAction::PointerAccelDown => {
                let accel = self.pointer.adjust_acceleration(-POINTER_ACCEL_STEP);
                self.status_msg = format!("鼠标加速度：{accel:.2}");
            }
            PanelAction::TogglePointerMode => {
                self.pointer.system = !self.pointer.system;
                self.pointer.pinned = None;
                self.status_msg = if self.pointer.system {
                    "鼠标控制：系统鼠标（系统指针原样显示）".to_owned()
                } else {
                    format!(
                        "鼠标控制：软件鼠标（速度 {:.2}x，加速度 {:.2}）",
                        self.pointer.feel().base_speed,
                        self.pointer.feel().acceleration
                    )
                };
            }
            PanelAction::TogglePointerTrail => {
                self.pointer.show_trail = !self.pointer.show_trail;
                self.status_msg = if self.pointer.show_trail {
                    "鼠标轨迹：开（轨迹 + 速度 / 加速度 + 坐标）".to_owned()
                } else {
                    "鼠标轨迹：关".to_owned()
                };
            }
            PanelAction::ResetView => {
                self.scale = SCALE_MIN;
                self.offset = 0;
            }
            PanelAction::ToggleGrid => {
                self.show_grid = !self.show_grid;
                self.status_msg = if self.show_grid {
                    "UI工程测试：开（元素边界 + 坐标读数，见右下角牌子）".to_owned()
                } else {
                    "UI工程测试：关".to_owned()
                };
            }

            PanelAction::ToggleTheme => self.switch_theme(),
            PanelAction::FollowSystemColors => self.follow_system_chart_colors(),

            PanelAction::AddChart => self.request_win(WinCmd::AddChart),
            PanelAction::OpenConsole => self.request_win(WinCmd::OpenConsole),
            _ => {
                self.status_msg = format!("你点击了：{label}");
            }
        }
    }

    pub(crate) fn zoom_step(&mut self, notches: i32) {
        self.scale = scale_after(self.scale, notches);
    }

    pub(crate) fn regenerate_random_walk(&mut self) {
        self.bar_seed = self
            .bar_seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        crate::ecs::regenerate_market(&mut self.world);
        self.offset = 0;
        self.scale = BAR_SCALE_DEFAULT;
        self.crosshair = None;
        self.status_msg = format!("已生成新的随机行情（种子 {}）", self.bar_seed);
    }
}
