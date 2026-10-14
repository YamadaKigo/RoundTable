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
use crate::ui::chart::*;

use crate::ui::colorpanel::*;
use crate::ui::icon::*;
use crate::ui::menu::*;
use crate::ui::panel::*;
use crate::ui::source::*;
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

/// 十字光标的状态：竖线在屏幕上的位置（已经吸附到像素中心）、它压着的那一根 K 线，
/// 以及横线的屏幕位置与由它反算出来的数值。三块图表共用同一条竖线（共用 offset / scale，
/// 映射天然一致），横线和数值标签只有鼠标压着的那一块才有 - 别的块那根鼠标 y 不在它值域里。
#[derive(Clone, Copy)]
pub(crate) struct Crosshair {
    /// 横线对应的数值（悬停那一块按自己的值域反算）。
    pub(crate) value: f64,
    /// 竖线压着的那一根（索引标签显示它）。
    pub(crate) index: usize,
    /// 竖线的精确 K 线位置（小数，跟着鼠标走，不吸附到 K 线中心）。
    pub(crate) fpos: f64,
    /// 横线 / 数值标签画在哪一块图表上（槽位下标）。
    pub(crate) pane: usize,
}

/// 三个图表面板的槽位：左栏三块各挂一个 Pane（阶段 2 起用户可增删 / 换指标，所以统一成列表）。
pub(crate) const PANE_CANDLE: usize = 0;
pub(crate) const PANE_VOLUME: usize = 1;
pub(crate) const PANE_MACD: usize = 2;

/// 一块图表面板 = 一个指标 + 它算出来的绘图指令（数据坐标）缓存。
/// 所有面板共用同一份 offset / scale，所以缩放平移天然同步。
/// dirty 只在「数据换代 / 换指标 / 首次加载 / 换涨跌色」时置真：compute 只在脏的时候跑，
/// 缩放 / 平移 / 鼠标移动 / 每帧重绘一律只重放缓存（见 ui::chart 的 replay_pane）。
pub(crate) struct Pane {
    pub(crate) indicator: Box<dyn indicator::Indicator>,
    pub(crate) cmds: Vec<indicator::DrawCmd>,
    /// 面板绘图区（主图）的高度，像素、8 的倍数：各面板自己的那份，不含标题栏 / 状态带 / 时间带。
    pub(crate) height: i32,
    pub(crate) dirty: bool,
}

impl Pane {
    /// 一个图表 = 一个指标 + 它算出来的指令缓存。要哪几条带由指标自己声明（layout()），
    /// 所以同一份指标挂到哪一块上画出来都一样 - 界面这边不认识任何具体指标。
    pub(crate) fn new(indicator: Box<dyn indicator::Indicator>) -> Self {
        Self {
            indicator,
            cmds: Vec::new(),
            height: 0,
            dirty: true,
        }
    }
}

impl StockApp {
    /// 三块面板的指标：蜡烛图 / 成交量 / MACD，构造时灌入整套颜色。
    /// 走 indicator::builtin()（指标列表就是它）；换色时只 set_palette，不重建。
    pub(crate) fn build_panes(pal: indicator::Palette) -> Vec<Pane> {
        indicator::default_layout()
            .into_iter()
            .map(|mut ind| {
                ind.set_palette(pal);
                // K 线图（蜡烛图 / 美国线）开局就带上默认的叠加均线配置。
                if ind.supports_overlay() {
                    ind.set_overlay_state(indicator::Overlay::DEFAULT);
                }
                Pane::new(ind)
            })
            .collect()
    }

