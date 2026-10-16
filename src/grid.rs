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
//    - 画布由「内容格」和「网格线格」交替铺成：内容格 8px（GRID），网格线格 1px（LINE_W），
//      所以节距 PITCH = GRID + LINE_W = 9px。
//    - 所有坐标都落在网格线上：第 i 条网格线的坐标 = i * PITCH。控件本体只占内容格，
//      那 1px 边框画在两端（以及相邻控件之间）的网格线格里 - 数下来一个跨 n 格的控件
//      占「n 个内容格 + n+1 条网格线格」，尺寸就是 cells(n) = 9n + 1。
//    - 相邻控件共用中间那条网格线格（两者的矩形在那一列/行上重叠 1px）：各自尺寸照样是
//      cells(n)、完全相等，中间也永远只有 1px，不可能出现两条线堆叠成 2px。
//    - 不需要边框的网格线格就留空、显示底色。

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


/// 2 格内容区（8 + 1 + 8 = 17px）正中间那一行（列）相对区块左上角的偏移：1px 线格 + 8px = 9。
/// 系统按钮 / 图标这类左右上下对称的符号以它为中心，落点永远是整数像素。
pub(crate) const CELL_CENTER: f32 = LINE_W + GRID;

/// 把一段跨度向上吸到整数格：结果一定是 cells(n) 的形状。
pub(crate) fn snap_cells_up(v: f32) -> f32 {
    let n = ((v - LINE_W) / PITCH).ceil().max(0.0);
    n * PITCH + LINE_W
}

/// 一块区域里「第 row 条横网格线起、跨 rows 格」的那一段：上沿落在网格线上，高度是 cells(rows)。
/// 相邻两段共用边界那条 1px 线格（两个矩形在那 1px 上重叠），Pane / 面板的内部分段都用它排版。
pub(crate) fn grid_band(area: egui::Rect, row: i32, rows: i32) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(area.min.x, area.min.y + grid_line(row)),
        egui::vec2(area.width(), cells(rows)),
    )
}

// ---- 自由尺寸窗口（不吸网格的子窗口）里的分段 ----
//  窗口区里的子窗口可以 1px 1px 地挪 / 缩（铁律 4），所以窗口本体不再是 cells(n) 的形状，
//  内部排版要分两种段：固定段 + 弹性段（吸零头的那一段）。
//    - 固定段（free_band）：尺寸永远是死的（最小化键 / 关闭键 / 状态带 / 时间带）；
//    - 弹性段（free_tail）：吃下窗口里剩下的全部，窗口多出来的那几个像素（溢出空间）
//      只许落进它里面 - 原则上落在右边和下方，而且只落在装得下零头的地方。

/// 自由尺寸窗口里的一条固定段：上沿 = 区域顶边 + 9*row，高 = cells(rows)（尺寸就是死的）。
pub(crate) fn free_band(area: egui::Rect, row: i32, rows: i32) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(area.min.x, area.min.y + grid_line(row)),
        egui::vec2(area.width(), cells(rows)),
    )
}

/// 自由尺寸窗口里的弹性段（尾段）：从区域顶边往下第 row 条网格线起，一直铺到区域下沿 -
/// 窗口尺寸不是整格时多出来的零头全归它。
pub(crate) fn free_tail(area: egui::Rect, row: i32) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::pos2(area.min.x, area.min.y + grid_line(row)),
        area.max,
    )
}

/// 把一段跨度吸到最近的整数格：结果一定是 cells(n) 的形状（拖拽条拖动时用它把像素吸成格）。
pub(crate) fn snap_cells_round(v: f32) -> f32 {
    let n = ((v - LINE_W) / PITCH).round().max(0.0);
    n * PITCH + LINE_W
}

// ---- 竖向：四个区域各占几格（标题栏 / 菜单栏 / 功能区 / 状态栏）----

pub(crate) const TITLE_ROWS: i32 = 2;
pub(crate) const MENU_ROWS: i32 = 2;
pub(crate) const WORK_ROWS: i32 = 96;
pub(crate) const STATUS_ROWS: i32 = 2;

// 各区域的高度：标题 / 菜单 / 状态都是一行文字（2 格 = cells(2) = 19px）。
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

