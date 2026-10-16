// ===========================================================================
//  模块 ui::winmgr：窗口区（桌面）的窗口管理器
// ===========================================================================
//  窗口区 = 主窗口里标题栏 / 菜单栏 / 状态栏之外那块最大的空白，所有子窗口都摆在这儿。
//  桌面上的每一块窗口（图表窗 / 按钮面板 / 以后新加的窗）都由这里的 WinMgr 统一管，
//  不许再各写一份 - 这就是「基于网格的窗口管理器」，后续 UI 逻辑的地基。
//
//  一、层级：wins 的下标就是层级（0 最底、最后一块最上）。谁压着谁 = 绘制顺序，不是 Z 值
//      （egui 是即时模式，后画的在上）。新窗口 / 被点到标题栏的窗口一律抬到最上面（activate）。
//  二、位置尺寸：自由像素，拖 1px 跟手，不吸 9px 网格（铁律 4：网格是排版用的，不是枷锁）。
//      窗口因而不再正好是整格大小，多出来的那几个像素（溢出空间）由窗口内部消化：
//        - 尺寸固定、绝不许变形的（最小化键 / 关闭键 / 状态带 / 时间带）永远钉在右边 / 下边；
//        - 弹性段（标题栏的标题区、图表区的绘图区）吃下剩下的全部；
//      所以零头原则上落在右边和下方，而且只落在「装得下零头」的地方 - 和主窗口那套一样。
//      （见 grid 的 free_band / free_tail：固定段尺寸是死的，弹性段拿零头。）
//  三、最小化：窗口压成一条标题栏（只留标题和两个键），统一堆在窗口区左上角，一条挨一条
//      （还没有任务栏，先这么堆着）；内容整块隐藏，还原时回到原来的位置尺寸。
//  四、新增窗口：从窗口区左上角起摆，那个位置被占了就往下 2 格（一条标题栏）、再往右 2 格地让，
//      一路让到空位 - 就是 Windows 那种新窗口堆叠。
//  五、会话：拖动 / 缩放都记在窗口自己的字段里（egui 是即时模式，不存就跨不过一帧），
//      而且一律以「按下那一刻的矩形」为基准算，拖得再快也不会飘。
//      热区 id 用窗口自己的稳定 id：层级一挪动，egui 的自增 id 就会错位，按一半的点击会丢。
use crate::app::*;
use crate::grid::*;
use crate::ui::chart::*;
use crate::ui::floatwin::*;
use crate::ui::panel::*;
use crate::ui::titlebar::*;
use eframe::egui;

/// 窗口区里最多摆几块图表窗（按钮面板不算）。
pub(crate) const MAX_CHART_WINS: usize = 8;
/// 新增窗口的堆叠步进：一条标题栏那么高（2 格 = 18px），横向也是 2 格。
pub(crate) const CASCADE_STEP: f32 = grid_line(WIN_TITLE_ROWS);
/// 新开一块图表窗的默认尺寸：60 x 40 格。
pub(crate) const NEW_WIN_W: f32 = cells(60);
pub(crate) const NEW_WIN_H: f32 = cells(40);
/// 最小化之后那条标题栏的宽度：标题 + 两个键都放得下（10 格 = 91px）。
pub(crate) const MINI_W: f32 = cells(10);
/// 所有图表窗的标题栏文字：一律叫「图表」（指标名写在图表栏上，见 ui::chart）。
pub(crate) const WIN_CHART_TITLE: &str = "图表";
/// 源码编辑窗标题栏的头一段（后面跟指标名，见 StockApp::source_title）。
pub(crate) const SRC_TITLE: &str = "源码";

const _: () = assert!((NEW_WIN_W as i32 - 1) % 9 == 0 && (NEW_WIN_H as i32 - 1) % 9 == 0);
const _: () = assert!((MINI_W as i32 - 1) % 9 == 0);
const _: () = assert!(MINI_W > 2.0 * WIN_CLOSE_W);

/// 窗口类型：图表窗（挂一块 Pane）/ 按钮面板 / 源码编辑窗。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum WinKind {
    Chart,
    Panel,
    /// 指标源码编辑窗（内容 = ui::editor；它和别的子窗口同级，一样能最小化）。
    Source,
}

