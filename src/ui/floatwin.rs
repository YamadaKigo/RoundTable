// ===========================================================================
//  模块 ui::floatwin：窗口（全工程唯一的窗口外壳 + 窗口「三件套」）
// ===========================================================================
//  窗口只有一个类：Win。窗口区里的每一块子窗口（图表窗 / 控制面板 / 源码编辑窗）和浮在
//  整界面之上的颜色面板（唯一一块「子窗口的子窗口」，开自控制面板）全都用它，
//  不许再各写一份 - 铁律：同类型的东西必须长一个样。
//
//  一套外壳 = 这几样（缺一样都算没走外壳）：
//    - 本体：底色 + 最外圈那条 1px 硬边框（边框由 end 排在内容之后画，才不会被内容盖掉）；
//    - 标题栏（2 格 = 一个文本行盒）：左文字（左边留 1 格空档，走 grid 的 text_left_x）；
//      右端系统键 cells(2) 见方（和主窗口标题栏那两个键同一套外观，见 paint_title_bar）；
//    - 标题栏下沿那条 1px 横线：它就是标题栏和下面第一段共用的那条网格线。必须由 end 排在
//      内容之后画 - 内容底色是从那条线起铺的，先画的线会被啃掉一截；
//    - 拖动：只认标题栏那一段（系统键左边那截），1px 跟手（不吸 9px 网格）；
//      鼠标压上去给十字箭头光标（和缩放那几条边一样，先告诉你「这儿能拖」）；
//    - 缩放：四条边 + 四个角都能抓（左边 / 上边也能往左 / 往上缩），鼠标压上去给对应方向的
//      箭头光标，1px 跟手，对边的边缘钉死在按下那一刻的位置上，缩到 min_size 为止、
//      整块夹在 bounds 里；
//    - 静默：只有最前面那块（激活的）窗口收鼠标，别的块连 hover 都不算数，点一下才激活
//      （见 ui::winmgr 的层级规则）。
//
//  每块窗口自带一套 9px 网格：原点 = 这块窗口自己的左上角。窗口可以 1px 1px 地挪 / 缩，
//  所以它跟主窗口的网格不再重合 - 内部元素一律按自己那套网格排（grid 的吸附入口都带 origin），
//  窗口动的时候里头的东西跟着一起动、相对位置一个像素都不变。
//
//  用法（每个窗口的固定套路）：状态整份 clone 到本地变量 -> begin -> 画内容 -> end -> 写回。
//    let out = win.begin(self, ui, bounds, "颜色设置", WinKeys::CloseOnly);
//    if out.close { /* 关闭键被点了：调用方自己收尾 */ }
//    if let Some(panel) = out.panel { 画内容; win.end(ui); }
//  （窗口区里的子窗口那个 input 由 ui::winmgr 按层级算：只有最前面那块是 Enabled。）
use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::titlebar::*;
use eframe::egui;

/// 缩放命中带的半宽：贴着窗口四条边和四个角，里外各这么多像素。
pub(crate) const EDGE_BAND: f32 = 4.0;

/// 颜色面板那块浮动窗的固定窗口 id：窗口区里的子窗口从 1 开始编号，两个编号错开一大截，
/// egui 的热区 id 撞不上。
pub(crate) const WIN_ID_COLOR: u64 = 0x43_4c_52;

/// 一块窗口的全部跨帧状态：开关、位置、尺寸、拖动会话、缩放会话。
/// egui 是即时模式，这些不存字段就跨不过一帧。
#[derive(Clone)]
pub(crate) struct Win {
    /// 稳定的 id：egui 的热区 id 用它（层级一挪，自动 id 就会错位，按一半的点击会丢）。
    pub(crate) id: u64,
    pub(crate) open: bool,
    /// 窗口左上角：自由像素，1px 1px 地挪（不吸任何网格）。
    pub(crate) pos: egui::Pos2,
    /// 窗口尺寸：自由像素，1px 1px 地缩（不吸任何网格）。
    pub(crate) size: egui::Vec2,
    /// 缩放的下限：窗口里的控件全放得下为止，再小就不让缩了。
    pub(crate) min_size: egui::Vec2,
    /// 能不能拉四条边 / 四个角改尺寸（颜色面板这种改了也没意义的就关掉）。
    pub(crate) resizable: bool,
    /// 最大化中：整块铺满 bounds（窗口区），不给拖也不给缩。
    pub(crate) maximized: bool,
    /// 最大化那一刻的位置尺寸：再点一次最大化键就还回去。
    max_restore: (egui::Pos2, egui::Vec2),
    /// 正在拖标题栏：按下那一刻鼠标相对窗口左上角的偏移（按它算，窗口不会跳）。
    drag: Option<egui::Vec2>,
    /// 正在缩放：抓住的是哪一处（None = 没在缩）。
    rs: WinEdge,
    /// 按下那一刻的窗口矩形：左上角那颗钉子以它为准，拖得再快也不会飘。
    rs_fix: egui::Rect,
    /// 抓住的那条边相对鼠标的偏移（按住那一刻算的）：边一律按「鼠标 + 这个偏移」现算。
    /// 顶到 min_size / bounds 上（或鼠标越过窗口区边界）就把这个偏移重新钉一次 -
    /// 多出来的那段拖拽不算数，所以往回拉 1px 边立刻跟 1px，不会先「拉回一段空距离」。
    rs_off: egui::Vec2,
}

