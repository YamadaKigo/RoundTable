use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::icon::CursorKind;
use eframe::egui;

pub(crate) struct MenuNode {
    pub(crate) label: &'static str,
    pub(crate) children: &'static [MenuNode],
}

impl MenuNode {
    pub(crate) const fn act(label: &'static str) -> Self {
        Self {
            label,
            children: &[],
        }
    }

    pub(crate) const fn sub(label: &'static str, children: &'static [MenuNode]) -> Self {
        Self { label, children }
    }

    pub(crate) fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

pub(crate) struct MenuTab {
    pub(crate) title: &'static str,
    pub(crate) items: &'static [MenuNode],
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MenuState {
    pub(crate) open: Option<usize>,
    pub(crate) path: Vec<usize>,
}

pub(crate) const MENU_DEPTH_MAX: usize = 4;

const FILE_RECENT_MORE: &[MenuNode] = &[
    MenuNode::act("三级示例 上证50"),
    MenuNode::act("三级示例 沪深300"),
];
const FILE_RECENT: &[MenuNode] = &[
    MenuNode::act("上证50.csv"),
    MenuNode::act("沪深300.csv"),
    MenuNode::sub("更多", FILE_RECENT_MORE),
];
const FILE_ITEMS: &[MenuNode] = &[
    MenuNode::act("新建"),
    MenuNode::act("打开"),
    MenuNode::sub("最近打开", FILE_RECENT),
    MenuNode::act("保存"),
    MenuNode::act("退出"),
];
const DATA_ITEMS: &[MenuNode] = &[
    MenuNode::act("抓取行情"),
    MenuNode::sub(
        "随机游走",
        &[
            MenuNode::act("按默认参数"),
            MenuNode::sub(
                "按参数",
                &[MenuNode::act("长度 512 根"), MenuNode::act("长度 1024 根")],
            ),
        ],
    ),
    MenuNode::act("导入CSV"),
    MenuNode::sub(
        "数据源设置",
        &[MenuNode::act("本地文件"), MenuNode::act("远程接口")],
    ),
];
const ANALYZE_ITEMS: &[MenuNode] = &[
    MenuNode::act("指标设置"),
    MenuNode::sub(
        "画线工具",
        &[
            MenuNode::act("趋势线"),
            MenuNode::act("水平线"),
            MenuNode::act("矩形"),
        ],
    ),
    MenuNode::act("回测"),
];

const LAYOUT_ITEMS: &[MenuNode] = &[MenuNode::act("控制面板"), MenuNode::act("新增图表")];
const TOOL_ITEMS: &[MenuNode] = &[
    MenuNode::sub(
        "设置",
        &[
            MenuNode::act("界面"),
            MenuNode::sub("配色", &[MenuNode::act("浅色"), MenuNode::act("深色")]),
        ],
    ),
    MenuNode::act("日志"),
];
const HELP_ITEMS: &[MenuNode] = &[
    MenuNode::act("关于"),
    MenuNode::sub(
        "帮助主题",
        &[MenuNode::act("快捷键"), MenuNode::act("常见问题")],
    ),
];

pub(crate) const TOP_TABS: [MenuTab; 6] = [
    MenuTab {
        title: "文件",
        items: FILE_ITEMS,
    },
    MenuTab {
        title: "数据",
        items: DATA_ITEMS,
    },
    MenuTab {
        title: "分析",
        items: ANALYZE_ITEMS,
    },
    MenuTab {
        title: "布局",
        items: LAYOUT_ITEMS,
    },
    MenuTab {
        title: "工具",
        items: TOOL_ITEMS,
    },
    MenuTab {
        title: "帮助",
        items: HELP_ITEMS,
    },
];

pub(crate) fn tab_width(app: &mut StockApp, title: &str) -> f32 {
    snap_cells_up(app.text_width(title) + 2.0 * TEXT_PAD_X).max(cells(3))
}

pub(crate) fn menu_panel_width(app: &mut StockApp, level: &[MenuNode]) -> f32 {
    let mut w = MENU_PANEL_W;
    for node in level {
        let arrow = if node.has_children() { PITCH } else { 0.0 };
        w = w.max(snap_cells_up(
            app.text_width(node.label) + 2.0 * TEXT_PAD_X + arrow,
        ));
    }
    w
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum MenuAlign {
    Left,
    Right,
}

pub(crate) fn menu_item_rects(
    bar: egui::Rect,
    widths: &[f32],
    align: MenuAlign,
) -> Vec<egui::Rect> {
    let item = |x: f32, w: f32| {
        egui::Rect::from_min_size(egui::pos2(x, bar.min.y), egui::vec2(w, bar.height()))
    };
    let mut rects = Vec::with_capacity(widths.len());
    match align {
        MenuAlign::Left => {
            let mut x = bar.min.x;
            for &w in widths {
                rects.push(item(x, w));
                x += w - LINE_W;
            }
        }
        MenuAlign::Right => {
            let mut x = bar.max.x;
            for &w in widths.iter().rev() {
                rects.push(item(x - w, w));
                x -= w - LINE_W;
            }
            rects.reverse();
        }
    }
    rects
}

fn snap_into(v: f32, lo: f32, hi: f32) -> f32 {
    let hi = hi.max(lo);
    let steps = ((hi - lo) / PITCH).floor();
    let k = ((v - lo) / PITCH).round().clamp(0.0, steps);
    lo + k * PITCH
}

#[derive(Clone, Copy)]
pub(crate) struct MenuPanel {
    pub(crate) rect: egui::Rect,
    pub(crate) rows: usize,
}

fn menu_hit_rect(rect: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        rect.min + egui::vec2(LINE_W, LINE_W),
        rect.max - egui::vec2(LINE_W, LINE_W),
    )
}

impl MenuPanel {
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