// 启动时的默认客户区尺寸：1126x919 = 125 x 102 格（标题 / 菜单 / 状态各 2 格 = 一个文本行盒，
// 功能区 96 格）。窗口现在可以自由缩放，这只是开机大小。
pub(crate) const WINDOW_W: f32 = cells(WINDOW_COLS);
pub(crate) const WINDOW_H: f32 = cells(WINDOW_ROWS);

// ---- 窗口尺寸下限（不再钉死尺寸，但也不能小到排不下）----
//   宽：左栏（图表）最小 35 格 + 右栏（面板列）35 格 + 1 格共用的线格 = 71 格 = 640px。
//   高：标题 / 菜单 / 状态各 2 格 + 左栏最小 62 格 = 68 格 = 613px。左栏那条下限当年是按
//    三块图表叠着算的（3*PANE_ROWS_MIN + 2），现在只有一块图表，留着这条下限图个保险。
pub(crate) const WINDOW_MIN_COLS: i32 = PANE_MIN_COLS * 2 + SPLITTER_COLS;
pub(crate) const LEFT_WORK_MIN_ROWS: i32 = 3 * PANE_ROWS_MIN + 2;
pub(crate) const WINDOW_MIN_ROWS: i32 = TITLE_ROWS + MENU_ROWS + STATUS_ROWS + LEFT_WORK_MIN_ROWS;
pub(crate) const WINDOW_MIN_W: f32 = cells(WINDOW_MIN_COLS);
pub(crate) const WINDOW_MIN_H: f32 = cells(WINDOW_MIN_ROWS);

// 标题栏右侧系统按钮：2 格 = cells(2) = 19x19（正好和标题栏一样高）；这一组贴着窗口右边框放。
pub(crate) const SYS_BTN_COLS: i32 = 2;
pub(crate) const SYS_BTN_W: f32 = cells(SYS_BTN_COLS);
pub(crate) const SYS_BTN_H: f32 = cells(SYS_BTN_COLS);

// 标题栏内部：可拖拽区从第 8 条网格线开始。
// 程序名不按固定像素摆，而是照菜单栏第一项那套居中规则排（见 ui::titlebar），两行文字才严格对齐。
pub(crate) const TITLE_DRAG_LINE: i32 = 8;

// 菜单栏：下拉面板宽 16 格、每行 2 格（一个文本行盒）。
pub(crate) const MENU_PANEL_W: f32 = cells(16);
pub(crate) const MENU_ROW_ROWS: i32 = 2;
pub(crate) const MENU_ROW_H: f32 = cells(MENU_ROW_ROWS);

// 区块内部文字的左内边距：1px 线格 + 1 格内容 = 9px，算盒子宽度用它
//（宽度只关心「边框 + 空档」正好 1 格）。
pub(crate) const TEXT_PAD_X: f32 = LINE_W + GRID;

// 区块内部左对齐文字的起点相对区块左边的偏移：左边那条 1px 边框 + 1 个内容格的空档 +
// 那条网格线格 = 10px，正好落在第 1 个内容格的起点上（内容格 = [9k+1, 9k+9)，见 font.rs
// 的网格文字模型）- 文字只有从内容格起点排，整串才和 9px 网格对齐（一对字符正好一条内容格）。
// 比 TEXT_PAD_X 多 1px：宽度只管「边框 + 空档」，落点还要让开那条网格线格。
pub(crate) const TEXT_INSET_X: f32 = LINE_W + PITCH;

/// 一个文本行盒跨几格：2 格（内容区 17px = 8 + 1 + 8，就是 font.rs 的 TEXT_BOX_H）。
/// 铁律：全界面凡是出现文字的地方，元素高度都得是它的整数倍，文字那一行才踩得在网格线上。
pub(crate) const TEXT_ROWS: i32 = 2;

// ---- 文字的横向落点：全界面的文字都必须踩在内容格上（内容格 = [9k+1, 9k+9)，见 font.rs）----
//  每个窗口自带一套网格：原点 = 这块窗口自己的左上角（窗口可以 1px 1px 地挪 / 缩，不吸主窗口的网格）。
//  所以这几个吸附入口一律要一个 origin 参数 = 「当前这套网格的原点」，
//  通常是所在区块（标题栏 / 面板 / 状态带 / 编辑器正文区）的左边那条线。
//  同一块窗口里，任何以 grid_line(k) 起算的区块，它的 min 都跟窗口左上角同余（差整数格），
//  所以拿区块自己的 min 当原点，和拿窗口左上角当原点，吸附结果完全一样。

