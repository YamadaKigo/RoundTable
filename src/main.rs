// ===========================================================================
//  【本项目铁律 · 改这个文件之前必读】
// ===========================================================================
//  1. 颜色：窗口内任何东西的颜色一律写成 6 位 #RRGGBB，在 const 里用
//     egui::Color32::from_rgb(0xRR, 0xGG, 0xBB)，注意是「两个十六进制字符一组」。
//     → 而且每个字节必须是「重复的同一个十六进制数字」，也就是严格 #aabbcc 形：
//       合法：00 11 22 33 44 55 66 77 88 99 aa bb cc dd ee ff
//             （例：#888888、#996633、#cccccc、#999999）
//       非法：80 / c0 / e0 / 12 这类两个数字不同的字节（例：#808080、#c0c0c0）
//     第三节颜色表末尾有编译期自检，色值写错格式直接编译不过。
//     禁止带透明度的写法、禁止 from_rgba / from_gray / from_black_alpha 之类。
//     全工程的色值只允许出现在第三节的颜色表里，别处不许再写字面色；
//     默认边框色 C_BORDER = #000000，默认文字色 C_TEXT = #000000，白字 = #FFFFFF。
//  2. 尺寸：内容格 8px（GRID）+ 网格线格 1px（LINE_W），节距 PITCH = 9px。
//     控件的宽/高都是 cells(n) = 9n + 1（跨 n 个内容格，含两端各 1px 网格线格）；
//     所有坐标都落在网格线上（i * PITCH），窗口尺寸也必须满足 cells(n)。
//     完整的布局模型见第二节，动任何布局之前先读那一节。
//  3. 外观：不用任何 egui 自带控件的默认外观，全部用 Painter 手绘
//     （纯色矩形 + 1px 硬边，无圆角、无阴影、无渐变、无抗锯齿）。
//     不使用 egui::Panel —— 它内部硬编码了非 8 倍数的 margin。
//  4. 文字：全界面 12px 宋体，硬边（零抗锯齿）—— 不交给 egui 光栅化，而是走
//     Windows GDI 的单色字形（GetGlyphOutlineW + GGO_BITMAP，1bpp 位图），自己贴图、
//     按整数像素落点画（见第五节）。pixels_per_point 固定 1.0，feathering 关闭。
//     全工程的文字只能走 StockApp::text 这一个入口。
//  5. 架构：窗口无边框、固定尺寸、不可缩放、不可最大化；移动窗口由自己逐帧实现
//     （绝不用 OS 的模态拖动/缩放循环，那会让 egui 停止重绘）；
//     所有跨帧状态都放在 StockApp 的字段里（egui 是即时模式）。
//  6. 注释：中文，按功能块写，不写逐行注释。
//  7. 线宽与网格线格：全界面只有 1px 的横线竖线（LINE_W = 1），线只画在网格线格里。
//     相邻控件的矩形允许在中间那条网格线格上重叠 1px（同一条线一起用），所以：
//       a) 相邻控件的本体尺寸永远相等（都是 cells(n)，不会出现 28px 配 27px）；
//       b) 两个控件之间永远只有 1px，绝不可能出现两条线紧挨着堆成 2px；
//       c) 区域分隔线也占一条网格线格，区域交界处同样只有 1px；分隔线和控件边框重合时
//          画的是同一个像素，不必特殊处理。
//  8. 窗口最外圈必须只有我们自己画的那 1px 黑边框：系统画上去的东西（DWM 投影、
//     标题栏样式位，以及 winit 留在窗口顶上的 1px 非客户区）全部自己清掉，
//     客户区撑满整个窗口（见第十三节）。
// ===========================================================================

// ===========================================================================
//  RoundTable —— Windows 桌面股票分析软件
//  当前进度：9px 节距（8px 内容格 + 1px 网格线格）+ 无边框固定窗口；
//            标题栏 / 菜单栏 / 功能区（左栏蜡烛图 + 右栏控制面板）/ 状态栏；
//            K 线支持滚轮缩放（锚点缩放，整数倍跳变）+ 左键拖拽平移。
// ===========================================================================
//  窗口策略：无边框 + 固定 cells(125) x cells(67) = 1126x604 + 不参与系统缩放；
//  移动窗口由自己逐帧实现（不用 OS 的模态拖动循环：那个循环里 egui 一帧都不重绘，
//  新露出的区域没被画过，看上去就是窗口周围一圈黑边）。
//  网格策略：内容格 8px + 网格线格 1px（节距 9px），控件尺寸一律是 cells(n)、坐标一律落在网格线上，
//  详见开头铁律和第二节。窗口尺寸同样是 cells(n)，每个像素都归在网格里，窗口边上不会露出没画过的黑边。
//
//  0.36.2 实测可用的 API（都在本机 registry 源码里确认过，不是凭记忆写的）：
//    App trait    : fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)
//                   （0.35 及以前是 update(&mut self, ctx, frame)）
//    App 清屏色   : fn clear_color(&self, &egui::Visuals) -> [f32; 4]
//    无边框       : egui::ViewportBuilder::with_decorations(false)
//    固定尺寸     : egui::ViewportBuilder::with_inner_size([w, h])
//    关掉缩放框   : egui::ViewportBuilder::with_resizable(false)
//    移动窗口     : egui::ViewportCommand::OuterPosition(Pos2)
//    最小化 / 关闭: egui::ViewportCommand::Minimized(true) / ViewportCommand::Close
//    像素吸附     : 自己写 px_center(v) = v.floor() + 0.5（见第四节）。
//                   不要用 Painter::round_to_pixel_center：它内部是
//                   (v - 0.5).round() + 0.5，在 v 正好是 0 时得到 -0.5（负数半值向
//                   绝对值大的方向舍入），画在 x = 0 / y = 0 的线会整条跑到窗口外 ——
//                   四条边框线只剩右、下两条看得见。
//    窗口外圈装饰 : 自己调 user32：清 WS_CAPTION / WS_EX_*EDGE 样式位（去 DWM 投影），
//                   再在窗口过程里拦 WM_NCCALCSIZE 把非客户区压成 0（去窗口顶上那条
//                   1px 系统边框，见第十三节）—— egui/winit 都没有现成开关
//    硬边文字     : 不用 egui 那套文字渲染（抗锯齿关不掉），自己调 gdi32：
//                   GetGlyphOutlineW + GGO_BITMAP 拿 1bpp 单色字形（见第五节）
// ===========================================================================

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::collections::HashMap;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::{Duration, Instant};

/// 行情数据层（见 src/data/）。
mod data;

// ===========================================================================
//  一、字体与字号
// ===========================================================================

// 统一字号：宋体 12px。文字由 Windows GDI 按这个字号光栅化成单色点阵字形（见第五节）。
const FONT_SIZE: f32 = 12.0;

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

const GRID: f32 = 8.0;
const LINE_W: f32 = 1.0;
const PITCH: f32 = GRID + LINE_W;

/// n 个内容格的跨度（含两端各 1px 网格线格）：9n + 1。控件宽高都用它。
const fn cells(n: i32) -> f32 {
    n as f32 * PITCH + LINE_W
}

/// 第 i 条网格线的坐标（以所在区域的左上角为原点）。
const fn grid_line(i: i32) -> f32 {
    i as f32 * PITCH
}

/// 第 i 个内容格的起点：网格线右边那 1px 之后。
const fn cell(i: i32) -> f32 {
    i as f32 * PITCH + LINE_W
}

/// 8px 的内容格没有正中间那一行（列）：中心落在半像素上，取中间两行（列）里的上一行当基线，
/// 保证符号还是整像素。28x28 这类偶数边长的框里，1px 的符号本来就不可能同时压中几何中心。
const CELL_MID: f32 = GRID / 2.0 - 1.0;

/// 把一段跨度向上吸到整数格：结果一定是 cells(n) 的形状。
fn snap_cells_up(v: f32) -> f32 {
    let n = ((v - LINE_W) / PITCH).ceil().max(0.0);
    n * PITCH + LINE_W
}

// ---- 竖向：四个区域各占几格（标题栏 / 菜单栏 / 功能区 / 状态栏）----

const TITLE_ROWS: i32 = 3;
const MENU_ROWS: i32 = 3;
const WORK_ROWS: i32 = 58;
const STATUS_ROWS: i32 = 3;

// 各区域的高度。
const TITLEBAR_H: f32 = cells(TITLE_ROWS);
const MENUBAR_H: f32 = cells(MENU_ROWS);
const STATUS_H: f32 = cells(STATUS_ROWS);

// 区域边界所在的网格线序号：上边界用它，下边界就是下一个区域的上边界（两个区域共用）。
const TITLE_LINE: i32 = 0;
const MENU_LINE: i32 = TITLE_LINE + TITLE_ROWS;
const WORK_LINE: i32 = MENU_LINE + MENU_ROWS;
const STATUS_LINE: i32 = WORK_LINE + WORK_ROWS;
const WINDOW_ROWS: i32 = STATUS_LINE + STATUS_ROWS;

// ---- 横向：窗口一共 125 格 ----

const WINDOW_COLS: i32 = 125;
// 分割条 1 格；左栏默认 89 格；右栏拿剩下的 35 格。
const SPLITTER_COLS: i32 = 1;
const CHART_COLS_DEFAULT: i32 = 89;
const PANEL_COLS: i32 = WINDOW_COLS - CHART_COLS_DEFAULT - SPLITTER_COLS;
// 两栏都不小于 35 格（316px，约等于原来定的 320px）；分割条最远拖到 SPLIT_MAX_COLS。
const PANE_MIN_COLS: i32 = 35;
const SPLIT_MAX_COLS: i32 = WINDOW_COLS - SPLITTER_COLS - PANE_MIN_COLS;

// 固定窗口客户区尺寸：1126x604 = 125 x 67 格。
const WINDOW_W: f32 = cells(WINDOW_COLS);
const WINDOW_H: f32 = cells(WINDOW_ROWS);

// 标题栏右侧系统按钮：3 格，也就是 cells(3) = 28x28；这一组贴着窗口右边框放。
const SYS_BTN_COLS: i32 = 3;
const SYS_BTN_W: f32 = cells(SYS_BTN_COLS);
const SYS_BTN_H: f32 = cells(SYS_BTN_COLS);

// 标题栏内部：程序名从第 1 条内网格线开始，可拖拽区从第 8 条网格线开始。
const TITLE_TEXT_X: f32 = grid_line(1);
const TITLE_DRAG_LINE: i32 = 8;

// 菜单栏：下拉面板宽 16 格、每行 3 格。
const MENU_PANEL_W: f32 = cells(16);
const MENU_ROW_ROWS: i32 = 3;
const MENU_ROW_H: f32 = cells(MENU_ROW_ROWS);

// 区域内部文字的左内边距：1px 线格 + 1 格内容，正好落在第一条内网格线上。
const TEXT_PAD_X: f32 = LINE_W + GRID;

// ---- 功能区里的图表 ----

// 右侧价格轴 8 格；绘图区上下各留 1 格。
const AXIS_COLS: i32 = 8;
const AXIS_W: f32 = cells(AXIS_COLS);
const PLOT_PAD_COLS: i32 = 1;

// ---- K 线的缩放与平移 ----
//  实体像素宽 scale 只能是奇数（3、5、7 … 15）：影线是固定 1px 的竖线，必须落在实体正中的
//  那一列像素上 —— 只有奇数宽的实体才有「正中那一列」（偶数宽的正中落在两列像素之间，
//  影线必然偏左或偏右一格）。滚轮 / 面板一格走 ±2 像素，全程只在奇数档之间走，踩不到偶数。
//  影线固定 1px（LINE_W），不跟着 scale 变。
//  缩到最小也保证实体有 3px：3px 是还能看出「上涨空心 / 下跌实心」的最小宽度。
//  一屏放几根 = 绘图区宽度 / 每根占的步进（实体宽 + 必要时那 1px 间隙），向下取整。

// 实体默认像素宽：5（奇数档里偏细的一档，够看出空心，一屏也能放得下足够多根）。
const BAR_SCALE_DEFAULT: u32 = 5;
/// 实体像素宽的上下限：缩到底 3px 一根（再细就看不出「上涨空心 / 下跌实心」），放到最大 15px 一根。
/// 两端都是奇数 —— 配上「一格 ±2」的步进，实体宽永远是奇数（再经 snap_scale 兜底）。
const SCALE_MIN: u32 = 3;
const SCALE_MAX: u32 = 15;
/// scale 到 3 以上才在蜡烛之间留 1px 间隙：1~2px 的实体本来就细，再留缝就散了。
/// SCALE_MIN 现在就是 3，所以这条规则实际总是成立；写成条件是为了把「什么时候留缝」写明白。
const SCALE_GAP_FROM: u32 = 3;
/// 触控板那种「按点」上报的滚轮增量，要攒够这么多点才算一格。
/// 0.36.2 的 InputOptions::line_scroll_speed 默认就是 40（Line 单位换算成点用的就是它），
/// 而 InputState 里那个字段是私有的、读不到，只能在这儿写同一个值。
const WHEEL_POINTS_PER_NOTCH: f32 = 40.0;

/// 一根 K 线在屏幕上占的横向步进：实体宽 scale +（scale >= 3 时）那 1px 间隙。
/// 「屏幕 x 落在第几根」「一屏放得下几根」「拖拽位移换算成几根」全用这一个口径。
fn bar_advance(scale: u32) -> f32 {
    let scale = snap_scale(scale);
    scale as f32 + if scale >= SCALE_GAP_FROM { LINE_W } else { 0.0 }
}

/// 把实体宽吸附到合法档位：先取奇数（| 1），再夹进 [SCALE_MIN, SCALE_MAX]（两端都是奇数）。
/// 绘图和步进一律过它 —— 实体宽奇数，影线那 1px 才落得住实体正中那一列。
fn snap_scale(scale: u32) -> u32 {
    (scale | 1).clamp(SCALE_MIN, SCALE_MAX)
}

