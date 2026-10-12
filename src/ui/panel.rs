// ===========================================================================
//  模块 ui::panel：两栏布局、分割条与右侧控制面板
// ===========================================================================
//  横向分割条、左栏上下拆分（K 线 / 成交量 / 指标）、右侧自绘按钮面板。
//  本文件 = 拆分前那份单文件里的「十、两栏布局、两根分割条与控制面板」。
use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::colorpanel::*;
use crate::data;
use eframe::egui;

// ===========================================================================
//  十、两栏布局、两根分割条与控制面板（自绘按钮）
// ===========================================================================
//  功能区横向切成三块：左栏 89 格 + 分割条 1 格 + 右栏控制面板 35 格 = 125 格（默认宽度下
//  正好等于窗口宽；窗口可以缩放，缩了之后按 split_max_x 现算上限）。三块相邻处共用网格线。
//  分割条：纯色 + 1px 深灰硬边，按住可以左右拖，步进 1 格（9px 节距），两栏都不小于 35 格。
//  左栏再纵向切成三块：K 线区 / 横向拖拽条 / 成交量区。横向拖拽条和竖向分割条同一套做法，
//  只是按住上下拖 —— 步进 1 格（8px），拖的是成交量区的高度，K 线区拿剩下的空间。
//  纵向这几块用的是「8px 内容格」口径（和颜色面板一致），横向那三块还是 cells(n) = 9n+1 口径。
//  控制面板：左边距和上边距都是 1 格；按钮从上往下排，相邻按钮共用中间那条网格线。
//  按钮：高 cells(3) = 28px；宽 = 内容宽 + 左右各 9px，向上吸到整数格（最多 24 格）；
//  三种状态三块纯色（普通灰 / 悬停稍亮 / 按下更暗）+ 1px 深灰硬边，无圆角、无阴影、无渐变。
//  所有按钮都走 draw_button 这一个函数，别处不要再抄一份外观。

/// 左栏 K 线区的默认宽度（也是分割条的初始位置）：89 格。
pub(crate) const CHART_PANE_W: f32 = cells(CHART_COLS_DEFAULT);

/// 右栏控制面板的默认宽度：35 格。面板实际宽度 = 功能区右边到分割条，
/// 拖分割条把右栏拉宽时，多出来的部分照样铺面板底色。
pub(crate) const PANEL_W: f32 = cells(PANEL_COLS);

/// 两栏的最小宽度：35 格。
pub(crate) const PANE_MIN_W: f32 = cells(PANE_MIN_COLS);

/// 分割条宽度：1 格，也就是 cells(1) = 10px（含两端各 1px 网格线格）。
pub(crate) const SPLITTER_W: f32 = cells(SPLITTER_COLS);

/// 左栏上下拆分（从上往下）：K 线区 / 拖拽条 / 成交量区 / 拖拽条 / 指标区。
/// K 线区默认 600px（75 格）、最小 240px（30 格）；成交量区和指标区默认都是 96px（12 格）、
/// 最小都是 64px（8 格）；两处之间各夹一条 8px 的横向拖拽条。
/// 注意：三块的默认值合计 600 + 96 + 96 + 固定开销 20 = 812px，比功能区在默认窗口高度下
/// 那 523px 高 —— 装不下时按 clamp_left_heights 的规矩从下往上收口（指标区、成交量区各自
/// 保住最小高度，K 线区吃剩下的），所以默认窗口下实际是 K 线 304px、成交量 96px、指标 96px
/// （指标区在底部再多吸收 K 线区吸零头剩下的 7px）；想看满 600px，把窗口拉高就行。
pub(crate) const KLINE_ROWS: i32 = 75;
pub(crate) const VOL_ROWS: i32 = 12;
pub(crate) const IND_ROWS: i32 = 12;
pub(crate) const KLINE_H_DEFAULT: f32 = KLINE_ROWS as f32 * GRID;
pub(crate) const KLINE_H_MIN: f32 = 30.0 * GRID;
pub(crate) const VOL_H_DEFAULT: f32 = VOL_ROWS as f32 * GRID;
pub(crate) const VOL_H_MIN: f32 = 8.0 * GRID;
pub(crate) const IND_H_DEFAULT: f32 = IND_ROWS as f32 * GRID;
pub(crate) const IND_H_MIN: f32 = 8.0 * GRID;