/// 桌面上的一块窗口。
pub(crate) struct ManagedWin {
    pub(crate) kind: WinKind,
    /// 标题栏上的字（图表窗一律是「图表」，按钮面板是「控制面板」，源码窗是「源码 - 指标名」）。
    pub(crate) title: String,
    pub(crate) minimized: bool,
    /// 最小化那一刻的位置尺寸：还原时用。
    pub(crate) restore: (egui::Pos2, egui::Vec2),
    /// 图表窗挂的 Pane 下标（按钮面板不用）。
    pub(crate) pane: usize,
    /// 第几个被最小化的（越小越先被最小化）：左上角那一摞按它从上往下排 -
    /// 先最小化的在最上面，和任务栏的顺序一个道理，跟层级怎么挪没关系。
    mini_seq: u64,
    /// 最小化那条标题栏的宽：按标题文字量算（保证标题整串都看得见），保底 MINI_W。
    /// 最小化的那一刻量一次就够了 - 最小化期间标题不会变。
    mini_w: f32,
    /// 窗口三件套：位置 / 尺寸 / 拖动与缩放会话（和浮动窗同源，全工程就这一份，见 ui::floatwin）。
    pub(crate) w: Win,
}

impl ManagedWin {
    /// 窗口本体（自己存的位置尺寸）。
    pub(crate) fn rect(&self) -> egui::Rect {
        self.w.rect()
    }

    /// 内容区 = 标题栏（2 格）下面那一整块（弹性段：窗口不是整格高时零头落在它里面）。
    pub(crate) fn body(rect: egui::Rect) -> egui::Rect {
        free_tail(rect, WIN_TITLE_ROWS)
    }
}

/// 窗口区（桌面）的窗口管理器：层级 + 位置尺寸 + 最小化 + 每帧驱动。
pub(crate) struct WinMgr {
    /// 0 最底、最后一块最上。
    pub(crate) wins: Vec<ManagedWin>,
    /// 这一帧每块 Pane 的绘图区（被压住的已经清成 NOTHING）：滚轮 / 平移 / 十字光标都用它。
    pub(crate) plots: Vec<egui::Rect>,
    /// 这一帧所有露着的图表窗里最宽的那块绘图区（0 = 一块都没露着）：
    /// 状态栏的「可见几根」和 offset 的收口按它算 - 鼠标停在按钮面板上时也照样准。
    pub(crate) plot_w_max: f32,
    /// 每块 Pane 的绘图区属于哪块窗口（画的时候填，用来判断有没有被压住）。
    plot_owner: Vec<usize>,
    /// 要求抬层级的那块窗口（最小化 / 还原时点的）：延后到下一帧开头兑现 -
    /// 会话处理中途挪层级会把下标搞乱。
    pending: Option<u64>,
    /// 这一帧的窗口区矩形（最小化那条小标题栏按它堆）：每帧由 plan / draw 填一次。
    pub(crate) work: egui::Rect,
    /// 是不是已经按默认布局摆过第一次了（窗口区多大要第一帧才知道）。
    placed: bool,
    next_id: u64,
    next_mini: u64,
}

impl WinMgr {
    pub(crate) fn new() -> Self {
        Self {
            wins: Vec::new(),
            plots: Vec::new(),
            plot_w_max: 0.0,
            plot_owner: Vec::new(),
            pending: None,
            work: egui::Rect::NOTHING,
            placed: false,
            next_id: 1,
            next_mini: 0,
        }
    }

    /// 把管理器整份端走：画一块窗口要 &mut StockApp，管理器自己不能还借在 app 里。
    /// 用法固定是 begin（take）-> ui -> 写回 app.win_mgr。
    pub(crate) fn take(&mut self) -> Self {
        std::mem::replace(self, Self::new())
    }