/// 缩放 notches 格之后的实体宽：一格 = ±2 像素，只为了从当前奇数档跨到下一个奇数档。
fn scale_after(scale: u32, notches: i32) -> u32 {
    let next = scale as i64 + 2 * notches as i64;
    snap_scale(next.clamp(SCALE_MIN as i64, SCALE_MAX as i64) as u32)
}

/// 绘图区里一屏能放几根 K 线：宽度 / 步进，向下取整（至少 1 根）。
fn visible_bars(plot_w: f32, scale: u32) -> usize {
    (plot_w / bar_advance(scale)).floor().max(1.0) as usize
}

// 价格轴上从上到下均匀标几个刻度。
const TICK_COUNT: usize = 5;

// 预生成的 K 线根数：比一屏能显示的（80 多根）多得多，留给以后的平移。
const BAR_COUNT: usize = 512;

// 随机种子和起始价：固定住，每次启动看到的都是同一段行情。
const BAR_SEED: u64 = 20_261_012;
const BAR_START: f64 = 20.0;

// 十字光标的价格 / 索引标签：高 2 格；宽度按文字算，也吸到整数格。
const CROSS_LABEL_H: f32 = cells(2);

// 空闲时也定期重绘一次，让状态栏的刷新数据持续可见（单位：毫秒）。
const IDLE_REPAINT_MS: u64 = 500;

// 编译期自检：凡是「尺寸」类的常量，减去那 1px 线格后必须能被 9 整除（也就是 cells(n)）。
const _: () = assert!(PITCH == 9.0);
const _: () = assert!((WINDOW_W as i32 - 1) % 9 == 0);
const _: () = assert!((WINDOW_H as i32 - 1) % 9 == 0);
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
//  三、颜色表（一律 6 位 #RRGGBB）
// ===========================================================================

// 经典 Win9x 灰色系，全部纯色：无渐变、无圆角、无阴影。
// 每个字节都是「重复的同一个十六进制数字」，严格满足 #aabbcc 规则（见开头铁律第 1 条）。
const C_TITLE_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
const C_MENU_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
const C_WORK_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
const C_STATUS_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
const C_BTN_FACE: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
const C_BTN_HOVER: egui::Color32 = egui::Color32::from_rgb(0xEE, 0xEE, 0xEE);
const C_CLOSE_HOVER: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
const C_TEXT: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00);
const C_TEXT_WHITE: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
const C_LINE: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);
const C_BTN_EDGE: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);
// 窗口最外圈那 1px 边框的颜色：纯黑（用户要求的默认值，窗口内所有硬边框都取它）。
const C_BORDER: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00);

// 图表：绘图区白底、右侧价格轴灰底；A 股习惯是涨红跌绿；刻度参考线用浅灰。
const C_CHART_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
const C_AXIS_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
const C_UP: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
const C_DOWN: egui::Color32 = egui::Color32::from_rgb(0x00, 0x99, 0x00);
const C_TICK: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);

// 十字光标：线用中灰（白底上够醒目，又和 #999999 的轴边 / 分隔线分得开），
// 标签用浅灰底 + 黑边 + 黑字，和菜单 hover 高亮是同一套灰度。
const C_CROSS: egui::Color32 = egui::Color32::from_rgb(0x66, 0x66, 0x66);
const C_CROSS_LABEL: egui::Color32 = egui::Color32::from_rgb(0xEE, 0xEE, 0xEE);

// 两栏布局与控制面板：分割条灰、面板灰、按钮深灰硬边、按下态更暗。
const C_SPLITTER: egui::Color32 = egui::Color32::from_rgb(0xAA, 0xAA, 0xAA);
const C_PANEL_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
const C_BTN_BORDER: egui::Color32 = egui::Color32::from_rgb(0x66, 0x66, 0x66);
const C_BTN_PRESS: egui::Color32 = egui::Color32::from_rgb(0xAA, 0xAA, 0xAA);

// 编译期自检：每个字节都必须是 0x00 / 0x11 / 0x22 … 0xFF（都是 17 的整数倍），
// 也就是「两个十六进制数字相同」。以后新加色值若写成 #808080 这类，这里直接报错。
const fn color_is_aabbcc(c: egui::Color32) -> bool {
    c.r() % 17 == 0 && c.g() % 17 == 0 && c.b() % 17 == 0
}
const _: () = assert!(color_is_aabbcc(C_TITLE_BG));
const _: () = assert!(color_is_aabbcc(C_MENU_BG));
const _: () = assert!(color_is_aabbcc(C_WORK_BG));
const _: () = assert!(color_is_aabbcc(C_STATUS_BG));
const _: () = assert!(color_is_aabbcc(C_BTN_FACE));
const _: () = assert!(color_is_aabbcc(C_BTN_HOVER));
const _: () = assert!(color_is_aabbcc(C_CLOSE_HOVER));
const _: () = assert!(color_is_aabbcc(C_TEXT));
const _: () = assert!(color_is_aabbcc(C_TEXT_WHITE));
const _: () = assert!(color_is_aabbcc(C_LINE));
const _: () = assert!(color_is_aabbcc(C_BTN_EDGE));
const _: () = assert!(color_is_aabbcc(C_BORDER));
const _: () = assert!(color_is_aabbcc(C_CHART_BG));
const _: () = assert!(color_is_aabbcc(C_AXIS_BG));
const _: () = assert!(color_is_aabbcc(C_UP));
const _: () = assert!(color_is_aabbcc(C_DOWN));
const _: () = assert!(color_is_aabbcc(C_TICK));
const _: () = assert!(color_is_aabbcc(C_CROSS));
const _: () = assert!(color_is_aabbcc(C_CROSS_LABEL));
const _: () = assert!(color_is_aabbcc(C_SPLITTER));
const _: () = assert!(color_is_aabbcc(C_PANEL_BG));
const _: () = assert!(color_is_aabbcc(C_BTN_BORDER));
const _: () = assert!(color_is_aabbcc(C_BTN_PRESS));

// ===========================================================================
//  四、像素级绘制工具
// ===========================================================================

/// 坐标吸附：返回坐标所在像素的中心（整数 + 0.5）。
/// 不用 egui 的 Painter::round_to_pixel_center：emath 的实现是
/// (v - 0.5).round() + 0.5，而它在 v 正好是 0 时会返回 -0.5（负数半值向下舍入），
/// 于是画在 x = 0 / y = 0 上的线整条落到窗口外 —— 四条边框线只有右、下两条看得见。
fn px_center(v: f32) -> f32 {
    v.floor() + 0.5
}


/// 1px 硬边水平线：只占一个像素行。
fn hline_hard(painter: &egui::Painter, x0: f32, x1: f32, y: f32, color: egui::Color32) {
    let top = px_center(y) - 0.5;
    let rect = egui::Rect::from_min_size(egui::pos2(x0, top), egui::vec2(x1 - x0, LINE_W));
    painter.rect_filled(rect, egui::CornerRadius::ZERO, color);
}

/// 1px 硬边垂直线：只占一个像素列。
fn vline_hard(painter: &egui::Painter, x: f32, y0: f32, y1: f32, color: egui::Color32) {
    let left = px_center(x) - 0.5;
    let rect = egui::Rect::from_min_size(egui::pos2(left, y0), egui::vec2(LINE_W, y1 - y0));
    painter.rect_filled(rect, egui::CornerRadius::ZERO, color);
}