/// 指标区纵轴上的刻度条数（和成交量区一样，2~3 条就够）。
pub(crate) const IND_TICK_COUNT: usize = 3;

/// 横向拖拽条高 8px（1 格），拖动步进也是 8px（1 格）。
pub(crate) const HSPLIT_H: f32 = GRID;
pub(crate) const HSPLIT_STEP: f32 = GRID;

/// 左栏里那两条横向拖拽条：上面那条改成交量区的高度，下面那条改指标区的高度。
/// 两条的行为一模一样，只是拖的是不同的块、记的是不同的偏移。
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum LeftSplit {
    Volume,
    Indicator,
}

/// 左栏三块之间的固定开销：两条拖拽条（各 8px）+ 它们上下那四条 1px 线格。
pub(crate) const LEFT_FIXED_H: f32 = 2.0 * HSPLIT_H + 4.0 * LINE_W;

// 编译期自检：这几块都得是 8 的倍数（规格原文就是「8 的倍数」口径），最小高度也不例外。
const _: () = assert!(KLINE_H_DEFAULT as i32 % 8 == 0 && KLINE_H_DEFAULT as i32 == 600);
const _: () = assert!(KLINE_H_MIN as i32 % 8 == 0 && KLINE_H_MIN as i32 == 240);
const _: () = assert!(VOL_H_DEFAULT as i32 % 8 == 0 && VOL_H_DEFAULT as i32 == 96);
const _: () = assert!(VOL_H_MIN as i32 % 8 == 0 && VOL_H_MIN as i32 == 64);
const _: () = assert!(IND_H_DEFAULT as i32 % 8 == 0 && IND_H_DEFAULT as i32 == 96);
const _: () = assert!(IND_H_MIN as i32 % 8 == 0 && IND_H_MIN as i32 == 64);
const _: () = assert!(HSPLIT_H % 8.0 == 0.0 && HSPLIT_STEP % 8.0 == 0.0);

/// 按钮高 3 格；相邻按钮共用中间那条网格线，所以往下排的步进是 grid_line(3) = 27px。
pub(crate) const BTN_ROWS: i32 = 3;
pub(crate) const BTN_H: f32 = cells(BTN_ROWS);
pub(crate) const BTN_STEP: f32 = grid_line(BTN_ROWS);

/// 按钮最大宽度：24 格（16 个汉字 = 192px，加左右各 9px 内边距）。
pub(crate) const BTN_MAX_COLS: i32 = 24;
pub(crate) const BTN_MAX_W: f32 = cells(BTN_MAX_COLS);

/// 图标区固定 2 格内容（16x16），图标和文字之间再留 1 格（8px）。
pub(crate) const ICON_W: f32 = 2.0 * GRID;
pub(crate) const ICON_GAP: f32 = GRID;

// 编译期自检：这些东西也必须站在网格上（尺寸都是 cells(n) = 9n + 1 的形状）。
const _: () = assert!((CHART_PANE_W as i32 - 1) % 9 == 0);
const _: () = assert!((PANEL_W as i32 - 1) % 9 == 0);
const _: () = assert!((PANE_MIN_W as i32 - 1) % 9 == 0);
const _: () = assert!((SPLITTER_W as i32 - 1) % 9 == 0);
const _: () = assert!((BTN_H as i32 - 1) % 9 == 0);
const _: () = assert!((BTN_MAX_W as i32 - 1) % 9 == 0);
const _: () = assert!((BTN_STEP as i32) % 9 == 0);
const _: () = assert!((ICON_W as i32) % 8 == 0);
const _: () = assert!((ICON_GAP as i32) % 8 == 0);
// 默认宽度关系：左栏 802 + 分割条 10 + 右栏 316 - 两条共用的网格线 2 = 窗口宽 1126（正好铺满）。
const _: () = assert!(
    CHART_PANE_W as i32 + SPLITTER_W as i32 + PANEL_W as i32 - 2 == WINDOW_W as i32
);

