// ===========================================================================
//  模块 ui::panel：两栏布局、分割条与右侧控制面板
// ===========================================================================
//  横向分割条、左栏上下拆分（K 线 / 成交量 / 指标）、右侧自绘按钮面板。
//  本文件 = 拆分前那份单文件里的「十、两栏布局、两根分割条与控制面板」。
use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::button::*;
use crate::ui::colorpanel::*;
use crate::ui::icon::*;
use crate::data;
use eframe::egui;

// ===========================================================================
//  十、两栏布局、两根分割条与控制面板（自绘按钮）
// ===========================================================================
//  功能区横向切成三块：左栏 89 格 + 分割条 1 格 + 右栏控制面板 35 格 = 125 格（默认宽度下
//  正好等于窗口宽；窗口可以缩放，缩了之后按 split_max_x 现算上限）。三块相邻处共用网格线。
//  分割条：纯色 + 1px 深灰硬边，按住可以左右拖，步进 1 格（9px 节距），两栏都不小于 35 格。
//  左栏再纵向切成三块图表：图表1 / 横向拖拽条 / 图表2 / 横向拖拽条 / 图表3。三块图表不分
//  主图 / 副图，用同一套规矩：高度都是整数格 cells(n)、下限都是 PANE_H_MIN。横向拖拽条和竖向
//  分割条同一套做法（整条 1 格 = 1px 线 + 8px 本体 + 1px 线），只是按住上下拖 - 步进 1 格，
//  相邻两块共用边界那条 1px 线格，整列落在窗口的 9px 网格线上。
//  控制面板：左边距和上边距都是 1 格；按钮从上往下排，相邻按钮共用中间那条网格线。
//  按钮：高 cells(2) = 19px（一个文本行盒）；宽 = 内容宽 + 左右各 9px，向上吸到整数格（最多 24 格）；
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

/// 左栏上下拆分（从上往下）：图表1 / 拖拽条 / 图表2 / 拖拽条 / 图表3。
/// 三块一律是「图表」（不分主图 / 副图）：高度都是整数格 cells(rows)，共用同一条最小高度
/// PANE_H_MIN；它们之间夹两条 1 格（cells(1) = 10px）的横向拖拽条，外观和竖向分割条一模一样。
/// 默认：图表1 52 格、图表2 / 图表3 各 21 格，合计 52 + 1 + 21 + 1 + 21 = 96 格，正好等于默认
/// 功能区高度（见 grid::WORK_ROWS）；窗口不够高时按 clamp_left_heights 从下往上收口。
pub(crate) const KLINE_ROWS: i32 = 52;
pub(crate) const VOL_ROWS: i32 = 21;
pub(crate) const IND_ROWS: i32 = 21;
pub(crate) const KLINE_H_DEFAULT: f32 = cells(KLINE_ROWS);
pub(crate) const VOL_H_DEFAULT: f32 = cells(VOL_ROWS);
pub(crate) const IND_H_DEFAULT: f32 = cells(IND_ROWS);

/// 横向拖拽条高 1 格（cells(1) = 10px：1px 线格 + 8px 内容格 + 1px 线格），
/// 和竖向分割条同一个口径；拖动步进 1 格（PITCH = 9px）。
pub(crate) const HSPLIT_H: f32 = cells(1);
pub(crate) const HSPLIT_STEP: f32 = PITCH;

/// 左栏里那两条横向拖拽条：上面那条改成交量区的高度，下面那条改指标区的高度。
/// 两条的行为一模一样，只是拖的是不同的块、记的是不同的偏移。
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum LeftSplit {
    Volume,
    Indicator,
}

/// 左栏三块之间的固定开销：两条拖拽条（各 cells(1)），再减掉相邻边界各自共用的那 1px
/// （四条边界线）：2*cells(1) - 4px = 16。三块高度加起来 = 可用高度 - LEFT_FIXED_H。
pub(crate) const LEFT_FIXED_H: f32 = 2.0 * HSPLIT_H - 4.0 * LINE_W;

// 编译期自检：三块默认高度都是 cells(n)、两条拖拽条 / 拖动步进都落在整数格上，默认合计正好填满默认功能区。
const _: () = assert!(KLINE_ROWS == 52 && VOL_ROWS == 21 && IND_ROWS == 21);
const _: () = assert!((KLINE_H_DEFAULT as i32 - 1) % 9 == 0);
const _: () = assert!((VOL_H_DEFAULT as i32 - 1) % 9 == 0);
const _: () = assert!((IND_H_DEFAULT as i32 - 1) % 9 == 0);
const _: () = assert!(KLINE_ROWS + 1 + VOL_ROWS + 1 + IND_ROWS == WORK_ROWS);
const _: () = assert!((HSPLIT_H as i32 - 1) % 9 == 0 && HSPLIT_STEP as i32 % 9 == 0);