    /// 开机那两块窗口：按钮面板 + 第一块图表窗（挂 Pane 0 = 蜡烛图，压在面板上面）。
    /// 位置尺寸由第一帧的默认布局摆（见 place_defaults），之后都归用户。
    pub(crate) fn with_default_windows() -> Self {
        let mut mgr = Self::new();
        mgr.add(
            WinKind::Panel,
            PANEL_TITLE,
            PANE_CANDLE,
            egui::vec2(PANEL_W, PANEL_CHILD_H),
            egui::vec2(PANEL_W, PANEL_CHILD_H),
        );
        mgr.add(
            WinKind::Chart,
            WIN_CHART_TITLE,
            PANE_CANDLE,
            egui::vec2(NEW_WIN_W, NEW_WIN_H),
            Self::chart_min_size(),
        );
        mgr
    }

    /// 建一块窗口（位置尺寸先给个初值，第一帧按默认布局 / 堆叠摆）。
    pub(crate) fn add(
        &mut self,
        kind: WinKind,
        title: &str,
        pane: usize,
        size: egui::Vec2,
        min_size: egui::Vec2,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        // 窗口三件套：位置尺寸先给个初值，第一帧按默认布局摆；窗口区里的窗口都能拉右 / 下 /
        // 右下角改尺寸（和浮动窗同一套，见 ui::floatwin 的 Win）。
        let mut w = Win::new(id, 0.0, 0.0, size.x, size.y);
        w.open = true;
        w.min_size = min_size;
        w.resizable = true;
        self.wins.push(ManagedWin {
            kind,
            title: title.to_owned(),
            minimized: false,
            restore: (egui::Pos2::ZERO, size),
            pane,
            mini_seq: 0,
            mini_w: MINI_W,
            w,
        });
        self.wins.len() - 1
    }

    /// 活着的图表窗有几块（新增按钮按它限量）。
    pub(crate) fn chart_count(&self) -> usize {
        self.wins
            .iter()
            .filter(|w| w.kind == WinKind::Chart && w.w.open)
            .count()
    }

    /// 源码编辑窗现在开着吗（编辑器 / 关闭键 / 换指标都要问它）。
    pub(crate) fn source_open(&self) -> bool {
        self.wins
            .iter()
            .any(|w| w.kind == WinKind::Source && w.w.open)
    }

    /// 源码编辑窗那块窗口的下标（没建过就是 None）。
    fn source_index(&self) -> Option<usize> {
        self.wins.iter().position(|w| w.kind == WinKind::Source)
    }

    /// 摆出源码编辑窗：建过一次就一直留着（位置尺寸归用户，关掉再开还在原地），
    /// 每次打开都抬到最上层、顺手从最小化那摞里捞出来。
    pub(crate) fn show_source(&mut self, work: egui::Rect) -> usize {
        if let Some(i) = self.source_index() {
            self.wins[i].w.open = true;
            self.wins[i].minimized = false;
            self.activate(i);
            return i;
        }
        let i = self.add(
            WinKind::Source,
            SRC_TITLE,
            0,
            egui::vec2(SRC_W, SRC_H),
            egui::vec2(SRC_W, SRC_H),
        );
        self.wins[i].w.pos = egui::pos2(SRC_DEFAULT_X, SRC_DEFAULT_Y);
        // 首帧之前（work 还没算出来）别夹：夹进一个空矩形会把位置推成无穷大。
        if work.is_positive() {
            self.wins[i].w.clamp_into(work);
        }
        self.wins[i].restore = (self.wins[i].w.pos, self.wins[i].w.size);
        i
    }

    /// 关掉源码编辑窗（窗口本体留着：下次打开还在用户摆的那个位置）。
    pub(crate) fn hide_source(&mut self) {
        if let Some(i) = self.source_index() {
            self.wins[i].w.open = false;
            self.wins[i].minimized = false;
            self.wins[i].w.end_sessions();
        }
    }

    /// 所有图表窗占着的 Pane 下标（关掉的也算）：给新窗口找空槽用，免得两块窗共用一个 Pane。
    #[allow(dead_code)]
    pub(crate) fn chart_panes(&self) -> Vec<usize> {
        self.wins
            .iter()
            .filter(|w| w.kind == WinKind::Chart)
            .map(|w| w.pane)
            .collect()
    }

    /// 关掉的图表窗留下的 Pane 槽（有就复用 - 别一个劲地新建 Pane，也别去占别人的）。
    pub(crate) fn free_chart_pane(&self) -> Option<usize> {
        self.wins
            .iter()
            .find(|w| w.kind == WinKind::Chart && !w.w.open)
            .map(|w| w.pane)
    }

