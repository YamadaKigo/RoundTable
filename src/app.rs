// ===========================================================================
//  模块 app：应用状态与主循环
// ===========================================================================
//  StockApp 的全部状态、每帧的 ui() 驱动、全局样式与渲染开关、窗口拖动。
//  具体界面块在 src/ui/ 里，这里只管状态与调度。
//  本文件 = 拆分前那份单文件里的「七、应用状态」+「十一、界面」+「十二、全局样式」三节。
use crate::data;
use crate::indicator;
use crate::font::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::colorpanel::*;
use crate::ui::menubar::*;
use crate::ui::panel::*;
use crate::win32::*;
use eframe::egui;
use raw_window_handle::HasWindowHandle;
use std::time::{Duration, Instant};

/// 程序名：标题栏上画的就是它，窗口标题（任务栏）也用它。
pub(crate) const APP_NAME: &str = "圆桌";

// ===========================================================================
//  七、应用状态
// ===========================================================================

/// 一次「自己实现」的窗口拖动。全程用屏幕坐标：窗口一边拖一边动，
/// 鼠标在窗口内的坐标会跟着漂，只有屏幕坐标才不会形成反馈回路（越拖越飘）。
#[derive(Clone, Copy)]
pub(crate) struct MoveSession {
    pub(crate) start_ptr: egui::Pos2,
    pub(crate) start_outer: egui::Rect,
    pub(crate) sent_pos: Option<egui::Pos2>,
}

/// 十字光标的状态：鼠标在绘图区里的位置（已经吸附到像素中心），
/// 以及由它反算出来的价格和 K 线索引（两个标签要显示这两个数）。
#[derive(Clone, Copy)]
pub(crate) struct Crosshair {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) price: f64,
    pub(crate) index: usize,
}

/// 映射结果缓存：K 线区的价格上下界 + 成交量区的最大量。
/// 只要「数据代次 / offset / visible / scale」四个键没变，就不重算（规格：数据不变不重算）。
/// 数据代次是必须的：换一段行情时 bars.len() 前后往往一样，光看长度会拿着旧范围接着画。
#[derive(Clone, Copy)]
pub(crate) struct ViewCache {
    pub(crate) generation: u64,
    pub(crate) offset: usize,
    pub(crate) visible: usize,
    pub(crate) scale: u32,
    pub(crate) price_hi: f64,
    pub(crate) price_lo: f64,
    pub(crate) vol_max: f64,
}

impl ViewCache {
    /// 一个必然失效的初值：gen 给 u64::MAX，第一次刷新一定重算。
    pub(crate) const fn invalid() -> Self {
        Self {
            generation: u64::MAX,
            offset: usize::MAX,
            visible: 0,
            scale: 0,
            price_hi: 0.0,
            price_lo: 0.0,
            vol_max: 0.0,
        }
    }
}

/// 指标区的跨帧状态：可选指标表 + 当前选中 + 算好的结果缓存。
/// 缓存键是「数据代次 + 指标 id」：数据没换代、指标也没换，就一次都不重算 ——
/// 指标要扫整段 K 线（几百根），不能每帧算一遍。
pub(crate) struct IndicatorPane {
    /// 可选指标。现在是内置的 MACD；以后加指标（含自定义 / 可编程指标）就往这张表里塞。
    pub(crate) list: Vec<Box<dyn indicator::Indicator>>,
    /// 当前选中的下标，永远落在 list 里。
    pub(crate) selected: usize,
    /// 结果缓存：(数据代次, 指标 id, 输出)。
    pub(crate) cache: Option<(u64, &'static str, indicator::Output)>,
}

impl IndicatorPane {
    pub(crate) fn new() -> Self {
        Self {
            list: indicator::builtin(),
            selected: 0,
            cache: None,
        }
    }

