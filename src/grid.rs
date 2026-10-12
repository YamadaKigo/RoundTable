// ===========================================================================
//  模块 grid：度量常数 + 像素级绘制工具
// ===========================================================================
//  8px 内容格 + 1px 网格线格（节距 9px）的全部常数，以及只碰整数像素的绘制原语。
//  全工程算尺寸、算坐标都走这里，不许在别处自己乘 9。
//  本文件 = 拆分前那份单文件里的「二、度量」+「四、像素级绘制工具」两节。
use eframe::egui;

// ===========================================================================
//  二、度量：8px 内容格 + 1px 网格线格（节距 9px）
// ===========================================================================
//  本项目的底层布局模型（动布局之前必读）：
//    · 画布由「内容格」和「网格线格」交替铺成：内容格 8px（GRID），网格线格 1px（LINE_W），
//      所以节距 PITCH = GRID + LINE_W = 9px。
//    · 所有坐标都落在网格线上：第 i 条网格线的坐标 = i * PITCH。控件本体只占内容格，
//      那 1px 边框画在两端（以及相邻控件之间）的网格线格里 —— 数下来一个跨 n 格的控件
//      占「n 个内容格 + n+1 条网格线格」，尺寸就是 cells(n) = 9n + 1。
//    · 相邻控件共用中间那条网格线格（两者的矩形在那一列/行上重叠 1px）：各自尺寸照样是
//      cells(n)、完全相等，中间也永远只有 1px，不可能出现两条线堆叠成 2px。
//    · 不需要边框的网格线格就留空、显示底色。

pub(crate) const GRID: f32 = 8.0;
pub(crate) const LINE_W: f32 = 1.0;
pub(crate) const PITCH: f32 = GRID + LINE_W;

/// n 个内容格的跨度（含两端各 1px 网格线格）：9n + 1。控件宽高都用它。
pub(crate) const fn cells(n: i32) -> f32 {
    n as f32 * PITCH + LINE_W
}

/// 第 i 条网格线的坐标（以所在区域的左上角为原点）。
pub(crate) const fn grid_line(i: i32) -> f32 {
    i as f32 * PITCH
}

/// 第 i 个内容格的起点：网格线右边那 1px 之后。
pub(crate) const fn cell(i: i32) -> f32 {
    i as f32 * PITCH + LINE_W
}

/// 8px 的内容格没有正中间那一行（列）：中心落在半像素上，取中间两行（列）里的上一行当基线，
/// 保证符号还是整像素。28x28 这类偶数边长的框里，1px 的符号本来就不可能同时压中几何中心。
pub(crate) const CELL_MID: f32 = GRID / 2.0 - 1.0;

/// 把一段跨度向上吸到整数格：结果一定是 cells(n) 的形状。
pub(crate) fn snap_cells_up(v: f32) -> f32 {
    let n = ((v - LINE_W) / PITCH).ceil().max(0.0);
    n * PITCH + LINE_W
}

/// 把一段尺寸吸到 8 的倍数（「8px 内容格」口径，和颜色面板一套 —— 不是 cells() 的 9n+1）。
/// 左栏上下拆分这几块、信息栏里每一项的宽度都用这个：规格里它们的要求是「8 的倍数」。
pub(crate) fn snap8_up(v: f32) -> f32 {
    (v / GRID).ceil() * GRID
}

/// 同上，向下吸到 8 的倍数（给「剩余高度」用：宁可空一行，也不许超出可用高度）。
pub(crate) fn snap8_down(v: f32) -> f32 {
    (v / GRID).floor() * GRID
}

// ---- 竖向：四个区域各占几格（标题栏 / 菜单栏 / 功能区 / 状态栏）----

pub(crate) const TITLE_ROWS: i32 = 3;
pub(crate) const MENU_ROWS: i32 = 3;
pub(crate) const WORK_ROWS: i32 = 58;
pub(crate) const STATUS_ROWS: i32 = 3;

// 各区域的高度。
pub(crate) const TITLEBAR_H: f32 = cells(TITLE_ROWS);
pub(crate) const MENUBAR_H: f32 = cells(MENU_ROWS);
pub(crate) const STATUS_H: f32 = cells(STATUS_ROWS);