    /// 抬到最上层：把它从列表里抽出来、压到末尾（最后一个 = 最上面）。
    pub(crate) fn activate(&mut self, i: usize) {
        if i + 1 >= self.wins.len() {
            return;
        }
        let w = self.wins.remove(i);
        self.wins.push(w);
    }

    /// 按 id 抬层级（延后一帧兑现的那种）。
    pub(crate) fn activate_id(&mut self, id: u64) {
        self.pending = Some(id);
    }

    /// 第一帧按默认布局摆一遍：第一块图表窗铺左栏、按钮面板钉在右列顶部（和以前一样）。
    /// 之后再开的新窗口一律按堆叠法摆（见 add_chart）。
    fn place_defaults(&mut self, work: egui::Rect) {
        if self.placed {
            return;
        }
        self.placed = true;
        let docked = StockApp::child_areas(work);
        let mut chart_done = false;
        for w in self.wins.iter_mut() {
            let r = match w.kind {
                WinKind::Chart if !chart_done => {
                    chart_done = true;
                    docked[0]
                }
                WinKind::Panel => docked[1],
                // 别的窗口（后加的图表窗 / 源码编辑窗）不参与默认布局：位置归用户，开的时候才摆。
                WinKind::Chart | WinKind::Source => continue,
            };
            w.w.pos = r.min;
            w.w.size = r.size();
            w.restore = (r.min, r.size());
        }
    }

    /// 把每一块窗口都夹回窗口区里（自由像素，不吸网格）：位置不许出去，尺寸不许大过窗口区。
    fn clamp_all(&mut self, work: egui::Rect) {
        for w in self.wins.iter_mut() {
            if w.w.open {
                w.w.clamp_into(work);
            }
        }
    }

    /// 最小化那条标题栏要多宽：左边 1 格空档 + 标题整串 + 右边 1 格空档 + 右端那两个键
    ///（两个键各 cells(2)、相邻共用中间那条 1px 线 = 2*cells(2) - LINE_W = 37px）。
    /// 向上取到整格（cells(n)），最小不小于 MINI_W - 标题长的窗口缩成条之后照样看得见全名。
    pub(crate) fn mini_width(app: &mut StockApp, title: &str) -> f32 {
        let need = app.text_width(title) + 2.0 * TEXT_PAD_X + 2.0 * SYS_BTN_W - LINE_W;
        cells(((need - LINE_W) / PITCH).ceil() as i32).max(MINI_W)
    }

    /// 最小化的窗口统一堆在窗口区左上角：一条标题栏高一条、从上往下排（还没有任务栏），
    /// 相邻两条共用中间那条 1px 线格，所以一条的步进就是一条标题栏（2 格 = 18px）。
    /// 宽按这块窗口自己的标题量（见 mini_width），一条一个宽。
    /// 位置是算出来的、不存字段 - 所以还原的时候原来拖到哪儿还回哪儿。
    fn mini_rect(&self, i: usize, k: usize) -> egui::Rect {
        let work = self.work;
        let w = self.wins[i].mini_w.min(work.width());
        let h = WIN_TITLE_H.min(work.height());
        egui::Rect::from_min_size(
            egui::pos2(work.min.x, work.min.y + grid_line(k as i32 * WIN_TITLE_ROWS)),
            egui::vec2(w, h),
        )
    }

    /// 这块窗口在「最小化那一摞」里排第几：先被最小化的排在上面（见 mini_seq）。
    fn mini_index(&self, i: usize) -> usize {
        let seq = self.wins[i].mini_seq;
        self.wins
            .iter()
            .filter(|w| w.w.open && w.minimized && w.mini_seq < seq)
            .count()
    }

    /// 第 i 块窗口这一帧画在哪：最小化的那块是堆出来的小标题栏，别的就是它自己。
    pub(crate) fn rect_of(&self, i: usize) -> egui::Rect {
        if self.wins[i].minimized {
            let k = self.mini_index(i);
            self.mini_rect(i, k)
        } else {
            self.wins[i].rect()
        }
    }

