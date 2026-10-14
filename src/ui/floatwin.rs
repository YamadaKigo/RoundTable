// ===========================================================================
//  模块 ui::floatwin：浮动窗口（全工程统一的窗口外壳）
// ===========================================================================
//  主窗口里那些「自绘浮动窗口」（颜色设置 / 源码查看 / 以后的各种窗口）统一走这一套外壳，
//  不许再各写一份 - 铁律：同类型的东西必须长一个样（边框、标题栏、那条下沿分隔线一个不落）。
//
//  一套外壳 = 这几样（缺一样都算没走外壳）：
//    - 本体：底色 + 最外圈那条 1px 硬边框（边框由 end 排在内容之后画，才不会被内容盖掉）；
//    - 标题栏（2 格 = 一个文本行盒）：左文字（左右各留 1 格空档，走 grid 的 text_left_x）、
//      右关闭键 cells(2) 见方（和主窗口标题栏那个关闭键同一套外观）；
//    - 标题栏下沿那条 1px 横线：它就是标题栏和下面第一段共用的那条网格线，必须画；
//    - 拖动：只认标题栏那一段（关闭键左边那截），整块按最近的网格线走 - 一次一格（9px），
//      不做 1px 跟手的平滑移动。平滑移动时窗口在动、内部文字却被 grid_floor 吸在绝对网格上，
//      看上去文字在窗口里来回漂、松手才归位；按格走就没这个问题（位置永远在网格线上）；
//    - 位置永远落在窗口的 9px 网格线上，并且整块夹在窗口里 - 窗口自己的边框、内部分段线
//      因而全都和窗口的网格线重合（面板里不可能有半格的边框）。
//
//  用法（每个窗口的固定套路）：状态整份 clone 到本地变量 -> begin -> 画内容 -> end -> 写回。
//    match win.begin(self, ui, layout, "颜色设置") {
//        FloatShell::Closed => { /* 关闭键被点了：调用方自己收尾 */ }
//        FloatShell::Open(panel) => { 画内容; win.end(&painter); }
//    }
use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::titlebar::*;
use eframe::egui;

/// 一块浮动窗口的全部跨帧状态：开关、位置、尺寸、拖动会话。
/// egui 是即时模式，这些不存字段就跨不过一帧。位置只有两个口径：在网格线上、或者正在拖。
#[derive(Clone)]
pub(crate) struct FloatWindow {
    pub(crate) open: bool,
    /// 窗口左上角：任何一帧都落在窗口的 9px 网格线上（拖动中也是 - 按最近的那条线走）。
    pub(crate) pos: egui::Pos2,
    /// 窗口尺寸：一定是 cells(n) 的形状（宽高都满足 (v-1) % 9 == 0）。
    pub(crate) w: f32,
    pub(crate) h: f32,
    /// 正在拖标题栏。
    pub(crate) drag: bool,
    /// 按下那一刻鼠标相对窗口左上角的偏移：拖动时按它算，窗口不会跳。
    pub(crate) grab: egui::Vec2,
}

/// begin 的结果：窗口要关了，还是开着（开着就把整块本体交给调用方排内容）。
/// 内容那边一律按 panel 自己算段 - 第 row 条网格线的偏移就是 grid_line(row)，
/// 标题栏占前 WIN_TITLE_ROWS 格，内容从 grid_line(WIN_TITLE_ROWS) 那条线起。
pub(crate) enum FloatShell {
    /// 这一帧点了关闭键（调用方负责收尾：颜色面板 = 取消，源码窗 = 直接关）。
    Closed,
    /// 窗口开着：panel = 整块本体（绝对坐标，左上角落在窗口的网格线上）。
    Open(egui::Rect),
}

