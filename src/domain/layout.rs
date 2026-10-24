use crate::engine::kit;

pub(crate) const GRID: f32 = 8.0;
pub(crate) const LINE_W: f32 = 1.0;
pub(crate) const PITCH: f32 = GRID + LINE_W;

pub(crate) const fn cells(n: i32) -> f32 {
    n as f32 * PITCH + LINE_W
}

pub(crate) const fn grid_line(i: i32) -> f32 {
    i as f32 * PITCH
}

pub(crate) fn snap_cells_up(v: f32) -> f32 {
    let n = ((v - LINE_W) / PITCH).ceil().max(0.0);
    n * PITCH + LINE_W
}

pub(crate) fn grid_band(area: kit::Rect, row: i32, rows: i32) -> kit::Rect {
    kit::Rect::from_min_size(
        kit::pos2(area.min.x, area.min.y + grid_line(row)),
        kit::vec2(area.width(), cells(rows)),
    )
}

pub(crate) fn free_band(area: kit::Rect, row: i32, rows: i32) -> kit::Rect {
    kit::Rect::from_min_size(
        kit::pos2(area.min.x, area.min.y + grid_line(row)),
        kit::vec2(area.width(), cells(rows)),
    )
}

pub(crate) fn free_tail(area: kit::Rect, row: i32) -> kit::Rect {
    kit::Rect::from_min_max(kit::pos2(area.min.x, area.min.y + grid_line(row)), area.max)
}

#[cfg(test)]
pub(crate) fn snap_cells_round(v: f32) -> f32 {
    let n = ((v - LINE_W) / PITCH).round().max(0.0);
    n * PITCH + LINE_W
}

pub(crate) const TITLE_ROWS: i32 = 2;
pub(crate) const MENU_ROWS: i32 = 2;
pub(crate) const WORK_ROWS: i32 = 94;
pub(crate) const TASKBAR_ROWS: i32 = 2;
pub(crate) const STATUS_ROWS: i32 = 2;

pub(crate) const TITLEBAR_H: f32 = cells(TITLE_ROWS);
pub(crate) const MENUBAR_H: f32 = cells(MENU_ROWS);
pub(crate) const TASKBAR_H: f32 = cells(TASKBAR_ROWS);
pub(crate) const STATUS_H: f32 = cells(STATUS_ROWS);

pub(crate) const TITLE_LINE: i32 = 0;
pub(crate) const MENU_LINE: i32 = TITLE_LINE + TITLE_ROWS;
pub(crate) const WORK_LINE: i32 = MENU_LINE + MENU_ROWS;
pub(crate) const TASKBAR_LINE: i32 = WORK_LINE + WORK_ROWS;
pub(crate) const STATUS_LINE: i32 = TASKBAR_LINE + TASKBAR_ROWS;
pub(crate) const WINDOW_ROWS: i32 = STATUS_LINE + STATUS_ROWS;

pub(crate) const WINDOW_COLS: i32 = 125;

pub(crate) const SPLITTER_COLS: i32 = 1;
pub(crate) const CHART_COLS_DEFAULT: i32 = 89;
pub(crate) const PANEL_COLS: i32 = WINDOW_COLS - CHART_COLS_DEFAULT - SPLITTER_COLS;

pub(crate) const PANE_MIN_COLS: i32 = 35;

pub(crate) const WINDOW_W: f32 = cells(WINDOW_COLS);
pub(crate) const WINDOW_H: f32 = cells(WINDOW_ROWS);

pub(crate) const WINDOW_MIN_COLS: i32 = PANE_MIN_COLS * 2 + SPLITTER_COLS;
pub(crate) const LEFT_WORK_MIN_ROWS: i32 = 3 * PANE_ROWS_MIN + 2;
pub(crate) const WINDOW_MIN_ROWS: i32 =
    TITLE_ROWS + MENU_ROWS + TASKBAR_ROWS + STATUS_ROWS + LEFT_WORK_MIN_ROWS;
pub(crate) const WINDOW_MIN_W: f32 = cells(WINDOW_MIN_COLS);
pub(crate) const WINDOW_MIN_H: f32 = cells(WINDOW_MIN_ROWS);