    /// 绘制顺序（从下往上）：先按层级画正常窗口，最小化的那一摞最后画 -
    /// 它们就是「任务栏」，永远压在最上面，堆在左上角一眼就能点到。
    fn draw_order(&self) -> Vec<usize> {
        let live = |w: &ManagedWin, mini: bool| w.w.open && w.minimized == mini;
        let mut out: Vec<usize> = (0..self.wins.len())
            .filter(|&i| live(&self.wins[i], false))
            .collect();
        out.extend((0..self.wins.len()).filter(|&i| live(&self.wins[i], true)));
        out
    }

    /// 鼠标压在谁身上（最上面那块）：被压住的窗口不该再收滚轮 / 平移 / 十字光标。
    fn top_win_at(&self, p: egui::Pos2) -> Option<usize> {
        self.draw_order()
            .into_iter()
            .rev()
            .find(|&i| self.rect_of(i).contains(p))
    }

    /// 这一帧「激活」的是哪一块：最前面那块非最小化的窗口（wins 的尾巴数回来）。
    /// 只有它能收鼠标 - 别的块一律静默，点一下才轮到它（见 plan 和 draw）。
    /// 最小化的那一摞不算：它们是任务栏，永远能点（见 draw）。
    pub(crate) fn live_index(&self) -> Option<usize> {
        self.draw_order()
            .into_iter()
            .rev()
            .find(|&i| !self.wins[i].minimized)
    }

    /// 这一次按下该归谁：鼠标压在谁的缩放命中带上就归谁（那几条带子是往窗口里外各扩 EDGE_BAND
    /// 的，贴边拿鼠标拽的时候外半截会压在**下面的窗口**上），没压在带子上就归鼠标底下最上面那块。
    /// 先看带子的原因：拽着 A 的边却把 B 激活了（B 挡在带子外面）是错的 - 出现缩放箭头的时候，
    /// 这一下就是「拽 A 的边」，跟 B 没关系。
    fn press_owner_at(&self, p: egui::Pos2) -> Option<usize> {
        self.draw_order()
            .into_iter()
            .rev()
            .find(|&i| {
                let w = &self.wins[i];
                w.w.open
                    && !w.minimized
                    && !w.w.maximized
                    && w.w.resizable
                    && edge_at(w.rect(), p, EDGE_BAND).any()
            })
            .or_else(|| self.top_win_at(p))
    }

    /// 新增一块图表窗：窗口区左上角起摆，那个位置被占了就让一条标题栏（往下 2 格 + 往右 2 格），
    /// 一路让到空位；整块夹在窗口区里。新窗口永远在最上层。
    pub(crate) fn add_chart(&mut self, work: egui::Rect, pane: usize) -> usize {
        let size = egui::vec2(
            NEW_WIN_W.min(work.width()),
            NEW_WIN_H.min(work.height()),
        );
        let mut pos = work.min;
        for k in 0..=self.wins.len() {
            let step = CASCADE_STEP * k as f32;
            let want = egui::pos2(work.min.x + step, work.min.y + step);
            pos = egui::pos2(
                clamp_soft(want.x, work.min.x, work.max.x - size.x),
                clamp_soft(want.y, work.min.y, work.max.y - size.y),
            );
            if !self.occupied(pos) {
                break;
            }
        }
        let i = self.add(
            WinKind::Chart,
            WIN_CHART_TITLE,
            pane,
            size,
            Self::chart_min_size(),
        );
        self.wins[i].w.pos = pos;
        self.wins[i].restore = (pos, size);
        // 新窗口就摆在列表末尾 = 最上层（铁律 1：新窗口压在最上面）。
        i
    }

    /// 图表窗的尺寸下限：高度够放下标题栏 + 状态带 + 时间带 + 一小块绘图区，宽度够放下轴和按钮。
    pub(crate) fn chart_min_size() -> egui::Vec2 {
        egui::vec2(PANE_MIN_W, PANE_H_MIN)
    }

    /// 这个左上角位置被占了吗（有窗口正好摆在那儿）- 堆叠就看这个。
    fn occupied(&self, p: egui::Pos2) -> bool {
        self.occupied_except(p, usize::MAX)
    }

