use crate::application::app::*;
use crate::domain::layout::*;
use crate::engine::kit;
use crate::engine::theme::*;
use crate::ui::icon::CursorKind;

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
    MenuNode::act("Example SSE 50"),
    MenuNode::act("Example CSI 300"),
];
const FILE_RECENT: &[MenuNode] = &[
    MenuNode::act("SSE50.csv"),
    MenuNode::act("CSI300.csv"),
    MenuNode::sub("More", FILE_RECENT_MORE),
];
const FILE_ITEMS: &[MenuNode] = &[
    MenuNode::act("New"),
    MenuNode::act("Open"),
    MenuNode::sub("Recent Files", FILE_RECENT),
    MenuNode::act("Save"),
    MenuNode::act("Exit"),
];
const DATA_ITEMS: &[MenuNode] = &[
    MenuNode::act("Fetch Market Data"),
    MenuNode::sub(
        "Random Walk",
        &[
            MenuNode::act("Use Defaults"),
            MenuNode::sub(
                "Use Parameters",
                &[MenuNode::act("Length 512"), MenuNode::act("Length 1024")],
            ),
        ],
    ),
    MenuNode::act("Import CSV"),
    MenuNode::sub(
        "Data Source",
        &[MenuNode::act("Local File"), MenuNode::act("Remote API")],
    ),
];
const ANALYZE_ITEMS: &[MenuNode] = &[
    MenuNode::act("Indicator Settings"),
    MenuNode::sub(
        "Drawing Tools",
        &[
            MenuNode::act("Trend Line"),
            MenuNode::act("Horizontal Line"),
            MenuNode::act("Rectangle"),
        ],
    ),
    MenuNode::act("Backtest"),
];

const LAYOUT_ITEMS: &[MenuNode] = &[MenuNode::act("Control Panel"), MenuNode::act("New Chart")];
const TOOL_ITEMS: &[MenuNode] = &[
    MenuNode::sub("Settings", &[MenuNode::act("Interface")]),
    MenuNode::act("Logs"),
];
const HELP_ITEMS: &[MenuNode] = &[
    MenuNode::act("About"),
    MenuNode::sub(
        "Help Topics",
        &[MenuNode::act("Shortcuts"), MenuNode::act("FAQ")],
    ),
];

pub(crate) const TOP_TABS: [MenuTab; 6] = [
    MenuTab {
        title: "File",
        items: FILE_ITEMS,
    },
    MenuTab {
        title: "Data",
        items: DATA_ITEMS,
    },
    MenuTab {
        title: "Analysis",
        items: ANALYZE_ITEMS,
    },
    MenuTab {
        title: "Layout",
        items: LAYOUT_ITEMS,
    },
    MenuTab {
        title: "Tools",
        items: TOOL_ITEMS,
    },
    MenuTab {
        title: "Help",
        items: HELP_ITEMS,
    },
];

pub(crate) fn tab_width(app: &mut AppState, title: &str) -> f32 {
    snap_cells_up(app.text_width(title) + 2.0 * TEXT_PAD_X).max(cells(3))
}

pub(crate) fn menu_panel_width(app: &mut AppState, level: &[MenuNode]) -> f32 {
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
}