/// 把坐标 v 向下吸到「原点 origin 那一套 9px 网格」上离它最近的竖网格线。
/// 区块自己 shrink(LINE_W) 出来的内框也能吸回同一条线（差 1px 不影响向下取整的结果）。
pub(crate) fn grid_floor(v: f32, origin: f32) -> f32 {
    origin + ((v - origin) / PITCH).floor() * PITCH
}

/// 把坐标 v 吸到原点 origin 那套网格上最近的竖网格线（四舍五入）：居中摆放的浮动标签用它。
pub(crate) fn grid_snap(v: f32, origin: f32) -> f32 {
    origin + ((v - origin) / PITCH).round() * PITCH
}

/// 左对齐文字的 x 落点：区块左边那条 1px 网格线右边第 1 个内容格。
/// origin = 这块窗口自己的网格原点（区块左边那条线所在的那一套网格）：
/// 先吸到左边那条线上再偏移，结果一定落在内容格起点，左边正好留出 1 格空档。
pub(crate) fn text_left_x(rect_left: f32, origin: f32) -> f32 {
    grid_floor(rect_left, origin) + TEXT_INSET_X
}

/// 右对齐文字的 x 落点（当作 Align2::RIGHT_* 的 pos.x）：文字末尾停在右边那条网格线的左边 1 格。
/// 参数是那条线的坐标（= 边框线所在的像素列，一定落在这块窗口的网格线上）- 最后一个字因而
/// 落在完整的内容格里，右边正好留出 1 格空档。
pub(crate) fn text_right_x(line_x: f32, origin: f32) -> f32 {
    grid_floor(line_x, origin) - GRID
}

// ---- 功能区里的图表 ----

// 右侧纵轴 9 格；绘图区上下各留 8px 边距（见 ui::chart 的 CHART_PAD）。
// 9 格 = VALUE_COLS_MAX + 2 = 数值最长 7 格、两边各留 1 格空档，十字光标的数值牌子正好塞进轴里。
pub(crate) const AXIS_COLS: i32 = 9;
pub(crate) const AXIS_W: f32 = cells(AXIS_COLS);

/// 图表里数值文字的最大墨迹宽度（内容格）：最长就是 "-9999.9万" / "9999亿" / "2000兆" 这类，
/// 7 格封顶。纵轴留 VALUE_COLS_MAX + 2 = 9 格，数值标签永远越不出轴那一列。
pub(crate) const VALUE_COLS_MAX: i32 = 7;

// ---- 左栏各块（原先夹在它们之间的横向拖拽条已取消，那套算法原样保留备用）----
//  现在的排法：左栏只有一块图表，占满功能区高度，自己画自己的边框（见 ui::panel 的 child_areas）。
//  老排法（代码还在，以后可能用回来）：K 线区 / 1px 线格 / 成交量区 / 1px 线格 / 指标区，
//  相邻两块共用边界那条 1px 线格，夹在中间那条 1 格（cells(1) = 10px）的横向拖拽条已取消，
//  各块高度都是整数格 cells(n) = 9n + 1（默认 52 / 21 / 21 格），固定开销见 LEFT_FIXED_H。
//  每个图表内部再从上下往下切（见 ui::chart 的 chart_rows）：标题栏 2 格 /（可选）状态带
//  （条数由指标自己声明）/（可选）时间带 / 中间的绘图区 - 各段都从网格线起、高度是 cells(k)，
//  相邻两段共用边界那条线格，线永远只有 1px；绘图区吃掉剩下的高度，最后一段一直铺到面板底边
//  （不留白条）。谁要哪几条带由指标自己声明（见 indicator::ChartLayout），所以同一套代码
//  挂出来的每一块图表都长一个样。

/// 时间轴相邻标签中心的最小间距：48px（6 格）。
pub(crate) const TIME_LABEL_GAP_MIN: f32 = 48.0;

// ---- 窗口（窗口区里的子窗口 / 源码编辑窗 / 颜色面板）：统一的一套尺寸 ----
//  一块窗口 = 标题栏 2 格 + 内容若干格，外壳由 ui::floatwin 的 Win 统一画（全工程只有这一份），
//  所以「标题栏多高、关闭键多大」这类尺寸也只有这一份。内容区第 row 条网格线的偏移
//  一律是 grid_line(row)，和 Pane / 颜色面板一个口径（各段共用边界那条 1px 线格）。