pub(crate) const SYS_BTN_COLS: i32 = 2;
pub(crate) const SYS_BTN_W: f32 = cells(SYS_BTN_COLS);
pub(crate) const SYS_BTN_H: f32 = cells(SYS_BTN_COLS);

pub(crate) const MENU_PANEL_W: f32 = cells(16);
pub(crate) const MENU_ROW_ROWS: i32 = 2;
pub(crate) const MENU_ROW_H: f32 = cells(MENU_ROW_ROWS);

pub(crate) const TEXT_PAD_X: f32 = LINE_W + GRID;

pub(crate) const TEXT_INSET_X: f32 = LINE_W + PITCH;

pub(crate) const TEXT_ROWS: i32 = 2;

pub(crate) fn grid_floor(v: f32, origin: f32) -> f32 {
    origin + ((v - origin) / PITCH).floor() * PITCH
}

pub(crate) fn text_left_x(rect_left: f32, origin: f32) -> f32 {
    grid_floor(rect_left, origin) + TEXT_INSET_X
}

pub(crate) fn text_right_x(line_x: f32, origin: f32) -> f32 {
    grid_floor(line_x, origin) - GRID
}

pub(crate) const AXIS_COLS: i32 = 14;
pub(crate) const AXIS_W: f32 = cells(AXIS_COLS);

pub(crate) const TIME_LABEL_GAP_MIN: f32 = 48.0;

pub(crate) const WIN_TITLE_ROWS: i32 = 2;
pub(crate) const WIN_TITLE_H: f32 = cells(WIN_TITLE_ROWS);

pub(crate) const WIN_CLOSE_W: f32 = cells(2);

pub(crate) const BAR_SCALE_DEFAULT: u32 = 5;

pub(crate) const SCALE_MIN: u32 = 3;
pub(crate) const SCALE_MAX: u32 = 15;

pub(crate) const SCALE_GAP_FROM: u32 = 3;

pub(crate) const WHEEL_POINTS_PER_NOTCH: f32 = 40.0;

pub(crate) fn bar_advance(scale: u32) -> f32 {
    let scale = snap_scale(scale);
    scale as f32 + if scale >= SCALE_GAP_FROM { LINE_W } else { 0.0 }
}

pub(crate) fn snap_scale(scale: u32) -> u32 {
    (scale | 1).clamp(SCALE_MIN, SCALE_MAX)
}

pub(crate) fn scale_after(scale: u32, notches: i32) -> u32 {
    let next = scale as i64 + 2 * notches as i64;
    snap_scale(next.clamp(SCALE_MIN as i64, SCALE_MAX as i64) as u32)
}

pub(crate) fn visible_bars(plot_w: f32, scale: u32) -> usize {
    (plot_w / bar_advance(scale)).floor().max(1.0) as usize
}

pub(crate) const BAR_COUNT: usize = 512;

pub(crate) const BAR_SEED: u64 = 20_261_012;
pub(crate) const BAR_START: f64 = 20.0;

pub(crate) const CROSS_LABEL_ROWS: i32 = 2;
pub(crate) const CROSS_LABEL_H: f32 = cells(CROSS_LABEL_ROWS);

pub(crate) const PANE_TITLE_ROWS: i32 = 2;
pub(crate) const PANE_TITLE_H: f32 = cells(PANE_TITLE_ROWS);

pub(crate) const CHART_BAR_ROWS: i32 = TEXT_ROWS;
pub(crate) const CHART_BAR_H: f32 = cells(CHART_BAR_ROWS);

pub(crate) const PANE_ROWS_MIN: i32 = 22;
pub(crate) const PANE_H_MIN: f32 = cells(PANE_ROWS_MIN);

pub(crate) const PANE_PLOT_ROWS_MIN: i32 = PANE_ROWS_MIN - PANE_TITLE_ROWS - 2 - 2 - CHART_BAR_ROWS;

pub(crate) const PANE_BTN_COLS: i32 = 3;
pub(crate) const PANE_BTN_W: f32 = cells(PANE_BTN_COLS);
pub(crate) const PANE_BTN_H: f32 = cells(PANE_TITLE_ROWS);