pub(crate) fn menu_item_rects(bar: kit::Rect, widths: &[f32], align: MenuAlign) -> Vec<kit::Rect> {
    let item = |x: f32, w: f32| {
        kit::Rect::from_min_size(kit::pos2(x, bar.min.y), kit::vec2(w, bar.height()))
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
    pub(crate) rect: kit::Rect,
    pub(crate) rows: usize,
}

fn menu_hit_rect(rect: kit::Rect) -> kit::Rect {
    kit::Rect::from_min_max(
        rect.min + kit::vec2(LINE_W, LINE_W),
        rect.max - kit::vec2(LINE_W, LINE_W),
    )
}

impl MenuPanel {
    pub(crate) fn left(anchor: kit::Rect, w: f32, rows: usize, window_bottom: f32) -> Self {
        let h = cells(MENU_ROW_ROWS * rows as i32);
        let frame = grid_line(1);
        let below = anchor.max.y - LINE_W;
        let outer_h = h + 2.0 * frame;
        let y = if below + outer_h > window_bottom {
            anchor.min.y + LINE_W - outer_h
        } else {
            below
        };
        Self {
            rect: kit::Rect::from_min_size(
                kit::pos2(anchor.min.x + frame, y + frame),
                kit::vec2(w, h),
            ),
            rows,
        }
    }

    pub(crate) fn bounded(anchor: kit::Rect, w: f32, rows: usize, bounds: kit::Rect) -> Self {
        let h = cells(MENU_ROW_ROWS * rows as i32);
        let frame = grid_line(1);
        let outer_w = w + 2.0 * frame;
        let outer_h = h + 2.0 * frame;
        let x = snap_into(
            anchor.min.x,
            bounds.min.x,
            (bounds.max.x - LINE_W - outer_w).max(bounds.min.x),
        );
        let below = anchor.max.y - LINE_W;
        let y_want = if below + outer_h <= bounds.max.y - LINE_W {
            below
        } else {
            anchor.min.y + LINE_W - outer_h
        };
        let y = snap_into(
            y_want,
            bounds.min.y,
            (bounds.max.y - LINE_W - outer_h).max(bounds.min.y),
        );
        Self {
            rect: kit::Rect::from_min_size(kit::pos2(x + frame, y + frame), kit::vec2(w, h)),
            rows,
        }
    }

    pub(crate) fn row(&self, j: usize) -> kit::Rect {
        kit::Rect::from_min_size(
            kit::pos2(
                self.rect.min.x,
                self.rect.min.y + grid_line(j as i32 * MENU_ROW_ROWS),
            ),
            kit::vec2(self.rect.width(), MENU_ROW_H),
        )
    }

    pub(crate) fn flyout(row: kit::Rect, w: f32, rows: usize, bounds: kit::Rect) -> Self {
        let h = cells(MENU_ROW_ROWS * rows as i32);
        let frame = grid_line(1);
        let outer_w = w + 2.0 * frame;
        let outer_h = h + 2.0 * frame;

        let right_open = row.max.x - LINE_W;
        let x_want = if right_open + outer_w <= bounds.max.x - LINE_W {
            right_open
        } else {
            row.min.x - outer_w + LINE_W
        };

        let y_want = row.min.y - LINE_W;
        Self {
            rect: kit::Rect::from_min_size(
                kit::pos2(
                    snap_into(
                        x_want,
                        bounds.min.x,
                        (bounds.max.x - LINE_W - outer_w).max(bounds.min.x),
                    ) + frame,
                    snap_into(
                        y_want,
                        bounds.min.y,
                        (bounds.max.y - LINE_W - outer_h).max(bounds.min.y),
                    ) + frame,
                ),
                kit::vec2(w, h),
            ),
            rows,
        }
    }

    pub(crate) fn paint_bg(&self, painter: &kit::Painter) {
        painter.rect_filled(
            self.frame_rect(),
            kit::CornerRadius::ZERO,
            theme().menu_popup_frame_bg,
        );
        painter.rect_filled(self.rect, kit::CornerRadius::ZERO, theme().menu_popup_bg);
    }

    pub(crate) fn paint_row_hover(&self, painter: &kit::Painter, row: kit::Rect) {
        painter.rect_filled(row, kit::CornerRadius::ZERO, theme().menu_popup_hover);
        border_hard(painter, row, theme().menu_popup_border);
    }

    pub(crate) fn paint_border(&self, painter: &kit::Painter) {
        border_hard(painter, self.frame_rect(), theme().menu_popup_frame_border);
        border_hard(painter, self.rect, theme().menu_popup_border);
    }

    pub(crate) fn frame_rect(&self) -> kit::Rect {
        self.rect.expand(grid_line(1))
    }

    pub(crate) fn ui_rows(
        &self,
        app: &mut AppState,
        ui: &mut kit::Ui,
        painter: &kit::Painter,
        mut draw_row: impl FnMut(&mut AppState, &kit::Painter, usize, kit::Rect),
    ) -> Option<usize> {
        self.paint_bg(painter);
        let mut clicked = None;
        for j in 0..self.rows {
            let row = self.row(j);
            let resp = interact_rect(
                ui,
                menu_hit_rect(row),
                kit::Id::new(("menu_popup_row", self.rect.width().to_bits(), j)),
                kit::Sense::click(),
            );
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
                    theme().menu_popup_separator,
                );
            }
            if resp.pressed() {
                clicked = Some(j);
            }
        }
        self.paint_border(painter);
        clicked
    }
}