/// 缩放抓的是窗口的哪一处：四条边 + 四个角都能抓（左边 / 上边也能往左 / 往上缩）。
#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) enum WinEdge {
    #[default]
    None,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl WinEdge {
    pub(crate) fn any(self) -> bool {
        self != WinEdge::None
    }

    /// 这一处带不带左边那条边（左 / 左上 / 左下）。
    fn left(self) -> bool {
        matches!(self, WinEdge::Left | WinEdge::TopLeft | WinEdge::BottomLeft)
    }

    /// 这一处带不带右边那条边（右 / 右上 / 右下）。
    fn right(self) -> bool {
        matches!(self, WinEdge::Right | WinEdge::TopRight | WinEdge::BottomRight)
    }

    /// 这一处带上边那条边（上 / 左上 / 右上）。
    fn top(self) -> bool {
        matches!(self, WinEdge::Top | WinEdge::TopLeft | WinEdge::TopRight)
    }

    /// 这一处带下边那条边（下 / 左下 / 右下）。
    fn bottom(self) -> bool {
        matches!(self, WinEdge::Bottom | WinEdge::BottomLeft | WinEdge::BottomRight)
    }

    /// 鼠标压在这一处时该显示的光标：四条边 / 两组对角各一个方向的箭头。
    fn cursor(self) -> Option<egui::CursorIcon> {
        match self {
            WinEdge::None => None,
            WinEdge::Left | WinEdge::Right => Some(egui::CursorIcon::ResizeHorizontal),
            WinEdge::Top | WinEdge::Bottom => Some(egui::CursorIcon::ResizeVertical),
            WinEdge::TopLeft | WinEdge::BottomRight => Some(egui::CursorIcon::ResizeNwSe),
            WinEdge::TopRight | WinEdge::BottomLeft => Some(egui::CursorIcon::ResizeNeSw),
        }
    }
}

/// 窗口标题栏右端要画哪几个系统键。
/// 窗口区里的窗口（图表窗 / 控制面板）和源码编辑窗都是三个（最小化 + 最大化 + 关闭）；
/// 最小化之后那条小标题栏只有两个（还原 + 关闭 - 它已经是「压成一条」了，再最大化没意义）；
/// 颜色面板这种不缩不最大化的只有一个关闭键。
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum WinKeys {
    /// 只有一个关闭键（颜色面板）。
    CloseOnly,
    /// 还原 + 关闭（最小化之后那条小标题栏）。
    MinAndClose,
    /// 最小化 + 最大化 + 关闭（正常窗口）。
    MinMaxClose,
}

impl WinKeys {
    /// 标题栏右端这一组键：每一项 =（哪个键，占哪个格子）。
    /// 格子下标就是 `sys_button_rects` 的下标（[0] 最小化、[1] 最大化、[2] 关闭，从右往左排），
    /// 返回顺序从左往右 - 所以「关闭」永远在最右边、它左边永远挨着「最大化」（如果有的话）。
    fn shown(self) -> &'static [(SysButton, usize)] {
        match self {
            // 只有一个键：就放在最右边那个格子里（贴着窗口右边框）。
            WinKeys::CloseOnly => &[(SysButton::Close, 2)],
            // 两个键：还原 + 关闭，用最右边那两个格子（和以前那套两键摆法一模一样）。
            WinKeys::MinAndClose => &[(SysButton::Minimize, 1), (SysButton::Close, 2)],
            WinKeys::MinMaxClose => &[
                (SysButton::Minimize, 0),
                (SysButton::Maximize, 1),
                (SysButton::Close, 2),
            ],
        }
    }
}