    /// 当前该用的整套颜色：K 线涨跌取 struct 字段（用户可在颜色面板里自定义），
    /// 数字涨跌 / MACD 两条线取当前皮肤（系统级配色）-「K 线跌色变青、数字跌色还是绿」
    /// 就靠这两套分开。
    pub(crate) fn current_palette(&self) -> indicator::Palette {
        let t = theme();
        let rgb = |c: egui::Color32| (c.r(), c.g(), c.b());
        // MA / EMA 各 6 条线的颜色：从皮肤那张表里整份搬过来（见 theme 的 ma / ema）。
        let six = |colors: [egui::Color32; 6]| {
            let mut out = [0u32; 6];
            for (i, c) in colors.iter().enumerate() {
                out[i] = indicator::rgb_u32(rgb(*c));
            }
            out
        };
        indicator::Palette {
            candle_up: indicator::rgb_u32(self.up_color),
            candle_down: indicator::rgb_u32(self.down_color),
            text_up: indicator::rgb_u32(rgb(t.text_up)),
            text_down: indicator::rgb_u32(rgb(t.text_down)),
            dif: indicator::rgb_u32(rgb(t.dif)),
            dea: indicator::rgb_u32(rgb(t.dea)),
            kdj_k: indicator::rgb_u32(rgb(t.kdj_k)),
            kdj_d: indicator::rgb_u32(rgb(t.kdj_d)),
            kdj_j: indicator::rgb_u32(rgb(t.kdj_j)),
            rsi_1: indicator::rgb_u32(rgb(t.rsi_1)),
            rsi_2: indicator::rgb_u32(rgb(t.rsi_2)),
            rsi_3: indicator::rgb_u32(rgb(t.rsi_3)),
            ma: six(t.ma),
            ema: six(t.ema),
        }
    }

    /// 整套颜色变了就重灌一遍（颜色是 compute 时写进指令里的），并让所有面板重算。
    pub(crate) fn sync_pane_palette(&mut self) {
        let want = self.current_palette();
        if self.panes_palette == want {
            return;
        }
        for pane in &mut self.panes {
            pane.indicator.set_palette(want);
            pane.dirty = true;
        }
        self.panes_palette = want;
    }

    /// 切到下一套皮肤：整套界面颜色立刻跟着变（每帧都从 theme() 取色）。
    /// 图表区的 K 线涨跌色如果还「指向系统级配色」，就一并归到新皮肤的那两项上
    /// （暗色皮肤下跌色变青）；用户自己定过色就不动它。改完让图表重算
    /// （柱子 / 蜡烛的颜色是 compute 时写进指令里的）。
    pub(crate) fn switch_theme(&mut self) {
        let next = theme().next();
        set_theme(next);
        if self.chart_follows_theme {
            self.adopt_theme_candle_colors(next);
        }
        self.mark_panes_dirty();
        self.status_msg = format!(
            "主题：{}（图表配色{}）",
            next.name(),
            if self.chart_follows_theme { "跟随系统" } else { "自定义" }
        );
    }

    /// 把 K 线涨跌色对齐到某套皮肤的系统级配色（up_color / down_color 字段就是当前生效值，
    /// 供颜色面板显示；真正画图还是走 current_palette）。
    fn adopt_theme_candle_colors(&mut self, t: Theme) {
        let (u, d) = (t.candle_up, t.candle_down);
        self.up_color = (u.r(), u.g(), u.b());
        self.down_color = (d.r(), d.g(), d.b());
    }

    /// 「图表配色跟随系统」：图表区的 K 线涨跌色重新指向当前皮肤的系统级配色，
    /// 并回到「跟随」状态（以后切皮肤它跟着走）。用户在颜色面板里定过色之后想反悔就点它。
    pub(crate) fn follow_system_chart_colors(&mut self) {
        self.chart_follows_theme = true;
        let t = theme();
        self.adopt_theme_candle_colors(t);
        self.mark_panes_dirty();
        self.status_msg = format!("图表配色已指向系统级配色（{}）", t.name());
    }

    /// 现在有「别的」拖拽在进行吗（拖窗口 / 拖分割条 / 拖横向拖拽条 / 拖浮动面板）？
    /// 图表区据此让路：拖这些东西的时候，十字光标和图表平移都不该响应，两者互不干扰。
    pub(crate) fn ui_drag_active(&self) -> bool {
        self.drag.is_some()
            || self.split_grab.is_some()
            || self.hsplit_grab.is_some()
            || self.vsplit_grab.is_some()
            || self.color_panel.drag != ColorDrag::None
            || self.color_panel.win.drag
            || self.source.win.drag
    }