/// 按钮的三种外观状态。
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ButtonState {
    Idle,
    Hover,
    Pressed,
}

impl ButtonState {
    /// 底色：三种状态各一块纯色，只有明暗差别。
    pub(crate) fn face(self) -> egui::Color32 {
        match self {
            ButtonState::Idle => C_BTN_FACE,
            ButtonState::Hover => C_BTN_HOVER,
            ButtonState::Pressed => C_BTN_PRESS,
        }
    }
}

/// 手绘图标：都画在 16x16 的图标区里，只用整数像素的矩形和直线，没有抗锯齿。
#[derive(Clone, Copy)]
pub(crate) enum IconKind {
    /// 叉（两条对角线）。
    Cross,
}

impl IconKind {
    /// 在给定的 16x16 矩形里画自己：先把矩形吸到整数像素，图形才可能左右对称。
    pub(crate) fn paint(self, painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
        let rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x.round(), rect.min.y.round()),
            egui::vec2(ICON_W, ICON_W),
        );
        match self {
            IconKind::Cross => {
                // 中心像素取 16 格图标的正中那一列（第 8 列），半径 5 → 11x11 的叉。
                let center = egui::pos2(rect.min.x + ICON_W / 2.0, rect.min.y + ICON_W / 2.0);
                paint_cross(painter, center, 5, color);
            }
        }
    }
}

/// 按钮内容：纯文字 / 图标 + 文字。
#[derive(Clone)]
pub(crate) enum ButtonContent {
    Text(String),
    IconText(IconKind, String),
}

impl ButtonContent {
    /// 状态栏回显用的名字。
    pub(crate) fn label(&self) -> String {
        match self {
            ButtonContent::Text(text) => text.clone(),
            ButtonContent::IconText(_, text) => text.clone(),
        }
    }
}

/// 控制面板按钮的动作。行情生成、缩放 / 平移、打开颜色面板都是真做的。
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PanelAction {
    /// 换一个新种子重新生成行情，并把视图（平移 / 缩放）归零。
    NewRandomWalk,
    /// 打开颜色面板，编辑目标定成涨色。
    OpenUpColor,
    /// 打开颜色面板，编辑目标定成跌色。
    OpenDownColor,
    /// 放大 / 缩小一格：跟滚轮同一套范围（3..=15 的奇数档），只是没有鼠标锚点。
    ZoomIn,
    ZoomOut,
    /// 重置视图：scale 回到最小、offset 回到最前面那根。
    ResetView,
    /// 把当前状态打到状态栏。
    PrintState,
}

/// 控制面板里的一个按钮：内容 + 动作 + 已经量好的宽度（宽度和面板位置无关，开机算一次就够）。
pub(crate) struct PanelButton {
    pub(crate) content: ButtonContent,
    pub(crate) action: PanelAction,
    pub(crate) w: f32,
}

