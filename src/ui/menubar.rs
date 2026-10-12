// ===========================================================================
//  模块 ui::menubar：菜单栏与下拉菜单
// ===========================================================================
//  六个顶级菜单 + 下拉面板。菜单项宽度 = 文字宽度 + 左右各 9px 内边距再吸到整数格。
//  本文件 = 拆分前那份单文件里的「八、菜单栏与下拉菜单」。
use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use eframe::egui;

// ===========================================================================
//  八、菜单栏与下拉菜单
// ===========================================================================
//  布局规则：菜单项宽度 = 文字宽度 + 左右各 9px 内边距，再向上吸到整数格
//  （两个汉字 12px 字 = 24px 宽 → cells(5) = 46px）；高度就是菜单栏的 cells(3) = 28px。
//  下拉面板宽 cells(16)、每行 cells(3)，面板上边和菜单栏共用一条网格线，左边和对应菜单项对齐。

/// 菜单栏上的六个顶级菜单。显示顺序看 TopMenu::ALL —— 那儿排的顺序就是标题栏上从左往右的顺序。
/// 每个菜单自己带一串下拉条目（见 items），条目的动作以后接功能，现在只在状态栏回显。
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum TopMenu {
    File,
    Data,
    Analyze,
    Layout,
    Tools,
    Help,
}

impl TopMenu {
    /// 显示顺序。
    pub(crate) const ALL: [TopMenu; 6] = [
        TopMenu::File,
        TopMenu::Data,
        TopMenu::Analyze,
        TopMenu::Layout,
        TopMenu::Tools,
        TopMenu::Help,
    ];

    /// 菜单个数（数组长度和地方都取它，省得写死两个地方）。
    pub(crate) const COUNT: usize = 6;

    /// 菜单栏上显示的文字。
    pub(crate) fn title(self) -> &'static str {
        match self {
            TopMenu::File => "文件",
            TopMenu::Data => "数据",
            TopMenu::Analyze => "分析",
            TopMenu::Layout => "布局",
            TopMenu::Tools => "工具",
            TopMenu::Help => "帮助",
        }
    }

    /// 下拉面板里的条目。本步只做展示：点了只在状态栏回显文字。
    pub(crate) fn items(self) -> &'static [&'static str] {
        match self {
            TopMenu::File => &["新建", "打开", "保存", "退出"],
            TopMenu::Data => &["抓取行情", "随机游走", "导入CSV", "数据源设置"],
            TopMenu::Analyze => &["指标设置", "画线工具", "回测"],
            TopMenu::Layout => &["单区", "左右分", "上下分", "四宫格"],
            TopMenu::Tools => &["设置", "日志"],
            TopMenu::Help => &["关于"],
        }
    }

    /// 在 ALL 里的下标：下拉面板要拿它对齐到对应菜单项的左边。
    pub(crate) fn index(self) -> usize {
        TopMenu::ALL.iter().position(|m| *m == self).unwrap_or(0)
    }
}

impl StockApp {
    /// 菜单项宽度 = 文字宽度 + 左右各 9px 内边距，向上吸到整数格（最小 3 格）。
    /// 字宽是硬边字形的整数步进之和（两个汉字 12px 字是 24），吸附后一定落在网格上。
    pub(crate) fn menu_item_width(&mut self, title: &str) -> f32 {
        snap_cells_up(self.text_width(title) + 2.0 * TEXT_PAD_X).max(cells(3))
    }

    /// 从左往右排开所有菜单项。相邻两项共用中间那条网格线（步进 = 宽 - 1px）：
    /// 各自尺寸照样是 cells(n)、完全相等，高亮块碰不到邻居的边框，中间也永远只有 1px。
    pub(crate) fn menu_item_rects(&mut self, bar: egui::Rect) -> [egui::Rect; TopMenu::COUNT] {
        let mut rects = [egui::Rect::NOTHING; TopMenu::COUNT];
        let mut x = bar.min.x;
        for (i, menu) in TopMenu::ALL.iter().enumerate() {
            let w = self.menu_item_width(menu.title());
            rects[i] =
                egui::Rect::from_min_size(egui::pos2(x, bar.min.y), egui::vec2(w, bar.height()));
            x += w - LINE_W;
        }
        rects
    }

    /// 下拉面板的矩形：上边和菜单栏共用那条网格线（anchor 的最下一行），
    /// 左边和菜单项对齐，高度 = 行数 x 3 格（相邻两行同样共用网格线）。
    pub(crate) fn menu_panel_rect(anchor: egui::Rect, open: TopMenu) -> egui::Rect {
        let rows = MENU_ROW_ROWS * open.items().len() as i32;
        egui::Rect::from_min_size(
            egui::pos2(anchor.min.x, anchor.max.y - LINE_W),
            egui::vec2(MENU_PANEL_W, cells(rows)),
        )
    }
}