/// 手绘「叉」：正中间只占 1 个像素，四条臂各斜着往外正好画 radius 个像素。
/// 第 d 层画 (cx-d, cy-d)、(cx+d, cy-d)、(cx-d, cy+d)、(cx+d, cy+d)，d = 0..radius ——
/// 关于中心像素左右、上下都严格镜像，而且不会像偶数边长的画法那样在中间堆出 2x2 的实心块。
/// 不用 line_segment：斜线交给光栅器会得到带灰边的 2px 阶梯，左右也不对称。
fn paint_cross(painter: &egui::Painter, center: egui::Pos2, radius: i32, color: egui::Color32) {
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
fn border_hard(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    hline_hard(painter, rect.min.x, rect.max.x, rect.min.y, color);
    hline_hard(painter, rect.min.x, rect.max.x, rect.max.y - LINE_W, color);
    vline_hard(painter, rect.min.x, rect.min.y, rect.max.y, color);
    vline_hard(painter, rect.max.x - LINE_W, rect.min.y, rect.max.y, color);
}

// ===========================================================================
//  五、硬边文字：Windows GDI 单色字形
// ===========================================================================
//  为什么不用 egui 自己的文字渲染：egui 0.36 的字形是 vello 做的解析式抗锯齿，
//  图集里存的是覆盖率灰度，而且没有任何开关能关掉（TessellationOptions::feathering
//  的文档明确写着「不影响文字」；FontTweak 只有 hinting 和位置微调；TextOptions 里的
//  color_transfer_function 只是一条连续曲线，做不出硬边）。而 12px 宋体的汉字笔画
//  本来就只有一个像素宽，把灰度抹平只会让笔画断开，比带抗锯齿更难看。
//  所以文字走 Windows 自己那条路：GetGlyphOutlineW + GGO_BITMAP 拿到的是 1bpp 单色
//  字形位图（就是 Win9x 上那种点阵字形，笔画完整、边缘零抗锯齿），自己拼进一张贴图，
//  按整数像素落点、NEAREST 采样画出去，中间没有任何插值。
//  约定：全工程的文字只能走 StockApp::text 这一个入口，别处不许再调 painter.text。

/// GGO_BITMAP：要单色（1bpp）字形位图，而不是矢量轮廓。
const GGO_BITMAP: u32 = 1;

/// GetGlyphOutlineW 出错时的返回值。
const GDI_ERROR: u32 = 0xFFFF_FFFF;

/// 字符高度取负数 = 用「字符高度」而不是「单元格高度」，正好等于字号。
const GDI_FONT_HEIGHT: i32 = -(FONT_SIZE as i32);

/// 单色字形位图的字节数上限：12px 的字形远小于它，超过就当这个字没有。
const GLYPH_BUF_MAX: usize = 4096;

/// 字形图集的边长（像素）。512x512 装得下几千个 12px 字形。
const GLYPH_ATLAS: usize = 512;

/// 字形之间留的空隙，免得相邻字形的边缘互相渗色。
const GLYPH_GAP: usize = 1;

/// CreateFontW 的参数：默认字符集、TrueType 优先、默认裁剪精度、非抗锯齿品质、默认间距。
const DEFAULT_CHARSET: u32 = 1;
const OUT_TT_PRECIS: u32 = 4;
const CLIP_DEFAULT_PRECIS: u32 = 0;
const NONANTIALIASED_QUALITY: u32 = 3;
const DEFAULT_PITCH: u32 = 0;

/// 候选字体名：宋体优先，取不到再退到别的。
const GDI_FACES: [&str; 4] = ["宋体", "新宋体", "SimSun", "NSimSun"];

/// GLYPHMETRICS：GDI 回给我们的字形度量。
#[repr(C)]
#[derive(Clone, Copy, Default)]
#[allow(dead_code)]
struct GlyphMetrics {
    black_box_x: u32,
    black_box_y: u32,
    origin_x: i32,
    origin_y: i32,
    cell_inc_x: i16,
    cell_inc_y: i16,
}

/// MAT2：字形变换矩阵（16.16 定点），必须传单位阵。
#[repr(C)]
#[allow(dead_code)]
struct Mat2 {
    e: [i32; 4],
}

/// TEXTMETRICW：必须和 Windows 里的定义一样大（60 字节），少一个字段就会被写坏内存。
#[repr(C)]
#[derive(Default)]
#[allow(dead_code)]
struct TextMetricW {
    height: i32,
    ascent: i32,
    descent: i32,
    internal_leading: i32,
    external_leading: i32,
    ave_char_width: i32,
    max_char_width: i32,
    weight: i32,
    overhang: i32,
    digitized_aspect_x: i32,
    digitized_aspect_y: i32,
    first_char: u16,
    last_char: u16,
    default_char: u16,
    break_char: u16,
    italic: u8,
    underlined: u8,
    struck_out: u8,
    pitch_and_family: u8,
    char_set: u8,
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateCompatibleDC(hdc: isize) -> isize;
    fn DeleteDC(hdc: isize) -> i32;
    #[allow(clippy::too_many_arguments)]
    fn CreateFontW(
        c_height: i32,
        c_width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike_out: u32,
        charset: u32,
        out_precision: u32,
        clip_precision: u32,
        quality: u32,
        pitch_and_family: u32,
        face: *const u16,
    ) -> isize;
    fn SelectObject(hdc: isize, obj: isize) -> isize;
    fn DeleteObject(obj: isize) -> i32;
    fn GetTextMetricsW(hdc: isize, tm: *mut TextMetricW) -> i32;
    fn GetGlyphOutlineW(
        hdc: isize,
        ch: u32,
        format: u32,
        gm: *mut GlyphMetrics,
        buf_size: u32,
        buf: *mut u8,
        mat: *const Mat2,
    ) -> u32;
}

/// 一个光栅化好的字形：位图尺寸、相对笔位/基线的偏移、步进，以及它在图集里的落点。
#[derive(Clone, Copy)]
struct Glyph {
    w: usize,
    h: usize,
    ox: i32,
    oy: i32,
    advance: i32,
    atlas: [usize; 2],
}

/// 硬边文字引擎：一个 GDI 内存 DC + 一张字形图集贴图 + 字形缓存。
struct HardText {
    dc: isize,
    font: isize,
    ascent: i32,
    line_h: i32,
    texture: egui::TextureHandle,
    next_x: usize,
    next_y: usize,
    shelf_h: usize,
    glyphs: HashMap<char, Option<Glyph>>,
}

impl HardText {
    /// 建内存 DC、选字体、问一次行高，再开一张空图集。任何一步失败都返回 None。
    fn new(ctx: &egui::Context) -> Option<Self> {
        unsafe {
            let dc = CreateCompatibleDC(0);
            if dc == 0 {
                return None;
            }

            // 逐个试候选字体名，第一个建成功的就用它。
            let mut font = 0isize;
            for face in GDI_FACES {
                let wide: Vec<u16> = face.encode_utf16().chain(std::iter::once(0)).collect();
                let f = CreateFontW(
                    GDI_FONT_HEIGHT,
                    0,
                    0,
                    0,
                    400,
                    0,
                    0,
                    0,
                    DEFAULT_CHARSET,
                    OUT_TT_PRECIS,
                    CLIP_DEFAULT_PRECIS,
                    NONANTIALIASED_QUALITY,
                    DEFAULT_PITCH,
                    wide.as_ptr(),
                );
                if f != 0 {
                    font = f;
                    break;
                }
            }
            if font == 0 {
                DeleteDC(dc);
                return None;
            }
            SelectObject(dc, font);

            let mut tm = TextMetricW::default();
            if GetTextMetricsW(dc, &mut tm) == 0 || tm.height <= 0 {
                DeleteObject(font);
                DeleteDC(dc);
                return None;
            }

            // 图集一开始整张透明，以后每做一个新字形就用 set_partial 补一小块。
            let blank =
                egui::ColorImage::filled([GLYPH_ATLAS, GLYPH_ATLAS], egui::Color32::TRANSPARENT);
            let texture = ctx.load_texture("rt_hard_text", blank, egui::TextureOptions::NEAREST);

            eprintln!("[文字] GDI 单色字形：行高 {}px，基线 {}px", tm.height, tm.ascent);
            Some(Self {
                dc,
                font,
                ascent: tm.ascent,
                line_h: tm.height,
                texture,
                next_x: 0,
                next_y: 0,
                shelf_h: 0,
                glyphs: HashMap::new(),
            })
        }
    }

    /// 取字形（带缓存）。None 表示这个字在这套字体里没有。
    fn glyph(&mut self, c: char) -> Option<Glyph> {
        if let Some(g) = self.glyphs.get(&c) {
            return *g;
        }
        let g = self.rasterize(c);
        self.glyphs.insert(c, g);
        g
    }

    /// 一串文字的像素宽度：所有字形步进之和，是整数。
    fn width(&mut self, s: &str) -> i32 {
        s.chars().map(|c| self.glyph(c).map_or(0, |g| g.advance)).sum()
    }

    /// 让 GDI 现场光栅化一个字形：先问要多大缓冲，再取位图，最后写进图集。
    fn rasterize(&mut self, c: char) -> Option<Glyph> {
        let mat = Mat2 {
            e: [0x0001_0000, 0, 0, 0x0001_0000],
        };
        let mut gm = GlyphMetrics::default();
        unsafe {
            let need = GetGlyphOutlineW(
                self.dc,
                c as u32,
                GGO_BITMAP,
                &mut gm,
                0,
                std::ptr::null_mut(),
                &mat,
            );
            if need == GDI_ERROR || need as usize > GLYPH_BUF_MAX {
                return None;
            }

            let (w, h) = (gm.black_box_x as usize, gm.black_box_y as usize);
            let advance = gm.cell_inc_x as i32;
            if need == 0 || w == 0 || h == 0 {
                // 空格这类没有墨迹的字形：只有步进，没有位图。
                return Some(Glyph {
                    w: 0,
                    h: 0,
                    ox: 0,
                    oy: 0,
                    advance,
                    atlas: [0, 0],
                });
            }

            let mut buf = vec![0u8; need as usize];
            let got = GetGlyphOutlineW(
                self.dc,
                c as u32,
                GGO_BITMAP,
                &mut gm,
                need,
                buf.as_mut_ptr(),
                &mat,
            );
            if got == GDI_ERROR || got == 0 {
                return None;
            }
            // 每行字节数：GDI 的 1bpp 位图按 4 字节对齐，用「总字节数 / 高度」反推最稳。
            let row = (got as usize) / h;
            if row * h != got as usize || row * 8 < w {
                return None;
            }

            let (ax, ay) = self.alloc(w, h)?;
            let mut patch = egui::ColorImage::filled([w, h], egui::Color32::TRANSPARENT);
            for y in 0..h {
                for x in 0..w {
                    // 1bpp：一个字节 8 个像素，最高位在最左边。
                    if buf[y * row + x / 8] & (0x80 >> (x % 8)) != 0 {
                        patch[(x, y)] = egui::Color32::WHITE;
                    }
                }
            }
            self.texture
                .set_partial([ax, ay], patch, egui::TextureOptions::NEAREST);

            Some(Glyph {
                w,
                h,
                ox: gm.origin_x,
                oy: gm.origin_y,
                advance,
                atlas: [ax, ay],
            })
        }
    }

    /// 在图集里挤出一块 w x h 的地方：简单的货架式打包，一行填满就换下一行。
    fn alloc(&mut self, w: usize, h: usize) -> Option<(usize, usize)> {
        if w + GLYPH_GAP > GLYPH_ATLAS || h + GLYPH_GAP > GLYPH_ATLAS {
            return None;
        }
        if self.next_x + w + GLYPH_GAP > GLYPH_ATLAS {
            self.next_x = 0;
            self.next_y += self.shelf_h + GLYPH_GAP;
            self.shelf_h = 0;
        }
        if self.next_y + h + GLYPH_GAP > GLYPH_ATLAS {
            // 图集满了就不再扩：缺的字直接不画（512x512 实际用不完）。
            return None;
        }
        let pos = (self.next_x, self.next_y);
        self.next_x += w + GLYPH_GAP;
        self.shelf_h = self.shelf_h.max(h);
        Some(pos)
    }

    /// 把一串文字画到 painter 上：整数像素落点 + 顶点色，贴图 NEAREST 采样，绝不插值。
    fn draw(
        &mut self,
        painter: &egui::Painter,
        pos: egui::Pos2,
        align: egui::Align2,
        s: &str,
        color: egui::Color32,
    ) {
        let w = self.width(s);
        let x0 = match align.x() {
            egui::Align::Min => pos.x,
            egui::Align::Center => pos.x - w as f32 / 2.0,
            egui::Align::Max => pos.x - w as f32,
        };
        let top = (pos.y - self.line_h as f32 / 2.0).round();
        let baseline = top + self.ascent as f32;
        let mut pen = x0.round();

        let mut mesh = egui::Mesh::with_texture(self.texture.id());
        for c in s.chars() {
            let Some(g) = self.glyph(c) else {
                // 字体里没有这个字：画个空方框占位（宁可缺字，也不出灰边）。
                painter.rect_stroke(
                    egui::Rect::from_min_size(
                        egui::pos2(pen, top),
                        egui::vec2(FONT_SIZE, self.line_h as f32),
                    ),
                    egui::CornerRadius::ZERO,
                    egui::Stroke::new(LINE_W, color),
                    egui::StrokeKind::Inside,
                );
                pen += FONT_SIZE;
                continue;
            };
            if g.w > 0 && g.h > 0 {
                let rect = egui::Rect::from_min_size(
                    egui::pos2(pen + g.ox as f32, baseline - g.oy as f32),
                    egui::vec2(g.w as f32, g.h as f32),
                );
                let side = GLYPH_ATLAS as f32;
                let uv = egui::Rect::from_min_max(
                    egui::pos2(g.atlas[0] as f32 / side, g.atlas[1] as f32 / side),
                    egui::pos2(
                        (g.atlas[0] + g.w) as f32 / side,
                        (g.atlas[1] + g.h) as f32 / side,
                    ),
                );
                mesh.add_rect_with_uv(rect, uv, color);
            }
            pen += g.advance as f32;
        }
        if !mesh.is_empty() {
            painter.add(egui::Shape::mesh(mesh));
        }
    }
}

impl Drop for HardText {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.font);
            DeleteDC(self.dc);
        }
    }
}

/// 把逻辑布局矩形切成四块骨架：标题栏 / 菜单栏 / 功能区 / 状态栏。
/// 每块的上下边都落在网格线上（第 0 / 3 / 6 / 64 条），高度都是 cells(行数)。
/// 相邻两块在共用的那条网格线上重叠 1px（见开头铁律第 7 条）：底色接缝永远只有 1px，
/// 不会出现两条线堆成 2px，也不会有没画到的缝。
fn split_areas(layout: egui::Rect) -> [egui::Rect; 4] {
    let x0 = layout.min.x;
    let x1 = layout.max.x;
    let area = |line: i32, rows: i32| {
        egui::Rect::from_min_size(
            egui::pos2(x0, layout.min.y + grid_line(line)),
            egui::vec2(x1 - x0, cells(rows)),
        )
    };

    [
        area(TITLE_LINE, TITLE_ROWS),
        area(MENU_LINE, MENU_ROWS),
        area(WORK_LINE, WORK_ROWS),
        area(STATUS_LINE, STATUS_ROWS),
    ]
}

// ===========================================================================
//  六、标题栏自绘系统按钮
// ===========================================================================

/// 标题栏右侧只有两个按钮：最小化、关闭（本步不要缩小/放大）。
#[derive(Clone, Copy, PartialEq)]
enum SysButton {
    Minimize,
    Close,
}

impl SysButton {
    /// 在 28x28（cells(3)）的按钮矩形里手绘符号：纯色填充 + 符号，无圆角、无阴影、无渐变。
    /// 符号落在内容格和网格线上，端点全是整数像素，没有抗锯齿。
    fn paint_symbol(&self, painter: &egui::Painter, rect: egui::Rect, symbol: egui::Color32) {
        // 符号全按整像素算：先把按钮框吸到整数坐标。
        let rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x.round(), rect.min.y.round()),
            rect.size(),
        );

        match self {
            SysButton::Minimize => {
                // 一条横线：正好占中间那个内容格（8px 宽），左右严格对称；竖直方向取内容格的中间行。
                let bar = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x + cell(1), rect.min.y + cell(1) + CELL_MID),
                    egui::vec2(GRID, LINE_W),
                );
                painter.rect_filled(bar, egui::CornerRadius::ZERO, symbol);
            }
            SysButton::Close => {
                // 叉的中心取按钮正中的那个像素，半径 5 → 11x11 的叉。
                // paint_cross 保证中心只有 1 个像素、四条臂关于它左右上下严格对称。
                let center = egui::pos2(
                    rect.min.x + cell(1) + CELL_MID,
                    rect.min.y + cell(1) + CELL_MID,
                );
                paint_cross(painter, center, 5, symbol);
            }
        }
    }

}

/// 标题栏右侧两个系统按钮的矩形，从右往左排：最右边那个贴着窗口右边框，
/// 两个按钮共用中间那条网格线 —— 各自都是 cells(3) = 28x28，中间永远只有 1px。
fn sys_button_rects(bar: egui::Rect) -> [egui::Rect; 2] {
    let mut rects = [egui::Rect::NOTHING; 2];
    // 最右按钮的右边缘 = 窗口右边框那条网格线 + 1px。
    let mut right = bar.max.x;
    for i in (0..2).rev() {
        rects[i] = egui::Rect::from_min_size(
            egui::pos2(right - SYS_BTN_W, bar.min.y),
            egui::vec2(SYS_BTN_W, SYS_BTN_H),
        );
        // 下一个按钮的右边缘 = 当前按钮左边那条网格线（两个按钮共用这 1px）。
        right = rects[i].min.x + LINE_W;
    }
    rects
}

// ===========================================================================
//  七、应用状态
// ===========================================================================

/// 一次「自己实现」的窗口拖动。全程用屏幕坐标：窗口一边拖一边动，
/// 鼠标在窗口内的坐标会跟着漂，只有屏幕坐标才不会形成反馈回路（越拖越飘）。
#[derive(Clone, Copy)]
struct MoveSession {
    start_ptr: egui::Pos2,
    start_outer: egui::Rect,
    sent_pos: Option<egui::Pos2>,
}

/// 十字光标的状态：鼠标在绘图区里的位置（已经吸附到像素中心），
/// 以及由它反算出来的价格和 K 线索引（两个标签要显示这两个数）。
#[derive(Clone, Copy)]
struct Crosshair {
    x: f32,
    y: f32,
    price: f64,
    index: usize,
}

/// 界面全部可变状态。egui 是即时模式：每帧都把整个界面重新描述一遍，
/// 所以任何要跨帧保留的数据都必须存在这些字段里。
struct StockApp {
    hwnd: isize,
    client_cover_ready: bool,
    open_menu: Option<TopMenu>,
    status_msg: String,
    frames: u64,
    last_frame_time: f64,
    frame_interval_ms: f32,
    frame_ms: f32,
    fps: f32,
    drag: Option<MoveSession>,
    /// 行情数据：启动时生成；在面板上点「生成随机数据 / 重新生成」会换一段新的。
    bars: Vec<data::random_walk::Bar>,
    /// 生成行情用的种子：每点一次按固定步进推到下一个，所以每次点出来的行情都不同又可复现。
    bar_seed: u64,
    /// 每根 K 线实体的像素宽度（奇数 3..=15）：滚轮一格 ±2，画出来永远是整数像素宽、影线居中。
    scale: u32,
    /// 屏幕上第一根 K 线在 bars 里的下标：由拖拽平移改，永远夹在 [0, 总根数 - 可见根数]。
    offset: usize,
    /// 这一帧一屏放得下几根（只跟绘图区宽度和 scale 有关），每帧在 clamp_view 里重算。
    visible: usize,
    /// 拖拽平移的起点：(按下那一刻的鼠标 x, 当时的 offset)；None = 没在拖。
    /// 记起点、每帧按总位移反算，而不是逐帧累加 —— 拖到边界钳住之后再往回拖才不会飘。
    pan: Option<(f32, usize)>,
    /// 滚轮攒下来的零头（不足一格的增量），凑够一格才走一格。
    zoom_acc: f32,
    /// 硬边文字引擎：字形由 Windows GDI 现场光栅化成 1bpp 位图（见第五节）。
    text_engine: Option<HardText>,
    /// 十字光标：None = 鼠标不在绘图区里（十字线不画）。
    crosshair: Option<Crosshair>,
    /// 左栏（K 线区）宽度：分割条相对功能区左边缘的位置，永远是 8 的倍数。
    split_x: f32,
    /// 拖分割条时记下「鼠标离分割条左边缘多远」，分割条不会跳；None = 没在拖。
    split_grab: Option<f32>,
    /// 右栏控制面板的按钮：内容 + 量好的宽度，构造时算一次。
    panel_buttons: Vec<PanelButton>,
    /// 当前被按下的按钮下标（按下瞬间更暗）；None = 没有按钮按着。
    panel_press: Option<usize>,
}