#[derive(Clone, Copy)]
pub(crate) struct MenuBarStyle {
    pane: bool,
}

impl MenuBarStyle {
    pub(crate) const CLASSIC: Self = Self { pane: false };

    pub(crate) const PANE_BAR: Self = Self { pane: true };

    fn colors(
        self,
    ) -> (
        kit::Color32,
        kit::Color32,
        kit::Color32,
        kit::Color32,
        kit::Color32,
    ) {
        if self.pane {
            (
                theme().chart_pane_bg,
                theme().chart_pane_text,
                theme().chart_pane_hover,
                theme().chart_pane_border,
                theme().chart_pane_separator,
            )
        } else {
            (
                theme().menu_bg,
                theme().menu_text,
                theme().menu_hover,
                theme().menu_border,
                theme().menu_separator,
            )
        }
    }
}

pub(crate) struct MenuBarOut {
    pub(crate) rects: Vec<kit::Rect>,

    pub(crate) open: Option<usize>,
}

pub(crate) struct MenuBar<'a> {
    bar: kit::Rect,
    titles: &'a [&'static str],
    align: MenuAlign,
    style: MenuBarStyle,
}

impl<'a> MenuBar<'a> {
    pub(crate) fn new(
        bar: kit::Rect,
        titles: &'a [&'static str],
        align: MenuAlign,
        style: MenuBarStyle,
    ) -> Self {
        Self {
            bar,
            titles,
            align,
            style,
        }
    }

    pub(crate) fn ui(
        &self,
        app: &mut AppState,
        ui: &mut kit::Ui,
        mut open: Option<usize>,
    ) -> MenuBarOut {
        let painter = ui.painter().clone();
        let (bg, text, hover, border, separator) = self.style.colors();
        let widths: Vec<f32> = self
            .titles
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

        painter.rect_filled(self.bar, kit::CornerRadius::ZERO, bg);

        let mut hovered = vec![false; self.titles.len()];
        for (i, rect) in rects.iter().enumerate() {
            let resp = interact_rect(
                ui,
                menu_hit_rect(*rect),
                kit::Id::new(("menu_bar_tab", self.titles[i], i)),
                kit::Sense::click(),
            );
            hovered[i] = resp.hovered();
            if resp.pressed() {
                open = if open == Some(i) { None } else { Some(i) };
            } else if hovered[i] && open.is_some_and(|cur| cur != i) {
                open = Some(i);
            }
        }

        for (i, title) in self.titles.iter().enumerate() {
            let item_open = open == Some(i);
            if hovered[i] || item_open {
                Self::paint_hover(&painter, rects[i], hover, border);
            }
            app.text(
                &painter,
                rects[i].center(),
                kit::Align2::CENTER_CENTER,
                title,
                text,
            );
        }

        for rect in &rects {
            vline_hard(
                &painter,
                rect.max.x - LINE_W,
                self.bar.min.y + LINE_W,
                self.bar.max.y - LINE_W,
                separator,
            );
        }

        hline_hard(
            &painter,
            self.bar.min.x,
            self.bar.max.x,
            self.bar.max.y - LINE_W,
            border,
        );

        MenuBarOut { rects, open }
    }

    fn paint_hover(
        painter: &kit::Painter,
        rect: kit::Rect,
        hover: kit::Color32,
        border: kit::Color32,
    ) {
        painter.rect_filled(rect, kit::CornerRadius::ZERO, hover);
        border_hard(painter, rect, border);
    }
}