    /// 数据换代（重新生成行情）：所有面板的缓存作废，下一帧重算。
    pub(crate) fn mark_panes_dirty(&mut self) {
        for pane in &mut self.panes {
            pane.dirty = true;
        }
    }


    /// 需要时才 compute：面板脏了才算一次（数据换代 / 换指标 / 首次加载 / 换涨跌色都会置脏）。
    pub(crate) fn ensure_pane_computed(&mut self, pane: usize) {
        if !self.panes[pane].dirty {
            return;
        }
        let cmds = self.panes[pane].indicator.compute(&self.bars);
        self.panes[pane].cmds = cmds;
        self.panes[pane].dirty = false;
    }
}

/// 界面全部可变状态。egui 是即时模式：每帧都把整个界面重新描述一遍，
/// 所以任何要跨帧保留的数据都必须存在这些字段里。
pub(crate) struct StockApp {
    pub(crate) hwnd: isize,
    pub(crate) client_cover_ready: bool,
    /// 界面用到的全部图标（16x16 PNG 读进来再自绘）：开机装一次，全工程共用。
    pub(crate) icons: IconSet,
    /// 这一帧要画的悬停便签（按钮矩形 + 文字）：按钮悬停时记一笔，ui() 结尾统一画在最上层。
    pub(crate) tooltip: Option<(egui::Rect, String)>,
    /// 主窗口菜单栏的状态（哪一项开着）。菜单栏 / 下拉面板是 ui::menu 那一套，全工程一份。
    pub(crate) menu: MenuState,
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
    /// 每根 K 线实体的像素宽度（奇数 3..=15）：滚轮一格 +/-2，画出来永远是整数像素宽、影线居中。
    pub(crate) scale: u32,
    /// 屏幕上第一根 K 线在 bars 里的下标：由拖拽平移改，永远夹在 [0, 总根数 - 可见根数]。
    pub(crate) offset: usize,
    /// 这一帧一屏放得下几根（只跟绘图区宽度和 scale 有关），每帧在 clamp_view 里重算。
    pub(crate) visible: usize,
    /// 拖拽平移的起点：(按下那一刻的鼠标 x, 当时的 offset)；None = 没在拖。
    /// 记起点、每帧按总位移反算，而不是逐帧累加 - 拖到边界钳住之后再往回拖才不会飘。
    pub(crate) pan: Option<(f32, usize)>,
    /// 滚轮攒下来的零头（不足一格的增量），凑够一格才走一格。
    pub(crate) zoom_acc: f32,
    /// 硬边文字引擎：字形由 Windows GDI 现场光栅化成 1bpp 位图（见第五节）。
    pub(crate) text_engine: Option<HardText>,
    /// 十字光标：None = 鼠标不在绘图区里（十字线不画）。
    pub(crate) crosshair: Option<Crosshair>,
    /// 三块图表面板（蜡烛 / 成交量 / MACD）：指标 + 指令缓存 + 高度 + 脏标志。
    pub(crate) panes: Vec<Pane>,
    /// 建当前这组面板时灌的那整套颜色：和 current_palette() 不一样就重灌（见 sync_pane_palette）。
    pub(crate) panes_palette: indicator::Palette,
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
    /// 图表区的 K 线涨跌色是不是「指向系统级配色」：
    /// true（默认）：跟着当前皮肤走；false：用户在颜色面板里定过色，切皮肤不动它。
    pub(crate) chart_follows_theme: bool,
    /// 颜色面板的全部跨帧状态（开关、位置、编辑目标、当前 HSV、待定色、Hex 文本、拖动会话）。
    pub(crate) color_panel: ColorPanel,
    /// 网格测试叠层：打开后把 9px 网格线整个铺在界面最上层（右栏「网格测试」按钮切换）。
    pub(crate) show_grid: bool,
    /// 哪个 Pane 的指标下拉面板正开着（记录槽位下标）；None = 都没开。
    pub(crate) pane_menu: Option<usize>,
    /// 那个面板要对齐的「切换」按钮矩形：每帧由 ui_pane_title 填上，面板自己按它定位。
    pub(crate) pane_menu_anchor: egui::Rect,
    /// K 线上的叠加均线配置（MA / EMA 各一个位掩码）：全工程就一份 - 主图是 K 线图的那块
    /// 图表（蜡烛图 / 美国线）认它，换指标 / 换皮肤都不动它。
    pub(crate) overlay: indicator::Overlay,
    /// 哪个 Pane 的「叠加」菜单正开着；None = 没开。
    pub(crate) overlay_menu: Option<usize>,
    /// 「叠加」菜单要对齐的那个按钮矩形。
    pub(crate) overlay_menu_anchor: egui::Rect,
    /// 源码查看窗的全部跨帧状态（开关、位置、切好的行、滚动位置、拖动会话）。
    pub(crate) source: SourceView,
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
            icons: IconSet::load_or_create(),
            tooltip: None,
            menu: MenuState::default(),
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
            panes: Self::build_panes(indicator::Palette::default()),
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