    /// 同上，但放过第 skip 块（给它自己找空位的时候用）。
    fn occupied_except(&self, p: egui::Pos2, skip: usize) -> bool {
        self.wins.iter().enumerate().any(|(i, w)| {
            i != skip && w.w.open && !w.minimized && (w.w.pos - p).length_sq() < 0.5
        })
    }

    /// 现在有窗口正在拖 / 正在缩吗？图表区据此让路（两者互不干扰）。
    pub(crate) fn drag_active(&self) -> bool {
        self.wins.iter().any(|w| w.w.dragging() || w.w.resizing())
    }

    /// 把某一块窗口摆回停靠布局的位置尺寸（布局菜单「控制面板」用），并让它开着。
    pub(crate) fn dock(&mut self, i: usize, r: egui::Rect) {
        self.wins[i].w.pos = r.min;
        self.wins[i].w.size = r.size();
        self.wins[i].restore = (r.min, r.size());
        self.wins[i].w.open = true;
        self.wins[i].minimized = false;
        self.wins[i].w.end_sessions();
    }

    /// 关掉一块窗口：面板和图表窗都能关（布局菜单里还能摆回来），顺手收掉它开着的下拉面板。
    fn close_win(&mut self, app: &mut StockApp, i: usize) {
        let pane = self.wins[i].pane;
        let title = self.wins[i].title.clone();
        self.wins[i].w.open = false;
        self.wins[i].w.end_sessions();
        self.wins[i].minimized = false;
        if self.wins[i].kind == WinKind::Chart {
            if app.pane_menu == Some(pane) {
                app.pane_menu = None;
            }
            if app.overlay_menu == Some(pane) {
                app.overlay_menu = None;
            }
        }
        app.status_msg = format!("{title}：已关闭");
    }

    /// 最小化一块窗口：现在的几何先记进 restore，然后压成一条标题栏、堆到左上角那一摞的最后
    /// （先最小化的排在最上面 - 和任务栏一个道理，跟层级怎么挪没关系）。
    /// mini_w = 那一条的宽（按标题量，见 mini_width）：标题有几个字就留几个字的宽，整串都看得见。
    pub(crate) fn minimize(&mut self, i: usize, mini_w: f32) {
        self.wins[i].restore = (self.wins[i].w.pos, self.wins[i].w.size);
        self.wins[i].mini_w = mini_w;
        self.wins[i].minimized = true;
        self.next_mini += 1;
        self.wins[i].mini_seq = self.next_mini;
        self.wins[i].w.end_sessions();
    }

    /// 还原一块最小化的窗口：还回原位原尺寸。
    pub(crate) fn restore(&mut self, i: usize) {
        let (pos, size) = self.wins[i].restore;
        self.wins[i].w.pos = pos;
        self.wins[i].w.size = size;
        self.wins[i].minimized = false;
    }

    /// 标题栏上那个最小化键：最小化 / 还原。抬层级延后一帧（见 activate_id）-
    /// 这会儿正画着它，下标不能中途动。
    fn toggle_minimize(&mut self, app: &mut StockApp, i: usize) {
        let id = self.wins[i].w.id;
        if self.wins[i].minimized {
            self.restore(i);
        } else {
            // 那一条标题栏的宽按标题量（见 mini_width）。
            let title = self.wins[i].title.clone();
            let w = Self::mini_width(app, &title);
            self.minimize(i, w);
        }
        self.activate_id(id);
    }