// 按钮的尺寸 / 状态 / 画法全在 ui::button 里 - 那是「按钮」这一类东西的唯一一份定义，
// 控制面板按钮、编辑器按钮栏、图表标题栏按钮都走它（下面这些 BTN_* 就是 use 进来的那份）。
// 图标本身（IconKind / ICON_W / ICON_H / ICON_GAP）在 ui::icon 里，同一个道理。

// 编译期自检：这些东西也必须站在网格上（尺寸都是 cells(n) = 9n + 1 的形状）。
const _: () = assert!((CHART_PANE_W as i32 - 1) % 9 == 0);
const _: () = assert!((PANEL_W as i32 - 1) % 9 == 0);
const _: () = assert!((PANE_MIN_W as i32 - 1) % 9 == 0);
const _: () = assert!((SPLITTER_W as i32 - 1) % 9 == 0);
const _: () = assert!((BTN_H as i32 - 1) % 9 == 0);
const _: () = assert!((BTN_MAX_W as i32 - 1) % 9 == 0);
const _: () = assert!((BTN_STEP as i32) % 9 == 0);
const _: () = assert!((ICON_W as i32) % 8 == 0);
const _: () = assert!((ICON_W as i32 + ICON_GAP as i32) % 9 == 0);
// 默认宽度关系：左栏 802 + 分割条 10 + 右栏 316 - 两条共用的网格线 2 = 窗口宽 1126（正好铺满）。
const _: () = assert!(
    CHART_PANE_W as i32 + SPLITTER_W as i32 + PANEL_W as i32 - 2 == WINDOW_W as i32
);

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
    /// 网格测试叠层开关：把 9px 网格线铺在界面最上层，用来核对控件边框有没有对齐网格。
    ToggleGrid,
    /// 切换皮肤（经典 / 暗色）：整套界面颜色立刻跟着变，图表区按新皮肤的系统级配色走。
    ToggleTheme,
    /// 图表区的 K 线涨跌色重新指向当前皮肤的系统级配色（用户在颜色面板里定过色后想跟回去就点它）。
    FollowSystemColors,
}

/// 控制面板里的一个按钮：通用按钮（内容 + 便签 + 量好的宽）+ 点它要做的动作。
pub(crate) struct PanelButton {
    pub(crate) button: Button,
    pub(crate) action: PanelAction,
}

impl StockApp {
    /// 功能区 ->（左栏, 分割条, 右栏）三块。三块相邻处共用一条网格线：
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
    /// 正在拖图表（平移）的时候整条让路：不抢拖动会话、也不显示调整光标 - 两者互不干扰。
    pub(crate) fn ui_splitter(&mut self, ui: &mut egui::Ui, bar: egui::Rect, work: egui::Rect) {
        let chart_dragging = self.pan.is_some();
        let resp = ui.allocate_rect(
            bar,
            if chart_dragging { egui::Sense::hover() } else { egui::Sense::drag() },
        );
        if !chart_dragging && (resp.hovered() || resp.dragged()) {
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
                // 新的左栏宽度 = 当前宽度 + 鼠标位移；先吸到最近的整数格再夹住 - 拖的是「格」，
                // 不是像素（吸到最近格，左右拖动的手感才对称，横向拖拽条也是这一套）。
                let want = self.split_x + (pointer.x - grab - bar.min.x);
                self.split_x = snap_cells_round(want).clamp(PANE_MIN_W, Self::split_max_x(work));
            }
        }
        if resp.drag_stopped() {
            self.split_grab = None;
        }