    /// 拿这一代数据下当前指标的输出，必要时重算。
    pub(crate) fn output(
        &mut self,
        generation: u64,
        bars: &[data::random_walk::Bar],
    ) -> &indicator::Output {
        let want = self.list[self.selected].id();
        let stale = match &self.cache {
            Some((cached_gen, id, _)) => *cached_gen != generation || *id != want,
            None => true,
        };
        if stale {
            // compute 借 &list[self]，写完缓存又要 &mut self：先算完再写，两个借用就不打架。
            let out = self.list[self.selected].compute(bars);
            self.cache = Some((generation, want, out));
        }
        &self.cache.as_ref().expect("上面刚保证有值").2
    }
}

/// 界面全部可变状态。egui 是即时模式：每帧都把整个界面重新描述一遍，
/// 所以任何要跨帧保留的数据都必须存在这些字段里。
pub(crate) struct StockApp {
    pub(crate) hwnd: isize,
    pub(crate) client_cover_ready: bool,
    pub(crate) open_menu: Option<TopMenu>,
    pub(crate) status_msg: String,
    pub(crate) frames: u64,
    pub(crate) last_frame_time: f64,
    pub(crate) frame_interval_ms: f32,
    pub(crate) frame_ms: f32,
    pub(crate) fps: f32,
    pub(crate) drag: Option<MoveSession>,
    /// 行情数据：启动时生成；在面板上点「生成随机数据 / 重新生成」会换一段新的。
    pub(crate) bars: Vec<data::random_walk::Bar>,
    /// 生成行情用的种子：每点一次按固定步进推到下一个，所以每次点出来的行情都不同又可复现。
    pub(crate) bar_seed: u64,
    /// 每根 K 线实体的像素宽度（奇数 3..=15）：滚轮一格 ±2，画出来永远是整数像素宽、影线居中。
    pub(crate) scale: u32,
    /// 屏幕上第一根 K 线在 bars 里的下标：由拖拽平移改，永远夹在 [0, 总根数 - 可见根数]。
    pub(crate) offset: usize,
    /// 这一帧一屏放得下几根（只跟绘图区宽度和 scale 有关），每帧在 clamp_view 里重算。
    pub(crate) visible: usize,
    /// 拖拽平移的起点：(按下那一刻的鼠标 x, 当时的 offset)；None = 没在拖。
    /// 记起点、每帧按总位移反算，而不是逐帧累加 —— 拖到边界钳住之后再往回拖才不会飘。
    pub(crate) pan: Option<(f32, usize)>,
    /// 滚轮攒下来的零头（不足一格的增量），凑够一格才走一格。
    pub(crate) zoom_acc: f32,
    /// 硬边文字引擎：字形由 Windows GDI 现场光栅化成 1bpp 位图（见第五节）。
    pub(crate) text_engine: Option<HardText>,
    /// 十字光标：None = 鼠标不在绘图区里（十字线不画）。
    pub(crate) crosshair: Option<Crosshair>,
    /// 行情数据代次：每次生成新行情 +1，用来判断映射缓存还算不算数。
    pub(crate) data_gen: u64,
    /// 价格 / 成交量映射的缓存（键：数据代次 + offset + visible + scale）。
    pub(crate) view_cache: ViewCache,
    /// K 线区高度（整数像素，永远是 8 的倍数）：默认 600，最小 240。
    pub(crate) kline_h: f32,
    /// 成交量区高度（整数像素，永远是 8 的倍数）：默认 96，最小 64。
    pub(crate) vol_h: f32,
    /// 指标区高度（整数像素，永远是 8 的倍数）：默认 96，最小 64。
    pub(crate) ind_h: f32,
    /// 拖成交量区上面那条拖拽条时记下「鼠标离拖拽条上边缘多远」，拖拽条不会跳；None = 没在拖。
    pub(crate) hsplit_grab: Option<f32>,
    /// 同上，拖指标区上面那条拖拽条时用的那一份。None = 没在拖。
    pub(crate) vsplit_grab: Option<f32>,
    /// 指标区：可选指标表 + 当前选中 + 结果缓存。
    pub(crate) indicators: IndicatorPane,
    /// 左栏（K 线区）宽度：分割条相对功能区左边缘的位置，永远是 8 的倍数。
    pub(crate) split_x: f32,
    /// 拖分割条时记下「鼠标离分割条左边缘多远」，分割条不会跳；None = 没在拖。
    pub(crate) split_grab: Option<f32>,
    /// 右栏控制面板的按钮：内容 + 量好的宽度，构造时算一次。
    pub(crate) panel_buttons: Vec<PanelButton>,
    /// 当前被按下的按钮下标（按下瞬间更暗）；None = 没有按钮按着。
    pub(crate) panel_press: Option<usize>,
    /// 涨色：内部一律 (u8, u8, u8)，每通道 0-255 整数。K 线画涨就用它（默认红）。
    pub(crate) up_color: (u8, u8, u8),
    /// 跌色：同上（默认绿）。
    pub(crate) down_color: (u8, u8, u8),
    /// 颜色面板的全部跨帧状态（开关、位置、编辑目标、当前 HSV、待定色、Hex 文本、拖动会话）。
    pub(crate) color_panel: ColorPanel,
}

impl StockApp {
    /// 构造：eframe 启动时调用一次。
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
            // 首帧 ui_kline 还没跑，状态栏先按默认布局算一次
            //（左栏净宽 = 左栏 - 外框那 1px - 价格轴）。
            visible: visible_bars(CHART_PANE_W - LINE_W - AXIS_W, BAR_SCALE_DEFAULT),
            pan: None,
            zoom_acc: 0.0,
            text_engine: HardText::new(&cc.egui_ctx),
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
        };
        // 按钮宽度要用硬边文字的实际像素宽来量，所以得等 self 建好再量（宽度跟面板位置无关，量一次就够）。
        app.build_panel_buttons();
        app
    }

    /// 统计刷新情况：帧号、两帧间隔、平滑帧率（帧耗时在 ui() 结尾测）。
    pub(crate) fn tick_frame(&mut self, ctx: &egui::Context) {
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
    /// 可见根数由 ui_kline 每帧重算，和 frame_ms 一样是「上一帧结尾」的值，差一帧显示。
    pub(crate) fn status_text(&self, _layout: egui::Rect) -> String {
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
    pub(crate) fn text(
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
    pub(crate) fn text_width(&mut self, s: &str) -> f32 {
        self.text_engine
            .as_mut()
            .map_or(0.0, |engine| engine.width(s) as f32)
    }

    /// 画四区域骨架（纯绘制，不处理交互）：底色 + 状态栏文字。
    /// 菜单栏内容在 ui_menubar 里画、功能区内容在 ui_kline / ui_volume 里画（它们各自要处理交互）。
    pub(crate) fn paint_skeleton(&mut self, painter: &egui::Painter, layout: egui::Rect, areas: [egui::Rect; 4]) {
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

    /// 区域之间的 1px 硬边分隔线：菜单栏 / 功能区 / 状态栏各自的上沿（区域矩形在那条线上重叠 1px）。
    /// 必须排在标题栏、菜单栏都画完之后调用：这几条线是区域的边框，优先级最高，
    /// 而菜单项的高亮是一块满 cells(3) 高的色块，正好压住菜单栏最上面那一行。
    pub(crate) fn paint_separators(&self, painter: &egui::Painter, layout: egui::Rect, areas: [egui::Rect; 4]) {
        for y in [areas[1].min.y, areas[2].min.y, areas[3].min.y] {
            hline_hard(painter, layout.min.x, layout.max.x, y, C_LINE);
        }
    }

    /// 推进正在进行的窗口拖动：每帧算出窗口该在哪，用 OuterPosition 挪过去。
    pub(crate) fn step_move(&mut self, ctx: &egui::Context, outer: Option<egui::Rect>) {
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

        // 5) 布局区 = 本帧客户区（1 逻辑点 = 1 像素）。窗口尺寸启动后就定死，
        //    布局一律按实际客户区排，只保证不小于下限（WINDOW_MIN_W/H），
        //    免得窗口小到把控件排成负数。
        let win_rect = ui.max_rect();
        let layout = egui::Rect::from_min_size(
            win_rect.min,
            win_rect.size().max(egui::vec2(WINDOW_MIN_W, WINDOW_MIN_H)),
        );

        // 6) 四区域底色 + 标题栏 + 菜单栏 + 功能区（左栏蜡烛图 + 右栏控制面板）。
        let areas = split_areas(layout);
        self.paint_skeleton(&ui.painter().clone(), layout, areas);
        self.ui_titlebar(ui, areas[0], outer_rect);
        let open_anchor = self.ui_menubar(ui, areas[1]);

        // 功能区横向切成三块：左栏 / 分割条 / 右栏控制面板，按左→右依次画。
        // 先把分割条位置收口：窗口变窄时右栏不能低于最小宽度。
        self.clamp_split(areas[2]);
        let [left, splitter, panel] = Self::work_columns(areas[2], self.split_x);
        // 左栏再上下切成五块：K 线区 / 拖拽条 / 成交量区 / 拖拽条 / 指标区。
        // 先收口块高再排版：三块都是 8 的倍数，指标区在底部吸收吸零头剩下的那 0..7px。
        self.clamp_left_heights(left);
        let [kline, hsplit, volume, vsplit, indicator] =
            Self::left_rows(left, self.kline_h, self.vol_h);
        self.ui_kline(ui, kline);
        self.ui_volume(&ui.painter().clone(), volume);
        self.ui_indicator(&ui.painter().clone(), indicator);
        self.ui_hsplitter(ui, hsplit, left, LeftSplit::Volume);
        self.ui_hsplitter(ui, vsplit, left, LeftSplit::Indicator);
        self.ui_splitter(ui, splitter, areas[2]);
        self.ui_panel(ui, panel);

        // 7) 区域分隔线：排在所有区域内容之后画，控件的高亮盖不住这几条 1px 边框。
        self.paint_separators(&ui.painter().clone(), layout, areas);

        // 8) 下拉面板：唯一的例外，它按设计浮在功能区之上，所以排在分隔线之后。
        if let Some((anchor, open)) = open_anchor {
            self.ui_menu_panel(ui, anchor, open);
        }

        // 9) 最外圈 1px 轮廓最后画：压在按钮之上，窗口轮廓永远完整。
        border_hard(&ui.painter().clone(), layout, C_BORDER);

        // 10) 颜色面板：按设计浮在整个界面之上，所以排在最后画（连最外圈轮廓也被它压住）。
        self.ui_color_panel(ui, layout);

        // 11) 推进窗口拖动。
        self.step_move(&ctx, outer_rect);

        // 12) 样式位每帧查一次：winit 在最小化/还原时会重新套用它的样式表，
        //     带 WS_CAPTION 的窗口会被 DWM 画上一圈投影。
        clear_window_frame(self.hwnd);

        // 13) 本帧真正花在构建界面上的时间。
        self.frame_ms = t0.elapsed().as_secs_f32() * 1000.0;

        // 14) 空闲也定期重绘，状态栏的数字才会持续跳动。
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
pub(crate) fn configure_zero_spacing(ctx: &egui::Context) {
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
pub(crate) fn configure_tessellation(ctx: &egui::Context) {
    ctx.tessellation_options_mut(|opt| {
        opt.feathering = false;
        opt.round_text_to_pixels = true;
        opt.round_line_segments_to_pixels = true;
        opt.round_rects_to_pixels = true;
    });
}

