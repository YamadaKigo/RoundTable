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
use crate::ui::icon::*;
use crate::ui::menu::*;
use crate::ui::panel::*;
use crate::ui::source::*;
use crate::ui::winmgr::*;
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

/// 要改窗口区的那些动作（新增图表窗 / 摆回控制面板）。
/// 它们一律先记一笔、由 run_win_cmd 在「窗口管理器写回字段之后」执行 -
/// 画窗口的那一段里 app.win_mgr 是空的（管理器被 take 走了，见 ui()），当场改会改到那份
/// 临时副本上，写回时整份丢掉：按钮点了没反应就是这么来的。
#[derive(Clone, Copy)]
pub(crate) enum WinCmd {
    /// 新增一块图表窗。
    AddChart,
    /// 把控制面板摆回来（关掉了就再打开）。
    OpenPanel,
    /// 打开某块 Pane 的源码编辑窗（它也是窗口区里的一块子窗口，见 ui::winmgr）。
    OpenSource(usize),
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
// 现在只摆一块图表窗（槽位 0），另外两个槽位保留备用：列表里照样留着，换指标照样换得动。
#[allow(dead_code)]
pub(crate) const PANE_VOLUME: usize = 1;
#[allow(dead_code)]
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
            || self.hsplit_grab.is_some()
            || self.vsplit_grab.is_some()
            || self.win_mgr.drag_active()
            || self.color_panel.drag != ColorDrag::None
            || self.color_panel.win.dragging()
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

    // ---- 窗口区的窗口管理（真正的实现在 ui::winmgr，这儿只是「动作」入口）----

    /// 记一笔窗口命令：真正执行在 run_win_cmd 里。
    /// 控制面板上的「新增图表」是在画窗口的过程中点的，那时候管理器正被借走（见 ui()），
    /// 所以不能当场做 - 当场做只会改到临时那份空管理器上，写回时整份丢掉。
    pub(crate) fn request_win(&mut self, cmd: WinCmd) {
        self.win_cmd = Some(cmd);
    }

    /// 执行这一帧记下的窗口命令。必须在窗口管理器写回 app.win_mgr 之后调（见 ui()）。
    pub(crate) fn run_win_cmd(&mut self) {
        match self.win_cmd.take() {
            None => {}
            Some(WinCmd::AddChart) => self.add_chart_window(),
            Some(WinCmd::OpenPanel) => self.open_panel_window(),
            Some(WinCmd::OpenSource(pane)) => self.open_source_window(pane),
        }
    }

    /// 新增一块图表窗（按钮面板的「新增图表」和布局菜单都走它）：新窗口从窗口区左上角起摆，
    /// 那个位置被占了就往下 2 格再往右 2 格地让；最多 MAX_CHART_WINS 块。
    pub(crate) fn add_chart_window(&mut self) {
        if self.win_mgr.chart_count() >= MAX_CHART_WINS {
            self.status_msg = format!("图表窗最多 {MAX_CHART_WINS} 块");
            return;
        }
        let pane = self.new_chart_pane();
        self.win_mgr.add_chart(self.work_rect, pane);
        self.status_msg = format!("图表窗：已新增（共 {} 块）", self.win_mgr.chart_count());
    }

    /// 给新窗口一块 Pane：优先捡「关掉的图表窗」留下的空槽（把它还原成默认的蜡烛图），
    /// 没有就新造一块。一块窗口一块 Pane（互不共用），所以换指标 / 开源码窗都只影响它自己。
    /// 老布局留下的两块备用槽（成交量 / MACD）不给新窗口用：新开的图表窗一律先是一块蜡烛图，
    /// 要什么指标自己拿「切换」换。
    pub(crate) fn new_chart_pane(&mut self) -> usize {
        let mut ind = indicator::builtin().swap_remove(0);
        ind.set_palette(self.current_palette());
        if ind.supports_overlay() {
            ind.set_overlay_state(self.overlay);
        }
        match self.win_mgr.free_chart_pane() {
            Some(free) => {
                self.panes[free] = Pane::new(ind);
                free
            }
            None => {
                self.panes.push(Pane::new(ind));
                self.panes.len() - 1
            }
        }
    }

    /// 布局菜单「控制面板」：把按钮面板摆回来（关掉了就再打开）。
    pub(crate) fn open_panel_window(&mut self) {
        let docked = Self::child_areas(self.work_rect);
        if let Some(i) = self
            .win_mgr
            .wins
            .iter()
            .position(|w| w.kind == WinKind::Panel)
        {
            self.win_mgr.dock(i, docked[1]);
            self.status_msg = "控制面板：已摆回来".to_owned();
        }
    }
}