        let painter = ui.painter().clone();
        painter.rect_filled(bar, egui::CornerRadius::ZERO, theme().splitter);
        // 只画左右两条 1px 竖线：上边那条正好和菜单栏分隔线同在一行、下边那条紧贴状态栏分隔线，
        // 自己再画就成了 2px 的堆叠，所以上下都不画，交给区域分隔线收边。
        vline_hard(&painter, bar.min.x, bar.min.y, bar.max.y, theme().ctrl_border);
        vline_hard(
            &painter,
            bar.max.x - LINE_W,
            bar.min.y,
            bar.max.y,
            theme().ctrl_border,
        );
    }

    /// 左栏 ->（图表1, 拖拽条, 图表2, 拖拽条, 图表3）。五块从上往下排，相邻两块共用边界那条 1px
    /// 网格线：后一块从「前一块的下边线」那一行开始，所以整列严丝合缝、每条边界都只有 1px，
    /// 而且全都落在窗口的 9px 网格线上（三块高度都是 cells(n)，拖拽条是 cells(1) = 10px）。
    /// 第三块一路铺到左栏底边（它的下沿就是状态栏上沿那条共用线），底下不留白条。
    pub(crate) fn left_rows(work: egui::Rect, kline_h: f32, vol_h: f32) -> [egui::Rect; 5] {
        let x0 = work.min.x;
        let w = work.width();
        let kline = egui::Rect::from_min_size(egui::pos2(x0, work.min.y), egui::vec2(w, kline_h));
        let bar1 = egui::Rect::from_min_size(
            egui::pos2(x0, kline.max.y - LINE_W),
            egui::vec2(w, HSPLIT_H),
        );
        let vol = egui::Rect::from_min_size(
            egui::pos2(x0, bar1.max.y - LINE_W),
            egui::vec2(w, vol_h),
        );
        let bar2 = egui::Rect::from_min_size(
            egui::pos2(x0, vol.max.y - LINE_W),
            egui::vec2(w, HSPLIT_H),
        );
        let ind = egui::Rect::from_min_max(
            egui::pos2(x0, bar2.max.y - LINE_W),
            egui::pos2(x0 + w, work.max.y),
        );
        [kline, bar1, vol, bar2, ind]
    }

    /// 把三块高度收口，都是整数格、各自不小于 PANE_H_MIN（三块同一套规矩，不分主图 / 副图）：
    ///   - 三块高度之和 = 可用高度 - LEFT_FIXED_H（两条拖拽条再减掉四条共用线）；
    ///   - 从下往上收口：图表3 先夹，图表2 再夹（各自保住下限），图表1 吃掉剩下的；
    ///   - 三块加起来正好等于可用高度，第三块铺到左栏底边，不留白条。
    /// 拖动（ui_hsplitter）和换窗口尺寸都从这儿过一道。
    pub(crate) fn clamp_left_heights(&mut self, work: egui::Rect) {
        let budget = (work.height() - LEFT_FIXED_H).max(3.0 * PANE_H_MIN);
        let min = PANE_H_MIN;
        let ind_max = (budget - 2.0 * min).max(min);
        self.ind_h = snap_cells_up(self.ind_h).clamp(min, ind_max);
        let vol_max = (budget - min - self.ind_h).max(min);
        self.vol_h = snap_cells_up(self.vol_h).clamp(min, vol_max);
        self.kline_h = budget - self.vol_h - self.ind_h;
    }

    /// 横向拖拽条：和竖向分割条同一套（纯色 + 1px 硬边），只是改成上下拖。
    /// 拖动按最近整数格吸附，它下面那块跟着变高、上面那块相应变矮，三块各自不小于最小高度。
    /// which 决定拖的是哪一块的高度（上面那条拖拽条改成交量区，下面那条改指标区），
    /// 拖出来的值都存 struct 字段（一定是整数格 cells(n) 的形状）。
    pub(crate) fn ui_hsplitter(
        &mut self,
        ui: &mut egui::Ui,
        bar: egui::Rect,
        work: egui::Rect,
        which: LeftSplit,
    ) {
        // 正在拖图表（平移）的时候整条让路：不抢拖动会话、也不显示调整光标 - 两者互不干扰。
        let chart_dragging = self.pan.is_some();
        let resp = ui.allocate_rect(
            bar,
            if chart_dragging { egui::Sense::hover() } else { egui::Sense::drag() },
        );
        if !chart_dragging && (resp.hovered() || resp.dragged()) {
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
                // 先吸到 8 的倍数再收口 - 拖的是「格」，块里画的东西跟着一起变。
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

        // 外观和竖向分割条一模一样：整条（cells(1) = 10px）铺底色，再压上下两条 1px 硬边 -
        // 这两条边就是它和上下两块共用的边界线，那两块自己不再画。
        let painter = ui.painter().clone();
        painter.rect_filled(bar, egui::CornerRadius::ZERO, theme().splitter);
        hline_hard(&painter, bar.min.x, bar.max.x, bar.min.y, theme().ctrl_border);
        hline_hard(&painter, bar.min.x, bar.max.x, bar.max.y - LINE_W, theme().ctrl_border);
    }

    /// 右栏控制面板：一列自绘按钮；缩放 / 重置直接改视图，其余按钮在状态栏回显一句。
    pub(crate) fn ui_panel(&mut self, ui: &mut egui::Ui, panel: egui::Rect) {
        let painter = ui.painter().clone();
        // 左边缘那 1px 是和分割条共用的网格线（分割条已经把它画成边框色了）：
        // 底色从线的右边开始铺，别把它盖掉 - 否则分割条的右边框就没了。
        let body = egui::Rect::from_min_max(
            egui::pos2(panel.min.x + LINE_W, panel.min.y),
            panel.max,
        );
        painter.rect_filled(body, egui::CornerRadius::ZERO, theme().panel_bg);

        // 1) 排位置：左边距和上边距都是 1 格；按钮从上往下排，相邻按钮共用中间那条网格线，
        //    所以步进是 grid_line(2) = 18px、按钮高是 cells(2) = 19px（宽度是开机量好的）。
        let mut rects = Vec::with_capacity(self.panel_buttons.len());
        let x = panel.min.x + grid_line(1);
        let mut y = panel.min.y + grid_line(1);
        for button in &self.panel_buttons {
            rects.push(egui::Rect::from_min_size(
                egui::pos2(x, y),
                egui::vec2(button.button.w, BTN_H),
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

        // 3) 画 + 收点击（内容 / 便签各复制一份小字符串：不能一边借 panel_buttons 一边调 &mut self）。
        for i in 0..rects.len() {
            let content = self.panel_buttons[i].button.content.clone();
            let tip = self.panel_buttons[i].button.tip.clone();
            let action = self.panel_buttons[i].action;
            let pressed = self.panel_press == Some(i);
            let label = self.panel_buttons[i].button.label();
            // draw_button 返回是否点击；点到了就按这个按钮的动作办。
            if self.draw_button(ui, &painter, rects[i], &content, pressed, &tip) {
                self.run_action(action, &label);
            }
        }
    }

    /// 控制面板的按钮表：内容 + 悬停便签 + 动作。宽度在这里一次量好（跟面板宽度无关）。
    /// 行情生成只留「重新生成」一个入口：换一个新种子生成一段新行情。
    pub(crate) fn build_panel_buttons(&mut self) {
        let items: [(ButtonContent, &str, PanelAction); 10] = [
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
                ButtonContent::IconText(IconKind::Cross, "重置视图".to_owned()),
                "缩放与平移都回到最初",
                PanelAction::ResetView,
            ),
            (
                ButtonContent::Text("打印当前状态".to_owned()),
                "把当前状态写到状态栏",
                PanelAction::PrintState,
            ),
            (
                ButtonContent::Text("网格测试".to_owned()),
                "叠加 9px 网格线，核对控件有没有落在网格线上",
                PanelAction::ToggleGrid,
            ),
            (
                ButtonContent::Text("切换主题".to_owned()),
                "在经典 / 暗色两套皮肤之间切换",
                PanelAction::ToggleTheme,
            ),
            (
                ButtonContent::Text("图表配色跟随系统".to_owned()),
                "K 线涨跌色重新跟当前皮肤走",
                PanelAction::FollowSystemColors,
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
    }

    /// 执行一个面板动作。缩放 / 平移都是直接改视图状态，下一帧 ui_chart 就按新值出图。
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
            PanelAction::ToggleGrid => {
                self.show_grid = !self.show_grid;
                self.status_msg = if self.show_grid {
                    "网格测试：开（界面上叠加 9px 网格线）".to_owned()
                } else {
                    "网格测试：关".to_owned()
                };
            }
            // 换皮肤：整套颜色 + 图表区系统级配色一起换（见 StockApp::switch_theme）。
            PanelAction::ToggleTheme => self.switch_theme(),
            PanelAction::FollowSystemColors => self.follow_system_chart_colors(),
            _ => {
                self.status_msg = format!("你点击了：{label}");
            }
        }
    }

    /// 面板上的放大 / 缩小：一格 +/-2 像素（见 scale_after），范围跟滚轮完全一样（3..=15 的奇数档）。
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
        // 数据换了一代：三块面板的指令缓存全部作废，下一帧重算。
        self.mark_panes_dirty();
        self.offset = 0;
        self.scale = BAR_SCALE_DEFAULT;
        self.crosshair = None;
        self.status_msg = format!("已生成新的随机行情（种子 {}）", self.bar_seed);
    }
}