pub(crate) const PANE_MENU_COLS: i32 = 20;
pub(crate) const PANE_MENU_W: f32 = cells(PANE_MENU_COLS);
pub(crate) const PANE_MENU_ROW_ROWS: i32 = 2;
pub(crate) const PANE_MENU_ROW_H: f32 = cells(PANE_MENU_ROW_ROWS);

pub(crate) const OVERLAY_MENU_COLS: i32 = 10;
pub(crate) const OVERLAY_MENU_W: f32 = cells(OVERLAY_MENU_COLS);

pub(crate) const SRC_COLS: i32 = 80;
pub(crate) const SRC_ROWS: i32 = 61;
pub(crate) const SRC_W: f32 = cells(SRC_COLS);
pub(crate) const SRC_H: f32 = cells(SRC_ROWS);
pub(crate) const SRC_MIN_COLS: i32 = 40;
pub(crate) const SRC_MIN_ROWS: i32 = 30;
pub(crate) const SRC_MIN_W: f32 = cells(SRC_MIN_COLS);
pub(crate) const SRC_MIN_H: f32 = cells(SRC_MIN_ROWS);
pub(crate) const SRC_TITLE_ROWS: i32 = PANE_TITLE_ROWS;
pub(crate) const SRC_TITLE_H: f32 = cells(SRC_TITLE_ROWS);

pub(crate) const SRC_MENU_ROWS: i32 = TEXT_ROWS;
pub(crate) const SRC_BAR_ROWS: i32 = TEXT_ROWS;

pub(crate) const SRC_HEAD_ROWS: i32 = SRC_TITLE_ROWS + SRC_MENU_ROWS + SRC_BAR_ROWS;
pub(crate) const SRC_GUTTER_COLS: i32 = 6;
pub(crate) const SRC_GUTTER_W: f32 = cells(SRC_GUTTER_COLS);
pub(crate) const SRC_BAR_W: f32 = GRID;
pub(crate) const SRC_LINE_H: f32 = 2.0 * GRID + LINE_W;

pub(crate) const SRC_ROW_PITCH: f32 = grid_line(2);

pub(crate) const CARET_W: f32 = 2.0;

pub(crate) const CARET_H: f32 = 13.0;

pub(crate) fn text_caret_rect(x: f32, line_top: f32, line_height: f32) -> kit::Rect {
    kit::Rect::from_min_size(
        kit::pos2(x, line_top + (line_height - CARET_H).max(0.0) / 2.0),
        kit::vec2(CARET_W, CARET_H.min(line_height)),
    )
}

pub(crate) const SRC_BODY_TOP: f32 = grid_line(SRC_HEAD_ROWS);

pub(crate) const SRC_BODY_ROWS: i32 = SRC_ROWS - SRC_HEAD_ROWS - 1;

pub(crate) const SRC_VISIBLE_LINES: usize = SRC_BODY_ROWS as usize / 2;

pub(crate) const SRC_DEFAULT_X: f32 = 198.0;
pub(crate) const SRC_DEFAULT_Y: f32 = 99.0;

const _: () = assert!(PITCH == 9.0);
const _: () = assert!((WINDOW_W as i32 - 1) % 9 == 0);
const _: () = assert!((WINDOW_H as i32 - 1) % 9 == 0);
const _: () = assert!((WINDOW_MIN_W as i32 - 1) % 9 == 0);
const _: () = assert!((WINDOW_MIN_H as i32 - 1) % 9 == 0);

const _: () = assert!(LEFT_WORK_MIN_ROWS == 3 * PANE_ROWS_MIN + 2);

const _: () = assert!(
    PANE_ROWS_MIN == PANE_TITLE_ROWS + TEXT_ROWS + TEXT_ROWS + CHART_BAR_ROWS + PANE_PLOT_ROWS_MIN
);
const _: () = assert!((CHART_BAR_H as i32 - 1) % 9 == 0);
const _: () = assert!(PANE_PLOT_ROWS_MIN >= 8);