/// UI工程测试登记的一块 UI 元素：名字 + 矩形 + 它自己那套 9px 网格的原点。
/// 全自绘的界面里，egui 自己不知道「哪块是控件」，所以元素边界由各处的画法自己报一笔
///（见 StockApp::probe），调试工具据此描框、判断网格溢出。
pub(crate) struct GridProbe {
    pub(crate) name: String,
    pub(crate) rect: egui::Rect,
    /// 这块元素自己那套网格的原点（通常是它所在窗口 / 区块的左上角）：
    /// 窗口可以 1px 1px 地挪，跟主窗口的网格不再重合，所以每块元素按自己那套网格判对齐。
    origin: egui::Pos2,
}

impl GridProbe {
    /// 四条边（左 / 上 / 右 / 下）各自离最近那条网格线差几个像素：0 = 正落在网格线上。
    /// 右 / 下两条边按「边框线那一列」算（矩形右上角是排他的，线画在 max - 1px）。
    fn off_grid(&self) -> [f32; 4] {
        let off = |v: f32, o: f32| {
            let d = (v - o) % PITCH;
            if d < 0.0 { d + PITCH } else { d }
        };
        [
            off(self.rect.min.x, self.origin.x),
            off(self.rect.min.y, self.origin.y),
            off(self.rect.max.x - LINE_W, self.origin.x),
            off(self.rect.max.y - LINE_W, self.origin.y),
        ]
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
    /// 图表面板（蜡烛 / 成交量 / MACD 三种指标）：指标 + 指令缓存 + 高度 + 脏标志。
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
    /// 窗口区（桌面）的窗口管理器：窗口区里的每一块窗口（图表窗 / 按钮面板）都归它管
    /// （层级 / 位置尺寸 / 最小化 / 每帧驱动，见 ui::winmgr）。
    pub(crate) win_mgr: WinMgr,
    /// 这一帧记下的窗口命令（见 run_win_cmd）：画窗口的过程中点的按钮不能当场改窗口管理器。
    pub(crate) win_cmd: Option<WinCmd>,
    /// 这一帧的窗口区矩形（新增窗口按它摆、窗口按它收口）：ui() 每帧填一次。
    pub(crate) work_rect: egui::Rect,
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
    /// UI工程测试叠层：打开后把 9px 网格线整个铺在界面最上层（右栏「UI工程测试」按钮切换）。
    pub(crate) show_grid: bool,
    /// UI工程测试这一帧登记下来的 UI 元素（名字 + 矩形 + 它自己那套网格的原点）。
    /// 只有 show_grid 开着才记东西；每帧开头清空。
    pub(crate) grid_probes: Vec<GridProbe>,
    /// UI工程测试要显示的光标位置（本帧鼠标在窗口里的位置）。
    pub(crate) grid_cursor: Option<egui::Pos2>,
    /// UI工程测试：当前这套网格的原点（画哪块窗口就用哪块的左上角，见 probe）。
    pub(crate) grid_origin: egui::Pos2,
    /// 哪个 Pane 的指标下拉面板正开着（记录槽位下标）；None = 都没开。
    pub(crate) pane_menu: Option<usize>,
    /// 那个面板要对齐的「切换」按钮矩形：每帧由 ui_pane_title 填上，面板自己按它定位。
    pub(crate) pane_menu_anchor: egui::Rect,
    /// 「切换」所在的那条图表栏（= 图表窗的菜单栏）：鼠标移出它和下拉面板就收起面板
    ///（和主窗口的菜单栏一个规矩），每帧由 ui_pane_title 填上。
    pub(crate) pane_menu_bar: egui::Rect,
    /// K 线上的叠加均线配置（MA / EMA 各一个位掩码）：全工程就一份 - 主图是 K 线图的那块
    /// 图表（蜡烛图 / 美国线）认它，换指标 / 换皮肤都不动它。
    pub(crate) overlay: indicator::Overlay,
    /// 哪个 Pane 的「叠加」菜单正开着；None = 没开。
    pub(crate) overlay_menu: Option<usize>,
    /// 「叠加」菜单要对齐的那个按钮矩形。
    pub(crate) overlay_menu_anchor: egui::Rect,
    /// 「叠加」所在的那条图表栏（同 pane_menu_bar）。
    pub(crate) overlay_menu_bar: egui::Rect,
    /// 源码编辑窗的内容状态（编辑的是哪个 Pane、编辑器本体、后台编译状态）：
    /// 窗口外壳（位置 / 尺寸 / 最小化 / 层级）归 ui::winmgr 管，它和别的子窗口是同级的一块。
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
            // 窗口区的窗口：第一块图表窗（挂 Pane 0 = 蜡烛图）+ 按钮面板。位置尺寸由
            // 窗口管理器第一帧按停靠布局摆（那时才知道窗口区多大），之后都归用户。
            win_mgr: WinMgr::with_default_windows(),
            win_cmd: None,
            work_rect: egui::Rect::NOTHING,
            panel_buttons: Vec::new(),
            panel_press: None,
            up_color: UP_RGB_DEFAULT,
            down_color: DOWN_RGB_DEFAULT,
            chart_follows_theme: true,
            color_panel: ColorPanel::new(),
            show_grid: false,
            grid_probes: Vec::new(),
            grid_cursor: None,
            grid_origin: egui::Pos2::ZERO,
            pane_menu: None,
            pane_menu_anchor: egui::Rect::NOTHING,
            pane_menu_bar: egui::Rect::NOTHING,
            overlay: indicator::Overlay::DEFAULT,
            overlay_menu: None,
            overlay_menu_anchor: egui::Rect::NOTHING,
            overlay_menu_bar: egui::Rect::NOTHING,
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

    /// 主窗口状态栏的几项：帧率 +（点过东西的话）刚才干了什么。
    /// 每一项自己是一格一格的宽度，项与项之间用点状虚线隔开（见 paint_skeleton）。
    /// 缩放 / 平移 / 可见几根是**图表**的事，写在每块图表窗自己的状态栏上（见 chart_status_segments）。
    pub(crate) fn status_segments(&self) -> Vec<String> {
        let mut segs = vec![format!("{:.0} FPS", self.fps)];
        if !self.status_msg.is_empty() {
            segs.push(self.status_msg.clone());
        }
        segs
    }

    /// 图表窗自己那条状态栏的几项：缩放 / 平移 / 可见几根。
    /// 所有图表共用同一份 offset / scale，所以每块图表窗那一条显示的数是一样的。
    pub(crate) fn chart_status_segments(&self) -> Vec<String> {
        vec![
            format!("缩放 {}", self.scale),
            format!("平移 {}", self.offset),
            format!("可见 {} / 共 {} 根", self.visible, self.bars.len()),
        ]
    }

    /// UI工程测试：登记一块 UI 元素（只有叠层开着才记东西，平时一分钱不花）。
    /// name = 读数牌上显示的名字；origin = 这块元素自己那套 9px 网格的原点
    ///（通常是它所在窗口 / 区块的左上角）。窗口区里的窗口按自己那套网格判对齐。
    /// name = 读数牌上显示的名字；判对齐用的是「当前这套网格的原点」（grid_origin）-
    /// 窗口区里的窗口在画之前会把它自己左上角设进去（见 ui::winmgr 的 draw），
    /// 主窗口那几块用布局区左上角。窗口可以 1px 1px 地挪，所以每块元素按自己那套网格判。
    pub(crate) fn probe(&mut self, name: &str, rect: egui::Rect) {
        if self.show_grid {
            self.grid_probes.push(GridProbe {
                name: name.to_owned(),
                rect,
                origin: self.grid_origin,
            });
        }
    }

    /// 一条状态栏：底色 + 顶沿那条 1px 线 + 从左往右的几项（每项自己一格一格宽、相邻两项之间
    /// 点状虚线）+ 各项的文字。主窗口的状态栏和每块图表窗自己的状态栏都走这一份 -
    /// 铁律：同类型的东西必须长一个样。
    pub(crate) fn paint_status_bar(
        &mut self,
        painter: &egui::Painter,
        band: egui::Rect,
        segs: &[String],
    ) {
        painter.rect_filled(band, egui::CornerRadius::ZERO, theme().status_bg);
        self.probe("状态栏", band);
        hline_hard(painter, band.min.x, band.max.x, band.min.y, theme().line);
        let mut x = band.min.x;
        let mut first = true;
        for seg in segs {
            let w = snap_cells_up(self.text_width(seg) + 2.0 * TEXT_PAD_X);
            if x + w > band.max.x {
                break;
            }
            if !first {
                vline_dotted(painter, x, band.min.y + LINE_W, band.max.y - LINE_W, theme().line);
            }
            self.text(
                painter,
                // 状态栏的网格原点 = 它自己那条左边线（区块自己的 min）。
                egui::pos2(text_left_x(x, band.min.x), band.center().y),
                egui::Align2::LEFT_CENTER,
                seg,
                theme().text,
            );
            x += w - LINE_W;
            first = false;
        }
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
        // 功能区那块是主窗口的「桌面」：底色和常见窗口保持一致（几块子窗口浮在它上面）。
        painter.rect_filled(work, egui::CornerRadius::ZERO, theme().desktop_bg);
        painter.rect_filled(status, egui::CornerRadius::ZERO, theme().status_bg);

        // 区域之间的 1px 分隔线不在这里画，见 paint_separators：它得排在标题栏、
        // 菜单栏的高亮之后，否则菜单项那块满 24px 的高亮会把上边线啃掉一条像素。

        // 状态栏那几项走全工程共用的一份（见 paint_status_bar）：主窗口这条只有帧率和操作回显，
        // 缩放 / 平移 / 可见几根在每块**图表窗自己的**状态栏上。
        let segs = self.status_segments();
        self.paint_status_bar(painter, status, &segs);
    }

    /// 区域之间的 1px 硬边分隔线：菜单栏 / 功能区 / 状态栏各自的上沿（区域矩形在那条线上重叠 1px）。
    /// 必须排在标题栏、菜单栏都画完之后调用：这几条线是区域的边框，优先级最高，
    /// 而菜单项的高亮是一块满 cells(2) 高的色块，正好压住菜单栏最上面那一行。
    pub(crate) fn paint_separators(&self, painter: &egui::Painter, layout: egui::Rect, areas: [egui::Rect; 4]) {
        for y in [areas[1].min.y, areas[2].min.y, areas[3].min.y] {
            hline_hard(painter, layout.min.x, layout.max.x, y, theme().line);
        }
    }

    /// 鼠标是不是压在某个浮动面板上（颜色面板 / 指标下拉面板 / 叠加菜单）：
    /// 这几块都画在最上层，压在它们上面的时候，底下的图表不该再收滚轮。
    pub(crate) fn floating_at(&self, p: egui::Pos2, window_bottom: f32) -> bool {
        if self.color_panel.win.open && self.color_panel.win.rect().contains(p) {
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

    /// UI 工程测试叠层：把窗口的 9px 网格线整个铺满（横竖各一遍，灰色点状线），画在最上层。
    /// 只是个调试视图，不影响任何布局与交互。
    ///
    /// 这是一个「UI 工程级的网格定位测试工具」，看的是四件事：
    ///   1) 网格边界：9px 网格线铺满整个窗口（灰色点状线）；
    ///   2) 所有 UI 元素的边界：本帧登记过的每一块（窗口 / 标题栏 / 图表各条带 / 按钮 / 菜单项 /
    ///      状态栏等等）都描一圈 - 四条边都落在网格线上的是**绿框**，有边没落上的是**橙框**，
    ///      并且在没对上的那条边上打一个刻度（差几个像素一眼能看出来）；
    ///   3) 网格溢出：元素不在整格上时，多出来的零头（1..8px）就落在没对上的那条边上，
    ///      橙框 + 刻度就是它；
    ///   4) 坐标信息：鼠标底下那一格（青框）+ 右下角那块读数牌子 - 光标像素坐标 / 格号 /
    ///      格内偏移 / 压着的那个元素的坐标、尺寸、格数和四条边各差多少。
    pub(crate) fn paint_grid_test(&mut self, painter: &egui::Painter, layout: egui::Rect) {
        let cols = ((layout.max.x - layout.min.x - LINE_W) / PITCH).floor() as i32;
        let rows = ((layout.max.y - layout.min.y - LINE_W) / PITCH).floor() as i32;
        for i in 0..=cols {
            let x = layout.min.x + grid_line(i);
            vline_dotted(painter, x, layout.min.y, layout.max.y, theme().grid_test);
        }
        for j in 0..=rows {
            let y = layout.min.y + grid_line(j);
            hline_dotted(painter, layout.min.x, layout.max.x, y, theme().grid_test);
        }

        // 2) 每一块登记过的元素：绿框 = 四条边都在网格线上，橙框 = 有边没落上。
        //    先画框 + 刻度，最后再画读数牌子（牌子压在最上面）。
        let probes = std::mem::take(&mut self.grid_probes);
        for probe in &probes {
            let off = probe.off_grid();
            let aligned = off.iter().all(|d| *d == 0.0);
            let color = if aligned {
                theme().grid_ok
            } else {
                theme().grid_bad
            };
            border_hard(painter, probe.rect, color);
            // 没对上的那条边：往元素里打一个 4px 的刻度，差几像素看得见。
            if off[0] != 0.0 {
                vline_hard(
                    painter,
                    probe.rect.min.x,
                    probe.rect.min.y,
                    probe.rect.min.y + 4.0,
                    color,
                );
            }
            if off[1] != 0.0 {
                hline_hard(
                    painter,
                    probe.rect.min.x,
                    probe.rect.min.x + 4.0,
                    probe.rect.min.y,
                    color,
                );
            }
            if off[2] != 0.0 {
                vline_hard(
                    painter,
                    probe.rect.max.x - LINE_W,
                    probe.rect.max.y - 4.0,
                    probe.rect.max.y,
                    color,
                );
            }
            if off[3] != 0.0 {
                hline_hard(
                    painter,
                    probe.rect.max.x - 4.0,
                    probe.rect.max.x,
                    probe.rect.max.y - LINE_W,
                    color,
                );
            }
        }

        // 3) 鼠标底下那一格 + 光标位置标记（坐标信息就在这儿落地）。
        if let Some(p) = self.grid_cursor {
            let cell = egui::Rect::from_min_size(
                egui::pos2(
                    layout.min.x + ((p.x - layout.min.x) / PITCH).floor() * PITCH,
                    layout.min.y + ((p.y - layout.min.y) / PITCH).floor() * PITCH,
                ),
                egui::vec2(PITCH, PITCH),
            );
            border_hard(painter, cell, theme().grid_cursor);
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(p.x.floor(), p.y.floor()),
                    egui::vec2(LINE_W, LINE_W),
                ),
                egui::CornerRadius::ZERO,
                theme().grid_cursor,
            );
        }

        // 4) 右下角那块读数牌子。
        self.paint_grid_readout(painter, layout, &probes);
        self.grid_probes = probes;
    }

    /// UI 工程测试的读数牌子（4 行文字，整块落在网格线上）：统计 / 光标坐标 + 格号 + 格内偏移 /
    /// 鼠标底下那个元素的坐标尺寸格数 / 它四条边各差多少（0 = 正落在网格线上）。
    /// 一行文字占 2 个内容格（一个文本行盒，见 font.rs 的网格文字模型）：汉字 2x2 格、
    /// ASCII 1x2 格，所以行距是 grid_line(2) - 按一格一行排的话字会被压扁到格子里，看不清。
    fn paint_grid_readout(
        &mut self,
        painter: &egui::Painter,
        layout: egui::Rect,
        probes: &[GridProbe],
    ) {
        let work = self.work_rect;
        if !work.is_positive() {
            return;
        }
        let rows = 4i32;
        let w = cells(62).min(work.width());
        let h = cells(rows * TEXT_ROWS);
        let rect = egui::Rect::from_min_size(
            egui::pos2(work.max.x - w, work.max.y - h),
            egui::vec2(w, h),
        );
        painter.rect_filled(rect, egui::CornerRadius::ZERO, theme().status_bg);

        let bad = probes
            .iter()
            .filter(|p| p.off_grid().iter().any(|d| *d != 0.0))
            .count();
        let mut lines = Vec::with_capacity(rows as usize);
        lines.push(format!(
            "UI工程测试  元素 {} 块 / 没对齐 {} 块",
            probes.len(),
            bad
        ));
        match self.grid_cursor {
            Some(p) => {
                let cell_x = ((p.x - layout.min.x) / PITCH).floor();
                let cell_y = ((p.y - layout.min.y) / PITCH).floor();
                let off_x = (p.x - layout.min.x) % PITCH;
                let off_y = (p.y - layout.min.y) % PITCH;
                lines.push(format!(
                    "光标 {:.0},{:.0}  格 {:.0},{:.0}  格内 {:.0},{:.0}",
                    p.x.round(),
                    p.y.round(),
                    cell_x,
                    cell_y,
                    off_x,
                    off_y
                ));
            }
            None => lines.push("光标：不在窗口里".to_owned()),
        }
        let under = self
            .grid_cursor
            .and_then(|p| probes.iter().rev().find(|q| q.rect.contains(p)));
        match under {
            Some(q) => {
                lines.push(format!(
                    "压着 {}  {:.0},{:.0}  {:.0}x{:.0}  格 {:.0}x{:.0}",
                    q.name,
                    q.rect.min.x,
                    q.rect.min.y,
                    q.rect.width(),
                    q.rect.height(),
                    (q.rect.width() - LINE_W) / PITCH,
                    (q.rect.height() - LINE_W) / PITCH
                ));
                let off = q.off_grid();
                lines.push(format!(
                    "四边偏差 左{:.0} 上{:.0} 右{:.0} 下{:.0}  {}",
                    off[0],
                    off[1],
                    off[2],
                    off[3],
                    if off.iter().all(|d| *d == 0.0) {
                        "全在网格线上"
                    } else {
                        "有网格溢出"
                    }
                ));
            }
            None => {
                lines.push("压着：空白".to_owned());
                lines.push("四边偏差：-".to_owned());
            }
        }
        for (i, row) in lines.iter().enumerate() {
            let y = rect.min.y + grid_line(i as i32 * TEXT_ROWS);
            if i > 0 {
                hline_hard(painter, rect.min.x + LINE_W, rect.max.x - LINE_W, y, theme().line);
            }
            self.text(
                painter,
                egui::pos2(text_left_x(rect.min.x, rect.min.x), y + CELL_CENTER),
                egui::Align2::LEFT_CENTER,
                row,
                theme().text,
            );
        }
        border_hard(painter, rect, theme().ctrl_border);
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

        // 5) 整块界面画一帧（见 draw_frame）。
        self.draw_frame(ui);

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
//  十一之二、整块界面一帧的绘制
// ===========================================================================

impl StockApp {
    /// 整块界面画一帧：骨架 -> 标题栏 / 菜单栏 -> 窗口区里的子窗口（窗口管理器：从下往上逐块画
    /// 外壳 + 内容）-> 区域分隔线 -> 菜单栏下拉面板 -> 外圈轮廓 -> 浮动面板（颜色面板 / 指标切换 /
    /// 叠加）-> 悬停便签 -> UI 工程测试叠层。运行时的 ui() 和测试的光栅化流水线都走这一份 -
    /// 两边必须是同一条（测试看到的画面 = 用户看到的画面）。
    /// 顺序就是层次：要让某样东西压住别的，改的是这个顺序，不是 Z 值（见铁律）。
    pub(crate) fn draw_frame(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let outer_rect = ctx.input(|i| i.viewport().outer_rect);

        // 后台编译是另开线程跑的：不管源码编辑窗开没开，每帧都看一眼结果（跑完了回状态栏）。
        self.poll_compile();

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
        // UI工程测试这一帧的登记先清空、光标位置记下来（各处画的时候自己报到 grid_probes）。
        self.grid_probes.clear();
        self.grid_cursor = ctx.input(|i| i.pointer.hover_pos());
        // 主窗口那几块（标题栏 / 菜单栏 / 功能区 / 状态栏）按布局区左上角那套网格判对齐；
        // 窗口区里的窗口在画之前会换成它自己左上角（见 ui::winmgr 的 draw）。
        self.grid_origin = layout.min;
        for (name, area) in [
            ("标题栏", areas[0]),
            ("菜单栏", areas[1]),
            ("功能区", areas[2]),
            ("状态栏", areas[3]),
        ] {
            self.probe(name, area);
        }
        self.paint_skeleton(&ui.painter().clone(), layout, areas);
        self.ui_titlebar(ui, areas[0], outer_rect);
        let menubar_state = self.menu.clone();
        let (open_anchor, menubar_state) = ui_menu_bar(self, ui, areas[1], &TOP_TABS, menubar_state);
        self.menu = menubar_state;

        // 6.1) 功能区 = 桌面（窗口区）：里面的窗口（图表窗 / 按钮面板 / 以后新加的）全部交给
        //      窗口管理器（ui::winmgr）管 - 层级、位置尺寸、最小化、内容都在那儿，外面一概不插手。
        //      窗口可以 1px 1px 地挪 / 缩（不吸网格），零头由窗口内部的弹性段吃掉。
        self.work_rect = areas[2];
        let mut mgr = self.win_mgr.take();
        mgr.plan(self, ui, areas[2]);
        self.win_mgr = mgr;
        // 每块图表窗的绘图区（被别的窗口压住的已经清成 NOTHING）：滚轮 / 平移 / 十字光标都用它。
        let plots = self.win_mgr.plots.clone();

        // 6.2) 图表交互：滚轮缩放 + 左键拖拽平移 + 十字光标（几块图表窗共用同一个视图）。
        //      视图动完再夹一次，出图和十字光标都用最终值。
        self.chart_interact(ui, &plots, &[]);
        // 一屏放得下几根按最宽的那块图表窗算（一块图表窗都没露着时保持原样）。
        let plot_w = self.win_mgr.plot_w_max;
        if plot_w > 0.0 {
            self.clamp_view(plot_w);
        }
        self.update_crosshair(ui, &plots);

        // 6.3) 把窗口区的窗口画出来（外壳 + 内容 + 边框，从下往上）。
        let mut mgr = self.win_mgr.take();
        mgr.draw(self, ui, areas[2]);
        self.win_mgr = mgr;

        // 7) 区域分隔线：排在所有区域内容之后画，控件的高亮盖不住这几条 1px 边框。
        self.paint_separators(&ui.painter().clone(), layout, areas);

        // 8) 下拉面板：唯一的例外，它按设计浮在功能区之上，所以排在分隔线之后。
        //    走菜单栏统一那一份（挂在菜单项下沿、左对齐、夹在窗口里、点外收起）。
        if let Some((idx, anchor)) = open_anchor {
            let items = TOP_TABS[idx].items;
            let mut state = self.menu.clone();
            // 点中的是**叶子**项（带子菜单的那几项点了只是把子菜单开出来，不收菜单）。
            if let Some(label) = ui_menu_popup(self, ui, anchor, items, &mut state, layout, &[areas[1]])
            {
                if TOP_TABS[idx].title == "布局" {
                    // 关掉的窗口得有路再摆回来：摆回按钮面板 / 直接开一块新图表。
                    // 这几个都是「记一笔」，真正执行在本帧末尾的 run_win_cmd（见那儿）。
                    match label {
                        "控制面板" => self.request_win(WinCmd::OpenPanel),
                        "新增图表" => self.request_win(WinCmd::AddChart),
                        _ => self.status_msg = format!("你点击了：{label}"),
                    }
                } else {
                    self.status_msg = format!("你点击了：{label}");
                }
            }
            self.menu = state;
        }

        // 9) 最外圈 1px 轮廓最后画：压在按钮之上，窗口轮廓永远完整。
        border_hard(&ui.painter().clone(), layout, theme().ctrl_border);

        // 10) 颜色面板：按设计浮在整个界面之上，所以排在最后画（连最外圈轮廓也被它压住）。
        //     浮动归浮动，位置还是夹在窗口区里（见 ui_color_panel 的 work）：
        //     子窗口一律进不了主窗口的标题栏 / 菜单栏 / 状态栏。
        self.ui_color_panel(ui, areas[2]);

        // 10.5) 指标下拉面板 / 叠加菜单：和颜色面板一样浮在界面之上，排在它之后画。
        //     （源码编辑窗不在这儿 - 它是窗口区里的一块子窗口，和图表窗 / 按钮面板同级，见下面）。
        self.ui_pane_menu(ui);
        self.ui_overlay_menu(ui);

        // 10.55) 窗口命令：本帧谁点了「新增图表 / 摆回控制面板 / 打开源码编辑窗」，到这儿才真做。
        //     窗口管理器早在 6.3 就写回字段了，现在改它才是改在正主身上（见 run_win_cmd）。
        self.run_win_cmd();

        // 10.6) 悬停便签：整帧最上层（浮动面板也压不住它），按钮悬停时记下的那一块。
        self.paint_tooltip(&ui.painter().clone(), layout);

        // 11) UI工程测试工具（调试用）：网格线 + 每块 UI 元素的边界框（对齐绿 / 没对齐橙）+
        //     鼠标格子 + 右下角的读数牌子，都铺在最上层（见 paint_grid_test）。
        if self.show_grid {
            self.paint_grid_test(&ui.painter().clone(), layout);
        }
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
        // 禁掉鼠标的那套「变灰」一律关掉：这个界面全是自绘的，禁用只是「这块不收鼠标」的意思
        //（见 ui::winmgr 的静默窗口），画面该什么样还是什么样，不能被它抹上一层半透明。
        style.visuals.disabled_alpha = 1.0;

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