impl StockApp {
    /// 构造：eframe 启动时调用一次。
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_tessellation(&cc.egui_ctx);
        configure_zero_spacing(&cc.egui_ctx);
        // 1 逻辑点 = 1 物理像素：关掉 DPI 缩放，免得 1.25x 这类非整数倍把像素网格插值糊掉。
        cc.egui_ctx.set_pixels_per_point(1.0);

        // 关系统装饰要用窗口句柄；万一这里拿不到就是 0，ui() 里还会再试一次。
        let hwnd = cc.window_handle().ok().map(|h| win32_hwnd(h.as_raw())).unwrap_or(0);
        clear_window_frame(hwnd);
        let client_cover_ready = install_client_cover(hwnd);

        let mut app = Self {
            hwnd,
            client_cover_ready,
            open_menu: None,
            status_msg: String::new(),
            frames: 0,
            last_frame_time: 0.0,
            frame_interval_ms: 0.0,
            frame_ms: 0.0,
            fps: 0.0,
            drag: None,
            bars: data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START),
            bar_seed: BAR_SEED,
            scale: BAR_SCALE_DEFAULT,
            offset: 0,
            // 首帧 ui_chart 还没跑，状态栏先按默认布局算一次
            //（左栏净宽 = 左栏 - 外框那 1px - 价格轴）。
            visible: visible_bars(CHART_PANE_W - LINE_W - AXIS_W, BAR_SCALE_DEFAULT),
            pan: None,
            zoom_acc: 0.0,
            text_engine: HardText::new(&cc.egui_ctx),
            crosshair: None,
            split_x: CHART_PANE_W,
            split_grab: None,
            panel_buttons: Vec::new(),
            panel_press: None,
        };
        // 按钮宽度要用硬边文字的实际像素宽来量，所以得等 self 建好再量（宽度跟面板位置无关，量一次就够）。
        app.build_panel_buttons();
        app
    }

    /// 统计刷新情况：帧号、两帧间隔、平滑帧率（帧耗时在 ui() 结尾测）。
    fn tick_frame(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if self.last_frame_time > 0.0 {
            let dt = (now - self.last_frame_time) as f32;
            if dt > 0.0 {
                self.frame_interval_ms = dt * 1000.0;
                let instant = 1.0 / dt;
                // 指数平滑，数字不会乱跳。
                self.fps = if self.fps > 0.0 {
                    self.fps * 0.85 + instant * 0.15
                } else {
                    instant
                };
            }
        }
        self.last_frame_time = now;
        self.frames += 1;
    }

    /// 状态栏文字：帧率 + 当前视图（缩放 / 平移 / 可见根数）+（点过菜单的话）刚才点了什么。
    /// 可见根数由 ui_chart 每帧重算，和 frame_ms 一样是「上一帧结尾」的值，差一帧显示。
    fn status_text(&self, _layout: egui::Rect) -> String {
        let mut text = format!(
            "{:.0} FPS   scale={} offset={} 可见{}根 共{}根",
            self.fps,
            self.scale,
            self.offset,
            self.visible,
            self.bars.len(),
        );
        if !self.status_msg.is_empty() {
            text.push_str("   ");
            text.push_str(&self.status_msg);
        }
        text
    }

    /// 画文字的唯一入口：全工程的文字都必须从这儿走（硬边位图字形，见第五节）。
    fn text(
        &mut self,
        painter: &egui::Painter,
        pos: egui::Pos2,
        align: egui::Align2,
        s: &str,
        color: egui::Color32,
    ) {
        if let Some(engine) = &mut self.text_engine {
            engine.draw(painter, pos, align, s, color);
        }
    }

    /// 一串文字的像素宽度：菜单项宽度就是按它算的（整数，像素级精确）。
    fn text_width(&mut self, s: &str) -> f32 {
        self.text_engine
            .as_mut()
            .map_or(0.0, |engine| engine.width(s) as f32)
    }

    /// 画四区域骨架（纯绘制，不处理交互）：底色 + 状态栏文字。
    /// 菜单栏内容在 ui_menubar 里画、功能区内容在 ui_chart 里画（它们各自要处理交互）。
    fn paint_skeleton(&mut self, painter: &egui::Painter, layout: egui::Rect, areas: [egui::Rect; 4]) {
        let [title, menu, work, status] = areas;
        painter.rect_filled(title, egui::CornerRadius::ZERO, C_TITLE_BG);
        painter.rect_filled(menu, egui::CornerRadius::ZERO, C_MENU_BG);
        painter.rect_filled(work, egui::CornerRadius::ZERO, C_WORK_BG);
        painter.rect_filled(status, egui::CornerRadius::ZERO, C_STATUS_BG);

        // 区域之间的 1px 分隔线不在这里画，见 paint_separators：它得排在标题栏、
        // 菜单栏的高亮之后，否则菜单项那块满 24px 的高亮会把上边线啃掉一条像素。

        // 状态栏：显示这一帧的画面刷新情况。
        let text = self.status_text(layout);
        self.text(
            painter,
            egui::pos2(status.min.x + TEXT_PAD_X, status.center().y),
            egui::Align2::LEFT_CENTER,
            &text,
            C_TEXT,
        );
    }

    /// 区域之间的 1px 硬边分隔线，画在网格线上（第 3 / 6 / 64 条）。
    /// 必须排在标题栏、菜单栏都画完之后调用：这几条线是区域的边框，优先级最高，
    /// 而菜单项的高亮是一块满 cells(3) 高的色块，正好压住菜单栏最上面那一行。
    fn paint_separators(&self, painter: &egui::Painter, layout: egui::Rect) {
        for line in [MENU_LINE, WORK_LINE, STATUS_LINE] {
            hline_hard(
                painter,
                layout.min.x,
                layout.max.x,
                layout.min.y + grid_line(line),
                C_LINE,
            );
        }
    }

    /// 标题栏：程序名 + 中间可拖拽区 + 右侧两个自绘按钮。
    fn ui_titlebar(&mut self, ui: &mut egui::Ui, bar: egui::Rect, outer: Option<egui::Rect>) {
        // Context / Painter 内部都是 Arc，克隆很便宜，克隆一份就不怕借用冲突。
        let ctx = ui.ctx().clone();
        let painter = ui.painter().clone();

        self.text(
            &painter,
            egui::pos2(bar.min.x + TITLE_TEXT_X, bar.center().y),
            egui::Align2::LEFT_CENTER,
            "股票分析",
            C_TEXT,
        );

        // 中间拖拽区：从第 8 条网格线开始，到系统按钮组左边那条线为止（不可见热区，不占布局）。
        // 按住就开始一次自己实现的窗口拖动（每帧发一次 OuterPosition）。
        let buttons = sys_button_rects(bar);
        let drag_x0 = bar.min.x + grid_line(TITLE_DRAG_LINE);
        let drag_rect = egui::Rect::from_min_max(
            egui::pos2(drag_x0, bar.min.y),
            egui::pos2(buttons[0].min.x.max(drag_x0), bar.max.y),
        );
        let drag_resp = ui.interact(drag_rect, egui::Id::new("rt_title_drag"), egui::Sense::drag());
        if self.drag.is_none() && drag_resp.is_pointer_button_down_on() {
            if let (Some(local), Some(o)) = (ctx.input(|i| i.pointer.latest_pos()), outer) {
                self.drag = Some(MoveSession {
                    start_ptr: egui::pos2(o.min.x + local.x, o.min.y + local.y),
                    start_outer: o,
                    sent_pos: None,
                });
            }
        }

        // 右侧两个按钮：从右往左依次是关闭、最小化。
        // 每个按钮都是 cells(3) = 28x28；相邻两个共用中间那条网格线，中间永远只有 1px。
        // 四条边一律全画：上边压在窗口顶边框上、下边压在区域分隔线上、右边压在窗口右边框上，
        // 反复画同一条 1px 线不会变粗（后画的压在上面，颜色以最后画的为准）。
        let kinds = [SysButton::Minimize, SysButton::Close];
        for (i, kind) in kinds.iter().enumerate() {
            let rect = buttons[i];
            let resp = ui.interact(rect, egui::Id::new(("rt_sys_btn", i)), egui::Sense::click());
            let hovered = resp.hovered();
            // 关闭键 hover 变红、叉叉变白；最小化按钮 hover 只把底色变浅。
            let close = *kind == SysButton::Close;
            let face = if hovered && close {
                C_CLOSE_HOVER
            } else if hovered {
                C_BTN_HOVER
            } else {
                C_BTN_FACE
            };
            let symbol = if hovered && close { C_TEXT_WHITE } else { C_TEXT };

            painter.rect_filled(rect, egui::CornerRadius::ZERO, face);
            border_hard(&painter, rect, C_BTN_EDGE);
            kind.paint_symbol(&painter, rect, symbol);

            if resp.clicked() {
                match kind {
                    SysButton::Minimize => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                    SysButton::Close => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
        }
    }

    /// 推进正在进行的窗口拖动：每帧算出窗口该在哪，用 OuterPosition 挪过去。
    fn step_move(&mut self, ctx: &egui::Context, outer: Option<egui::Rect>) {
        let Some(mut session) = self.drag else {
            return;
        };
        if !ctx.input(|i| i.pointer.primary_down()) {
            self.drag = None;
            return;
        }
        if let (Some(local), Some(o)) = (ctx.input(|i| i.pointer.latest_pos()), outer) {
            let ptr = egui::pos2(o.min.x + local.x, o.min.y + local.y);
            let d = ptr - session.start_ptr;
            // 永远以按下那一刻的窗口位置为基准，避免累积误差。
            let new_pos = session.start_outer.min + d;
            // 取整到整物理像素：带小数会让「位置变没变」的判断永远成立。
            let new_pos = egui::pos2(new_pos.x.round(), new_pos.y.round());
            // 位置真的变了才发命令，重复发同一个位置等于白跑一帧。
            if session.sent_pos != Some(new_pos) {
                session.sent_pos = Some(new_pos);
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(new_pos));
            }
        }
        self.drag = Some(session);
    }
}

// ===========================================================================
//  八、菜单栏与下拉菜单
// ===========================================================================
//  布局规则：菜单项宽度 = 文字宽度 + 左右各 9px 内边距，再向上吸到整数格
//  （两个汉字 12px 字 = 24px 宽 → cells(5) = 46px）；高度就是菜单栏的 cells(3) = 28px。
//  下拉面板宽 cells(16)、每行 cells(3)，面板上边和菜单栏共用一条网格线，左边和对应菜单项对齐。
//  交互：点菜单项展开/收起；已展开时 hover 到别的菜单就自动切过去；
//  点面板里的条目 → 收起 + 状态栏回显；点菜单栏和面板以外的地方 → 收起。

/// 菜单栏上的 6 个菜单（声明顺序不是显示顺序，显示顺序见 ALL）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum TopMenu {
    File,
    Data,
    Analyze,
    Layout,
    Tools,
    Help,
}

impl TopMenu {
    /// 显示顺序。
    const ALL: [TopMenu; 6] = [
        TopMenu::File,
        TopMenu::Data,
        TopMenu::Analyze,
        TopMenu::Layout,
        TopMenu::Tools,
        TopMenu::Help,
    ];

    /// 菜单个数（数组长度和地方都取它，省得写死两个地方）。
    const COUNT: usize = 6;

    /// 菜单栏上显示的文字。
    fn title(self) -> &'static str {
        match self {
            TopMenu::File => "文件",
            TopMenu::Data => "数据",
            TopMenu::Analyze => "分析",
            TopMenu::Layout => "布局",
            TopMenu::Tools => "工具",
            TopMenu::Help => "帮助",
        }
    }

    /// 下拉面板里的条目。本步只做展示：点了只在状态栏回显文字。
    fn items(self) -> &'static [&'static str] {
        match self {
            TopMenu::File => &["新建", "打开", "保存", "退出"],
            TopMenu::Data => &["抓取行情", "随机游走", "导入CSV", "数据源设置"],
            TopMenu::Analyze => &["指标设置", "画线工具", "回测"],
            TopMenu::Layout => &["单区", "左右分", "上下分", "四宫格"],
            TopMenu::Tools => &["设置", "日志"],
            TopMenu::Help => &["关于"],
        }
    }

    /// 在 ALL 里的下标：下拉面板要拿它对齐到对应菜单项的左边。
    fn index(self) -> usize {
        TopMenu::ALL.iter().position(|m| *m == self).unwrap_or(0)
    }
}

impl StockApp {
    /// 菜单项宽度 = 文字宽度 + 左右各 9px 内边距，向上吸到整数格（最小 3 格）。
    /// 字宽是硬边字形的整数步进之和（两个汉字 12px 字是 24），吸附后一定落在网格上。
    fn menu_item_width(&mut self, title: &str) -> f32 {
        snap_cells_up(self.text_width(title) + 2.0 * TEXT_PAD_X).max(cells(3))
    }