pub(crate) fn menu_bar_ui(
    app: &mut AppState,
    ui: &mut kit::Ui,
    bar: kit::Rect,
    titles: &[&'static str],
    align: MenuAlign,
    open: Option<usize>,
    style: MenuBarStyle,
) -> MenuBarOut {
    MenuBar::new(bar, titles, align, style).ui(app, ui, open)
}

pub(crate) fn ui_menu_bar(
    app: &mut AppState,
    ui: &mut kit::Ui,
    bar: kit::Rect,
    tabs: &[MenuTab],
    state: MenuState,
) -> (Option<(usize, kit::Rect)>, MenuState) {
    let titles: Vec<&str> = tabs.iter().map(|t| t.title).collect();
    let out = menu_bar_ui(
        app,
        ui,
        bar,
        &titles,
        MenuAlign::Left,
        state.open,
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
    app: &mut AppState,
    ui: &mut kit::Ui,
    anchor: kit::Rect,
    items: &'static [MenuNode],
    state: &mut MenuState,
    bounds: kit::Rect,
    keep: &[kit::Rect],
) -> Option<&'static str> {
    ui_menu_popup_inner(app, ui, anchor, items, state, bounds, keep)
}

pub(crate) fn ui_menu_popup_persistent(
    app: &mut AppState,
    ui: &mut kit::Ui,
    anchor: kit::Rect,
    items: &'static [MenuNode],
    state: &mut MenuState,
    bounds: kit::Rect,
    keep: &[kit::Rect],
) -> Option<&'static str> {
    ui_menu_popup_inner(app, ui, anchor, items, state, bounds, keep)
}

fn ui_menu_popup_inner(
    app: &mut AppState,
    ui: &mut kit::Ui,
    anchor: kit::Rect,
    items: &'static [MenuNode],
    state: &mut MenuState,
    bounds: kit::Rect,
    keep: &[kit::Rect],
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
    keep_rects.extend(chain.iter().map(|(panel, _)| panel.frame_rect()));
    if pointer.is_some_and(|p| {
        chain
            .iter()
            .any(|(panel, _)| panel.frame_rect().contains(p))
    }) {
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
                kit::pos2(text_left_x(row.min.x, panel.rect.min.x), row.center().y),
                kit::Align2::LEFT_CENTER,
                node.label,
                theme().menu_popup_text,
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

fn paint_submenu_arrow(painter: &kit::Painter, row: kit::Rect) {
    let x = (row.max.x - PITCH + LINE_W).floor();
    let y = (row.center().y - 2.5).floor();
    for (i, w) in [1.0f32, 2.0, 3.0, 2.0, 1.0].into_iter().enumerate() {
        painter.rect_filled(
            kit::Rect::from_min_size(kit::pos2(x, y + i as f32), kit::vec2(w, LINE_W)),
            kit::CornerRadius::ZERO,
            theme().menu_text,
        );
    }
}

pub(crate) fn close_on_outside_click(ui: &kit::Ui, keep: &[kit::Rect], state: &mut MenuState) {
    if state.open.is_none() {
        return;
    }
    let outside = ui.ctx().input(|i| {
        i.pointer.primary_pressed()
            && i.pointer
                .interact_pos()
                .is_some_and(|p| !keep.iter().any(|r| r.contains(p)))
    });
    if outside {
        state.open = None;
    }
}
