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
//  圆桌 RoundTable —— Windows 桌面股票分析软件
//  当前进度：9px 节距（8px 内容格 + 1px 网格线格）+ 无边框窗口；
//            标题栏 / 菜单栏 / 功能区（左栏蜡烛图 + 右栏控制面板）/ 状态栏；
//            K 线支持滚轮缩放（锚点缩放，整数倍跳变）+ 左键拖拽平移。
//  文件结构（一个模块一个文件，都在 crate 内可见）：
//    main.rs            程序入口 + 单元测试
//    grid.rs            度量常数 + 像素级绘制工具
//    theme.rs           颜色表
//    font.rs            硬边文字引擎（Windows GDI 1bpp 字形）
//    app.rs             StockApp 状态、每帧 ui() 驱动、全局样式
//    win32.rs           窗口杂项（去掉系统画在窗口外圈的装饰）
//    ui/titlebar.rs     标题栏（程序名 + 系统按钮 + 拖动热区）
//    ui/menubar.rs      菜单栏与下拉菜单
//    ui/chart.rs        K 线区 / 成交量区 / 指标区 / 时间轴
//    ui/panel.rs        两栏布局、分割条、右侧控制面板
//    ui/colorpanel.rs   涨跌颜色与自绘颜色面板
//    data/              行情数据层
//    indicator/         指标层（纯计算，不依赖界面）
// ===========================================================================
//  窗口策略：无边框，开机 cells(125) x cells(67) = 1126x604；尺寸在启动时定死，不做拖动缩放，
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
//                   绝对值大的方向舍入），画在 x = 0 / y = 0 的线会整条跑到窗口外 ——
//                   四条边框线只剩右、下两条看得见。
//    窗口外圈装饰 : 自己调 user32：清 WS_CAPTION / WS_EX_*EDGE 样式位（去 DWM 投影），
//                   再在窗口过程里拦 WM_NCCALCSIZE 把非客户区压成 0（去窗口顶上那条
//                   1px 系统边框，见第十三节）—— egui/winit 都没有现成开关
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
//  十六、单元测试
// ===========================================================================
//  只测「画出来像素级对不对称」这类眼睛容易漏、算起来一清二楚的事。
//  做法：起一个真的 egui Context（不用窗口、不用 GPU），把一根蜡烛交给真的 epaint
//  光栅化管线（TessellationOptions 跟运行时完全一致：关抗锯齿、吸附像素栅格），
//  再把三角形按「像素中心是否落在里面」填进网格 —— 关掉抗锯齿后显卡干的就是这件事，
//  所以这张网格就是屏幕上真正看到的样子，逐列数得清清楚楚。

#[cfg(test)]
mod tests {
    use super::*;
    
    
    use crate::ui::panel::*;
    
