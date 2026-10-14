// ===========================================================================
//  模块 ui::menu：菜单栏与下拉面板（全工程唯一一份）
// ===========================================================================
//  主窗口的菜单栏、源码编辑窗的菜单栏、图表上那些下拉面板（指标 / 叠加）全走这一份，
//  不许再各写一份 - 铁律：同类型的东西必须长一个样（菜单项宽 / 行高 / hover 高亮 /
//  边框 / 与邻居共用的那条 1px 网格线，一个都不能差）。
//
//  菜单栏（ui_menu_bar）：
//    - 左对齐：从区块左边那条网格线起、一项挨一项往右排（不右对齐、也不居中）；
//    - 每项宽 = 文字宽 + 左右各 1 格空档，吸到整数格（铁律：文字左右各留 1 格）；
//    - 相邻两项共用中间那条 1px 网格线（步进 = 宽 - 1px），高亮块碰不到邻居的边框；
//    - 下沿那条 1px 线 = 菜单栏和下一段共用的边界线，最后画（不然会被 hover 高亮啃掉一条）。
//  下拉面板（MenuPanel）：
//    - 上沿压在菜单栏 / 触发控件下沿那条网格线上（两段共用这 1px）；
//    - 每行 2 格（相邻两行共用中间那条网格线），1px 硬边框最后画；
//    - 两种对齐：down（左边和触发它的控件对齐）/ right（右边对齐，底下放不下就翻上去）。
use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use eframe::egui;

/// 菜单栏的一项：标题 + 展开后要画哪些条目。
pub(crate) struct MenuTab {
    pub(crate) title: &'static str,
    pub(crate) items: &'static [&'static str],
}

/// 一块菜单栏 / 下拉面板的状态：哪一项开着（None = 都收着）。
/// 菜单都是「同时只开一个」，所以主窗口、编辑器、图表上的菜单共用这一个小状态。
#[derive(Clone, Copy, Default)]
pub(crate) struct MenuState {
    pub(crate) open: Option<usize>,
}

/// 主窗口的菜单栏：顺序就是从左往右的顺序。条目动作现在只在状态栏回显。
pub(crate) const TOP_TABS: [MenuTab; 6] = [
    MenuTab {
        title: "文件",
        items: &["新建", "打开", "保存", "退出"],
    },
    MenuTab {
        title: "数据",
        items: &["抓取行情", "随机游走", "导入CSV", "数据源设置"],
    },
    MenuTab {
        title: "分析",
        items: &["指标设置", "画线工具", "回测"],
    },
    MenuTab {
        title: "布局",
        items: &["单区", "左右分", "上下分", "四宫格"],
    },
    MenuTab {
        title: "工具",
        items: &["设置", "日志"],
    },
    MenuTab {
        title: "帮助",
        items: &["关于"],
    },
];

/// 菜单项宽度 = 文字宽 + 左右各 1 格（9px）内边距，向上吸到整数格（最少 3 格）。
pub(crate) fn tab_width(app: &mut StockApp, title: &str) -> f32 {
    snap_cells_up(app.text_width(title) + 2.0 * TEXT_PAD_X).max(cells(3))
}

/// 菜单项矩形：从左边界起、左对齐排开。相邻两项共用中间那条网格线（步进 = 宽 - 1px）：
/// 各自尺寸照样是 cells(n)、完全相等，中间永远只有 1px。
pub(crate) fn tab_rects(app: &mut StockApp, bar: egui::Rect, tabs: &[MenuTab]) -> Vec<egui::Rect> {
    let mut rects = Vec::with_capacity(tabs.len());
    let mut x = bar.min.x;
    for tab in tabs {
        let w = tab_width(app, tab.title);
        rects.push(egui::Rect::from_min_size(
            egui::pos2(x, bar.min.y),
            egui::vec2(w, bar.height()),
        ));
        x += w - LINE_W;
    }
    rects
}

/// 一块下拉面板的几何：位置 + 行数。行高 / 行距只有这一份定义，所有下拉面板都用它。
#[derive(Clone, Copy)]
pub(crate) struct MenuPanel {
    pub(crate) rect: egui::Rect,
    pub(crate) rows: usize,
}

impl MenuPanel {
    /// 挂在某个控件下面：左边和它对齐，上沿压在它下沿那条网格线上（两段共用这 1px）。
    pub(crate) fn down(anchor: egui::Rect, w: f32, rows: usize) -> Self {
        Self {
            rect: egui::Rect::from_min_size(
                egui::pos2(anchor.min.x, anchor.max.y - LINE_W),
                egui::vec2(w, cells(MENU_ROW_ROWS * rows as i32)),
            ),
            rows,
        }
    }

    /// 挂在某个控件旁边、右边对齐：默认向下开（上沿压在控件下沿那条网格线上），
    /// 底下放不下就向上开（下沿压在控件上沿那条线上）- 边界同样只有 1px。
    pub(crate) fn right(anchor: egui::Rect, w: f32, rows: usize, window_bottom: f32) -> Self {
        let h = cells(MENU_ROW_ROWS * rows as i32);
        let x = anchor.max.x - w;
        let below = anchor.max.y - LINE_W;
        let y = if below + h > window_bottom {
            anchor.min.y + LINE_W - h
        } else {
            below
        };
        Self {
            rect: egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
            rows,
        }
    }