    pub(crate) fn bounded(anchor: egui::Rect, w: f32, rows: usize, bounds: egui::Rect) -> Self {
        let h = cells(MENU_ROW_ROWS * rows as i32);
        let x = snap_into(anchor.min.x, bounds.min.x, bounds.max.x - LINE_W - w);
        let below = anchor.max.y - LINE_W;
        let y_want = if below + h <= bounds.max.y - LINE_W {
            below
        } else {
            anchor.min.y + LINE_W - h
        };
        let y = snap_into(y_want, bounds.min.y, bounds.max.y - LINE_W - h);
        Self {
            rect: egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
            rows,
        }
    }

    pub(crate) fn row(&self, j: usize) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(
                self.rect.min.x,
                self.rect.min.y + grid_line(j as i32 * MENU_ROW_ROWS),
            ),
            egui::vec2(self.rect.width(), MENU_ROW_H),
        )
    }

    pub(crate) fn flyout(row: egui::Rect, w: f32, rows: usize, bounds: egui::Rect) -> Self {
        let h = cells(MENU_ROW_ROWS * rows as i32);

        let right_open = row.max.x - LINE_W;
        let x_want = if right_open + w <= bounds.max.x - LINE_W {
            right_open
        } else {
            row.min.x - w + LINE_W
        };

        let y_want = row.min.y - LINE_W;
        Self {
            rect: egui::Rect::from_min_size(
                egui::pos2(
                    snap_into(
                        x_want,
                        bounds.min.x,
                        (bounds.max.x - LINE_W - w).max(bounds.min.x),
                    ),
                    snap_into(
                        y_want,
                        bounds.min.y,
                        (bounds.max.y - LINE_W - h).max(bounds.min.y),
                    ),
                ),
                egui::vec2(w, h),
            ),
            rows,
        }
    }

    pub(crate) fn paint_bg(&self, painter: &egui::Painter) {
        painter.rect_filled(self.rect, egui::CornerRadius::ZERO, theme().menu_bg);
    }

    pub(crate) fn paint_row_hover(&self, painter: &egui::Painter, row: egui::Rect) {
        painter.rect_filled(row, egui::CornerRadius::ZERO, theme().btn_press);
        border_hard(painter, row, theme().ctrl_border);
    }

    pub(crate) fn paint_border(&self, painter: &egui::Painter) {
        border_hard(painter, self.rect, theme().ctrl_border);
    }

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
            let resp = interact_rect(ui, menu_hit_rect(row), egui::Sense::click());
            if resp.hovered() {
                app.pointer.force_hint(CursorKind::Normal, ui);
                self.paint_row_hover(painter, row);
            }
            draw_row(app, painter, j, row);
            if j + 1 < self.rows {
                hline_hard(
                    painter,
                    row.min.x + LINE_W,
                    row.max.x - LINE_W,
                    row.max.y - LINE_W,
                    theme().line,
                );
            }
            if resp.clicked() {
                clicked = Some(j);
            }
        }
        self.paint_border(painter);
        clicked
    }
}

#[derive(Clone, Copy)]
pub(crate) struct MenuBarStyle {
    pub(crate) hover_switch: bool,

    pub(crate) lead_divider: bool,
}

impl MenuBarStyle {
    pub(crate) const CLASSIC: Self = Self {
        hover_switch: true,
        lead_divider: false,
    };

    pub(crate) const PANE_BAR: Self = Self {
        hover_switch: false,
        lead_divider: true,
    };
}

pub(crate) struct MenuBarOut {
    pub(crate) rects: Vec<egui::Rect>,

    pub(crate) clicked: Option<usize>,

