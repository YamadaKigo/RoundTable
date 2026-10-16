// ===========================================================================
//  模块 ui::chart：左栏图表（统一的一种「图表」，没有主图 / 副图之分）
// ===========================================================================
//  左栏三块一律是「图表」：每块挂一个指标（indicator::Indicator）。画什么全由指标说了算 -
//  同一份指标代码挂到哪一块上画出来都一样；界面这边只给几何：
//    标题栏 2 格 / 1px 线格 /（可选）状态带 / 1px 线格 / 绘图区 / 1px 线格 /（可选）时间带。
//  要哪几条带由指标自己声明（layout()，见 indicator::ChartLayout）。
//  指标一共六个钩子：compute 出行情数据（脏了才算、整段缓存）；layout / axis / time / status /
//  overlay 每帧现算（只产装饰，从缓存好的数据指令里读数，不碰行情数据）。
//  本文件每帧只做三件事：重放缓存、现算那四个钩子、把数据坐标翻成像素（replay）。
//  缩放 / 平移 / 鼠标移动只换映射，一次 compute 都不做。
//  十字光标：三块共用同一套 x 映射，竖线天然连成一条；横线 / 数值标签只画鼠标压着的那一块。
//  所有坐标都吸到像素中心（grid::px_center），尺寸都是 cells(n) 或 8 的倍数。
//  边框只落在网格线格里：相邻两段共用边界那条线格，所以任何一条边都只有 1px，不会叠成 2px。

use crate::app::*;
use crate::grid::*;
use crate::indicator::{ChartCtx, ChartLayout, DrawCmd};
use crate::ui::menu::*;

use crate::theme::*;
use eframe::egui;

/// 绘图区上下各留 8px（1 格）边距 - 所有图表一个口径，不管它是蜡烛、量柱还是 MACD。
pub(crate) const CHART_PAD: f32 = GRID;

// ===========================================================================
//  画一根蜡烛
// ===========================================================================

/// 画一根蜡烛：1px 竖影线 + 实体（上涨空心 / 下跌实心）。参数全是像素：
/// left = 实体左边缘（整数像素），bar_w = 实体像素宽（奇数），
/// y_open / y_close / y_high / y_low = 四价换算好的像素中心 y。
/// 影线的中心列 = left + (bar_w - 1) / 2：bar_w 是奇数时它才落在整整一列像素的中间，
/// 实体左右两边的像素列数严格相等（tests::wick_is_centered_in_the_body_for_every_scale 盯着这条）。
/// 参数多是因为它只管画：几何全在调用处算好。
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_candle(
    painter: &egui::Painter,
    left: f32,
    bar_w: f32,
    rising: bool,
    color: egui::Color32,
    y_open: f32,
    y_close: f32,
    y_high: f32,
    y_low: f32,
) {
    let x = px_center(left + (bar_w - LINE_W) / 2.0);
    painter.line_segment(
        [egui::pos2(x, y_high), egui::pos2(x, y_low)],
        egui::Stroke::new(LINE_W, color),
    );

    let top = (y_open - 0.5).round().min((y_close - 0.5).round());
    let height = ((y_open - 0.5).round() - (y_close - 0.5).round()).abs() + LINE_W;
    let rect = egui::Rect::from_min_size(egui::pos2(left, top), egui::vec2(bar_w, height));

    // 上涨空心：先铺满再挖掉内部（四边各留 1px 硬边，中间填绘图区底色）；
    // 下跌实心：整块纯色。宽度或高度不足 3 像素时画不出「空心」，一律退化成实心。
    let hollow = rising && bar_w >= 3.0 && height >= 3.0;
    painter.rect_filled(rect, egui::CornerRadius::ZERO, color);
    if hollow {
        let inner = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + LINE_W, rect.min.y + LINE_W),
            egui::pos2(rect.max.x - LINE_W, rect.max.y - LINE_W),
        );
        painter.rect_filled(inner, egui::CornerRadius::ZERO, theme().chart_bg);
    }
}

/// 画一根美国线：中间一竖是最高到最低（1px，落在实体正中那一列），
/// 左边那横从中点往左到实体左边缘 = 开盘价，右边那横从中点往右到右边缘 = 收盘价。
/// 参数口径和 paint_candle 一样：全是像素，几何在调用处算好。
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_ohlc(
    painter: &egui::Painter,
    left: f32,
    bar_w: f32,
    color: egui::Color32,
    y_open: f32,
    y_close: f32,
    y_high: f32,
    y_low: f32,
) {
    let center = px_center(left + (bar_w - LINE_W) / 2.0);
    painter.line_segment(
        [egui::pos2(center, y_high), egui::pos2(center, y_low)],
        egui::Stroke::new(LINE_W, color),
    );
    // 开 / 收那两横都从正中的那一列往外伸，宽度严格相等（实体宽是奇数，center 正好在中点）。
    let right = left + bar_w;
    hline_hard(painter, center, right, y_close, color);
    hline_hard(painter, left, center, y_open, color);
}

// ===========================================================================
//  几何：一块图表内部的分段 + 绘图区 / 纵轴
// ===========================================================================