    /// 每帧第一段（画之前）：摆默认布局 -> 收口 -> 处理激活 -> 算好每块图表窗的绘图区。
    /// 绘图区必须在画之前就定下来：图表交互和十字光标是这一帧现算现画的。
    pub(crate) fn plan(&mut self, app: &mut StockApp, ui: &mut egui::Ui, work: egui::Rect) {
        self.work = work;
        self.place_defaults(work);
        // 上一帧谁要求抬层级（最小化 / 还原点出来的），这一帧开头兑现。
        if let Some(id) = self.pending.take() {
            if let Some(i) = self.wins.iter().position(|w| w.w.id == id) {
                self.activate(i);
            }
        }
        // 松手：所有窗口的拖 / 缩会话一起结束。
        if !ui.input(|i| i.pointer.primary_down()) {
            for w in self.wins.iter_mut() {
                w.w.end_sessions();
            }
        }
        self.clamp_all(work);

        // 点在窗口上 = 激活（抬到最上层），点哪儿都算（标题栏 / 内容 / 边角都行）。
        // 和 Windows 的窗口管理器一个逻辑：点一下**既是激活也是操作** - 按在标题栏上就是「激活 +
        // 开始拖」，按在按钮上就是「激活 + 按下那个按钮」，不用先点一下激活再点一下操作。
        // 压着的那块被抬到最前之后，这一帧下面的窗口全在它下面，鼠标自然归它（见 draw）。
        // 两个例外：压在浮动面板 / 下拉面板上不算（那几块画在整界面最上层，被它们压着的窗口
        // 不该被顺带抬起来）；压在谁的缩放命中带上就归谁（见 press_owner_at - 拽着边的时候
        // 带子外面那半截可能压着别的窗口，不能把那个窗口抬上来）。
        if ui.input(|i| i.pointer.primary_pressed()) {
            if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
                if !app.floating_at(p, ui.max_rect().max.y) {
                    if let Some(i) = self.press_owner_at(p) {
                        self.activate(i);
                    }
                }
            }
        }