    /// 状态栏的几项：帧率 / 缩放 / 平移 / 可见根数 /（点过菜单的话）刚才点了什么。
    /// 每一项自己是一格一格的宽度，项与项之间用点状虚线隔开（见 paint_skeleton）。
    /// 可见根数由 ui_chart 每帧重算，和 frame_ms 一样是「上一帧结尾」的值，差一帧显示。
    pub(crate) fn status_segments(&self) -> Vec<String> {
        let mut segs = vec![
            format!("{:.0} FPS", self.fps),
            format!("缩放 {}", self.scale),
            format!("平移 {}", self.offset),
            format!("可见 {} / 共 {} 根", self.visible, self.bars.len()),
        ];
        if !self.status_msg.is_empty() {
            segs.push(self.status_msg.clone());
        }
        segs
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
    /// 菜单栏内容在 ui::menu 的 ui_menu_bar 里画，功能区内容在 ui::panel / ui::chart 里画
    /// （它们各自要处理交互）。
    pub(crate) fn paint_skeleton(&mut self, painter: &egui::Painter, _layout: egui::Rect, areas: [egui::Rect; 4]) {
        let [title, menu, work, status] = areas;
        painter.rect_filled(title, egui::CornerRadius::ZERO, theme().title_bg);
        painter.rect_filled(menu, egui::CornerRadius::ZERO, theme().menu_bg);
        painter.rect_filled(work, egui::CornerRadius::ZERO, theme().work_bg);
        painter.rect_filled(status, egui::CornerRadius::ZERO, theme().status_bg);

        // 区域之间的 1px 分隔线不在这里画，见 paint_separators：它得排在标题栏、
        // 菜单栏的高亮之后，否则菜单项那块满 24px 的高亮会把上边线啃掉一条像素。

        // 状态栏：几项从左往右排，每项都是整数格、相邻两项共用边界那条 1px 网格线，
        // 所以项与项之间的点状虚线分割线正好落在网格线上；文字左右各留 1 格。
        let mut x = status.min.x;
        let mut first = true;
        for seg in self.status_segments() {
            let w = snap_cells_up(self.text_width(&seg) + 2.0 * TEXT_PAD_X);
            if x + w > status.max.x {
                break;
            }
            if !first {
                vline_dotted(painter, x, status.min.y + LINE_W, status.max.y - LINE_W, theme().line);
            }
            self.text(
                painter,
                egui::pos2(text_left_x(x), status.center().y),
                egui::Align2::LEFT_CENTER,
                &seg,
                theme().text,
            );
            x += w - LINE_W;
            first = false;
        }
    }

    /// 区域之间的 1px 硬边分隔线：菜单栏 / 功能区 / 状态栏各自的上沿（区域矩形在那条线上重叠 1px）。
    /// 必须排在标题栏、菜单栏都画完之后调用：这几条线是区域的边框，优先级最高，
    /// 而菜单项的高亮是一块满 cells(2) 高的色块，正好压住菜单栏最上面那一行。
    pub(crate) fn paint_separators(&self, painter: &egui::Painter, layout: egui::Rect, areas: [egui::Rect; 4]) {
        for y in [areas[1].min.y, areas[2].min.y, areas[3].min.y] {
            hline_hard(painter, layout.min.x, layout.max.x, y, theme().line);
        }
    }

    /// 鼠标是不是压在某个浮动面板上（颜色面板 / 指标下拉面板 / 源码窗）：
    /// 这几块都画在最上层，压在它们上面的时候，底下的图表不该再收滚轮。
    pub(crate) fn floating_at(&self, p: egui::Pos2, window_bottom: f32) -> bool {
        if self.color_panel.win.open && self.color_panel.win.panel().contains(p) {
            return true;
        }
        if self.source.win.open && self.source.win.panel().contains(p) {
            return true;
        }
        if self.pane_menu.is_some() {
            let count = indicator::builtin_names().len();
            if self.pane_menu_rect(count, window_bottom).is_some_and(|r| r.contains(p)) {
                return true;
            }
        }
        if self.overlay_menu.is_some() {
            let count = Self::overlay_items().len();
            if self.overlay_menu_rect(count, window_bottom).is_some_and(|r| r.contains(p)) {
                return true;
            }
        }
        false
    }

    /// 网格测试叠层：把窗口的 9px 网格线整个铺满（横竖各一遍），画在最上层。
    /// 只是个调试视图，不影响任何布局与交互。
    pub(crate) fn paint_grid_test(&self, painter: &egui::Painter, layout: egui::Rect) {
        let cols = ((layout.max.x - layout.min.x - LINE_W) / PITCH).floor() as i32;
        let rows = ((layout.max.y - layout.min.y - LINE_W) / PITCH).floor() as i32;
        for i in 0..=cols {
            let x = layout.min.x + grid_line(i);
            vline_hard(painter, x, layout.min.y, layout.max.y, theme().grid_test);
        }
        for j in 0..=rows {
            let y = layout.min.y + grid_line(j);
            hline_hard(painter, layout.min.x, layout.max.x, y, theme().grid_test);
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
        theme().title_bg.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let t0 = Instant::now();

        // 1) 刷新统计（状态栏要显示它）。
        self.tick_frame(&ctx);
        // 悬停便签每帧重新收集：这一帧有按钮被悬停才会重新记上。
        self.tooltip = None;

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
        let menubar_state = self.menu;
        let (open_anchor, menubar_state) = ui_menu_bar(self, ui, areas[1], &TOP_TABS, menubar_state);
        self.menu = menubar_state;

        // 功能区横向切成三块：左栏 / 分割条 / 右栏控制面板，按左->右依次画。
        // 先把分割条位置收口：窗口变窄时右栏不能低于最小宽度。
        self.clamp_split(areas[2]);
        let [left, splitter, panel] = Self::work_columns(areas[2], self.split_x);
        // 左栏再上下切成五块：图表1 / 拖拽条 / 图表2 / 拖拽条 / 图表3。
        // 先收口块高再排版：三块都是 8 的倍数，最后一块在底部吸收吸零头剩下的那 0..7px。
        self.clamp_left_heights(left);
        let [chart1, hsplit, chart2, vsplit, chart3] =
            Self::left_rows(left, self.kline_h, self.vol_h);
        let areas_3 = [chart1, chart2, chart3];

        // 三块图表的绘图区（纯几何，先算出来供交互与十字光标用）+ 登记高度 + 按需重算缓存。
        let mut plots = [egui::Rect::NOTHING; 3];
        for (i, area) in areas_3.iter().enumerate() {
            let layout = self.panes[i].indicator.layout();
            let (_, _, main, _) = chart_rows(*area, layout);
            plots[i] = frame_rects(main).0;
            self.panes[i].height = main.height() as i32;
        }
        self.sync_pane_palette();
        for i in 0..self.panes.len() {
            self.ensure_pane_computed(i);
        }

        // 交互：滚轮缩放 + 左键拖拽平移。三块共用同一个 offset / scale，在哪一块里操作都算；
        // 视图动完再夹一次，十字光标和出图都用最终值，不会慢一帧。
        // 三条拖拽条（左栏两条横向的 + 两栏之间那条竖向的）也交给它：按在它们上面不算图表平移。
        self.chart_interact(ui, &plots, &[hsplit, vsplit, splitter]);
        self.clamp_view(plots[0].width());

        // 十字光标：三块共用一条竖线（共用 x 映射），横线只画鼠标压着的那一块。
        self.update_crosshair(ui, &plots);

        // 三块图表走同一条流水线（同为「图表」，没有主图 / 副图之分）。
        self.ui_chart(ui, PANE_CANDLE, chart1);
        self.ui_chart(ui, PANE_VOLUME, chart2);
        self.ui_chart(ui, PANE_MACD, chart3);

        self.ui_hsplitter(ui, hsplit, left, LeftSplit::Volume);
        self.ui_hsplitter(ui, vsplit, left, LeftSplit::Indicator);
        self.ui_splitter(ui, splitter, areas[2]);
        self.ui_panel(ui, panel);

        // 7) 区域分隔线：排在所有区域内容之后画，控件的高亮盖不住这几条 1px 边框。
        self.paint_separators(&ui.painter().clone(), layout, areas);

        // 8) 下拉面板：唯一的例外，它按设计浮在功能区之上，所以排在分隔线之后。
        if let Some((idx, anchor)) = open_anchor {
            let tab = &TOP_TABS[idx];
            let panel = MenuPanel::down(anchor, MENU_PANEL_W, tab.items.len());
            let state = self.menu;
            let (clicked, state) = ui_dropdown(self, ui, panel, tab.items, state);
            self.menu = state;
            if let Some(j) = clicked {
                self.status_msg = format!("你点击了：{}", tab.items[j]);
            }
        }

        // 9) 最外圈 1px 轮廓最后画：压在按钮之上，窗口轮廓永远完整。
        border_hard(&ui.painter().clone(), layout, theme().ctrl_border);

        // 10) 颜色面板：按设计浮在整个界面之上，所以排在最后画（连最外圈轮廓也被它压住）。
        self.ui_color_panel(ui, layout);

        // 10.5) 指标下拉面板 / 叠加菜单 / 源码查看窗：和颜色面板一样浮在界面之上，排在它之后画。
        self.ui_pane_menu(ui);
        self.ui_overlay_menu(ui);
        self.ui_source_window(ui, layout);

        // 10.6) 悬停便签：整帧最上层（浮动面板也压不住它），按钮悬停时记下的那一块。
        self.paint_tooltip(&ui.painter().clone(), layout);

        // 11) 网格测试叠层（调试用）：把 9px 网格线铺在最上层。控件边框要是落在网格线上，
        //     就会被它整条盖住；没落上的会露在它旁边，一眼就能看出来。
        if self.show_grid {
            self.paint_grid_test(&ui.painter().clone(), layout);
        }

        // 12) 推进窗口拖动。
        self.step_move(&ctx, outer_rect);

        // 13) 样式位每帧查一次：winit 在最小化/还原时会重新套用它的样式表，
        //     带 WS_CAPTION 的窗口会被 DWM 画上一圈投影。
        clear_window_frame(self.hwnd);

        // 14) 本帧真正花在构建界面上的时间。
        self.frame_ms = t0.elapsed().as_secs_f32() * 1000.0;

        // 15) 空闲也定期重绘，状态栏的数字才会持续跳动。
        ctx.request_repaint_after(Duration::from_millis(IDLE_REPAINT_MS));

    }
}

// ===========================================================================
//  十二、全局样式与渲染开关（窗口一启动就设好，之后不再动）
// ===========================================================================

/// 把 egui 默认样式里的留白全部清零。
/// 默认值大量不是 8 的倍数（item_spacing、button_padding (4,1)、indent 18 ...），
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