/// 一块图表内部的竖直分段：标题栏 2 格 / 1px /（可选）状态带 / 1px / 绘图区 / 1px /
///（可选）时间带 / 1px / 图表窗自己的状态栏 2 格。
/// 每段都是「跨若干格」的段（矩形是 cells(rows)），相邻两段共用边界那条 1px 线格 -
/// 两个矩形在那一行上重叠 1px，所以线永远只有 1px，不可能叠成 2px。
/// 绘图区取「剩下的全部高度」（精确值，不再向下吸 8）：几条带正好把面板铺满，底下不留白条
/// （最下面那条是图表窗的状态栏，见 chart_bar）。
/// 面板高度本身就是整数格（cells(n)），所以绘图区高度自动是「格数 x9 再减掉 1px 线格」，边界全在网格线上。
/// 返回 (标题栏, 状态带, 绘图区, 时间带)；不需要的那段是 None（状态栏永远有，单独用 chart_bar 算）。
/// 要哪几条带由 layout 声明，所以同一份指标挂到哪一块上都切出一样的几段。
pub(crate) fn chart_rows(
    area: egui::Rect,
    layout: ChartLayout,
) -> (egui::Rect, Option<egui::Rect>, egui::Rect, Option<egui::Rect>) {
    let title = grid_band(area, 0, PANE_TITLE_ROWS);
    let mut row = PANE_TITLE_ROWS;
    let status = if layout.status_rows > 0 {
        let band = grid_band(area, row, layout.status_rows);
        row += layout.status_rows;
        Some(band)
    } else {
        None
    };
    // 绘图区从第 row 条网格线下面那 1px 起：那条线就是它和上一段共用的上边框。
    let main_y = area.min.y + grid_line(row) + LINE_W;
    // 底下的尾巴：最下面是状态栏（2 格），它上面（可选）是时间带。相邻两段共用边界那条 1px 线，
    // 所以总高度 = 两段各自的高度 - 一条共用的线（不这么算的话绘图区下沿会差 1px、落不在网格线上）。
    let tail = if layout.time_rows > 0 {
        cells(layout.time_rows) + CHART_BAR_H - LINE_W
    } else {
        CHART_BAR_H
    };
    let main_h = (area.max.y - main_y - tail).max(0.0);
    let main = egui::Rect::from_min_size(
        egui::pos2(area.min.x, main_y),
        egui::vec2(area.width(), main_h),
    );
    // 时间带整段起自主图下沿那条线：它的上边框就是那条线，两段共用。
    let time = if layout.time_rows > 0 {
        Some(egui::Rect::from_min_size(
            egui::pos2(area.min.x, main_y + main_h),
            egui::vec2(area.width(), cells(layout.time_rows)),
        ))
    } else {
        None
    };
    (title, status, main, time)
}

/// 图表窗自己那条状态栏（整块图表最下面一条，2 格）：缩放 / 平移 / 可见几根写在这儿。
/// 它是「窗口」的一部分 - 主窗口的状态栏只留帧率和操作回显（调试信息）。
pub(crate) fn chart_bar(area: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(area.min.x, area.max.y - CHART_BAR_H),
        egui::vec2(area.width(), CHART_BAR_H),
    )
}

/// 一块图表的绘图区 ->（绘图区, 右侧纵轴）：纵轴占最右边 AXIS_COLS 个内容格，
/// 两者共用中间那条网格线，各自尺寸都是整数格。绘图区左边缘再让开 1px：最左那一列是
/// 窗口外框的线格，不让开的话最左那根蜡烛会被黑边框啃掉一列。
pub(crate) fn frame_rects(area: egui::Rect) -> (egui::Rect, egui::Rect) {
    let edge = area.max.x - AXIS_W;
    let plot = egui::Rect::from_min_max(
        egui::pos2(area.min.x + LINE_W, area.min.y),
        egui::pos2(edge, area.max.y),
    );
    let axis = egui::Rect::from_min_max(egui::pos2(edge, area.min.y), area.max);
    (plot, axis)
}

// ===========================================================================
//  重放：把缓存的 DrawCmd（数据坐标）翻成像素
// ===========================================================================
//  每帧都走这一段，但一次 compute 都不做 - 屏幕坐标只在这里现算。
//  缩放 / 平移只改 offset / scale，映射参数跟着变，指令本身原封不动。

/// 一屏重放映射：plot 定 x，top / span_px / hi / lo 定 y（值越大越靠上）。
/// 各图表的值域不同，映射参数由调用方给。
pub(crate) struct CmdMap {
    plot: egui::Rect,
    /// 可见区间（全局索引，右开）：区间外的指令直接跳过。
    from: usize,
    to: usize,
    /// 一根 K 线的横向步进、实体像素宽（都是整数口径）。
    advance: f32,
    bar_w: f32,
    /// 值 -> 像素行：y = top + (hi - v) / (hi - lo) * span_px。
    top: f32,
    span_px: f32,
    hi: f64,
    lo: f64,
}

impl CmdMap {
    /// 全局下标在不在可见区间里。
    fn visible(&self, i: usize) -> bool {
        i >= self.from && i < self.to
    }

    /// 全局下标 -> 实体左边缘（整数像素）。
    fn x_left(&self, i: usize) -> f32 {
        (self.plot.min.x + i.saturating_sub(self.from) as f32 * self.advance).round()
    }

    /// 折线 / 蜡烛影线的 x：实体正中那一列（和蜡烛影线同一列，几层严格对齐）。
    fn line_x(&self, i: usize) -> f32 {
        self.x_left(i) + (self.bar_w - LINE_W) / 2.0
    }

    /// 可以带小数的 K 线下标 -> 屏幕 x（十字光标竖线用：跟着鼠标走，不吸附到 K 线中心）。
    fn line_at(&self, i: f64) -> f32 {
        self.plot.min.x + (i - self.from as f64) as f32 * self.advance
    }

    /// 值域跨度（塌了就取一个极小值，避免除零）。
    fn span_data(&self) -> f64 {
        (self.hi - self.lo).max(f64::EPSILON)
    }

    /// 值 -> 像素中心行（值域之外夹在上下边界）。
    fn y(&self, v: f64) -> f32 {
        let t = ((self.hi - v) / self.span_data()).clamp(0.0, 1.0) as f32;
        px_center(self.top + t * self.span_px)
    }
}

/// 一块图表的像素几何：四段带 + 重放映射。replay 拿它把数据坐标翻成像素。
pub(crate) struct ChartGeom {
    plot: egui::Rect,
    axis: egui::Rect,
    status: Option<egui::Rect>,
    time: Option<egui::Rect>,
    map: CmdMap,
    /// 绘图区下沿那条边界线所在的像素行（有时间带 = 共用的那条线；没时间带 = Pane 自己的
    /// 下边框线）。十字光标那两块标签的下边框一律压在它上面 - 铁律：不许 2px 线叠加。
    bottom: f32,
    /// 这块图表自己的 9px 网格原点：窗口左上角（内容区让开一条标题栏 = 2 格，
    /// 差整数格、和窗口左上角同余，所以拿内容区左上角当原点，吸附结果一模一样）。
    /// 文字落点 / 标签吸附全按它算 - 窗口 1px 1px 地挪，里头的东西跟着一起挪，不会在窗口里漂。
    origin: egui::Pos2,
}