/// 浮动窗口的标题栏格数：2 格 = 一个文本行盒（和 Pane 标题栏 / 菜单栏一个口径）。
pub(crate) const WIN_TITLE_ROWS: i32 = 2;
pub(crate) const WIN_TITLE_H: f32 = cells(WIN_TITLE_ROWS);
/// 标题栏里的关闭键：跨 2 格 = cells(2) = 19x19（和标题栏一样高），贴着窗口右上角。
pub(crate) const WIN_CLOSE_W: f32 = cells(2);

// ---- K 线的缩放与平移 ----
//  实体像素宽 scale 只能是奇数（3、5、7 ... 15）：影线是固定 1px 的竖线，必须落在实体正中的
//  那一列像素上 - 只有奇数宽的实体才有「正中那一列」（偶数宽的正中落在两列像素之间，
//  影线必然偏左或偏右一格）。滚轮 / 面板一格走 +/-2 像素，全程只在奇数档之间走，踩不到偶数。
//  影线固定 1px（LINE_W），不跟着 scale 变。
//  缩到最小也保证实体有 3px：3px 是还能看出「上涨空心 / 下跌实心」的最小宽度。
//  一屏放几根 = 绘图区宽度 / 每根占的步进（实体宽 + 必要时那 1px 间隙），向下取整。

// 实体默认像素宽：5（奇数档里偏细的一档，够看出空心，一屏也能放得下足够多根）。
pub(crate) const BAR_SCALE_DEFAULT: u32 = 5;
/// 实体像素宽的上下限：缩到底 3px 一根（再细就看不出「上涨空心 / 下跌实心」），放到最大 15px 一根。
/// 两端都是奇数 - 配上「一格 +/-2」的步进，实体宽永远是奇数（再经 snap_scale 兜底）。
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
/// 绘图和步进一律过它 - 实体宽奇数，影线那 1px 才落得住实体正中那一列。
pub(crate) fn snap_scale(scale: u32) -> u32 {
    (scale | 1).clamp(SCALE_MIN, SCALE_MAX)
}

/// 缩放 notches 格之后的实体宽：一格 = +/-2 像素，只为了从当前奇数档跨到下一个奇数档。
pub(crate) fn scale_after(scale: u32, notches: i32) -> u32 {
    let next = scale as i64 + 2 * notches as i64;
    snap_scale(next.clamp(SCALE_MIN as i64, SCALE_MAX as i64) as u32)
}

/// 绘图区里一屏能放几根 K 线：宽度 / 步进，向下取整（至少 1 根）。
pub(crate) fn visible_bars(plot_w: f32, scale: u32) -> usize {
    (plot_w / bar_advance(scale)).floor().max(1.0) as usize
}

// 预生成的 K 线根数：比一屏能显示的（80 多根）多得多，留给以后的平移。
pub(crate) const BAR_COUNT: usize = 512;

// 随机种子和起始价：固定住，每次启动看到的都是同一段行情。
pub(crate) const BAR_SEED: u64 = 20_261_012;
pub(crate) const BAR_START: f64 = 20.0;

// 十字光标的价格 / 索引标签：高 2 格（cells(2) = 19px，上下两条边框正好落在两条网格线上）；
// 宽度按文字算，也吸到整数格。浮动标签的纵向上沿用 floating_label_top 吸附网格（见下）。
pub(crate) const CROSS_LABEL_ROWS: i32 = 2;
pub(crate) const CROSS_LABEL_H: f32 = cells(CROSS_LABEL_ROWS);

// 空闲时也定期重绘一次，让状态栏的刷新数据持续可见（单位：毫秒）。
pub(crate) const IDLE_REPAINT_MS: u64 = 500;
// ---- 图表（Pane）的内部构件：标题栏 / 两个按钮 / 指标下拉面板 / 源码查看窗 ----
//  一块图表内部自上而下都是「跨若干格」的段，相邻两段共用边界那条 1px 线格：
//    标题栏 2 格 /（可选）状态带（2 格）/ 绘图区（吃掉剩下的高度）/（可选）时间带。
//  要哪几条带由指标自己声明（见 indicator::ChartLayout）。各段都从网格线起、高度是 cells(k)
//  = 9k+1，所以边界线各归一段、谁也不叠谁的像素；边框只落在网格线格里，永远只有 1px。