/// begin 的结果：这一帧点了哪个键、内容可以画在哪。
/// 给的是整块窗口（含标题栏）：内容那边自己按 grid_line(WIN_TITLE_ROWS) 往下让开一条标题栏
/// （浮动的两块窗拿整块当坐标基准；窗口区里的窗口用 free_tail 取内容区，见 ManagedWin::body）。
#[derive(Clone, Copy)]
pub(crate) struct WinOut {
    /// 整块窗口；关掉的时候是 None。
    pub(crate) panel: Option<egui::Rect>,
    pub(crate) minimize: bool,
    pub(crate) maximize: bool,
    pub(crate) close: bool,
}

/// 光标压在窗口的哪一处缩放位上（四条边 / 四个角，各带一条 EDGE_BAND 宽的命中带）。
/// 角 = 同时挨着两条边；两条边都够不着就是 None。
pub(crate) fn edge_at(rect: egui::Rect, p: egui::Pos2, band: f32) -> WinEdge {
    let left = p.x >= rect.min.x - band && p.x <= rect.min.x + band;
    let right = p.x >= rect.max.x - band && p.x <= rect.max.x + band;
    let top = p.y >= rect.min.y - band && p.y <= rect.min.y + band;
    let bottom = p.y >= rect.max.y - band && p.y <= rect.max.y + band;
    // 边也得落在窗口的跨度里（含命中带）：否则窗口外远处那一段横线上也算「贴着左边」。
    let in_x = p.x >= rect.min.x - band && p.x <= rect.max.x + band;
    let in_y = p.y >= rect.min.y - band && p.y <= rect.max.y + band;
    match (left && in_y, right && in_y, top && in_x, bottom && in_x) {
        (true, _, true, _) => WinEdge::TopLeft,
        (_, true, true, _) => WinEdge::TopRight,
        (true, _, _, true) => WinEdge::BottomLeft,
        (_, true, _, true) => WinEdge::BottomRight,
        (true, _, _, _) => WinEdge::Left,
        (_, true, _, _) => WinEdge::Right,
        (_, _, true, _) => WinEdge::Top,
        (_, _, _, true) => WinEdge::Bottom,
        _ => WinEdge::None,
    }
}

/// 缩放时鼠标坐标 -> 那条边该落哪一行 / 哪一列：压到 bounds 的边缘附近（一个命中带内）
/// 就直接贴齐那条边。
/// 为什么要贴：主窗口右边那条边框就是整个客户区的最后一列，鼠标最多只能停在它上面
/// （再往右是显示器外面），不贴的话窗口右边框永远差 1px，到不了和主窗口边框重叠的位置。
fn edge_target(v: f32, lo: f32, hi: f32, band: f32) -> f32 {
    if v <= lo + band {
        lo
    } else if v >= hi - band {
        hi
    } else {
        v.round()
    }
}

