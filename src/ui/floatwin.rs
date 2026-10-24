use crate::application::app::*;
use crate::domain::layout::*;
use crate::engine::kit;
use crate::engine::theme::*;
use crate::ui::icon::*;
use crate::ui::titlebar::*;

pub(crate) const WIN_FRAME: f32 = grid_line(1);

pub(crate) const EDGE_BAND: f32 = WIN_FRAME;

pub(crate) const WINDOW_DRAG_SPEED: f32 = 1.4;

#[derive(Clone)]
pub(crate) struct Win {
    pub(crate) id: u64,
    pub(crate) open: bool,

    pub(crate) pos: kit::Pos2,

    pub(crate) size: kit::Vec2,

    pub(crate) min_size: kit::Vec2,
    title_min_w: f32,

    pub(crate) resizable: bool,

    pub(crate) maximized: bool,

    max_restore: (kit::Pos2, kit::Vec2),

    drag: Option<kit::Vec2>,

    drag_pin: Option<kit::Pos2>,

    open_guard: bool,

    grip_hot: WinEdge,
    grip_down: bool,

    handle_hot: bool,

    rs: WinEdge,

    pub(crate) edge_guard: kit::Rect,

    rs_fix: kit::Rect,

    rs_off: kit::Vec2,

    move_guards: Vec<(u64, kit::Rect, bool)>,
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub(crate) enum WinEdge {
    #[default]
    None,
    Top,
    TopLeft,
    TopRight,
    Left,
    Right,
    Bottom,
    BottomLeft,
    BottomRight,
}

impl WinEdge {
    pub(crate) fn any(self) -> bool {
        self != WinEdge::None
    }

    fn left(self) -> bool {
        matches!(self, WinEdge::Left | WinEdge::TopLeft | WinEdge::BottomLeft)
    }

    fn right(self) -> bool {
        matches!(
            self,
            WinEdge::Right | WinEdge::TopRight | WinEdge::BottomRight
        )
    }

    fn top(self) -> bool {
        matches!(self, WinEdge::Top | WinEdge::TopLeft | WinEdge::TopRight)
    }

    fn bottom(self) -> bool {
        matches!(
            self,
            WinEdge::Bottom | WinEdge::BottomLeft | WinEdge::BottomRight
        )
    }

    fn cursor(self) -> Option<kit::CursorIcon> {
        match self {
            WinEdge::None => None,
            WinEdge::Left | WinEdge::Right => Some(kit::CursorIcon::ResizeHorizontal),
            WinEdge::Top => Some(kit::CursorIcon::ResizeVertical),
            WinEdge::Bottom => Some(kit::CursorIcon::ResizeVertical),
            WinEdge::TopLeft | WinEdge::BottomRight => Some(kit::CursorIcon::ResizeNwSe),
            WinEdge::TopRight | WinEdge::BottomLeft => Some(kit::CursorIcon::ResizeNeSw),
        }
    }