/// 标题栏：跨 2 格（17px 内容 = 一个文本行盒）。它下沿那条线格就是和下一段共用的边界线。
pub(crate) const PANE_TITLE_ROWS: i32 = 2;
pub(crate) const PANE_TITLE_H: f32 = cells(PANE_TITLE_ROWS);

/// 图表窗自己那条状态栏（整块图表最下面一条，2 格）：缩放 / 平移 / 可见几根写在这儿。
/// 它属于「图表窗」，不属于「图表」- 主窗口的状态栏只留调试信息（帧率 / 操作回显）。
pub(crate) const CHART_BAR_ROWS: i32 = TEXT_ROWS;
pub(crate) const CHART_BAR_H: f32 = cells(CHART_BAR_ROWS);

/// 一块图表（Pane）的最小高度（格）：标题栏 2 格 + 状态带 2 格 + 时间带 2 格 + 状态栏 2 格
/// + 绘图区至少 14 格（= 22 格 = cells(22) = 199px）。所有图表不分主图 / 副图，统一用这一条下限 -
/// 拉到最小也放得下全部元素，文字不会挤在一起，绘图区也还有 14 格（125px）可用。
pub(crate) const PANE_ROWS_MIN: i32 = 22;
pub(crate) const PANE_H_MIN: f32 = cells(PANE_ROWS_MIN);
/// 绘图区最小格数：总格数减去标题栏 / 状态带 / 时间带 / 状态栏（四条带各 2 格 = 一个文本行盒）。
pub(crate) const PANE_PLOT_ROWS_MIN: i32 =
    PANE_ROWS_MIN - PANE_TITLE_ROWS - 2 - 2 - CHART_BAR_ROWS;


/// 标题栏右侧那两个按钮（「切换」「代码」）：宽度按文字量（铁律：文字左右各留 1 格空档），
/// 这里只留一条下限；高就是标题栏那 2 格，上下两条边框正好压在 Pane 顶边和标题栏下沿那条线上。
pub(crate) const PANE_BTN_COLS: i32 = 3;
pub(crate) const PANE_BTN_W: f32 = cells(PANE_BTN_COLS);
pub(crate) const PANE_BTN_H: f32 = cells(PANE_TITLE_ROWS);

/// 指标切换下拉面板：宽 20 格；每项 2 格 - 和菜单栏那份同一套（行高 cells(2)、步进 grid_line(2)），
/// 相邻两行共用中间那条网格线。
pub(crate) const PANE_MENU_COLS: i32 = 20;
pub(crate) const PANE_MENU_W: f32 = cells(PANE_MENU_COLS);
pub(crate) const PANE_MENU_ROW_ROWS: i32 = 2;
pub(crate) const PANE_MENU_ROW_H: f32 = cells(PANE_MENU_ROW_ROWS);

/// 「叠加」菜单（K 线图标题栏那个按钮下拉出来的均线开关表）：宽 10 格 - 最左边 1 格是色块标记、
/// 接着是名字（最长 "EMA250" = 6 个 ASCII 格），左右各 1 格空档还富余；每项 2 格，
/// 行高 / 行距和指标菜单同一套。一共 12 项（MA / EMA 各 6 条）。
pub(crate) const OVERLAY_MENU_COLS: i32 = 10;
pub(crate) const OVERLAY_MENU_W: f32 = cells(OVERLAY_MENU_COLS);