/// 0xRRGGBB -> Color32。
fn color32(c: u32) -> egui::Color32 {
    egui::Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

/// 扫一遍可见区间里的指令，求数值上下界（纵轴范围）。
/// HLine 不参与 - 它是参考线，不能让它把值域撑开；可见区间里一条数据指令都没有就返回 (0, 0)。
pub(crate) fn pane_range(cmds: &[DrawCmd], from: usize, to: usize) -> (f64, f64) {
    let vis = |i: usize| i >= from && i < to;
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for cmd in cmds {
        match cmd {
            DrawCmd::Candle { i, h, l, .. } => {
                if vis(*i) {
                    lo = lo.min(*l);
                    hi = hi.max(*h);
                }
            }
            DrawCmd::Bar { i, v0, v1, .. } => {
                if vis(*i) {
                    lo = lo.min(*v0).min(*v1);
                    hi = hi.max(*v0).max(*v1);
                }
            }
            DrawCmd::Ohlc { i, h, l, .. } => {
                if vis(*i) {
                    lo = lo.min(*l);
                    hi = hi.max(*h);
                }
            }
            DrawCmd::Polyline { pts, .. } => {
                for (i, p) in pts {
                    if vis(*i) {
                        lo = lo.min(*p);
                        hi = hi.max(*p);
                    }
                }
            }
            DrawCmd::Line { i1, p1, i2, p2, .. } => {
                if vis(*i1) {
                    lo = lo.min(*p1);
                    hi = hi.max(*p1);
                }
                if vis(*i2) {
                    lo = lo.min(*p2);
                    hi = hi.max(*p2);
                }
            }
            // 叠加线（MA / EMA）只叠在 K 线上，不参与值域 - 250 日均线跑在可见区间之外是常事，
            // 让它算进来会把 K 线压扁。
            DrawCmd::Overlay { .. }
            | DrawCmd::HLine { .. }
            | DrawCmd::VLine { .. }
            | DrawCmd::Text { .. }
            | DrawCmd::AxisText { .. }
            | DrawCmd::TimeText { .. }
            | DrawCmd::StatusText { .. }
            | DrawCmd::StatusField { .. } => {}
        }
    }
    if lo > hi {
        (0.0, 0.0)
    } else {
        (lo, hi)
    }
}

// ===========================================================================
//  图表的交互、十字光标与出图
// ===========================================================================

impl StockApp {
    /// 一块图表在可见区间里的纵轴值域：先扫数据指令的上下界，再交指标自己修正
    /// （MACD 就把它掰成以 0 对称）。交互 / 十字光标 / 出图三处都用它，算法必须一致。
    pub(crate) fn pane_view_range(&self, pane: usize, from: usize, to: usize) -> (f64, f64) {
        let (lo, hi) = pane_range(&self.panes[pane].cmds, from, to);
        self.panes[pane].indicator.value_range(lo, hi)
    }

    /// 交互：滚轮缩放（锚在鼠标那一根上）+ 左键拖拽平移。三块图表共用同一个 offset / scale，
    /// 所以在哪一块的绘图区里操作都是同一个视图；标题栏 / 状态带 / 拖拽条上不接这些操作。
    /// bars 是「不许算图表平移」的那几条拖拽条（拖拽条取消后传空切片；这一套保留备用）：
    /// 按在它们上面永远不算图表平移 - 拖拽条和图表区互不干扰，谁也别抢谁的拖动会话。
    pub(crate) fn chart_interact(
        &mut self,
        ui: &egui::Ui,
        plots: &[egui::Rect],
        bars: &[egui::Rect],
    ) {
        // 已经在拖别的东西（窗口 / 分割条 / 拖拽条 / 浮动面板）时，图表整块让路：
        // 不管两边谁先抢到会话，只要别的拖拽会话已经开起来，图表平移立刻收手。
        // （按下那一帧拖拽条还没轮到处理，图表可能先把 pan 抢了；所以这里每帧都清一次，
        //  真动起来之前就还回去了 - 左键左右拖拖拽条不会再带着图表跑。）
        if self.ui_drag_active() {
            self.pan = None;
            return;
        }

        // 鼠标压在哪一块绘图区里（多块图表窗各有各的绘图区，只有压着的那一块算数）。
        let hover = ui.input(|i| i.pointer.hover_pos());
        let over_plot = hover.and_then(|p| {
            plots
                .iter()
                .position(|r| r.contains(p))
                .map(|i| (i, p))
        });

        // 滚轮：鼠标压在绘图区里、且没压在浮动面板上才算数；缩放锚点取压着的那一块的左边。
        let notches = self.take_wheel_notches(ui, over_plot.map(|(_, p)| p));
        if notches != 0 {
            if let Some((i, p)) = over_plot {
                self.zoom_around(p.x - plots[i].min.x, notches);
            }
        }

        // 左键拖拽平移：在哪一块的绘图区里按下都算；记下按下那一刻的鼠标 x 和 offset，
        // 每帧按总位移反算，拖到边界被钳住之后再往回拖，视图不会飘。
        let bottom = ui.max_rect().max.y;
        let (pressed, down, press_pos, latest) = ui.input(|i| {
            (
                i.pointer.primary_pressed(),
                i.pointer.primary_down(),
                i.pointer.interact_pos(),
                i.pointer.latest_pos(),
            )
        });
        // 正在拖窗口 / 拖分割条 / 拖横向拖拽条时，图表不许跟着平移：两者互不干扰
        //（拖那些条的时候按下的焦点属于那条，不该再开一个图表平移）。
        if pressed {
            if let Some(p) = press_pos {
                if plots.iter().any(|r| r.contains(p))
                    && !bars.iter().any(|r| r.contains(p))
                    && !self.floating_at(p, bottom)
                {
                    self.pan = Some((p.x, self.offset));
                }
            }
        }
        if down {
            if let (Some((start_x, start_offset)), Some(p)) = (self.pan, latest) {
                let bars = (((p.x - start_x) / bar_advance(self.scale)) as f64).round();
                self.offset = (start_offset as f64 - bars).max(0.0) as usize;
            }
        } else {
            self.pan = None;
        }
    }

    /// 十字光标：鼠标压在哪一块的绘图区里。竖线在三块里都画（共用 x 映射，天然连成一条）；
    /// 横线 + 数值标签只画鼠标压着的那一块（别的块那根鼠标 y 不在它值域里）。
    /// 拖拽平移时鼠标滑出绘图区也继续显示（坐标夹回最近的那一块），十字线不会闪。
    pub(crate) fn update_crosshair(&mut self, ui: &egui::Ui, plots: &[egui::Rect]) {
        // 正在拖窗口 / 拖分割条 / 拖横向拖拽条时，图表区整块不响应十字光标：
        // 鼠标这时候扫过绘图区也不会冒十字线，拖拽条和图表区互不干扰。
        if self.ui_drag_active() {
            self.crosshair = None;
            return;
        }
        let hover = ui.input(|i| i.pointer.hover_pos());
        let dragging = self.pan.is_some();
        let point = hover.or_else(|| {
            if dragging {
                ui.input(|i| i.pointer.latest_pos())
            } else {
                None
            }
        });
        let Some(p) = point else {
            self.crosshair = None;
            return;
        };
        // 鼠标压在哪一块的绘图区里；拖拽中滑出去了就归到纵向最近的那一块。
        let pane = match plots.iter().position(|r| r.contains(p)) {
            Some(i) => i,
            None if dragging => plots
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    (p.y - a.center().y)
                        .abs()
                        .partial_cmp(&(p.y - b.center().y).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map_or(0, |(i, _)| i),
            None => {
                self.crosshair = None;
                return;
            }
        };
        // 压在某个浮动面板上（面板浮在图表上面）就不算。
        if !dragging && self.floating_at(p, ui.max_rect().max.y) {
            self.crosshair = None;
            return;
        }
        if self.visible == 0 {
            self.crosshair = None;
            return;
        }

        let plot = plots[pane];
        let from = self.offset;
        let to = (from + self.visible).min(self.bars.len());
        let advance = bar_advance(self.scale);
        let top = plot.min.y + CHART_PAD;
        let span_px = (plot.max.y - CHART_PAD - top)
            .max(LINE_W)
            .min((self.panes[pane].height as f32).max(LINE_W));
        let (lo, hi) = self.pane_view_range(pane, from, to);
        let span = (hi - lo).max(f64::EPSILON);

        // 鼠标坐标是浮点，先吸附到像素中心再画：否则 1px 的线会横跨两列像素、被渲染成 2px 灰边。
        let x = px_center(p.x.clamp(plot.min.x, plot.max.x - LINE_W));
        let y = px_center(p.y.clamp(top, top + span_px));
        let fpos = (from as f32 + (x - plot.min.x) / advance).max(0.0);
        let index = (fpos.floor() as usize).min(self.bars.len().saturating_sub(1));
        // 横线数值 = 映射的逆运算（同一个 top / span、同一个 1px 修正），
        // 所以标签上的数字和横线所在的高度严丝合缝。
        let t = ((y - top) / span_px).clamp(0.0, 1.0) as f64;
        self.crosshair = Some(Crosshair {
            value: hi - span * t,
            index,
            fpos: fpos as f64,
            pane,
        });
    }

    /// 画一块图表的内容：整块窗口的标题栏（含关闭 / 最小化键）在它上面，由 ui::winmgr 画；
    /// 这儿从「图表栏」那一条横栏起算 - 几何按指标声明的几条带切，重放缓存 + 现算四个钩子。
    /// 这儿不认识任何具体指标 - 加指标只改 src/indicator/，这个文件一行都不用动。
    pub(crate) fn ui_chart(&mut self, ui: &mut egui::Ui, pane: usize, area: egui::Rect) {
        let painter = ui.painter().clone();
        let layout = self.panes[pane].indicator.layout();
        let (title, status, main, time) = chart_rows(area, layout);
        let (plot, axis) = frame_rects(main);

        // 铺底：整块工作区白 -> 绘图区白 -> 最右那条纵轴灰。
        // 纵轴灰只铺到绘图区下沿为止：时间带我这边整条都是白的（右边不再拖一条灰尾巴）。
        painter.rect_filled(area, egui::CornerRadius::ZERO, theme().work_bg);
        painter.rect_filled(plot, egui::CornerRadius::ZERO, theme().chart_bg);
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(axis.min.x, main.min.y - LINE_W),
                egui::pos2(axis.max.x, main.max.y),
            ),
            egui::CornerRadius::ZERO,
            theme().axis_bg,
        );
        // 绘图区上沿那条线（和上一段共用）+ 时间带上沿那条线（和绘图区下沿共用）：一条 1px。
        hline_hard(&painter, area.min.x, area.max.x, main.min.y - LINE_W, theme().line);
        if let Some(t) = time {
            hline_hard(&painter, area.min.x, area.max.x, t.min.y, theme().line);
        }
        // 图表栏最后画：底色盖住上面的窗口白，几个按钮和下沿那条线也在这一层。
        // （关闭 / 最小化键在整块窗口的标题栏上，见 ui::winmgr，不在这儿。）
        self.ui_pane_title(ui, pane, title);
        // UI工程测试：这块图表窗里的几条带各报一笔（按窗口自己那套网格判对齐）。
        self.probe("图表栏", title);
        if let Some(s) = status {
            self.probe("图表状态带", s);
        }
        self.probe("绘图区", main);
        if let Some(t) = time {
            self.probe("时间带", t);
        }
        self.probe("图表窗状态栏", chart_bar(area));
        // 图表窗自己那条状态栏（整块最下面一条）：缩放 / 平移 / 可见几根写在这儿 -
        // 主窗口的状态栏只留帧率和操作回显（见 app::StockApp::status_segments）。
        let bar_segs = self.chart_status_segments();
        let bar = chart_bar(area);
        self.paint_status_bar(&painter, bar, &bar_segs);

        // 高度记下来、涨跌色可能刚被改过、缓存脏了就重算、一屏几根现算。
        self.panes[pane].height = main.height() as i32;
        self.sync_pane_palette();
        self.ensure_pane_computed(pane);
        self.clamp_view(plot.width());
        let visible = self.visible;
        if visible == 0 {
            return;
        }

        let from = self.offset;
        let to = (from + visible).min(self.bars.len());
        let advance = bar_advance(self.scale);
        let bar_w = snap_scale(self.scale) as f32;
        let (lo, hi) = self.pane_view_range(pane, from, to);
        let top = plot.min.y + CHART_PAD;
        let span_px = (plot.max.y - CHART_PAD - top)
            .max(LINE_W)
            .min((self.panes[pane].height as f32).max(LINE_W));
        let time_step = self.time_step(advance);
        let map = CmdMap {
            plot,
            from,
            to,
            advance,
            bar_w,
            top,
            span_px,
            hi,
            lo,
        };
        let geom = ChartGeom {
            plot,
            axis,
            status,
            time,
            map,
            // 绘图区下沿那条「边界线」所在的像素行，十字光标的标签下边框要压在它上面：
            //   - 有时间带：就是绘图区和时间带共用的那条线 = time.min.y（= main.max.y）；
            //   - 没有时间带（比如最下面那块 MACD）：是这块 Pane 自己的下边框线，也就是
            //     main.max.y 往上 1px（main.max.y 是「下一个像素行」，不是边框线本身）。
            // 不分开算的话，没时间带的那块会把标签下边框画进状态栏里，和边框线叠成 2px。
            bottom: time.map_or(main.max.y - LINE_W, |t| t.min.y),
            // 这块窗口自己那套网格的原点：拿内容区左上角当原点（和窗口左上角差整数格，同余）。
            origin: area.min,
        };

        // 每帧现算的四个钩子：从缓存好的数据指令里读数。
        // 画字要 &mut self，所以先把数据指令整份挪出来（Vec 只是换个主人），画完再放回去。
        let data = std::mem::take(&mut self.panes[pane].cmds);
        let (axis_cmds, time_cmds, status_cmds, overlay_cmds) = {
            let ctx = ChartCtx {
                bars: &self.bars,
                cmds: &data,
                from,
                to,
                visible,
                advance,
                time_step,
                lo,
                hi,
                cursor_bar: self.crosshair.map(|c| c.index),
                cursor_index: self.crosshair.map(|c| c.fpos),
                cursor_value: self.crosshair.filter(|c| c.pane == pane).map(|c| c.value),
            };
            let ind = &self.panes[pane].indicator;
            (
                ind.axis(&ctx),
                ind.time(&ctx),
                ind.status(&ctx),
                ind.overlay(&ctx),
            )
        };

        // 重放顺序 = 绘制顺序：轴（刻线 / 数值）-> 数据 -> 轴左边那条线 -> 时间 -> 状态 -> 十字光标。
        self.replay_cmds(&painter, &geom, &axis_cmds);
        self.replay_cmds(&painter, &geom, &data);
        vline_hard(&painter, axis.min.x, axis.min.y, axis.max.y, theme().line);
        self.replay_cmds(&painter, &geom, &time_cmds);
        self.replay_cmds(&painter, &geom, &status_cmds);
        self.replay_cmds(&painter, &geom, &overlay_cmds);
        self.panes[pane].cmds = data;
    }

    /// 重放一串指令：把数据坐标翻成像素画出来。屏幕坐标只在这里现算。
    fn replay_cmds(&mut self, painter: &egui::Painter, geom: &ChartGeom, cmds: &[DrawCmd]) {
        let map = &geom.map;
        // 状态带里那几项从左往右接着排：每一项都是整数格（cells(n)），相邻两项共用边界那条 1px
        // 网格线，所以每两项之间的点状虚线分割线正好落在网格线上。文字左右各留 1 格

        //（宽按 TEXT_PAD_X 算、落点用 TEXT_INSET_X，见 grid.rs）。
        let mut sx = geom.status.map_or(0.0, |r| r.min.x);
        let mut first = true;
        for cmd in cmds {
            match cmd {
                DrawCmd::Candle { i, o, h, l, c, color } => {
                    if !map.visible(*i) {
                        continue;
                    }
                    paint_candle(
                        painter,
                        map.x_left(*i),
                        map.bar_w,
                        c >= o,
                        color32(*color),
                        map.y(*o),
                        map.y(*c),
                        map.y(*h),
                        map.y(*l),
                    );
                }
                DrawCmd::Bar { i, v0, v1, color, hollow } => {
                    if !map.visible(*i) {
                        continue;
                    }
                    let left = map.x_left(*i);
                    let (a, b) = (map.y(*v0), map.y(*v1));
                    let rect = egui::Rect::from_min_max(
                        egui::pos2(left, a.min(b)),
                        egui::pos2(left + map.bar_w, a.max(b)),
                    );
                    painter.rect_filled(rect, egui::CornerRadius::ZERO, color32(*color));
                    // 空心柱（成交量里上涨那一档）：四边各留 1px 硬边，中间填绘图区底色 -
                    // 和上涨蜡烛同一个做法；太窄太矮画不出「空心」就保持实心。
                    if *hollow && map.bar_w >= 3.0 && rect.height() >= 3.0 {
                        let inner = egui::Rect::from_min_max(
                            egui::pos2(rect.min.x + LINE_W, rect.min.y + LINE_W),
                            egui::pos2(rect.max.x - LINE_W, rect.max.y - LINE_W),
                        );
                        painter.rect_filled(inner, egui::CornerRadius::ZERO, theme().chart_bg);
                    }
                }
                DrawCmd::Ohlc { i, o, h, l, c, color } => {
                    if !map.visible(*i) {
                        continue;
                    }
                    paint_ohlc(
                        painter,
                        map.x_left(*i),
                        map.bar_w,
                        color32(*color),
                        map.y(*o),
                        map.y(*c),
                        map.y(*h),
                        map.y(*l),
                    );
                }
                DrawCmd::Polyline { pts, color, w } | DrawCmd::Overlay { pts, color, w } => {
                    debug_assert!(*w >= 1, "折线宽度至少 1px");
                    let mut run: Vec<(f32, f32)> = Vec::new();
                    for (i, p) in pts {
                        if map.visible(*i) {
                            run.push((map.line_x(*i), map.y(*p)));
                        }
                    }
                    // 只有两个点以上才连得成线，单点直接丢掉。
                    if run.len() > 1 {
                        polyline_hard(painter, &run, color32(*color));
                    }
                }
                DrawCmd::Line { i1, p1, i2, p2, color, w } => {
                    debug_assert!(*w >= 1, "直线宽度至少 1px");
                    painter.line_segment(
                        [
                            egui::pos2(map.line_x(*i1), map.y(*p1)),
                            egui::pos2(map.line_x(*i2), map.y(*p2)),
                        ],
                        egui::Stroke::new(*w as f32, color32(*color)),
                    );
                }
                DrawCmd::HLine { p, color, w } => {
                    debug_assert!(*w >= 1, "参考线宽度至少 1px");
                    // 值域覆盖到它才画（值域外的水平参考线不画出来）。
                    if *p >= map.lo && *p <= map.hi {
                        hline_hard(
                            painter,
                            geom.plot.min.x,
                            geom.plot.max.x,
                            map.y(*p),
                            color32(*color),
                        );
                    }
                }
                DrawCmd::VLine { i, color, w } => {
                    debug_assert!(*w >= 1, "竖线宽度至少 1px");
                    vline_hard(
                        painter,
                        map.line_at(*i),
                        geom.plot.min.y,
                        geom.plot.max.y,
                        color32(*color),
                    );
                }
                DrawCmd::Text { i, p, s, color } => {
                    let pos = egui::pos2(map.line_x(*i), map.y(*p));
                    self.text(painter, pos, egui::Align2::CENTER_CENTER, s, color32(*color));
                }
                DrawCmd::AxisText { p, s, fg, bg } => {
                    let y = map.y(*p);
                    if *bg == 0 {
                        // 纵轴刻度数字：右对齐写在轴里（末尾停在轴右框那条线左边 1 格，
                        // 落点由 axis_tick_x 按窗口自己的右沿算，1px 1px 地跟着窗口缩）。
                        self.text(
                            painter,
                            egui::pos2(Self::axis_tick_x(geom.axis), y),
                            egui::Align2::RIGHT_CENTER,
                            s,
                            color32(*fg),
                        );
                    } else {
                        // 十字光标的数值标签：衬一块带 1px 边框的底色块，盖住底下的刻度数字。
                        // 绘图区矮到放不下标签就干脆不画 - 宁可少一个标签，也不让文字压到别的元素上。
                        if geom.plot.height() < CROSS_LABEL_H {
                            continue;
                        }
                        let w = self.cross_label_w(s);
                        let x0 = Self::cross_axis_label_x(geom.axis, w);
                        // 上沿吸到网格线、整段夹进绘图区上下两条边框线之间：牌子上下两条边框各占
                        // 1px 网格线格，靠到绘图区上/下沿时是和那条边框线重合，绝不再叠出 2px。
                        let y0 = floating_label_top(
                            y,
                            CROSS_LABEL_ROWS,
                            geom.plot.min.y - LINE_W,
                            geom.bottom,
                            geom.origin.y,
                        );
                        self.label_box(painter, s, egui::pos2(x0, y0), w, color32(*bg), color32(*fg));
                    }
                }
                DrawCmd::TimeText { i, s, fg, bg } => {
                    if *bg == 0 {
                        // 时间带里的一格：贴在那一列正中；压到绘图区边缘的干脆不画。
                        let Some(band) = geom.time else {
                            continue;
                        };
                        let cx = map.line_x(*i);
                        let w = self.text_width(s);
                        if cx - w / 2.0 < geom.plot.min.x || cx + w / 2.0 > geom.plot.max.x - LINE_W {
                            continue;
                        }
                        self.text(
                            painter,
                            egui::pos2(cx, band.center().y),
                            egui::Align2::CENTER_CENTER,
                            s,
                            color32(*fg),
                        );
                    } else {
                        // 十字光标的索引标签：贴在绘图区下沿、竖线那一列的正中。
                        let w = self.cross_label_w(s);
                        let x0 =
                            Self::cross_time_label_x(geom.plot, map.line_x(*i), w, geom.origin.x);
                        // 贴在绘图区下沿：牌子上沿落在下沿往上 2 格那条网格线上，下边框正好压在这块
                        // 图表下沿那条边界线上（时间带的上沿 / Pane 自己的下边框，同一条 1px 线），
                        // 绝不在它下面再叠出 2px 的横线（见 ChartGeom::bottom）。
                        self.label_box(
                            painter,
                            s,
                            egui::pos2(x0, geom.bottom - PITCH * CROSS_LABEL_ROWS as f32),
                            w,
                            color32(*bg),
                            color32(*fg),
                        );
                    }
                }
                DrawCmd::StatusText { s, color } => {
                    let Some(band) = geom.status else {
                        continue;
                    };
                    let w = snap_cells_up(self.text_width(s) + 2.0 * TEXT_PAD_X);
                    if sx + w > band.max.x {
                        break;
                    }
                    self.status_divider(painter, band, sx, first);
                    self.text(
                        painter,
                        egui::pos2(text_left_x(sx, geom.origin.x), band.center().y),
                        egui::Align2::LEFT_CENTER,
                        s,
                        color32(*color),
                    );
                    sx += w - LINE_W;
                    first = false;
                }
                DrawCmd::StatusField { label, value, tmpl, color } => {
                    let Some(band) = geom.status else {
                        continue;
                    };
                    // 项宽按固定的「模板」量（不按当前值），数值位数变化时后面几项不会抖。
                    let w = snap_cells_up(
                        self.text_width(label)
                            + GRID
                            + self.text_width(tmpl)
                            + 2.0 * TEXT_PAD_X,
                    );
                    if sx + w > band.max.x {
                        break;
                    }
                    self.status_divider(painter, band, sx, first);
                    self.text(
                        painter,
                        egui::pos2(text_left_x(sx, geom.origin.x), band.center().y),
                        egui::Align2::LEFT_CENTER,
                        label,
                        theme().text,
                    );
                    self.text(
                        painter,
                        egui::pos2(
                            text_right_x(sx + w - LINE_W, geom.origin.x),
                            band.center().y,
                        ),
                        egui::Align2::RIGHT_CENTER,
                        value,
                        color32(*color),
                    );
                    sx += w - LINE_W;
                    first = false;
                }
            }
        }
    }

    /// 状态带里两项之间的点状虚线分割线：落在两项共用的那条网格线上（#999999、1px 点状）。
    /// 第一项不画（它左边就是面板边框）；线只在带的内容行里画，上下各让开那条 1px 边框线。
    fn status_divider(&self, painter: &egui::Painter, band: egui::Rect, x: f32, first: bool) {
        if first {
            return;
        }
        vline_dotted(painter, x, band.min.y + LINE_W, band.max.y - LINE_W, theme().line);
    }

    /// 一个浮动小标签：纯色底 + 1px 硬边 + 居中文字，位置尺寸都落在整数像素上。
    fn label_box(
        &mut self,
        painter: &egui::Painter,
        text: &str,
        pos: egui::Pos2,
        w: f32,
        bg: egui::Color32,
        fg: egui::Color32,
    ) {
        let rect = egui::Rect::from_min_size(pos, egui::vec2(w, CROSS_LABEL_H));
        tag_box(painter, rect, bg, theme().ctrl_border);
        self.text(painter, rect.center(), egui::Align2::CENTER_CENTER, text, fg);
    }

    /// 图表栏那几个按钮（就是这条菜单栏的「项」，从左往右）：认叠加的（K 线图）多一个「叠加」，
    /// 排在「切换」左边；别的指标只有「切换 / 代码」。
    pub(crate) const PANE_TABS_KLINE: [&'static str; 3] = ["叠加", "切换", "代码"];
    pub(crate) const PANE_TABS_PLAIN: [&'static str; 2] = ["切换", "代码"];

    /// 这块图表该有哪几个按钮：只有认叠加的（K 线图两种）才有「叠加」。
    fn pane_title_tabs(&self, pane: usize) -> &'static [&'static str] {
        if self.panes[pane].indicator.supports_overlay() {
            &Self::PANE_TABS_KLINE
        } else {
            &Self::PANE_TABS_PLAIN
        }
    }

    /// 图表栏那几个按钮的矩形：从右往左排（最右边那个贴着 Pane 右边框），相邻两个共用中间
    /// 那条竖线（步进 = 宽 - 1px）；高就是标题栏那一段。排版整个交给菜单栏那一份
    ///（ui::menu 的 menu_item_rects，右对齐）- 图表栏就是一条菜单栏，只是按钮往右靠。
    /// 宽度由调用方按文字量好（tab_width：文字宽 + 左右各 1 格，向上吸到整数格，保底 3 格）。
    /// widths 从左往右对应；返回顺序从左往右（K 线图是 [叠加, 切换, 代码]，别的图是 [切换, 代码]）。
    /// 生产代码那条路走 ui::menu 的 menu_bar_ui（它自己算同一套矩形），这条口子是给测试核对排版的。
    #[allow(dead_code)]
    pub(crate) fn pane_button_rects(title: egui::Rect, widths: &[f32]) -> Vec<egui::Rect> {
        menu_item_rects(title, widths, MenuAlign::Right)
    }

    /// 图表栏（图表窗内容区最上面那一条横栏，整块窗口的标题栏在它上面 - 见 ui::winmgr）：
    /// 它**就是一条菜单栏** - 只是按钮往右靠、左边多一段文字（指标名）。底色 / 项宽 / hover
    /// 高亮 / 项间竖线 / 下沿那条 1px 线 / 点击开合与 hover 换项的规矩，全走 ui::menu 的
    /// menu_bar_ui：主窗口菜单栏长什么样，它就长什么样（铁律：同类型的东西必须长一个样）。
    /// 「叠加」开合均线叠加菜单（只有 K 线图有，见 Indicator::supports_overlay）；「切换」开合指标
    /// 下拉面板（面板在每帧最后画，见 ui::panemenu）；「代码」打开源码查看窗。两个下拉菜单互斥 -
    /// 开一个就把另一个收起来（和主窗口菜单栏「同时只开一个」是同一个规矩）。
    /// 返回按钮的矩形（从左往右，下拉面板各拿自己那个当锚点）。
    pub(crate) fn ui_pane_title(
        &mut self,
        ui: &mut egui::Ui,
        pane: usize,
        title: egui::Rect,
    ) -> Vec<egui::Rect> {
        let tabs = self.pane_title_tabs(pane);
        let overlay_slot = self.panes[pane].indicator.supports_overlay().then_some(0);
        let switch_slot = usize::from(overlay_slot.is_some());
        let code_slot = switch_slot + 1;
        // 展开状态：0 = 叠加（只有 K 线图有）、接着是「切换」；「代码」不是下拉项，不占展开状态。
        let open = if self.overlay_menu == Some(pane) {
            overlay_slot
        } else if self.pane_menu == Some(pane) {
            Some(switch_slot)
        } else {
            None
        };
        let name = self.panes[pane].indicator.name().to_owned();
        let out = menu_bar_ui(
            self,
            ui,
            title,
            tabs,
            MenuAlign::Right,
            open,
            Some(&name),
            MenuBarStyle::PANE_BAR,
        );
        // 「代码」不是下拉项：点了就记一笔开源码编辑窗（真正打开在本帧末尾的 run_win_cmd -
        // 这会儿窗口管理器正被借走），不留一个「展开中」的状态。
        let mut open = out.open;
        if out.clicked == Some(code_slot) {
            self.request_win(WinCmd::OpenSource(pane));
            open = None;
        }
        // 展开状态照菜单栏那套落到这块图表的两个菜单上（点开 / 再点收起 / hover 换项都走这条）。
        if overlay_slot.is_some() && open == overlay_slot {
            self.overlay_menu = Some(pane);
            self.pane_menu = None;
        } else if open == Some(switch_slot) {
            self.pane_menu = Some(pane);
            self.overlay_menu = None;
        } else {
            self.overlay_menu = None;
            self.pane_menu = None;
        }
        // 锚点 = 对应的那个按钮（下拉面板挂在它下沿 / 右边缘跟它对齐）；顺手把这条图表栏
        //（= 这块图表的菜单栏）也记下来 - 鼠标移出面板和它，面板就收起（见 ui::panemenu）。
        if self.overlay_menu == Some(pane) {
            if let Some(i) = overlay_slot {
                self.overlay_menu_anchor = out.rects[i];
                self.overlay_menu_bar = title;
            }
        }
        if self.pane_menu == Some(pane) {
            self.pane_menu_anchor = out.rects[switch_slot];
            self.pane_menu_bar = title;
        }
        out.rects
    }

    /// 这一帧滚轮滚了几格（正 = 往上滚 = 放大）。只有鼠标在绘图区里、且没压在浮动面板上
    /// 才算数：在控制面板或别处滚滚轮，K 线不动，也不会把增量攒在字段里留着下次发作。
    /// 不足一格的零头（触控板那种按点上报的增量）存在 zoom_acc 里，下一帧接着凑。
    pub(crate) fn take_wheel_notches(&mut self, ui: &egui::Ui, hover: Option<egui::Pos2>) -> i32 {
        // 鼠标不在任何绘图区里、正压在某个浮动面板上（面板在绘图区上面）、或者正在拖别的东西
        // （拖窗口 / 拖分割条 / 拖横向拖拽条 / 拖浮动面板），都不收滚轮 - 拖拽条和图表互不干扰。
        let bottom = ui.max_rect().max.y;
        if hover.is_none() || self.ui_drag_active() || hover.is_some_and(|p| self.floating_at(p, bottom)) {
            self.zoom_acc = 0.0;
            return 0;
        }
        let mut acc = self.zoom_acc;
        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::MouseWheel { unit, delta, .. } = event {
                    // Windows 上一格就是 1 行：winit 把 WM_MOUSEWHEEL 的 +/-120 除以 120。
                    acc += match unit {
                        egui::MouseWheelUnit::Line => delta.y,
                        egui::MouseWheelUnit::Point => delta.y / WHEEL_POINTS_PER_NOTCH,
                        egui::MouseWheelUnit::Page => delta.y,
                    };
                }
            }
        });
        let whole = acc.trunc();
        self.zoom_acc = acc - whole;
        whole as i32
    }

    /// 锚点缩放：鼠标指着的那根 K 线，缩放前后必须停在同一屏幕 x 上（手感全靠这个）。
    ///   mouse_bar  = offset + (mouse_x - 绘图区左边) / 旧步进
    ///   new_offset = mouse_bar - (mouse_x - 绘图区左边) / 新步进
    /// 最后把这根 K 线的下标取整 - offset 是整数索引，只能整根整根地挪。
    pub(crate) fn zoom_around(&mut self, x_rel: f32, notches: i32) {
        let old = snap_scale(self.scale);
        let new = scale_after(old, notches);
        if new == old {
            return;
        }
        let mouse_bar = self.offset as f32 + x_rel / bar_advance(old);
        self.scale = new;
        self.offset = (mouse_bar - x_rel / bar_advance(new)).round().max(0.0) as usize;
    }

    /// 重算一屏放几根，并把 offset 夹回 [0, 总根数 - 可见根数]：边界处钳住，
    /// 既拖不出左边的空白，也不会让最后一根后面露出空槽。
    pub(crate) fn clamp_view(&mut self, plot_w: f32) {
        self.visible = visible_bars(plot_w, self.scale).min(self.bars.len());
        self.offset = self.offset.min(self.bars.len().saturating_sub(self.visible));
    }

    /// 时间轴标注间隔（单位：根）：让相邻标签中心至少隔 48px（6 格）。
    /// 用「MM-DD」和跨年那种「YY-MM-DD」里更宽的一个来算，窄屏 / 大 scale 下也不会挤成一团。
    pub(crate) fn time_step(&mut self, advance: f32) -> usize {
        let label_w = self.text_width("00-00").max(self.text_width("00-00-00"));
        let need = (label_w + GRID).max(TIME_LABEL_GAP_MIN);
        ((need / advance).ceil() as usize).max(1)
    }

    /// 浮动标签的宽度 = 文字宽 + 左右各 TEXT_PAD_X，向上吸到整数格（最小 2 格）；
    /// 上限 VALUE_COLS_MAX + 2 格（就是纵轴那 9 格宽）- 数值再长，标签也越不出轴那一列。
    pub(crate) fn cross_label_w(&mut self, text: &str) -> f32 {
        snap_cells_up(self.text_width(text) + 2.0 * TEXT_PAD_X)
            .max(cells(2))
            .min(cells(VALUE_COLS_MAX + 2))
    }

    /// 纵轴刻度数字的落点（Align2::RIGHT_CENTER 的 x）：轴右框那条线往左 1 格 - 末尾那个字
    /// 因而落在完整的内容格里（内容格 = [9k+1, 9k+9)），右边正好留 1 格空档。
    /// 【一律按窗口自己的右沿算，1px 1px 地走】窗口可以 1px 1px 地缩，靠右的元素就得跟着
    /// 1px 1px 地挪 - 要是拿 9px 网格去吸（grid_floor 那一套），窗口宽 1px 地变时数字要等
    /// 满一格才跳一次，看上去就是「窗口缩放时元素 8px 8px 地动」。
    pub(crate) fn axis_tick_x(axis: egui::Rect) -> f32 {
        axis.max.x - LINE_W - GRID
    }

    /// 十字光标的数值标签（轴里那块带 1px 边框的小牌子）的左边 x：牌子右边缘停在刻度数字
    /// 右边那条空档上（也就是轴右框那条线往左 1 格），宽度就是 cross_label_w（cells 形状）。
    /// 和刻度数字一样一律按窗口自己的右沿算、1px 1px 地走；牌子比轴还宽（数值位数太多）就
    /// 贴住轴的左框，宁可越出去一点也不越格。
    pub(crate) fn cross_axis_label_x(axis: egui::Rect, w: f32) -> f32 {
        let right = Self::axis_tick_x(axis) - w;
        clamp_soft(right, axis.min.x, right)
    }

    /// 时间带里那个索引标签（贴绘图区下沿）的左边 x：先按十字光标那一列居中，再吸到最近的
    /// 竖网格线（牌子左边必须是网格线），最后夹在绘图区里。
    /// 吸附的原点是这块窗口自己的网格（origin）- 窗口挪一格，牌子跟着挪一格，相对位置一个
    /// 像素都不变；夹的两条边则是绘图区自己的左右沿，按 1px 算（窗口 1px 1px 地缩时，
    /// 贴边的那块牌子才会跟着 1px 1px 地挪，而不是等满一格才跳）。
    pub(crate) fn cross_time_label_x(plot: egui::Rect, cx: f32, w: f32, origin: f32) -> f32 {
        clamp_soft(
            grid_snap((cx - w / 2.0).round(), origin),
            plot.min.x - LINE_W,
            plot.max.x - w - GRID,
        )
    }
}