impl Win {
    /// 新建一块窗口：初始位置 / 尺寸（下限也先按当前尺寸）、默认不可缩放、默认关着。
    pub(crate) fn new(id: u64, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            id,
            open: false,
            pos: egui::pos2(x, y),
            size: egui::vec2(w, h),
            min_size: egui::vec2(w, h),
            resizable: false,
            maximized: false,
            max_restore: (egui::pos2(x, y), egui::vec2(w, h)),
            drag: None,
            rs: WinEdge::None,
            rs_fix: egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
            rs_off: egui::Vec2::ZERO,
        }
    }

    /// 窗口本体（自由像素）。
    pub(crate) fn rect(&self) -> egui::Rect {
        egui::Rect::from_min_size(self.pos, self.size)
    }

    /// 标题栏那一段（2 格）。
    pub(crate) fn title_band(&self) -> egui::Rect {
        free_band(self.rect(), 0, WIN_TITLE_ROWS)
    }

    /// 正在拖标题栏。
    pub(crate) fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// 正在缩放。
    pub(crate) fn resizing(&self) -> bool {
        self.rs.any()
    }

    /// 会话结束（松手）：拖动 / 缩放都归零。
    pub(crate) fn end_sessions(&mut self) {
        self.drag = None;
        self.rs = WinEdge::None;
    }

    /// 手动开 / 关一次拖动会话（测试和宿主自己模拟「正拖着这块窗」时用；
    /// 真拖动走 begin 里的 drag_title）。
    #[allow(dead_code)]
    pub(crate) fn set_dragging(&mut self, on: bool) {
        self.drag = if on { Some(egui::Vec2::ZERO) } else { None };
    }

    /// 把窗口收进 bounds 里：尺寸不许大过 bounds，位置不许出去（每帧都过一遍）。
    pub(crate) fn clamp_into(&mut self, bounds: egui::Rect) {
        self.size.x = clamp_soft(self.size.x, self.min_size.x.min(bounds.width()), bounds.width());
        self.size.y = clamp_soft(self.size.y, self.min_size.y.min(bounds.height()), bounds.height());
        self.pos.x = clamp_soft(self.pos.x, bounds.min.x, bounds.max.x - self.size.x);
        self.pos.y = clamp_soft(self.pos.y, bounds.min.y, bounds.max.y - self.size.y);
    }

    /// 最大化键：最大化（铺满 bounds = 窗口区）/ 再点一次还原回原来的位置尺寸。
    /// 记下来的那份「原来」在还原时还要再夹一次（窗口区可能在这中间变小了）。
    pub(crate) fn toggle_maximize(&mut self, bounds: egui::Rect) {
        if self.maximized {
            self.maximized = false;
            self.pos = self.max_restore.0;
            self.size = self.max_restore.1;
            self.clamp_into(bounds);
        } else {
            self.max_restore = (self.pos, self.size);
            self.maximized = true;
            self.pos = bounds.min;
            self.size = bounds.size();
        }
        self.end_sessions();
    }

    /// 画外壳前半段 + 处理拖动 / 标题栏那几个键：底色 -> 吃事件的整块矩形 -> 标题栏 -> 拖动。
    /// 最外圈边框和标题栏下沿那条线不在这儿画（见 end）：它们得压在内容之上。
    ///
    /// 【几何先算、画在后】拖动 / 缩放都在动笔之前把位置尺寸定下来（见 update_geometry），
    /// 之后的标题栏、内容（调用方接着画）、外框全部取自同一个矩形 - 整块窗口是「铁板一块」。
    /// 反过来（内容先按旧矩形画、外框又按新矩形画）看上去就是：标题栏慢半拍、外框和内容错位、
    /// 错位那条缝里露出窗口区的底色（一块白色的错位带），连本该重合的 1px 线也会错开成 2px。
    pub(crate) fn begin(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        bounds: egui::Rect,
        title_text: &str,
        keys: WinKeys,
    ) -> WinOut {
        // 0) 这一帧的几何：按会话和鼠标这一刻的落点算准，1px 跟手。
        self.update_geometry(ui, bounds);
        let rect = self.rect();

        let painter = ui.painter().clone();

        // 1) 底色 + 一整块「只吃事件、不画东西」的矩形：底下的图表 / 控制面板抢不到这块地方
        //    （在窗口空白处按下不会把 K 线拖走）；窗口自己的控件又排在这块之后，照样点得到。
        painter.rect_filled(rect, egui::CornerRadius::ZERO, theme().work_bg);
        interact_rect(ui, rect, egui::Sense::click_and_drag());

        // 2) 标题栏 + 右端那几个系统键；拖动把手就是键左边那一段。
        let title = self.title_band();
        let keys_rects = sys_button_rects(title);
        let handle = egui::Rect::from_min_max(
            title.min,
            egui::pos2(keys_rects[0].min.x.max(title.min.x), title.max.y),
        );
        let hit = paint_title_bar(
            app,
            ui,
            &painter,
            title,
            self.id,
            title_text,
            keys,
            self.maximized,
        );
        // 3) 拖标题栏：按在把手上就开一次会话（偏移记下来，几何从下一帧起由几何那一步跟）。
        self.drag_title(ui, handle);

        WinOut {
            panel: Some(rect),
            minimize: hit == Some(SysButton::Minimize),
            maximize: hit == Some(SysButton::Maximize),
            close: hit == Some(SysButton::Close),
        }
    }

    /// 这一帧的位置尺寸（一律在画之前调）：松手就收会话 -> 拖动按按下时记的偏移把窗口挪到
    /// 鼠标位置 -> 缩放把抓住的那条边挪到鼠标位置 -> 最后整块夹进 bounds 里。
    /// 算出来的几何这一帧就生效，所以窗口的每一项（标题栏 / 内容 / 外框）看到的都是同一个矩形。
    fn update_geometry(&mut self, ui: &mut egui::Ui, bounds: egui::Rect) {
        // 松手了：拖动 / 缩放会话一起结束。
        if !ui.input(|i| i.pointer.primary_down()) {
            self.end_sessions();
        }
        // 最大化中：铺满 bounds，不给拖也不给缩（它就是整块窗口区）。
        if self.maximized {
            self.end_sessions();
            self.pos = bounds.min;
            self.size = bounds.size();
            return;
        }
        let pointer = ui.input(|i| i.pointer.latest_pos());
        if let (Some(grab), Some(p)) = (self.drag, pointer) {
            let want = p - grab;
            let at = egui::pos2(
                clamp_soft(want.x, bounds.min.x, bounds.max.x - self.size.x),
                clamp_soft(want.y, bounds.min.y, bounds.max.y - self.size.y),
            );
            // 顶到窗口区边上了（鼠标比窗口跑得快）：把「按下时的偏移」重新钉一次，
            // 多出来的那段拖拽就不算数了 - 往回拉 1px 窗口立刻跟 1px。
            if at != want {
                self.drag = Some(p - at);
            }
            self.pos = at;
        }
        if self.rs.any() {
            if let Some(p) = pointer {
                self.stretch(p, bounds);
            }
        }
        // 每帧再夹一次：窗口区变小 / 窗口变多之后，谁都出不去。
        self.clamp_into(bounds);
    }

    /// 画外壳后半段：缩放命中带（右 / 下 / 右下角）-> 标题栏下沿那条线 -> 最外圈 1px 硬边框。
    /// 必须排在窗口内容之后调：边框和那条线才压得住内容，缩放那几条带子才抢得到鼠标。
    ///
    /// 标题栏下沿那条线为什么得补一遍：窗口里的分段底色都是从「那条线格」开始铺的
    /// （和别的区域共用这 1px），先画的线会被啃掉一截 - 所以线和边框一样，都得压在内容之上
    /// 重新来一遍，整块窗口的下沿才是一条完整的直线。
    pub(crate) fn end(&mut self, ui: &mut egui::Ui) {
        // 最大化中的窗口没有缩放命中带（先还原再说）。
        if !self.maximized {
            self.resize_grips(ui);
        }
        paint_shell_border(&ui.painter().clone(), self.rect());
    }

    /// 拖标题栏：鼠标按在把手上就开一次拖动会话，记下按下那一刻鼠标相对窗口左上角的偏移。
    /// 几何每帧按它现算（见 update_geometry），拖得再快也不飘、也不会累积误差。
    /// 鼠标压在把手上时给十字箭头光标：和四条边那几种缩放箭头一个意思，
    /// 先告诉你「这一块能拖着走」（不然标题栏看着跟内容区没差别）。
    fn drag_title(&mut self, ui: &mut egui::Ui, handle: egui::Rect) {
        let resp = ui.interact(
            handle,
            egui::Id::new(("rt_win_drag", self.id)),
            egui::Sense::click_and_drag(),
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
        }
        if resp.is_pointer_button_down_on() && self.drag.is_none() {
            if let Some(p) = ui.input(|i| i.pointer.latest_pos()) {
                // 按下那一刻鼠标相对窗口左上角的偏移：拖动时按它算，窗口不会跳。
                self.drag = Some(p - self.pos);
                self.rs = WinEdge::None;
            }
        }
    }

    /// 缩放命中带：四条边 + 四个角（左边 / 上边也能往左 / 往上缩）。鼠标压上去给对应方向的
    /// 箭头光标；按住拖就是改尺寸（1px 跟手），对边的边缘钉死、缩到 min_size 为止、
    /// 整块夹在 bounds 里。
    /// 这儿只管「开一次缩放会话 + 换光标」：改几何的活儿归 update_geometry（在画之前做）。
    fn resize_grips(&mut self, ui: &mut egui::Ui) {
        if !self.resizable {
            return;
        }
        let rect = self.rect();
        let b = EDGE_BAND;
        // 八块互不重叠的命中带：四个角各 b x b 一块，四条边各一条（两头让给角）。
        let band = |x0: f32, y0: f32, x1: f32, y1: f32| {
            egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1))
        };
        let bands = [
            (band(rect.min.x - b, rect.min.y - b, rect.min.x + b, rect.min.y + b), WinEdge::TopLeft),
            (band(rect.max.x - b, rect.min.y - b, rect.max.x + b, rect.min.y + b), WinEdge::TopRight),
            (band(rect.min.x - b, rect.max.y - b, rect.min.x + b, rect.max.y + b), WinEdge::BottomLeft),
            (band(rect.max.x - b, rect.max.y - b, rect.max.x + b, rect.max.y + b), WinEdge::BottomRight),
            (band(rect.min.x - b, rect.min.y + b, rect.min.x + b, rect.max.y - b), WinEdge::Left),
            (band(rect.max.x - b, rect.min.y + b, rect.max.x + b, rect.max.y - b), WinEdge::Right),
            (band(rect.min.x + b, rect.min.y - b, rect.max.x - b, rect.min.y + b), WinEdge::Top),
            (band(rect.min.x + b, rect.max.y - b, rect.max.x - b, rect.max.y + b), WinEdge::Bottom),
        ];
        let mut hot = WinEdge::None;
        let mut down = false;
        for (k, (band, edge)) in bands.into_iter().enumerate() {
            let resp = ui.interact(
                band,
                egui::Id::new(("rt_win_edge", self.id, k)),
                egui::Sense::click_and_drag(),
            );
            if resp.hovered() || resp.is_pointer_button_down_on() {
                hot = edge;
            }
            down |= resp.is_pointer_button_down_on();
        }
        if let Some(cursor) = hot.cursor() {
            ui.ctx().set_cursor_icon(cursor);
        }
        if !self.rs.any() && down {
            if let Some(p) = ui.input(|i| i.pointer.latest_pos()) {
                self.rs = edge_at(rect, p, b);
                self.rs_fix = rect;
                // 抓住的那条边（或那两条边）相对鼠标的偏移：按住那一刻算一次，
                // 之后边一律按「鼠标 + 偏移」现算（见 stretch）。
                self.rs_off = egui::vec2(
                    if self.rs.left() {
                        rect.min.x - p.x
                    } else if self.rs.right() {
                        rect.max.x - p.x
                    } else {
                        0.0
                    },
                    if self.rs.top() {
                        rect.min.y - p.y
                    } else if self.rs.bottom() {
                        rect.max.y - p.y
                    } else {
                        0.0
                    },
                );
            }
        }
    }

    /// 把抓住的那一条边（角就是两条边）挪到鼠标位置（1px 精度），再按 min_size / bounds 夹紧。
    /// 对边按 rs_fix（按下那一刻的矩形）钉死，拖得再快也不会飘。
    /// 边一律按「鼠标 + rs_off」现算（rs_off = 按住那一刻边与鼠标的偏移），
    /// 顶到 min_size / bounds 上、或鼠标压到窗口区边界（edge_target 贴齐那条边，窗口边框才能
    /// 和主窗口的边框重叠）就把偏移重新钉一次：多出来的那段拖拽不算数，
    /// 往回拉 1px 边立刻跟 1px。
    fn stretch(&mut self, p: egui::Pos2, bounds: egui::Rect) {
        let fix = self.rs_fix;
        let min = self.min_size;
        let mut x0 = fix.min.x;
        let mut x1 = fix.max.x;
        let mut y0 = fix.min.y;
        let mut y1 = fix.max.y;
        if self.rs.left() {
            // 左边跟手：右边那条边钉住。
            let want = edge_target(p.x + self.rs_off.x, bounds.min.x, bounds.max.x, EDGE_BAND);
            x0 = clamp_soft(want, bounds.min.x, (x1 - min.x).max(bounds.min.x));
            self.rs_off.x = x0 - p.x;
        } else if self.rs.right() {
            let want = edge_target(p.x + self.rs_off.x, bounds.min.x, bounds.max.x, EDGE_BAND);
            x1 = clamp_soft(want, (x0 + min.x).min(bounds.max.x), bounds.max.x);
            self.rs_off.x = x1 - p.x;
        }
        if self.rs.top() {
            // 上边跟手：下边那条边钉住。
            let want = edge_target(p.y + self.rs_off.y, bounds.min.y, bounds.max.y, EDGE_BAND);
            y0 = clamp_soft(want, bounds.min.y, (y1 - min.y).max(bounds.min.y));
            self.rs_off.y = y0 - p.y;
        } else if self.rs.bottom() {
            let want = edge_target(p.y + self.rs_off.y, bounds.min.y, bounds.max.y, EDGE_BAND);
            y1 = clamp_soft(want, (y0 + min.y).min(bounds.max.y), bounds.max.y);
            self.rs_off.y = y1 - p.y;
        }
        self.pos = egui::pos2(x0, y0);
        self.size = egui::vec2(
            (x1 - x0).max(LINE_W),
            (y1 - y0).max(LINE_W),
        );
    }
}