    pub(crate) open: Option<usize>,
}

pub(crate) struct MenuBar<'a, 'b> {
    bar: egui::Rect,
    lead: Option<String>,
    titles: &'a [&'static str],
    align: MenuAlign,
    label: Option<&'b str>,
    style: MenuBarStyle,
}

impl<'a, 'b> MenuBar<'a, 'b> {
    pub(crate) fn new(
        bar: egui::Rect,
        titles: &'a [&'static str],
        align: MenuAlign,
        label: Option<&'b str>,
        style: MenuBarStyle,
    ) -> Self {
        Self {
            bar,
            lead: None,
            titles,
            align,
            label,
            style,
        }
    }

    pub(crate) fn with_lead(mut self, lead: &str) -> Self {
        self.lead = Some(lead.to_owned());
        self
    }

    pub(crate) fn ui(
        &self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        mut open: Option<usize>,
    ) -> MenuBarOut {
        let painter = ui.painter().clone();
        let offset = usize::from(self.lead.is_some());
        let mut display_titles: Vec<&str> = Vec::with_capacity(self.titles.len() + offset);
        if let Some(lead) = &self.lead {
            display_titles.push(lead.as_str());
        }
        display_titles.extend(self.titles.iter().copied());
        let widths: Vec<f32> = display_titles
            .iter()
            .map(|title| tab_width(app, title))
            .collect();
        let rects = menu_item_rects(self.bar, &widths, self.align);

        if ui
            .input(|i| i.pointer.hover_pos())
            .is_some_and(|p| rects.iter().any(|rect| menu_hit_rect(*rect).contains(p)))
        {
            app.pointer.force_hint(CursorKind::Normal, ui);
        }

        painter.rect_filled(self.bar, egui::CornerRadius::ZERO, theme().menu_bg);

        if let Some(text) = self.label {
            app.text(
                &painter,
                egui::pos2(
                    text_left_x(self.bar.min.x, self.bar.min.x),
                    self.bar.center().y,
                ),
                egui::Align2::LEFT_CENTER,
                text,
                theme().text,
            );
        }
        let mut hovered = vec![false; display_titles.len()];
        let mut clicked = None;
        for (i, rect) in rects.iter().enumerate() {
            let resp = interact_rect(ui, menu_hit_rect(*rect), egui::Sense::click());
            hovered[i] = resp.hovered();
            if offset == 1 && i == 0 {
                continue;
            }
            let item = i - offset;
            if resp.clicked() {
                clicked = Some(item);
                open = if open == Some(item) { None } else { Some(item) };
            } else if self.style.hover_switch && hovered[i] && open.is_some_and(|cur| cur != item) {
                open = Some(item);
            }
        }

        for (i, title) in display_titles.iter().enumerate() {
            let item_open = !(offset == 1 && i == 0) && open == Some(i.saturating_sub(offset));
            if hovered[i] || item_open {
                Self::paint_hover(&painter, rects[i]);
            }
            app.text(
                &painter,
                rects[i].center(),
                egui::Align2::CENTER_CENTER,
                title,
                theme().text,
            );
        }

        if self.style.lead_divider {
            vline_hard(
                &painter,
                rects[0].min.x,
                self.bar.min.y + LINE_W,
                self.bar.max.y - LINE_W,
                theme().line,
            );
        }
        for rect in &rects {
            vline_hard(
                &painter,
                rect.max.x - LINE_W,
                self.bar.min.y + LINE_W,
                self.bar.max.y - LINE_W,
                theme().line,
            );
        }

        hline_hard(
            &painter,
            self.bar.min.x,
            self.bar.max.x,
            self.bar.max.y - LINE_W,
            theme().ctrl_border,
        );

        MenuBarOut {
            rects: rects[offset..].to_vec(),
            clicked,
            open,
        }
    }

    fn paint_hover(painter: &egui::Painter, rect: egui::Rect) {
        painter.rect_filled(rect, egui::CornerRadius::ZERO, theme().btn_press);
        border_hard(painter, rect, theme().ctrl_border);
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn menu_bar_ui(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    bar: egui::Rect,
    titles: &[&'static str],
    align: MenuAlign,
    open: Option<usize>,
    label: Option<&str>,
    style: MenuBarStyle,
) -> MenuBarOut {
    MenuBar::new(bar, titles, align, label, style).ui(app, ui, open)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn menu_bar_ui_with_lead(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    bar: egui::Rect,
    lead: &str,
    titles: &[&'static str],
    align: MenuAlign,
    open: Option<usize>,
    label: Option<&str>,
    style: MenuBarStyle,
) -> MenuBarOut {
    MenuBar::new(bar, titles, align, label, style)
        .with_lead(lead)
        .ui(app, ui, open)
}

pub(crate) fn ui_menu_bar(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    bar: egui::Rect,
    tabs: &[MenuTab],
    state: MenuState,
) -> (Option<(usize, egui::Rect)>, MenuState) {
    let titles: Vec<&str> = tabs.iter().map(|t| t.title).collect();
    let out = menu_bar_ui(
        app,
        ui,
        bar,
        &titles,
        MenuAlign::Left,
        state.open,
        None,
        MenuBarStyle::CLASSIC,
    );
    let switched = out.open != state.open;
    (
        out.open.map(|i| (i, out.rects[i])),
        MenuState {
            open: out.open,

            path: if switched { Vec::new() } else { state.path },
        },
    )
}

pub(crate) fn ui_menu_popup(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    anchor: egui::Rect,
    items: &'static [MenuNode],
    state: &mut MenuState,
    bounds: egui::Rect,
    keep: &[egui::Rect],
) -> Option<&'static str> {
    ui_menu_popup_inner(app, ui, anchor, items, state, bounds, keep)
}

pub(crate) fn ui_menu_popup_persistent(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    anchor: egui::Rect,
    items: &'static [MenuNode],
    state: &mut MenuState,
    bounds: egui::Rect,
    keep: &[egui::Rect],
) -> Option<&'static str> {
    ui_menu_popup_inner(app, ui, anchor, items, state, bounds, keep)
}

#[allow(clippy::too_many_arguments)]
fn ui_menu_popup_inner(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    anchor: egui::Rect,
    items: &'static [MenuNode],
    state: &mut MenuState,
    bounds: egui::Rect,
    keep: &[egui::Rect],
) -> Option<&'static str> {
    if state.open.is_none() {
        state.path.clear();
        return None;
    }

    let pointer = ui.ctx().input(|i| i.pointer.hover_pos());
    let mut chain: Vec<(MenuPanel, &'static [MenuNode])> = Vec::new();
    let mut path: Vec<usize> = Vec::new();
    let mut row_anchor = anchor;
    let mut level = items;
    while !level.is_empty() && chain.len() < MENU_DEPTH_MAX {
        let w = menu_panel_width(app, level);
        let panel = if chain.is_empty() {
            MenuPanel::bounded(anchor, w, level.len(), bounds)
        } else {
            MenuPanel::flyout(row_anchor, w, level.len(), bounds)
        };
        let hovered = pointer
            .and_then(|p| (0..level.len()).find(|&j| menu_hit_rect(panel.row(j)).contains(p)));
        let next = match hovered {
            Some(j) if level[j].has_children() => Some(j),

            Some(_) => None,

            None => state
                .path
                .get(chain.len())
                .copied()
                .filter(|&j| j < level.len() && level[j].has_children()),
        };
        chain.push((panel, level));
        match next {
            Some(j) => {
                path.push(j);
                row_anchor = panel.row(j);
                level = level[j].children;
            }
            None => break,
        }
    }
    let mut keep_rects = keep.to_vec();
    keep_rects.extend(chain.iter().map(|(p, _)| p.rect));
    if pointer.is_some_and(|p| chain.iter().any(|(panel, _)| panel.rect.contains(p))) {
        app.pointer.force_hint(CursorKind::Normal, ui);
    }
    close_on_outside_click(ui, &keep_rects, state);

    let mut clicked: Option<(usize, usize)> = None;
    for (k, (panel, level)) in chain.iter().enumerate() {
        let painter = ui.painter().clone();
        let hit = panel.ui_rows(app, ui, &painter, |app, painter, j, row| {
            let node = &level[j];

            app.text(
                painter,
                egui::pos2(text_left_x(row.min.x, panel.rect.min.x), row.center().y),
                egui::Align2::LEFT_CENTER,
                node.label,
                theme().text,
            );
            if node.has_children() {
                paint_submenu_arrow(painter, row);
            }
        });
        if let Some(j) = hit {
            clicked = Some((k, j));
        }
    }
    let Some((k, j)) = clicked else {
        state.path = path;
        return None;
    };
    let node = &chain[k].1[j];
    if node.has_children() {
        path.truncate(k + 1);
        state.path = path;
        return None;
    }

    state.open = None;
    state.path.clear();
    Some(node.label)
}

fn paint_submenu_arrow(painter: &egui::Painter, row: egui::Rect) {
    let x = (row.max.x - PITCH + LINE_W).floor();
    let y = (row.center().y - 2.5).floor();
    for (i, w) in [1.0f32, 2.0, 3.0, 2.0, 1.0].into_iter().enumerate() {
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(x, y + i as f32), egui::vec2(w, LINE_W)),
            egui::CornerRadius::ZERO,
            theme().text,
        );
    }
}

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