    pub(crate) fn cursor_kind(self) -> CursorKind {
        match self {
            WinEdge::None => CursorKind::Normal,
            WinEdge::Left | WinEdge::Right => CursorKind::ResizeH,
            WinEdge::Top | WinEdge::Bottom => CursorKind::ResizeV,
            WinEdge::TopLeft | WinEdge::BottomRight => CursorKind::ResizeNwSe,
            WinEdge::TopRight | WinEdge::BottomLeft => CursorKind::ResizeNeSw,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum WinKeys {
    Panel,

    Window,

    PinnedWindow,

    ExclusiveWindow,

    Mini,
}

impl WinKeys {
    pub(crate) fn for_state(self, pinned: bool, exclusive: bool) -> Self {
        match (self, pinned, exclusive) {
            (WinKeys::Window, true, _) => WinKeys::PinnedWindow,
            (WinKeys::Window, false, true) => WinKeys::ExclusiveWindow,
            (keys, _, _) => keys,
        }
    }

    pub(crate) fn shown(self) -> &'static [(SysButton, usize)] {
        match self {
            WinKeys::Panel => &[
                (SysButton::Pin, 0),
                (SysButton::Minimize, 1),
                (SysButton::Close, 2),
            ],
            WinKeys::Window => &[
                (SysButton::Exclusive, 0),
                (SysButton::Pin, 1),
                (SysButton::Minimize, 2),
                (SysButton::Maximize, 3),
                (SysButton::Close, 4),
            ],
            WinKeys::PinnedWindow => &[
                (SysButton::Pin, 0),
                (SysButton::Minimize, 1),
                (SysButton::Maximize, 2),
                (SysButton::Close, 3),
            ],
            WinKeys::ExclusiveWindow => &[
                (SysButton::Exclusive, 0),
                (SysButton::Minimize, 1),
                (SysButton::Maximize, 2),
                (SysButton::Close, 3),
            ],
            WinKeys::Mini => &[(SysButton::Close, 0)],
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct WinOut {
    pub(crate) panel: Option<kit::Rect>,
    pub(crate) restore_maximized: bool,
    pub(crate) exclusive: bool,
    pub(crate) pin: bool,
    pub(crate) minimize: bool,
    pub(crate) maximize: bool,
    pub(crate) close: bool,
}

pub(crate) fn edge_at(rect: kit::Rect, p: kit::Pos2, band: f32) -> WinEdge {
    let on_line =
        (p.x >= rect.min.x && p.x < rect.min.x + LINE_W && p.y >= rect.min.y && p.y < rect.max.y)
            || (p.x >= rect.max.x - LINE_W
                && p.x < rect.max.x
                && p.y >= rect.min.y
                && p.y < rect.max.y)
            || (p.y >= rect.min.y
                && p.y < rect.min.y + LINE_W
                && p.x >= rect.min.x
                && p.x < rect.max.x)
            || (p.y >= rect.max.y - LINE_W
                && p.y < rect.max.y
                && p.x >= rect.min.x
                && p.x < rect.max.x);
    if on_line {
        return WinEdge::None;
    }

    let in_x = p.x >= rect.min.x + LINE_W && p.x < rect.max.x - LINE_W;
    let in_y = p.y >= rect.min.y + LINE_W && p.y < rect.max.y - LINE_W;
    let left = in_y && p.x >= rect.min.x + LINE_W && p.x < rect.min.x + band;
    let right = in_y && p.x >= rect.max.x - band && p.x < rect.max.x - LINE_W;
    let top = in_x && p.y >= rect.min.y + LINE_W && p.y < rect.min.y + band;
    let bottom = in_x && p.y >= rect.max.y - band && p.y < rect.max.y - LINE_W;

    match (top, left, right, bottom) {
        (true, true, _, _) => WinEdge::TopLeft,
        (true, _, true, _) => WinEdge::TopRight,
        (_, true, _, true) => WinEdge::BottomLeft,
        (_, _, true, true) => WinEdge::BottomRight,
        (true, _, _, _) => WinEdge::Top,
        (_, true, _, _) => WinEdge::Left,
        (_, _, true, _) => WinEdge::Right,
        (_, _, _, true) => WinEdge::Bottom,
        _ => WinEdge::None,
    }
}

fn edge_target(v: f32, lo: f32, hi: f32, _band: f32) -> f32 {
    v.round().clamp(lo, hi)
}

fn press_anchor(ui: &kit::Ui) -> Option<kit::Pos2> {
    ui.input(|i| i.pointer.press_origin().or_else(|| i.pointer.latest_pos()))
}

pub(crate) fn overlap_span(a0: f32, a1: f32, b0: f32, b1: f32) -> f32 {
    (a1.min(b1) - a0.max(b0)).max(0.0)
}

#[cfg(test)]
pub(crate) fn shares_border(a: kit::Rect, b: kit::Rect) -> bool {
    let w = overlap_span(a.min.x, a.max.x, b.min.x, b.max.x);
    let h = overlap_span(a.min.y, a.max.y, b.min.y, b.max.y);
    w <= LINE_W || h <= LINE_W
}

pub(crate) fn interior_overlap(a: kit::Rect, b: kit::Rect) -> bool {
    let w = overlap_span(a.min.x, a.max.x, b.min.x, b.max.x);
    let h = overlap_span(a.min.y, a.max.y, b.min.y, b.max.y);
    w > LINE_W && h > LINE_W
}

fn fits_bounds(rect: kit::Rect, bounds: kit::Rect) -> bool {
    rect.min.x >= bounds.min.x - 0.01
        && rect.min.y >= bounds.min.y - 0.01
        && rect.max.x <= bounds.max.x + 0.01
        && rect.max.y <= bounds.max.y + 0.01
}

fn guard_side_position(
    guard: kit::Rect,
    start: kit::Rect,
    desired: kit::Pos2,
    size: kit::Vec2,
    bounds: kit::Rect,
) -> Option<kit::Pos2> {
    let target = kit::Rect::from_min_size(desired, size);
    let vertical = overlap_span(target.min.y, target.max.y, guard.min.y, guard.max.y);
    let horizontal = overlap_span(target.min.x, target.max.x, guard.min.x, guard.max.x);
    let mut best: Option<kit::Pos2> = None;
    let mut best_distance = f32::INFINITY;
    let mut consider = |pos: kit::Pos2| {
        let rect = kit::Rect::from_min_size(pos, size);
        if !fits_bounds(rect, bounds) {
            return;
        }
        let distance = (pos - desired).length_sq();
        if distance < best_distance {
            best = Some(pos);
            best_distance = distance;
        }
    };
    if start.max.x <= guard.min.x + LINE_W
        && desired.x + size.x > guard.min.x + LINE_W
        && vertical > LINE_W
    {
        consider(kit::pos2(guard.min.x + LINE_W - size.x, desired.y));
    }
    if start.min.x >= guard.max.x - LINE_W && desired.x < guard.max.x - LINE_W && vertical > LINE_W
    {
        consider(kit::pos2(guard.max.x - LINE_W, desired.y));
    }
    if start.max.y <= guard.min.y + LINE_W
        && desired.y + size.y > guard.min.y + LINE_W
        && horizontal > LINE_W
    {
        consider(kit::pos2(desired.x, guard.min.y + LINE_W - size.y));
    }
    if start.min.y >= guard.max.y - LINE_W
        && desired.y < guard.max.y - LINE_W
        && horizontal > LINE_W
    {
        consider(kit::pos2(desired.x, guard.max.y - LINE_W));
    }
    best
}

impl Win {
    pub(crate) fn new(id: u64, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            id,
            open: false,
            pos: kit::pos2(x, y),
            size: kit::vec2(w, h),
            min_size: kit::vec2(w, h),
            title_min_w: w,
            resizable: false,
            maximized: false,
            max_restore: (kit::pos2(x, y), kit::vec2(w, h)),
            drag: None,
            drag_pin: None,
            open_guard: false,
            grip_hot: WinEdge::None,
            grip_down: false,
            handle_hot: false,
            rs: WinEdge::None,
            edge_guard: kit::Rect::NOTHING,
            rs_fix: kit::Rect::from_min_size(kit::pos2(x, y), kit::vec2(w, h)),
            rs_off: kit::Vec2::ZERO,
            move_guards: Vec::new(),
        }
    }

    pub(crate) fn set_move_guards(
        &mut self,
        guards: impl IntoIterator<Item = (u64, kit::Rect, bool)>,
    ) {
        self.move_guards.clear();
        self.move_guards.extend(guards);
    }

    pub(crate) fn rect(&self) -> kit::Rect {
        kit::Rect::from_min_size(self.pos, self.size)
    }

    pub(crate) fn inner_rect(&self) -> kit::Rect {
        self.rect().shrink(WIN_FRAME)
    }

    pub(crate) fn title_band(&self) -> kit::Rect {
        free_band(self.inner_rect(), 0, WIN_TITLE_ROWS)
    }

    pub(crate) fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    pub(crate) fn resizing(&self) -> bool {
        self.rs.any()
    }

    pub(crate) fn resize_edge(&self) -> Option<WinEdge> {
        self.rs.any().then_some(self.rs)
    }

    pub(crate) fn hover_kind(&self) -> Option<CursorKind> {
        if let Some(edge) = self.resize_edge() {
            return Some(edge.cursor_kind());
        }
        if self.drag.is_some() {
            return Some(CursorKind::Move);
        }
        if self.grip_hot.any() {
            return Some(self.grip_hot.cursor_kind());
        }
        if self.handle_hot {
            return Some(CursorKind::Move);
        }
        None
    }

    pub(crate) fn begin_resize(&mut self, p: kit::Pos2) {
        if !self.resizable || self.maximized || self.open_guard {
            return;
        }
        let rect = self.rect();
        if self.edge_guard.contains(p) {
            return;
        }
        let edge = edge_at(rect, p, EDGE_BAND);
        if !edge.any() {
            return;
        }
        self.rs = edge;
        self.rs_fix = rect;

        self.rs_off = kit::vec2(
            if edge.left() {
                rect.min.x - p.x
            } else if edge.right() {
                rect.max.x - p.x
            } else {
                0.0
            },
            if edge.bottom() {
                rect.max.y - p.y
            } else if edge.top() {
                rect.min.y - p.y
            } else {
                0.0
            },
        );
        self.drag = None;
        self.drag_pin = None;
    }

    pub(crate) fn end_sessions(&mut self) {
        self.drag = None;
        self.drag_pin = None;
        self.rs = WinEdge::None;
    }

    #[cfg(test)]
    pub(crate) fn set_dragging(&mut self, on: bool) {
        self.drag = if on { Some(kit::Vec2::ZERO) } else { None };
    }

    pub(crate) fn guard_open_click(&mut self) {
        self.open_guard = true;
        self.end_sessions();
    }

    pub(crate) fn clamp_into(&mut self, bounds: kit::Rect) {
        let min_w = self.min_size.x.max(self.title_min_w);
        self.size.x = clamp_soft(self.size.x, min_w.min(bounds.width()), bounds.width());
        self.size.y = clamp_soft(
            self.size.y,
            self.min_size.y.min(bounds.height()),
            bounds.height(),
        );
        self.pos.x = clamp_soft(self.pos.x, bounds.min.x, bounds.max.x - self.size.x);
        self.pos.y = clamp_soft(self.pos.y, bounds.min.y, bounds.max.y - self.size.y);
    }

    pub(crate) fn toggle_maximize(&mut self, bounds: kit::Rect) {
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

    pub(crate) fn restore_rect(&self) -> (kit::Pos2, kit::Vec2) {
        if self.maximized {
            self.max_restore
        } else {
            (self.pos, self.size)
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "hard-edge primitive keeps geometry arguments explicit"
    )]
    pub(crate) fn begin(
        &mut self,
        app: &mut AppState,
        ui: &mut kit::Ui,
        bounds: kit::Rect,
        title_text: &str,
        keys: WinKeys,
        busy: bool,
        active: bool,
        pinned: bool,
        minimized: bool,
        exclusive: bool,
    ) -> WinOut {
        let keys = keys.for_state(pinned, exclusive);
        let title_width = snap_cells_up(app.text_width(title_text) + 2.0 * TEXT_PAD_X);
        self.title_min_w = 2.0 * WIN_FRAME
            + title_width.max(cells(3))
            + sys_keys_width(keys.shown(), self.maximized, minimized, pinned, exclusive)
            + LINE_W;
        self.update_geometry(app, ui, bounds);
        let rect = self.rect();

        let painter = ui.painter().clone();

        painter.rect_filled(rect, kit::CornerRadius::ZERO, theme().work_bg);
        let enabled = ui.is_enabled();
        if enabled {
            if active {
                self.grips(ui);
            } else {
                self.grip_hot = WinEdge::None;
                self.grip_down = false;
            }
        } else {
            self.grip_hot = WinEdge::None;
            self.grip_down = false;
            self.handle_hot = false;
        }

        let title = self.title_band();
        let keys_rects = sys_button_rects(
            title,
            keys.shown(),
            self.maximized,
            minimized,
            pinned,
            exclusive,
        );

        let mut handle_end = title.max.x;
        for &(_, slot) in keys.shown() {
            handle_end = handle_end.min(keys_rects[slot].min.x);
        }
        let handle = kit::Rect::from_min_max(
            title.min,
            kit::pos2(handle_end.max(title.min.x), title.max.y),
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
            pinned,
            minimized,
            exclusive,
        );

        let pointer = ui.input(|i| i.pointer.latest_pos()).or(app.pointer.live);
        let in_handle = pointer.is_some_and(|p| handle.contains(p));
        self.handle_hot = enabled && active && in_handle;
        if self.handle_hot {
            ui.ctx().set_cursor_icon(kit::CursorIcon::Move);
        }
        let primary_pressed = ui.input(|i| i.pointer.primary_pressed());
        let double_clicked =
            ui.input(|i| i.pointer.button_double_clicked(kit::PointerButton::Primary));
        let mut restore_maximized = false;
        let mut maximize = false;
        if enabled && active && in_handle {
            if self.maximized {
                restore_maximized = double_clicked;
            } else {
                if double_clicked {
                    maximize = true;
                    self.end_sessions();
                } else if primary_pressed
                    && !busy
                    && !self.open_guard
                    && self.drag.is_none()
                    && !self.rs.any()
                    && let Some(p) = pointer
                {
                    self.drag = Some(p - self.pos);
                    self.drag_pin = Some(p);
                    app.pointer.pinned = Some(p);
                    self.rs = WinEdge::None;
                }
            }
        }

        WinOut {
            panel: Some(rect),
            restore_maximized,
            exclusive: hit == Some(SysButton::Exclusive),
            pin: hit == Some(SysButton::Pin),
            minimize: hit == Some(SysButton::Minimize),
            maximize: maximize || hit == Some(SysButton::Maximize),
            close: hit == Some(SysButton::Close),
        }
    }

    fn update_geometry(&mut self, app: &mut AppState, ui: &mut kit::Ui, bounds: kit::Rect) {
        let frame_rect = self.rect();
        if !ui.input(|i| i.pointer.primary_down()) {
            self.open_guard = false;
            self.end_sessions();
        }

        if self.maximized {
            self.end_sessions();
            self.pos = bounds.min;
            self.size = bounds.size();
            return;
        }

        let pointer = app
            .pointer
            .motion_live
            .or_else(|| ui.input(|i| i.pointer.latest_pos()));
        if let Some(grab) = self.drag {
            let step = match (pointer, self.drag_pin) {
                (Some(p), Some(last)) => p - last,
                (Some(p), None) => p - (self.pos + grab),
                (None, _) => kit::Vec2::ZERO,
            } * WINDOW_DRAG_SPEED;
            let want = kit::pos2(self.pos.x + step.x, self.pos.y + step.y);

            let at = kit::pos2(
                clamp_soft(want.x, bounds.min.x, bounds.max.x - self.size.x),
                clamp_soft(want.y, bounds.min.y, bounds.max.y - self.size.y),
            );
            self.pos = kit::pos2(at.x.round(), at.y.round());
        }
        if self.rs.any() {
            if let Some(p) = pointer {
                self.stretch(p, bounds);
            }
        }

        self.clamp_into(bounds);
        if self.drag.is_some() {
            self.enforce_move_guards(frame_rect, bounds);
        }
        if self.rs.any()
            && !app.pointer.system
            && let Some(p) = pointer
        {
            app.pointer.pinned = Some(self.resize_pointer(p));
        }

        if self.drag.is_some() {
            if app.pointer.system {
                self.drag_pin = pointer;
            } else if let Some(grab) = self.drag {
                let pin = self.pos + grab;
                self.drag_pin = Some(pin);
                app.settle_pointer_pin(pin, ui.input(|i| i.time));
            }
        }
    }

    fn resize_pointer(&self, pointer: kit::Pos2) -> kit::Pos2 {
        let rect = self.rect();
        let x = if self.rs.left() {
            rect.min.x - self.rs_off.x
        } else if self.rs.right() {
            rect.max.x - self.rs_off.x
        } else {
            pointer.x
        };
        let y = if self.rs.top() {
            rect.min.y - self.rs_off.y
        } else if self.rs.bottom() {
            rect.max.y - self.rs_off.y
        } else {
            pointer.y
        };
        kit::pos2(x, y)
    }

    fn enforce_move_guards(&mut self, frame_rect: kit::Rect, bounds: kit::Rect) {
        if self.move_guards.is_empty() {
            return;
        }
        if self
            .move_guards
            .iter()
            .any(|(_, guard, exclusive)| *exclusive && interior_overlap(frame_rect, *guard))
        {
            return;
        }
        let size = self.size;
        for _ in 0..=self.move_guards.len() {
            let desired = self.pos;
            let mut moved = false;
            for (_, guard, _) in &self.move_guards {
                if interior_overlap(frame_rect, *guard) {
                    continue;
                }
                if let Some(next) = guard_side_position(*guard, frame_rect, desired, size, bounds)
                    && next != desired
                {
                    self.pos = next;
                    moved = true;
                    break;
                }
            }
            if !moved {
                break;
            }
        }
    }

    pub(crate) fn end(&mut self, ui: &mut kit::Ui, busy: bool) {
        if !self.maximized {
            self.resize_grips(ui, busy);
        }
        paint_shell_border(&ui.painter().clone(), self.rect());
    }

    fn grips(&mut self, ui: &mut kit::Ui) {
        self.grip_hot = WinEdge::None;
        self.grip_down = false;
        if !self.resizable || self.maximized {
            return;
        }
        let rect = self.rect();
        let pointer = ui.input(|i| i.pointer.hover_pos());
        let Some(p) = pointer else {
            return;
        };
        if self.edge_guard.contains(p) {
            return;
        }
        self.grip_hot = edge_at(rect, p, EDGE_BAND);
        self.grip_down = self.grip_hot.any() && ui.input(|i| i.pointer.primary_down());
    }

    fn resize_grips(&mut self, ui: &mut kit::Ui, busy: bool) {
        if !self.resizable || self.maximized {
            return;
        }

        let hot = if self.rs.any() {
            self.rs
        } else {
            self.grip_hot
        };
        if let Some(cursor) = hot.cursor() {
            ui.ctx().set_cursor_icon(cursor);
        }
        let down = self.grip_down;

        if busy {
            return;
        }
        if down && !self.rs.any() && self.drag.is_none() && ui.input(|i| i.pointer.primary_down()) {
            if let Some(p) = press_anchor(ui) {
                self.begin_resize(p);
            }
        }
    }

    fn stretch(&mut self, p: kit::Pos2, bounds: kit::Rect) {
        let fix = self.rs_fix;
        let min = kit::vec2(self.min_size.x.max(self.title_min_w), self.min_size.y);
        let mut x0 = fix.min.x;
        let mut x1 = fix.max.x;
        let mut y0 = fix.min.y;
        let mut y1 = fix.max.y;
        if self.rs.left() {
            let want = edge_target(p.x + self.rs_off.x, bounds.min.x, bounds.max.x, EDGE_BAND);
            x0 = clamp_soft(want, bounds.min.x, (x1 - min.x).max(bounds.min.x));
        } else if self.rs.right() {
            let want = edge_target(p.x + self.rs_off.x, bounds.min.x, bounds.max.x, EDGE_BAND);
            x1 = clamp_soft(want, (x0 + min.x).min(bounds.max.x), bounds.max.x);
        }
        if self.rs.bottom() {
            let want = edge_target(p.y + self.rs_off.y, bounds.min.y, bounds.max.y, EDGE_BAND);
            y1 = clamp_soft(want, (y0 + min.y).min(bounds.max.y), bounds.max.y);
        } else if self.rs.top() {
            let want = edge_target(p.y + self.rs_off.y, bounds.min.y, bounds.max.y, EDGE_BAND);
            y0 = clamp_soft(want, bounds.min.y, (y1 - min.y).max(bounds.min.y));
        }
        self.clamp_resize_guards(&mut x0, &mut x1, &mut y0, &mut y1);
        self.pos = kit::pos2(x0, y0);
        self.size = kit::vec2((x1 - x0).max(LINE_W), (y1 - y0).max(LINE_W));
    }

    fn clamp_resize_guards(&self, x0: &mut f32, x1: &mut f32, y0: &mut f32, y1: &mut f32) {
        let fix = self.rs_fix;
        if self
            .move_guards
            .iter()
            .any(|(_, guard, exclusive)| *exclusive && interior_overlap(fix, *guard))
        {
            return;
        }
        for (_, guard, _) in &self.move_guards {
            let vertical = overlap_span(*y0, *y1, guard.min.y, guard.max.y);
            if self.rs.right() && fix.max.x <= guard.min.x + LINE_W && vertical > LINE_W {
                *x1 = x1.min(guard.min.x + LINE_W);
            }
            if self.rs.left() && fix.min.x >= guard.max.x - LINE_W && vertical > LINE_W {
                *x0 = x0.max(guard.max.x - LINE_W);
            }
            let horizontal = overlap_span(*x0, *x1, guard.min.x, guard.max.x);
            if self.rs.bottom() && fix.max.y <= guard.min.y + LINE_W && horizontal > LINE_W {
                *y1 = y1.min(guard.min.y + LINE_W);
            }
            if self.rs.top() && fix.min.y >= guard.max.y - LINE_W && horizontal > LINE_W {
                *y0 = y0.max(guard.max.y - LINE_W);
            }
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "hard-edge primitive keeps geometry arguments explicit"
)]
pub(crate) fn paint_title_bar(
    app: &mut AppState,
    ui: &mut kit::Ui,
    painter: &kit::Painter,
    title: kit::Rect,
    _id: u64,
    text: &str,
    keys: WinKeys,
    maximized: bool,
    pinned: bool,
    minimized: bool,
    exclusive: bool,
) -> Option<SysButton> {
    painter.rect_filled(title, kit::CornerRadius::ZERO, theme().title_bg);

    let rects = sys_button_rects(title, keys.shown(), maximized, minimized, pinned, exclusive);
    let mut keys_left = title.max.x;
    for &(_, slot) in keys.shown() {
        keys_left = keys_left.min(rects[slot].min.x);
    }
    let text_slot = title_text_slot(app, title, text);
    let text_slot = kit::Rect::from_min_max(
        text_slot.min,
        kit::pos2(text_slot.max.x.min(keys_left), text_slot.max.y),
    );

    if (text_slot.max.x - keys_left).abs() > 0.5 {
        paint_title_text_edge(painter, text_slot);
    }
    paint_title_fill(painter, title, text_slot, keys_left);

    let text_color = if minimized {
        theme().taskbar_text
    } else {
        theme().title_text
    };
    app.text(
        painter,
        kit::pos2(text_left_x(title.min.x, title.min.x), title.center().y),
        kit::Align2::LEFT_CENTER,
        text,
        text_color,
    );

    let mut hit = None;
    let pointer = ui.input(|i| i.pointer.hover_pos()).or(app.pointer.live);
    let pointer_pressed = ui.input(|i| i.pointer.primary_pressed());
    let window_active = minimized || (ui.is_enabled() && !app.shell.win_silent);
    if minimized
        && window_active
        && pointer.is_some_and(|p| {
            title.contains(p)
                && !keys
                    .shown()
                    .iter()
                    .any(|(_, slot)| rects[*slot].contains(p))
        })
    {
        app.shell.tooltip = Some((title, text.to_owned()));
    }
    for &(kind, slot) in keys.shown() {
        let rect = rects[slot];
        let hot = window_active && pointer.is_some_and(|p| rect.contains(p));
        let restore = minimized && kind == SysButton::Minimize;
        paint_sys_button(
            app, painter, rect, hot, kind, maximized, restore, pinned, exclusive,
        );

        if hot && pointer_pressed {
            hit = Some(kind);
        }
    }
    if minimized {
        app.shell.tooltip = None;
    }
    hit
}

pub(crate) fn paint_shell_border(painter: &kit::Painter, rect: kit::Rect) {
    let inner = rect.shrink(WIN_FRAME);
    hline_hard(
        painter,
        inner.min.x,
        inner.max.x,
        inner.min.y + grid_line(WIN_TITLE_ROWS),
        theme().window_border,
    );
    border_hard(painter, rect, theme().window_border);
    border_hard(painter, inner, theme().window_border);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclusive_guard_stops_on_one_pixel_shared_border() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(800.0, 800.0));
        let guard = kit::Rect::from_min_size(kit::pos2(100.0, 100.0), kit::vec2(200.0, 200.0));
        let mut window = Win::new(7, 500.0, 100.0, 80.0, 80.0);
        window.set_move_guards(vec![(9, guard, true)]);
        window.drag = Some(kit::vec2(1.0, 1.0));
        let frame = window.rect();
        window.pos = kit::pos2(250.0, 150.0);
        window.enforce_move_guards(frame, bounds);
        assert_eq!(window.pos.x, guard.max.x - LINE_W);
        assert!(shares_border(window.rect(), guard));
    }

    #[test]
    fn exclusive_guard_slides_along_the_contact_edge() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(800.0, 800.0));
        let guard = kit::Rect::from_min_size(kit::pos2(100.0, 100.0), kit::vec2(200.0, 200.0));
        let mut window = Win::new(7, 500.0, 100.0, 80.0, 80.0);
        window.set_move_guards(vec![(9, guard, true)]);
        window.drag = Some(kit::vec2(1.0, 1.0));
        let frame = window.rect();
        window.pos = kit::pos2(250.0, 240.0);
        window.enforce_move_guards(frame, bounds);
        assert_eq!(window.pos.x, guard.max.x - LINE_W);
        assert_eq!(window.pos.y, 240.0);
        assert!(shares_border(window.rect(), guard));
    }

    #[test]
    fn exclusive_guard_ignores_overlap_until_the_window_fully_exits() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(800.0, 800.0));
        let guard = kit::Rect::from_min_size(kit::pos2(100.0, 100.0), kit::vec2(200.0, 200.0));
        let mut window = Win::new(7, 150.0, 150.0, 80.0, 80.0);
        window.set_move_guards(vec![(9, guard, true)]);
        window.drag = Some(kit::vec2(1.0, 1.0));
        let frame = window.rect();
        window.pos = kit::pos2(200.0, 200.0);
        window.enforce_move_guards(frame, bounds);
        assert_eq!(window.pos, kit::pos2(200.0, 200.0));

        window.end_sessions();
        window.pos = kit::pos2(500.0, 120.0);
        window.set_move_guards(vec![(9, guard, true)]);
        window.drag = Some(kit::vec2(1.0, 1.0));
        let frame = window.rect();
        window.pos = kit::pos2(250.0, 120.0);
        window.enforce_move_guards(frame, bounds);
        assert_eq!(window.pos.x, guard.max.x - LINE_W);
    }