/// 标题栏：底色 + 左边文字 + 右端那几个系统键（外观 / 点击都在这一份里）。
/// 返回这一帧被点中的键。子窗口的标题栏、浮动窗的标题栏、最小化之后那条小标题栏都走它。
/// minimax = 这块窗口现在是不是最大化着（最大化键画方框、最大化中画「还原」那个双框）。
/// keys = 右端画哪几个键（见 WinKeys）：只有一个键的时候就摆在最右边那个格子里，
/// 两个 / 三个键从最右边往左连着摆 - 最大化永远夹在最小化和关闭中间。
/// 下沿那条线不在这儿画（见 paint_shell_border）：它得压在内容之上。
pub(crate) fn paint_title_bar(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    title: egui::Rect,
    id: u64,
    text: &str,
    keys: WinKeys,
    maximized: bool,
) -> Option<SysButton> {
    painter.rect_filled(title, egui::CornerRadius::ZERO, theme().title_bg);
    // 文字左边留 1 格空档（原点 = 这块窗口自己那套网格 = 标题栏自己的左边那条线）。
    app.text(
        painter,
        egui::pos2(text_left_x(title.min.x, title.min.x), title.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        theme().text,
    );

    // 系统键的格子固定三格：[0] 最小化、[1] 最大化、[2] 关闭（从右往左）。
    // 这块窗口要画哪几个由 keys.shown() 说了算（颜色面板只画关闭、最小化那条画还原 + 关闭）。
    let rects = sys_button_rects(title);
    let mut hit = None;
    for &(kind, slot) in keys.shown() {
        let rect = rects[slot];
        // 热区 id 按「这个键是干什么的」算（不是按格子）：最小化 / 还原是同一个 id，
        // 所以「同一点再点一下」照样能把压成条的窗口还原回来。
        let resp = ui.interact(
            rect,
            egui::Id::new(("rt_win_key", id, kind)),
            egui::Sense::click(),
        );
        let hot = resp.hovered();
        let (face, symbol) = match kind {
            SysButton::Close => (close_key_face(hot), close_key_symbol(hot)),
            SysButton::Minimize | SysButton::Maximize => (
                if hot { theme().btn_hover } else { theme().btn_face },
                theme().text,
            ),
        };
        painter.rect_filled(rect, egui::CornerRadius::ZERO, face);
        border_hard(painter, rect, theme().ctrl_border);
        // 最小化之后那条小标题栏上的键是「还原」：图标也得是还原那个双框（不是最小化那条横线）
        // - 铁律：图标画的必须是这个键点下去会干的事。
        let restore = matches!(keys, WinKeys::MinAndClose) && kind == SysButton::Minimize;
        if restore {
            paint_restore_symbol(
                painter,
                egui::pos2(rect.min.x + CELL_CENTER, rect.min.y + CELL_CENTER),
                symbol,
                face,
            );
        } else {
            // 最大化那个键按当前状态画（没最大化 = 正方形，最大化中 = 还原那个双框）。
            kind.paint_symbol_max(painter, rect, symbol, face, maximized);
        }
        // UI 工程测试：标题栏上的每个系统键也算一块元素。
        app.probe(
            match (kind, restore) {
                (SysButton::Minimize, true) => "还原键",
                (SysButton::Minimize, false) => "最小化键",
                (SysButton::Maximize, _) => "最大化键",
                (SysButton::Close, _) => "关闭键",
            },
            rect,
        );
        if resp.clicked() {
            hit = Some(kind);
        }
    }
    hit
}

/// 画外壳收尾：标题栏下沿那条 1px 横线 + 最外圈那条 1px 硬边框。
/// 一律由调用方排在内容之后调（最小化那条小标题栏没有内容，画完标题栏直接调它）。
pub(crate) fn paint_shell_border(painter: &egui::Painter, rect: egui::Rect) {
    hline_hard(
        painter,
        rect.min.x,
        rect.max.x,
        rect.min.y + grid_line(WIN_TITLE_ROWS),
        theme().ctrl_border,
    );
    border_hard(painter, rect, theme().ctrl_border);
}