    /// 从左往右排开所有菜单项。相邻两项共用中间那条网格线（步进 = 宽 - 1px）：
    /// 各自尺寸照样是 cells(n)、完全相等，高亮块碰不到邻居的边框，中间也永远只有 1px。
    fn menu_item_rects(&mut self, bar: egui::Rect) -> [egui::Rect; TopMenu::COUNT] {
        let mut rects = [egui::Rect::NOTHING; TopMenu::COUNT];
        let mut x = bar.min.x;
        for (i, menu) in TopMenu::ALL.iter().enumerate() {
            let w = self.menu_item_width(menu.title());
            rects[i] =
                egui::Rect::from_min_size(egui::pos2(x, bar.min.y), egui::vec2(w, bar.height()));
            x += w - LINE_W;
        }
        rects
    }

    /// 下拉面板的矩形：上边和菜单栏共用那条网格线（anchor 的最下一行），
    /// 左边和菜单项对齐，高度 = 行数 x 3 格（相邻两行同样共用网格线）。
    fn menu_panel_rect(anchor: egui::Rect, open: TopMenu) -> egui::Rect {
        let rows = MENU_ROW_ROWS * open.items().len() as i32;
        egui::Rect::from_min_size(
            egui::pos2(anchor.min.x, anchor.max.y - LINE_W),
            egui::vec2(MENU_PANEL_W, cells(rows)),
        )
    }
}

impl StockApp {
    /// 菜单栏：点击展开/收起、hover 自动切换，再画高亮和文字，最后处理「点别处收起」。
    /// 返回值是当前展开菜单的锚点矩形和菜单本身（没展开就是 None）：下拉面板交给
    /// 调用方在区域分隔线之后画，那样面板以外的任何东西都盖不住那几条 1px 边框。
    fn ui_menubar(&mut self, ui: &mut egui::Ui, bar: egui::Rect) -> Option<(egui::Rect, TopMenu)> {
        let painter = ui.painter().clone();
        let rects = self.menu_item_rects(bar);

        // 1) 交互：点一下展开、再点一下收起；已经展开时 hover 到别的菜单就切过去。
        let mut hovered = [false; TopMenu::COUNT];
        for (i, menu) in TopMenu::ALL.iter().enumerate() {
            let resp = ui.allocate_rect(rects[i], egui::Sense::click());
            hovered[i] = resp.hovered();
            if resp.clicked() {
                self.open_menu = if self.open_menu == Some(*menu) {
                    None
                } else {
                    Some(*menu)
                };
            } else if hovered[i] && self.open_menu.is_some_and(|open| open != *menu) {
                self.open_menu = Some(*menu);
            }
        }

        // 2) 画菜单栏：底色由骨架画，这里只补 hover / 展开态的高亮和文字。
        for (i, menu) in TopMenu::ALL.iter().enumerate() {
            if hovered[i] || self.open_menu == Some(*menu) {
                painter.rect_filled(rects[i], egui::CornerRadius::ZERO, C_BTN_HOVER);
            }
            self.text(
                &painter,
                rects[i].center(),
                egui::Align2::CENTER_CENTER,
                menu.title(),
                C_TEXT,
            );
        }

        // 3) 点在菜单栏和下拉面板以外的地方 → 收起面板。
        if self.open_menu.is_some() {
            let panel = self
                .open_menu
                .map(|open| Self::menu_panel_rect(rects[open.index()], open));
            let outside = ui.ctx().input(|i| {
                i.pointer.primary_clicked()
                    && i.pointer
                        .interact_pos()
                        .is_some_and(|p| !bar.contains(p) && !panel.is_some_and(|r| r.contains(p)))
            });
            if outside {
                self.open_menu = None;
            }
        }

        self.open_menu.map(|open| (rects[open.index()], open))
    }
}

impl StockApp {
    /// 下拉面板：先铺底，再画每一条（hover 高亮 + 文字 + 点击回显），
    /// 最后把 1px 硬边压在最上面 —— 先画边的话会被 hover 高亮啃掉一条像素。
    fn ui_menu_panel(&mut self, ui: &mut egui::Ui, anchor: egui::Rect, open: TopMenu) {
        let painter = ui.painter().clone();
        let panel = Self::menu_panel_rect(anchor, open);
        painter.rect_filled(panel, egui::CornerRadius::ZERO, C_MENU_BG);

        let mut clicked = false;
        for (j, label) in open.items().iter().enumerate() {
            // 相邻两行共用一条网格线：步进 grid_line(3) = 27px，行高 cells(3) = 28px。
            let row = egui::Rect::from_min_size(
                egui::pos2(panel.min.x, panel.min.y + grid_line(j as i32 * MENU_ROW_ROWS)),
                egui::vec2(MENU_PANEL_W, MENU_ROW_H),
            );
            let resp = ui.allocate_rect(row, egui::Sense::click());
            if resp.hovered() {
                painter.rect_filled(row, egui::CornerRadius::ZERO, C_BTN_HOVER);
            }
            self.text(
                &painter,
                egui::pos2(row.min.x + TEXT_PAD_X, row.center().y),
                egui::Align2::LEFT_CENTER,
                *label,
                C_TEXT,
            );
            if resp.clicked() {
                self.status_msg = format!("你点击了：{label}");
                clicked = true;
            }
        }

        // 1px 硬边最后画：先画边的话会被 hover 高亮和行底色啃掉一条像素。
        border_hard(&painter, panel, C_BORDER);

        // 点了条目就收起面板：这一步排在「点外面」判断之前，此刻鼠标还在面板里。
        if clicked {
            self.open_menu = None;
        }
    }
}

// ===========================================================================
//  九、功能区的蜡烛图
// ===========================================================================
//  绘图区在左、价格轴在右：价格轴占最右 8 个内容格（cells(8) = 73px），两者高度都等于功能区高度。
//  价格映射：取「可见 K 线」的最高价和最低价，线性映射到绘图区高度，上下各留 1 格，
//  结果吸到像素中心（整数行 + 0.5），所以实体和影线都落在整像素行上。
//  索引映射：绘图区宽度按 1 格一根均分，每根画在自己格子的正中。
//  只遍历可见的那一段（offset .. offset + visible），不碰全量数据。
//  十字光标：鼠标在绘图区里才画（离开就消失），两条 1px 硬边线严格限制在绘图区内部；
//  鼠标坐标先吸附到像素中心，再反算价格（y_of 的逆）和索引；右侧价格标签、
//  底部索引标签都用纯色矩形 + 1px 黑边，四边落在整数像素上。

/// 画一根蜡烛：1px 竖影线 + 实体（上涨空心 / 下跌实心）。参数全是像素：
/// left = 实体左边缘（整数像素），bar_w = 实体像素宽（奇数），
/// y_open / y_close / y_high / y_low = 四价换算好的像素中心 y。
/// 影线的中心列 = left + (bar_w - 1) / 2：bar_w 是奇数时它才落在整整一列像素的中间，
/// 实体左右两边的像素列数严格相等（tests::wick_is_centered_in_the_body_for_every_scale 盯着这条）。
/// 参数多是因为它只管画：几何全在调用处算好（那种「一堆像素坐标」的函数这里一律照旧）。
#[allow(clippy::too_many_arguments)]
fn paint_candle(
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
        painter.rect_filled(inner, egui::CornerRadius::ZERO, C_CHART_BG);
    }
}

impl StockApp {
    /// 功能区左栏 →（绘图区, 价格轴）：价格轴占最右边 AXIS_COLS 个内容格，
    /// 两者共用中间那条网格线，各自尺寸都是整数格。
    /// 绘图区左边缘再让开 1px：最左那一列是窗口外框的线格（铁律第 2/8 条），
    /// 不让开的话，最左那根蜡烛会被黑边框啃掉一列，看着就不居中了。
    fn chart_rects(work: egui::Rect) -> (egui::Rect, egui::Rect) {
        let edge = work.max.x - AXIS_W;
        let plot = egui::Rect::from_min_max(
            egui::pos2(work.min.x + LINE_W, work.min.y),
            egui::pos2(edge, work.max.y),
        );
        let axis = egui::Rect::from_min_max(egui::pos2(edge, work.min.y), work.max);
        (plot, axis)
    }

    /// 画功能区左栏：白底绘图区 + 右侧灰色价格轴 + 蜡烛 + 刻度 + 十字光标。
    /// 缩放 / 平移也在这里处理：鼠标位置这一帧只取一次，先动视图、再按最终的
    /// scale / offset 出图，十字光标的索引标签才不会慢一帧。
    fn ui_chart(&mut self, ui: &mut egui::Ui, work: egui::Rect) {
        let painter = ui.painter().clone();
        let (plot, axis) = Self::chart_rects(work);
        painter.rect_filled(plot, egui::CornerRadius::ZERO, C_CHART_BG);
        painter.rect_filled(axis, egui::CornerRadius::ZERO, C_AXIS_BG);

        // 绘图区里真正允许画东西的范围：上下各留 1 格（那条网格线也留出来）。
        let pad = grid_line(PLOT_PAD_COLS);
        let body = egui::Rect::from_min_max(
            egui::pos2(plot.min.x, plot.min.y + pad),
            egui::pos2(plot.max.x, plot.max.y - pad),
        );

        // 鼠标位置这一帧只取一次：缩放锚点、平移起点、十字光标都用它，互相不覆盖状态。
        // （拖拽中鼠标滑出绘图区也还留着它，十字线不会闪掉。）
        let hover = ui.input(|i| i.pointer.hover_pos());

        // ---- 视图操作：滚轮缩放（锚点缩放）+ 左键拖拽平移 ----
        let notches = self.take_wheel_notches(ui, plot, hover);
        if notches != 0 {
            if let Some(p) = hover {
                self.zoom_around(p.x - plot.min.x, notches);
            }
        }
        self.pan_by_drag(ui, plot);
        // 视图动完再重算一屏放几根，并把 offset 夹回合法范围（边界处钳住，拖不出空白）。
        self.clamp_view(plot.width());

        let visible = self.visible;
        if visible == 0 {
            return;
        }

        // 价格范围 = 可见 K 线的最高价 / 最低价（只扫这一段）。
        let (hi, lo) = {
            let slice = &self.bars[self.offset..self.offset + visible];
            let hi = slice.iter().fold(f64::MIN, |acc, bar| acc.max(bar.h));
            let lo = slice.iter().fold(f64::MAX, |acc, bar| acc.min(bar.l));
            (hi, lo)
        };
        let span = (hi - lo).max(f64::EPSILON);

        // 价格 → 像素中心：最高价贴 body 最上一行，最低价贴最下一行。
        let y_of = |price: f64| -> f32 {
            let t = ((hi - price) / span) as f32;
            px_center(body.min.y + t * (body.height() - LINE_W))
        };

        // 刻度：从上到下 5 条横向参考线铺满绘图区，价格数字右对齐写在轴内。
        for k in 0..TICK_COUNT {
            let t = k as f32 / (TICK_COUNT - 1) as f32;
            let y = y_of(hi - (hi - lo) * t as f64);
            hline_hard(&painter, plot.min.x, plot.max.x, y, C_TICK);
            let label = format!("{:.2}", hi - (hi - lo) * t as f64);
            self.text(
                &painter,
                egui::pos2(axis.max.x - TEXT_PAD_X, y),
                egui::Align2::RIGHT_CENTER,
                &label,
                C_TEXT,
            );
        }

        // 蜡烛：横向按「实体 scale 像素 + 必要时那 1px 间隙」一根挨一根铺开 ——
        // 第 i 根的实体左边 = 绘图区左边缘 + i * 步进，都是整数像素，直接落在像素栅格上，
        // 所以实体永远是整数像素宽（不会出现 1.5px），一屏正好放得下 visible 根。
        // 每根交给 paint_candle：先影线（固定 1px 竖线，最高→最低，画在实体正中那一列），
        // 再实体（开盘→收盘）；实体宽是奇数（snap_scale 兜底），正中那一列才存在。
        let advance = bar_advance(self.scale);
        let bar_w = snap_scale(self.scale) as f32;
        let slice = &self.bars[self.offset..self.offset + visible];
        for (i, bar) in slice.iter().enumerate() {
            let left = (plot.min.x + i as f32 * advance).round();
            let rising = bar.c >= bar.o;
            let color = if rising { C_UP } else { C_DOWN };
            paint_candle(
                &painter,
                left,
                bar_w,
                rising,
                color,
                y_of(bar.o),
                y_of(bar.c),
                y_of(bar.h),
                y_of(bar.l),
            );
        }

        // 价格轴左边那条 1px 硬边最后画，压在蜡烛上面。
        vline_hard(&painter, axis.min.x, axis.min.y, axis.max.y, C_LINE);

        // ---- 十字光标 ----
        // 用帧首取好的 hover：只有落在绘图区里才算数，鼠标在价格轴、菜单栏、状态栏或
        // 窗口外面时直接清空（十字线立刻消失）；正在拖拽平移时不清空、坐标夹进绘图区，
        // 十字线一直跟着（否则往左右拖出绘图区那一瞬间线就闪没了）。
        let offset = self.offset;
        let dragging = self.pan.is_some();
        let point = hover.or_else(|| {
            if dragging {
                ui.input(|i| i.pointer.latest_pos())
            } else {
                None
            }
        });
        self.crosshair = point
            .filter(|p| dragging || plot.contains(*p))
            .map(|p| {
                // 鼠标坐标是浮点，先吸附到像素中心再画：否则 1px 的线会横跨两列像素、
                // 被渲染成 2px 灰边。横竖都夹在绘图区里，右边界给价格轴那条分隔线留位置。
                let x = px_center(p.x.clamp(plot.min.x, plot.max.x - LINE_W));
                let y = px_center(p.y.clamp(plot.min.y, plot.max.y - LINE_W));
                // 价格 = y_of 的逆运算（同一个 body 高度、同一个 1px 修正），
                // 所以标签上的数字和十字线所在的高度严丝合缝。
                let t = ((y - body.min.y) / (body.height() - LINE_W)).clamp(0.0, 1.0);
                // 索引 = 这一列落在第几根里（按步进算）+ 屏幕起点 offset，越界夹在可见范围内。
                let slot = ((x - plot.min.x) / advance).floor().max(0.0) as usize;
                Crosshair {
                    x,
                    y,
                    price: hi - span * t as f64,
                    index: offset + slot.min(visible - 1),
                }
            });

        // 再画：两条 1px 硬边线（只落在绘图区内部）+ 右侧价格标签 + 底部索引标签。
        if let Some(cross) = self.crosshair {
            vline_hard(&painter, cross.x, plot.min.y, plot.max.y, C_CROSS);
            hline_hard(&painter, plot.min.x, plot.max.x, cross.y, C_CROSS);

            // 价格标签：贴在价格轴里右对齐（和刻度数字同一条基准线），底色盖住底下的刻度。
            let price_text = format!("{:.2}", cross.price);
            let w = self.cross_label_w(&price_text).min(axis.width() - 2.0 * LINE_W);
            let x0 = (axis.max.x - TEXT_PAD_X - w)
                .round()
                .clamp(axis.min.x + LINE_W, axis.max.x - w - LINE_W);
            let y0 = (cross.y - CROSS_LABEL_H / 2.0)
                .round()
                .clamp(plot.min.y, plot.max.y - CROSS_LABEL_H);
            self.cross_label(&painter, &price_text, egui::pos2(x0, y0), w);

            // 索引标签：水平方向居中在竖线上，垂直方向贴绘图区下沿。
            let index_text = format!("{}", cross.index);
            let w = self.cross_label_w(&index_text).min(plot.width());
            let x0 = (cross.x - w / 2.0)
                .round()
                .clamp(plot.min.x, plot.max.x - w);
            self.cross_label(
                &painter,
                &index_text,
                egui::pos2(x0, plot.max.y - CROSS_LABEL_H),
                w,
            );
        }
    }