impl StockApp {
    /// 功能区 →（左栏, 分割条, 右栏）三块。三块相邻处共用一条网格线：
    /// 分割条左边那条线就是左栏的右边框，右边那条线就是右栏的左边框。
    pub(crate) fn work_columns(work: egui::Rect, split_x: f32) -> [egui::Rect; 3] {
        let x0 = work.min.x;
        let y0 = work.min.y;
        let y1 = work.max.y;
        // split_x 是左栏的宽度（一定是 cells(n)），所以左栏右边那条网格线在 x0 + split_x - 1px 上。
        let bar_x = x0 + split_x - LINE_W;
        let left = egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(bar_x + LINE_W, y1));
        let splitter =
            egui::Rect::from_min_max(egui::pos2(bar_x, y0), egui::pos2(bar_x + SPLITTER_W, y1));
        // 右栏一直铺到功能区右边界（也是窗口最外圈右边框）：
        // 少铺这 1px，它就没被任何东西盖住，会露出功能区那块白底，在最右边形成一条白竖线；
        // 最外圈边框是最后画的，正好压在上面，所以铺满不会把边框糊掉。
        let panel = egui::Rect::from_min_max(
            egui::pos2(bar_x + SPLITTER_W - LINE_W, y0),
            egui::pos2(work.max.x, y1),
        );
        [left, splitter, panel]
    }

    /// 分割条能拖到的最右边：再往右右栏就低于最小宽度了。按当前功能区宽度现算
    /// （窗口可以缩放，SPLIT_MAX 那种编译期常量已经不适用）。结果一定是 cells(n)。
    pub(crate) fn split_max_x(work: egui::Rect) -> f32 {
        // 一屏能放几格：cells(n) 宽 = 9n + 1，所以 n = (w - 1) / 9 向下取整。
        let cols = ((work.width() - LINE_W) / PITCH).floor() as i32;
        cells((cols - SPLITTER_COLS - PANE_MIN_COLS).max(PANE_MIN_COLS))
    }

    /// 把分割条位置收口：左栏宽度永远是整数格（cells(n)），并且两栏都不小于 35 格。
    /// 窗口变窄 / 变宽之后每帧都过一道，不用管上次拖到哪儿。
    pub(crate) fn clamp_split(&mut self, work: egui::Rect) {
        self.split_x = snap_cells_up(self.split_x).clamp(PANE_MIN_W, Self::split_max_x(work));
    }

    /// 分割条：纯色 + 1px 硬边，按住左右拖，一步 1 格（9px 节距）；两栏都不小于 35 格。
    pub(crate) fn ui_splitter(&mut self, ui: &mut egui::Ui, bar: egui::Rect, work: egui::Rect) {
        let resp = ui.allocate_rect(bar, egui::Sense::drag());
        if resp.hovered() || resp.dragged() {
            // 鼠标停在分割条上时给个左右调整的光标。
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        if resp.drag_started() {
            // 记住按下那一刻鼠标离分割条左边缘多远：拖动时按这个偏移算，分割条不会跳。
            self.split_grab = ui.input(|i| i.pointer.latest_pos()).map(|p| p.x - bar.min.x);
        }
        if resp.dragged() {
            if let (Some(grab), Some(pointer)) =
                (self.split_grab, ui.input(|i| i.pointer.latest_pos()))
            {
                // 新的左栏宽度 = 当前宽度 + 鼠标位移；先吸到整数格再夹住 —— 拖的是「格」，不是像素。
                let want = self.split_x + (pointer.x - grab - bar.min.x);
                self.split_x = snap_cells_up(want).clamp(PANE_MIN_W, Self::split_max_x(work));
            }
        }
        if resp.drag_stopped() {
            self.split_grab = None;
        }

        let painter = ui.painter().clone();
        painter.rect_filled(bar, egui::CornerRadius::ZERO, C_SPLITTER);
        // 只画左右两条 1px 竖线：上边那条正好和菜单栏分隔线同在一行、下边那条紧贴状态栏分隔线，
        // 自己再画就成了 2px 的堆叠，所以上下都不画，交给区域分隔线收边。
        vline_hard(&painter, bar.min.x, bar.min.y, bar.max.y, C_BTN_BORDER);
        vline_hard(
            &painter,
            bar.max.x - LINE_W,
            bar.min.y,
            bar.max.y,
            C_BTN_BORDER,
        );
    }

    /// 左栏 →（K 线区, 拖拽条, 成交量区, 拖拽条, 指标区）。五块从上往下排：
    /// K 线区 / 1px 线格 / 拖拽条 8px / 1px 线格 / 成交量区 / 1px 线格 / 拖拽条 8px / 1px 线格 / 指标区。
    /// 那四条 1px 线格各占一整行，夹在两块之间（谁也不吃掉谁的像素）；
    /// 指标区的下边缘就是左栏底边（状态栏上沿那条网格线）：K 线区高度向下吸 8 的倍数之后
    /// 剩下的那 0..7px 余量归指标区，所以底下永远不会剩一条白条。
    pub(crate) fn left_rows(work: egui::Rect, kline_h: f32, vol_h: f32) -> [egui::Rect; 5] {
        let x0 = work.min.x;
        let w = work.width();
        let kline = egui::Rect::from_min_size(egui::pos2(x0, work.min.y), egui::vec2(w, kline_h));
        let bar1 = egui::Rect::from_min_size(
            egui::pos2(x0, kline.max.y + LINE_W),
            egui::vec2(w, HSPLIT_H),
        );
        let vol = egui::Rect::from_min_size(
            egui::pos2(x0, bar1.max.y + LINE_W),
            egui::vec2(w, vol_h),
        );
        let bar2 = egui::Rect::from_min_size(
            egui::pos2(x0, vol.max.y + LINE_W),
            egui::vec2(w, HSPLIT_H),
        );
        // 指标区铺到底（下边缘 = 左栏底边），K 线区吸零头剩下的 0..7px 由它吸收。
        let ind = egui::Rect::from_min_max(
            egui::pos2(x0, bar2.max.y + LINE_W),
            egui::pos2(x0 + w, work.max.y),
        );
        [kline, bar1, vol, bar2, ind]
    }

    /// 把三块高度收口，都是 8 的倍数、各自不小于最小高度：
    ///   · 指标区（最下面那块）：上限 = 给上面两块留出各自最小高度之后还剩多少；
    ///   · 成交量区：同理，这次给 K 线区留出最小高度；
    ///   · K 线区：吃掉剩下的空间，向下吸到 8 的倍数。
    /// 吸零头剩下的那 0..7px 让指标区在底部吸收（left_rows 铺到底），所以底下不留白条。
    /// 拖动（ui_hsplitter）和换窗口尺寸都从这儿过一道，三块高度都落在 8 的倍数上。
    pub(crate) fn clamp_left_heights(&mut self, work: egui::Rect) {
        let avail = work.height();
        let ind_max = snap8_down((avail - LEFT_FIXED_H - KLINE_H_MIN - VOL_H_MIN).max(IND_H_MIN));
        self.ind_h = snap8_up(self.ind_h.clamp(IND_H_MIN, ind_max));
        let vol_max = snap8_down((avail - LEFT_FIXED_H - KLINE_H_MIN - self.ind_h).max(VOL_H_MIN));
        self.vol_h = snap8_up(self.vol_h.clamp(VOL_H_MIN, vol_max));
        self.kline_h =
            snap8_down((avail - LEFT_FIXED_H - self.vol_h - self.ind_h).max(KLINE_H_MIN));
    }

    /// 横向拖拽条：和竖向分割条同一套（纯色 + 1px 硬边），只是改成上下拖。
    /// 拖动步进 8px（1 格），它下面那块跟着变高、上面那块相应变矮，三块各自不小于最小高度。
    /// which 决定拖的是哪一块的高度（上面那条拖拽条改成交量区，下面那条改指标区），
    /// 拖出来的值都存 struct 字段（整数像素，8 的倍数）。
    pub(crate) fn ui_hsplitter(
        &mut self,
        ui: &mut egui::Ui,
        bar: egui::Rect,
        work: egui::Rect,
        which: LeftSplit,
    ) {
        let resp = ui.allocate_rect(bar, egui::Sense::drag());
        if resp.hovered() || resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
        if resp.drag_started() {
            // 记住按下那一刻鼠标离拖拽条上边缘多远：拖动时按这个偏移算，拖拽条不会跳。
            let grab = ui.input(|i| i.pointer.latest_pos()).map(|p| p.y - bar.min.y);
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
                // 新的块高 = 当前块高 - 鼠标纵向位移（往下拖 = 拖拽条下移 = 下面那块变矮）；
                // 先吸到 8 的倍数再收口 —— 拖的是「格」，块里画的东西跟着一起变。
                let want = match which {
                    LeftSplit::Volume => self.vol_h - (pointer.y - grab - bar.min.y),
                    LeftSplit::Indicator => self.ind_h - (pointer.y - grab - bar.min.y),
                };
                let want = (want / HSPLIT_STEP).round() * HSPLIT_STEP;
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
        painter.rect_filled(bar, egui::CornerRadius::ZERO, C_SPLITTER);
        // 上下两条 1px 线格就是它和上下两块的分界线，画在这儿（那两块自己不再画）。
        hline_hard(&painter, bar.min.x, bar.max.x, bar.min.y - LINE_W, C_BTN_BORDER);
        hline_hard(&painter, bar.min.x, bar.max.x, bar.max.y, C_BTN_BORDER);
    }

    /// 按钮宽度 = 内容宽 + 左右各 9px 内边距，向上吸到整数格，最多 24 格。
    /// 内容宽：纯文字 = 文字宽；图标 + 文字 = 2 格 + 1 格 + 文字宽。
    pub(crate) fn button_width(&mut self, content: &ButtonContent) -> f32 {
        let inner = match content {
            ButtonContent::Text(text) => self.text_width(text),
            ButtonContent::IconText(_, text) => ICON_W + ICON_GAP + self.text_width(text),
        };
        snap_cells_up(inner + 2.0 * TEXT_PAD_X).min(BTN_MAX_W)
    }

    /// 通用按钮：登记点击 → 按状态铺底色和 1px 深灰硬边 → 按内容画图标 / 文字，
    /// 返回这一帧是否被点击。全工程的按钮都从这儿走，别处不要再写一份外观。
    pub(crate) fn draw_button(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        rect: egui::Rect,
        content: &ButtonContent,
        pressed: bool,
    ) -> bool {
        let resp = ui.allocate_rect(rect, egui::Sense::click());
        let state = if pressed || resp.is_pointer_button_down_on() {
            ButtonState::Pressed
        } else if resp.hovered() {
            ButtonState::Hover
        } else {
            ButtonState::Idle
        };

        // 先铺底色再描边：边框永远压在高亮之上，四条边都完整。
        painter.rect_filled(rect, egui::CornerRadius::ZERO, state.face());
        border_hard(painter, rect, C_BTN_BORDER);

        // 内容：纯文字就整块居中；带图标时「图标 + 间隔 + 文字」作为一个整体居中。
        match content {
            ButtonContent::Text(text) => {
                self.text(painter, rect.center(), egui::Align2::CENTER_CENTER, text, C_TEXT);
            }
            ButtonContent::IconText(kind, text) => {
                let total = ICON_W + ICON_GAP + self.text_width(text);
                let icon_x = (rect.center().x - total / 2.0).round();
                let icon = egui::Rect::from_min_size(
                    egui::pos2(icon_x, rect.center().y - ICON_W / 2.0),
                    egui::vec2(ICON_W, ICON_W),
                );
                kind.paint(painter, icon, C_TEXT);
                self.text(
                    painter,
                    egui::pos2(icon.max.x + ICON_GAP, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    text,
                    C_TEXT,
                );
            }
        }
        resp.clicked()
    }

    /// 右栏控制面板：一列自绘按钮；缩放 / 重置直接改视图，其余按钮在状态栏回显一句。
    pub(crate) fn ui_panel(&mut self, ui: &mut egui::Ui, panel: egui::Rect) {
        let painter = ui.painter().clone();
        painter.rect_filled(panel, egui::CornerRadius::ZERO, C_PANEL_BG);

        // 1) 排位置：左边距和上边距都是 1 格；按钮从上往下排，相邻按钮共用中间那条网格线，
        //    所以步进是 grid_line(3) = 27px、按钮高是 cells(3) = 28px（宽度是开机量好的）。
        let mut rects = Vec::with_capacity(self.panel_buttons.len());
        let x = panel.min.x + grid_line(1);
        let mut y = panel.min.y + grid_line(1);
        for button in &self.panel_buttons {
            rects.push(egui::Rect::from_min_size(
                egui::pos2(x, y),
                egui::vec2(button.w, BTN_H),
            ));
            y += BTN_STEP;
        }

        // 2) 鼠标现在压在哪一个按钮上？按下的那个存进 struct 字段。
        let pointer = ui.input(|i| i.pointer.hover_pos());
        let down = ui.input(|i| i.pointer.primary_down());
        let hot = rects
            .iter()
            .position(|r| pointer.is_some_and(|p| r.contains(p)));
        self.panel_press = if down { hot } else { None };

        // 3) 画 + 收点击（内容复制一份小字符串：不能一边借 panel_buttons 一边调 &mut self）。
        for i in 0..rects.len() {
            let content = self.panel_buttons[i].content.clone();
            let action = self.panel_buttons[i].action;
            let pressed = self.panel_press == Some(i);
            // draw_button 返回是否点击；点到了就按这个按钮的动作办。
            if self.draw_button(ui, &painter, rects[i], &content, pressed) {
                self.run_action(action, &content.label());
            }
        }
    }

    /// 控制面板的按钮表：内容 + 动作。宽度在这里一次量好（跟面板宽度无关）。
    /// 行情生成只留「重新生成」一个入口：换一个新种子生成一段新行情。
    pub(crate) fn build_panel_buttons(&mut self) {
        let items = [
            (
                ButtonContent::Text("重新生成".to_owned()),
                PanelAction::NewRandomWalk,
            ),
            (
                ButtonContent::Text("设置涨色".to_owned()),
                PanelAction::OpenUpColor,
            ),
            (
                ButtonContent::Text("设置跌色".to_owned()),
                PanelAction::OpenDownColor,
            ),
            (ButtonContent::Text("放大".to_owned()), PanelAction::ZoomIn),
            (ButtonContent::Text("缩小".to_owned()), PanelAction::ZoomOut),
            (
                ButtonContent::IconText(IconKind::Cross, "重置视图".to_owned()),
                PanelAction::ResetView,
            ),
            (
                ButtonContent::Text("打印当前状态".to_owned()),
                PanelAction::PrintState,
            ),
        ];
        self.panel_buttons = items
            .into_iter()
            .map(|(content, action)| {
                let w = self.button_width(&content);
                PanelButton { content, action, w }
            })
            .collect();
    }

    /// 执行一个面板动作。缩放 / 平移都是直接改视图状态，下一帧 ui_kline 就按新值出图。
    pub(crate) fn run_action(&mut self, action: PanelAction, label: &str) {
        match action {
            PanelAction::NewRandomWalk => {
                self.regenerate_random_walk();
            }
            // 状态栏本来就在实时显示 scale / offset，缩放和重置不用再回显一句。
            PanelAction::ZoomIn => self.zoom_step(1),
            PanelAction::ZoomOut => self.zoom_step(-1),
            // 颜色面板：点按钮就是「打开面板 + 指定编辑目标」。
            PanelAction::OpenUpColor => self.open_color_panel(ColorTarget::Up),
            PanelAction::OpenDownColor => self.open_color_panel(ColorTarget::Down),
            PanelAction::ResetView => {
                self.scale = SCALE_MIN;
                self.offset = 0;
            }
            _ => {
                self.status_msg = format!("你点击了：{label}");
            }
        }
    }

    /// 面板上的放大 / 缩小：一格 ±2 像素（见 scale_after），范围跟滚轮完全一样（3..=15 的奇数档）。
    /// 没有鼠标锚点，就地改 scale、offset 先不动，越界交给下一帧的 clamp_view 夹。
    pub(crate) fn zoom_step(&mut self, notches: i32) {
        self.scale = scale_after(self.scale, notches);
    }

    /// 换一个新种子，重新生成一段随机游走行情，并把视图（平移 / 缩放）归零。
    /// 种子按固定步进往前推：每次点出来的行情都不一样，但整条序列完全可复现。
    pub(crate) fn regenerate_random_walk(&mut self) {
        self.bar_seed = self
            .bar_seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.bars = data::random_walk::generate(self.bar_seed, BAR_COUNT, BAR_START);
        // 数据换了一代：映射缓存（价格上下界 / 最大量）必须跟着作废。
        self.data_gen = self.data_gen.wrapping_add(1);
        self.offset = 0;
        self.scale = BAR_SCALE_DEFAULT;
        self.crosshair = None;
        self.status_msg = format!("已生成新的随机行情（种子 {}）", self.bar_seed);
    }
}