impl FloatWindow {
    /// 新建一块窗口：位置必须是网格线上的点（i*9）。
    pub(crate) fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        debug_assert!((x as i32) % 9 == 0 && (y as i32) % 9 == 0, "浮动窗口的初始位置要落在网格线上");
        debug_assert!((w as i32 - 1) % 9 == 0 && (h as i32 - 1) % 9 == 0, "浮动窗口的尺寸要满足 cells(n)");
        Self {
            open: false,
            pos: egui::pos2(x, y),
            w,
            h,
            drag: false,
            grab: egui::Vec2::ZERO,
        }
    }

    /// 窗口本体（相对窗口原点，绝对坐标在窗口的网格线上）。
    pub(crate) fn panel(&self) -> egui::Rect {
        egui::Rect::from_min_size(self.pos, egui::vec2(self.w, self.h))
    }

    /// 把左上角吸到窗口的 9px 网格线上，再整块夹回窗口里（每帧都过一道）。
    /// 拖动中也走它：所以窗口一格格地动，绝不会停在半格的像素上。
    pub(crate) fn snap_pos(&mut self, layout: egui::Rect) {
        let snap = |v: f32, origin: f32| origin + ((v - origin) / PITCH).round() * PITCH;
        let want_x = snap(self.pos.x, layout.min.x);
        let want_y = snap(self.pos.y, layout.min.y);
        let max_x = snap((layout.max.x - self.w).max(layout.min.x), layout.min.x);
        let max_y = snap((layout.max.y - self.h).max(layout.min.y), layout.min.y);
        self.pos.x = clamp_soft(want_x, layout.min.x, max_x);
        self.pos.y = clamp_soft(want_y, layout.min.y, max_y);
    }

    /// 画外壳 + 处理拖动 / 关闭：底色 -> 吃事件的整块矩形 -> 标题栏 -> 标题栏下沿那条线。
    /// 外框不在这儿画（见 end）：边框得压在内容之上，所以排在内容之后。
    pub(crate) fn begin(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        layout: egui::Rect,
        title_text: &str,
    ) -> FloatShell {
        // 每帧先把位置收在网格线上：拖动中也是一格格地动，松手不需要「归位」这一步。
        self.snap_pos(layout);
        if !ui.input(|i| i.pointer.primary_down()) {
            self.drag = false;
        }

        let painter = ui.painter().clone();
        let panel = self.panel();

        // 0) 底色 + 一整块「只吃事件、不画东西」的矩形：本函数是每帧最后跑的，底下的图表 /
        //    控制面板都抢不到这块地方（在窗口空白处按下不会把 K 线拖走）；窗口自己的控件
        //    又排在这块之后，照样点得到。
        painter.rect_filled(panel, egui::CornerRadius::ZERO, theme().work_bg);
        ui.allocate_rect(panel, egui::Sense::click_and_drag());

        let pointer = ui.input(|i| i.pointer.latest_pos());

        // 1) 标题栏：横贯整个窗口宽，左文字 + 右关闭键，中间那一段留给拖动。
        let title = grid_band(panel, 0, WIN_TITLE_ROWS);
        painter.rect_filled(title, egui::CornerRadius::ZERO, theme().title_bg);
        app.text(
            &painter,
            egui::pos2(text_left_x(title.min.x), title.center().y),
            egui::Align2::LEFT_CENTER,
            title_text,
            theme().text,
        );
        let close = egui::Rect::from_min_size(
            egui::pos2(panel.max.x - WIN_CLOSE_W, panel.min.y),
            egui::vec2(WIN_CLOSE_W, WIN_CLOSE_W),
        );
        let drag_bar = egui::Rect::from_min_max(title.min, egui::pos2(close.min.x, title.max.y));

        // 拖标题栏 = 挪窗口：会话记在 self.drag 里，位置每帧吸回网格线（一格格地动）。
        let drag_resp = ui.allocate_rect(drag_bar, egui::Sense::click_and_drag());
        if drag_resp.is_pointer_button_down_on() && !self.drag {
            if let Some(p) = pointer {
                self.drag = true;
                self.grab = p - panel.min;
            }
        }
        if self.drag {
            if let Some(p) = pointer {
                self.pos = p - self.grab;
                self.snap_pos(layout);
            }
        }

        // 关闭键：和主窗口标题栏那个关闭键同一套外观（hover 变红、叉叉变白）。
        let close_resp = ui.allocate_rect(close, egui::Sense::click());
        let close_hot = close_resp.hovered();
        painter.rect_filled(close, egui::CornerRadius::ZERO, close_key_face(close_hot));
        border_hard(&painter, close, theme().ctrl_border);
        paint_cross(
            &painter,
            egui::pos2(close.min.x + CELL_CENTER, close.min.y + CELL_CENTER),
            4,
            close_key_symbol(close_hot),
        );

        // 2) 标题栏下沿那条 1px 横线：它就是标题栏和下一段共用的那条网格线。
        //    铁律：同类型的窗口必须长一个样，颜色设置窗口有这条线，别的窗口就也得有。
        hline_hard(
            &painter,
            panel.min.x,
            panel.max.x,
            title.max.y - LINE_W,
            theme().ctrl_border,
        );

        if close_resp.clicked() {
            return FloatShell::Closed;
        }
        FloatShell::Open(panel)
    }

    /// 画最外圈那条 1px 硬边框 + 把标题栏下沿那条横线补一遍：排在内容之后调用。
    /// 窗口里的分段底色都是从「那条线格」开始铺的（和别的区域共用这 1px），先画的线会被啃掉一截 -
    /// 所以线和边框一样，都得压在内容之上重新来一遍，整块窗口的下沿才是一条完整的直线。
    pub(crate) fn end(&self, painter: &egui::Painter) {
        let panel = self.panel();
        let title = grid_band(panel, 0, WIN_TITLE_ROWS);
        hline_hard(
            painter,
            panel.min.x,
            panel.max.x,
            title.max.y - LINE_W,
            theme().ctrl_border,
        );
        border_hard(painter, panel, theme().ctrl_border);
    }
}