impl StockApp {
    /// 菜单栏：点击展开/收起、hover 自动切换，再画高亮和文字，最后处理「点别处收起」。
    /// 返回值是当前展开菜单的锚点矩形和菜单本身（没展开就是 None）：下拉面板交给
    /// 调用方在区域分隔线之后画，那样面板以外的任何东西都盖不住那几条 1px 边框。
    pub(crate) fn ui_menubar(&mut self, ui: &mut egui::Ui, bar: egui::Rect) -> Option<(egui::Rect, TopMenu)> {
        let painter = ui.painter().clone();
        let rects = self.menu_item_rects(bar);

        // 1) 交互：点一下展开、再点一下收起；已经展开时 hover 到别的菜单就切过去。
        let mut hovered = [false; TopMenu::COUNT];
        for (i, menu) in TopMenu::ALL.iter().enumerate() {
            let resp = ui.allocate_rect(rects[i], egui::Sense::click());
            hovered[i] = resp.hovered();
            if resp.clicked() {
                self.open_menu = if self.open_menu == Some(*menu) {
                    None
                } else {
                    Some(*menu)
                };
            } else if hovered[i] && self.open_menu.is_some_and(|open| open != *menu) {
                self.open_menu = Some(*menu);
            }
        }

        // 2) 画菜单栏：底色由骨架画，这里只补 hover / 展开态的高亮和文字。
        for (i, menu) in TopMenu::ALL.iter().enumerate() {
            if hovered[i] || self.open_menu == Some(*menu) {
                painter.rect_filled(rects[i], egui::CornerRadius::ZERO, C_BTN_HOVER);
            }
            self.text(
                &painter,
                rects[i].center(),
                egui::Align2::CENTER_CENTER,
                menu.title(),
                C_TEXT,
            );
        }

        // 3) 点在菜单栏和下拉面板以外的地方 → 收起面板。
        if self.open_menu.is_some() {
            let panel = self
                .open_menu
                .map(|open| Self::menu_panel_rect(rects[open.index()], open));
            let outside = ui.ctx().input(|i| {
                i.pointer.primary_clicked()
                    && i.pointer
                        .interact_pos()
                        .is_some_and(|p| !bar.contains(p) && !panel.is_some_and(|r| r.contains(p)))
            });
            if outside {
                self.open_menu = None;
            }
        }

        self.open_menu.map(|open| (rects[open.index()], open))
    }
}

impl StockApp {
    /// 下拉面板：先铺底，再画每一条（hover 高亮 + 文字 + 点击回显），
    /// 最后把 1px 硬边压在最上面 —— 先画边的话会被 hover 高亮啃掉一条像素。
    pub(crate) fn ui_menu_panel(&mut self, ui: &mut egui::Ui, anchor: egui::Rect, open: TopMenu) {
        let painter = ui.painter().clone();
        let panel = Self::menu_panel_rect(anchor, open);
        painter.rect_filled(panel, egui::CornerRadius::ZERO, C_MENU_BG);

        let mut clicked = false;
        for (j, label) in open.items().iter().enumerate() {
            // 相邻两行共用一条网格线：步进 grid_line(3) = 27px，行高 cells(3) = 28px。
            let row = egui::Rect::from_min_size(
                egui::pos2(panel.min.x, panel.min.y + grid_line(j as i32 * MENU_ROW_ROWS)),
                egui::vec2(MENU_PANEL_W, MENU_ROW_H),
            );
            let resp = ui.allocate_rect(row, egui::Sense::click());
            if resp.hovered() {
                painter.rect_filled(row, egui::CornerRadius::ZERO, C_BTN_HOVER);
            }
            self.text(
                &painter,
                egui::pos2(row.min.x + TEXT_PAD_X, row.center().y),
                egui::Align2::LEFT_CENTER,
                *label,
                C_TEXT,
            );
            if resp.clicked() {
                self.status_msg = format!("你点击了：{label}");
                clicked = true;
            }
        }

        // 1px 硬边最后画：先画边的话会被 hover 高亮和行底色啃掉一条像素。
        border_hard(&painter, panel, C_BORDER);

        // 点了条目就收起面板：这一步排在「点外面」判断之前，此刻鼠标还在面板里。
        if clicked {
            self.open_menu = None;
        }
    }
}