// 区域边界所在的网格线序号：上边界用它，下边界就是下一个区域的上边界（两个区域共用）。
pub(crate) const TITLE_LINE: i32 = 0;
pub(crate) const MENU_LINE: i32 = TITLE_LINE + TITLE_ROWS;
pub(crate) const WORK_LINE: i32 = MENU_LINE + MENU_ROWS;
pub(crate) const STATUS_LINE: i32 = WORK_LINE + WORK_ROWS;
pub(crate) const WINDOW_ROWS: i32 = STATUS_LINE + STATUS_ROWS;

// ---- 横向：窗口一共 125 格 ----

pub(crate) const WINDOW_COLS: i32 = 125;
// 分割条 1 格；左栏默认 89 格；右栏拿剩下的 35 格。
pub(crate) const SPLITTER_COLS: i32 = 1;
pub(crate) const CHART_COLS_DEFAULT: i32 = 89;
pub(crate) const PANEL_COLS: i32 = WINDOW_COLS - CHART_COLS_DEFAULT - SPLITTER_COLS;
// 两栏都不小于 35 格（316px，约等于原来定的 320px）；分割条最远拖到哪儿由 split_max_x 按当前宽度现算。
pub(crate) const PANE_MIN_COLS: i32 = 35;

// 启动时的默认客户区尺寸：1126x604 = 125 x 67 格。窗口现在可以自由缩放，这只是开机大小。
pub(crate) const WINDOW_W: f32 = cells(WINDOW_COLS);
pub(crate) const WINDOW_H: f32 = cells(WINDOW_ROWS);

// ---- 窗口尺寸下限（不再钉死尺寸，但也不能小到排不下）----
//   宽：两栏各自最小 35 格 + 1 格分割条 = 71 格 = 640px。
//   高：标题 / 菜单 / 状态各 3 格，加上左栏最小（K 线 240 + 成交量 64 + 指标 64 + 固定开销 20
//    = 388px → 49 格）= 58 格 = 523px。
pub(crate) const WINDOW_MIN_COLS: i32 = PANE_MIN_COLS * 2 + SPLITTER_COLS;
pub(crate) const LEFT_WORK_MIN_ROWS: i32 = 49;
pub(crate) const WINDOW_MIN_ROWS: i32 = TITLE_ROWS + MENU_ROWS + STATUS_ROWS + LEFT_WORK_MIN_ROWS;
pub(crate) const WINDOW_MIN_W: f32 = cells(WINDOW_MIN_COLS);
pub(crate) const WINDOW_MIN_H: f32 = cells(WINDOW_MIN_ROWS);

// 标题栏右侧系统按钮：3 格，也就是 cells(3) = 28x28；这一组贴着窗口右边框放。
pub(crate) const SYS_BTN_COLS: i32 = 3;
pub(crate) const SYS_BTN_W: f32 = cells(SYS_BTN_COLS);
pub(crate) const SYS_BTN_H: f32 = cells(SYS_BTN_COLS);

// 标题栏内部：可拖拽区从第 8 条网格线开始。
// 程序名不按固定像素摆，而是照菜单栏第一项那套居中规则排（见 ui::titlebar），两行文字才严格对齐。
pub(crate) const TITLE_DRAG_LINE: i32 = 8;

// 菜单栏：下拉面板宽 16 格、每行 3 格。
pub(crate) const MENU_PANEL_W: f32 = cells(16);
pub(crate) const MENU_ROW_ROWS: i32 = 3;
pub(crate) const MENU_ROW_H: f32 = cells(MENU_ROW_ROWS);

// 区域内部文字的左内边距：1px 线格 + 1 格内容，正好落在第一条内网格线上。
pub(crate) const TEXT_PAD_X: f32 = LINE_W + GRID;

// ---- 功能区里的图表 ----

// 右侧价格轴 8 格；绘图区上下各留 1 格。
pub(crate) const AXIS_COLS: i32 = 8;
pub(crate) const AXIS_W: f32 = cells(AXIS_COLS);
pub(crate) const PLOT_PAD_COLS: i32 = 1;

// ---- K 线区内部三层 + 成交量区（本次新增）----
//  这几块的高度按「8px 内容格」算（和颜色面板同一套口径，不是 cells(n) 的 9n+1）：
//    K 线区（默认 600 = 75 格）/ 1px 线格 / 横向拖拽条 8px / 1px 线格 / 成交量区（默认 96 = 12 格）。
//  K 线区内部再从上下往下切：信息栏 24 / 1px 线格 / 主图（剩余吸到 8 的倍数）/ 1px 线格 / 时间轴 24。
//  三块加起来的固定开销 = 拖拽条 8 + 两条 1px 线格 = 10px（见 LEFT_FIXED_H）。

