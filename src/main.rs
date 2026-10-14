// ===========================================================================
//  【本项目铁律 - 改这个文件之前必读】
// ===========================================================================
//  1. 颜色：全工程的色值只允许出现在 theme.rs 的系统级配色表（struct Theme）里，别处一个
//     字面色都不许写 - 界面代码一律走 theme()（当前皮肤）取色，写成 `theme().text` 这样。
//     色值一律写成 6 位 #RRGGBB，在 const 里用 egui::Color32::from_rgb(0xRR, 0xGG, 0xBB)，
//     注意是「两个十六进制字符一组」；
//     -> 而且每个字节必须是「重复的同一个十六进制数字」，也就是严格 #aabbcc 形：
//       合法：00 11 22 33 44 55 66 77 88 99 aa bb cc dd ee ff
//             （例：#888888、#996633、#cccccc、#999999）
//       非法：80 / c0 / e0 / 12 这类两个数字不同的字节（例：#808080、#c0c0c0）
//     theme.rs 末尾对每一套皮肤逐项做编译期自检，色值写错格式直接编译不过。
//     禁止带透明度的写法、禁止 from_rgba / from_gray / from_black_alpha 之类。
//     一套皮肤 = 一张表：UI 里所有元素的颜色都在里面（一个不落），以后做皮肤自定义工具
//     改的就是这张表。切皮肤走 theme.rs 的 set_theme（面板「切换主题」按钮 -> StockApp::switch_theme）。
//  2. 尺寸：内容格 8px（GRID）+ 网格线格 1px（LINE_W），节距 PITCH = 9px。
//     控件的宽/高都是 cells(n) = 9n + 1（跨 n 个内容格，含两端各 1px 网格线格）；
//     所有坐标都落在网格线上（i * PITCH），窗口尺寸也必须满足 cells(n)。
//     完整的布局模型见第二节，动任何布局之前先读那一节。
//  3. 外观：不用任何 egui 自带控件的默认外观，全部用 Painter 手绘
//     （纯色矩形 + 1px 硬边，无圆角、无阴影、无渐变、无抗锯齿）。
//     不使用 egui::Panel - 它内部硬编码了非 8 倍数的 margin。
//  4. 文字：全界面 16px 点阵字，硬边（零抗锯齿）- 不交给 egui 光栅化，而是走 Windows GDI 的
//     单色字形（汉字 GetGlyphOutlineW + GGO_BITMAP 拿宋体自带的 16px 点阵；ASCII 走 Fixedsys
//     8x16 点阵字，想换回宋体 16px 的 ASCII 就把 font.rs 的 ASCII_FROM_CJK 改成 true - 两条路都是
//     1bpp 位图；字形换了排版一个字都不动），自己贴图、按整数像素落点画（见 font.rs）。
//     pixels_per_point 固定 1.0，feathering 关闭。全工程的文字只能走 StockApp::text 这一个入口。
//     文字按网格排版（见 font.rs 的「网格文字模型」）：一个字符的字格就是它占的那几个内容格 -
//     汉字 2x2 格（宽 8+1+8 = 17px、高 8+1+8 = 17px，16x16 字形摆在左上），
//     ASCII 1x2 格（宽 8px、高 17px，8x16 字形摆满）。
//     横向步进因而只有两个值：汉字 18px（2 个内容格）、ASCII 9px（1 个内容格）；
//     起点落在内容格起点上，整串必然和 9px 网格对齐 - 汉字每 2 格一个、ASCII 每 1 格一个。
//     纵向：一个文本行盒 = 2 个内容格（内容区高 17px），汉字和 ASCII 共用同一条基线、底边对齐。
//     界面文字只允许「中文 + ASCII」：别的符号（非 ASCII 的标点 / 箭头 / 数学符号）一个都不许出现，
//     由源码扫描测试 allow_only_chinese_and_ascii_in_sources 把关。
//     【铁律】出现文字的地方，左右两边必须各留至少 1 个网格单位（GRID = 8px）的空档：
//     按文字量宽度的盒子一律用 snap_cells_up(文字宽 + 2 * TEXT_PAD_X) 这类算式算，
//     按钮的宽就是 cells(文字内容格数 + 2)（「切换」「代码」这类按钮都照这条量），
//     不许把文字直接怼到边框 / 分割线上。
//     落点也要踩在内容格上（内容格起点 = 9k + 1，网格线格在最左那 1px）：横向落点全工程只有
//     grid.rs 的 text_left_x / text_right_x 两个入口 - 左对齐传左边那条网格线（区块外框的
//     min.x、或它 shrink(LINE_W) 出来的内框都行，会吸到同一条线上），右对齐传右边那条网格线；
//     居中文字在 cells(n) 盒子里居中之后也正好落在 10px 上（n = 文字格数 + 2）。
//     【铁律】全界面从元素尺寸到文字落点都必须踩在 9px 网格上 - 只有图表内部那几处随行情动态
//     走的文字（时间轴 / 纵轴刻度 / 十字光标数值牌）没法预先对齐，这些也得把宽度约束在自己那一列
//     里（见 grid.rs 的 AXIS_COLS / VALUE_COLS_MAX），绝不许溢出自己的区域；图表通用状态带
//     （指标自己声明的那条）的文字照样必须对齐网格。
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
//     d) 不许出现 2px 线叠加：任何一条线都不许停在别的线旁边那一行 / 那一列上 - 要么和它重合
//        成同一条线，要么中间空出至少一整行 / 一列。浮动小牌子（十字光标数值牌 / 索引牌、悬停
//        便签）照这条摆：牌子边框要么压在相邻控件的边框线上，要么离它一整格（见 grid.rs 的
//        floating_label_top、ui::button 的 paint_tooltip）；滚动条把手贴着上 / 下边框时也一样。
//  8. 窗口最外圈必须只有我们自己画的那 1px 边框（默认 #999999）：系统画上去的东西（DWM 投影、
//     标题栏样式位，以及 winit 留在窗口顶上的 1px 非客户区）全部自己清掉，
//     客户区撑满整个窗口（见第十三节）。
//  9. 控件区划分：UI 里每一个控件区域的外圈都必须有横竖线划分（线 1px、落在网格线格里） -
//     标题栏 / 菜单栏 / 快捷按钮栏 / 正文区 / 状态栏这类上下相邻的段，段与段之间的横线一定要
//     画出来；各段的底色各自往里让开那条共用的线格，分隔线统一排在所有内容之后补画一遍
//     （谁都盖不掉）。样板见 ui::editor 的 paint_separators、ui::floatwin 的 end。
// 10. 图标：按钮 / 菜单上的图标一律是 assets/icons/<名字>.png 里的 16x16 PNG，开机读进来、
//     按像素自绘（ui::icon 的 IconSet::load_or_create；文件不在 / 读不动就按内置点阵写一份出来）。
//     PNG 里 #ff00ff（洋红）是透明色，别的像素一律吸到 #aabbcc 形（每个通道是 0x11 的倍数）；
//     白色 #ffffff 是「跟随皮肤文字色」的墨迹。改图标直接改这些 PNG，不用动代码。
// 11. 控件抽象：按钮只有 ui::button 的 Button / ButtonContent 一份（文字 / 图标 / 图标 + 文字三种
//     内容 + 悬停便签 + 量好的宽度），菜单栏与下拉面板只有 ui::menu 一份（主窗口菜单栏和编辑器
//     菜单栏共用同一套绘制），浮动窗外壳只有 ui::floatwin 一份。要加新控件先看能不能套进这几个
//     类，别各写一份外观；按钮一律左对齐排，右对齐的排法不做了（主窗口和编辑器一致）。
// 12. 悬停便签：鼠标停在按钮 / 菜单项上时弹一块文字便签说明含义 - 按钮把 (矩形, 文字) 记进
//     StockApp::tooltip，由 ui() 结尾统一画在整帧最上层（ui::button 的 paint_tooltip），
//     浮动窗也压不住它；便签的上 / 下沿压在被悬停控件的那条边框线上，同样只有 1px。
// ===========================================================================

// ===========================================================================
//  圆桌 RoundTable - Windows 桌面股票分析软件
//  当前进度：9px 节距（8px 内容格 + 1px 网格线格）+ 无边框窗口；
//            标题栏 / 菜单栏 / 功能区（左栏图表 + 右栏控制面板）/ 状态栏；
//            K 线支持滚轮缩放（锚点缩放，整数倍跳变）+ 左键拖拽平移；
//            文字按网格排版（16px 点阵字：汉字 2 格 / ASCII 1 格）+ 两套皮肤（经典 / 暗色）可切换；
//            按钮 / 菜单栏 / 悬停便签各抽象成一份（ui::button / ui::menu），图标是 16x16 PNG；
//            指标源码编辑器（ui::editor + ui::source）：菜单栏 / 按钮栏 / 行号 / 滚动条 / 光标 /
//            选区 / 撤销 / 中文输入法一整套。
//  文件结构（一个模块一个文件，都在 crate 内可见）：
//    main.rs            程序入口 + 单元测试（铁律的开头那份说明也在这儿，改代码前先读它）
//    grid.rs            度量常数 + 像素级绘制工具（算尺寸 / 算坐标只有这一个入口）
//    theme.rs           主题 / 系统级配色表（一套皮肤 = 一张表）
//    font.rs            硬边文字引擎（Windows GDI 1bpp 字形，网格文字模型）
//    app.rs             StockApp 状态、每帧 ui() 驱动、全局样式、文字入口
//    win32.rs           窗口杂项（去掉系统画在窗口外圈的装饰）
//    -- 界面层 ui/：控件区一律「外壳 + 内容」两块 --
//    ui/mod.rs          子模块清单 + 设计逻辑（每个控件区外圈都得有落在网格线上的划分线）
//    ui/titlebar.rs     标题栏（程序名 + 系统按钮 + 拖动热区），主窗口和浮动窗共用
//    ui/menu.rs         全工程唯一的菜单栏 + 下拉面板（主窗口 / 编辑器共用同一套绘制）
//    ui/button.rs       全工程唯一的按钮（文字 / 图标 / 图标 + 文字 + 悬停便签）
//    ui/icon.rs         像素图标：assets/icons/<名字>.png 的 16x16 PNG 读进来再自绘
//    ui/chart.rs        图表区（K 线 / 成交量 / 指标 / 时间轴 / 十字光标），三块同一种「图表」
//    ui/panel.rs        两栏布局、分割条、右侧控制面板
//    ui/colorpanel.rs   涨跌颜色与自绘颜色面板
//    ui/panemenu.rs     图表标题栏「切换」的下拉面板（换指标）
//    ui/overlaymenu.rs  图表标题栏「叠加」的下拉面板（MA / EMA 开关）
//    ui/floatwin.rs     浮动窗外壳（底色 / 标题栏 / 关闭键 / 按格拖动），子窗口都套它
//    ui/source.rs       源码编辑窗：外壳 + 保存 / 编译 + 状态栏回显
//    ui/editor.rs       自绘可编辑文本编辑器（菜单栏 / 按钮栏 / 行号 / 光标 / 输入法）
//    ui/textbuf.rs      文本缓冲区（行 + 字符索引光标 + 选区 + 撤销栈，纯逻辑）
//    ui/textbox.rs      通用自绘单行输入框（颜色面板的 Hex 框用它，以后输入框都走这儿）
//    -- 数据 / 指标层 --
//    data/              行情数据层
//    indicator/         指标层（DrawCmd 数据坐标指令 + Indicator 插件接口 + 内置指标）
//  架构约定：
//    * 界面层 ui/ 只负责「把 StockApp 的状态画成像素 + 把鼠标键盘变成动作」，不碰行情数据；
//      指标层 indicator/ 只产数据坐标的 DrawCmd，对齐 / 落点全在 ui 层现算（replay）；
//    * 跨帧状态一律放 StockApp 的字段里（egui 是即时模式），窗口 / 面板各自的状态放各自的
//      struct（FloatWindow / TextEditor / ColorPanel ...），ui() 每帧按固定顺序驱动；
//    * 每帧的绘制顺序：骨架 -> 标题栏 / 菜单栏 -> 功能区 -> 面板 -> 浮动窗 -> 下拉面板 ->
//      悬停便签（最上层）。要让某样东西压住别的，改的是这个顺序，不是 Z 值。
// ===========================================================================
//  窗口策略：无边框，开机 cells(125) x cells(102) = 1126x919；尺寸在启动时定死，不做拖动缩放，
//  也没有最大化（系统的模态缩放循环里 egui 一帧都不重绘，画面会被 DWM 整块拉伸，新露出的区域
//  也没被画过，看上去就是窗口周围一圈黑边）。标题栏右侧只有最小化 / 关闭两个按钮。
//  布局一律按实际客户区排（下限见 WINDOW_MIN_W/H），移动窗口是自己逐帧实现的（见 step_move）。
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
//                   绝对值大的方向舍入），画在 x = 0 / y = 0 的线会整条跑到窗口外 -
//                   四条边框线只剩右、下两条看得见。
//    窗口外圈装饰 : 自己调 user32：清 WS_CAPTION / WS_EX_*EDGE 样式位（去 DWM 投影），
//                   再在窗口过程里拦 WM_NCCALCSIZE 把非客户区压成 0（去窗口顶上那条
//                   1px 系统边框，见第十三节）- egui/winit 都没有现成开关
//    硬边文字     : 不用 egui 那套文字渲染（抗锯齿关不掉），自己调 gdi32：
//                   GetGlyphOutlineW + GGO_BITMAP 拿 1bpp 单色字形（见第五节）
// ===========================================================================

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//  本文件 = 文件头 +「十四、程序入口」+「十六、单元测试」。
use eframe::egui;

/// 应用状态与主循环。
mod app;
/// 行情数据层（见 src/data/）。
mod data;
/// 硬边文字引擎（Windows GDI 1bpp 字形）。
mod font;
/// 度量常数与像素级绘制工具。
mod grid;
/// 指标层（见 src/indicator/）：只做纯计算，不依赖界面。
mod indicator;
/// 颜色表。
mod theme;
/// 界面层：标题栏 / 菜单栏 / 图表 / 两栏与按钮面板 / 颜色面板。
mod ui;
/// 窗口杂项（Windows 专用）。
mod win32;

// 拆分之后，各部分还是按原来那套名字直接用（都在 crate 内可见）。
use crate::app::*;
use crate::grid::*;

// ===========================================================================
//  十四、程序入口
// ===========================================================================