const _: () = assert!(TEXT_ROWS == 2);
const _: () = assert!(
    TITLE_ROWS == TEXT_ROWS
        && MENU_ROWS == TEXT_ROWS
        && TASKBAR_ROWS == TEXT_ROWS
        && STATUS_ROWS == TEXT_ROWS
);
const _: () = assert!(PANE_TITLE_ROWS == TEXT_ROWS && PANE_MENU_ROW_ROWS == TEXT_ROWS);
const _: () = assert!(MENU_ROW_ROWS == TEXT_ROWS);
const _: () = assert!((PANE_H_MIN as i32 - 1) % 9 == 0);
const _: () = assert!((TITLEBAR_H as i32 - 1) % 9 == 0);
const _: () = assert!((MENUBAR_H as i32 - 1) % 9 == 0);
const _: () = assert!((TASKBAR_H as i32 - 1) % 9 == 0);
const _: () = assert!((STATUS_H as i32 - 1) % 9 == 0);
const _: () = assert!((SYS_BTN_W as i32 - 1) % 9 == 0);
const _: () = assert!((SYS_BTN_H as i32 - 1) % 9 == 0);
const _: () = assert!((MENU_PANEL_W as i32 - 1) % 9 == 0);
const _: () = assert!((MENU_ROW_H as i32 - 1) % 9 == 0);
const _: () = assert!((AXIS_W as i32 - 1) % 9 == 0);
const _: () = assert!((CROSS_LABEL_H as i32 - 1) % 9 == 0);

const _: () = assert!((PANE_TITLE_H as i32 - 1) % 9 == 0);
const _: () = assert!((PANE_BTN_W as i32 - 1) % 9 == 0 && (PANE_BTN_H as i32 - 1) % 9 == 0);
const _: () = assert!((PANE_MENU_W as i32 - 1) % 9 == 0 && (PANE_MENU_ROW_H as i32 - 1) % 9 == 0);
const _: () = assert!((OVERLAY_MENU_W as i32 - 1) % 9 == 0);
const _: () = assert!((SRC_W as i32 - 1) % 9 == 0 && (SRC_H as i32 - 1) % 9 == 0);
const _: () = assert!((SRC_MIN_W as i32 - 1) % 9 == 0 && (SRC_MIN_H as i32 - 1) % 9 == 0);
const _: () = assert!((SRC_TITLE_H as i32 - 1) % 9 == 0 && (SRC_GUTTER_W as i32 - 1) % 9 == 0);

const _: () =
    assert!(SRC_MENU_ROWS == TEXT_ROWS && SRC_BAR_ROWS == TEXT_ROWS && SRC_HEAD_ROWS == 6);
const _: () = assert!((SRC_BODY_TOP as i32) % 9 == 0 && SRC_BODY_ROWS > 0);

const _: () = assert!((WIN_TITLE_H as i32 - 1) % 9 == 0 && (WIN_CLOSE_W as i32 - 1) % 9 == 0);
const _: () = assert!(WIN_TITLE_ROWS == SRC_TITLE_ROWS && WIN_TITLE_ROWS == 2);

const _: () = assert!(SRC_BODY_ROWS % 2 == 0 && SRC_BODY_ROWS > 0);
const _: () = assert!(SRC_VISIBLE_LINES * 2 == SRC_BODY_ROWS as usize);

const _: () = assert!((SRC_DEFAULT_X as i32) % 9 == 0 && (SRC_DEFAULT_Y as i32) % 9 == 0);

const _: () = assert!(SCALE_MIN % 2 == 1 && SCALE_MAX % 2 == 1 && BAR_SCALE_DEFAULT % 2 == 1);

const _: () = assert!(CHART_COLS_DEFAULT + SPLITTER_COLS + PANEL_COLS == WINDOW_COLS);

pub(crate) fn px_center(v: f32) -> f32 {
    v.floor() + 0.5
}

pub(crate) fn hline_hard(painter: &kit::Painter, x0: f32, x1: f32, y: f32, color: kit::Color32) {
    painter.rect_filled(hline_rect(x0, x1, y), kit::CornerRadius::ZERO, color);
}

pub(crate) fn hline_hard_width(
    painter: &kit::Painter,
    x0: f32,
    x1: f32,
    y: f32,
    color: kit::Color32,
    width: u32,
) {
    let width = width.max(1) as f32;
    painter.rect_filled(
        kit::Rect::from_min_size(
            kit::pos2(x0, px_center(y) - width / 2.0),
            kit::vec2(x1 - x0, width),
        ),
        kit::CornerRadius::ZERO,
        color,
    );
}