/// 信息栏高度：24px（3 格）。
pub(crate) const INFO_H: f32 = 24.0;

/// 信息栏里的八项：标签 + 量宽度用的模板值（模板只为了算宽度，不参与显示）。
/// 模板比实际数值宽一点点没关系 —— 数值是右对齐的，多出来的宽度全在标签那侧当空隙。
pub(crate) const INFO_LABELS: [&str; 8] = ["日期", "开", "高", "低", "收", "涨跌", "涨跌幅", "成交量"];
pub(crate) const INFO_TEMPLATES: [&str; 8] = [
    "0000-00-00",
    "0000.00",
    "0000.00",
    "0000.00",
    "0000.00",
    "-0000.00",
    "-0000.00%",
    "0000.0万",
];

/// 时间轴高度：24px（3 格）。
pub(crate) const TIME_AXIS_H: f32 = 24.0;

/// 时间轴相邻标签中心的最小间距：48px（6 格）。
pub(crate) const TIME_LABEL_GAP_MIN: f32 = 48.0;

/// 成交量区：量柱上下各留 8px（1 格）边距，右侧刻度轴 2~3 条刻度。
pub(crate) const VOL_PAD_Y: f32 = GRID;
pub(crate) const VOL_TICK_COUNT: usize = 3;

/// 成交量刻度 / 信息栏成交量那一项的单位换算：1e8 起步用「亿」，1e4 起步用「万」。
pub(crate) const YI: f64 = 1.0e8;
pub(crate) const WAN: f64 = 1.0e4;

// ---- K 线的缩放与平移 ----
//  实体像素宽 scale 只能是奇数（3、5、7 … 15）：影线是固定 1px 的竖线，必须落在实体正中的
//  那一列像素上 —— 只有奇数宽的实体才有「正中那一列」（偶数宽的正中落在两列像素之间，
//  影线必然偏左或偏右一格）。滚轮 / 面板一格走 ±2 像素，全程只在奇数档之间走，踩不到偶数。
//  影线固定 1px（LINE_W），不跟着 scale 变。
//  缩到最小也保证实体有 3px：3px 是还能看出「上涨空心 / 下跌实心」的最小宽度。
//  一屏放几根 = 绘图区宽度 / 每根占的步进（实体宽 + 必要时那 1px 间隙），向下取整。

// 实体默认像素宽：5（奇数档里偏细的一档，够看出空心，一屏也能放得下足够多根）。
pub(crate) const BAR_SCALE_DEFAULT: u32 = 5;
/// 实体像素宽的上下限：缩到底 3px 一根（再细就看不出「上涨空心 / 下跌实心」），放到最大 15px 一根。
/// 两端都是奇数 —— 配上「一格 ±2」的步进，实体宽永远是奇数（再经 snap_scale 兜底）。
pub(crate) const SCALE_MIN: u32 = 3;
pub(crate) const SCALE_MAX: u32 = 15;
/// scale 到 3 以上才在蜡烛之间留 1px 间隙：1~2px 的实体本来就细，再留缝就散了。
/// SCALE_MIN 现在就是 3，所以这条规则实际总是成立；写成条件是为了把「什么时候留缝」写明白。
pub(crate) const SCALE_GAP_FROM: u32 = 3;
/// 触控板那种「按点」上报的滚轮增量，要攒够这么多点才算一格。
/// 0.36.2 的 InputOptions::line_scroll_speed 默认就是 40（Line 单位换算成点用的就是它），
/// 而 InputState 里那个字段是私有的、读不到，只能在这儿写同一个值。
pub(crate) const WHEEL_POINTS_PER_NOTCH: f32 = 40.0;

/// 一根 K 线在屏幕上占的横向步进：实体宽 scale +（scale >= 3 时）那 1px 间隙。
/// 「屏幕 x 落在第几根」「一屏放得下几根」「拖拽位移换算成几根」全用这一个口径。
pub(crate) fn bar_advance(scale: u32) -> f32 {
    let scale = snap_scale(scale);
    scale as f32 + if scale >= SCALE_GAP_FROM { LINE_W } else { 0.0 }
}

