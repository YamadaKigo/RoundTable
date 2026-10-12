// ===========================================================================
//  模块 ui::titlebar：标题栏
// ===========================================================================
//  程序名 + 中间可拖拽区 + 右侧两个自绘系统按钮（最小化 / 关闭）。
//  本文件 = 拆分前那份单文件里的「六、标题栏自绘系统按钮」+ 第七节里的 ui_titlebar。
use crate::app::*;
use crate::grid::*;
use crate::theme::*;
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
    /// 在 28x28（cells(3)）的按钮矩形里手绘符号：纯色填充 + 符号，无圆角、无阴影、无渐变。
    /// 符号落在内容格和网格线上，端点全是整数像素，没有抗锯齿。
    pub(crate) fn paint_symbol(&self, painter: &egui::Painter, rect: egui::Rect, symbol: egui::Color32) {
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
/// 相邻两个共用中间那条网格线 —— 各自都是 cells(3) = 28x28，中间永远只有 1px。
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

        // 程序名和菜单栏第一项用同一条排版规则：同样宽（menu_item_width）、同样居中。
        // 两个名字都是两个汉字、宽度一样，所以「圆桌」和它正下方的「文件」逐像素对齐。
        let name_slot = egui::Rect::from_min_size(
            bar.min,
            egui::vec2(self.menu_item_width(APP_NAME), bar.height()),
        );
        self.text(
            &painter,
            name_slot.center(),
            egui::Align2::CENTER_CENTER,
            APP_NAME,
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
            // hover 一律用面板里通用的那个高亮色，不区分按钮。
            let face = if hovered { C_BTN_HOVER } else { C_BTN_FACE };
            let symbol = C_TEXT;

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
}