/// 源码编辑窗：80 格宽 x 61 格高 = cells(80) x cells(61) = 721 x 550。
/// 从上往下：标题栏 2 格（外壳画）+ 编辑器菜单栏 2 格 + 编辑器按钮栏 2 格 + 正文区；
/// 正文区左边是行号区 6 格 = cells(6) = 55px、右边是竖直滚动条 1 格 = 8px 宽，
/// 代码行高 2 格 = 17px（文本行盒）。
pub(crate) const SRC_COLS: i32 = 80;
pub(crate) const SRC_ROWS: i32 = 61;
pub(crate) const SRC_W: f32 = cells(SRC_COLS);
pub(crate) const SRC_H: f32 = cells(SRC_ROWS);
pub(crate) const SRC_TITLE_ROWS: i32 = PANE_TITLE_ROWS;
pub(crate) const SRC_TITLE_H: f32 = cells(SRC_TITLE_ROWS);
/// 编辑器自己的菜单栏 / 按钮栏：各 2 格（一个文本行盒，和标题栏 / 菜单栏同一口径）。
pub(crate) const SRC_MENU_ROWS: i32 = TEXT_ROWS;
pub(crate) const SRC_BAR_ROWS: i32 = TEXT_ROWS;
/// 正文区上面那一摞：标题栏 + 菜单栏 + 按钮栏 = 6 格。
pub(crate) const SRC_HEAD_ROWS: i32 = SRC_TITLE_ROWS + SRC_MENU_ROWS + SRC_BAR_ROWS;
pub(crate) const SRC_GUTTER_COLS: i32 = 6;
pub(crate) const SRC_GUTTER_W: f32 = cells(SRC_GUTTER_COLS);
pub(crate) const SRC_BAR_W: f32 = GRID;
pub(crate) const SRC_LINE_H: f32 = 2.0 * GRID + LINE_W;
/// 相邻两行代码之间共用那条 1px 网格线：一行的步进 = 2 格内容 + 1px 线 = grid_line(2) = 18px。
pub(crate) const SRC_ROW_PITCH: f32 = grid_line(2);
/// 正文区第一行的上沿（相对面板左上角）：按钮栏下沿那条网格线 - 按钮栏和正文区共用它。
pub(crate) const SRC_BODY_TOP: f32 = grid_line(SRC_HEAD_ROWS);
/// 正文区能用的格数：上面那一摞 6 格之后、最底下留 1 格余量 = 54 格 = 486px。
pub(crate) const SRC_BODY_ROWS: i32 = SRC_ROWS - SRC_HEAD_ROWS - 1;
/// 一屏几行 = 正文区 54 格 / 每行 2 格 = 27 行（486px / 18px）。
pub(crate) const SRC_VISIBLE_LINES: usize = SRC_BODY_ROWS as usize / 2;

/// 源码窗第一次打开时的左上角（198 = 22*9、99 = 11*9）：落在窗口网格线上，参数和颜色面板一致。
pub(crate) const SRC_DEFAULT_X: f32 = 198.0;
pub(crate) const SRC_DEFAULT_Y: f32 = 99.0;