    use crate::ui::colorpanel::*;
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
        out.textures_delta.clear();
        rasterize(&ctx.tessellate(shapes, 1.0))
    }

    /// 三角形 (a, b, c) 的有向面积（叉积的 z 分量）。
    fn edge(a: egui::Pos2, b: egui::Pos2, c: egui::Pos2) -> f32 {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    }

    /// 测试用的最小 StockApp：只为了能调 ui_color_panel / ui_kline / ui_volume。
    /// 文字引擎留空（测试里不画字），行情数据也留空（要用的话测试自己填），其余字段给个能过的初值。
    fn blank_app() -> StockApp {
        StockApp {
            hwnd: 0,
            client_cover_ready: false,
            open_menu: None,
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
            data_gen: 1,
            view_cache: ViewCache::invalid(),
            kline_h: KLINE_H_DEFAULT,
            vol_h: VOL_H_DEFAULT,
            ind_h: IND_H_DEFAULT,
            hsplit_grab: None,
            vsplit_grab: None,
            indicators: IndicatorPane::new(),
            split_x: CHART_PANE_W,
            split_grab: None,
            panel_buttons: Vec::new(),
            panel_press: None,
            up_color: UP_RGB_DEFAULT,
            down_color: DOWN_RGB_DEFAULT,
            color_panel: ColorPanel::new(),
        }
    }

    /// 上面那个再打开颜色面板（面板相关的测试都用它）。
    fn color_panel_app() -> StockApp {
        let mut app = blank_app();
        app.color_panel.open = true;
        app.color_panel.pos = egui::pos2(CP_DEFAULT_X, CP_DEFAULT_Y);
        app.color_panel.load((34, 136, 255));
        app
    }

    /// 把颜色面板画在整窗画布（1126x604）上再光栅化：画布尺寸 = 窗口尺寸（1 逻辑点 = 1 像素）。
    fn render_panel(app: &mut StockApp) -> Vec<Vec<Option<egui::Color32>>> {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
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
        out.textures_delta.clear();
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
                if edge(a.pos, b.pos, c.pos) == 0.0 {
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

    /// 颜色面板的尺寸与栅格：本体 224x312（都是 8 的倍数），SV 调色板正好 26x16 个 8x8 色块。
    #[test]
    fn color_panel_sits_on_the_8px_grid() {
        assert_eq!(CP_W as i32, 224, "面板宽");
        assert_eq!(CP_H as i32, 312, "面板高");
        assert_eq!(CP_W as i32 % 8, 0, "面板宽必须是 8 的倍数");
        assert_eq!(CP_H as i32 % 8, 0, "面板高必须是 8 的倍数");
        assert_eq!(SV_COLS * SV_ROWS, 416, "SV 调色板应该是 26x16 块");
        assert_eq!(SV_BLOCK as i32, 8, "每块应该是 8x8");
        assert_eq!(SV_COLS as f32 * SV_BLOCK, CP_CONTENT_W);
        assert_eq!(SV_ROWS as f32 * SV_BLOCK, CP_SV_H);
        assert_eq!(CP_TRACK_W as i32, 152, "滑块轨道宽");
        assert_eq!(CP_HANDLE_TRAVEL as i32, 144, "把手能走的距离（整数像素）");
        assert_eq!(CP_HALF_W as i32, 104, "底部两个按钮各占一半");
        // 中间那条共用的竖线落在正中间的网格线上（104 = 13 格）；左半边要盖住它，所以是 105px。
        assert_eq!(CP_LEFT_W as i32, 105, "左半边（含中间共用的 1px 线格）");
        assert_eq!(CP_HALF_W as i32 % 8, 0, "中间那条竖线必须落在 8 的倍数上");
        assert_eq!(CP_LEFT_W + CP_HALF_W - LINE_W, CP_CONTENT_W);
        // Hex 输入框的矩形：3 个内容格 + 两端各 1px 线格 = 25px；下边框正好压在 CP_Y_BTN
        // 这条横网格线上（264 + 25 - 1 = 288 = CP_Y_BTN）。
        assert_eq!(CP_HEX_BOX_H as i32, 25, "Hex 输入框矩形高（含两端线格）");
        assert_eq!(
            (CP_Y_HEX + CP_HEX_BOX_H - LINE_W) as i32,
            CP_Y_BTN as i32,
            "Hex 输入框下边框没压在 CP_Y_BTN 这条网格线上"
        );
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
        assert_eq!(bar_of_hue(0), 0);
        assert_eq!(bar_of_hue(360), HUE_BARS - 1);
        assert_eq!(bar_value(0, 255), 0);
        assert_eq!(bar_value(CP_TRACK_W as i32 - 1, 255), 255);
        assert_eq!(bar_value(CP_TRACK_W as i32 - 1, 360), 360);
    }

    /// 面板真画出来的样子：本体正好 224x312（最外圈那一圈像素是黑的），SV 调色板逐块点名
    /// （26x16 块、每块 8x8，相邻两块颜色必须不同 —— 证明是一块一块拼的，不是平滑渐变），
    /// 色相条逐根点名（208 根 1px 竖条，每根都是它自己那个色相角的纯色）。
    #[test]
    fn color_panel_paints_blocks_and_bars() {
        let mut app = color_panel_app();
        let grid = render_panel(&mut app);

        let px = CP_DEFAULT_X as usize;
        let py = CP_DEFAULT_Y as usize;
        let (w, h) = (CP_W as usize, CP_H as usize);

        // 外圈那一圈像素是 1px 硬边（占的是网格线格，不属于本体）。
        assert_eq!(grid[py][px], Some(C_BORDER), "面板左上角应该是边框");
        assert_eq!(grid[py][px + w - 1], Some(C_BORDER), "面板右上角应该是边框");
        assert_eq!(
            grid[py + h - 1][px + w - 1],
            Some(C_BORDER),
            "面板右下角应该是边框"
        );
        // 边框再往外一个像素什么都没画：本体尺寸正好是 w x h。
        assert_eq!(grid[py + 100][px + w], None, "面板右边多画了一个像素");
        assert_eq!(grid[py + h][px + 100], None, "面板下边多画了一个像素");
        // 最上面那条是灰标题栏（横贯整个面板宽），往下正文是工作区白。
        assert_eq!(grid[py + 4][px + 2], Some(C_TITLE_BG), "标题栏底色");
        assert_eq!(
            grid[py + CP_Y_SV as usize + 2][px + 2],
            Some(C_WORK_BG),
            "正文那块应该是工作区白"
        );

        // SV 调色板：逐块点名（取块中心，避开块边框和当前位置的标记）。
        let sx = px + CP_PAD as usize;
        let sy = py + CP_Y_SV as usize;
        for row in 0..SV_ROWS {
            for col in 0..SV_COLS {
                let want = rgb32(hsv_to_rgb(app.color_panel.h, s_of_col(col), v_of_row(row)));
                let x = sx + col as usize * SV_BLOCK as usize + 4;
                let y = sy + row as usize * SV_BLOCK as usize + 4;
                assert_eq!(grid[y][x], Some(want), "SV 块 ({col}, {row}) 的颜色不对");
            }
        }
        // 同一行里相邻两块颜色必须不同：证明是 8px 一块一块拼的，不是平滑渐变。
        for col in 0..SV_COLS - 1 {
            // 取第 col 块的最后一列和下一块的第一列，正好跨在块的分界上。
            let x = sx + (col as usize + 1) * SV_BLOCK as usize - 1;
            assert_ne!(
                grid[sy + 4][x],
                grid[sy + 4][x + 1],
                "第 {col} 块和下一块颜色一样"
            );
        }

        // 色相条：每根 1px 竖条都应该是它自己那个色相角的纯色（跳过被把手压住的那 8 根）。
        let hx = px + CP_PAD as usize;
        let hy = py + CP_Y_HUE as usize;
        let handle_left = hue_handle_left(bar_of_hue(app.color_panel.h));
        let covered = handle_left as i32..handle_left as i32 + CP_HUE_HANDLE_W as i32;
        for i in 0..HUE_BARS {
            if covered.contains(&i) {
                continue;
            }
            let want = rgb32(hsv_to_rgb(hue_of_bar(i), 255, 255));
            assert_eq!(
                grid[hy + 8][hx + i as usize],
                Some(want),
                "第 {i} 根色相条的颜色不对"
            );
        }
    }

    /// 把手只停在整像素列上：0..=255（R / G / B / S / V）和 0..=360（H）每个取值都过一遍，
    /// 左边缘不许出现小数，也不许越出轨道那 144px 的可走范围。
    #[test]
    fn slider_handles_land_on_whole_pixels() {
        // 轨道起点 = 面板左边缘 + 内边距 + 标签宽 + 间距，四个都是整数常量。
        let track_x = CP_DEFAULT_X + CP_PAD + CP_SLIDER_LABEL_W + CP_SLIDER_GAP;
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
    /// （它同时是底部按钮的顶边）这两条横网格线 —— 上边不能少画（被分隔线盖成灰的），
    /// 下边也不能和旁边那条线叠成 2px。非法输入时整圈变红，红边同样压在网格线上。
    #[test]
    fn hex_box_frame_sits_on_grid_lines() {
        let px = CP_DEFAULT_X as usize;
        let py = CP_DEFAULT_Y as usize;
        let x = px + CP_PAD as usize + 20;
        let top = py + CP_Y_HEX as usize;
        let bottom = py + CP_Y_BTN as usize;

        // 合法：上下两条边都是分隔线那个灰，上下各一个像素都是干净的（不多出一条线）。
        let mut app = color_panel_app();
        let grid = render_panel(&mut app);
        assert_eq!(grid[top][x], Some(C_BTN_EDGE), "上边框没压在网格线上");
        assert_eq!(grid[bottom][x], Some(C_BTN_EDGE), "下边框没压在网格线上");
        assert_eq!(grid[top - 1][x], Some(C_WORK_BG), "上边框上面还多了一条线");
        assert_eq!(grid[bottom - 1][x], Some(C_WORK_BG), "下边框上面还多了一条线");
        assert_eq!(grid[bottom + 1][x], Some(C_BTN_FACE), "下边框下面还多了一条线");

        // 非法：整圈变红，而且红边一样压在网格线上（不会被分隔线盖成灰的）。
        let mut app = color_panel_app();
        app.color_panel.hex = "#2288F".to_owned();
        let grid = render_panel(&mut app);
        assert_eq!(grid[top][x], Some(C_CLOSE_HOVER), "非法时上边框没变红");
        assert_eq!(grid[bottom][x], Some(C_CLOSE_HOVER), "非法时下边框没变红");
        assert_eq!(grid[bottom - 1][x], Some(C_WORK_BG), "非法时下边框上面还有一条线");
    }

    /// 竖着的线也必须对齐：内容列最右那一条竖线 —— Hex 输入框的右边框、「取消」按钮的右边框、
    /// SV 调色板 / 色相条的右边缘 —— 全得落在同一列上；中间那条共用的竖线只能占 1px（左右两半共用）。
    /// 这一条就是「8px 格 + 1px 网格线」在横向上的体现：右半边矩形比左半边多那 1px，
    /// 多出来的正是中间那条共用的线格。
    #[test]
    fn panel_right_edges_line_up() {
        let mut app = color_panel_app();
        let grid = render_panel(&mut app);

        let px = CP_DEFAULT_X as usize;
        let py = CP_DEFAULT_Y as usize;
        let left = px + CP_PAD as usize;
        // 内容列最后一列：26 格 = 208px，相对第 207 列。
        let right = left + CP_CONTENT_W as usize - 1;
        // 各取一段的中间某一行，避开横向的分隔线。
        let hex_row = py + CP_Y_HEX as usize + 8;
        let btn_row = py + CP_Y_BTN as usize + 8;
        let hue_row = py + CP_Y_HUE as usize + 8;

        // Hex 输入框：右边框压在这一列上，再往右一格是面板底色（不能多画一列）。
        assert_eq!(
            grid[hex_row][right],
            Some(C_BTN_EDGE),
            "Hex 输入框右边框不在内容列最后一列上"
        );
        assert_eq!(
            grid[hex_row][right + 1],
            Some(C_WORK_BG),
            "Hex 输入框右边框右边多了一列"
        );
        // 取消按钮：右边框和 Hex 输入框在同一列（就是用户看见那条错位）。
        assert_eq!(
            grid[btn_row][right],
            Some(C_BTN_BORDER),
            "取消按钮右边框和 Hex 输入框没对齐"
        );
        // 色相条：横贯整条内容列，最右那根 1px 竖条也要压在这一列上，右边不能留白、也不能多铺出去。
        assert_ne!(grid[hue_row][right], Some(C_WORK_BG), "色相条右边留了白边");
        assert_eq!(
            grid[hue_row][right + 1],
            Some(C_WORK_BG),
            "色相条右边多铺了一列"
        );

        // 中间那条共用竖线：左半边（应用）的右边框和右半边（取消）的左边框是同一条，只有 1px，
        // 而且必须落在 8px 网格线上（内容列一半 = 13 格 = 104px，就是正中间那条线）。
        let mid = left + CP_HALF_W as usize;
        assert_eq!((mid - px) % 8, 0, "中间那条共用的竖线没落在 8px 网格线上");
        assert_eq!(grid[btn_row][mid], Some(C_BTN_BORDER), "中间那条竖线不见了");
        assert_eq!(grid[btn_row][mid - 1], Some(C_BTN_FACE), "中间那条竖线不止 1px");
        assert_eq!(grid[btn_row][mid + 1], Some(C_BTN_FACE), "中间那条竖线不止 1px");

        // 左边框同样要压在内容列第一列上。
        assert_eq!(
            grid[btn_row][left],
            Some(C_BTN_BORDER),
            "应用按钮左边框不在内容列第一列上"
        );
        assert_eq!(
            grid[hex_row][left],
            Some(C_BTN_EDGE),
            "Hex 输入框左边框不在内容列第一列上"
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
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(WINDOW_W, canvas_h as f32),
        ));
        let mut out = ctx.run_ui(raw, |ui| {
            app.ui_kline(ui, kline);
            app.ui_volume(&ui.painter().clone(), volume);
            app.ui_indicator(&ui.painter().clone(), indicator);
            app.ui_hsplitter(ui, bar, work, LeftSplit::Volume);
            app.ui_hsplitter(ui, bar2, work, LeftSplit::Indicator);
        });
        let shapes = std::mem::take(&mut out.shapes);
        out.textures_delta.clear();
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
    /// 同一个颜色的像素列集合，两边必须完全相等 —— 只要有一根柱子偏 1px，集合立刻对不上。
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
        let (kplot, _) = StockApp::chart_rects(StockApp::kline_layers(kline)[1]);
        let (vplot, _) = StockApp::chart_rects(volume);
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

    /// 左栏上下五块的几何：三块高度都是 8 的倍数、每两块之间夹着 1px + 8px + 1px，
    /// 五块落位都在整数像素上；指标区在底部把 K 线区吸零头剩下的 0..7px 吃掉，
    /// 所以它一路铺到左栏底部（不留白条），画出来的高度只比 ind_h 多 0..7px。
    /// 拖动 / 换窗口尺寸之后也还是这个规矩，且三块各自不低于自己的最小高度。
    #[test]
    fn left_column_splits_on_8px_grid() {
        assert_eq!(KLINE_H_DEFAULT as i32, 600, "K 线区默认 600px");
        assert_eq!(KLINE_H_MIN as i32, 240, "K 线区最小 240px");
        assert_eq!(VOL_H_DEFAULT as i32, 96, "成交量区默认 96px");
        assert_eq!(VOL_H_MIN as i32, 64, "成交量区最小 64px");
        assert_eq!(IND_H_DEFAULT as i32, 96, "指标区默认 96px");
        assert_eq!(IND_H_MIN as i32, 64, "指标区最小 64px");
        assert_eq!(HSPLIT_H as i32, 8, "横向拖拽条 8px");

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
            (64.0, 5000.0),
            (5000.0, 64.0),
        ] {
            app.vol_h = want_vol;
            app.ind_h = want_ind;
            app.clamp_left_heights(work);
            let [kline, bar, vol, bar2, ind] = StockApp::left_rows(work, app.kline_h, app.vol_h);
            assert_eq!(app.vol_h % 8.0, 0.0, "成交量区高度不是 8 的倍数");
            assert_eq!(app.ind_h % 8.0, 0.0, "指标区高度不是 8 的倍数");
            assert_eq!(kline.height() % 8.0, 0.0, "K 线区高度不是 8 的倍数");
            assert_eq!(bar.height(), HSPLIT_H, "上面那条拖拽条高度不是 8px");
            assert_eq!(bar2.height(), HSPLIT_H, "下面那条拖拽条高度不是 8px");
            assert!(kline.height() >= KLINE_H_MIN, "K 线区低于最小高度");
            assert!(vol.height() >= VOL_H_MIN, "成交量区低于最小高度");
            assert!(ind.height() >= IND_H_MIN, "指标区低于最小高度");
            // 量区高度就是收口后的 vol_h；只有最下面的指标区多吸收那 0..7px 的零头。
            assert_eq!(vol.height(), app.vol_h);
            assert!(ind.height() >= app.ind_h && ind.height() < app.ind_h + 8.0);
            // 从上往下严丝合缝：K 线区 / 1px / 拖拽条 / 1px / 成交量区 / 1px / 拖拽条 / 1px / 指标区。
            assert_eq!(kline.min.y, work.min.y);
            assert_eq!(bar.min.y, kline.max.y + LINE_W);
            assert_eq!(vol.min.y, bar.max.y + LINE_W);
            assert_eq!(bar2.min.y, vol.max.y + LINE_W);
            assert_eq!(ind.min.y, bar2.max.y + LINE_W);
            // 指标区正好铺到左栏底部：底下不留白条，五块加起来不多不少。
            assert_eq!(ind.max.y, work.max.y, "指标区没铺到左栏底部");
            assert_eq!(
                kline.height() + LEFT_FIXED_H + vol.height() + ind.height(),
                work.height(),
                "五块加起来不等于左栏高度"
            );
            for r in [kline, bar, vol, bar2, ind] {
                assert!(r.min.x.fract() == 0.0 && r.min.y.fract() == 0.0);
            }
        }
        // 默认窗口高度下：成交量和指标各保持规格里的 96px，K 线区拿剩下的 304px（38 格）。
        let mut app = blank_app();
        app.clamp_left_heights(work);
        assert_eq!(app.vol_h, VOL_H_DEFAULT);
        assert_eq!(app.ind_h, IND_H_DEFAULT);
        assert_eq!(app.kline_h, 304.0, "默认窗口下 K 线区应该是 304px（38 格）");
    }

    /// 上下三块之间的线格：每条都只有 1px 厚、各占自己那一行，既不缺一条也不叠成 2px；
    /// 拖拽条本体正好 8px，不会被旁边那两条线啃掉像素。
    #[test]
    fn left_column_lines_are_1px() {
        let mut app = blank_app();
        let (grid, [kline, bar, volume, bar2, indicator]) = render_left(&mut app);
        let x = 100usize; // 左栏正中某一列：不落在任何竖线上

        // K 线区内部：信息栏下沿、时间轴上沿，各 1px（上下邻居都是干净的白底）。
        let [info, main, time] = StockApp::kline_layers(kline);
        assert_eq!(grid[info.max.y as usize][x], Some(C_LINE), "信息栏下沿那条线不对");
        assert_eq!(grid[info.max.y as usize - 1][x], Some(C_WORK_BG), "信息栏下沿多画了一行");
        assert_eq!(grid[main.min.y as usize][x], Some(C_CHART_BG), "主图没紧跟着信息栏");
        assert_eq!(grid[time.min.y as usize - 1][x], Some(C_LINE), "时间轴上沿那条线不对");
        assert_eq!(grid[time.min.y as usize][x], Some(C_WORK_BG), "时间轴没紧跟着那条线");

        // K 线区与拖拽条之间：线 1px、本体 8px、再一条线 1px，然后立刻是成交量区。
        let top = kline.max.y as usize;
        assert_eq!(bar.height() as usize, 8, "拖拽条本体不是 8px");
        assert_eq!(grid[top][x], Some(C_BTN_BORDER), "K 线区下沿那条线不对");
        assert_eq!(grid[top - 1][x], Some(C_WORK_BG), "K 线区下沿那条线多画了一行");
        for r in 1..=8 {
            assert_eq!(grid[top + r][x], Some(C_SPLITTER), "拖拽条第 {r} 行不是本体色");
        }
        assert_eq!(grid[top + 9][x], Some(C_BTN_BORDER), "拖拽条下沿那条线不对");
        assert_eq!(grid[top + 10][x], Some(C_CHART_BG), "成交量区没紧跟着拖拽条");
        assert_eq!(volume.min.y as usize, top + 10);

        // 成交量区与下面那条拖拽条之间：一样是线 1px、本体 8px、再一条线 1px，然后立刻是指标区。
        let bottom = volume.max.y as usize;
        assert_eq!(bar2.height() as usize, 8, "下面那条拖拽条本体不是 8px");
        assert_eq!(grid[bottom][x], Some(C_BTN_BORDER), "成交量区下沿那条线不对");
        assert_eq!(grid[bottom - 1][x], Some(C_CHART_BG), "成交量区下沿多画了一行");
        for r in 1..=8 {
            assert_eq!(grid[bottom + r][x], Some(C_SPLITTER), "下面那条拖拽条第 {r} 行不是本体色");
        }
        assert_eq!(grid[bottom + 9][x], Some(C_BTN_BORDER), "下面那条拖拽条下沿不对");
        assert_eq!(grid[bottom + 10][x], Some(C_CHART_BG), "指标区没紧跟着拖拽条");
        assert_eq!(indicator.min.y as usize, bottom + 10);
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

    /// 成交量刻度 / 信息栏用的单位：1 亿以上走「亿」、1 万以上走「万」，都保留 1 位小数。
    #[test]
    fn info_bar_items_are_multiples_of_8() {
        // 用真的硬边文字引擎量（GDI + 宋体 12px）：宽度必须是 8 的倍数，加起来还得放得下左栏。
        let ctx = egui::Context::default();
        let mut app = blank_app();
        app.text_engine = HardText::new(&ctx);
        if app.text_engine.is_none() {
            return; // 没拿到 GDI 字体就量不出宽度，跳过（正常 Windows 上不会走到这儿）
        }
        let mut total = 0.0;
        for i in 0..INFO_LABELS.len() {
            let w = app.info_item_w(i);
            assert_eq!(w as i32 % 8, 0, "第 {i} 项「{}」的宽度不是 8 的倍数：{w}", INFO_LABELS[i]);
            total += w;
        }
        assert!(
            total + GRID <= CHART_PANE_W,
            "信息栏八项加起来 {total}px，放不下左栏 {CHART_PANE_W}px"
        );
        // 时间轴标注间隔用真字体量出来的话也够宽（相邻标签中心至少 48px）。
        for scale in [SCALE_MIN, 5, SCALE_MAX] {
            let advance = bar_advance(scale);
            assert!(app.time_step(advance) as f32 * advance >= TIME_LABEL_GAP_MIN);
        }
    }

    /// 成交量刻度 / 信息栏用的单位：1 亿以上走「亿」、1 万以上走「万」，都保留 1 位小数。
    #[test]
    fn volume_text_uses_wan_and_yi() {
        assert_eq!(fmt_volume(1.5e8), "1.5亿");
        assert_eq!(fmt_volume(1.0e8), "1.0亿");
        assert_eq!(fmt_volume(2.3456e7), "2345.6万");
        assert_eq!(fmt_volume(1.0e4), "1.0万");
        assert_eq!(fmt_volume(9999.0), "9999");
        assert_eq!(fmt_volume(0.0), "0");
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

        // 功能区变高：K 线区先长（它拿剩下的空间），成交量区和指标区保持规格里的 96px；底部不留白条。
        let mut base_app = blank_app();
        base_app.clamp_left_heights(a[2]);
        let mut big_app = blank_app();
        big_app.clamp_left_heights(b[2]);
        assert_eq!(base_app.kline_h, 304.0, "默认窗口下 K 线区应该是 304px");
        assert_eq!(base_app.vol_h, VOL_H_DEFAULT, "成交量区默认 96px");
        assert_eq!(base_app.ind_h, IND_H_DEFAULT, "指标区默认 96px");
        assert_eq!(big_app.kline_h % 8.0, 0.0);
        assert_eq!(big_app.vol_h, VOL_H_DEFAULT, "成交量区不该被窗口高度改掉");
        assert_eq!(big_app.ind_h, IND_H_DEFAULT, "指标区不该被窗口高度改掉");
        assert!(
            big_app.kline_h > base_app.kline_h,
            "窗口拉高后 K 线区没跟着变高：{} -> {}",
            base_app.kline_h,
            big_app.kline_h
        );
        let [_k, _bar, _vol, _bar2, ind] =
            StockApp::left_rows(b[2], big_app.kline_h, big_app.vol_h);
        assert_eq!(ind.max.y, b[2].max.y, "指标区没铺到左栏底部");
    }

    /// 把拖拽条往上拖（成交量区变高）时里面的量柱跟着拉伸：柱子从量区底部的 8px 边距起，
    /// 高度随量区高度走 —— 不是固定高度的一小段。
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
        // 指标区压到最小，把纵向空间尽量让给成交量区，好把「量区拉高」这件事测出来。
        app.ind_h = IND_H_MIN;
        app.vol_h = VOL_H_MIN;
        let (grid_short, [_, _, vol_short, _, _]) =
            render_left_in(&mut app, work, WINDOW_H as usize);
        let short_px = count_down(&grid_short, vol_short);

        app.vol_h = 240.0;
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
        // 底部边距（量区底 - 8px）—— 量区一拉高，柱子的两个端点跟着走，这就是拉伸。
        let (vplot, _) = StockApp::chart_rects(vol_tall);
        let x = vplot.min.x as usize + 6 * bar_advance(SCALE_MIN) as usize;
        let top = vol_tall.min.y as usize + VOL_PAD_Y as usize;
        let base = vol_tall.max.y as usize - VOL_PAD_Y as usize;
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
        assert_eq!(
            rows[rows.len() - 1],
            base - 1,
            "量柱没落到下边距（量区底 - 8px）"
        );
    }

    /// 价格轴那一条灰底上下贯通：K 线区里时间轴下面吸零头剩下的那几行也得是轴灰，
    /// 不能露出白底（以前右侧就在这儿留了一条白色横条）。
    #[test]
    fn axis_column_fills_to_the_kline_bottom() {
        let mut app = blank_app();
        let (grid, [kline, _bar, _vol, _bar2, _ind]) = render_left(&mut app);
        let [info, main, _time] = StockApp::kline_layers(kline);
        let (_plot, axis) = StockApp::chart_rects(main);
        // 这条用例靠的是「时间轴下面还剩几行」—— K 线区高度不是 9 的倍数时会剩 0..7px。
        assert!(info.max.y < main.min.y, "信息栏和主图之间没留线格");
        for y in main.min.y as usize..kline.max.y as usize {
            for x in axis.min.x as usize..axis.max.x as usize {
                assert_ne!(
                    grid[y][x],
                    Some(C_WORK_BG),
                    "价格轴列 ({x}, {y}) 露出了白底：右侧灰底没上下贯通"
                );
            }
        }
        // 信息栏那一行整条都是工作区白（包括最右边那几列）—— 灰底只从主图顶端开始。
        assert_eq!(
            grid[info.max.y as usize - 1][axis.min.x as usize + 4],
            Some(C_WORK_BG),
            "信息栏那一行本来就该是白的"
        );
    }
}