    /// 这一帧滚轮滚了几格（正 = 往上滚 = 放大）。只有鼠标在绘图区里才算数：
    /// 在控制面板或别处滚滚轮，K 线不动，也不会把增量攒在字段里留着下次发作。
    /// 不足一格的零头（触控板那种按点上报的增量）存在 zoom_acc 里，下一帧接着凑。
    fn take_wheel_notches(
        &mut self,
        ui: &egui::Ui,
        plot: egui::Rect,
        hover: Option<egui::Pos2>,
    ) -> i32 {
        if !hover.is_some_and(|p| plot.contains(p)) {
            self.zoom_acc = 0.0;
            return 0;
        }
        let mut acc = self.zoom_acc;
        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::MouseWheel { unit, delta, .. } = event {
                    // Windows 上一格就是 1 行：winit 把 WM_MOUSEWHEEL 的 ±120 除以 120。
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
    /// 最后把这根 K 线的下标取整 —— offset 是整数索引，只能整根整根地挪。
    fn zoom_around(&mut self, x_rel: f32, notches: i32) {
        let old = snap_scale(self.scale);
        let new = scale_after(old, notches);
        if new == old {
            return;
        }
        let mouse_bar = self.offset as f32 + x_rel / bar_advance(old);
        self.scale = new;
        self.offset = (mouse_bar - x_rel / bar_advance(new)).round().max(0.0) as usize;
    }

    /// 左键拖拽平移：在绘图区按住左键左右拖，offset 按「位移 / 步进」根变化（取整）。
    /// 向右拖 = 看更早的数据（offset 变小），向左拖 = 看更晚的数据（offset 变大）。
    /// 记的是按下那一刻的鼠标 x 和 offset，每帧按总位移反算；拖到边界被钳住之后再往回拖，
    /// 视图会从钳住的位置接着走，不会「飘」。
    fn pan_by_drag(&mut self, ui: &egui::Ui, plot: egui::Rect) {
        let resp = ui.interact(plot, egui::Id::new("rt_chart_pan"), egui::Sense::drag());
        let down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
        if resp.drag_started_by(egui::PointerButton::Primary) {
            let x = ui.input(|i| i.pointer.latest_pos()).map_or(0.0, |p| p.x);
            self.pan = Some((x, self.offset));
        }
        if resp.dragged_by(egui::PointerButton::Primary) {
            if let (Some((start_x, start_offset)), Some(p)) =
                (self.pan, ui.input(|i| i.pointer.latest_pos()))
            {
                let bars = (((p.x - start_x) / bar_advance(self.scale)) as f64).round();
                self.offset = (start_offset as f64 - bars).max(0.0) as usize;
            }
        }
        // 左键一松就结束这次平移（不只认这一帧的 drag_stopped）：pan 只在这个函数里维护，
        // 而且「还按着左键」是它唯一的存活条件 —— 十字光标靠它判断自己该不该继续显示。
        if !down {
            self.pan = None;
        }
    }

    /// 重算一屏放几根，并把 offset 夹回 [0, 总根数 - 可见根数]：边界处钳住，
    /// 既拖不出左边的空白，也不会让最后一根后面露出空槽。
    fn clamp_view(&mut self, plot_w: f32) {
        self.visible = visible_bars(plot_w, self.scale).min(self.bars.len());
        self.offset = self.offset.min(self.bars.len().saturating_sub(self.visible));
    }

    /// 十字光标标签的宽度 = 文字宽 + 左右各 9px 内边距，向上吸到整数格（最小 2 格）。
    fn cross_label_w(&mut self, text: &str) -> f32 {
        snap_cells_up(self.text_width(text) + 2.0 * TEXT_PAD_X).max(cells(2))
    }

    /// 画一个十字光标标签：纯色底 + 1px 黑边 + 居中文字，位置和尺寸都落在整数像素上。
    fn cross_label(&mut self, painter: &egui::Painter, text: &str, pos: egui::Pos2, w: f32) {
        let rect = egui::Rect::from_min_size(pos, egui::vec2(w, CROSS_LABEL_H));
        painter.rect_filled(rect, egui::CornerRadius::ZERO, C_CROSS_LABEL);
        border_hard(painter, rect, C_BORDER);
        self.text(
            painter,
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            C_TEXT,
        );
    }
}

// ===========================================================================
//  十、两栏布局、分割条与控制面板（自绘按钮）
// ===========================================================================
//  功能区横向切成三块：左栏 K 线区 89 格 + 分割条 1 格 + 右栏控制面板 35 格 = 125 格，
//  正好等于窗口宽（三块相邻处共用网格线，见开头铁律第 7 条）。
//  分割条：纯色 + 1px 深灰硬边，按住可以左右拖，步进 1 格（9px 节距），两栏都不小于 35 格。
//  控制面板：左边距和上边距都是 1 格；按钮从上往下排，相邻按钮共用中间那条网格线。
//  按钮：高 cells(3) = 28px；宽 = 内容宽 + 左右各 9px，向上吸到整数格（最多 24 格）；
//  三种状态三块纯色（普通灰 / 悬停稍亮 / 按下更暗）+ 1px 深灰硬边，无圆角、无阴影、无渐变。
//  所有按钮都走 draw_button 这一个函数，别处不要再抄一份外观。

/// 左栏 K 线区的默认宽度（也是分割条的初始位置）：89 格。
const CHART_PANE_W: f32 = cells(CHART_COLS_DEFAULT);

/// 右栏控制面板的默认宽度：35 格。面板实际宽度 = 功能区右边到分割条，
/// 拖分割条把右栏拉宽时，多出来的部分照样铺面板底色。
const PANEL_W: f32 = cells(PANEL_COLS);

/// 两栏的最小宽度：35 格。
const PANE_MIN_W: f32 = cells(PANE_MIN_COLS);

/// 分割条宽度：1 格，也就是 cells(1) = 10px（含两端各 1px 网格线格）。
const SPLITTER_W: f32 = cells(SPLITTER_COLS);

/// 分割条能拖到的最右边：再往右右栏就低于最小宽度了。
const SPLIT_MAX: f32 = cells(SPLIT_MAX_COLS);

/// 按钮高 3 格；相邻按钮共用中间那条网格线，所以往下排的步进是 grid_line(3) = 27px。
const BTN_ROWS: i32 = 3;
const BTN_H: f32 = cells(BTN_ROWS);
const BTN_STEP: f32 = grid_line(BTN_ROWS);

/// 按钮最大宽度：24 格（16 个汉字 = 192px，加左右各 9px 内边距）。
const BTN_MAX_COLS: i32 = 24;
const BTN_MAX_W: f32 = cells(BTN_MAX_COLS);

/// 图标区固定 2 格内容（16x16），图标和文字之间再留 1 格（8px）。
const ICON_W: f32 = 2.0 * GRID;
const ICON_GAP: f32 = GRID;

// 编译期自检：这些东西也必须站在网格上（尺寸都是 cells(n) = 9n + 1 的形状）。
const _: () = assert!((CHART_PANE_W as i32 - 1) % 9 == 0);
const _: () = assert!((PANEL_W as i32 - 1) % 9 == 0);
const _: () = assert!((PANE_MIN_W as i32 - 1) % 9 == 0);
const _: () = assert!((SPLITTER_W as i32 - 1) % 9 == 0);
const _: () = assert!((SPLIT_MAX as i32 - 1) % 9 == 0);
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
enum ButtonState {
    Idle,
    Hover,
    Pressed,
}

impl ButtonState {
    /// 底色：三种状态各一块纯色，只有明暗差别。
    fn face(self) -> egui::Color32 {
        match self {
            ButtonState::Idle => C_BTN_FACE,
            ButtonState::Hover => C_BTN_HOVER,
            ButtonState::Pressed => C_BTN_PRESS,
        }
    }
}

/// 手绘图标：都画在 16x16 的图标区里，只用整数像素的矩形和直线，没有抗锯齿。
#[derive(Clone, Copy)]
enum IconKind {
    /// 叉（两条对角线）。
    Cross,
    /// 三角（向下的实心三角）。
    Triangle,
}

impl IconKind {
    /// 名字：图标按钮没有文字，状态栏回显就用它。
    fn name(self) -> &'static str {
        match self {
            IconKind::Cross => "叉",
            IconKind::Triangle => "三角",
        }
    }

    /// 在给定的 16x16 矩形里画自己：先把矩形吸到整数像素，图形才可能左右对称。
    fn paint(self, painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
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
            IconKind::Triangle => {
                // 向下的实心三角：从下往上一行比一行宽 2 像素。
                // 宽度取偶数、中线落在图标第 7/8 列之间的格线上，左右才严格对称。
                let rows = (ICON_W / 2.0) as i32;
                for row in 0..rows {
                    let y = rect.max.y - LINE_W - row as f32;
                    let block = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x + 7.0 - row as f32, y),
                        egui::vec2(2.0 * (row + 1) as f32, LINE_W),
                    );
                    painter.rect_filled(block, egui::CornerRadius::ZERO, color);
                }
            }
        }
    }
}

/// 按钮内容：纯文字 / 纯图标 / 图标 + 文字（后两种先留好，后面的工具栏直接用）。
#[allow(dead_code)]
#[derive(Clone)]
enum ButtonContent {
    Text(String),
    Icon(IconKind),
    IconText(IconKind, String),
}

impl ButtonContent {
    /// 状态栏回显用的名字。
    fn label(&self) -> String {
        match self {
            ButtonContent::Text(text) => text.clone(),
            ButtonContent::Icon(kind) => kind.name().to_owned(),
            ButtonContent::IconText(_, text) => text.clone(),
        }
    }
}

/// 控制面板按钮的动作。行情生成和缩放 / 平移都是真做的；配色切换还没做，先按回显处理。
#[derive(Clone, Copy, PartialEq)]
enum PanelAction {
    /// 换一个新种子重新生成行情，并把视图（平移 / 缩放）归零。
    NewRandomWalk,
    /// 切换涨跌配色（还没做）。
    ToggleColors,
    /// 放大 / 缩小一格：跟滚轮同一套范围（3..=15 的奇数档），只是没有鼠标锚点。
    ZoomIn,
    ZoomOut,
    /// 重置视图：scale 回到最小、offset 回到最前面那根。
    ResetView,
    /// 把当前状态打到状态栏。
    PrintState,
}

/// 控制面板里的一个按钮：内容 + 动作 + 已经量好的宽度（宽度和面板位置无关，开机算一次就够）。
struct PanelButton {
    content: ButtonContent,
    action: PanelAction,
    w: f32,
}