// 编译期自检：凡是「尺寸」类的常量，减去那 1px 线格后必须能被 9 整除（也就是 cells(n)）。
const _: () = assert!(PITCH == 9.0);
const _: () = assert!((WINDOW_W as i32 - 1) % 9 == 0);
const _: () = assert!((WINDOW_H as i32 - 1) % 9 == 0);
const _: () = assert!((WINDOW_MIN_W as i32 - 1) % 9 == 0);
const _: () = assert!((WINDOW_MIN_H as i32 - 1) % 9 == 0);
// 最小窗口高度里，功能区那块必须放得下左栏的最小高度（按三块图表叠着算的保守值：三块各
// PANE_ROWS_MIN 格，中间夹两条 1 格拖拽条共用线格，所以列里总格数 = 3*PANE_ROWS_MIN + 2）。
const _: () = assert!(LEFT_WORK_MIN_ROWS == 3 * PANE_ROWS_MIN + 2);
// 最小高度必须刚好装得下标题栏 / 状态带 / 时间带 / 状态栏，还留出绘图区那几格；
// 而且下限本身是 cells(n)。
const _: () = assert!(
    PANE_ROWS_MIN == PANE_TITLE_ROWS + TEXT_ROWS + TEXT_ROWS + CHART_BAR_ROWS + PANE_PLOT_ROWS_MIN
);
const _: () = assert!((CHART_BAR_H as i32 - 1) % 9 == 0);
const _: () = assert!(PANE_PLOT_ROWS_MIN >= 8);
// 文本行盒 / 标题 / 菜单 / 状态栏都必须正好是 2 格（一个文本行盒），文字才踩得在网格线上。
const _: () = assert!(TEXT_ROWS == 2);
const _: () = assert!(TITLE_ROWS == TEXT_ROWS && MENU_ROWS == TEXT_ROWS && STATUS_ROWS == TEXT_ROWS);
const _: () = assert!(PANE_TITLE_ROWS == TEXT_ROWS && PANE_MENU_ROW_ROWS == TEXT_ROWS);
const _: () = assert!(MENU_ROW_ROWS == TEXT_ROWS);
const _: () = assert!((PANE_H_MIN as i32 - 1) % 9 == 0);
const _: () = assert!((TITLEBAR_H as i32 - 1) % 9 == 0);
const _: () = assert!((MENUBAR_H as i32 - 1) % 9 == 0);
const _: () = assert!((STATUS_H as i32 - 1) % 9 == 0);
const _: () = assert!((SYS_BTN_W as i32 - 1) % 9 == 0);
const _: () = assert!((SYS_BTN_H as i32 - 1) % 9 == 0);
const _: () = assert!((MENU_PANEL_W as i32 - 1) % 9 == 0);
const _: () = assert!((MENU_ROW_H as i32 - 1) % 9 == 0);
const _: () = assert!((AXIS_W as i32 - 1) % 9 == 0);
const _: () = assert!((CROSS_LABEL_H as i32 - 1) % 9 == 0);
// Pane / 源码窗里新加的那些段和按钮，尺寸也必须都是 cells(n)。
const _: () = assert!((PANE_TITLE_H as i32 - 1) % 9 == 0);
const _: () = assert!((PANE_BTN_W as i32 - 1) % 9 == 0 && (PANE_BTN_H as i32 - 1) % 9 == 0);
const _: () = assert!((PANE_MENU_W as i32 - 1) % 9 == 0 && (PANE_MENU_ROW_H as i32 - 1) % 9 == 0);
const _: () = assert!((OVERLAY_MENU_W as i32 - 1) % 9 == 0);
const _: () = assert!((SRC_W as i32 - 1) % 9 == 0 && (SRC_H as i32 - 1) % 9 == 0);
const _: () = assert!((SRC_TITLE_H as i32 - 1) % 9 == 0 && (SRC_GUTTER_W as i32 - 1) % 9 == 0);
// 编辑器的菜单栏 / 按钮栏和标题栏一个口径：全是 2 格，合起来 6 格在正文区上面。
const _: () = assert!(SRC_MENU_ROWS == TEXT_ROWS && SRC_BAR_ROWS == TEXT_ROWS && SRC_HEAD_ROWS == 6);
const _: () = assert!((SRC_BODY_TOP as i32) % 9 == 0 && SRC_BODY_ROWS > 0);
// 浮动窗口那套统一尺寸：标题栏 / 关闭键都得是 cells(n)，而且和源码窗 / 颜色面板各说各的那两个
// 标题栏格数完全同值 - 外壳只有一份，三处口径必须一致。
const _: () = assert!((WIN_TITLE_H as i32 - 1) % 9 == 0 && (WIN_CLOSE_W as i32 - 1) % 9 == 0);
const _: () = assert!(WIN_TITLE_ROWS == SRC_TITLE_ROWS && WIN_TITLE_ROWS == 2);
// 代码区格数必须能被「每行 2 格」整除，不然最底下一行会缺一截。
const _: () = assert!(SRC_BODY_ROWS % 2 == 0 && SRC_BODY_ROWS > 0);
const _: () = assert!(SRC_VISIBLE_LINES * 2 == SRC_BODY_ROWS as usize);
// 源码窗的默认位置也得落在窗口网格线上（22*9 / 11*9）。
const _: () = assert!((SRC_DEFAULT_X as i32) % 9 == 0 && (SRC_DEFAULT_Y as i32) % 9 == 0);
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
/// 于是画在 x = 0 / y = 0 上的线整条落到窗口外 - 四条边框线只有右、下两条看得见。
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

/// 1px 点状虚线（竖）：每 2px 画 1px 一个点，只碰整数像素 - 状态带 / 状态栏里的分割线用它。
/// 线本身只占一个像素列，落在网格线格上，所以跟实线一样只有 1px 宽。
pub(crate) fn vline_dotted(painter: &egui::Painter, x: f32, y0: f32, y1: f32, color: egui::Color32) {
    let left = px_center(x) - 0.5;
    let mut y = y0.floor();
    while y < y1 {
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(left, y), egui::vec2(LINE_W, LINE_W)),
            egui::CornerRadius::ZERO,
            color,
        );
        y += 2.0;
    }
}

/// 1px 点状虚线（横）：同上，只占一个像素行 - UI 工程测试叠层的横网格线用它。
pub(crate) fn hline_dotted(painter: &egui::Painter, x0: f32, x1: f32, y: f32, color: egui::Color32) {
    let top = px_center(y) - 0.5;
    let mut x = x0.floor();
    while x < x1 {
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(x, top), egui::vec2(LINE_W, LINE_W)),
            egui::CornerRadius::ZERO,
            color,
        );
        x += 2.0;
    }
}