    #[test]
    fn any_exclusive_overlap_suspends_all_exclusive_guards() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(800.0, 800.0));
        let first = kit::Rect::from_min_size(kit::pos2(100.0, 100.0), kit::vec2(200.0, 200.0));
        let second = kit::Rect::from_min_size(kit::pos2(360.0, 100.0), kit::vec2(200.0, 200.0));
        let mut window = Win::new(7, 150.0, 150.0, 80.0, 80.0);
        window.set_move_guards(vec![(9, first, true), (10, second, true)]);
        window.drag = Some(kit::vec2(1.0, 1.0));
        let frame = window.rect();
        window.pos = kit::pos2(400.0, 120.0);
        window.enforce_move_guards(frame, bounds);
        assert_eq!(window.pos, kit::pos2(400.0, 120.0));
    }

    #[test]
    fn exclusive_rect_does_not_block_an_entire_row_or_column() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(800.0, 800.0));
        let guard = kit::Rect::from_min_size(kit::pos2(100.0, 100.0), kit::vec2(200.0, 200.0));

        let mut row = Win::new(7, 10.0, 10.0, 80.0, 80.0);
        row.set_move_guards(vec![(9, guard, true)]);
        row.drag = Some(kit::vec2(1.0, 1.0));
        let frame = row.rect();
        row.pos = kit::pos2(350.0, 10.0);
        row.enforce_move_guards(frame, bounds);
        assert_eq!(row.pos, kit::pos2(350.0, 10.0));

        let mut column = Win::new(8, 10.0, 120.0, 80.0, 80.0);
        column.set_move_guards(vec![(9, guard, true)]);
        column.drag = Some(kit::vec2(1.0, 1.0));
        let frame = column.rect();
        column.pos = kit::pos2(10.0, 400.0);
        column.enforce_move_guards(frame, bounds);
        assert_eq!(column.pos, kit::pos2(10.0, 400.0));
    }

    #[test]
    fn a_fast_move_does_not_cross_an_exclusive_window() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(800.0, 800.0));
        let guard = kit::Rect::from_min_size(kit::pos2(100.0, 100.0), kit::vec2(200.0, 200.0));
        let mut window = Win::new(7, 10.0, 120.0, 80.0, 80.0);
        window.set_move_guards(vec![(9, guard, true)]);
        window.drag = Some(kit::vec2(1.0, 1.0));
        let frame = window.rect();
        window.pos = kit::pos2(120.0, 120.0);
        window.enforce_move_guards(frame, bounds);
        assert_eq!(window.rect().max.x, guard.min.x + LINE_W);
        assert!(shares_border(window.rect(), guard));
    }

    #[test]
    fn a_fast_resize_stops_on_the_exclusive_border() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(800.0, 800.0));
        let guard = kit::Rect::from_min_size(kit::pos2(100.0, 100.0), kit::vec2(200.0, 200.0));
        let mut window = Win::new(7, 10.0, 120.0, 80.0, 80.0);
        window.set_move_guards(vec![(9, guard, true)]);
        window.rs_fix = window.rect();
        window.rs = WinEdge::Right;
        window.stretch(kit::pos2(400.0, 160.0), bounds);
        assert_eq!(window.rect().max.x, guard.min.x + LINE_W);
        assert!(shares_border(window.rect(), guard));
    }

    #[test]
    fn resize_follows_the_pointer_back_immediately_after_the_border() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(800.0, 800.0));
        let guard = kit::Rect::from_min_size(kit::pos2(100.0, 100.0), kit::vec2(200.0, 200.0));
        let mut window = Win::new(7, 10.0, 120.0, 80.0, 80.0);
        window.set_move_guards(vec![(9, guard, true)]);
        window.rs_fix = window.rect();
        window.rs = WinEdge::Right;
        window.stretch(kit::pos2(400.0, 160.0), bounds);
        assert_eq!(window.rect().max.x, guard.min.x + LINE_W);
        window.stretch(kit::pos2(99.0, 160.0), bounds);
        assert_eq!(window.rect().max.x, 99.0);
    }

    #[test]
    fn resize_stops_at_the_workspace_edge_without_bouncing_back() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(400.0, 400.0));
        let mut window = Win::new(7, 10.0, 120.0, 80.0, 80.0);
        window.resizable = true;
        let press = kit::pos2(window.rect().center().x, window.rect().max.y - 4.0);
        window.begin_resize(press);
        window.stretch(kit::pos2(press.x, 640.0), bounds);
        assert_eq!(window.rect().max.y, bounds.max.y);
        window.stretch(kit::pos2(press.x, 639.0), bounds);
        assert_eq!(window.rect().max.y, bounds.max.y);
        window.stretch(kit::pos2(press.x, bounds.max.y - 5.0), bounds);
        assert_eq!(window.rect().max.y, bounds.max.y - 1.0);
    }

    #[test]
    fn resize_pointer_keeps_the_original_grab_offset_at_the_workspace_edge() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(400.0, 400.0));
        let mut window = Win::new(7, 10.0, 120.0, 80.0, 80.0);
        window.resizable = true;
        let press = kit::pos2(window.rect().max.x - 4.0, window.rect().center().y);
        window.begin_resize(press);
        window.stretch(kit::pos2(900.0, press.y), bounds);
        assert_eq!(window.rect().max.x, bounds.max.x);
        assert_eq!(
            window.resize_pointer(kit::pos2(900.0, press.y)),
            kit::pos2(bounds.max.x - 4.0, press.y)
        );

        let top_press = kit::pos2(window.rect().center().x, window.rect().min.y + 3.0);
        window.begin_resize(top_press);
        window.stretch(kit::pos2(top_press.x, -100.0), bounds);
        assert_eq!(window.rect().min.y, bounds.min.y);
        assert_eq!(
            window.resize_pointer(kit::pos2(top_press.x, -100.0)),
            kit::pos2(top_press.x, bounds.min.y + 3.0)
        );
    }

    #[test]
    fn frame_band_resizes_but_the_outer_line_does_not() {
        let rect = kit::Rect::from_min_size(kit::pos2(100.0, 100.0), kit::vec2(200.0, 160.0));
        assert_eq!(rect.shrink(WIN_FRAME).size(), kit::vec2(182.0, 142.0));
        assert_eq!(
            edge_at(
                rect,
                kit::pos2(rect.max.x - 1.0, rect.center().y),
                EDGE_BAND
            ),
            WinEdge::None
        );
        assert_eq!(
            edge_at(
                rect,
                kit::pos2(rect.max.x - WIN_FRAME / 2.0, rect.center().y),
                EDGE_BAND
            ),
            WinEdge::Right
        );
        assert_eq!(
            edge_at(
                rect,
                kit::pos2(rect.center().x, rect.min.y + LINE_W),
                EDGE_BAND
            ),
            WinEdge::Top
        );
        assert_eq!(
            edge_at(
                rect,
                kit::pos2(rect.min.x + LINE_W, rect.min.y + LINE_W),
                EDGE_BAND
            ),
            WinEdge::TopLeft
        );
    }

    #[test]
    fn maximize_fills_the_work_area_without_exclusive_edges() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(500.0, 500.0));
        let guard = kit::Rect::from_min_size(kit::pos2(100.0, 100.0), kit::vec2(200.0, 200.0));
        let mut window = Win::new(7, 100.0, 150.0, 100.0, 100.0);
        window.set_move_guards(vec![(1, guard, true)]);

        window.toggle_maximize(bounds);

        assert_eq!(window.rect(), bounds);
        assert!(interior_overlap(window.rect(), guard));
    }

    #[test]
    fn pinned_and_exclusive_title_keys_hide_the_opposite_toggle() {
        let normal = WinKeys::Window.for_state(false, false);
        let pinned = WinKeys::Window.for_state(true, false);
        let exclusive = WinKeys::Window.for_state(false, true);
        assert!(
            normal
                .shown()
                .iter()
                .any(|(kind, _)| *kind == SysButton::Exclusive)
        );
        assert!(
            pinned
                .shown()
                .iter()
                .all(|(kind, _)| *kind != SysButton::Exclusive)
        );
        assert_eq!(pinned.shown().len(), 4);
        assert!(
            exclusive
                .shown()
                .iter()
                .all(|(kind, _)| *kind != SysButton::Pin)
        );
        assert!(
            exclusive
                .shown()
                .iter()
                .any(|(kind, _)| *kind == SysButton::Exclusive)
        );
        assert_eq!(exclusive.shown().len(), 4);
    }

    #[test]
    fn a_non_suspending_guard_still_blocks() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(800.0, 800.0));
        let guard = kit::Rect::from_min_size(kit::pos2(100.0, 100.0), kit::vec2(200.0, 200.0));
        let mut window = Win::new(7, 10.0, 120.0, 80.0, 80.0);
        window.set_move_guards(vec![(9, guard, false)]);
        window.drag = Some(kit::vec2(1.0, 1.0));
        let frame = window.rect();
        window.pos = kit::pos2(120.0, 120.0);
        window.enforce_move_guards(frame, bounds);
        assert_eq!(window.rect().max.x, guard.min.x + LINE_W);
        assert!(shares_border(window.rect(), guard));
    }

    #[test]
    fn exclusive_overlap_still_suspends_normal_guards() {
        let bounds = kit::Rect::from_min_size(kit::Pos2::ZERO, kit::vec2(800.0, 800.0));
        let normal = kit::Rect::from_min_size(kit::pos2(360.0, 100.0), kit::vec2(200.0, 200.0));
        let exclusive = kit::Rect::from_min_size(kit::pos2(100.0, 100.0), kit::vec2(200.0, 200.0));
        let mut window = Win::new(7, 150.0, 150.0, 80.0, 80.0);
        window.set_move_guards(vec![(9, normal, false), (10, exclusive, true)]);
        window.drag = Some(kit::vec2(1.0, 1.0));
        let frame = window.rect();
        window.pos = kit::pos2(400.0, 120.0);
        window.enforce_move_guards(frame, bounds);
        assert_eq!(window.pos, kit::pos2(400.0, 120.0));
    }
}