fn main() -> eframe::Result {
    // ViewportBuilder 描述原生窗口。尺寸是「逻辑点」，而本工程强制 pixels_per_point = 1.0，
    // 所以逻辑点 == 物理像素。
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // 无边框：标题栏完全自绘（任务栏上仍显示这个标题）。
            .with_title(APP_NAME)
            .with_decorations(false)
            .with_inner_size([WINDOW_W, WINDOW_H])
            // 窗口尺寸不再钉死：布局按实际客户区排（见 ui()），下限就是两栏各自最小那一档。
            .with_min_inner_size([WINDOW_MIN_W, WINDOW_MIN_H])
            // 系统那条看不见的 WS_SIZEBOX 缩放边框照样每帧清掉（见 clear_window_frame）：
            // 在它上面按下会被系统当成「非客户区拖框」，进入系统的模态缩放循环，循环里
            // egui 一帧都不重绘、画面被 DWM 整块拉伸。所以不做拖动缩放，也没有最大化按钮，
            // 窗口尺寸就是启动时那一档。
            .with_maximized(false),
        ..Default::default()
    };

    // run_native 会一直阻塞到窗口关闭；第三个参数是「应用工厂」闭包，
    // 必须返回 Result<Box<dyn App>, 错误>，所以是 Ok(Box::new(...))。
    eframe::run_native(
        APP_NAME,
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
//        一直留着）- 清样式位即可，见 clear_window_frame（每帧查一次，因为 winit
//        在最小化/还原时会把样式重新套回去）；
//     b) 窗口顶上那 1px 系统边框：winit 用 WM_NCCALCSIZE 的 hack 让客户区整体下移
//        1px，把窗口最上面一行让给了系统 - 在窗口过程里拦 WM_NCCALCSIZE，把非客户区
//        压成 0（rgrc[0] = 窗口矩形），见 rt_wndproc / install_client_cover。
//     注：Win11 的圆角 / 1px 窗口边框要用 DWMWA_WINDOW_CORNER_PREFERENCE(33) 与
//     DWMWA_BORDER_COLOR(34) 处理，Win10 不认这两个属性（返回 E_INVALIDARG），
//     本机就不必调了。
// 11. 文字抗锯齿：egui 0.36 没有任何开关能关掉字形抗锯齿（feathering 的文档明确
//     不影响文字，FontTweak 只有 hinting，TextOptions 只有覆盖率曲线），而 16px 宋体 /
//     Fixedsys 的点阵字笔画本来就只有一个像素宽，抹灰度只会让笔画断掉。所以文字不交给
//     egui 光栅化，改用 Windows GDI：汉字走 GetGlyphOutlineW + GGO_BITMAP 直接拿 1bpp
//     单色字形，ASCII 走 Fixedsys 光栅字（都在 gdi32.dll，见 font.rs），零抗锯齿、笔画完整。
// ===========================================================================

// ===========================================================================
//  十六、单元测试
// ===========================================================================
//  只测「画出来像素级对不对称」这类眼睛容易漏、算起来一清二楚的事。
//  做法：起一个真的 egui Context（不用窗口、不用 GPU），把一根蜡烛交给真的 epaint
//  光栅化管线（TessellationOptions 跟运行时完全一致：关抗锯齿、吸附像素栅格），
//  再把三角形按「像素中心是否落在里面」填进网格 - 关掉抗锯齿后显卡干的就是这件事，
//  所以这张网格就是屏幕上真正看到的样子，逐列数得清清楚楚。

#[cfg(test)]
mod tests {
    use super::*;
    
    
    use crate::ui::panel::*;
    use crate::ui::button::*;
    use crate::ui::icon::*;
    use crate::ui::menu::*;
    use crate::ui::editor::*;
    use crate::ui::textbox::*;
    use crate::ui::textbuf::*;
    
    use crate::ui::colorpanel::*;
    use crate::ui::source::*;
    use crate::ui::chart::*;
    use crate::theme::*;
    
    use crate::font::*;
    // 拆分之后测试用到的那几层：按需引入，非测试构建里这些名字不出现。

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
        absorb_textures(&mut out.textures_delta);
        rasterize(&ctx.tessellate(shapes, 1.0))
    }

    /// 三角形 (a, b, c) 的有向面积（叉积的 z 分量）。
    fn edge(a: egui::Pos2, b: egui::Pos2, c: egui::Pos2) -> f32 {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    }

    /// 测试用的最小 StockApp：只为了能调 ui_color_panel / ui_chart 这类绘制入口。
    /// 文字引擎留空（测试里不画字），行情数据也留空（要用的话测试自己填），其余字段给个能过的初值。
    fn blank_app() -> StockApp {
        StockApp {
            hwnd: 0,
            client_cover_ready: false,
            icons: IconSet::builtin(),
            tooltip: None,
            menu: MenuState::default(),
            status_msg: String::new(),
            frames: 0,
            last_frame_time: 0.0,
            frame_interval_ms: 0.0,
            frame_ms: 0.0,
            fps: 0.0,
            drag: None,
            bars: Vec::new(),
            bar_seed: BAR_SEED,
            scale: BAR_SCALE_DEFAULT,
            offset: 0,
            visible: 0,
            pan: None,
            zoom_acc: 0.0,
            text_engine: None,
            crosshair: None,
            panes: StockApp::build_panes(indicator::Palette::default()),
            panes_palette: indicator::Palette::default(),
            kline_h: KLINE_H_DEFAULT,
            vol_h: VOL_H_DEFAULT,
            ind_h: IND_H_DEFAULT,
            hsplit_grab: None,
            vsplit_grab: None,
            split_x: CHART_PANE_W,
            split_grab: None,
            panel_buttons: Vec::new(),
            panel_press: None,
            up_color: UP_RGB_DEFAULT,
            down_color: DOWN_RGB_DEFAULT,
            chart_follows_theme: true,
            color_panel: ColorPanel::new(),
            show_grid: false,
            pane_menu: None,
            pane_menu_anchor: egui::Rect::NOTHING,
            overlay: indicator::Overlay::DEFAULT,
            overlay_menu: None,
            overlay_menu_anchor: egui::Rect::NOTHING,
            source: SourceView::new(),
        }
    }

    /// 上面那个再打开颜色面板（面板相关的测试都用它）。
    fn color_panel_app() -> StockApp {
        let mut app = blank_app();
        app.color_panel.win.open = true;
        app.color_panel.win.pos = egui::pos2(CP_DEFAULT_X, CP_DEFAULT_Y);
        app.color_panel.load((34, 136, 255));
        app
    }

    /// 把颜色面板画在整窗画布（1126x604）上再光栅化：画布尺寸 = 窗口尺寸（1 逻辑点 = 1 像素）。
    fn render_panel(app: &mut StockApp) -> Vec<Vec<Option<egui::Color32>>> {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        // 字形图集挂在 ctx 上：这份 dump 用自己的 ctx，引擎就得重建在这个 ctx 上，
        // 否则光栅器收不到字形贴图，dump 出来的文字又变回一个个实心方块。
        if let Some(engine) = HardText::new(&ctx) {
            app.text_engine = Some(engine);
        }
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(WINDOW_W, WINDOW_H),
        ));
        let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        let mut out = ctx.run_ui(raw, |ui| {
            app.ui_color_panel(ui, layout);
        });
        let shapes = std::mem::take(&mut out.shapes);
        absorb_textures(&mut out.textures_delta);
        rasterize_into(
            &ctx.tessellate(shapes, 1.0),
            WINDOW_W as usize,
            WINDOW_H as usize,
        )
    }

    /// 把网格三角形填进像素网格：像素中心落在三角形里就算被覆盖。
    fn rasterize(prims: &[egui::ClippedPrimitive]) -> Vec<Vec<Option<egui::Color32>>> {
        rasterize_into(prims, CANVAS_W, CANVAS_H)
    }

    thread_local! {
        /// 离屏光栅化用的贴图表：egui 的字形图集是运行时才上传的（走 TexturesDelta），
        /// 光栅器却只拿到三角形，不把这一帧的贴图收下来，dump 出来的文字就只是实心方块。
        static DUMP_TEXTURES: std::cell::RefCell<
            std::collections::HashMap<egui::TextureId, egui::ColorImage>,
        > = std::cell::RefCell::new(std::collections::HashMap::new());
    }

    /// 把一帧的贴图更新收进 DUMP_TEXTURES：整张上传就换掉、局部补丁就贴上去，
    /// 字体图集（egui 自己那套灰度位图）和释放掉的贴图就删掉。
    fn absorb_textures(delta: &mut egui::TexturesDelta) {
        DUMP_TEXTURES.with(|cell| {
            let mut store = cell.borrow_mut();
            for (id, list) in &delta.set {
                for img in list {
                    let egui::ImageData::Color(src) = &img.image;
                    match img.pos {
                        None => {
                            store.insert(*id, (**src).clone());
                        }
                        Some(p) => {
                            if let Some(dst) = store.get_mut(id) {
                                for y in 0..src.size[1] {
                                    for x in 0..src.size[0] {
                                        let (dx, dy) = (p[0] + x, p[1] + y);
                                        if dx < dst.size[0] && dy < dst.size[1] {
                                            dst[(dx, dy)] = src[(x, y)];
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for id in &delta.free {
                store.remove(id);
            }
            // 收下之后必须清掉：TexturesDelta 的 Drop 见到没处理的更新会直接 panic。
            delta.set.clear();
            delta.free.clear();
        });
    }

    /// 按 uv 取一个贴图像素（NEAREST，夹在边界内）。贴图没收到（纯色图形）就返回 None。
    fn sample_texture(id: egui::TextureId, uv: egui::Pos2) -> Option<egui::Color32> {
        DUMP_TEXTURES.with(|cell| {
            cell.borrow().get(&id).map(|img| {
                let sx = (uv.x * img.size[0] as f32) as isize;
                let sy = (uv.y * img.size[1] as f32) as isize;
                let sx = sx.clamp(0, img.size[0] as isize - 1) as usize;
                let sy = sy.clamp(0, img.size[1] as isize - 1) as usize;
                img[(sx, sy)]
            })
        })
    }

    /// 同上，画布尺寸自己给：颜色面板那一张要铺满整个窗口，比蜡烛画布大得多。
    fn rasterize_into(
        prims: &[egui::ClippedPrimitive],
        w: usize,
        h: usize,
    ) -> Vec<Vec<Option<egui::Color32>>> {
        let mut grid = vec![vec![None; w]; h];
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
                let area = edge(a.pos, b.pos, c.pos);
                if area == 0.0 {
                    continue; // 退化成线的三角形，跳过
                }
                let x0 = a.pos.x.min(b.pos.x).min(c.pos.x).floor().max(0.0) as usize;
                let x1 = (a.pos.x.max(b.pos.x).max(c.pos.x).ceil() as usize).min(w);
                let y0 = a.pos.y.min(b.pos.y).min(c.pos.y).floor().max(0.0) as usize;
                let y1 = (a.pos.y.max(b.pos.y).max(c.pos.y).ceil() as usize).min(h);
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
                            // 带贴图的三角形（文字走这条）：按重心插值取 uv，透明像素不画。
                            let solid = match sample_texture(mesh.texture_id, egui::pos2(
                                (a.uv.x * d[0] + b.uv.x * d[1] + c.uv.x * d[2]) / area,
                                (a.uv.y * d[0] + b.uv.y * d[1] + c.uv.y * d[2]) / area,
                            )) {
                                None => true,
                                Some(px) => px.a() > 0,
                            };
                            if solid {
                                grid[y][x] = Some(a.color);
                            }
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

    /// 实体左右两边的像素数必须严格相等 - 影线那 1px 正好落在正中那一列。
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

    /// 颜色面板的尺寸与栅格：本体 26 格宽 x 36 格高 = cells(26) x cells(36) = 235x325，
    /// 去掉两端各 1px 边框剩 233px 内容列（26 个内容格 + 25 条内部线格）；
    /// SV 调色板正好 26x16 个 8x8 色块，块与块之间那 1px 就是共用的网格线。
    #[test]
    fn color_panel_sits_on_the_grid() {
        assert_eq!(CP_W as i32, 235, "面板宽");
        assert_eq!(CP_H as i32, 325, "面板高");
        assert_eq!((CP_W as i32 - 1) % 9, 0, "面板宽必须是 cells(n)（9n+1）");
        assert_eq!((CP_H as i32 - 1) % 9, 0, "面板高必须是 cells(n)（9n+1）");
        assert_eq!(CP_CONTENT_W as i32, 233, "内容列宽");
        // 内部 11 段加起来正好 36 格。
        assert_eq!(
            CP_TITLE_ROWS
                + CP_SV_ROWS
                + CP_HUE_ROWS
                + 3 * CP_SLIDER_ROWS
                + 3 * CP_SLIDER_ROWS
                + CP_HEX_ROWS
                + CP_BTN_ROWS,
            CP_ROWS,
            "内部各段格数加起来不是面板总格数"
        );
        assert_eq!(SV_COLS * SV_ROWS, 416, "SV 调色板应该是 26x16 块");
        assert_eq!(SV_BLOCK as i32, 8, "每块应该是 8x8");
        // 26 列色块 + 25 条共用线正好铺满内容宽；16 行 + 15 条线正好铺满 SV 段内容高。
        assert_eq!(
            SV_COLS as f32 * SV_BLOCK + (SV_COLS - 1) as f32 * LINE_W,
            CP_CONTENT_W
        );
        assert_eq!(
            SV_ROWS as f32 * SV_BLOCK + (SV_ROWS - 1) as f32 * LINE_W,
            cells(CP_SV_ROWS) - 2.0 * LINE_W
        );
        // 色相条：233 根 1px 竖条正好铺满内容宽。
        assert_eq!(HUE_BARS as f32, CP_CONTENT_W);
        // 滑块一行的横向分格加起来正好是面板的 26 格；轨道和把手都是整格（尺寸 = cells(n)）。
        assert_eq!(
            CP_SLIDER_LABEL_COLS + CP_SLIDER_GAP_COLS + CP_TRACK_COLS + CP_SLIDER_GAP_COLS
                + CP_SLIDER_VALUE_COLS,
            CP_COLS
        );
        assert_eq!((CP_TRACK_W as i32 - 1) % 9, 0, "轨道宽必须是 cells(n)");
        assert_eq!((CP_HANDLE_W as i32 - 1) % 9, 0, "把手宽必须是 cells(n)");
        // 底部两个按钮各 13 格 = 118px，共用中间那条竖线后正好等于面板宽。
        assert_eq!(CP_HALF_W as i32, 118, "底部两个按钮各占一半");
        assert_eq!(CP_LEFT_W as i32, 118, "左半边");
        assert_eq!(CP_LEFT_W + CP_HALF_W - LINE_W, CP_W);
        // Hex 输入框 2 格 = 19px：上沿在 CP_Y_HEX、下沿正好压在 CP_Y_BTN 上（288 + 18 = 306）。
        assert_eq!((CP_Y_BTN - CP_Y_HEX) as i32, CP_HEX_ROWS * 9, "Hex 段高不是 2 格");
        assert_eq!(CP_Y_HEX as i32, 288, "Hex 段上沿");
        assert_eq!(CP_Y_BTN as i32, 306, "底部按钮段上沿");
    }

    /// Hex：只认 # 加正好 6 位十六进制；三位简写、rgb()、位数不对一律非法；显示一律大写。
    #[test]
    fn hex_text_is_strict() {
        assert_eq!(parse_hex("#2288FF"), Some((0x22, 0x88, 0xFF)));
        assert_eq!(parse_hex("#2288ff"), Some((0x22, 0x88, 0xFF)));
        assert_eq!(hex_of((0x22, 0x88, 0xFF)), "#2288FF");
        for bad in [
            "#28F",
            "rgb(34,136,255)",
            "2288FF",
            "#2288F",
            "#2288FFF",
            "#GG88FF",
            "#",
            "",
        ] {
            assert_eq!(parse_hex(bad), None, "{bad} 竟然被当成合法颜色了");
        }
    }

    /// HSV / RGB 换算：六个基准色对得上；整片颜色绕一圈回来最多差 2 个色阶（H 取整到 1 度）。
    #[test]
    fn color_conversions_are_sane() {
        assert_eq!(hsv_to_rgb(0, 255, 255), (255, 0, 0));
        assert_eq!(hsv_to_rgb(120, 255, 255), (0, 255, 0));
        assert_eq!(hsv_to_rgb(240, 255, 255), (0, 0, 255));
        assert_eq!(hsv_to_rgb(0, 0, 255), (255, 255, 255));
        assert_eq!(hsv_to_rgb(0, 0, 0), (0, 0, 0));
        assert_eq!(rgb_to_hsv(255, 0, 0), (0, 255, 255));
        assert_eq!(rgb_to_hsv(0, 0, 0), (0, 0, 0));
        let near = |a: u8, b: u8| (a as i32 - b as i32).abs() <= 2;
        for r in (0..=255).step_by(17) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(17) {
                    let (h, s, v) = rgb_to_hsv(r, g, b);
                    let back = hsv_to_rgb(h, s, v);
                    assert!(
                        near(back.0, r) && near(back.1, g) && near(back.2, b),
                        "rgb({r},{g},{b}) 绕 HSV 一圈变成 {back:?}，中间是 ({h},{s},{v})"
                    );
                }
            }
        }
    }

    /// 面板里的块 <-> 数值映射：两端必须正好落在最小值和满值上（点最左最小、点最右最大）。
    #[test]
    fn palette_index_maps_hit_both_ends() {
        assert_eq!(s_of_col(0), 0);
        assert_eq!(s_of_col(SV_COLS - 1), 255);
        assert_eq!(v_of_row(0), 255);
        assert_eq!(v_of_row(SV_ROWS - 1), 0);
        assert_eq!(col_of_s(0), 0);
        assert_eq!(col_of_s(255), SV_COLS - 1);
        assert_eq!(row_of_v(255), 0);
        assert_eq!(row_of_v(0), SV_ROWS - 1);
        assert_eq!(hue_of_bar(0), 0);
        assert_eq!(hue_of_bar(HUE_BARS - 1), 360);
        assert_eq!(hue_step(0), 0);
        assert_eq!(hue_step(360), HUE_STEPS);
        assert_eq!(hue_of_step(0), 0);
        assert_eq!(hue_of_step(HUE_STEPS), 360);
        assert_eq!(bar_value(0, 255), 0);
        assert_eq!(bar_value(CP_TRACK_INNER_W - 1, 255), 255);
        assert_eq!(bar_value(CP_TRACK_INNER_W - 1, 360), 360);
        assert_eq!(handle_step(0.0), 0);
        assert_eq!(handle_step(1.0), CP_HANDLE_STEPS);
    }
    /// 面板真画出来的样子：本体正好 235x325（最外圈那一圈像素是默认边框色），SV 调色板逐块点名
    /// （26x16 块、每块 8x8，块与块之间那条共用网格线只有 1px），色相条逐根点名
    /// （233 根 1px 竖条，每根都是它自己那个色相角的纯色）。
    #[test]
    fn color_panel_paints_blocks_and_bars() {
        let mut app = color_panel_app();
        let grid = render_panel(&mut app);

        let px = CP_DEFAULT_X as usize;
        let py = CP_DEFAULT_Y as usize;
        let (w, h) = (CP_W as usize, CP_H as usize);

        // 外圈那一圈像素是 1px 硬边（占的是网格线格，不属于本体）。
        assert_eq!(grid[py][px], Some(C_CTRL_BORDER), "面板左上角应该是边框");
        assert_eq!(grid[py][px + w - 1], Some(C_CTRL_BORDER), "面板右上角应该是边框");
        assert_eq!(
            grid[py + h - 1][px + w - 1],
            Some(C_CTRL_BORDER),
            "面板右下角应该是边框"
        );
        // 边框再往外一个像素什么都没画：本体尺寸正好是 w x h。
        assert_eq!(grid[py + 100][px + w], None, "面板右边多画了一个像素");
        assert_eq!(grid[py + h][px + 100], None, "面板下边多画了一个像素");
        // 最上面那条是灰标题栏；滑块行里轨道左边的空白是工作区白。
        assert_eq!(grid[py + 4][px + 2], Some(C_TITLE_BG), "标题栏底色");
        assert_eq!(
            grid[py + CP_Y_RGB as usize + 8][px + 20],
            Some(C_WORK_BG),
            "滑块行空白处应该是工作区白"
        );

        // SV 调色板：逐块点名（取块中心，避开块之间的网格线和选中框）。
        let sx = px + LINE_W as usize;
        let sy = py + CP_Y_SV as usize + LINE_W as usize;
        for row in 0..SV_ROWS {
            for col in 0..SV_COLS {
                let want = rgb32(hsv_to_rgb(app.color_panel.h, s_of_col(col), v_of_row(row)));
                let x = sx + col as usize * PITCH as usize + 4;
                let y = sy + row as usize * PITCH as usize + 4;
                assert_eq!(grid[y][x], Some(want), "SV 块 ({col}, {row}) 的颜色不对");
            }
        }
        // 选中那块四周那圈线按底色反差取色（这块色偏暗 -> 白框）。
        let sel = egui::Rect::from_min_size(
            egui::pos2(
                sx as f32 - LINE_W + col_of_s(app.color_panel.s) as f32 * PITCH,
                sy as f32 - LINE_W + row_of_v(app.color_panel.v) as f32 * PITCH,
            ),
            egui::vec2(SV_BLOCK + 2.0 * LINE_W, SV_BLOCK + 2.0 * LINE_W),
        );
        assert_eq!(
            grid[sel.min.y as usize][(sel.min.x + 5.0) as usize],
            Some(contrast_on(app.color_panel.rgb)),
            "选中框的颜色不对（应该按底色反差取色）"
        );
        // 相邻两块之间那条 1px 竖线是共用的网格线（默认边框色）：证明是一块一块拼的，不是平滑渐变。
        // 选中框那 10px 宽的一圈会盖住它两侧的网格线，那几列跳过。
        let sel_cover = sel.min.x as usize..sel.max.x as usize;
        for col in 0..SV_COLS - 1 {
            let x = sx + (col as usize + 1) * PITCH as usize - 1;
            if sel_cover.contains(&x) {
                continue;
            }
            assert_eq!(
                grid[sy + 4][x],
                Some(C_CTRL_BORDER),
                "第 {col} 块和第 {} 块之间那条网格线不见了",
                col + 1
            );
        }

        // 色相条：每根 1px 竖条都应该是它自己那个色相角的纯色（跳过被把手压住的那 10px）。
        let hx = px + LINE_W as usize;
        let hy = py + CP_Y_HUE as usize + LINE_W as usize;
        let handle_x0 = px + hue_handle_left(app.color_panel.h) as usize;
        let covered = handle_x0..handle_x0 + CP_HANDLE_W as usize;
        for i in 0..HUE_BARS as usize {
            let x = hx + i;
            if covered.contains(&x) {
                continue;
            }
            let want = rgb32(hsv_to_rgb(hue_of_bar(i as i32), 255, 255));
            assert_eq!(grid[hy + 8][x], Some(want), "第 {i} 根色相条的颜色不对");
        }
    }

    /// 把手只停在整像素列上：0..=255（R / G / B / S / V）和 0..=360（H）每个取值都过一遍，
    /// 左边缘不许出现小数，也不许越出轨道那 CP_HANDLE_TRAVEL 像素的可走范围。
    #[test]
    fn slider_handles_land_on_whole_pixels() {
        // 轨道起点 = 面板左边缘 + 第 CP_TRACK_LINE 条竖网格线相对面板左边缘的偏移。
        let track_x = CP_DEFAULT_X + grid_line(CP_TRACK_LINE);
        assert!(track_x.fract() == 0.0, "轨道起点本身就得在整像素上");
        for max in [255, 360] {
            for v in 0..=max {
                let hx = handle_left(track_x, v as f32 / max as f32);
                assert!(hx.fract() == 0.0, "值 {v}/{max} 的把手落在小数像素 {hx} 上");
                assert!(
                    hx >= track_x && hx <= track_x + CP_HANDLE_TRAVEL,
                    "值 {v}/{max} 的把手跑出轨道可走范围了：{hx}"
                );
            }
        }
    }

    /// Hex 输入框那圈边框必须压在网格线格上：上边正好是 CP_Y_HEX、下边正好是 CP_Y_BTN
    /// （它同时是底部按钮的顶边）这两条横网格线 - 上边不能少画（被分隔线盖成灰的），
    /// 下边也不能和旁边那条线叠成 2px。非法输入时整圈变红，红边同样压在网格线上。
    #[test]
    fn hex_box_frame_sits_on_grid_lines() {
        let px = CP_DEFAULT_X as usize;
        let py = CP_DEFAULT_Y as usize;
        let x = px + LINE_W as usize + 20;
        let top = py + CP_Y_HEX as usize;
        let bottom = py + CP_Y_BTN as usize;

        // 合法：上下两条边都是默认边框色，上下各一个像素都是干净的（不多出一条线）。
        let mut app = color_panel_app();
        let grid = render_panel(&mut app);
        assert_eq!(grid[top][x], Some(C_CTRL_BORDER), "上边框没压在网格线上");
        assert_eq!(grid[bottom][x], Some(C_CTRL_BORDER), "下边框没压在网格线上");
        assert_eq!(grid[top - 1][x], Some(C_WORK_BG), "上边框上面还多了一条线");
        assert_eq!(grid[bottom - 1][x], Some(C_WORK_BG), "下边框上面还多了一条线");
        assert_eq!(grid[bottom + 1][x], Some(C_BTN_FACE), "下边框下面还多了一条线");

        // 非法：整圈变红，而且红边一样压在网格线上（不会被分隔线盖成灰的）。
        let mut app = color_panel_app();
        app.color_panel.hex.set_text("#2288F");
        let grid = render_panel(&mut app);
        assert_eq!(grid[top][x], Some(C_CLOSE_HOVER), "非法时上边框没变红");
        assert_eq!(grid[bottom][x], Some(C_CLOSE_HOVER), "非法时下边框没变红");
        assert_eq!(grid[bottom - 1][x], Some(C_WORK_BG), "非法时下边框上面还有一条线");
    }

    /// 竖着的线也必须对齐：面板左右两条外框就是各段控件（Hex 输入框 / 色相条 / 底部按钮）的
    /// 左右边框 - 全得落在同一列上；底部中间那条共用的竖线只能占 1px（左右两半共用）。
    /// 这一条就是「8px 格 + 1px 网格线」在横向上的体现：内容列 233px 夹在两条 1px 边框之间。
    #[test]
    fn panel_right_edges_line_up() {
        let mut app = color_panel_app();
        let grid = render_panel(&mut app);

        let px = CP_DEFAULT_X as usize;
        let py = CP_DEFAULT_Y as usize;
        let (w, _h) = (CP_W as usize, CP_H as usize);
        let inner_l = px + LINE_W as usize;
        let inner_r = px + w - 1 - LINE_W as usize;
        // 各取一段的中间某一行，避开横向的分隔线。
        let hex_row = py + CP_Y_HEX as usize + 8;
        let btn_row = py + CP_Y_BTN as usize + 8;
        let hue_row = py + CP_Y_HUE as usize + 8;

        // Hex 输入框：左右两条边就是面板外框那两列，往里一格才是内容（白底）。
        assert_eq!(grid[hex_row][px], Some(C_CTRL_BORDER), "Hex 输入框左边框不对");
        assert_eq!(
            grid[hex_row][px + w - 1],
            Some(C_CTRL_BORDER),
            "Hex 输入框右边框不对"
        );
        assert_eq!(grid[hex_row][inner_l], Some(C_WORK_BG), "Hex 输入框左边多了一条线");
        assert_eq!(grid[hex_row][inner_r], Some(C_WORK_BG), "Hex 输入框右边多了一条线");

        // 色相条：横贯整条内容列，最右那根 1px 竖条也要压在这一列上，右边不能留白。
        assert_ne!(grid[hue_row][inner_r], Some(C_WORK_BG), "色相条右边留了白边");
        assert_eq!(grid[hue_row][px + w - 1], Some(C_CTRL_BORDER), "色相条右边框不对");

        // 中间那条共用竖线：左半边（应用）的右边框和右半边（取消）的左边框是同一条，只有 1px，
        // 而且必须落在 9px 网格线上（13 格 = 117px，就是正中间那条线）。
        let mid = px + CP_HALF_W as usize - LINE_W as usize;
        assert_eq!((mid - px) % 9, 0, "中间那条共用的竖线没落在 9px 网格线上");
        assert_eq!(grid[btn_row][mid], Some(C_CTRL_BORDER), "中间那条竖线不见了");
        assert_eq!(grid[btn_row][mid - 1], Some(C_BTN_FACE), "中间那条竖线不止 1px");
        assert_eq!(grid[btn_row][mid + 1], Some(C_BTN_FACE), "中间那条竖线不止 1px");

        // 左边框同样要压在内容列第一列上（应用按钮 / 取消按钮的左右边框就是面板外框）。
        assert_eq!(grid[btn_row][px], Some(C_CTRL_BORDER), "应用按钮左边框不对");
        assert_eq!(
            grid[btn_row][px + w - 1],
            Some(C_CTRL_BORDER),
            "取消按钮右边框不对"
        );
    }
    /// 左栏的测试坐标：整块左栏从 (0, 0) 起，高度就是功能区高度。
    fn left_work() -> egui::Rect {
        egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(CHART_PANE_W, cells(WORK_ROWS)),
        )
    }

    /// 把左栏五块按时序画在整窗画布上再光栅化（几何和 ui() 里那套完全一样）。
    fn render_left(app: &mut StockApp) -> (Vec<Vec<Option<egui::Color32>>>, [egui::Rect; 5]) {
        render_left_in(app, left_work(), WINDOW_H as usize)
    }

    /// 同上，但功能区矩形和画布高度自己给：测「窗口拉高之后各块跟着变」要用更大的左栏。
    fn render_left_in(
        app: &mut StockApp,
        work: egui::Rect,
        canvas_h: usize,
    ) -> (Vec<Vec<Option<egui::Color32>>>, [egui::Rect; 5]) {
        app.clamp_left_heights(work);
        let [kline, bar, volume, bar2, indicator] =
            StockApp::left_rows(work, app.kline_h, app.vol_h);
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        // 字形图集挂在 ctx 上：这份 dump 用自己的 ctx，引擎就得重建在这个 ctx 上，
        // 否则光栅器收不到字形贴图，dump 出来的文字又变回一个个实心方块。
        if let Some(engine) = HardText::new(&ctx) {
            app.text_engine = Some(engine);
        }
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(WINDOW_W, canvas_h as f32),
        ));
        let mut out = ctx.run_ui(raw, |ui| {
            app.ui_chart(ui, PANE_CANDLE, kline);
            app.ui_chart(ui, PANE_VOLUME, volume);
            app.ui_chart(ui, PANE_MACD, indicator);
            app.ui_hsplitter(ui, bar, work, LeftSplit::Volume);
            app.ui_hsplitter(ui, bar2, work, LeftSplit::Indicator);
        });
        let shapes = std::mem::take(&mut out.shapes);
        absorb_textures(&mut out.textures_delta);
        (
            rasterize_into(&ctx.tessellate(shapes, 1.0), WINDOW_W as usize, canvas_h),
            [kline, bar, volume, bar2, indicator],
        )
    }

    /// [x0, x1) 这几列里，只要 [y0, y1) 里有一行画着 `color`，就把这一列算数。
    fn color_cols(
        grid: &[Vec<Option<egui::Color32>>],
        x0: usize,
        x1: usize,
        y0: usize,
        y1: usize,
        color: egui::Color32,
    ) -> Vec<usize> {
        (x0..x1)
            .filter(|&x| (y0..y1).any(|y| grid[y][x] == Some(color)))
            .collect()
    }

    /// 量柱和蜡烛严格对齐：同一帧里 K 线区和成交量区画出来的柱子，占的像素列一模一样。
    /// 做法：造一批「全部下跌」的等高等低 K 线（实体实心、量柱都够高），在两个区里各取
    /// 同一个颜色的像素列集合，两边必须完全相等 - 只要有一根柱子偏 1px，集合立刻对不上。
    #[test]
    fn volume_bars_align_with_candles() {
        let bars: Vec<data::random_walk::Bar> = (0..48)
            .map(|i| data::random_walk::Bar {
                o: 100.0,
                h: 101.0,
                l: 89.0,
                c: 90.0,
                vol: 1.0e7 + (i % 7) as f64 * 1.0e6,
                date: data::random_walk::DATE_START.add_days(i as i32),
            })
            .collect();
        let mut app = blank_app();
        app.bars = bars;
        app.scale = SCALE_MIN;

        let (grid, [kline, _bar, volume, _bar2, _ind]) = render_left(&mut app);

        // 两根柱子 / 蜡烛都是「跌色」（这批 K 线全跌）；窗口里只有这两个区会用到这个颜色。
        let down = rgb32(DOWN_RGB_DEFAULT);
        let (_, _, kmain, _) = chart_rows(kline, app.panes[PANE_CANDLE].indicator.layout());
        let (kplot, _) = frame_rects(kmain);
        let (vplot, _) = frame_rects(volume);
        let k_cols = color_cols(
            &grid,
            kplot.min.x as usize,
            kplot.max.x as usize,
            kplot.min.y as usize,
            kplot.max.y as usize,
            down,
        );
        let v_cols = color_cols(
            &grid,
            vplot.min.x as usize,
            vplot.max.x as usize,
            volume.min.y as usize,
            volume.max.y as usize,
            down,
        );
        assert!(!k_cols.is_empty(), "蜡烛一根都没画出来");
        assert_eq!(k_cols, v_cols, "量柱和蜡烛占的像素列对不上（没严格对齐）");
    }

    /// 左栏上下五块的几何：三块图表不分主图 / 副图，用同一套规矩 - 高度都是整数格
    /// cells(n) = 9n + 1；每两块之间夹着一条 1 格（cells(1) = 10px）的横向拖拽条；相邻两块
    /// 共用边界那条 1px 网格线，所以整列严丝合缝落在窗口的 9px 网格线上。最后一块铺到左栏底边，
    /// 三块各自不低于统一下限 PANE_H_MIN。拖动 / 换窗口尺寸之后也还是这个规矩。
    #[test]
    fn left_column_splits_on_the_grid() {
        assert_eq!(KLINE_H_DEFAULT as i32, 469, "图表1 默认 52 格 = cells(52)");
        assert_eq!(VOL_H_DEFAULT as i32, 190, "图表2 默认 21 格 = cells(21)");
        assert_eq!(IND_H_DEFAULT as i32, 190, "图表3 默认 21 格 = cells(21)");
        assert_eq!(PANE_H_MIN as i32, 181, "三块图表统一下限 20 格 = cells(20)");
        assert_eq!(HSPLIT_H as i32, 10, "横向拖拽条 1 格 = cells(1) = 10px");
        assert_eq!(LEFT_FIXED_H as i32, 16, "固定开销 = 两条拖拽条 - 四条共用线");

        let work = egui::Rect::from_min_size(
            egui::pos2(0.0, 54.0),
            egui::vec2(CHART_PANE_W, cells(WORK_ROWS)),
        );
        let mut app = blank_app();
        // 两条拖拽条各拖到两个极端：每次收口完都要满足全部约束。
        for (want_vol, want_ind) in [
            (VOL_H_DEFAULT, IND_H_DEFAULT),
            (0.0, 0.0),
            (10000.0, 10000.0),
            (300.0, 300.0),
            (PANE_H_MIN, 5000.0),
            (5000.0, PANE_H_MIN),
        ] {
            app.vol_h = want_vol;
            app.ind_h = want_ind;
            app.clamp_left_heights(work);
            let [kline, bar, vol, bar2, ind] = StockApp::left_rows(work, app.kline_h, app.vol_h);
            // 三块高度都是整数格（cells(n) = 9n + 1），谁也不低于统一下限。
            for h in [app.kline_h, app.vol_h, app.ind_h] {
                assert_eq!((h as i32 - 1) % 9, 0, "块高 {h} 不是整数格 cells(n)");
                assert!(h >= PANE_H_MIN, "块高 {h} 低于统一最小高度");
            }
            assert_eq!(bar.height(), HSPLIT_H, "上面那条拖拽条高度不是 1 格");
            assert_eq!(bar2.height(), HSPLIT_H, "下面那条拖拽条高度不是 1 格");
            // 排版矩形的高度就是收口后的格高，一分不多一分不少。
            assert_eq!(kline.height(), app.kline_h, "图表1 高度和收口值对不上");
            assert_eq!(vol.height(), app.vol_h, "图表2 高度和收口值对不上");
            assert_eq!(ind.height(), app.ind_h, "图表3 高度和收口值对不上");
            // 从上往下严丝合缝，相邻两块共用边界那条 1px 网格线。
            assert_eq!(kline.min.y, work.min.y);
            assert_eq!(bar.min.y, kline.max.y - LINE_W);
            assert_eq!(vol.min.y, bar.max.y - LINE_W);
            assert_eq!(bar2.min.y, vol.max.y - LINE_W);
            assert_eq!(ind.min.y, bar2.max.y - LINE_W);
            // 第三块正好铺到左栏底部：底下不留白条，五块加起来不多不少。
            assert_eq!(ind.max.y, work.max.y, "图表3 没铺到左栏底部");
            assert_eq!(
                kline.height() + LEFT_FIXED_H + vol.height() + ind.height(),
                work.height(),
                "五块加起来不等于左栏高度"
            );
            for r in [kline, bar, vol, bar2, ind] {
                assert!(r.min.x.fract() == 0.0 && r.min.y.fract() == 0.0);
            }
        }
        // 默认窗口高度下：三块各保持规格里的默认高度（52 / 21 / 21 格）。
        let mut app = blank_app();
        app.clamp_left_heights(work);
        assert_eq!(app.kline_h, KLINE_H_DEFAULT);
        assert_eq!(app.vol_h, VOL_H_DEFAULT);
        assert_eq!(app.ind_h, IND_H_DEFAULT);
    }

    /// 上下三块之间的线格：相邻两块共用边界那条线，永远只有 1px，既不缺一条也不叠成 2px；
    /// 横向拖拽条整条 1 格（cells(1) = 10px）= 上 1px 线 + 8px 本体 + 下 1px 线，
    /// 上下两块自己不再画那条边界线，所以边界处永远只有 1px。三块图表的内部分段同理。
    #[test]
    fn left_column_lines_are_1px() {
        let mut app = blank_app();
        let (grid, [kline, bar, volume, bar2, indicator]) = render_left(&mut app);
        let x = 100usize; // 左栏正中某一列：不落在任何竖线上
        let px = |y: usize| grid[y][x];

        // 图表1 内部：标题栏下沿、状态带下沿、时间带上沿，各 1px（上下邻居都是干净底色）。
        let layout = app.panes[PANE_CANDLE].indicator.layout();
        let (title, status, main, time) = chart_rows(kline, layout);
        let (info, time) = (status.unwrap(), time.unwrap());
        assert_eq!(px(title.max.y as usize - 1), Some(C_LINE), "标题栏下沿那条线不对");
        assert_eq!(px(title.max.y as usize - 2), Some(C_TITLE_BG), "标题栏下沿多画了一行");
        assert_eq!(px(main.min.y as usize - 1), Some(C_LINE), "状态带下沿那条线不对");
        assert_eq!(px(main.min.y as usize - 2), Some(C_WORK_BG), "状态带下沿多画了一行");
        assert_eq!(px(main.min.y as usize), Some(C_CHART_BG), "绘图区没紧跟着状态带");
        assert_eq!(px(time.min.y as usize), Some(C_LINE), "时间带上沿那条线不对");
        assert_eq!(px(time.min.y as usize + 1), Some(C_WORK_BG), "时间带没紧跟着那条线");
        assert_eq!(info.min.y, title.max.y - LINE_W, "状态带和标题栏没共用那条线");
        assert_eq!(time.max.y, kline.max.y, "时间带没铺到图表底边");

        // 图表1 与上面那条拖拽条之间：上 1px 线 + 8px 本体 + 下 1px 线，然后立刻是图表2 的标题栏。
        let b1 = bar.min.y as usize;
        assert_eq!(bar.height() as usize, HSPLIT_H as usize, "拖拽条不是 1 格");
        assert_eq!(px(b1 - 1), Some(C_WORK_BG), "图表1 下沿多画了一行");
        assert_eq!(px(b1), Some(C_CTRL_BORDER), "图表1 / 拖拽条共用的上边线不对");
        for r in 1..(HSPLIT_H as usize - 1) {
            assert_eq!(px(b1 + r), Some(C_SPLITTER), "拖拽条第 {r} 行不是本体色");
        }
        assert_eq!(px(b1 + HSPLIT_H as usize - 1), Some(C_CTRL_BORDER), "拖拽条下沿那条线不对");
        assert_eq!(volume.min.y as usize, b1 + HSPLIT_H as usize - 1, "图表2 和拖拽条没共用那条线");
        assert_eq!(px(volume.min.y as usize + 1), Some(C_TITLE_BG), "图表2 的标题栏没紧跟着拖拽条");

        // 图表2 与下面那条拖拽条之间：同样上 1px + 8px + 下 1px，然后立刻是图表3 的标题栏。
        let b2 = bar2.min.y as usize;
        assert_eq!(bar2.height() as usize, HSPLIT_H as usize, "下面那条拖拽条不是 1 格");
        assert_eq!(px(b2 - 1), Some(C_CHART_BG), "图表2 下沿多画了一行");
        assert_eq!(px(b2), Some(C_CTRL_BORDER), "图表2 / 拖拽条共用的上边线不对");
        for r in 1..(HSPLIT_H as usize - 1) {
            assert_eq!(px(b2 + r), Some(C_SPLITTER), "下面那条拖拽条第 {r} 行不是本体色");
        }
        assert_eq!(px(b2 + HSPLIT_H as usize - 1), Some(C_CTRL_BORDER), "下面那条拖拽条下沿不对");
        assert_eq!(indicator.min.y as usize, b2 + HSPLIT_H as usize - 1, "图表3 和拖拽条没共用那条线");
        assert_eq!(px(indicator.min.y as usize + 1), Some(C_TITLE_BG), "图表3 的标题栏没紧跟着拖拽条");
    }

    /// 几根拖动条（竖向分割条 + 两条横向拖拽条）都落在网格上：整条 1 格（cells(1) = 10px）
    /// = 1px 线 + 8px 本体 + 1px 线，两端那两条 1px 线各压在相邻两块的共用网格线上；
    /// 拖到两个极端之后，左栏宽度 / 三块高度仍都是整数格，两栏都不小于最小宽度。
    #[test]
    fn splitters_sit_on_the_grid() {
        assert_eq!(SPLITTER_W as i32, 10, "竖分割条 1 格 = cells(1)");
        assert_eq!(HSPLIT_H as i32, 10, "横拖拽条 1 格 = cells(1)");
        let work = left_work();

        // 竖向分割条：左栏 / 分割条 / 右栏三块相邻处共用网格线，整条落在网格列上。
        let [left, splitter, panel] = StockApp::work_columns(work, CHART_PANE_W);
        assert_eq!(left.max.x - LINE_W, splitter.min.x, "左栏和分割条没共用那条线");
        assert_eq!(splitter.max.x - LINE_W, panel.min.x, "分割条和右栏没共用那条线");
        assert_eq!(panel.max.x, work.max.x, "右栏没铺到功能区右边");
        assert_eq!(splitter.min.x % 9.0, 0.0, "分割条左边不在竖网格线上");
        assert_eq!((splitter.max.x - LINE_W) % 9.0, 0.0, "分割条右边不在竖网格线上");

        // 分割条拖到两个极端：左栏宽度永远是整数格，两栏都不低于最小宽度。
        let mut app = blank_app();
        for want in [-1000.0, 0.0, 1000.0, 5000.0] {
            app.split_x = want;
            app.clamp_split(work);
            assert_eq!((app.split_x as i32 - 1) % 9, 0, "左栏宽度不是整数格");
            assert!(app.split_x >= PANE_MIN_W, "左栏低于最小宽度");
            assert!(app.split_x <= StockApp::split_max_x(work), "左栏超过最大宽度");
            let [_l, _s, panel] = StockApp::work_columns(work, app.split_x);
            assert!(panel.width() >= PANE_MIN_W - LINE_W, "右栏低于最小宽度");
        }
    }

    /// 时间轴标注：相邻标签中心至少隔 48px（6 格），而且不会宽得离谱（够用就停）。
    #[test]
    fn timeline_labels_never_crowd() {
        let mut app = blank_app();
        for scale in [SCALE_MIN, 5, 9, SCALE_MAX] {
            let advance = bar_advance(scale);
            let step = app.time_step(advance);
            let gap = step as f32 * advance;
            assert!(gap >= TIME_LABEL_GAP_MIN, "scale={scale} 的标注间距只有 {gap}px");
            assert!(
                gap < TIME_LABEL_GAP_MIN + advance,
                "scale={scale} 的标注间隔多留了一格：{gap}px"
            );
        }
    }

    /// 状态带里那几项（蜡烛图状态条的 8 项）的宽度必须是整数格 cells(n) = 9n + 1 -
    /// 相邻两项共用边界那条 1px 网格线，点状虚线分割线正好落在那条线上。
    /// 状态条由指标自己声明（indicator::candle），项宽就是重放时那一套算式。
    /// 16px 点阵字比原来的 12px 宽得多，默认左栏宽度（89 格 = 802px）一排放不下全部 8 项：
    /// 排不下的由重放那边丢掉（见 replay_cmds 的 `break`），绝不许挤在一起或溢出面板。
    #[test]
    fn status_fields_sit_on_the_grid() {
        use crate::indicator::candle::{INFO_LABELS, INFO_TEMPLATES};
        // 用文字引擎量（硬边引擎只影响是否有字形；宽度只跟字符占几格有关）。
        let ctx = egui::Context::default();
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        if app.text_engine.is_none() {
            return; // 没拿到 GDI 字体就跳过（正常 Windows 上不会走到这儿）
        }
        let mut sx = 0.0;
        let mut fit = 0;
        for i in 0..INFO_LABELS.len() {
            let w = snap_cells_up(
                app.text_width(INFO_LABELS[i])
                    + GRID
                    + app.text_width(INFO_TEMPLATES[i])
                    + 2.0 * TEXT_PAD_X,
            );
            assert_eq!(
                (w as i32 - 1) % 9,
                0,
                "第 {i} 项「{}」宽 {w} 不是整数格 cells(n)",
                INFO_LABELS[i]
            );
            assert!(w >= cells(1), "第 {i} 项比一格还窄");
            if sx + w > CHART_PANE_W {
                break;
            }
            sx += w - LINE_W;
            fit += 1;
        }
        // 默认左栏宽度下一排正好放得下前 6 项（日期 / 开 / 高 / 低 / 收 / 涨跌）；后两项放不下、
        // 由重放那边丢掉 - 这是「绝不溢出」的见证。窗口拉宽 / 左栏变宽，能放下的项数就跟着变多。
        assert_eq!(fit, 6, "默认左栏一排放得下的项数变了：{fit} 项");
        assert!(sx + LINE_W <= CHART_PANE_W, "状态带排出来的宽度超过了左栏");
        // 时间轴标注间隔用真字体量出来的话也够宽（相邻标签中心至少 48px）。
        for scale in [SCALE_MIN, 5, SCALE_MAX] {
            let advance = bar_advance(scale);
            assert!(app.time_step(advance) as f32 * advance >= TIME_LABEL_GAP_MIN);
        }
    }

    /// 成交量刻度 / 信息栏用的单位：1e12 起步「兆」/ 1e8 起步「亿」（都取整）、1e4 起步「万」
    /// （整数部分 <= 3 位留 1 位小数，4 位就取整），更小给整数。
    /// 最长的一档 "999.9万" 正好 7 格 = 纵轴给数值留的宽度上限（VALUE_COLS_MAX）。
    #[test]
    fn volume_text_uses_wan_and_yi() {
        use crate::indicator::volume::fmt_volume;
        assert_eq!(fmt_volume(2.0e12), "2兆");
        assert_eq!(fmt_volume(1.5e8), "2亿");
        assert_eq!(fmt_volume(1.0e8), "1亿");
        assert_eq!(fmt_volume(9.99e7), "9990万");
        assert_eq!(fmt_volume(2.3456e7), "2346万");
        assert_eq!(fmt_volume(1.0e4), "1.0万");
        assert_eq!(fmt_volume(9999.0), "9999");
        assert_eq!(fmt_volume(0.0), "0");
        // 最宽的那一档也不许超过纵轴给数值的 7 个内容格，不然标签会顶出轴那一列。
        for (v, s) in [(9.999e7, "9999万"), (1.0e12, "1兆"), (9.9e5, "99.0万")] {
            assert!(
                crate::font::text_width(&fmt_volume(v)) <= VALUE_COLS_MAX * 9,
                "{s} 比纵轴留给数值的 7 格还宽"
            );
        }
    }

    /// 行情日期：只走工作日（跳过周末）、逐日递增，512 根正好跨过新年
    /// （时间轴上那种 YY-MM-DD 的跨年标签才有得测）；成交量落在万 / 亿两个量级里。
    #[test]
    fn bar_dates_skip_weekends() {
        let bars = data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START);
        assert_eq!(bars[0].date, data::random_walk::Date::new(2024, 1, 2));
        // 2024-01-05 是周五，下一根直接跳到 01-08 周一。
        assert_eq!(bars[3].date, data::random_walk::Date::new(2024, 1, 5));
        assert_eq!(bars[4].date, data::random_walk::Date::new(2024, 1, 8));
        assert_eq!(bars[5].date, data::random_walk::Date::new(2024, 1, 9));
        for w in bars.windows(2) {
            assert!(w[0].date < w[1].date, "日期没有严格向后走");
        }
        assert_ne!(
            bars[0].date.y,
            bars[BAR_COUNT - 1].date.y,
            "512 根应该跨过一个新年"
        );
        use crate::indicator::volume::{WAN, YI};
        let max = bars.iter().fold(0.0f64, |a, b| a.max(b.vol));
        assert!(max > YI, "最大成交量 {max} 没上亿，刻度里就只剩「万」了");
        assert!(bars.iter().all(|b| b.vol > WAN && b.h >= b.o.max(b.c) && b.l <= b.o.min(b.c)));
    }

    /// 窗口尺寸由客户区决定：默认尺寸下四区域落位和原来的固定布局一模一样；
    /// 客户区变大之后状态栏还贴着底，多出来的高度全部给功能区。
    #[test]
    fn window_layout_follows_the_client_rect() {
        let base = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        let a = split_areas(base);
        assert_eq!(a[0].min.y, 0.0);
        assert_eq!(a[0].max.y, TITLEBAR_H);
        assert_eq!(a[1].min.y, grid_line(MENU_LINE));
        assert_eq!(a[2].min.y, grid_line(WORK_LINE));
        assert_eq!(a[2].height(), cells(WORK_ROWS), "默认尺寸下功能区高度变了");
        assert_eq!(a[3].min.y, WINDOW_H - STATUS_H);
        assert_eq!(a[3].max.y, WINDOW_H);
        // 功能区下沿和状态栏上沿共用一条线格（重叠 1px）。
        assert_eq!(a[2].max.y, a[3].min.y + LINE_W);

        // 客户区加高 180px：状态栏仍在底部，多出来的高度全归功能区。
        let big = egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(WINDOW_W, WINDOW_H + 180.0),
        );
        let b = split_areas(big);
        assert_eq!(b[1].min.y, a[1].min.y);
        assert_eq!(b[2].min.y, a[2].min.y);
        assert_eq!(b[3].max.y, big.max.y, "状态栏没贴着底");
        assert_eq!(b[2].max.y, b[3].min.y + LINE_W);
        assert_eq!(b[2].height() - a[2].height(), 180.0, "多出来的高度没给功能区");

        // 功能区变高：图表1 先长（它拿剩下的空间），图表2 / 图表3 保持规格里的 21 格（190px）；
        // 底部不留白条，三块高度仍是整数格。
        let mut base_app = blank_app();
        base_app.clamp_left_heights(a[2]);
        let mut big_app = blank_app();
        big_app.clamp_left_heights(b[2]);
        assert_eq!(base_app.kline_h, KLINE_H_DEFAULT, "默认窗口下图表1 应该是 52 格");
        assert_eq!(base_app.vol_h, VOL_H_DEFAULT, "图表2 默认 21 格");
        assert_eq!(base_app.ind_h, IND_H_DEFAULT, "图表3 默认 21 格");
        assert_eq!((big_app.kline_h as i32 - 1) % 9, 0, "图表1 高度不是整数格");
        assert_eq!(big_app.vol_h, VOL_H_DEFAULT, "图表2 不该被窗口高度改掉");
        assert_eq!(big_app.ind_h, IND_H_DEFAULT, "图表3 不该被窗口高度改掉");
        assert!(
            big_app.kline_h > base_app.kline_h,
            "窗口拉高后图表1 没跟着变高：{} -> {}",
            base_app.kline_h,
            big_app.kline_h
        );
        let [_k, _bar, _vol, _bar2, ind] =
            StockApp::left_rows(b[2], big_app.kline_h, big_app.vol_h);
        assert_eq!(ind.max.y, b[2].max.y, "图表3 没铺到左栏底部");
    }

    /// 把拖拽条往上拖（成交量区变高）时里面的量柱跟着拉伸：柱子从量区底部的 8px 边距起，
    /// 高度随量区高度走 - 不是固定高度的一小段。
    #[test]
    fn volume_bars_stretch_with_the_pane() {
        let bars: Vec<data::random_walk::Bar> = (0..48)
            .map(|i| data::random_walk::Bar {
                o: 100.0,
                h: 101.0,
                l: 89.0,
                c: 90.0,
                vol: 1.0e7 + (i % 7) as f64 * 1.0e6,
                date: data::random_walk::DATE_START.add_days(i as i32),
            })
            .collect();
        let down = rgb32(DOWN_RGB_DEFAULT);
        // 同一批数据、同一个可见区间，只把成交量区拉高：量柱占的像素数必须变多（被拉长了）。
        let count_down = |grid: &Vec<Vec<Option<egui::Color32>>>, r: egui::Rect| -> usize {
            (r.min.y as usize..r.max.y as usize)
                .map(|y| {
                    (r.min.x as usize..r.max.x as usize)
                        .filter(|&x| grid[y][x] == Some(down))
                        .count()
                })
                .sum()
        };

        let mut app = blank_app();
        app.bars = bars;
        app.scale = SCALE_MIN;
        // 左栏高度不变，只改成交量区高度（就是拖那根横向拖拽条干的事）。
        let work = left_work();
        // 图表3 压到最小，把纵向空间尽量让给图表2，好把「量区拉高」这件事测出来。
        app.ind_h = PANE_H_MIN;
        app.vol_h = PANE_H_MIN;
        let (grid_short, [_, _, vol_short, _, _]) =
            render_left_in(&mut app, work, WINDOW_H as usize);
        let short_px = count_down(&grid_short, vol_short);

        app.vol_h = cells(40); // 40 格，明确比最小高度（20 格）高一大截
        let (grid_tall, [_, _, vol_tall, _, ind_tall]) =
            render_left_in(&mut app, work, WINDOW_H as usize);
        let tall_px = count_down(&grid_tall, vol_tall);

        // 现在最下面那块是指标区，它才顶到左栏底边；量区下沿在它上面。
        assert_eq!(ind_tall.max.y, work.max.y, "指标区下沿没贴住左栏底部");
        assert!(vol_tall.max.y < work.max.y, "量区下面还有指标区，不该顶到左栏底");
        assert!(vol_tall.min.y < vol_short.min.y, "量区上沿没跟着上移");
        assert!(
            vol_tall.height() > vol_short.height() + 120.0,
            "量区没被拉高：{} -> {}",
            vol_short.height(),
            vol_tall.height()
        );
        assert!(tall_px > short_px, "量柱没跟着量区一起拉伸");
        // 柱子上下两端都锚在量区里：最高那根顶到上边距（量区顶 + 8px），所有柱子都落到
        // 底部边距（量区底 - 8px）- 量区一拉高，柱子的两个端点跟着走，这就是拉伸。
        let (_, _, vmain, _) = chart_rows(vol_tall, crate::indicator::ChartLayout::NONE);
        let (vplot, _) = frame_rects(vmain);
        let x = vplot.min.x as usize + 6 * bar_advance(SCALE_MIN) as usize;
        let top = vmain.min.y as usize + CHART_PAD as usize;
        let base = vmain.max.y as usize - CHART_PAD as usize;
        // 这一列就是最高的那根量柱：它占的像素行必须正好从量区上边距开始、落到下边距为止。
        // （上端点落在一个半像素边界上，光栅器可能把它算进也可能不算，所以允许差 1px。）
        let rows: Vec<usize> = (vol_tall.min.y as usize..vol_tall.max.y as usize)
            .filter(|&y| grid_tall[y][x] == Some(down))
            .collect();
        assert!(!rows.is_empty(), "最高的那根量柱没画出来");
        assert!(
            rows[0] == top || rows[0] == top + 1,
            "最高的那根没顶到上边距：第一行 {}，期望 {}",
            rows[0],
            top
        );
        // 0 值落在下边距那一行上：柱子最后一行为 base（和 0 刻度线同一条），上下两块图表一个口径。
        assert_eq!(
            rows[rows.len() - 1],
            base,
            "量柱没落到下边距（量区底 - 8px，0 值那条线上）"
        );
    }

    /// 右侧价格轴的灰底只铺在绘图区那一段：状态带 / 时间带里整条都是工作区白
    /// - 时间带右边不再拖一条灰色尾巴（用户要求时间轴右边整个底色都是白的）。
    #[test]
    fn axis_column_is_gray_only_beside_the_plot() {
        let mut app = blank_app();
        let (grid, [kline, _bar, _vol, _bar2, _ind]) = render_left(&mut app);
        let layout = app.panes[PANE_CANDLE].indicator.layout();
        let (_, status, main, time) = chart_rows(kline, layout);
        let info = status.unwrap();
        let tband = time.unwrap();
        let (_plot, axis) = frame_rects(main);
        assert!(info.min.y < main.min.y, "状态带和绘图区之间没留线格");
        // 绘图区那一段：轴列整段是灰底（含刻度数 / 刻度线），不能露出工作区白。
        for y in main.min.y as usize..main.max.y as usize {
            for x in axis.min.x as usize..axis.max.x as usize {
                assert_ne!(
                    grid[y][x],
                    Some(C_WORK_BG),
                    "价格轴列 ({x}, {y}) 露出了白底：绘图区里的轴灰没铺满"
                );
            }
        }
        let probe_x = axis.min.x as usize + 4;
        // 状态带那一行（原来会拖灰尾巴的地方）必须是白的。
        for y in info.min.y as usize + 1..main.min.y as usize - 1 {
            assert_eq!(grid[y][probe_x], Some(C_WORK_BG), "状态带右侧露出了灰底");
        }
        // 时间带那一行也一样：整个底色都是白的，右边没有灰尾巴。
        // 时间带最后一行就是本块底边那条网格线（和下面的拖拽条共用），排除掉。
        for y in tband.min.y as usize + 1..tband.max.y as usize - 1 {
            assert_eq!(
                grid[y][probe_x],
                Some(C_WORK_BG),
                "时间带右侧露出了灰底（不该有灰色区域） y={y}"
            );
        }
    }
    /// 内置指标都必须能报出自己的源码（source() 用 include_str! 在编译期把整份文件嵌进来），
    /// 顺序也必须是蜡烛 / 成交量 / MACD - 就是三个 Pane 槽位的下标。
    #[test]
    fn builtin_indicators_carry_their_own_source() {
        let builtins = indicator::builtin();
        assert_eq!(
            builtins.len(),
            6,
            "内置指标应该是蜡烛 / 美国线 / 成交量 / MACD / KDJ / RSI 六个"
        );
        // 每个指标的源码就是它自己那个文件（include_str!），认一下各自的类型名。
        let want = [
            "struct Candle",
            "struct Bars",
            "struct Volume",
            "struct Macd",
            "struct Kdj",
            "struct Rsi",
        ];
        for (i, ind) in builtins.iter().enumerate() {
            let src = ind.source();
            assert!(!src.is_empty(), "{} 没报出自己的源码", ind.name());
            assert!(
                src.contains(want[i]),
                "{} 的 source() 不是它自己那个文件",
                ind.name()
            );
            // path() 由 file!() 给出：相对仓库根、指向真的源文件（保存要写回它）。
            // file!() 在 Windows 上给的是反斜杠，比对前先归一成斜杠。
            let path = ind.path();
            let norm = path.replace('\\', "/");
            assert!(
                norm.starts_with("src/indicator/") && norm.ends_with(".rs"),
                "{} 的 path() 不是源文件路径：{path}",
                ind.name()
            );
            assert!(
                std::path::Path::new(path).is_file(),
                "{} 的 path() 指向的文件不存在：{path}",
                ind.name()
            );
        }
        assert_eq!((PANE_CANDLE, PANE_VOLUME, PANE_MACD), (0, 1, 2), "三个面板槽位对不上");
    }

    /// 纵轴范围只扫可见区间里的数据指令（蜡烛 / 柱 / 折线 / 直线），横线和文字不算数
    /// - 横线是参考线，不能让它把值域撑开。
    #[test]
    fn pane_range_scans_only_visible_data_cmds() {
        use crate::indicator::DrawCmd;
        let cmds = vec![
            DrawCmd::Candle { i: 0, o: 1.0, h: 3.0, l: 0.5, c: 2.0, color: 0x000000 },
            DrawCmd::Candle { i: 1, o: 2.0, h: 9.0, l: 1.0, c: 3.0, color: 0x000000 },
            DrawCmd::Bar { i: 1, v0: -50.0, v1: 100.0, color: 0x000000, hollow: false },
            DrawCmd::HLine { p: 999.0, color: 0x000000, w: 1 },
            DrawCmd::Text { i: 1, p: -999.0, s: "x".to_owned(), color: 0x000000 },
        ];
        // 只看下标 0：只有第一根蜡烛可见。
        assert_eq!(pane_range(&cmds, 0, 1), (0.5, 3.0), "只该扫到第一根蜡烛的高低");
        // 看下标 0..=1：加上第二根蜡烛和那根柱子（横线和文字都不算）。
        assert_eq!(pane_range(&cmds, 0, 2), (-50.0, 100.0), "柱子的极值和蜡烛的极值没一起算进来");
        // 可见区间里一条数据指令都没有 -> (0, 0)。
        assert_eq!(pane_range(&cmds, 5, 9), (0.0, 0.0), "空区间该返回 (0, 0)");
    }

    /// compute 只在面板脏的时候跑：干净时连缓存那块内存都不换（缩放 / 平移 / 每帧重绘都不该触发
    /// 重算），数据换代和换涨跌色都会置脏、下一帧重算。
    #[test]
    fn pane_compute_runs_only_when_dirty() {
        let mut app = blank_app();
        app.bars = data::random_walk::generate(BAR_SEED, 8, BAR_START);
        let pane = PANE_CANDLE;

        // 首帧：面板刚建出来是脏的 -> 算一次，缓存有内容、脏标志清掉。
        assert!(app.panes[pane].dirty, "新面板应该一开始就是脏的");
        app.ensure_pane_computed(pane);
        assert!(!app.panes[pane].cmds.is_empty(), "首帧应该算出了指令");
        assert!(!app.panes[pane].dirty, "算完应该清脏");

        // 干净时再调：compute 不该被调用（缓存那块内存原地不动）。
        let ptr = app.panes[pane].cmds.as_ptr();
        let len = app.panes[pane].cmds.len();
        app.ensure_pane_computed(pane);
        assert_eq!(app.panes[pane].cmds.as_ptr(), ptr, "干净时不该重算");
        assert_eq!(app.panes[pane].cmds.len(), len);

        // 数据换代置脏 -> 所有面板都要重算。
        app.mark_panes_dirty();
        assert!(app.panes.iter().all(|p| p.dirty), "数据换代应该把所有面板置脏");
        app.ensure_pane_computed(pane);
        assert!(!app.panes[pane].dirty);

        // 换涨跌色也置脏；重算之后蜡烛用的就是新的那对颜色。
        app.up_color = (0x11, 0x22, 0x33);
        app.down_color = (0x44, 0x55, 0x66);
        app.sync_pane_palette();
        assert!(app.panes[pane].dirty, "换色应该置脏");
        app.ensure_pane_computed(pane);
        let (up, down) = (
            indicator::rgb_u32(app.up_color),
            indicator::rgb_u32(app.down_color),
        );
        let mut candles = 0;
        for cmd in &app.panes[pane].cmds {
            if let crate::indicator::DrawCmd::Candle { color, .. } = cmd {
                assert!(*color == up || *color == down, "蜡烛没换成新的涨跌色");
                candles += 1;
            }
        }
        assert_eq!(candles, app.bars.len(), "每根 K 线都该有一根蜡烛指令");
    }

    /// 图表内部竖直分段的几何：标题栏 / 状态带 / 绘图区 / 时间带都是「跨若干格」的段，
    /// 相邻两段共用同一条 1px 线格（谁也不多吃谁的像素），各段边界都落在网格线上；
    /// 最后一段铺到面板底边（不吸零头、不留白条）。要哪几条带由指标自己声明，
    /// 同一份指标挂到哪一块都一样。
    #[test]
    fn chart_rows_share_lines_and_snap_to_the_grid() {
        let work = left_work();
        let [kline, _bar, volume, _bar2, indicator] =
            StockApp::left_rows(work, KLINE_H_DEFAULT, VOL_H_DEFAULT);
        // 某条边相对本块左上角是不是落在 9px 网格线上。
        let on_grid = |y: f32, area: egui::Rect| (y - area.min.y) % 9.0 == 0.0;
        // 蜡烛图那种「状态带 + 时间带」的切法（各 2 格 = 一个文本行盒）。
        let (title, status, main, time) =
            chart_rows(kline, crate::indicator::ChartLayout::new(2, 2));
        let (status, time) = (status.unwrap(), time.unwrap());
        assert_eq!(title.height(), cells(PANE_TITLE_ROWS), "标题栏不是 2 格");
        assert_eq!(title.max.y - LINE_W, status.min.y, "标题栏和状态带没共用那条线");
        assert_eq!(status.height(), cells(2), "状态带不是 2 格");
        assert_eq!(status.max.y, main.min.y, "状态带和绘图区没共用那条线");
        assert_eq!(main.max.y, time.min.y, "绘图区和时间带没共用那条线");
        assert_eq!(time.height(), cells(2), "时间带不是 2 格");
        // 各段边界都落在网格线上（绘图区 / 时间带的上沿再让开那条 1px 线格）。
        assert!(on_grid(title.min.y, kline), "标题栏上沿没落在网格线上");
        assert!(on_grid(status.min.y, kline), "状态带上沿没落在网格线上");
        assert!(on_grid(main.min.y - LINE_W, kline), "绘图区上沿没落在网格线上");
        assert!(on_grid(time.min.y, kline), "时间带上沿没落在网格线上");
        assert_eq!(time.max.y, kline.max.y, "时间带没铺到面板底边（底下不该留白条）");
        println!(
            "图表1 {}px = 标题栏 cells(2) + 状态带 cells(2) + 绘图区 {} + 时间带 cells(2)",
            kline.height(),
            main.height(),
        );

        // 只要标题栏 + 绘图区的图表（成交量这种没声明任何带的）。
        let (vtitle, vstatus, vmain, vtime) = chart_rows(volume, crate::indicator::ChartLayout::NONE);
        assert!(vstatus.is_none() && vtime.is_none());
        assert_eq!(vtitle.max.y, vmain.min.y, "标题栏和绘图区没共用那条线");
        assert_eq!(vmain.max.y, volume.max.y, "绘图区没铺到面板底边");
        println!(
            "图表2 {}px = 标题栏 cells(2) + 绘图区 {}",
            volume.height(),
            vmain.height(),
        );
        // 只要「状态带」的图表（MACD）：标题栏 + 状态带 + 绘图区。
        let (_, mstatus, mmain, mtime) =
            chart_rows(indicator, chart_rows_layout_with_status());
        assert!(mstatus.is_some() && mtime.is_none());
        assert_eq!(mmain.max.y, indicator.max.y, "绘图区没铺到面板底边");
        println!(
            "图表3 {}px = 标题栏 + 状态带 + 绘图区 {}px",
            indicator.height(),
            mmain.height()
        );
    }

    /// 只要一条 2 格状态带的 layout（MACD 那种）。
    fn chart_rows_layout_with_status() -> crate::indicator::ChartLayout {
        crate::indicator::ChartLayout::with_status(2)
    }

    /// Pane 标题栏两个按钮（切换 / 代码）的宽：按铁律「文字左右各留 1 格空档」量
    ///（文字宽 + 左右各 1 格，向上吸到整数格）。测试里不用真字形引擎，pure 算宽就够了。
    fn pane_btn_widths() -> Vec<f32> {
        let w = |s: &str| snap_cells_up(crate::font::text_width(s) as f32 + 2.0 * TEXT_PAD_X);
        vec![w("切换"), w("代码")]
    }

    /// Pane 标题栏：左右两个按钮各按文字量宽（2 个汉字 = 4 个内容格，加左右各 1 格空档 = 6 格）、
    /// 相邻两个共用中间那条竖线，每一条边框都落在网格线上（竖边框落在竖网格线上、
    /// 上边框压在 Pane 顶边、下边框压在标题栏下沿）。
    #[test]
    fn pane_titlebar_holds_two_buttons_on_the_grid() {
        let work = left_work();
        let [kline, _bar, _vol, _bar2, _ind] = StockApp::left_rows(work, KLINE_H_DEFAULT, VOL_H_DEFAULT);
        let (title, _info, _main, _time) = chart_rows(kline, crate::indicator::ChartLayout::new(2, 2));
        let buttons = StockApp::pane_button_rects(title, &pane_btn_widths());
        // 铁律：按钮宽 = cells(文字内容格数 + 2) - 文字左右各留 1 格空档
        //（「切换」「代码」各 2 个汉字 = 4 个内容格）。
        assert_eq!(pane_btn_widths(), vec![cells(6), cells(6)]);
        assert_eq!(buttons[0].width(), cells(6));
        assert_eq!(buttons[0].height(), PANE_BTN_H);
        assert_eq!(buttons[1].min.x, buttons[0].max.x - LINE_W, "两个按钮没共用中间那条线");
        assert_eq!(buttons[1].max.x, kline.max.x, "最右边的按钮没贴着 Pane 右边框");
        for b in &buttons {
            for x in [b.min.x, b.max.x - LINE_W] {
                assert_eq!((x - kline.min.x) % 9.0, 0.0, "按钮竖边框 {x} 不在网格线上");
            }
            assert_eq!(b.min.y, kline.min.y, "按钮上边框没压在 Pane 顶边上");
            assert_eq!(b.max.y, title.max.y, "按钮下边框没压在标题栏下沿");
        }
        println!(
            "标题栏 {}px，两个按钮各 {}x{}",
            title.height(),
            buttons[0].width(),
            PANE_BTN_H
        );
    }

    /// 指标切换下拉面板：列的就是 builtin() 的名字（顺序一致）；点某一项就把那个指标换到 Pane 上
    /// 并置脏（下一帧重算）；面板右边缘和「切换」按钮对齐、上沿压在标题栏下沿那条线上。
    #[test]
    fn pane_menu_lists_builtins_and_swaps_the_indicator() {
        let names = indicator::builtin_names();
        assert_eq!(
            names.join(","),
            "蜡烛图,美国线,成交量,MACD(12,26,9),KDJ(9,3,3),RSI(6,12,24)",
            "内置指标的名字 / 顺序变了"
        );
        let mut app = blank_app();
        let work = left_work();
        let [kline, _bar, _vol, _bar2, _ind] = StockApp::left_rows(work, KLINE_H_DEFAULT, VOL_H_DEFAULT);
        let (title, _info, _main, _time) = chart_rows(kline, crate::indicator::ChartLayout::new(2, 2));
        let buttons = StockApp::pane_button_rects(title, &pane_btn_widths());
        app.pane_menu = Some(PANE_CANDLE);
        app.pane_menu_anchor = buttons[0];
        let panel = app.pane_menu_rect(names.len(), WINDOW_H).unwrap();
        assert_eq!(panel.max.x, buttons[0].max.x, "面板右边缘没和「切换」按钮对齐");
        assert_eq!(panel.min.y, buttons[0].max.y - LINE_W, "面板上沿没压在标题栏下沿那条线上");
        assert_eq!(panel.height(), cells(PANE_MENU_ROW_ROWS * names.len() as i32));
        assert_eq!((panel.max.x - PANE_MENU_W - kline.min.x) % 9.0, 0.0, "面板左边不在网格线上");

        // 换指标：槽位 0 从蜡烛换成成交量 - 名字变了、缓存置脏、重算出来的是量柱指令。
        assert_eq!(app.panes[PANE_CANDLE].indicator.name(), "蜡烛图");
        app.set_pane_indicator(PANE_CANDLE, 2);
        assert_eq!(app.panes[PANE_CANDLE].indicator.name(), "成交量");
        assert!(app.panes[PANE_CANDLE].dirty, "换指标之后没置脏");
        app.bars = data::random_walk::generate(BAR_SEED, 8, BAR_START);
        app.ensure_pane_computed(PANE_CANDLE);
        assert!(
            app.panes[PANE_CANDLE]
                .cmds
                .iter()
                .all(|c| matches!(c, crate::indicator::DrawCmd::Bar { .. })),
            "换过指标之后算出来的不是成交量柱"
        );
    }

    /// 下拉菜单的上下沿：默认压在按钮下沿那条网格线上；最下面那块图表往下开会越过窗口下沿，
    /// 于是翻上去压在按钮上沿那条线上 - 两种摆法的上下沿都必须落在网格线上、面板不出窗口。
    #[test]
    fn pane_menu_flips_up_when_it_would_overflow_the_window() {
        let mut app = blank_app();
        let work = left_work();
        let [kline, _b0, _vol, _b1, ind] = StockApp::left_rows(work, KLINE_H_DEFAULT, VOL_H_DEFAULT);
        let names = indicator::builtin_names();
        let h = cells(PANE_MENU_ROW_ROWS * names.len() as i32);
        let w = |s: &str| snap_cells_up(crate::font::text_width(s) as f32 + 2.0 * TEXT_PAD_X);
        let widths = [w("切换"), w("代码")];

        // 上面那块、窗口够高：往下开（上沿压在标题栏下沿那条线上）。
        let (title, _, _, _) = chart_rows(kline, crate::indicator::ChartLayout::new(2, 2));
        app.pane_menu_anchor = StockApp::pane_button_rects(title, &widths)[0];
        let panel = app.pane_menu_rect(names.len(), WINDOW_H).unwrap();
        assert_eq!(panel.min.y, app.pane_menu_anchor.max.y - LINE_W, "没压在标题栏下沿那条线上");
        assert_eq!((panel.min.y) % PITCH, 0.0, "面板上沿不在网格线上");
        assert_eq!((panel.max.y - LINE_W) % PITCH, 0.0, "面板下沿不在网格线上");
        assert_eq!(panel.height(), h);

        // 底下放不下（窗口变矮 / 拖完拖拽条之后最下面那块很薄）：翻上去，
        // 下沿压在标题栏上沿那条线上，面板自己的上下沿照样落在网格线上。
        let (title, _, _, _) = chart_rows(ind, crate::indicator::ChartLayout::new(2, 2));
        app.pane_menu_anchor = StockApp::pane_button_rects(title, &widths)[0];
        let anchor = app.pane_menu_anchor;
        let panel = app.pane_menu_rect(names.len(), anchor.max.y).unwrap();
        assert_eq!(panel.max.y - LINE_W, anchor.min.y, "翻上去之后没压在标题栏上沿那条线上");
        assert_eq!(panel.height(), h);
        assert_eq!((panel.min.y) % PITCH, 0.0, "翻上去之后面板上沿不在网格线上");
        assert_eq!((panel.max.y - LINE_W) % PITCH, 0.0, "翻上去之后面板下沿不在网格线上");
        assert!(panel.max.y <= anchor.max.y, "翻上去之后盖住了按钮那一行");
    }

    /// K 线图（蜡烛图 / 美国线）标题栏多一个「叠加」按钮，排在「切换」左边；三个按钮宽都按文字量、
    /// 相邻两个共用中间那条线、竖边框全落在网格线上。不认叠加的图表有没有这个按钮由 supports_overlay 定。
    #[test]
    fn kline_pane_title_holds_three_buttons_on_the_grid() {
        let work = left_work();
        let [kline, _b0, _vol, _b1, _ind] = StockApp::left_rows(work, KLINE_H_DEFAULT, VOL_H_DEFAULT);
        let (title, _, _, _) = chart_rows(kline, crate::indicator::ChartLayout::new(2, 2));
        let w = |s: &str| snap_cells_up(crate::font::text_width(s) as f32 + 2.0 * TEXT_PAD_X);
        let widths = vec![w("叠加"), w("切换"), w("代码")];
        assert_eq!(widths, vec![cells(6), cells(6), cells(6)], "三个按钮各按 2 个汉字 + 左右 1 格量");
        let buttons = StockApp::pane_button_rects(title, &widths);
        assert_eq!(buttons.len(), 3);
        assert_eq!(buttons[2].max.x, kline.max.x, "最右边的「代码」没贴着 Pane 右边框");
        assert_eq!(buttons[1].min.x, buttons[0].max.x - LINE_W, "「切换」和「叠加」没共用中间那条线");
        assert_eq!(buttons[2].min.x, buttons[1].max.x - LINE_W, "「代码」和「切换」没共用中间那条线");
        for (i, b) in buttons.iter().enumerate() {
            for x in [b.min.x, b.max.x - LINE_W] {
                assert_eq!((x - kline.min.x) % 9.0, 0.0, "按钮 {i} 的竖边框不在网格线上");
            }
            assert_eq!(b.height(), PANE_BTN_H);
        }
        // 认不认叠加：只有两种 K 线图认（顺序和 builtin() 一致）。
        let list = indicator::builtin();
        let supports: Vec<bool> = list.iter().map(|ind| ind.supports_overlay()).collect();
        assert_eq!(
            supports,
            vec![true, true, false, false, false, false],
            "只有 K 线图（蜡烛图 / 美国线）才该出现「叠加」按钮"
        );
    }

    /// 「叠加」菜单：12 行 = MA5/10/20/60/120/250 + EMA5/10/20/60/120/250；面板右边缘和「叠加」
    /// 按钮对齐、上沿压在标题栏下沿那条线上；最长那行（EMA250 = 6 个 ASCII 格）左右各留得下 1 格；
    /// 改配置要灌给 K 线图并置脏，不认叠加的图表不掺和。
    #[test]
    fn overlay_menu_toggles_ma_and_ema_on_the_grid() {
        let items = StockApp::overlay_items();
        let labels: Vec<String> = items.iter().map(|it| it.label.clone()).collect();
        assert_eq!(
            labels.join(","),
            "MA5,MA10,MA20,MA60,MA120,MA250,EMA5,EMA10,EMA20,EMA60,EMA120,EMA250",
            "叠加菜单的行 / 顺序变了"
        );
        let mut app = blank_app();
        let work = left_work();
        let [kline, _b0, _vol, _b1, _ind] = StockApp::left_rows(work, KLINE_H_DEFAULT, VOL_H_DEFAULT);
        let (title, _, _, _) = chart_rows(kline, crate::indicator::ChartLayout::new(2, 2));
        let w = |s: &str| snap_cells_up(crate::font::text_width(s) as f32 + 2.0 * TEXT_PAD_X);
        let buttons = StockApp::pane_button_rects(title, &[w("叠加"), w("切换"), w("代码")]);
        app.overlay_menu = Some(PANE_CANDLE);
        app.overlay_menu_anchor = buttons[0];
        let panel = app.overlay_menu_rect(items.len(), WINDOW_H).unwrap();
        assert_eq!(panel.max.x, buttons[0].max.x, "面板右边缘没和「叠加」按钮对齐");
        assert_eq!(panel.min.y, buttons[0].max.y - LINE_W, "面板上沿没压在标题栏下沿那条线上");
        assert_eq!(panel.height(), cells(PANE_MENU_ROW_ROWS * items.len() as i32));
        assert_eq!((panel.max.x - OVERLAY_MENU_W - kline.min.x) % 9.0, 0.0, "面板左边不在网格线上");

        // 每一行：左边 1 格是色块标记、右边是名字 - 最长那行（EMA250）左右都还留得下 1 格空档。
        let text_x = text_left_x(panel.min.x) + PITCH;
        assert!(text_x - (panel.min.x + LINE_W) >= GRID, "颜色标记那 1 格放不下");
        let longest = crate::font::text_width("EMA250") as f32;
        assert!(
            panel.max.x - LINE_W - (text_x + longest) >= GRID,
            "最长的名字右边留不出 1 格空档（面板 {OVERLAY_MENU_W} 太窄）"
        );

        // 翻一位（EMA20）：灌给蜡烛图、置脏；MA 那几位和别的图表都不动。
        let before = app.panes[PANE_CANDLE].indicator.overlay_state();
        app.overlay = before;
        app.overlay.toggle_ema(2);
        app.sync_overlay();
        let after = app.panes[PANE_CANDLE].indicator.overlay_state();
        assert_eq!(after.ema, before.ema ^ (1 << 2), "EMA20 没被翻上");
        assert_eq!(after.ma, before.ma, "MA 那几位不该动");
        assert!(app.panes[PANE_CANDLE].dirty, "改完叠加没置脏");
        assert!(!app.panes[PANE_VOLUME].indicator.supports_overlay(), "成交量不该认叠加");
    }

    /// 源码窗：源码就是指标自己那个文件（include_str! 整份嵌进来，所以连 include_str! 那一行
    /// 本身也在里面）、能看见的行数、总行数、滚动上限；画一帧之后面板四条边框 / 标题栏底色 /
    /// 行号区那条竖线都落在窗口的网格线上。
    #[test]
    fn source_window_shows_the_whole_own_file() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        app.open_source(PANE_CANDLE);
        assert!(app.source.win.open && app.source.pane == PANE_CANDLE);
        let visible = SRC_VISIBLE_LINES;
        let total = app.source.editor.buf.lines.len();
        assert_eq!(visible, 27, "正文区一屏应该放得下 27 行（54 格 / 每行 2 格）");
        assert!(total > visible, "源码应该比一屏长，不然不用滚动");
        assert_eq!(app.source.editor.scroll, 0, "刚打开应该从第 1 行开始");
        assert_eq!(app.source.editor.max_scroll(), total - visible);

        let joined = app.source.editor.buf.lines.join("\n");
        assert!(joined.contains("include_str!(\"candle.rs\")"), "源码里没有 include_str! 那一行");
        assert!(joined.contains("impl Indicator for Candle"), "源码不是 candle.rs");
        assert!(joined.contains("蜡烛"), "中文注释没进来");
        assert!(!joined.contains('\t'), "Tab 没换成 4 个空格");
        assert!(!joined.contains('\r'), "行尾的换行符没清掉");
        println!("源码窗：一屏 {visible} 行 / 共 {total} 行（candle.rs）");
        for name in ["volume.rs", "macd.rs"] {
            let src = crate::indicator::builtin()
                .into_iter()
                .find(|ind| ind.source().contains(name))
                .expect("内置指标里没有这个文件");
            assert!(src.source().contains(&format!("include_str!(\"{name}\")")));
        }

        // 把窗画一帧（和运行时同一套光栅化开关）：越界的 scroll 会被夹回上限，
        // 面板四条边框 / 标题栏底色 / 行号区那条竖线都必须落在窗口的网格线上。
        app.source.editor.scroll = 9999;
        let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        let mut out = ctx.run_ui(raw, |ui| {
            app.ui_source_window(ui, layout);
        });
        let shapes = std::mem::take(&mut out.shapes);
        absorb_textures(&mut out.textures_delta);
        let grid = rasterize_into(
            &ctx.tessellate(shapes, 1.0),
            WINDOW_W as usize,
            WINDOW_H as usize,
        );
        assert_eq!(
            app.source.editor.scroll,
            total - visible,
            "滚动位置没夹回上限"
        );
        let (x0, y0) = (SRC_DEFAULT_X as usize, SRC_DEFAULT_Y as usize);
        let (px1, py1) = (x0 + SRC_W as usize - 1, y0 + SRC_H as usize - 1);
        assert_eq!(grid[y0][x0 + 300], Some(C_CTRL_BORDER), "面板上边框不对");
        assert_eq!(grid[py1][x0 + 300], Some(C_CTRL_BORDER), "面板下边框不对");
        assert_eq!(grid[y0 + 200][x0], Some(C_CTRL_BORDER), "面板左边框不对");
        assert_eq!(grid[y0 + 200][px1], Some(C_CTRL_BORDER), "面板右边框不对");
        assert_eq!(grid[y0 + 4][x0 + 300], Some(C_TITLE_BG), "标题栏底色不对");
        let gutter_line = x0 + grid_line(SRC_GUTTER_COLS) as usize;
        assert_eq!(grid[y0 + 200][gutter_line], Some(C_CTRL_BORDER), "行号区右边那条线不对");
        assert_eq!(grid[y0 + 200][gutter_line - 2], Some(C_PANEL_BG), "行号区底色不对");
        // 标题栏下沿那条网格线是标题栏和代码区共用的：线格上一条完整的硬边，线格之后 1px
        // 才是代码区第一行的内容（所以代码区的底色从那条线往下 1px 开始）。
        assert_eq!(
            grid[y0 + SRC_BODY_TOP as usize][x0 + 300],
            Some(C_CTRL_BORDER),
            "标题栏下沿那条线不对"
        );
        // 正文区一屏 27 行占 486px、可用 495px：最后一行的下一行只有底色，一个墨点都没有。
        let empty_y = y0 + SRC_BODY_TOP as usize + SRC_VISIBLE_LINES * SRC_ROW_PITCH as usize + 1;
        assert_eq!(grid[empty_y][x0 + 300], Some(C_WORK_BG), "代码区底色不对");
    }

    /// 源码窗的位置与滚动：位置每帧吸到窗口的 9px 网格线上（拖动中也是一格格地动，松手不需要
    /// 再归位 - 和颜色面板同一套 FloatWindow 外壳）；滚动按整数行、夹在 [0, 总行数 - 一屏行数]。
    #[test]
    fn source_window_position_and_scroll_follow_the_grid() {
        assert_eq!(SRC_W as i32, 721, "面板宽 = cells(80)");
        assert_eq!(SRC_H as i32, 550, "面板高 = cells(61)");
        assert_eq!(SRC_VISIBLE_LINES, 27, "一屏 27 行 = 54 格 / 每行 2 格");
        let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        let mut sv = SourceView::new();
        sv.editor.buf.lines = (0..100).map(|i| format!("{i}")).collect();
        assert_eq!(sv.editor.max_scroll(), 100 - 27);

        // 开局位置就落在网格线上（198 = 22*9、99 = 11*9）。
        assert_eq!(sv.win.pos, egui::pos2(SRC_DEFAULT_X, SRC_DEFAULT_Y));

        // 拖动中每帧都吸格：203.4 / 97.6 吸到最近的网格线（23*9 = 207、11*9 = 99）。
        sv.win.pos = egui::pos2(203.4, 97.6);
        sv.win.snap_pos(layout);
        assert_eq!(sv.win.pos, egui::pos2(23.0 * PITCH, 11.0 * PITCH));

        // 拖出窗口就整块夹回来：左边 / 上边贴 0，右边 / 下边贴窗口上离它最近的网格线。
        sv.win.pos = egui::pos2(-40.0, WINDOW_H + 40.0);
        sv.win.snap_pos(layout);
        assert_eq!(sv.win.pos, egui::pos2(0.0, WINDOW_H - SRC_H));
        assert_eq!((WINDOW_H - SRC_H) % PITCH, 0.0, "夹回来的下边界也得在网格线上");
    }

    /// 源码窗里的文字也要守铁律：行号右对齐到行号区那条线上、代码从它的右边 1 格起排，
    /// 两边都留得下 1 格空档 - 最长的那一行截断之后也不会顶到滚动条那条线上。
    #[test]
    fn source_window_text_keeps_a_cell_on_both_sides() {
        let gutter_line = grid_line(SRC_GUTTER_COLS);
        let bar_line = grid_line(SRC_COLS - 1);

        // 行号：3 位数（最长的行号）右对齐之后，左边离面板边框还有 1 格以上。
        let num_w = text_width("999") as f32;
        let num_left = text_right_x(gutter_line) - num_w;
        assert_eq!(((num_left - LINE_W) as i32) % PITCH as i32, 0, "行号左边没落在内容格上");
        assert!(num_left - LINE_W >= GRID, "行号左边只剩不到 1 格空档");

        // 代码：起点在行号区右边那条线往右 1 格，可用宽度是整数格，右边还留 1 格。
        let code_x = text_left_x(gutter_line);
        assert_eq!(((code_x - LINE_W) as i32) % PITCH as i32, 0, "代码起点没落在内容格上");
        assert_eq!((code_x - gutter_line as f32) as i32, (LINE_W + PITCH) as i32);
        let code_w = bar_line - GRID - code_x;
        assert_eq!((code_w as i32) % PITCH as i32, 0, "代码区宽度不是整数格");
        // 满宽那一行（宽度 = code_w）的右边缘离滚动条那条线还剩 1 格。
        assert_eq!(bar_line as i32 - (code_x + code_w) as i32, GRID as i32);
    }
    /// 同一份指标挂到任意一块图表上，画出来必须一模一样 - 界面里没有「主图 / 副图」之分，
    /// 一切都由指标自己声明（要哪几条带、画什么线条）。
    #[test]
    fn the_same_indicator_renders_the_same_in_any_chart() {
        let render = |app: &mut StockApp, pane: usize, area: egui::Rect| {
            let ctx = egui::Context::default();
            configure_tessellation(&ctx);
            ctx.set_pixels_per_point(1.0);
            let mut raw = egui::RawInput::default();
            raw.screen_rect = Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(area.max.x, area.max.y),
            ));
            let mut out = ctx.run_ui(raw, |ui| {
                app.ui_chart(ui, pane, area);
            });
            let shapes = std::mem::take(&mut out.shapes);
            absorb_textures(&mut out.textures_delta);
            rasterize_into(
                &ctx.tessellate(shapes, 1.0),
                area.max.x as usize,
                area.max.y as usize,
            )
        };

        let mut app = blank_app();
        app.bars = data::random_walk::generate(BAR_SEED, 64, BAR_START);
        // 图表 1 本来就是蜡烛，把图表 3 也换成一模一样的蜡烛。
        app.panes[PANE_MACD].indicator = Box::new(crate::indicator::candle::Candle::default());
        app.panes[PANE_MACD].dirty = true;
        let area = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(CHART_PANE_W, 300.0));
        let a = render(&mut app, PANE_CANDLE, area);
        let b = render(&mut app, PANE_MACD, area);
        assert_eq!(a, b, "同一个指标挂到两块图表上，画出来的像素不一样");
    }

    /// 十字光标：竖线在三块图表里都画（共用 x 映射，天然连成一条）；横线只画鼠标压着的那一块。
    #[test]
    fn crosshair_spans_every_chart() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.bars = data::random_walk::generate(BAR_SEED, 64, BAR_START);

        // 三块的几何（和 ui() 里那套一样）。
        let work = left_work();
        app.clamp_left_heights(work);
        let [kline, _bar1, volume, _bar2, indicator] =
            StockApp::left_rows(work, app.kline_h, app.vol_h);
        let areas = [kline, volume, indicator];
        let mut plots = [egui::Rect::NOTHING; 3];
        for (i, area) in areas.iter().enumerate() {
            let layout = app.panes[i].indicator.layout();
            let (_, _, main, _) = chart_rows(*area, layout);
            plots[i] = frame_rects(main).0;
            app.panes[i].height = main.height() as i32;
        }
        app.clamp_view(plots[0].width());
        for i in 0..app.panes.len() {
            app.ensure_pane_computed(i);
        }

        // 鼠标压在第 2 块（成交量）的绘图区里。
        let target = plots[1].center();
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(WINDOW_W, WINDOW_H),
        ));
        raw.events = vec![egui::Event::PointerMoved(target)];
        let mut out = ctx.run_ui(raw, |ui| {
            app.update_crosshair(ui, &plots);
        });
        absorb_textures(&mut out.textures_delta);
        let cross = app.crosshair.expect("鼠标在图上应该有十字光标");
        assert_eq!(cross.pane, PANE_VOLUME, "横线该落在鼠标压着的那一块");

        // 竖线那一列在三块图表里都得有（共用 x 映射）。
        let from = app.offset;
        let advance = bar_advance(app.scale);
        let x = px_center(plots[0].min.x + (cross.fpos - from as f64) as f32 * advance) as usize;
        let (grid, [k, _b, v, _b2, m]) = render_left(&mut app);
        for (i, area) in [(PANE_CANDLE, k), (PANE_VOLUME, v), (PANE_MACD, m)] {
            let layout = app.panes[i].indicator.layout();
            let (_, _, main, _) = chart_rows(area, layout);
            let plot = frame_rects(main).0;
            let found = (plot.min.y as usize..plot.max.y as usize).any(|y| grid[y][x] == Some(C_CROSS));
            assert!(found, "第 {} 块图表里没画十字光标的竖线", i + 1);
        }
        // 横线（整行铺满绘图区）只出现在悬停的那一块里。
        let width = |area: egui::Rect, pane: usize| -> (usize, usize, usize) {
            let layout = app.panes[pane].indicator.layout();
            let (_, _, main, _) = chart_rows(area, layout);
            let plot = frame_rects(main).0;
            let mut widest = 0;
            for y in plot.min.y as usize..plot.max.y as usize {
                let n = (plot.min.x as usize..plot.max.x as usize)
                    .filter(|&xx| grid[y][xx] == Some(C_CROSS))
                    .count();
                widest = widest.max(n);
            }
            (widest, plot.min.y as usize, plot.max.y as usize)
        };

        assert!(width(v, PANE_VOLUME).0 > 100, "悬停那一块没有横线");
        assert!(width(k, PANE_CANDLE).0 <= 1, "没悬停的块不该有横线");
        assert!(width(m, PANE_MACD).0 <= 1, "没悬停的块不该有横线");
    }

    /// 文字按网格对齐：一个字一个步进 - 汉字 2 个内容格（18px）、ASCII 1 个内容格（9px）。
    /// 于是任何一串文字的宽度都是 9 的整数倍，起点又落在内容格起点上，整串与网格对齐。
    #[test]
    fn text_advances_on_the_grid() {
        // 一个字的步进。
        for c in ['0', '9', 'A', 'z', '.', ',', '%', '+', '-', '/', ' '] {
            assert_eq!(char_advance(c), 9.0, "{c:?} 该是 ASCII（1 个内容格）");
        }
        for c in ['圆', '桌', '万', '亿', '：', '，', '（'] {
            assert_eq!(char_advance(c), 18.0, "{c:?} 该是中文（2 个内容格）");
        }
        // 2 个 ASCII = 2 格，2 个汉字 = 4 格。
        assert_eq!(text_advance("1234"), 36.0);
        assert_eq!(text_advance("圆桌"), 36.0);
        assert_eq!(text_advance("123") % 9.0, 0.0, "ASCII 的步进本身就是整格");
        // 排版宽度一律是 9 的整数倍（盒子 / 落点全靠这条）。
        assert_eq!(text_width(""), 0);
        assert_eq!(text_width("1234"), 36);
        assert_eq!(text_width("123"), 27);
        assert_eq!(text_width("12.5%"), 45);
        assert_eq!(text_width("圆桌"), 36);
        assert_eq!(text_width("圆桌 2024"), 81);
        for s in ["0", "1234.56", "圆桌", "-1200.5万", "MACD(12,26,9)", "2026-10-14"] {
            assert_eq!(text_width(s) % 9, 0, "{s} 的宽度不是 9 的整数倍");
        }
    }

    /// 字距模型的硬指标：一个 ASCII 字形的墨迹必须塞得进它那 9px 的字格（8x16 点阵字墨迹
    /// 只有 5~7px 宽），一个汉字的墨迹必须塞得进 18px，一串字符的总墨迹也不许超出
    /// text_advance() 的宽度。这条就是「字与字不许互相压住」的兜底。
    #[test]
    fn every_glyph_fits_its_advance() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        if app.text_engine.is_none() {
            return; // 没拿到 GDI 字体就跳过（正常 Windows 上不会走到这儿）
        }
        // 把一串字画在 x = 内容格起点上，返回墨迹的左右列号（没有墨迹就返回 None）。
        let probe = |app: &mut StockApp, s: &str, w: usize| -> Option<(usize, usize)> {
            let mut raw = egui::RawInput::default();
            raw.screen_rect =
                Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w as f32, 24.0)));
            let color = theme().text;
            let mut out = ctx.run_ui(raw, |ui| {
                let painter = ui.painter().clone();
                app.text(
                    &painter,
                    egui::pos2(TEXT_INSET_X, 12.0),
                    egui::Align2::LEFT_CENTER,
                    s,
                    color,
                );
            });
            let shapes = std::mem::take(&mut out.shapes);
            absorb_textures(&mut out.textures_delta);
            let grid = rasterize_into(&ctx.tessellate(shapes, 1.0), w, 24);
            let mut lo = usize::MAX;
            let mut hi = 0usize;
            for row in grid.iter().take(24) {
                for (x, cell) in row.iter().enumerate() {
                    if cell.is_some() {
                        lo = lo.min(x);
                        hi = hi.max(x);
                    }
                }
            }
            (lo != usize::MAX).then_some((lo, hi))
        };

        // ASCII：每个字的墨迹都得 <= ADV_ASCII（9px），等于说一个字符一格塞得下。
        for c in "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.,%+-*/()".chars() {
            let s = c.to_string();
            let (lo, hi) = probe(&mut app, &s, 64).unwrap_or_else(|| panic!("{c:?} 没画出墨迹"));
            let ink = (hi - lo + 1) as f32;
            assert!(
                ink <= ADV_ASCII,
                "{c:?} 的墨迹 {ink}px 比步进 {ADV_ASCII}px 还宽，字会互相压住"
            );
            // 落点也在自己那格附近：不会跑到整串宽度之外。
            assert!(
                (lo as f32) < TEXT_INSET_X + ADV_ASCII && hi as f32 >= TEXT_INSET_X,
                "{c:?} 的墨迹落在 {lo} 到 {hi}，没踩着内容格起点 {TEXT_INSET_X}"
            );
        }
        // 中文：一个字的墨迹 <= ADV_WIDE（18px）。
        for c in ['圆', '桌', '万', '亿', '：', '，'] {
            let s = c.to_string();
            let (lo, hi) = probe(&mut app, &s, 64).unwrap_or_else(|| panic!("{c:?} 没画出墨迹"));
            let ink = (hi - lo + 1) as f32;
            assert!(ink <= ADV_WIDE, "{c:?} 的墨迹 {ink}px 比步进 {ADV_WIDE}px 还宽");
        }
        // 整串：10 个数字 = 10 个内容格 = 90px，墨迹不许超出这个宽度。
        let s = "0123456789";
        let (lo, hi) = probe(&mut app, s, 128).expect("整串没画出墨迹");
        // 第一个字的 8px 字形贴在自己那个 9px 字格的左边，所以墨迹基本就从内容格起点起笔；
        // 但整串的墨迹必须整个落在 [起点, 起点 + text_advance) 这条窗里。
        assert!(
            lo as f32 >= TEXT_INSET_X && lo as f32 <= TEXT_INSET_X + 2.0,
            "整串没从内容格起点附近起笔：{lo}"
        );
        assert!(
            (hi as f32) < TEXT_INSET_X + text_advance(s),
            "整串墨迹排到 {hi}，超出了 text_advance() = {}px 的宽度",
            text_advance(s)
        );
    }
    /// 铁律：界面文字只允许「中文 + ASCII」。源码会被「源码窗」原样渲染出来，所以这条约束
    /// 落在源码本身：任何非 ASCII 字符都必须是 is_wide_char 认的中文（汉字 / 中文标点 / 全角
    /// 形式），别的符号（非 ASCII 的标点 / 箭头 / 数学符号）一个都不许有；控制字符只允许 \t \n \r。
    #[test]
    fn allow_only_chinese_and_ascii_in_sources() {
        fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(dir).expect("读不到源码目录") {
                let p = e.expect("目录项").path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push(p);
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        walk(&root, &mut files);
        assert!(files.len() >= 15, "源码文件数不对：{}", files.len());
        for p in files {
            let text = std::fs::read_to_string(&p).expect("源码读成 UTF-8 失败");
            for (i, c) in text.chars().enumerate() {
                let ascii_ok = c.is_ascii() && (c == '\t' || c == '\n' || c == '\r' || !c.is_control());
                assert!(
                    ascii_ok || is_wide_char(c),
                    "{} 第 {} 个字符 {c:?}（U+{:04X}）既不是中文也不是 ASCII",
                    p.display(),
                    i,
                    c as u32
                );
            }
        }
    }

    /// 皮肤：切换之后界面每帧读到的颜色整套换掉；暗色皮肤的关键色值符合要求。
    #[test]
    fn theme_switches_the_whole_palette() {
        set_theme(Theme::CLASSIC);
        assert_eq!(theme().id, ThemeId::Classic);
        assert_eq!(rgb32_u32(theme().ctrl_border), 0x999999);
        assert_eq!(rgb32_u32(theme().text), 0x000000);

        // 暗色：红色线框（#CC0000）/ 白字 / K 线涨色不变、跌色变青 / 数字跌色还是绿。
        set_theme(Theme::DARK);
        assert_eq!(theme().id, ThemeId::Dark);
        assert_eq!(rgb32_u32(theme().ctrl_border), 0xCC0000, "线框该是红色");
        assert_eq!(rgb32_u32(theme().text), 0xFFFFFF, "字该是白色");
        assert_eq!(rgb32_u32(theme().candle_up), 0xFF0000, "K 线涨色不变");
        assert_eq!(rgb32_u32(theme().candle_down), 0x00FFFF, "K 线跌色该变青");
        assert_eq!(rgb32_u32(theme().text_down), 0x009900, "数字跌色还是绿");
        assert_eq!(rgb32_u32(theme().text_up), 0xFF0000, "数字涨色还是红");

        set_theme(Theme::CLASSIC);
        assert_eq!(theme().id, ThemeId::Classic);
    }

    /// 「切换主题」按钮：点一下整套皮肤 + K 线涨跌色一起换，图表跟着重算；再点一下换回来。
    #[test]
    fn theme_button_switches_skin_and_candle_palette() {
        set_theme(Theme::CLASSIC);
        let mut app = blank_app();
        app.bars = data::random_walk::generate(BAR_SEED, 8, BAR_START);
        app.sync_pane_palette();
        assert_eq!(app.down_color, DOWN_RGB_DEFAULT, "默认跌色是绿");

        app.switch_theme();
        assert_eq!(theme().id, ThemeId::Dark);
        assert_eq!(app.up_color, (0xFF, 0x00, 0x00));
        assert_eq!(app.down_color, (0x00, 0xFF, 0xFF));
        assert!(app.panes.iter().all(|p| p.dirty), "换皮肤该让图表重算");

        app.switch_theme();
        assert_eq!(theme().id, ThemeId::Classic);
        assert_eq!(app.down_color, DOWN_RGB_DEFAULT);
        set_theme(Theme::CLASSIC);
    }
    /// 图表区的 K 线涨跌色默认「指向系统级配色」：切皮肤跟着变；
    /// 用户一旦在颜色面板里定过色（chart_follows_theme = false），切皮肤就不动它；
    /// 点「图表配色跟随系统」再把它拉回当前皮肤那一套。
    #[test]
    fn chart_colors_point_at_the_system_palette_until_customized() {
        set_theme(Theme::CLASSIC);
        let mut app = blank_app();
        assert!(app.chart_follows_theme, "默认该指向系统级配色");

        // 跟随状态：换皮肤，K 线涨跌色跟着换成暗色皮肤那一套（涨红不变、跌色变青）。
        app.switch_theme();
        assert_eq!(theme().id, ThemeId::Dark);
        assert_eq!(app.up_color, (0xFF, 0x00, 0x00));
        assert_eq!(app.down_color, (0x00, 0xFF, 0xFF));

        // 用户自定义之后就不再指向系统：再切皮肤，这两个颜色一个字节都不动。
        app.chart_follows_theme = false;
        app.up_color = (0x11, 0x22, 0x33);
        app.down_color = (0x44, 0x55, 0x66);
        app.switch_theme();
        assert_eq!(theme().id, ThemeId::Classic);
        assert_eq!(app.up_color, (0x11, 0x22, 0x33), "自定义过的颜色不该被切皮肤改掉");
        assert_eq!(app.down_color, (0x44, 0x55, 0x66), "自定义过的颜色不该被切皮肤改掉");

        // 重新指向系统级配色：回到当前皮肤那一套，并恢复「跟随」。
        app.follow_system_chart_colors();
        assert!(app.chart_follows_theme);
        let t = theme();
        assert_eq!(app.up_color, (t.candle_up.r(), t.candle_up.g(), t.candle_up.b()));
        assert_eq!(app.down_color, (t.candle_down.r(), t.candle_down.g(), t.candle_down.b()));
        assert!(app.panes.iter().all(|p| p.dirty), "配色变了该让图表重算");
        set_theme(Theme::CLASSIC);
    }

    /// 铁律「出现文字的地方，左右两边至少留 1 个网格单位（GRID = 8px）」：
    /// 凡是按文字量宽度的盒子（按钮 / 菜单项 / 状态栏项 / 浮动标签 / 滑块数值栏），
    /// 盒宽减去文字宽都必须 >= 2 格（中文 18px / ASCII 9px 的字距模型下，这里就是兜底的那道闸）。
    #[test]
    fn every_text_box_keeps_one_cell_of_padding() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        if app.text_engine.is_none() {
            return;
        }
        let min_pad = 2.0 * GRID;

        // 控制面板按钮：纯文字 = 文字宽；图标 + 文字 = 图标 + 间隔 + 文字宽。
        app.build_panel_buttons();
        assert!(app.panel_buttons.len() >= 10, "面板按钮不该只有这么几个");
        let buttons: Vec<(String, bool, f32)> = app
            .panel_buttons
            .iter()
            .map(|b| {
                (
                    b.button.content.label(),
                    matches!(&b.button.content, ButtonContent::IconText(..) | ButtonContent::Icon(..)),
                    b.button.w,
                )
            })
            .collect();
        for (text, has_icon, w) in buttons {
            let inner = app.text_width(&text) + if has_icon { ICON_W + ICON_GAP } else { 0.0 };
            assert!(
                w - inner >= min_pad,
                "按钮「{text}」左右空档不足：盒宽 {w}、内容宽 {inner}"
            );
        }

        // 菜单栏的每一项。
        for title in ["文件", "数据", "分析", "布局", "工具", "帮助"] {
            let w = tab_width(&mut app, title);
            assert!(
                w - app.text_width(title) >= min_pad,
                "菜单「{title}」左右空档不足"
            );
        }

        // 状态栏的每一项。
        for seg in app.status_segments() {
            let w = snap_cells_up(app.text_width(&seg) + 2.0 * TEXT_PAD_X);
            assert!(w - app.text_width(&seg) >= min_pad, "状态栏项「{seg}」空档不足");
        }

        // 颜色面板滑块右侧的数值栏：最长三位数（"255" / "360"）也得左右各留 1 格。
        let value_w = app.text_width("255");
        assert!(
            cells(CP_SLIDER_VALUE_COLS) - value_w >= min_pad,
            "滑块数值栏放不下三位数：栏宽 {}、数值宽 {value_w}",
            cells(CP_SLIDER_VALUE_COLS)
        );

        // Pane 标题栏那几个自绘按钮（叠加 / 切换 / 代码）：和面板按钮走同一个 button_width，
        // 宽 = cells(文字内容格数 + 2) - 文字左右各 1 格空档（用户点过名的这几项）。
        for text in ["叠加", "切换", "代码"] {
            let content = ButtonContent::Text(text.to_owned());
            let w = app.button_width(&content);
            assert!(
                w - app.text_width(text) >= min_pad,
                "Pane 标题栏按钮「{text}」左右空档不足：盒宽 {w}"
            );
            assert_eq!(w, snap_cells_up(app.text_width(text) + 2.0 * TEXT_PAD_X));
            assert!(w >= PANE_BTN_W, "Pane 标题栏按钮比下限还窄");
        }

        // 图表上的浮动小标签（十字光标数值 / 索引）。
        for s in ["12.34", "123"] {
            assert!(
                app.cross_label_w(s) - app.text_width(s) >= min_pad,
                "浮动标签「{s}」空档不足"
            );
        }
    }
    /// 铁律的下一层：文字不只不能贴边，还得整串落在「内容格」里 - 内容格的起点是
    /// 9k + 1（网格线格占最左那 1px），所以文字起点的坐标必须满足 (x - LINE_W) % PITCH == 0。
    /// 左对齐靠 TEXT_INSET_X；居中靠「盒子宽 = 文字宽 + 19」在 cells(n) 盒里居中（偏移正好 10）。
    #[test]
    fn text_starts_on_a_content_cell() {
        // 左对齐的内边距就是「1px 边框 + 1 格空档 + 那条线格」。
        assert_eq!(TEXT_INSET_X, LINE_W + PITCH);
        assert_eq!(((TEXT_INSET_X as i32 - LINE_W as i32) % PITCH as i32), 0);
        assert!(TEXT_INSET_X > TEXT_PAD_X, "落点要比算宽度用的大 1px（让开线格）");

        // 图标 + 间隔必须是 9 的整数倍，否则图标后面那串文字就会偏出内容格。
        assert_eq!(((ICON_W + ICON_GAP) as i32) % 9, 0, "图标区 + 间隔不是整数格");

        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        if app.text_engine.is_none() {
            return;
        }
        // 居中：盒宽 = 文字宽 + 2*TEXT_PAD_X 向上吸格，居中后的偏移必须正好是 TEXT_INSET_X。
        for s in ["重新生成", "图表配色跟随系统", "打印当前状态", "12.34", "255", "360"] {
            let text = app.text_width(s);
            let box_w = snap_cells_up(text + 2.0 * TEXT_PAD_X);
            let offset = ((box_w - text) / 2.0).round();
            assert_eq!(
                offset, TEXT_INSET_X,
                "「{s}」在 cells 盒里居中之后没落在内容格起点（偏移 {offset}）"
            );
        }
    }
    /// 右对齐文字和浮动小标签的落点也必须踩在内容格上：纵轴刻度、十字光标的数值牌、
    /// 时间带的索引牌 - 这些落点都是算出来贴上去的，最容易写成「贴边 1px」而不自知。
    #[test]
    fn right_aligned_and_floating_labels_land_on_content_cells() {
        // 落点工具本身：左对齐吸到左边那条线，右对齐停在那条线左边 1 格。
        assert_eq!(text_left_x(0.0), TEXT_INSET_X);
        assert_eq!(text_left_x(LINE_W), TEXT_INSET_X, "shrink 过 1px 的内框也得吸到同一条线");
        assert_eq!(text_left_x(PITCH), PITCH + TEXT_INSET_X);
        assert_eq!(text_right_x(9.0 * 89.0), 9.0 * 89.0 - GRID);

        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        if app.text_engine.is_none() {
            return;
        }
        app.bars = data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START);

        // 三块图表的几何（和 ui() 里那套一样），逐块查纵轴和绘图区。
        let work = left_work();
        app.clamp_left_heights(work);
        let [kline, _bar, volume, _bar2, indicator] =
            StockApp::left_rows(work, app.kline_h, app.vol_h);
        for area in [kline, volume, indicator] {
            let (_, _, main, _) = chart_rows(area, app.panes[0].indicator.layout());
            let (plot, axis) = frame_rects(main);
            let axis_line = (axis.max.x - LINE_W) as i32;
            assert_eq!(axis_line % PITCH as i32, 0, "轴的右边框不在竖网格线上");

            // 纵轴刻度：右对齐，末尾停在右边框那条线左边 1 格 -> 最后一个字落在完整内容格里。
            let tick = StockApp::axis_tick_x(axis) as i32;
            assert_eq!((tick - LINE_W as i32) % PITCH as i32, 0, "轴刻度没压在内容格上：{tick}");
            assert_eq!(axis_line - tick, GRID as i32, "轴刻度和轴的右边框之间的空档不是 1 格");

            // 十字光标的数值牌：左边落在网格线上（里头的居中文字才踩得准），右边跟轴右边框留 1 格。
            for s in ["20.00", "1234", "255"] {
                let w = app.cross_label_w(s);
                let x0 = StockApp::cross_axis_label_x(axis, w) as i32;
                assert_eq!(x0 % PITCH as i32, 0, "数值牌左边不在网格线上：{x0}");
                assert!(x0 >= axis.min.x as i32, "数值牌跑到轴左边去了：{x0}");
                assert!(
                    x0 + w as i32 <= axis_line - GRID as i32,
                    "数值牌顶到轴的右边框上了：{x0}+{w}"
                );
            }

            // 时间带的索引牌：牌子左边落在网格线上，整块夹在绘图区里。
            for (cx, s) in [(plot.min.x, "123"), (plot.center().x, "12"), (plot.max.x, "1234")] {
                let w = app.cross_label_w(s);
                let x0 = StockApp::cross_time_label_x(plot, cx, w) as i32;
                assert_eq!(x0 % PITCH as i32, 0, "索引牌左边不在网格线上：{x0}");
                assert!(x0 >= plot.min.x as i32 - LINE_W as i32, "索引牌跑出绘图区左边：{x0}");
                assert!(x0 + w as i32 <= plot.max.x as i32, "索引牌顶出绘图区右边");
            }
        }

        // 颜色面板：Hex 输入框里的文字按外框落点 - 内框（shrink 1px）得和外框吸到同一条线。
        let panel = egui::Rect::from_min_size(
            egui::pos2(CP_DEFAULT_X, CP_DEFAULT_Y),
            egui::vec2(CP_W, CP_H),
        );
        let hex_band = cp_band(panel, CP_ROW_HEX, CP_HEX_ROWS);
        let hex_text = text_left_x(hex_band.min.x) as i32;
        assert_eq!(
            (hex_text - LINE_W as i32) % PITCH as i32,
            0,
            "Hex 文字没压在内容格上：{hex_text}"
        );
        assert_eq!(text_left_x(hex_band.shrink(LINE_W).min.x) as i32, hex_text);
    }
    /// 纵轴刻度数字也守同一条铁律：字号换成按格算（每字 9px）之后，轴里最长的那个数值
    /// 还得放得下并且左右各留 1 格 - 轴就 8 格宽，数值一长最先顶出来。
    #[test]
    fn axis_ticks_fit_inside_the_axis_column() {
        let ctx = egui::Context::default();
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        if app.text_engine.is_none() {
            return;
        }
        app.bars = data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START);
        // 刻度文本就是 "%.2f"，最长的那个只会出现在最高价 / 最低价上。
        let hi = app.bars.iter().map(|b| b.h).fold(f64::MIN, f64::max);
        let lo = app.bars.iter().map(|b| b.l).fold(f64::MAX, f64::min);
        for v in [hi, lo] {
            let s = format!("{v:.2}");
            let w = app.text_width(&s);
            assert!(
                w + 2.0 * GRID <= AXIS_W,
                "纵轴刻度「{s}」装不下：轴宽 {AXIS_W}、文字宽 {w}"
            );
        }
    }

    /// 拖横向拖拽条（或拖窗口 / 拖分割条）的时候，鼠标扫过绘图区不该冒出十字光标，
    /// 也不该把图表拖走 - 拖拽条和图表区互不干扰；松开之后一切照旧。
    #[test]
    fn splitter_drag_does_not_touch_the_chart() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.bars = data::random_walk::generate(BAR_SEED, 64, BAR_START);

        let work = left_work();
        app.clamp_left_heights(work);
        let [kline, bar1, volume, bar2, indicator] =
            StockApp::left_rows(work, app.kline_h, app.vol_h);
        let bars = [bar1, bar2, egui::Rect::NOTHING];
        let areas = [kline, volume, indicator];
        let mut plots = [egui::Rect::NOTHING; 3];
        for (i, area) in areas.iter().enumerate() {
            let l = app.panes[i].indicator.layout();
            let (_, _, main, _) = chart_rows(*area, l);
            plots[i] = frame_rects(main).0;
            app.panes[i].height = main.height() as i32;
        }
        app.clamp_view(plots[0].width());
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        let target = plots[1].center();

        let press = |pos: egui::Pos2| {
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
            ]
        };

        // 没有别的拖拽时：在绘图区按下就该开图表平移，鼠标在图上就该有十字光标（正常行为）。
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(screen);
        raw.events = press(target);
        let mut out = ctx.run_ui(raw, |ui| {
            app.chart_interact(ui, &plots, &bars);
            app.update_crosshair(ui, &plots);
        });
        absorb_textures(&mut out.textures_delta);
        assert!(app.pan.is_some(), "绘图区按下该能平移");
        assert!(app.crosshair.is_some(), "鼠标在图上该有十字光标");
        app.pan = None;

        // 正在拖横向拖拽条：同一套输入，图表区整块让路。
        app.hsplit_grab = Some(5.0);
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(screen);
        raw.events = press(target);
        let mut out = ctx.run_ui(raw, |ui| {
            app.chart_interact(ui, &plots, &bars);
            app.update_crosshair(ui, &plots);
        });
        absorb_textures(&mut out.textures_delta);
        assert!(app.pan.is_none(), "拖拖拽条时不该把图表拖走");
        assert!(app.crosshair.is_none(), "拖拖拽条时不该响应图表区的十字光标");

        // 拖拖拽条时滚轮也不许缩放：鼠标压在绘图区里 + 滚轮，同一套输入，K 线该纹丝不动。
        let before = (app.scale, app.offset);
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(screen);
        raw.events = vec![
            egui::Event::PointerMoved(target),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Line,
                delta: egui::vec2(0.0, 3.0),
                phase: egui::TouchPhase::Move,
                modifiers: egui::Modifiers::default(),
            },
        ];
        let mut out = ctx.run_ui(raw, |ui| {
            app.chart_interact(ui, &plots, &bars);
            app.update_crosshair(ui, &plots);
        });
        absorb_textures(&mut out.textures_delta);
        assert_eq!(
            (app.scale, app.offset),
            before,
            "拖拖拽条时滚轮不该缩放 / 平移 K 线"
        );
        assert!(app.crosshair.is_none(), "拖拖拽条时滚轮扫过绘图区也不该冒十字线");
        app.hsplit_grab = None;

        // 松开拖拽条之后：十字光标照常回来。
        app.hsplit_grab = None;
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(screen);
        raw.events = vec![egui::Event::PointerMoved(target)];
        let mut out = ctx.run_ui(raw, |ui| {
            app.update_crosshair(ui, &plots);
        });
        absorb_textures(&mut out.textures_delta);
        assert!(app.crosshair.is_some(), "松开拖拽条之后十字光标该恢复");

        // 正在拖浮动面板（颜色面板 / 源码窗）的时候也一样：鼠标扫过绘图区不该冒十字线。
        app.color_panel.win.drag = true;
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(screen);
        raw.events = vec![egui::Event::PointerMoved(target)];
        let mut out = ctx.run_ui(raw, |ui| {
            app.update_crosshair(ui, &plots);
        });
        absorb_textures(&mut out.textures_delta);
        assert!(app.crosshair.is_none(), "拖浮动窗口时不该响应图表区的十字光标");
        app.color_panel.win.drag = false;
        app.source.win.drag = true;
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(screen);
        raw.events = vec![egui::Event::PointerMoved(target)];
        let mut out = ctx.run_ui(raw, |ui| {
            app.update_crosshair(ui, &plots);
        });
        absorb_textures(&mut out.textures_delta);
        assert!(app.crosshair.is_none(), "拖源码窗时不该响应图表区的十字光标");
        app.source.win.drag = false;
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(screen);
        raw.events = vec![egui::Event::PointerMoved(target)];
        let mut out = ctx.run_ui(raw, |ui| {
            app.update_crosshair(ui, &plots);
        });
        absorb_textures(&mut out.textures_delta);
        assert!(app.crosshair.is_some(), "松开浮动窗口之后十字光标该恢复");
    }

    /// 图表状态带（状态轴）里，项与项之间的分割线是 #999999 的点状虚线，落在两项共用的那条
    /// 竖网格线上（线只占 1px）。这是「状态轴里面的元素之间要有分割线」的落实。
    #[test]
    fn chart_status_band_has_dotted_dividers_on_the_grid() {
        use crate::indicator::candle::{INFO_LABELS, INFO_TEMPLATES};
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        if app.text_engine.is_none() {
            return; // 没拿到 GDI 字体就量不出宽度，跳过
        }
        app.bars = data::random_walk::generate(BAR_SEED, 64, BAR_START);
        let work = left_work();
        app.clamp_left_heights(work);
        let [kline, _bar, _vol, _bar2, _ind] = StockApp::left_rows(work, app.kline_h, app.vol_h);
        let layout = app.panes[PANE_CANDLE].indicator.layout();
        let (_, status, _, _) = chart_rows(kline, layout);
        let band = status.unwrap();
        // 和重放时同一套算式算出每条分割线该落在哪一列（相对状态带左边）。
        let mut want = Vec::new();
        let mut sx = band.min.x;
        let mut first = true;
        for i in 0..INFO_LABELS.len() {
            let w = snap_cells_up(
                app.text_width(INFO_LABELS[i])
                    + GRID
                    + app.text_width(INFO_TEMPLATES[i])
                    + 2.0 * TEXT_PAD_X,
            );
            if sx + w > band.max.x {
                break;
            }
            if !first {
                want.push(sx - band.min.x);
            }
            sx += w - LINE_W;
            first = false;
        }
        assert!(want.len() >= 2, "状态带至少要有几条分割线才测得出");

        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(WINDOW_W, WINDOW_H),
        ));
        let mut out = ctx.run_ui(raw, |ui| {
            app.ui_chart(ui, PANE_CANDLE, kline);
        });
        let shapes = std::mem::take(&mut out.shapes);
        absorb_textures(&mut out.textures_delta);
        let grid = rasterize_into(&ctx.tessellate(shapes, 1.0), WINDOW_W as usize, WINDOW_H as usize);
        let rows = band.min.y as usize + 1..band.max.y as usize - 1;
        for dx in want {
            assert_eq!(dx % 9.0, 0.0, "状态带分割线没落在竖网格线上");
            let col = (band.min.x + dx) as usize;
            let lit = rows.clone().filter(|&y| grid[y][col] == Some(C_LINE)).count();
            assert!(lit > 0, "状态带分割线 {col} 没画出来");
            assert!(lit < rows.len(), "状态带分割线 {col} 是实线，不是点状虚线");
        }
    }

    /// 窗口内状态栏：几项从左往右排，项与项之间的分割线是 #999999 的点状虚线，
    /// 落在两项共用的那条竖网格线上；文字左右各留 1 格（TEXT_PAD_X = 1 格 + 1px）。
    #[test]
    fn status_bar_has_dotted_dividers_on_the_grid() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        if app.text_engine.is_none() {
            return;
        }
        app.fps = 60.4;
        app.bars = data::random_walk::generate(BAR_SEED, 80, BAR_START);
        app.visible = 80;
        let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        let areas = split_areas(layout);
        let status = areas[3];
        // 和 paint_skeleton 同一套算式算出每条分割线该落在哪一列（相对状态栏左边）。
        let mut want = Vec::new();
        let mut sx = status.min.x;
        let mut first = true;
        for seg in app.status_segments() {
            let w = snap_cells_up(app.text_width(&seg) + 2.0 * TEXT_PAD_X);
            if sx + w > status.max.x {
                break;
            }
            if !first {
                want.push(sx - status.min.x);
            }
            sx += w - LINE_W;
            first = false;
        }
        assert!(want.len() >= 2, "状态栏至少要有几条分割线才测得出");

        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        let mut out = ctx.run_ui(raw, |ui| {
            app.paint_skeleton(ui.painter(), layout, areas);
        });
        let shapes = std::mem::take(&mut out.shapes);
        absorb_textures(&mut out.textures_delta);
        let grid = rasterize_into(&ctx.tessellate(shapes, 1.0), WINDOW_W as usize, WINDOW_H as usize);
        let rows = status.min.y as usize + 1..status.max.y as usize - 1;
        for dx in want {
            assert_eq!(dx % 9.0, 0.0, "状态栏分割线没落在竖网格线上");
            let col = (status.min.x + dx) as usize;
            let lit = rows.clone().filter(|&y| grid[y][col] == Some(C_LINE)).count();
            assert!(lit > 0, "状态栏分割线 {col} 没画出来");
            assert!(lit < rows.len(), "状态栏分割线 {col} 是实线，不是点状虚线");
        }
    }

    /// 三块图表拉到统一的最小高度（PANE_H_MIN = 20 格）时还能正常出图、不 panic：
    /// 绘图区高度仍是正的，十字光标连着标签也还能建出来。这是「拉到极致不会崩」的兜底用例。
    #[test]
    fn panes_render_at_the_minimum_height() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.bars = data::random_walk::generate(BAR_SEED, 64, BAR_START);
        app.text_engine = HardText::new(&ctx);

        // 左栏只放到「三块各 20 格 + 两条 1 格拖拽条」那么高：三块都被压到统一下限。
        let work = egui::Rect::from_min_size(
            egui::pos2(0.0, 54.0),
            egui::vec2(CHART_PANE_W, cells(LEFT_WORK_MIN_ROWS)),
        );
        app.ind_h = PANE_H_MIN;
        app.vol_h = PANE_H_MIN;
        app.clamp_left_heights(work);
        assert_eq!(app.kline_h, PANE_H_MIN);
        assert_eq!(app.vol_h, PANE_H_MIN);
        assert_eq!(app.ind_h, PANE_H_MIN);

        let [kline, _bar, volume, _bar2, indicator] =
            StockApp::left_rows(work, app.kline_h, app.vol_h);
        let areas = [kline, volume, indicator];
        let mut plots = [egui::Rect::NOTHING; 3];
        for (i, area) in areas.iter().enumerate() {
            let layout = app.panes[i].indicator.layout();
            let (_, _, main, _) = chart_rows(*area, layout);
            plots[i] = frame_rects(main).0;
            app.panes[i].height = main.height() as i32;
            assert!(plots[i].height() > 0.0, "第 {} 块绘图区高度非正", i + 1);
        }
        app.clamp_view(plots[0].width());
        for i in 0..app.panes.len() {
            app.ensure_pane_computed(i);
        }

        // 鼠标压在最下面那块（指标区）的最小绘图区里：十字光标与标签的吸附 / 夹取不能 panic。
        let target = plots[2].center();
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(WINDOW_W, WINDOW_H),
        ));
        raw.events = vec![egui::Event::PointerMoved(target)];
        let mut out = ctx.run_ui(raw, |ui| {
            app.update_crosshair(ui, &plots);
            for (i, area) in areas.iter().enumerate() {
                app.ui_chart(ui, i, *area);
            }
        });
        assert!(app.crosshair.is_some(), "最小高度下十字光标没建起来");
        let _ = ctx.tessellate(std::mem::take(&mut out.shapes), 1.0);
        absorb_textures(&mut out.textures_delta);
    }

    /// 左栏拉到最小宽度（PANE_MIN_W = 35 格 = 316px）时，三块图表也都能正常出图、不 panic：
    /// 标题栏两个按钮还放得下（贴着右边框），绘图区宽度仍是正的。
    #[test]
    fn panes_render_at_the_minimum_width() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.bars = data::random_walk::generate(BAR_SEED, 64, BAR_START);
        app.text_engine = HardText::new(&ctx);

        let work = egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(PANE_MIN_W, cells(WORK_ROWS)),
        );
        app.clamp_left_heights(work);
        let [kline, _bar, volume, _bar2, indicator] =
            StockApp::left_rows(work, app.kline_h, app.vol_h);
        let areas = [kline, volume, indicator];
        // 标题栏两个按钮是最小宽度下最吃横向空间的东西：最右那个贴右边框、最左那个不出左边框。
        let layout = app.panes[PANE_CANDLE].indicator.layout();
        let (title, _, _, _) = chart_rows(kline, layout);
        let buttons = StockApp::pane_button_rects(title, &pane_btn_widths());
        assert_eq!(buttons[1].max.x, kline.max.x, "最右按钮没贴着右边框");
        assert!(buttons[0].min.x >= kline.min.x, "按钮超出左栏");

        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(WINDOW_W, WINDOW_H),
        ));
        let mut out = ctx.run_ui(raw, |ui| {
            for (i, area) in areas.iter().enumerate() {
                app.ui_chart(ui, i, *area);
            }
        });
        let _ = ctx.tessellate(std::mem::take(&mut out.shapes), 1.0);
        absorb_textures(&mut out.textures_delta);
    }

    /// 照 ui() 的顺序把整窗画一遍再光栅化（跳过只有真窗口才有的句柄 / 拖动逻辑）。
    /// 整窗渲染（不带鼠标）：不关心十字光标的 dump / 测试都走这个。
    fn render_window(app: &mut StockApp) -> Vec<Vec<Option<egui::Color32>>> {
        render_window_at(app, None)
    }

    /// 上面的「带鼠标」版本：pointer = Some 时先跑一遍交互 + 十字光标，再照常画整窗，
    /// 于是图上的十字线和两块浮动小牌子（纵轴数值牌 / 时间带索引牌）都会出现在 dump 里。
    fn render_window_at(
        app: &mut StockApp,
        pointer: Option<egui::Pos2>,
    ) -> Vec<Vec<Option<egui::Color32>>> {
        if app.panel_buttons.is_empty() {
            app.build_panel_buttons();
        }
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        // 字形图集挂在 ctx 上：这份 dump 用自己的 ctx，引擎就得重建在这个 ctx 上，
        // 否则光栅器收不到字形贴图，dump 出来的文字又变回一个个实心方块。
        if let Some(engine) = HardText::new(&ctx) {
            app.text_engine = Some(engine);
        }
        let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        // 塞一个鼠标位置进来，等价于真窗口收到 PointerMoved。
        if let Some(p) = pointer {
            raw.events = vec![egui::Event::PointerMoved(p)];
        }
        let mut out = ctx.run_ui(raw, |ui| {
            let painter = ui.painter().clone();
            let areas = split_areas(layout);
            app.paint_skeleton(&painter, layout, areas);
            app.ui_titlebar(ui, areas[0], Some(layout));
            let menubar_state = app.menu;
            let (_, menubar_state) = ui_menu_bar(app, ui, areas[1], &TOP_TABS, menubar_state);
            app.menu = menubar_state;
            app.clamp_split(areas[2]);
            let [left, splitter, panel] = StockApp::work_columns(areas[2], app.split_x);
            app.clamp_left_heights(left);
            let [chart1, hsplit, chart2, vsplit, chart3] =
                StockApp::left_rows(left, app.kline_h, app.vol_h);
            let areas_3 = [chart1, chart2, chart3];
            let mut plots = [egui::Rect::NOTHING; 3];
            for (i, area) in areas_3.iter().enumerate() {
                let l = app.panes[i].indicator.layout();
                let (_, _, main, _) = chart_rows(*area, l);
                plots[i] = frame_rects(main).0;
                app.panes[i].height = main.height() as i32;
            }
            app.sync_pane_palette();
            for i in 0..app.panes.len() {
                app.ensure_pane_computed(i);
            }
            app.clamp_view(plots[0].width());
            if pointer.is_some() {
                let bars = [hsplit, vsplit, splitter];
                app.chart_interact(ui, &plots, &bars);
                app.update_crosshair(ui, &plots);
            }
            app.ui_chart(ui, PANE_CANDLE, chart1);
            app.ui_chart(ui, PANE_VOLUME, chart2);
            app.ui_chart(ui, PANE_MACD, chart3);
            app.ui_hsplitter(ui, hsplit, left, LeftSplit::Volume);
            app.ui_hsplitter(ui, vsplit, left, LeftSplit::Indicator);
            app.ui_splitter(ui, splitter, areas[2]);
            app.ui_panel(ui, panel);
            app.paint_separators(&painter, layout, areas);
            border_hard(&painter, layout, C_CTRL_BORDER);
            // 两个下拉面板和 ui() 一样排在最上层（没开的时候它们什么都不画）。
            app.ui_pane_menu(ui);
            app.ui_overlay_menu(ui);
            // 网格测试叠层（调试用）：和 ui() 一样排在最上层，控件边框落在网格线上就会被它盖住。
            if app.show_grid {
                app.paint_grid_test(&painter, layout);
            }
        });
        let shapes = std::mem::take(&mut out.shapes);
        absorb_textures(&mut out.textures_delta);
        rasterize_into(&ctx.tessellate(shapes, 1.0), WINDOW_W as usize, WINDOW_H as usize)
    }

    /// 把像素网格写成 24 位 BMP（None 当白底）。只给人工肉眼检查用。
    fn write_bmp(path: &str, grid: &[Vec<Option<egui::Color32>>]) {
        let h = grid.len();
        let w = grid.first().map_or(0, |r| r.len());
        let row_pad = (4 - (w * 3) % 4) % 4;
        let row_size = w * 3 + row_pad;
        let pixel_bytes = row_size * h;
        let file_size = 54 + pixel_bytes;
        let mut out = Vec::with_capacity(file_size);
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&(file_size as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&54u32.to_le_bytes());
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&(w as i32).to_le_bytes());
        out.extend_from_slice(&(h as i32).to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&24u16.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(pixel_bytes as u32).to_le_bytes());
        out.extend_from_slice(&2835u32.to_le_bytes());
        out.extend_from_slice(&2835u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        for y in (0..h).rev() {
            for px in &grid[y] {
                let (r, g, b) = match px {
                    Some(c) => (c.r(), c.g(), c.b()),
                    None => (255, 255, 255),
                };
                out.extend_from_slice(&[b, g, r]);
            }
            out.extend(std::iter::repeat_n(0u8, row_pad));
        }
        std::fs::write(path, out).expect("写 BMP 失败");
    }

    /// 整窗渲染结果落一份 BMP 到 target/ 供人工检查（默认忽略，手动 `--ignored` 跑）。
    #[test]
    #[ignore]
    fn dump_full_window_for_visual_check() {
        let mut app = blank_app();
        app.bars = data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START);
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        app.text_engine = HardText::new(&ctx);
        app.fps = 60.0;
        app.visible = visible_bars(CHART_PANE_W - LINE_W - AXIS_W, BAR_SCALE_DEFAULT);
        let grid = render_window(&mut app);
        write_bmp("target/ui_full.bmp", &grid);

        // 再落一张暗色皮肤的，肉眼看看切换主题之后整屏的样子。
        set_theme(Theme::DARK);
        app.up_color = (Theme::DARK.candle_up.r(), Theme::DARK.candle_up.g(), Theme::DARK.candle_up.b());
        app.down_color = (Theme::DARK.candle_down.r(), Theme::DARK.candle_down.g(), Theme::DARK.candle_down.b());
        app.mark_panes_dirty();
        let dark = render_window(&mut app);
        write_bmp("target/ui_full_dark.bmp", &dark);
        // 再把十字光标（含纵轴数值牌 / 时间带索引牌两块浮动小牌子）落一张：
        // 鼠标压在 K 线区绘图区正中，肉眼核对两块牌子也踩在网格上。
        set_theme(Theme::CLASSIC);
        app.up_color = UP_RGB_DEFAULT;
        app.down_color = DOWN_RGB_DEFAULT;
        app.mark_panes_dirty();
        let work = left_work();
        app.clamp_left_heights(work);
        let [kline, _b0, _vol, _b1, _ind] = StockApp::left_rows(work, app.kline_h, app.vol_h);
        let (_, _, main, _) = chart_rows(kline, app.panes[PANE_CANDLE].indicator.layout());
        let pos = frame_rects(main).0.center();
        let cross = render_window_at(&mut app, Some(pos));
        write_bmp("target/ui_crosshair.bmp", &cross);
        // 再落一张开了「网格测试」叠层的：全界面的 9px 网格线（洋红）铺在最上层，
        // 控件边框只要落在网格线上就会被它整条盖住，没盖住的会露在线旁边 - 肉眼核对对齐用。
        app.show_grid = true;
        let grid_on = render_window(&mut app);
        write_bmp("target/ui_grid.bmp", &grid_on);
        app.show_grid = false;
        set_theme(Theme::CLASSIC);
    }

    /// 肉眼看浮动面板：颜色面板 + 源码窗各落一张 BMP - 用来核对面板里的文字有没有越出
    /// 它自己那一格 / 压到边框上。人工检查用，不做断言。
    #[test]
    #[ignore]
    fn dump_panels_for_visual_check() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);

        let mut app = color_panel_app();
        app.text_engine = HardText::new(&ctx);
        let grid = render_panel(&mut app);
        write_bmp("target/ui_color_panel.bmp", &grid);

        // 源码窗：换成 MACD（源码最长的那个内置指标）再打开。
        let mut app2 = blank_app();
        app2.text_engine = HardText::new(&ctx);
        app2.bars = data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START);
        app2.open_source(PANE_MACD);
        let grid2 = render_overlays(&mut app2);
        write_bmp("target/ui_source.bmp", &grid2);
    }

    /// 肉眼看新指标（美国线 / KDJ / RSI）和「叠加」菜单：各落一张 BMP。
    #[test]
    #[ignore]
    fn dump_new_indicators_for_visual_check() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        app.bars = data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START);
        app.fps = 60.0;
        app.visible = visible_bars(CHART_PANE_W - LINE_W - AXIS_W, BAR_SCALE_DEFAULT);
        // 三个槽位换成 美国线 / KDJ / RSI（builtin() 里的下标 1 / 4 / 5）。
        app.set_pane_indicator(PANE_CANDLE, 1);
        app.set_pane_indicator(PANE_VOLUME, 4);
        app.set_pane_indicator(PANE_MACD, 5);
        let grid = render_window(&mut app);
        write_bmp("target/ui_indicators.bmp", &grid);

        // 再落一张「叠加」菜单开着的（默认勾着 MA5/10/20/60 - 菜单里前四行的色块是实心的）。
        app.set_pane_indicator(PANE_CANDLE, 0);
        app.set_pane_indicator(PANE_VOLUME, 2);
        app.set_pane_indicator(PANE_MACD, 3);
        app.overlay_menu = Some(PANE_CANDLE);
        let grid2 = render_window(&mut app);
        write_bmp("target/ui_overlay.bmp", &grid2);

        // 再来一张暗色皮肤的（同样的蜡烛图 +「叠加」菜单）：核对新加的那些颜色在黑底上认不认得出。
        app.overlay_menu = None;
        set_theme(Theme::DARK);
        app.up_color = (Theme::DARK.candle_up.r(), Theme::DARK.candle_up.g(), Theme::DARK.candle_up.b());
        app.down_color = (Theme::DARK.candle_down.r(), Theme::DARK.candle_down.g(), Theme::DARK.candle_down.b());
        app.mark_panes_dirty();
        app.set_pane_indicator(PANE_VOLUME, 4);
        app.set_pane_indicator(PANE_MACD, 5);
        app.overlay_menu = Some(PANE_CANDLE);
        let grid3 = render_window(&mut app);
        write_bmp("target/ui_indicators_dark.bmp", &grid3);
        set_theme(Theme::CLASSIC);
    }

    /// 整窗 + 浮动层（颜色面板 / 源码窗）：只给 dump 用，正式那套流水线在 app.rs 的 ui() 里。
    fn render_overlays(app: &mut StockApp) -> Vec<Vec<Option<egui::Color32>>> {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        // 字形图集挂在 ctx 上：这份 dump 用自己的 ctx，引擎就得重建在这个 ctx 上，
        // 否则光栅器收不到字形贴图，dump 出来的文字又变回一个个实心方块。
        if let Some(engine) = HardText::new(&ctx) {
            app.text_engine = Some(engine);
        }
        let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        let mut out = ctx.run_ui(raw, |ui| {
            let painter = ui.painter().clone();
            let areas = split_areas(layout);
            app.paint_skeleton(&painter, layout, areas);
            app.ui_source_window(ui, layout);
        });
        let shapes = std::mem::take(&mut out.shapes);
        absorb_textures(&mut out.textures_delta);
        rasterize_into(&ctx.tessellate(shapes, 1.0), WINDOW_W as usize, WINDOW_H as usize)
    }
    /// 整窗的网格对齐总检：所有区域 / 分栏 / 图表内的边界都落在 9px 网格线上，
    /// 相邻两块共用边界那条线永远只有 1px。这是给「强迫症式」检查兜底的一条。
    #[test]
    fn full_window_borders_sit_on_the_grid() {
        let mut app = blank_app();
        app.bars = data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START);
        let grid = render_window(&mut app);
        let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        let areas = split_areas(layout);
        // 四个区域的上沿都在网格线上；功能区下沿和状态栏上沿共用那条线。
        for y in [areas[1].min.y, areas[2].min.y, areas[3].min.y] {
            assert_eq!(y % 9.0, 0.0, "区域分隔线 {y} 没落在横网格线上");
        }
        assert_eq!(areas[2].max.y, areas[3].min.y + LINE_W, "功能区和状态栏没共用那条线");
        // 左栏三块的边界都落在网格线上。
        let [left, _sp, _panel] = StockApp::work_columns(areas[2], app.split_x);
        let [chart1, _h1, chart2, _h2, chart3] = StockApp::left_rows(left, app.kline_h, app.vol_h);
        // 三块的上沿都是网格线；第三块的下沿比最后那条网格线多 1px（那条线是和状态栏共用的）。
        for y in [chart1.min.y, chart2.min.y, chart3.min.y, chart3.max.y - LINE_W] {
            assert_eq!(
                ((y - areas[2].min.y) as i32) % 9,
                0,
                "图表边界 {y} 没落在横网格线上"
            );
        }
        // 每一块图表内部：标题栏 / 状态带 / 绘图区 / 时间带的边界也都在网格线上。
        for (i, area) in [chart1, chart2, chart3].iter().enumerate() {
            let l = app.panes[i].indicator.layout();
            let (title, status, main, time) = chart_rows(*area, l);
            assert_eq!(title.min.y, area.min.y);
            assert_eq!(
                (main.min.y - LINE_W - area.min.y) % 9.0,
                0.0,
                "第 {} 块绘图区上沿不在网格线上",
                i + 1
            );
            if let Some(s) = status {
                assert_eq!((s.min.y - area.min.y) % 9.0, 0.0, "状态带上沿不在网格线上");
            }
            if let Some(t) = time {
                assert_eq!((t.min.y - area.min.y) % 9.0, 0.0, "时间带上沿不在网格线上");
                assert_eq!(t.max.y, area.max.y, "时间带没铺到图表底边");
            } else {
                assert_eq!(main.max.y, area.max.y, "绘图区没铺到图表底边");
            }
        }
        // 顶上两处区域分隔线只有 1px：这一行是线、上一行还是干净底色（标题栏 / 菜单栏的灰底）。
        let col = 40usize;
        for y in [areas[1].min.y, areas[2].min.y] {
            let row = y as usize;
            assert_eq!(grid[row][col], Some(C_LINE), "区域分隔线 {y} 没画出来");
            assert_eq!(grid[row - 1][col], Some(C_TITLE_BG), "区域分隔线 {y} 不止 1px");
        }
    }

    /// 图标（快捷按钮的图标 = 读 16x16 PNG 再自绘）：启动装图标走「读 assets/icons/<名字>.png
    /// -> 转成 16x16 色块 -> 自绘」这条路，没有文件就按内置点阵写一份出来（默认图标就这么生成，
    /// 以后改图标直接改这些 PNG）。这一条把默认图标落盘再读回来核对：每张都是 16x16 真彩 + alpha、
    /// 透明色是 #ff00ff、有墨的像素都吸到 #aabbcc 形，读回来的图和内置点阵逐像素一致。
    #[test]
    fn icon_pngs_round_trip_through_disk() {
        let set = IconSet::load_or_create();
        let dir = icons_dir();
        for kind in IconKind::ALL {
            let name = kind.file_stem();
            let path = dir.join(format!("{name}.png"));
            let bytes =
                std::fs::read(&path).unwrap_or_else(|e| panic!("读不到图标 {name}.png：{e}"));
            let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
                .read_info()
                .expect("PNG 头读不动");
            let (w, h) = {
                let info = reader.info();
                (info.width, info.height)
            };
            assert_eq!((w, h), (16, 16), "{name}.png 不是 16x16");
            let mut buf = vec![0u8; reader.output_buffer_size().unwrap_or(0)];
            let info = reader.next_frame(&mut buf).expect("PNG 数据读不动");
            assert_eq!(info.color_type, png::ColorType::Rgba, "{name}.png 不是真彩 + alpha");
            for px in buf[..info.buffer_size()].chunks_exact(4) {
                let (r, g, b) = (px[0], px[1], px[2]);
                if (r, g, b) == ICON_TRANSPARENT_RGB {
                    continue;
                }
                for c in [r, g, b] {
                    assert_eq!(c % 17, 0, "{name}.png 里有没吸到 #aabbcc 形的颜色：{c:#04x}");
                }
            }
            assert!(
                *set.get(kind) == IconImage::from_art(&kind.art()),
                "{name}.png 读回来和内置点阵对不上"
            );
        }
    }

    /// 铁律（不许出现 2px 线叠加）：十字光标在时间轴上方那个索引牌，下边框必须正好压在绘图区
    /// 下沿那条线（和时间带共用的那条网格线）上，不能停在它上面一行 - 否则时间轴上方就会看到
    /// 一条 2px 的横线。牌子上沿因此落在下沿往上 2 格的网格线上，上下两条边框都只占 1px。
    #[test]
    fn crosshair_time_label_never_stacks_on_the_timeline_border() {
        let mut app = blank_app();
        app.bars = data::random_walk::generate(BAR_SEED, 64, BAR_START);
        // 几何和 render_window_at 里那套完全一样（整窗 -> 功能区 -> 左栏三块）。
        let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        let areas = split_areas(layout);
        app.clamp_split(areas[2]);
        let [left, _sp, _panel] = StockApp::work_columns(areas[2], app.split_x);
        app.clamp_left_heights(left);
        let [kline, _h1, _c2, _h2, _c3] = StockApp::left_rows(left, app.kline_h, app.vol_h);
        let (_, _, main, time) = chart_rows(kline, app.panes[PANE_CANDLE].indicator.layout());
        let time = time.expect("蜡烛图有时间带");
        let plot = frame_rects(main).0;
        // 绘图区和时间带共用的那条线：两段的边界都落在它上面。
        let shared = main.max.y as usize;
        assert_eq!(time.min.y as usize, shared, "时间带上沿不是那条共用的线");

        let grid = render_window_at(&mut app, Some(plot.center()));
        app.crosshair.expect("鼠标在绘图区里应该有十字光标");

        // 索引牌上沿落在网格线上；下边框就是 2 格之后那条网格线 = 共用的那条线。
        let label_top = (plot.max.y - PITCH * CROSS_LABEL_ROWS as f32) as usize;
        assert_eq!(label_top % 9, 0, "索引牌上边框没落在网格线上");
        assert_eq!(label_top + 18, shared, "索引牌下边框没压在共用的那条线上");

        let (x0, x1) = (plot.min.x as usize, plot.max.x as usize);
        // 共用的那条线上一行必须是干净底色：牌子的下边框要是落在这儿，就是 2px 叠线。
        // 某一横排里最长的边框色连排（牌子的竖边框只有 1px，横边框是一长条）。
        let longest = |row: usize| -> usize {
            let (mut best, mut cur) = (0usize, 0usize);
            for x in x0..x1 {
                cur = if grid[row][x] == Some(C_LINE) { cur + 1 } else { 0 };
                best = best.max(cur);
            }
            best
        };
        // 牌子的上边框是一条落在网格线上的横线（宽度就是牌子宽度，至少 2 格）。
        assert!(
            longest(label_top) >= 19,
            "绘图区下沿没看到索引牌的上边框（它该是一条落在网格线上的横线）"
        );
        // 共用的那条线上一行：牌子只有两条 1px 竖边框经过，绝不许再出一条横线（那就叠成 2px 了）。
        assert!(
            longest(shared - 1) <= 1,
            "索引牌和时间轴之间叠出了 2px 横线（连排 {}px）",
            longest(shared - 1)
        );
    }

    // =======================================================================
    //  可编辑源码编辑器与自绘输入框（ui::textbuf / ui::textbox / ui::editor）
    // =======================================================================

    /// 按下一个键（要不要带 Ctrl）。egui 的 Key 事件字段不少，测试里只挑用得上的填。
    fn key_event(key: egui::Key, ctrl: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: if ctrl {
                egui::Modifiers::CTRL
            } else {
                egui::Modifiers::NONE
            },
        }
    }

    /// 把一串事件喂给缓冲区：走的就是源码编辑器那条路（多行 + 中文和 ASCII 过滤）。
    fn feed(buf: &mut TextBuffer, preedit: &mut String, events: &[egui::Event]) -> EditOutcome {
        apply_edit_events(buf, preedit, events, true, &char_allowed)
    }

    /// 列是【字符索引】不是字节索引：中文一个字符 3 个字节，按字节来早就 panic 或者切出半个字了。
    #[test]
    fn editable_buffer_counts_columns_in_chars() {
        let mut buf = TextBuffer::from_text("中文abc");
        assert_eq!(buf.line_count(), 1);
        buf.move_end();
        buf.backspace();
        assert_eq!(buf.line(0), "中文ab", "退格没删掉最后一个 ASCII 字符");
        buf.move_home();
        buf.insert_char('测');
        assert_eq!(buf.line(0), "测中文ab", "行首插中文不对");
        buf.delete();
        assert_eq!(buf.line(0), "测文ab", "删一个中文字符不对（列不是字节）");
        for _ in 0..20 {
            buf.move_right();
        }
        assert_eq!(buf.cursor, (0, 4), "光标停不到行尾（4 个字符）");
    }

    /// 行首退格并上一行、行尾 Delete 并下一行、回车从光标处断行。
    #[test]
    fn editable_buffer_joins_and_splits_lines() {
        let mut buf = TextBuffer::from_text("第一行\n第二行");
        buf.set_cursor((1, 0), false);
        buf.backspace();
        assert_eq!(buf.line_count(), 1, "行首退格应该并成一行");
        assert_eq!(buf.line(0), "第一行第二行");
        assert_eq!(buf.cursor, (0, 3), "并完光标该停在接缝上");
        buf.insert_newline();
        assert_eq!(buf.line_count(), 2, "回车应该断成两行");
        assert_eq!(buf.line(0), "第一行");
        assert_eq!(buf.line(1), "第二行");
        buf.set_cursor((0, 3), false);
        buf.delete();
        assert_eq!(buf.line_count(), 1, "行尾 Delete 应该并成一行");
        assert_eq!(buf.line(0), "第一行第二行");
    }

    /// 上下键移行：列尽量保持，撞上短行就落到行尾再继续。
    #[test]
    fn editable_buffer_vertical_moves_keep_the_column() {
        let mut buf = TextBuffer::from_text("abcd\nab\nabcdef");
        buf.set_cursor((0, 3), false);
        buf.move_vertical(true);
        assert_eq!(buf.cursor, (1, 2), "短行应该落到行尾");
        buf.move_vertical(true);
        assert_eq!(buf.cursor, (2, 2), "列保持在上一次落点，不该跳回 3");
        buf.move_vertical(false);
        assert_eq!(buf.cursor, (1, 2), "往上也一样");
    }

    /// Tab 插 4 个空格（不插制表符）：点阵字体里制表符宽不定，等宽对齐就靠这个。
    #[test]
    fn editable_buffer_tab_inserts_spaces() {
        let mut buf = TextBuffer::from_text("x");
        let mut preedit = String::new();
        buf.move_end();
        let out = feed(&mut buf, &mut preedit, &[key_event(egui::Key::Tab, false)]);
        assert!(out.changed);
        assert_eq!(buf.line(0), "x    ");
        assert!(!buf.to_text().contains('\t'), "制表符不许进缓冲区");
    }

    /// 全选 / 复制 / 剪切 / 粘贴（剪贴板里带换行要正确拆成多行）/ 撤销。
    #[test]
    fn editable_buffer_selection_copy_cut_paste_undo() {
        let mut buf = TextBuffer::from_text("一行\n二行");
        let mut preedit = String::new();
        let out = feed(&mut buf, &mut preedit, &[key_event(egui::Key::A, true)]);
        assert!(!out.changed, "全选不改内容");
        assert_eq!(buf.selected_text(), "一行\n二行");
        let out = feed(&mut buf, &mut preedit, &[key_event(egui::Key::C, true)]);
        assert_eq!(out.copy.as_deref(), Some("一行\n二行"), "复制要给出选中文本");
        assert_eq!(buf.to_text(), "一行\n二行", "复制不许动内容");
        let out = feed(&mut buf, &mut preedit, &[key_event(egui::Key::X, true)]);
        assert!(out.changed);
        assert_eq!(out.copy.as_deref(), Some("一行\n二行"));
        assert_eq!(buf.to_text(), "", "剪切应该把内容拿走");
        let out = feed(
            &mut buf,
            &mut preedit,
            &[egui::Event::Paste("aa\nbb\ncc".to_owned())],
        );
        assert!(out.changed);
        assert_eq!(buf.line_count(), 3, "粘贴带换行的文本要拆成 3 行");
        assert_eq!(buf.to_text(), "aa\nbb\ncc");
        let out = feed(&mut buf, &mut preedit, &[key_event(egui::Key::Z, true)]);
        assert!(out.changed);
        assert_eq!(buf.to_text(), "", "撤销应该退回粘贴之前");
    }

    /// 中文输入法：候选串（Preedit）只是挂在光标上的临时串，Commit 才是真的上屏。
    /// DeleteSurrounding 按【字符数】删（列本来就是字符索引）。
    #[test]
    fn editable_buffer_ime_preedit_and_commit() {
        let mut buf = TextBuffer::from_text("");
        let mut preedit = String::new();
        let pre = egui::Event::Ime(egui::ImeEvent::Preedit {
            text: "zhong".to_owned(),
            active_range_chars: None,
        });
        let out = feed(&mut buf, &mut preedit, &[pre]);
        assert!(!out.changed, "候选串不能写进缓冲区");
        assert_eq!(preedit, "zhong");
        assert_eq!(buf.to_text(), "", "候选串没落盘才对");
        let out = feed(
            &mut buf,
            &mut preedit,
            &[egui::Event::Ime(egui::ImeEvent::Commit("中".to_owned()))],
        );
        assert!(out.changed);
        assert_eq!(preedit, "", "上屏之后候选串要清掉");
        assert_eq!(buf.to_text(), "中");
        let out = feed(
            &mut buf,
            &mut preedit,
            &[egui::Event::Ime(egui::ImeEvent::DeleteSurrounding {
                before_chars: 1,
                after_chars: 0,
            })],
        );
        assert!(out.changed);
        assert_eq!(buf.to_text(), "", "DeleteSurrounding 该把那个中文字删掉");
    }

    /// 装载必须无损：磁盘上那份源码（行尾归一成 \n）和缓冲区里的一字不差。
    /// 保存就是拿缓冲区写回去的，装载时少一个字符就是悄悄改用户的文件 -
    /// 中文标点（、。「」（）之类）和全角符号全都得原样留着。
    #[test]
    fn editable_buffer_round_trips_the_source_file() {
        let raw = std::fs::read_to_string("src/indicator/candle.rs").expect("读不到 candle.rs");
        let want = raw.replace("\r\n", "\n");
        assert!(want.contains('。') && want.contains('「'), "这份文件本来就带中文标点");
        let buf = TextBuffer::from_text(&want);
        assert_eq!(buf.to_text(), want, "装载之后和原文件对不上");
        assert!(buf.to_text().contains('。'), "中文句号被过滤掉了");
        assert!(buf.to_text().contains('「'), "中文引号被过滤掉了");
        // 末尾那个换行也不能丢：丢了下一次保存就把最后一行给改了。
        let tail = TextBuffer::from_text("a\n");
        assert_eq!(tail.to_text(), "a\n", "末尾换行没保住");
    }

    /// 落盘照原文件的行尾：仓库里的源文件清一色 CRLF，保存不能把整个文件的行尾改成 LF
    /// （那样每一行都会进 diff）。没有原文件就按仓库惯例的 CRLF 写，且不带 BOM。
    #[test]
    fn source_save_keeps_the_line_endings() {
        let mut path = std::env::temp_dir();
        path.push("rt_write_source_check.rs");
        std::fs::write(&path, "a\r\nb\r\n").unwrap();
        write_source(&path, "a\nb\nc").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "a\r\nb\r\nc");
        let fresh = path.with_extension("fresh.rs");
        write_source(&fresh, "x\ny").unwrap();
        assert_eq!(std::fs::read_to_string(&fresh).unwrap(), "x\r\ny");
        let bytes = std::fs::read(&fresh).unwrap();
        assert_ne!(bytes[0], 0xEF, "写出去的文件不许带 BOM");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&fresh);
    }

    /// 单行输入框（颜色面板的 Hex 框用它）：限长、剪贴板带换行要并回一行、
    /// 点一下按字符宽反算列。
    #[test]
    fn single_line_input_limits_joins_and_maps_clicks() {
        let mut input = TextInput::new(HEX_LEN_MAX);
        input.set_text("#2288FF");
        assert_eq!(input.text(), "#2288FF");
        input.set_text("1234567890");
        assert_eq!(input.text(), "1234567", "超过 7 个字符要砍尾巴");
        input.set_text("ab\ncd");
        assert_eq!(input.text(), "abcd", "单行框里不许出现第二行");
        // 点一下定位光标：ASCII 一格 9px、中文一格 18px，取字符中点分界。
        assert_eq!(col_at("abc", 0.0), 0);
        assert_eq!(col_at("abc", 12.0), 1);
        assert_eq!(col_at("中文", 14.0), 1);
        assert_eq!(col_at("ab", 999.0), 2, "点在行尾右边就是行尾");
    }

    /// 编辑器窗口那几条横带都踩在网格上：标题栏 / 菜单栏 / 按钮栏各 2 格，正文区上沿正好
    /// 落在网格线上，正文区高度按 2 格一行分；图标按钮是整数格宽，内容区还比 16x16 图标
    /// 富余 1px（图标绝不压到边框上）；菜单项的文字左右各留 1 格（铁律）。
    #[test]
    fn editor_chrome_sits_on_the_grid() {
        assert_eq!(SRC_MENU_ROWS, TEXT_ROWS, "菜单栏不是 2 格");
        assert_eq!(SRC_BAR_ROWS, TEXT_ROWS, "按钮栏不是 2 格");
        assert_eq!(SRC_HEAD_ROWS, SRC_TITLE_ROWS + TEXT_ROWS + TEXT_ROWS);
        assert_eq!(
            SRC_BODY_TOP as i32,
            grid_line(SRC_HEAD_ROWS) as i32,
            "正文区上沿不在网格线上"
        );
        assert_eq!(SRC_BODY_TOP as i32 % PITCH as i32, 0);
        assert_eq!(
            SRC_ROWS,
            SRC_HEAD_ROWS + SRC_BODY_ROWS + 1,
            "标题栏 / 菜单栏 / 按钮栏 / 正文区 / 底边那条线加起来应该正好是窗口高"
        );

        // 图标按钮：整数格宽，内容区（去掉左右各 1px 边框）放得下 16x16 的图标。
        let w = cells(2);
        assert_eq!(((w - LINE_W) as i32) % PITCH as i32, 0, "按钮宽不是整数格");
        assert!(w - 2.0 * LINE_W >= ICON_W, "内容区塞不下 16x16 图标");
        assert_eq!(
            ((ICON_W + ICON_GAP) as i32) % PITCH as i32,
            0,
            "图标 + 间隔不是整数格"
        );

        let mut app = blank_app();
        assert_eq!(EDITOR_TABS.len(), 3, "编辑器菜单：文件 / 编辑 / 查看");
        assert_eq!(EDITOR_BUTTONS.len(), 9, "按钮栏应该是 9 个图标按钮");
        for tab in EDITOR_TABS {
            let title = tab.title;
            let w = tab_width(&mut app, title);
            assert_eq!(((w - LINE_W) as i32) % PITCH as i32, 0, "菜单项宽不是整数格");
            assert!(
                w - app.text_width(title) >= 2.0 * GRID,
                "菜单「{title}」左右空档不到 1 格"
            );
        }
    }

    /// 悬停便签（快捷按钮 / 菜单按钮统一走 ui::button）：鼠标压上去按钮就把便签记进
    /// StockApp::tooltip，便签是一块 1px 硬边的小牌子，上沿正好压在按钮下沿那条线上 -
    /// 和按钮的下边框共用那 1px，绝不叠成 2px。
    #[test]
    fn hover_tooltip_shares_the_button_bottom_border() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        if app.text_engine.is_none() {
            return;
        }
        // 按钮左上角落在网格线上（tooltip 的 x 也吸到网格线，两者左对齐）。
        let rect = egui::Rect::from_min_size(egui::pos2(90.0, 90.0), egui::vec2(cells(6), BTN_H));
        let content = ButtonContent::Text("测试".to_owned());
        let tip = "按钮含义";
        let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        // 悬停要等按钮登记过一帧之后才认（egui 的命中测试用的是上一帧的矩形）。
        let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
            let mut raw = egui::RawInput::default();
            raw.screen_rect = Some(layout);
            raw.events = events;
            let mut out = ctx.run_ui(raw, |ui| {
                let painter = ui.painter().clone();
                app.draw_button(ui, &painter, rect, &content, false, tip);
                app.paint_tooltip(&painter, layout);
            });
            let shapes = std::mem::take(&mut out.shapes);
            absorb_textures(&mut out.textures_delta);
            rasterize_into(
                &ctx.tessellate(shapes, 1.0),
                WINDOW_W as usize,
                WINDOW_H as usize,
            )
        };
        frame(&mut app, vec![]);
        let grid = frame(&mut app, vec![egui::Event::PointerMoved(rect.center())]);
        let w = snap_cells_up(app.text_width(tip) + 2.0 * TEXT_PAD_X);
        let (x0, x1) = (rect.min.x as usize, (rect.min.x + w) as usize);
        let longest = |row: usize| -> usize {
            let (mut best, mut cur) = (0usize, 0usize);
            for x in x0..x1 {
                cur = if grid[row][x] == Some(C_LINE) { cur + 1 } else { 0 };
                best = best.max(cur);
            }
            best
        };
        // 便签上沿 = 按钮下边框那一行：整宽都是边框色，上面一行是干净底色（合起来只有 1px）。
        let top = (rect.max.y - LINE_W) as usize;
        assert_eq!(longest(top), x1 - x0, "便签上沿没压在按钮下沿那条线上");
        assert!(longest(top - 1) <= 1, "按钮下边框上面一行也画了横线");
        // 便签下边框落在 2 格之后那条网格线上，再往下一行是干净底色。
        let bottom = top + grid_line(TEXT_ROWS) as usize;
        assert_eq!(longest(bottom), x1 - x0, "便签下边框没画出来");
        assert!(longest(bottom + 1) <= 1, "便签下边框下面一行也画了横线");
    }

    /// 铁律（每个控件区的外圈都得有落在网格线上的划分线）：源码编辑器里标题栏 / 菜单栏 / 按钮栏 /
    /// 正文区之间那三条横线都得画出来，整宽、只占 1px - 相邻行是干净底色，不许叠成 2px。
    #[test]
    fn editor_sections_are_split_by_single_grid_lines() {
        let mut app = blank_app();
        app.open_source(PANE_MACD);
        let grid = render_overlays(&mut app);
        let panel = egui::Rect::from_min_size(
            egui::pos2(SRC_DEFAULT_X, SRC_DEFAULT_Y),
            egui::vec2(SRC_W, SRC_H),
        );
        let (x0, x1) = (panel.min.x as usize, panel.max.x as usize);
        // 某一横排里最长的边框色连排（竖边框只有 1px，横分隔线是一长条）。
        let longest = |row: usize| -> usize {
            let (mut best, mut cur) = (0usize, 0usize);
            for x in x0..x1 {
                cur = if grid[row][x] == Some(C_LINE) { cur + 1 } else { 0 };
                best = best.max(cur);
            }
            best
        };
        // 标题栏下沿 + 菜单栏下沿 + 按钮栏下沿，三条都落在横网格线上。
        for rows in [SRC_TITLE_ROWS, SRC_TITLE_ROWS + SRC_MENU_ROWS, SRC_HEAD_ROWS] {
            let y = (panel.min.y + grid_line(rows)) as usize;
            assert_eq!(y % 9, 0, "分隔线 {y} 没落在横网格线上");
            assert!(longest(y) > 100, "第 {rows} 条网格线上没画出分隔线");
            // 上下两行都不许再出一条横线（出了就是 2px 叠线）。
            assert!(longest(y - 1) <= 1, "分隔线 {y} 上面一行也画了横线");
            assert!(longest(y + 1) <= 1, "分隔线 {y} 下面一行也画了横线");
        }
    }

    /// 颜色面板的 Hex 框（通用自绘输入框的第一个用户）：点一下拿到焦点，敲的字进缓冲区并按
    /// 十六进制当场刷 RGB；不合法就只红边、什么颜色都不改。
    #[test]
    fn hex_input_takes_typing_and_validates() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = color_panel_app();
        app.text_engine = HardText::new(&ctx);
        if app.text_engine.is_none() {
            return;
        }
        let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        let id = egui::Id::new("rt_color_hex");
        let inside = egui::pos2(
            CP_DEFAULT_X + 1.0 + 8.0,
            CP_DEFAULT_Y + CP_Y_HEX + 9.0,
        );
        let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
            let mut raw = egui::RawInput::default();
            raw.screen_rect = Some(layout);
            raw.events = events;
            let mut out = ctx.run_ui(raw, |ui| {
                app.ui_color_panel(ui, layout);
            });
            absorb_textures(&mut out.textures_delta);
            let _ = ctx.tessellate(std::mem::take(&mut out.shapes), 1.0);
        };
        let press = vec![
            egui::Event::PointerMoved(inside),
            egui::Event::PointerButton {
                pos: inside,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ];
        let release = vec![egui::Event::PointerButton {
            pos: inside,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }];

        frame(&mut app, vec![]);
        frame(&mut app, vec![egui::Event::PointerMoved(inside)]);
        frame(&mut app, press);
        frame(&mut app, release);
        assert!(ctx.memory(|m| m.has_focus(id)), "点 Hex 框没拿到焦点");

        frame(&mut app, vec![key_event(egui::Key::A, true)]);
        frame(&mut app, vec![egui::Event::Text("#00FF00".to_owned())]);
        assert_eq!(app.color_panel.hex.text(), "#00FF00");
        assert_eq!(app.color_panel.rgb, (0, 255, 0), "合法 hex 没刷 RGB");
        assert!(app.color_panel.hex_valid);

        frame(&mut app, vec![key_event(egui::Key::A, true)]);
        frame(&mut app, vec![egui::Event::Text("#00FF0".to_owned())]);
        assert!(!app.color_panel.hex_valid, "少一位该判非法");
        assert_eq!(app.color_panel.rgb, (0, 255, 0), "非法输入不该改颜色");
    }

    /// 编辑器的接线：在正文区点一下 -> 拿到焦点 -> 敲的字进缓冲区，而且焦点一直留着；
    /// 点窗口外面就把焦点还回去，没焦点时一个键都不该收。
    #[test]
    fn editor_body_takes_focus_and_types() {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        if app.text_engine.is_none() {
            return;
        }
        app.bars = data::random_walk::generate(BAR_SEED, 64, BAR_START);
        app.open_source(PANE_CANDLE);
        let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
        let id = egui::Id::new(EDITOR_ID);
        let inside = egui::pos2(SRC_DEFAULT_X + 40.0, SRC_DEFAULT_Y + SRC_BODY_TOP + 9.0);
        let outside = egui::pos2(SRC_DEFAULT_X - 40.0, SRC_DEFAULT_Y - 40.0);

        let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
            let mut raw = egui::RawInput::default();
            raw.screen_rect = Some(layout);
            raw.events = events;
            let mut out = ctx.run_ui(raw, |ui| {
                app.ui_source_window(ui, layout);
            });
            absorb_textures(&mut out.textures_delta);
            let _ = ctx.tessellate(std::mem::take(&mut out.shapes), 1.0);
        };
        let press = |pos: egui::Pos2| {
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ]
        };
        let release = |pos: egui::Pos2| {
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }]
        };

        // 先空跑一帧：egui 的命中判定用的是【上一帧】登记过的控件，头一帧就按下去的话
        // press 谁也找不到（真窗口里窗口早画过好几帧了）。
        frame(&mut app, vec![]);
        frame(&mut app, vec![egui::Event::PointerMoved(inside)]);
        frame(&mut app, press(inside));
        frame(&mut app, release(inside));
        assert!(ctx.memory(|m| m.has_focus(id)), "点了正文区没拿到焦点");

        // 隔几帧再敲字：焦点必须一直留着 - egui 每帧都把「这一帧没登记过的」控件踢出焦点，
        // 正文区得每帧拿同一个 id 登记自己（ui.interact，不是 allocate_rect）。
        let col = app.source.editor.buf.cursor;
        for _ in 0..3 {
            frame(&mut app, vec![]);
        }
        assert!(ctx.memory(|m| m.has_focus(id)), "点完之后焦点没留住");
        frame(&mut app, vec![egui::Event::Text("xy".to_owned())]);
        assert_eq!(
            app.source.editor.buf.cursor,
            (col.0, col.1 + 2),
            "拿到焦点之后敲的字没进缓冲区"
        );

        // 点窗口外面：焦点还回去，再敲字就不该动了。
        frame(&mut app, press(outside));
        frame(&mut app, release(outside));
        assert!(!ctx.memory(|m| m.has_focus(id)), "点窗口外面焦点没还回去");
        let text = app.source.editor.buf.to_text();
        frame(&mut app, vec![egui::Event::Text("z".to_owned())]);
        assert_eq!(app.source.editor.buf.to_text(), text, "没焦点还在收键盘事件");
    }

    /// 编辑窗开着某个 Pane、那个 Pane 又换了指标：缓冲区必须整份换成新指标那一份。
    /// 保存是按「当前指标的 path()」写盘的 - 缓冲区不换，就会把旧指标的文字写进新指标的文件里。
    #[test]
    fn switching_indicator_reloads_the_open_editor() {
        let mut app = blank_app();
        app.open_source(PANE_MACD);
        assert!(app.source.editor.buf.to_text().contains("struct Macd"));
        app.set_pane_indicator(PANE_MACD, 4);
        assert!(
            app.source.editor.buf.to_text().contains("struct Kdj"),
            "换了指标，编辑窗里的缓冲区还是旧指标的"
        );
        assert_eq!(app.source.pane, PANE_MACD);
        assert_eq!(
            app.source.editor.buf.to_text(),
            app.panes[PANE_MACD].indicator.source().replace("\r\n", "\n"),
            "缓冲区和当前指标的源码对不上"
        );
    }

    /// 再点一下「代码」不该把手里这份改动冲掉（同一个 Pane 只把窗口提到最前）；
    /// 换到别的 Pane 才重新灌一份源码。
    #[test]
    fn reopening_the_same_pane_keeps_the_edits() {
        let mut app = blank_app();
        app.open_source(PANE_CANDLE);
        app.source.editor.buf.insert_text("// 我改过", &char_allowed);
        app.open_source(PANE_CANDLE);
        assert!(
            app.source.editor.buf.to_text().contains("我改过"),
            "同一个 Pane 再点一次「代码」把改动冲掉了"
        );
        app.open_source(PANE_MACD);
        assert!(
            app.source.editor.buf.to_text().contains("struct Macd"),
            "换了 Pane 该重新灌一份源码"
        );
    }

    /// 超长行横向截断（不做软换行、不横向滚动）：截出来的宽度必须还在限宽里。
    #[test]
    fn long_lines_are_truncated_to_fit() {
        let long = "a".repeat(200);
        let cut = fit_line(&long, 90.0);
        assert!(text_advance(&cut) <= 90.0, "截断之后还是超宽");
        assert_eq!(cut.chars().count(), 10, "90px 只放得下 10 个 ASCII 格");
        assert_eq!(fit_line("short", 90.0), "short", "放得下就原样返回");
        let mixed = fit_line("中文中文", 90.0);
        assert_eq!(mixed, "中文中文", "90px 放得下 4 个中文（72px）");
        let narrow = fit_line("中文中文", 60.0);
        assert_eq!(narrow, "中文中", "60px 只放得下 3 个中文");
    }
}