pub(crate) fn vline_hard(painter: &kit::Painter, x: f32, y0: f32, y1: f32, color: kit::Color32) {
    painter.rect_filled(vline_rect(x, y0, y1), kit::CornerRadius::ZERO, color);
}

pub(crate) fn vline_hard_width(
    painter: &kit::Painter,
    x: f32,
    y0: f32,
    y1: f32,
    color: kit::Color32,
    width: u32,
) {
    let width = width.max(1) as f32;
    painter.rect_filled(
        kit::Rect::from_min_size(
            kit::pos2(px_center(x) - width / 2.0, y0),
            kit::vec2(width, y1 - y0),
        ),
        kit::CornerRadius::ZERO,
        color,
    );
}

pub(crate) fn hline_rect(x0: f32, x1: f32, y: f32) -> kit::Rect {
    kit::Rect::from_min_size(
        kit::pos2(x0, px_center(y) - 0.5),
        kit::vec2(x1 - x0, LINE_W),
    )
}

pub(crate) fn vline_rect(x: f32, y0: f32, y1: f32) -> kit::Rect {
    kit::Rect::from_min_size(
        kit::pos2(px_center(x) - 0.5, y0),
        kit::vec2(LINE_W, y1 - y0),
    )
}

pub(crate) fn border_rects(rect: kit::Rect) -> [kit::Rect; 4] {
    [
        hline_rect(rect.min.x, rect.max.x, rect.min.y),
        hline_rect(rect.min.x, rect.max.x, rect.max.y - LINE_W),
        vline_rect(rect.min.x, rect.min.y, rect.max.y),
        vline_rect(rect.max.x - LINE_W, rect.min.y, rect.max.y),
    ]
}

pub(crate) fn clamp_soft(v: f32, lo: f32, hi: f32) -> f32 {
    if hi.is_nan() || lo.is_nan() || hi < lo {
        lo
    } else {
        v.clamp(lo, hi)
    }
}

pub(crate) fn floating_label_top(
    y_center: f32,
    rows: i32,
    top_line: f32,
    bot_line: f32,
    _origin: f32,
) -> f32 {
    let span = PITCH * rows as f32;
    let snap = (y_center - cells(rows) / 2.0).round();
    clamp_soft(snap, top_line, bot_line - span)
}

pub(crate) fn tag_box(
    painter: &kit::Painter,
    rect: kit::Rect,
    bg: kit::Color32,
    border: kit::Color32,
) {
    painter.rect_filled(rect, kit::CornerRadius::ZERO, bg);
    border_hard(painter, rect, border);
}

pub(crate) fn border_hard(painter: &kit::Painter, rect: kit::Rect, color: kit::Color32) {
    for r in border_rects(rect) {
        painter.rect_filled(r, kit::CornerRadius::ZERO, color);
    }
}

pub(crate) fn interact_rect(
    ui: &mut kit::Ui,
    rect: kit::Rect,
    id: kit::Id,
    sense: kit::Sense,
) -> kit::Response {
    ui.interact(rect, id.with(ui.interaction_salt()), sense)
}

pub(crate) fn polyline_hard_width(
    painter: &kit::Painter,
    pts: &[(f32, f32)],
    color: kit::Color32,
    width: u32,
) {
    let width = width.max(1) as f32;
    let mut mesh = kit::Mesh::default();
    mesh.vertices.reserve(pts.len().saturating_mul(4));
    mesh.indices.reserve(pts.len().saturating_mul(6));

    if pts.len() == 1 {
        let half = (width - 1.0) * 0.5;
        mesh.add_colored_rect(
            kit::Rect::from_min_max(
                kit::pos2(pts[0].0.round() - half, pts[0].1.round() - half),
                kit::pos2(
                    pts[0].0.round() - half + width,
                    pts[0].1.round() - half + width,
                ),
            ),
            color,
        );
        painter.add(kit::Shape::mesh(mesh));
        return;
    }
    for pair in pts.windows(2) {
        mesh.add_hard_line(
            kit::pos2(pair[0].0, pair[0].1),
            kit::pos2(pair[1].0, pair[1].1),
            width,
            color,
        );
    }
    painter.add(kit::Shape::mesh(mesh));
}