impl StockApp {
    /// 功能区 →（左栏, 分割条, 右栏）三块。三块相邻处共用一条网格线：
    /// 分割条左边那条线就是左栏的右边框，右边那条线就是右栏的左边框。
    fn work_columns(work: egui::Rect, split_x: f32) -> [egui::Rect; 3] {
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

    /// 分割条：纯色 + 1px 硬边，按住左右拖，一步 1 格（9px 节距）；两栏都不小于 35 格。
    fn ui_splitter(&mut self, ui: &mut egui::Ui, bar: egui::Rect) {
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
                self.split_x = snap_cells_up(want).clamp(PANE_MIN_W, SPLIT_MAX);
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

    /// 按钮宽度 = 内容宽 + 左右各 9px 内边距，向上吸到整数格，最多 24 格。
    /// 内容宽：纯文字 = 文字宽；纯图标 = 2 格；图标 + 文字 = 2 格 + 1 格 + 文字宽。
    fn button_width(&mut self, content: &ButtonContent) -> f32 {
        let inner = match content {
            ButtonContent::Text(text) => self.text_width(text),
            ButtonContent::Icon(_) => ICON_W,
            ButtonContent::IconText(_, text) => ICON_W + ICON_GAP + self.text_width(text),
        };
        snap_cells_up(inner + 2.0 * TEXT_PAD_X).min(BTN_MAX_W)
    }

    /// 通用按钮：登记点击 → 按状态铺底色和 1px 深灰硬边 → 按内容画图标 / 文字，
    /// 返回这一帧是否被点击。全工程的按钮都从这儿走，别处不要再写一份外观。
    fn draw_button(
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
            ButtonContent::Icon(kind) => {
                let icon = egui::Rect::from_center_size(rect.center(), egui::vec2(ICON_W, ICON_W));
                kind.paint(painter, icon, C_TEXT);
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
    fn ui_panel(&mut self, ui: &mut egui::Ui, panel: egui::Rect) {
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
    /// 「生成随机数据」和「重新生成」现在都是：换一个新种子生成一段新行情。
    fn build_panel_buttons(&mut self) {
        let items = [
            (
                ButtonContent::IconText(IconKind::Triangle, "生成随机数据".to_owned()),
                PanelAction::NewRandomWalk,
            ),
            (
                ButtonContent::Text("重新生成".to_owned()),
                PanelAction::NewRandomWalk,
            ),
            (
                ButtonContent::Text("切换涨跌配色".to_owned()),
                PanelAction::ToggleColors,
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

    /// 执行一个面板动作。缩放 / 平移都是直接改视图状态，下一帧 ui_chart 就按新值出图。
    fn run_action(&mut self, action: PanelAction, label: &str) {
        match action {
            PanelAction::NewRandomWalk => {
                self.regenerate_random_walk();
            }
            // 状态栏本来就在实时显示 scale / offset，缩放和重置不用再回显一句。
            PanelAction::ZoomIn => self.zoom_step(1),
            PanelAction::ZoomOut => self.zoom_step(-1),
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
    fn zoom_step(&mut self, notches: i32) {
        self.scale = scale_after(self.scale, notches);
    }

    /// 换一个新种子，重新生成一段随机游走行情，并把视图（平移 / 缩放）归零。
    /// 种子按固定步进往前推：每次点出来的行情都不一样，但整条序列完全可复现。
    fn regenerate_random_walk(&mut self) {
        self.bar_seed = self
            .bar_seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.bars = data::random_walk::generate(self.bar_seed, BAR_COUNT, BAR_START);
        self.offset = 0;
        self.scale = BAR_SCALE_DEFAULT;
        self.crosshair = None;
        self.status_msg = format!("已生成新的随机行情（种子 {}）", self.bar_seed);
    }
}

// ===========================================================================
//  十一、界面：每帧执行一次
// ===========================================================================

impl eframe::App for StockApp {
    /// 清屏色：eframe 默认是近乎不透明的近黑色（12,12,12,180），
    /// 任何没被本帧画面盖住的像素露出来就是黑边，所以换成和标题栏同色。
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        C_TITLE_BG.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let t0 = Instant::now();

        // 1) 刷新统计（状态栏要显示它）。
        self.tick_frame(&ctx);

        // 3) 窗口句柄：构造时没拿到就在这里补一次；顺带保证客户区已经盖满窗口
        //    （去系统外圈装饰的关键一步，正常在构造时就装好了）。
        if self.hwnd == 0 {
            if let Ok(h) = frame.window_handle() {
                self.hwnd = win32_hwnd(h.as_raw());
            }
        }
        if !self.client_cover_ready && install_client_cover(self.hwnd) {
            self.client_cover_ready = true;
        }

        // 4) 窗口外框（屏幕坐标，拖动时要用它换算）。pixels_per_point 固定 1.0，点 = 像素。
        let outer_rect = ctx.input(|i| i.viewport().outer_rect);

        // 5) 布局区：正常就是 cells(125) x cells(67) = 1126x604；万一客户区更小，
        //    仍按 1126x604 排版，超出的部分被窗口裁掉 —— 不重排、不压缩、不变形。
        let win_rect = ui.max_rect();
        let layout = egui::Rect::from_min_size(
            win_rect.min,
            win_rect.size().max(egui::vec2(WINDOW_W, WINDOW_H)),
        );

        // 6) 四区域底色 + 标题栏 + 菜单栏 + 功能区（左栏蜡烛图 + 右栏控制面板）。
        let areas = split_areas(layout);
        self.paint_skeleton(&ui.painter().clone(), layout, areas);
        self.ui_titlebar(ui, areas[0], outer_rect);
        let open_anchor = self.ui_menubar(ui, areas[1]);

        // 功能区横向切成三块：左栏 K 线 / 分割条 / 右栏控制面板，按左→右依次画。
        let [left, splitter, panel] = Self::work_columns(areas[2], self.split_x);
        self.ui_chart(ui, left);
        self.ui_splitter(ui, splitter);
        self.ui_panel(ui, panel);

        // 7) 区域分隔线：排在所有区域内容之后画，控件的高亮盖不住这几条 1px 边框。
        self.paint_separators(&ui.painter().clone(), layout);

        // 8) 下拉面板：唯一的例外，它按设计浮在功能区之上，所以排在分隔线之后。
        if let Some((anchor, open)) = open_anchor {
            self.ui_menu_panel(ui, anchor, open);
        }

        // 9) 最外圈 1px 轮廓最后画：压在按钮之上，窗口轮廓永远完整。
        border_hard(&ui.painter().clone(), layout, C_BORDER);

        // 10) 推进窗口拖动。
        self.step_move(&ctx, outer_rect);

        // 11) 样式位每帧查一次：winit 在最小化/还原时会重新套用它的样式表，
        //     带 WS_CAPTION 的窗口会被 DWM 画上一圈投影。
        clear_window_frame(self.hwnd);

        // 12) 本帧真正花在构建界面上的时间。
        self.frame_ms = t0.elapsed().as_secs_f32() * 1000.0;

        // 13) 空闲也定期重绘，状态栏的数字才会持续跳动。
        ctx.request_repaint_after(Duration::from_millis(IDLE_REPAINT_MS));

    }
}

// ===========================================================================
//  十二、全局样式与渲染开关（窗口一启动就设好，之后不再动）
// ===========================================================================

/// 把 egui 默认样式里的留白全部清零。
/// 默认值大量不是 8 的倍数（item_spacing、button_padding (4,1)、indent 18 …），
/// 只要用到自带控件，网格立刻就歪。另外 0.36.2 的 Frame::side_top_panel 硬编码
/// Margin::symmetric(8, 2)、Frame::central_panel 硬编码 inner_margin(8)，
/// 所以本文件压根不用 Panel；将来要用必须显式 .frame(egui::Frame::NONE)。
fn configure_zero_spacing(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::Vec2::ZERO;
        style.spacing.window_margin = egui::Margin::ZERO;
        style.spacing.menu_margin = egui::Margin::ZERO;
        style.spacing.button_padding = egui::Vec2::ZERO;
        style.spacing.indent = 0.0;
        style.spacing.interact_size = egui::Vec2::ZERO;
        style.spacing.slider_width = 0.0;
        style.spacing.slider_rail_height = 0.0;
        style.spacing.combo_width = 0.0;
        style.spacing.combo_height = 0.0;
        style.spacing.text_edit_width = 0.0;
        style.spacing.menu_width = 0.0;
        style.spacing.menu_spacing = 0.0;
        style.spacing.tooltip_width = 0.0;
        style.spacing.extra_text_line_spacing = 0.0;
        style.spacing.icon_width = 0.0;
        style.spacing.icon_width_inner = 0.0;
        style.spacing.icon_spacing = 0.0;
        style.spacing.default_area_size = egui::Vec2::ZERO;

        // 视觉部分：一律直角、无阴影，避免出现非网格的装饰像素。
        style.visuals.window_corner_radius = egui::CornerRadius::ZERO;
        style.visuals.menu_corner_radius = egui::CornerRadius::ZERO;
        style.visuals.window_shadow = egui::Shadow::NONE;

    });
}

/// 渲染开关：关图形抗锯齿（feathering）、开像素网格对齐。
fn configure_tessellation(ctx: &egui::Context) {
    ctx.tessellation_options_mut(|opt| {
        opt.feathering = false;
        opt.round_text_to_pixels = true;
        opt.round_line_segments_to_pixels = true;
        opt.round_rects_to_pixels = true;
    });
}

// ===========================================================================
//  十三、去掉系统画在窗口外圈的东西（Windows 专用）
// ===========================================================================
//  窗口最外圈原本有两层系统装饰，两层都得自己清：
//    a) 窗口最上面那 1px —— 用户看到的那条「奇怪颜色的 1px 边框」就是它。
//       winit 为了让无边框窗口带上投影，在 WM_NCCALCSIZE 里把客户区整体下移 1px
//       （winit 源码注释原话：leads to a small black 1px border on the top）。
//       于是窗口最上面那一行永远不在客户区里：我们自己画不到，露出来的是 DWM 画的
//       1px 窗口边框（颜色取自 Windows 的窗口边框色，本机渲染成一条深灰渐变，
//       两个上角更深）。egui / winit 都没有对应开关，只能在窗口过程里拦 WM_NCCALCSIZE：
//       把非客户区压成 0、客户区矩形 = 整个窗口矩形，窗口 100% 变成我们的画布，
//       最外圈于是只剩我们自己画的那 1px 黑边框（见 install_client_cover）。
//    b) DWM 投影（窗口外面那一圈阴影）。DWM 给「带标题栏样式 WS_CAPTION 的窗口」
//       画投影，而 winit 为了 Aero Snap 一直留着这个样式位；把样式位清掉即可，
//       winit 在最小化/还原时会把样式重新套回去，所以每帧查一次（见 clear_window_frame）。
//  这些是 Windows 专属的窗口细节，用不到 windows-sys，直接声明要用的几个函数。
//  窗口样式是 32 位值，用 GetWindowLongW / SetWindowLongW（32/64 位通吃）；
//  窗口过程是 64 位指针，只能走 SetWindowLongPtrW。
#[link(name = "user32")]
unsafe extern "system" {
    fn GetWindowLongW(hwnd: isize, index: i32) -> i32;
    fn SetWindowLongW(hwnd: isize, index: i32, value: i32) -> i32;
    fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
    fn SetWindowPos(hwnd: isize, after: isize, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
    fn DefWindowProcW(hwnd: isize, msg: u32, wp: usize, lp: isize) -> isize;
    fn CallWindowProcW(prev: isize, hwnd: isize, msg: u32, wp: usize, lp: isize) -> isize;
}

const GWL_STYLE: i32 = -16;
const GWL_EXSTYLE: i32 = -20;
const GWLP_WNDPROC: i32 = -4;
const WS_CAPTION: i32 = 0x00C0_0000;
const WS_THICKFRAME: i32 = 0x0004_0000;
const WS_EX_WINDOWEDGE: i32 = 0x0000_0100;
const WS_EX_CLIENTEDGE: i32 = 0x0000_0200;
const WS_EX_STATICEDGE: i32 = 0x0000_2000;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_FRAMECHANGED: u32 = 0x0020;

const WM_NCCALCSIZE: u32 = 0x0083;

/// 被我们挤掉的那个系统窗口过程（其实是 winit 的），其余消息原样转给它。
static PREV_WNDPROC: AtomicIsize = AtomicIsize::new(0);

/// 从 raw-window-handle 的句柄里取出 Win32 的 HWND（拿不到就返回 0）。
fn win32_hwnd(raw: RawWindowHandle) -> isize {
    match raw {
        RawWindowHandle::Win32(h) => h.hwnd.get(),
        _ => 0,
    }
}

/// 清掉会让系统画出投影 / 3D 边 / 边框的那几个样式位，再让窗口重算一次框架。
///
/// 为什么要清：
///   投影 —— DWM 会给「带标题栏样式（WS_CAPTION）的窗口」画一圈投影；winit 为了
///   保住 Aero Snap 之类的功能，无边框窗口也一直留着 WS_CAPTION，而 egui-winit
///   在 decorations(false) 时还会主动请求这个阴影（它那个开关只影响客户区顶部
///   1px 的处理，治不了 DWM 本身）。
///   3D 边 —— WS_EX_WINDOWEDGE / CLIENTEDGE / STATICEDGE 会让系统沿窗口外圈画一条
///   立体边（左上亮、右下暗）。
/// winit 在最小化/还原这类标志位变化时会把样式重新套回去，所以这里每帧查一次。
fn clear_window_frame(hwnd: isize) {
    if hwnd == 0 {
        return;
    }
    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) };
    let ex_style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) };
    let style_dirty = style & (WS_CAPTION | WS_THICKFRAME) != 0;
    let ex_dirty = ex_style & (WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE) != 0;
    if !style_dirty && !ex_dirty {
        return;
    }
    unsafe {
        if style_dirty {
            SetWindowLongW(hwnd, GWL_STYLE, style & !(WS_CAPTION | WS_THICKFRAME));
        }
        if ex_dirty {
            SetWindowLongW(
                hwnd,
                GWL_EXSTYLE,
                ex_style & !(WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE),
            );
        }
        SetWindowPos(
            hwnd,
            0,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

/// 我们自己的窗口过程：只拦 WM_NCCALCSIZE，把非客户区定成 0。
/// 进来时 lParam 里的 rgrc[0] 就是「这一次要用的窗口矩形」，原样不动 + 返回 0
/// 就等于宣告「客户区 = 整个窗口」。
/// 注意不要去调 GetWindowRect 拿窗口矩形：最小化/还原过程中那个值还是旧位置
/// （最小化时是任务栏图标的位置），拿它算出来的客户区会把整个画面挪到窗口外。
unsafe extern "system" fn rt_wndproc(hwnd: isize, msg: u32, wp: usize, lp: isize) -> isize {
    if msg == WM_NCCALCSIZE && wp != 0 {
        return 0;
    }
    // 其余消息原样转给 winit，别把它的逻辑打断。
    let prev = PREV_WNDPROC.load(Ordering::Relaxed);
    unsafe {
        if prev == 0 {
            DefWindowProcW(hwnd, msg, wp, lp)
        } else {
            CallWindowProcW(prev, hwnd, msg, wp, lp)
        }
    }
}

/// 装上窗口过程钩子，并让系统立刻重算一次非客户区（此后客户区 = 整个窗口）。
fn install_client_cover(hwnd: isize) -> bool {
    if hwnd == 0 {
        return false;
    }
    if PREV_WNDPROC.load(Ordering::Relaxed) != 0 {
        // 已经装过就别再装：重复装会把我们自己的过程当成「上一个」，形成自套自。
        return true;
    }
    let prev = unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, rt_wndproc as *const () as isize) };
    if prev == 0 {
        return false;
    }
    PREV_WNDPROC.store(prev, Ordering::Relaxed);
    unsafe {
        SetWindowPos(
            hwnd,
            0,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
    true
}

// ===========================================================================
//  十四、程序入口
// ===========================================================================

fn main() -> eframe::Result {
    // ViewportBuilder 描述原生窗口。尺寸是「逻辑点」，而本工程强制 pixels_per_point = 1.0，
    // 所以逻辑点 == 物理像素。
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // 无边框：标题栏完全自绘（任务栏上仍显示这个标题）。
            .with_title("股票分析")
            .with_decorations(false)
            .with_inner_size([WINDOW_W, WINDOW_H])
            // 关掉系统缩放框：本步窗口尺寸固定不变。顺带解决两个老问题 ——
            // with_resizable(true) 会让 Windows 加一条看不见的 WS_SIZEBOX 边框，
            // 在那条边上按下会被系统当成「非客户区拖框」，进入系统的模态缩放循环：
            // 循环里 egui 一帧都不重绘，画面被 DWM 整块拉伸（文字、按钮一起变形），
            // 松手才恢复；那条边本身也没人绘制，露出一圈黑边。
            .with_resizable(false)
            .with_maximized(false),
        ..Default::default()
    };

    // run_native 会一直阻塞到窗口关闭；第三个参数是「应用工厂」闭包，
    // 必须返回 Result<Box<dyn App>, 错误>，所以是 Ok(Box::new(...))。
    eframe::run_native(
        "股票分析",
        native_options,
        Box::new(|cc| Ok(Box::new(StockApp::new(cc)))),
    )
}

// ===========================================================================
//  0.36.2 版本差异备忘（升级依赖时最容易踩的坑）
// ===========================================================================
//  1. App trait：0.36 要写 fn ui(&mut self, ui: &mut Ui, frame: &mut Frame)；
//     0.35 及以前是 fn update(&mut self, ctx: &Context, frame: &mut Frame)。
//  2. 无边框是 ViewportBuilder::with_decorations(false)，没有 undecorated()。
//     但无边框窗口的客户区顶上会被 winit 的「阴影 hack」留出 1px 非客户区
//     （它源码注释里的「small black 1px border on the top」；WM_NCCALCSIZE 里
//     rgrc[0].top / bottom 各 +1），这一行我们画不到，露出来就是 DWM 画的窗口边框。
//     用窗口过程钩子把非客户区压成 0 解决（第十三节 install_client_cover），
//     之后客户区 = 整个窗口，最外圈只剩我们自己画的 1px 黑边框。
//  3. eframe 默认清屏色是近黑色 Color32::from_rgba_unmultiplied(12, 12, 12, 180)，
//     窗口边上一圈黑边就是它；用 eframe::App::clear_color 返回自己的底色根治。
//  4. Painter 没有 round_to_pixel()；round_to_pixel_center() 在坐标正好为 0 时会
//     偏到窗口外（v = 0 得到 -0.5），1px 线的吸附要自己算（第四节 px_center）。
//  5. 视口命令叫 OuterPosition / InnerSize（没有 SetOuterPosition / SetInnerSize）。
//     拖窗口故意不用 ViewportCommand::StartDrag：它其实是把控制权交给 Windows 的
//     模态拖动循环，循环里 egui 不重绘，窗口挪动后新露出的区域从没被画过，
//     看上去就是一圈黑边。自己每帧发 OuterPosition 就没这问题。
//  6. 描边要带 StrokeKind（Inside / Middle / Outside）；圆角类型是 CornerRadius。
//  7. egui 命中测试是「同一层里后注册的在上」（hit_test.rs：距离相同取最后一个），
//     所以热区/按钮的注册顺序就是它们的优先级。
//  8. 渲染后端是 wgpu（eframe 0.36.2 的默认特性里没有 glow）；glow_options 字段
//     只在开了 eframe 的 glow feature 时才存在，本项目没开。
//  9. 无边框窗口外圈的系统阴影：DWM 会给「带 WS_CAPTION 样式」的窗口画一圈投影，
//     而 winit 为了 Aero Snap 一直留着这个样式位，egui-winit 在 decorations(false)
//     时还会主动「请求」这个阴影（with_undecorated_shadow(true)）。winit 那个开关
//     只影响无边框窗口客户区顶部 1px 的处理，并不能让 DWM 不画投影；egui 侧也没有
//     对应字段（ViewportBuilder::with_has_shadow 只在 macOS 生效）。本程序自己调
//     user32 清掉 WS_CAPTION / WS_THICKFRAME 解决（见第十三节）。
// 10. 无边框窗口外面那圈系统装饰有两个来源，都得自己关（见第十三节）：
//     a) DWM 投影：来自窗口的 WS_CAPTION / WS_THICKFRAME 样式位（winit 为了 Aero Snap
//        一直留着）—— 清样式位即可，见 clear_window_frame（每帧查一次，因为 winit
//        在最小化/还原时会把样式重新套回去）；
//     b) 窗口顶上那 1px 系统边框：winit 用 WM_NCCALCSIZE 的 hack 让客户区整体下移
//        1px，把窗口最上面一行让给了系统 —— 在窗口过程里拦 WM_NCCALCSIZE，把非客户区
//        压成 0（rgrc[0] = 窗口矩形），见 rt_wndproc / install_client_cover。
//     注：Win11 的圆角 / 1px 窗口边框要用 DWMWA_WINDOW_CORNER_PREFERENCE(33) 与
//     DWMWA_BORDER_COLOR(34) 处理，Win10 不认这两个属性（返回 E_INVALIDARG），
//     本机就不必调了。
// 11. 文字抗锯齿：egui 0.36 没有任何开关能关掉字形抗锯齿（feathering 的文档明确
//     不影响文字，FontTweak 只有 hinting，TextOptions 只有覆盖率曲线），而 12px 宋体
//     的笔画只有一个像素宽，抹灰度只会让笔画断掉。所以文字不交给 egui 光栅化，改用
//     Windows GDI：GetGlyphOutlineW + GGO_BITMAP 直接拿 1bpp 单色字形（gdi32.dll，
//     见第五节），零抗锯齿、笔画完整，也顺手省掉了一整套字体加载代码。
// ===========================================================================

// ===========================================================================
//  十五、单元测试
// ===========================================================================
//  只测「画出来像素级对不对称」这类眼睛容易漏、算起来一清二楚的事。
//  做法：起一个真的 egui Context（不用窗口、不用 GPU），把一根蜡烛交给真的 epaint
//  光栅化管线（TessellationOptions 跟运行时完全一致：关抗锯齿、吸附像素栅格），
//  再把三角形按「像素中心是否落在里面」填进网格 —— 关掉抗锯齿后显卡干的就是这件事，
//  所以这张网格就是屏幕上真正看到的样子，逐列数得清清楚楚。

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试画布：够放下最大号的蜡烛（实体最宽 15px，影线上下各留一段）。
    const CANVAS_W: usize = 40;
    const CANVAS_H: usize = 120;

    /// 蜡烛落位：实体左边缘 20，实体占 y = 40..50，影线上下各外扩 20 行。
    const LEFT: f32 = 20.0;
    const BODY_TOP: f32 = 40.5;
    const BODY_BOTTOM: f32 = 50.5;
    const WICK_TOP: f32 = 20.5;
    const WICK_BOTTOM: f32 = 70.5;

    /// 取两个样本行：实体外（只可能是影线）与实体正中（量实体宽度）。
    const WICK_ROW: usize = 30;
    const BODY_ROW: usize = 45;

    /// 画一根蜡烛并光栅化成像素网格：None = 没画东西。
    fn render_candle(scale: u32, rising: bool) -> Vec<Vec<Option<egui::Color32>>> {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);

        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(CANVAS_W as f32, CANVAS_H as f32),
        ));

        let bar_w = snap_scale(scale) as f32;
        let color = if rising { C_UP } else { C_DOWN };
        let mut out = ctx.run_ui(raw, |ui| {
            paint_candle(
                ui.painter(),
                LEFT,
                bar_w,
                rising,
                color,
                BODY_TOP,
                BODY_BOTTOM,
                WICK_TOP,
                WICK_BOTTOM,
            );
        });
        // 测试不碰纹理：显式清掉，否则 epaint 会在 Drop 时断言「还有没处理的 delta」。
        let shapes = std::mem::take(&mut out.shapes);
        out.textures_delta.clear();
        rasterize(&ctx.tessellate(shapes, 1.0))
    }

    /// 三角形 (a, b, c) 的有向面积（叉积的 z 分量）。
    fn edge(a: egui::Pos2, b: egui::Pos2, c: egui::Pos2) -> f32 {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    }

    /// 把网格三角形填进像素网格：像素中心落在三角形里就算被覆盖。
    fn rasterize(prims: &[egui::ClippedPrimitive]) -> Vec<Vec<Option<egui::Color32>>> {
        let mut grid = vec![vec![None; CANVAS_W]; CANVAS_H];
        for prim in prims {
            let egui::epaint::Primitive::Mesh(mesh) = &prim.primitive else {
                continue;
            };
            for tri in mesh.indices.chunks_exact(3) {
                let [a, b, c] = [
                    mesh.vertices[tri[0] as usize],
                    mesh.vertices[tri[1] as usize],
                    mesh.vertices[tri[2] as usize],
                ];
                if edge(a.pos, b.pos, c.pos) == 0.0 {
                    continue; // 退化成线的三角形，跳过
                }
                let x0 = a.pos.x.min(b.pos.x).min(c.pos.x).floor().max(0.0) as usize;
                let x1 = (a.pos.x.max(b.pos.x).max(c.pos.x).ceil() as usize).min(CANVAS_W);
                let y0 = a.pos.y.min(b.pos.y).min(c.pos.y).floor().max(0.0) as usize;
                let y1 = (a.pos.y.max(b.pos.y).max(c.pos.y).ceil() as usize).min(CANVAS_H);
                for y in y0..y1 {
                    for x in x0..x1 {
                        let p = egui::pos2(x as f32 + 0.5, y as f32 + 0.5);
                        let d = [
                            edge(b.pos, c.pos, p),
                            edge(c.pos, a.pos, p),
                            edge(a.pos, b.pos, p),
                        ];
                        let (neg, pos) = (d.iter().any(|v| *v < 0.0), d.iter().any(|v| *v > 0.0));
                        if !(neg && pos) {
                            grid[y][x] = Some(a.color);
                        }
                    }
                }
            }
        }
        grid
    }

    /// 某一行上画着 `color` 的所有列号（从左到右）。
    fn cols_of(
        grid: &[Vec<Option<egui::Color32>>],
        row: usize,
        color: egui::Color32,
    ) -> Vec<usize> {
        (0..CANVAS_W)
            .filter(|&x| grid[row][x] == Some(color))
            .collect()
    }

    /// 调试用字符画：R = 涨、G = 跌、w = 空心内部、. = 空。
    fn ascii(grid: &[Vec<Option<egui::Color32>>], rows: std::ops::Range<usize>) -> String {
        let mut out = String::new();
        for row in grid.iter().take(rows.end).skip(rows.start) {
            for cell in row {
                out.push(match cell {
                    None => '.',
                    Some(c) if *c == C_UP => 'R',
                    Some(c) if *c == C_DOWN => 'G',
                    Some(_) => 'w',
                });
            }
            out.push('\n');
        }
        out
    }

    /// 实体左右两边的像素数必须严格相等 —— 影线那 1px 正好落在正中那一列。
    fn assert_centered(label: &str, body: &[usize], wick: usize, art: String) {
        assert_eq!(wick - body[0], body[body.len() - 1] - wick, "{label}\n{art}");
    }

    /// 实心（下跌）蜡烛：影线在实体之外那段只有 1px，左右两边的实体像素数必须相等。
    #[test]
    fn wick_is_centered_in_the_body_for_every_scale() {
        for scale in (SCALE_MIN..=SCALE_MAX).step_by(2) {
            let grid = render_candle(scale, false);
            let art = ascii(&grid, 28..52);
            println!("下跌实心 scale={scale}\n{art}");

            let body = cols_of(&grid, BODY_ROW, C_DOWN);
            let wick = cols_of(&grid, WICK_ROW, C_DOWN);
            let label = format!("实体 {scale}px 里影线没在正中");
            assert_eq!(body.len(), snap_scale(scale) as usize, "{label}\n{art}");
            assert_eq!(wick.len(), 1, "影线不是 1px 宽\n{art}");
            assert_centered(&label, &body, wick[0], art);
        }
    }

    /// 空心（上涨）蜡烛：两根竖边、中间那块底色、影线，三者共用同一条中心轴。
    #[test]
    fn hollow_body_keeps_the_wick_in_the_middle() {
        for scale in (SCALE_MIN..=SCALE_MAX).step_by(2) {
            let grid = render_candle(scale, true);
            let art = ascii(&grid, 28..52);
            println!("上涨空心 scale={scale}\n{art}");

            let edges = cols_of(&grid, BODY_ROW, C_UP);
            let inner = cols_of(&grid, BODY_ROW, C_CHART_BG);
            let wick = cols_of(&grid, WICK_ROW, C_UP);
            let label = format!("空心实体 {scale}px 里影线没在正中");
            assert_eq!(edges.len(), 2, "空心实体的竖边不是两根\n{art}");
            assert_eq!(inner.len(), snap_scale(scale) as usize - 2, "{label}\n{art}");
            assert_eq!(wick.len(), 1, "影线不是 1px 宽\n{art}");
            let axis = edges[0] + edges[1];
            assert_eq!(axis, inner[0] + inner[inner.len() - 1], "{label}\n{art}");
            assert_eq!(wick[0] * 2, axis, "{label}\n{art}");
        }
    }
}