    /// 第 j 行：行高 2 格（相邻两行共用中间那条网格线，步进 = grid_line(2) = 18px）。
    pub(crate) fn row(&self, j: usize) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(self.rect.min.x, self.rect.min.y + grid_line(j as i32 * MENU_ROW_ROWS)),
            egui::vec2(self.rect.width(), MENU_ROW_H),
        )
    }

    /// 铺面板底色。
    pub(crate) fn paint_bg(&self, painter: &egui::Painter) {
        painter.rect_filled(self.rect, egui::CornerRadius::ZERO, theme().menu_bg);
    }

    /// 一条条目的 hover 高亮。
    pub(crate) fn paint_row_hover(&self, painter: &egui::Painter, row: egui::Rect) {
        painter.rect_filled(row, egui::CornerRadius::ZERO, theme().btn_hover);
    }

    /// 1px 硬边框：必须排在最后画（先画的话会被 hover 高亮和行底色啃掉一条像素）。
    pub(crate) fn paint_border(&self, painter: &egui::Painter) {
        border_hard(painter, self.rect, theme().ctrl_border);
    }

    /// 下拉面板的骨架：铺底 -> 逐行登记可点区域（悬停就高亮）-> 把这一行交给 draw_row 画内容 ->
    /// 1px 硬边压在最后。返回被点中的行下标（没点就是 None）。三处下拉面板（菜单栏、指标切换、
    /// 叠加）都走这一份 - 行高 / 高亮 / 边框 / 点击处理只有这一处实现。
    pub(crate) fn ui_rows(
        &self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        mut draw_row: impl FnMut(&mut StockApp, &egui::Painter, usize, egui::Rect),
    ) -> Option<usize> {
        self.paint_bg(painter);
        let mut clicked = None;
        for j in 0..self.rows {
            let row = self.row(j);
            let resp = ui.allocate_rect(row, egui::Sense::click());
            if resp.hovered() {
                self.paint_row_hover(painter, row);
            }
            draw_row(app, painter, j, row);
            if resp.clicked() {
                clicked = Some(j);
            }
        }
        self.paint_border(painter);
        clicked
    }
}

/// 菜单栏：交互（点开 / 再点收起 / hover 换项）+ 高亮 + 文字 + 下沿那条线。
/// 返回这一帧打开着的项下标和它的锚点矩形（没开就是 None），以及更新后的状态。
pub(crate) fn ui_menu_bar(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    bar: egui::Rect,
    tabs: &[MenuTab],
    mut state: MenuState,
) -> (Option<(usize, egui::Rect)>, MenuState) {
    let painter = ui.painter().clone();
    let rects = tab_rects(app, bar, tabs);
    let mut hovered = vec![false; tabs.len()];
    for (i, rect) in rects.iter().enumerate() {
        let resp = ui.allocate_rect(*rect, egui::Sense::click());
        hovered[i] = resp.hovered();
        if resp.clicked() {
            state.open = if state.open == Some(i) { None } else { Some(i) };
        } else if hovered[i] && state.open.is_some_and(|open| open != i) {
            state.open = Some(i);
        }
    }

    for (i, tab) in tabs.iter().enumerate() {
        if hovered[i] || state.open == Some(i) {
            painter.rect_filled(rects[i], egui::CornerRadius::ZERO, theme().btn_hover);
        }
        app.text(
            &painter,
            rects[i].center(),
            egui::Align2::CENTER_CENTER,
            tab.title,
            theme().text,
        );
    }
    // 下沿那条 1px 线：菜单栏和下一段共用的那条网格线，最后画。
    hline_hard(
        &painter,
        bar.min.x,
        bar.max.x,
        bar.max.y - LINE_W,
        theme().ctrl_border,
    );

    // 点菜单栏和下拉面板以外的地方 -> 收起面板。
    if let Some(i) = state.open {
        let panel = MenuPanel::down(rects[i], MENU_PANEL_W, tabs[i].items.len());
        close_on_outside_click(ui, &[bar, panel.rect], &mut state);
    }
    (state.open.map(|i| (i, rects[i])), state)
}

/// 下拉面板：铺底 -> 逐条画（hover 高亮 + 文字）-> 1px 硬边压在最后。
/// 返回这一帧被点中的条目下标（没点就是 None）；点中了顺手把菜单收起来。
pub(crate) fn ui_dropdown(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    panel: MenuPanel,
    items: &[&str],
    mut state: MenuState,
) -> (Option<usize>, MenuState) {
    let painter = ui.painter().clone();
    let clicked = panel.ui_rows(app, ui, &painter, |app, painter, j, row| {
        if let Some(&label) = items.get(j) {
            app.text(
                painter,
                egui::pos2(text_left_x(row.min.x), row.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                theme().text,
            );
        }
    });
    if clicked.is_some() {
        state.open = None;
    }
    (clicked, state)
}

/// 点「菜单栏和下拉面板以外」的地方就把菜单收起来。
pub(crate) fn close_on_outside_click(ui: &egui::Ui, keep: &[egui::Rect], state: &mut MenuState) {
    if state.open.is_none() {
        return;
    }
    let outside = ui.ctx().input(|i| {
        i.pointer.primary_clicked()
            && i.pointer
                .interact_pos()
                .is_some_and(|p| !keep.iter().any(|r| r.contains(p)))
    });
    if outside {
        state.open = None;
    }
}