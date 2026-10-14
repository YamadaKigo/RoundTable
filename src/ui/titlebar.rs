// ===========================================================================
//  模块 ui::titlebar：标题栏
// ===========================================================================
//  程序名 + 中间可拖拽区 + 右侧两个自绘系统按钮（最小化 / 关闭）。
//  本文件 = 拆分前那份单文件里的「六、标题栏自绘系统按钮」+ 第七节里的 ui_titlebar。
use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::menu::*;
use eframe::egui;

// ===========================================================================
//  六、标题栏自绘系统按钮
// ===========================================================================

/// 标题栏右侧两个按钮：最小化、关闭。
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum SysButton {
    Minimize,
    Close,
}

impl SysButton {
    /// 在 19x19（cells(2)）的按钮矩形里手绘符号：纯色填充 + 符号，无圆角、无阴影、无渐变。
    /// 符号都以 2 格内容区的正中（CELL_CENTER = 9）为中心，端点全是整数像素，没有抗锯齿。
    pub(crate) fn paint_symbol(&self, painter: &egui::Painter, rect: egui::Rect, symbol: egui::Color32) {
        // 符号全按整像素算：先把按钮框吸到整数坐标。
        let rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x.round(), rect.min.y.round()),
            rect.size(),
        );
        // 内容区正中：横向 / 纵向都是「1px 边框 + 8px」= 9。
        let cx = rect.min.x + CELL_CENTER;
        let cy = rect.min.y + CELL_CENTER;

        match self {
            SysButton::Minimize => {
                // 一条横线：8px 宽（一条内容格），以内容正中为中心（左右各让 4px），竖直取正中那一行。
                let bar = egui::Rect::from_min_size(
                    egui::pos2(cx - GRID / 2.0, cy),
                    egui::vec2(GRID, LINE_W),
                );
                painter.rect_filled(bar, egui::CornerRadius::ZERO, symbol);
            }
            SysButton::Close => {
                // 叉以内容正中为中心、半径 4 -> 9x9。
                // paint_cross 保证中心只有 1 个像素、四条臂关于它左右上下严格对称。
                paint_cross(painter, egui::pos2(cx, cy), 4, symbol);
            }
        }
    }

}

/// 关闭键的底色（主窗口标题栏 / 浮动窗口标题栏共用这一份 - 铁律：同类型的东西必须长一个样）：
/// 常态就是普通按钮底，鼠标压上去一律变红（theme 的 close_hover）。
pub(crate) fn close_key_face(hot: bool) -> egui::Color32 {
    if hot {
        theme().close_hover
    } else {
        theme().btn_face
    }
}

/// 关闭键上那个叉的颜色：hover 时反白（红底上看得清），常态用普通文字色。
pub(crate) fn close_key_symbol(hot: bool) -> egui::Color32 {
    if hot {
        theme().text_white
    } else {
        theme().text
    }
}

/// 标题栏右侧两个系统按钮的矩形，从右往左排：最右边那个贴着窗口右边框，
/// 相邻两个共用中间那条网格线 - 各自都是 cells(2) = 19x19，中间永远只有 1px。
/// 返回顺序是从左往右：最小化、关闭（[0] 是最左边那个）。
pub(crate) fn sys_button_rects(bar: egui::Rect) -> [egui::Rect; 2] {
    let mut rects = [egui::Rect::NOTHING; 2];
    // 最右按钮的右边缘 = 窗口右边框那条网格线 + 1px。
    let mut right = bar.max.x;
    for i in (0..2).rev() {
        rects[i] = egui::Rect::from_min_size(
            egui::pos2(right - SYS_BTN_W, bar.min.y),
            egui::vec2(SYS_BTN_W, SYS_BTN_H),
        );
        // 下一个按钮的右边缘 = 当前按钮左边那条网格线（相邻两个共用这 1px）。
        right = rects[i].min.x + LINE_W;
    }
    rects
}

impl StockApp {
    /// 标题栏：程序名 + 中间可拖拽区 + 右侧两个自绘按钮（最小化 / 关闭）。
    pub(crate) fn ui_titlebar(
        &mut self,
        ui: &mut egui::Ui,
        bar: egui::Rect,
        outer: Option<egui::Rect>,
    ) {
        // Context / Painter 内部都是 Arc，克隆很便宜，克隆一份就不怕借用冲突。
        let ctx = ui.ctx().clone();
        let painter = ui.painter().clone();

        // 程序名和菜单栏第一项用同一条排版规则：同样宽（menu::tab_width）、同样居中。
        // 两个名字都是两个汉字、宽度一样，所以「圆桌」和它正下方的「文件」逐像素对齐。
        let name_slot = egui::Rect::from_min_size(
            bar.min,
            egui::vec2(tab_width(self, APP_NAME), bar.height()),
        );
        self.text(
            &painter,
            name_slot.center(),
            egui::Align2::CENTER_CENTER,
            APP_NAME,
            theme().text,
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
        // 每个按钮都是 cells(2) = 19x19（和标题栏一样高）；相邻两个共用中间那条网格线。
        // 四条边一律全画：上边压在窗口顶边框上、下边压在区域分隔线上、右边压在窗口右边框上，
        // 反复画同一条 1px 线不会变粗（后画的压在上面，颜色以最后画的为准）。
        let kinds = [SysButton::Minimize, SysButton::Close];
        for (i, kind) in kinds.iter().enumerate() {
            let rect = buttons[i];
            let resp = ui.interact(rect, egui::Id::new(("rt_sys_btn", i)), egui::Sense::click());
            let hovered = resp.hovered();
            // 关闭键走「红底 + 反白叉」那一套（和浮动窗的关闭键共用 close_key_face / close_key_symbol）；
            // 最小化键还是面板里通用的高亮色，不跟着变红。
            let (face, symbol) = match kind {
                SysButton::Close => (close_key_face(hovered), close_key_symbol(hovered)),
                SysButton::Minimize => (
                    if hovered { theme().btn_hover } else { theme().btn_face },
                    theme().text,
                ),
            };

            painter.rect_filled(rect, egui::CornerRadius::ZERO, face);
            border_hard(&painter, rect, theme().ctrl_border);
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
}