/// 把实体宽吸附到合法档位：先取奇数（| 1），再夹进 [SCALE_MIN, SCALE_MAX]（两端都是奇数）。
/// 绘图和步进一律过它 —— 实体宽奇数，影线那 1px 才落得住实体正中那一列。
pub(crate) fn snap_scale(scale: u32) -> u32 {
    (scale | 1).clamp(SCALE_MIN, SCALE_MAX)
}

/// 缩放 notches 格之后的实体宽：一格 = ±2 像素，只为了从当前奇数档跨到下一个奇数档。
pub(crate) fn scale_after(scale: u32, notches: i32) -> u32 {
    let next = scale as i64 + 2 * notches as i64;
    snap_scale(next.clamp(SCALE_MIN as i64, SCALE_MAX as i64) as u32)
}

/// 绘图区里一屏能放几根 K 线：宽度 / 步进，向下取整（至少 1 根）。
pub(crate) fn visible_bars(plot_w: f32, scale: u32) -> usize {
    (plot_w / bar_advance(scale)).floor().max(1.0) as usize
}

// 价格轴上从上到下均匀标几个刻度。
pub(crate) const TICK_COUNT: usize = 5;

// 预生成的 K 线根数：比一屏能显示的（80 多根）多得多，留给以后的平移。
pub(crate) const BAR_COUNT: usize = 512;

// 随机种子和起始价：固定住，每次启动看到的都是同一段行情。
pub(crate) const BAR_SEED: u64 = 20_261_012;
pub(crate) const BAR_START: f64 = 20.0;

// 十字光标的价格 / 索引标签：高 2 格；宽度按文字算，也吸到整数格。
pub(crate) const CROSS_LABEL_H: f32 = cells(2);

// 空闲时也定期重绘一次，让状态栏的刷新数据持续可见（单位：毫秒）。
pub(crate) const IDLE_REPAINT_MS: u64 = 500;

// 编译期自检：凡是「尺寸」类的常量，减去那 1px 线格后必须能被 9 整除（也就是 cells(n)）。
const _: () = assert!(PITCH == 9.0);
const _: () = assert!((WINDOW_W as i32 - 1) % 9 == 0);
const _: () = assert!((WINDOW_H as i32 - 1) % 9 == 0);
const _: () = assert!((WINDOW_MIN_W as i32 - 1) % 9 == 0);
const _: () = assert!((WINDOW_MIN_H as i32 - 1) % 9 == 0);
// 最小窗口高度里，功能区那块必须放得下左栏三块的最小高度：
// 固定开销 20（两条拖拽条 + 四条 1px 线格）+ K 线 240 + 成交量 64 + 指标 64 = 388px。
const _: () = assert!(cells(LEFT_WORK_MIN_ROWS) >= 388.0);
const _: () = assert!((TITLEBAR_H as i32 - 1) % 9 == 0);
const _: () = assert!((MENUBAR_H as i32 - 1) % 9 == 0);
const _: () = assert!((STATUS_H as i32 - 1) % 9 == 0);
const _: () = assert!((SYS_BTN_W as i32 - 1) % 9 == 0);
const _: () = assert!((SYS_BTN_H as i32 - 1) % 9 == 0);
const _: () = assert!((MENU_PANEL_W as i32 - 1) % 9 == 0);
const _: () = assert!((MENU_ROW_H as i32 - 1) % 9 == 0);
const _: () = assert!((AXIS_W as i32 - 1) % 9 == 0);
const _: () = assert!((CROSS_LABEL_H as i32 - 1) % 9 == 0);
// K 线实体宽必须是奇数：影线 1px 要落在实体正中那一列，偶数宽的实体没有正中的列。
const _: () = assert!(SCALE_MIN % 2 == 1 && SCALE_MAX % 2 == 1 && BAR_SCALE_DEFAULT % 2 == 1);
// 左栏 + 分割条 + 右栏 的格数必须正好等于窗口宽，不然右边会多出没铺满的像素。
const _: () = assert!(CHART_COLS_DEFAULT + SPLITTER_COLS + PANEL_COLS == WINDOW_COLS);

// ===========================================================================
//  四、像素级绘制工具
// ===========================================================================

/// 坐标吸附：返回坐标所在像素的中心（整数 + 0.5）。
/// 不用 egui 的 Painter::round_to_pixel_center：emath 的实现是
/// (v - 0.5).round() + 0.5，而它在 v 正好是 0 时会返回 -0.5（负数半值向下舍入），
/// 于是画在 x = 0 / y = 0 上的线整条落到窗口外 —— 四条边框线只有右、下两条看得见。
pub(crate) fn px_center(v: f32) -> f32 {
    v.floor() + 0.5
}