/// 「软夹取」：正常就是 v.clamp(lo, hi)，但 hi < lo（区域太窄 / 太矮，图片里塞不下标签）时
/// 返回 lo - f32::clamp 在 hi < lo 时会 panic，绘图代码绝不能因为面板被拖小就崩掉。
pub(crate) fn clamp_soft(v: f32, lo: f32, hi: f32) -> f32 {
    if !(hi >= lo) {
        lo
    } else {
        v.clamp(lo, hi)
    }
}

/// 浮动标签（纵轴数值牌 / 时间带索引牌）的上沿 y：标签高 cells(rows)，先把「以 y_center 为中心」
/// 的位置吸附到原点 origin 那套网格上最近的横网格线（上沿因此落在网格线上），再把整段夹进
/// [top_line, bot_line] 两条网格线之间。上沿落在网格线上、下边框就正好落在 rows 格之后那条
/// 网格线上 - 两条边框各占 1px，永远不可能和相邻的线（比如绘图区下沿 / 时间带上沿）叠成 2px
/// （铁律：不许 2px 线叠加）。top_line / bot_line 传边界线所在的网格线坐标（Region 边框线
/// 所在的那一行），origin 传这块窗口自己的网格原点（横向那一套的 y 分量）。
pub(crate) fn floating_label_top(
    y_center: f32,
    rows: i32,
    top_line: f32,
    bot_line: f32,
    origin: f32,
) -> f32 {
    let span = PITCH * rows as f32;
    let snap = grid_snap(y_center - cells(rows) / 2.0, origin);
    clamp_soft(snap, top_line, bot_line - span)
}

/// 手绘「叉」：正中间只占 1 个像素，四条臂各斜着往外正好画 radius 个像素。
/// 第 d 层画 (cx-d, cy-d)、(cx+d, cy-d)、(cx-d, cy+d)、(cx+d, cy+d)，d = 0..radius -
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

/// 浮动小牌子（十字光标的数值牌 / 索引牌、按钮的悬停便签）：一块纯色底 + 1px 硬边。
/// 底色和边框色由调用方给（底色各用各的，边框一律走皮肤的控制件边框色），文字由调用方叠上去。
/// 牌子摆在哪儿由调用方按网格线算好 - 铁律：它的边框要么压在相邻控件的边框线上，要么离一整格。
pub(crate) fn tag_box(
    painter: &egui::Painter,
    rect: egui::Rect,
    bg: egui::Color32,
    border: egui::Color32,
) {
    painter.rect_filled(rect, egui::CornerRadius::ZERO, bg);
    border_hard(painter, rect, border);
}

/// 1px 硬边框：画在逻辑区最外圈那一圈像素上，不占布局尺寸。
pub(crate) fn border_hard(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    hline_hard(painter, rect.min.x, rect.max.x, rect.min.y, color);
    hline_hard(painter, rect.min.x, rect.max.x, rect.max.y - LINE_W, color);
    vline_hard(painter, rect.min.x, rect.min.y, rect.max.y, color);
    vline_hard(painter, rect.max.x - LINE_W, rect.min.y, rect.max.y, color);
}

/// 登记一块可交互区域：**id 按这块矩形自己算**，不是 egui 的自动 id（allocate_rect 那一套）。
///
/// 为什么不能用自动 id：egui 的自动 id = 「这一帧第几个登记的东西」算出来的，跟登记顺序绑死。
/// 窗口区里的窗口会因「点一下激活」换绘制顺序（被点的那块画到最后 = 压在最上面），
/// 顺序一变，同一颗按钮的自动 id 就变了 - 按下的那一下算出来的 id 和松手时对不上，
/// 点击直接丢掉。所以窗口区里的控件一律走这个口子：矩形不动，id 就不动，跟层级无关。
///
/// 同一个矩形被登记两次（两块窗口摆在同一处）时，后登记的（画在上面的那块）赢 - 正好是想要的。
pub(crate) fn interact_rect(
    ui: &egui::Ui,
    rect: egui::Rect,
    sense: egui::Sense,
) -> egui::Response {
    let id = egui::Id::new((
        "rt_widget",
        rect.min.x.to_bits(),
        rect.min.y.to_bits(),
        rect.max.x.to_bits(),
        rect.max.y.to_bits(),
    ));
    ui.interact(rect, id, sense)
}

/// 硬边折线：按像素列走，每列把上一列的 y 到这一列的 y 竖着填满 - 斜线是硬像素阶梯，
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