        // 每块图表窗的绘图区：只有最前面那块（激活的）才算数（别的块静默，清成 NOTHING）。
        // （正在平移时不收口：鼠标滑出窗口的途中十字线不该闪。）
        let panes = app.panes.len();
        self.plots.clear();
        self.plots.resize(panes, egui::Rect::NOTHING);
        self.plot_owner.clear();
        self.plot_owner.resize(panes, usize::MAX);
        let live = self.live_index();
        let all = app.pan.is_some();
        self.plot_w_max = 0.0;
        for i in self.draw_order() {
            if !self.wins[i].w.open || self.wins[i].kind != WinKind::Chart || self.wins[i].minimized {
                continue;
            }
            let pane = self.wins[i].pane;
            let body = ManagedWin::body(self.rect_of(i));
            let layout = app.panes[pane].indicator.layout();
            let (_, _, main, _) = chart_rows(body, layout);
            self.plots[pane] = frame_rects(main).0;
            self.plot_owner[pane] = i;
            self.plot_w_max = self.plot_w_max.max(self.plots[pane].width());
            if !all && live != Some(i) {
                self.plots[pane] = egui::Rect::NOTHING;
            }
        }
    }

    /// 每帧第二段（画）：从下往上逐块画 - 外壳前半 -> 内容（调用方自己画）-> 外壳后半。
    /// 鼠标归谁由 egui 的层级命中说了算（后画的压在上面）：被压住的窗口那一块根本收不到鼠标，
    /// 露出来的部分照样能 hover / 点 - 点它就是「激活 + 操作」那一下（见 plan）。
    ///
    /// 【窗口区里的控件不许用 egui 的自动 id】点一下就激活会换绘制顺序（被点的那块画到最后），
    /// 而自动 id 是按「这一帧第几个登记的」算的：顺序一变，同一颗按钮的 id 跟着变，
    /// 按下的那一下就跟松手时对不上、点击直接丢。所以这儿一律走 grid::interact_rect（id 按矩形算）。
    pub(crate) fn draw(&mut self, app: &mut StockApp, ui: &mut egui::Ui, work: egui::Rect) {
        self.work = work;
        for i in self.draw_order() {
            if !self.wins[i].w.open {
                continue;
            }
            // UI工程测试：这块窗口里的元素都按「它自己左上角」那套 9px 网格判对齐
            //（窗口可以 1px 1px 地挪，跟主窗口的网格不再重合）。
            let prev_origin = app.grid_origin;
            app.grid_origin = self.rect_of(i).min;
            let body = self.begin_win(app, ui, i);
            if let Some(body) = body {
                // UI工程测试：窗口本体 / 标题栏 / 内容区各报一笔。
                app.probe(&format!("窗口 {}", self.wins[i].title), self.rect_of(i));
                app.probe(
                    "窗口标题栏",
                    free_band(self.rect_of(i), 0, WIN_TITLE_ROWS),
                );
                self.draw_content(app, ui, i, body);
            }
            self.end_win(ui, i);
            app.grid_origin = prev_origin;
        }
    }

    /// 一块窗口的内容（调用方自己画）：图表窗是图表，按钮面板是那一列按钮。
    /// 画之前它那一段的绘图区得先填上（图表交互和十字光标按它算）。
    fn draw_content(&mut self, app: &mut StockApp, ui: &mut egui::Ui, i: usize, body: egui::Rect) {
        match self.wins[i].kind {
            WinKind::Panel => app.ui_panel(ui, body),
            WinKind::Chart => {
                let pane = self.wins[i].pane;
                let layout = app.panes[pane].indicator.layout();
                let (_, _, main, _) = chart_rows(body, layout);
                self.plots[pane] = frame_rects(main).0;
                self.plot_owner[pane] = i;
                app.ui_chart(ui, pane, body);
            }
            WinKind::Source => {
                // 标题每帧现取（改过没有 / 编的是哪个指标的源码都写在标题栏上）。
                self.wins[i].title = app.source_title();
                // 编辑器只画内容（菜单栏 + 按钮栏 + 正文 + 它自己的下拉面板），外壳上面 begin_win
                // 已经画过；排版拿整块窗口当坐标基准 - 标题栏那 2 格和编辑器自己的口径本来就是
                // 同一条（SRC_TITLE_ROWS == WIN_TITLE_ROWS）。
                let rect = self.wins[i].w.rect();
                let cmd = app.ui_source_body(ui, rect);
                app.run_editor_cmd(cmd);
            }
        }
    }

    /// 外壳前半：本体底色 + 吃事件的整块 + 标题栏（标题 + 最小化键 + 关闭键 + 拖动把手）。
    /// 返回内容区；最小化的窗口只剩一条标题栏，返回 None（这种窗口不画内容）。
    fn begin_win(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        i: usize,
    ) -> Option<egui::Rect> {
        let name = self.wins[i].title.clone();
        if self.wins[i].minimized {
            // 最小化那条小标题栏：只剩一条标题栏和两个键（位置是堆出来的、不给拖也不给缩）。
            // 它就是「任务栏」上的一项：整块吃事件（免得点穿到下面的窗口上），一直能点。
            let rect = self.rect_of(i);
            let title = free_band(rect, 0, WIN_TITLE_ROWS);
            let painter = ui.painter().clone();
            interact_rect(ui, rect, egui::Sense::click_and_drag());
            let hit = paint_title_bar(
                app,
                ui,
                &painter,
                title,
                self.wins[i].w.id,
                &name,
                WinKeys::MinAndClose,
                false,
            );
            match hit {
                Some(SysButton::Minimize) => self.toggle_minimize(app, i),
                Some(SysButton::Close) => self.close_win(app, i),
                // 最小化那条上没有最大化键（它已经是「压成一条」了）。
                Some(SysButton::Maximize) => {}
                None => {}
            }
            return None;
        }
        // 外壳前半整份交给窗口三件套（底色 / 吃事件的整块 / 标题栏 / 拖动 / 两个键）。
        let work = self.work;
        let out = self.wins[i]
            .w
            .begin(app, ui, work, &name, WinKeys::MinMaxClose);
        if out.close {
            self.close_win(app, i);
        } else if out.maximize {
            self.wins[i].w.toggle_maximize(work);
        } else if out.minimize {
            self.toggle_minimize(app, i);
        }
        // 内容区 = 整块窗口让开一条标题栏（图表栏 / 面板那一列按钮都从这条线往下排）。
        out.panel.map(ManagedWin::body)
    }

    /// 外壳后半：缩放命中带（四条边 + 四个角）+ 标题栏下沿那条线 + 最外圈那条 1px 硬边框。
    /// 必须排在内容之后调：边框和那条线才压得住内容，缩放那几条带子才抢得到鼠标。
    fn end_win(&mut self, ui: &mut egui::Ui, i: usize) {
        if self.wins[i].minimized {
            // 最小化那条：没内容也没缩放，画完标题栏直接收边（下沿那条线就是它的下边框）。
            let rect = self.rect_of(i);
            paint_shell_border(&ui.painter().clone(), rect);
        } else {
            self.wins[i].w.end(ui);
        }
    }
}