/// 1px 硬边水平线：只占一个像素行。
pub(crate) fn hline_hard(painter: &egui::Painter, x0: f32, x1: f32, y: f32, color: egui::Color32) {
    let top = px_center(y) - 0.5;
    let rect = egui::Rect::from_min_size(egui::pos2(x0, top), egui::vec2(x1 - x0, LINE_W));
    painter.rect_filled(rect, egui::CornerRadius::ZERO, color);
}

/// 1px 硬边垂直线：只占一个像素列。
pub(crate) fn vline_hard(painter: &egui::Painter, x: f32, y0: f32, y1: f32, color: egui::Color32) {
    let left = px_center(x) - 0.5;
    let rect = egui::Rect::from_min_size(egui::pos2(left, y0), egui::vec2(LINE_W, y1 - y0));
    painter.rect_filled(rect, egui::CornerRadius::ZERO, color);
}

/// 手绘「叉」：正中间只占 1 个像素，四条臂各斜着往外正好画 radius 个像素。
/// 第 d 层画 (cx-d, cy-d)、(cx+d, cy-d)、(cx-d, cy+d)、(cx+d, cy+d)，d = 0..radius ——
/// 关于中心像素左右、上下都严格镜像，而且不会像偶数边长的画法那样在中间堆出 2x2 的实心块。
/// 不用 line_segment：斜线交给光栅器会得到带灰边的 2px 阶梯，左右也不对称。
pub(crate) fn paint_cross(painter: &egui::Painter, center: egui::Pos2, radius: i32, color: egui::Color32) {
    let cx = center.x.round() as i32;
    let cy = center.y.round() as i32;
    let put = |col: i32, row: i32| {
        let block = egui::Rect::from_min_size(
            egui::pos2(col as f32, row as f32),
            egui::vec2(LINE_W, LINE_W),
        );
        painter.rect_filled(block, egui::CornerRadius::ZERO, color);
    };
    for d in 0..=radius {
        put(cx - d, cy - d);
        put(cx + d, cy - d);
        put(cx - d, cy + d);
        put(cx + d, cy + d);
    }
}

/// 1px 硬边框：画在逻辑区最外圈那一圈像素上，不占布局尺寸。
pub(crate) fn border_hard(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    hline_hard(painter, rect.min.x, rect.max.x, rect.min.y, color);
    hline_hard(painter, rect.min.x, rect.max.x, rect.max.y - LINE_W, color);
    vline_hard(painter, rect.min.x, rect.min.y, rect.max.y, color);
    vline_hard(painter, rect.max.x - LINE_W, rect.min.y, rect.max.y, color);
}

/// 硬边折线：按像素列走，每列把上一列的 y 到这一列的 y 竖着填满 —— 斜线是硬像素阶梯，
/// 没有抗锯齿灰边（line_segment 交给光栅器会得到带灰边的 2px 阶梯，所以这里不用它）。
/// pts 是 (列, 行)：列取整数像素列号，行是像素中心。
pub(crate) fn polyline_hard(painter: &egui::Painter, pts: &[(f32, f32)], color: egui::Color32) {
    // 一列里把 y0..y1 竖着填满（含两个端点行），保证相邻两列之间不留缝。
    let fill = |x: f32, y0: f32, y1: f32| {
        let (a, b) = (y0.min(y1).round(), y0.max(y1).round());
        painter.rect_filled(
            egui::Rect::from_min_max(egui::pos2(x, a), egui::pos2(x + LINE_W, b + LINE_W)),
            egui::CornerRadius::ZERO,
            color,
        );
    };
    if pts.len() == 1 {
        fill(pts[0].0.round(), pts[0].1, pts[0].1);
        return;
    }
    for pair in pts.windows(2) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        let (c0, c1) = (x0.round() as i32, x1.round() as i32);
        if c1 <= c0 {
            // 同一列（或者数据顺序回退）：竖着连一段，别让线断开。
            fill(c0 as f32, y0, y1);
            continue;
        }
        let mut prev = y0.round();
        for x in c0..=c1 {
            let t = (x - c0) as f32 / (c1 - c0) as f32;
            let y = y0 + (y1 - y0) * t;
            fill(x as f32, prev, y);
            prev = y.round();
        }
    }
}

