use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::icon::*;
use crate::ui::titlebar::*;
use eframe::egui;

pub(crate) const EDGE_BAND: f32 = 5.0;

pub(crate) const WIN_ID_COLOR: u64 = 0x43_4c_52;

pub(crate) const WINDOW_DRAG_SPEED: f32 = 1.4;

#[derive(Clone)]
pub(crate) struct Win {
    pub(crate) id: u64,
    pub(crate) open: bool,

    pub(crate) pos: egui::Pos2,

    pub(crate) size: egui::Vec2,

    pub(crate) min_size: egui::Vec2,

    pub(crate) resizable: bool,

    pub(crate) maximized: bool,

    max_restore: (egui::Pos2, egui::Vec2),

    drag: Option<egui::Vec2>,

    drag_pin: Option<egui::Pos2>,

    open_guard: bool,

    grip_hot: WinEdge,
    grip_down: bool,

    handle_hot: bool,

    rs: WinEdge,

    pub(crate) edge_guard: egui::Rect,

    rs_fix: egui::Rect,

    rs_off: egui::Vec2,
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub(crate) enum WinEdge {
    #[default]
    None,
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
        matches!(self, WinEdge::Left | WinEdge::BottomLeft)
    }

    fn right(self) -> bool {
        matches!(self, WinEdge::Right | WinEdge::BottomRight)
    }

    fn bottom(self) -> bool {
        matches!(
            self,
            WinEdge::Bottom | WinEdge::BottomLeft | WinEdge::BottomRight
        )
    }

    fn cursor(self) -> Option<egui::CursorIcon> {
        match self {
            WinEdge::None => None,
            WinEdge::Left | WinEdge::Right => Some(egui::CursorIcon::ResizeHorizontal),
            WinEdge::Bottom => Some(egui::CursorIcon::ResizeVertical),
            WinEdge::BottomRight => Some(egui::CursorIcon::ResizeNwSe),
            WinEdge::BottomLeft => Some(egui::CursorIcon::ResizeNeSw),
        }
    }

    pub(crate) fn cursor_kind(self) -> CursorKind {
        match self {
            WinEdge::None => CursorKind::Normal,
            WinEdge::Left | WinEdge::Right => CursorKind::ResizeH,
            WinEdge::Bottom => CursorKind::ResizeV,
            WinEdge::BottomRight => CursorKind::ResizeNwSe,
            WinEdge::BottomLeft => CursorKind::ResizeNeSw,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum WinKeys {
    CloseOnly,

    MinAndClose,

    MinMaxClose,
}

impl WinKeys {
    fn shown(self) -> &'static [(SysButton, usize)] {
        match self {
            WinKeys::CloseOnly => &[(SysButton::Close, 2)],

            WinKeys::MinAndClose => &[(SysButton::Minimize, 1), (SysButton::Close, 2)],
            WinKeys::MinMaxClose => &[
                (SysButton::Minimize, 0),
                (SysButton::Maximize, 1),
                (SysButton::Close, 2),
            ],
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct WinOut {
    pub(crate) panel: Option<egui::Rect>,
    pub(crate) minimize: bool,
    pub(crate) maximize: bool,
    pub(crate) close: bool,
}

pub(crate) fn edge_at(rect: egui::Rect, p: egui::Pos2, band: f32) -> WinEdge {
    let left = p.x >= rect.min.x - band && p.x <= rect.min.x + band;
    let right = p.x >= rect.max.x - band && p.x <= rect.max.x + band;
    let bottom = p.y >= rect.max.y - band && p.y <= rect.max.y + band;

    let in_x = p.x >= rect.min.x - band && p.x <= rect.max.x + band;
    let in_y = p.y >= rect.min.y - band && p.y <= rect.max.y + band;

    if p.y < rect.min.y + WIN_TITLE_H {
        return WinEdge::None;
    }
    match (left && in_y, right && in_y, bottom && in_x) {
        (true, _, true) => WinEdge::BottomLeft,
        (_, true, true) => WinEdge::BottomRight,
        (true, _, _) => WinEdge::Left,
        (_, true, _) => WinEdge::Right,
        (_, _, true) => WinEdge::Bottom,
        _ => WinEdge::None,
    }
}

fn edge_target(v: f32, lo: f32, hi: f32, band: f32) -> f32 {
    if v <= lo + band {
        lo
    } else if v >= hi - band {
        hi
    } else {
        v.round()
    }
}

fn press_anchor(ui: &egui::Ui) -> Option<egui::Pos2> {
    ui.input(|i| i.pointer.press_origin().or_else(|| i.pointer.latest_pos()))
}

impl Win {
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
            drag_pin: None,
            open_guard: false,
            grip_hot: WinEdge::None,
            grip_down: false,
            handle_hot: false,
            rs: WinEdge::None,
            edge_guard: egui::Rect::NOTHING,
            rs_fix: egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
            rs_off: egui::Vec2::ZERO,
        }
    }

    pub(crate) fn rect(&self) -> egui::Rect {
        egui::Rect::from_min_size(self.pos, self.size)
    }

    pub(crate) fn title_band(&self) -> egui::Rect {
        free_band(self.rect(), 0, WIN_TITLE_ROWS)
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

    pub(crate) fn begin_resize(&mut self, p: egui::Pos2) {
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

        self.rs_off = egui::vec2(
            if edge.left() {
                rect.min.x - p.x
            } else if edge.right() {
                rect.max.x - p.x
            } else {
                0.0
            },
            if edge.bottom() { rect.max.y - p.y } else { 0.0 },
        );
        self.drag = None;
        self.drag_pin = None;
    }

    pub(crate) fn end_sessions(&mut self) {
        self.drag = None;
        self.drag_pin = None;
        self.rs = WinEdge::None;
    }

    #[allow(dead_code)]
    pub(crate) fn set_dragging(&mut self, on: bool) {
        self.drag = if on { Some(egui::Vec2::ZERO) } else { None };
    }

    pub(crate) fn guard_open_click(&mut self) {
        self.open_guard = true;
        self.end_sessions();
    }

    pub(crate) fn clamp_into(&mut self, bounds: egui::Rect) {
        self.size.x = clamp_soft(
            self.size.x,
            self.min_size.x.min(bounds.width()),
            bounds.width(),
        );
        self.size.y = clamp_soft(
            self.size.y,
            self.min_size.y.min(bounds.height()),
            bounds.height(),
        );
        self.pos.x = clamp_soft(self.pos.x, bounds.min.x, bounds.max.x - self.size.x);
        self.pos.y = clamp_soft(self.pos.y, bounds.min.y, bounds.max.y - self.size.y);
    }

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

    pub(crate) fn begin(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        bounds: egui::Rect,
        title_text: &str,
        keys: WinKeys,
        busy: bool,
    ) -> WinOut {
        self.update_geometry(app, ui, bounds);
        let rect = self.rect();

        let painter = ui.painter().clone();

        painter.rect_filled(rect, egui::CornerRadius::ZERO, theme().work_bg);
        interact_rect(ui, rect, egui::Sense::click_and_drag());

        self.grips(ui);

        let title = self.title_band();
        let keys_rects = sys_button_rects(title);

        let mut handle_end = title.max.x;
        for &(_, slot) in keys.shown() {
            handle_end = handle_end.min(keys_rects[slot].min.x);
        }
        let handle = egui::Rect::from_min_max(
            title.min,
            egui::pos2(handle_end.max(title.min.x), title.max.y),
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

        if let Some(p) = self.drag_title(ui, handle, busy, app.pointer.live) {
            app.pointer.pinned = Some(p);
        }

        WinOut {
            panel: Some(rect),
            minimize: hit == Some(SysButton::Minimize),
            maximize: hit == Some(SysButton::Maximize),
            close: hit == Some(SysButton::Close),
        }
    }

    fn update_geometry(&mut self, app: &mut StockApp, ui: &mut egui::Ui, bounds: egui::Rect) {
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
            .live
            .or_else(|| ui.input(|i| i.pointer.latest_pos()));
        if let Some(grab) = self.drag {
            let step = match (pointer, self.drag_pin) {
                (Some(p), Some(last)) => p - last,
                (Some(p), None) => p - (self.pos + grab),
                (None, _) => egui::Vec2::ZERO,
            } * WINDOW_DRAG_SPEED;
            let want = egui::pos2(self.pos.x + step.x, self.pos.y + step.y);

            let at = egui::pos2(
                clamp_soft(want.x, bounds.min.x, bounds.max.x - self.size.x),
                clamp_soft(want.y, bounds.min.y, bounds.max.y - self.size.y),
            );
            self.pos = egui::pos2(at.x.round(), at.y.round());
        }
        if self.rs.any() {
            if let Some(p) = pointer {
                self.stretch(p, bounds);
            }
        }

        self.clamp_into(bounds);

        if self.drag.is_some() {
            if app.pointer.system {
                self.drag_pin = pointer;
            } else if let Some(grab) = self.drag {
                let pin = self.pos + grab;
                self.drag_pin = Some(pin);
                app.pointer.pinned = Some(pin);
            }
        }
    }

    pub(crate) fn end(&mut self, ui: &mut egui::Ui, busy: bool) {
        if !self.maximized {
            self.resize_grips(ui, busy);
        }
        paint_shell_border(&ui.painter().clone(), self.rect());
    }

    fn drag_title(
        &mut self,
        ui: &mut egui::Ui,
        handle: egui::Rect,
        busy: bool,
        live: Option<egui::Pos2>,
    ) -> Option<egui::Pos2> {
        let resp = ui.interact(
            handle,
            egui::Id::new(("rt_win_drag", self.id)),
            egui::Sense::click_and_drag(),
        );

        self.handle_hot = resp.hovered();
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
        }
        if !resp.is_pointer_button_down_on()
            || busy
            || self.open_guard
            || self.drag.is_some()
            || self.rs.any()
        {
            return None;
        }
        if !ui.input(|i| i.pointer.primary_down()) {
            return None;
        }

        let p = live.or_else(|| press_anchor(ui))?;

        if self.resizable && edge_at(self.rect(), p, EDGE_BAND).any() {
            return None;
        }

        self.drag = Some(p - self.pos);

        self.drag_pin = Some(p);
        self.rs = WinEdge::None;
        Some(p)
    }

    fn grips(&mut self, ui: &mut egui::Ui) {
        self.grip_hot = WinEdge::None;
        self.grip_down = false;
        if !self.resizable {
            return;
        }
        let rect = self.rect();
        let b = EDGE_BAND;
        let pointer = ui.input(|i| i.pointer.hover_pos());
        let band = |x0: f32, y0: f32, x1: f32, y1: f32| {
            egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1))
        };

        let bands = [
            (
                band(
                    rect.min.x - b,
                    rect.max.y - b,
                    rect.min.x + b,
                    rect.max.y + b,
                ),
                WinEdge::BottomLeft,
            ),
            (
                band(
                    rect.max.x - b,
                    rect.max.y - b,
                    rect.max.x + b,
                    rect.max.y + b,
                ),
                WinEdge::BottomRight,
            ),
            (
                band(
                    rect.min.x - b,
                    rect.min.y + WIN_TITLE_H,
                    rect.min.x + b,
                    rect.max.y - b,
                ),
                WinEdge::Left,
            ),
            (
                band(
                    rect.max.x - b,
                    rect.min.y + WIN_TITLE_H,
                    rect.max.x + b,
                    rect.max.y - b,
                ),
                WinEdge::Right,
            ),
            (
                band(
                    rect.min.x + b,
                    rect.max.y - b,
                    rect.max.x - b,
                    rect.max.y + b,
                ),
                WinEdge::Bottom,
            ),
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

        self.grip_hot = match (hot, pointer) {
            (WinEdge::None, _) => WinEdge::None,
            (_, Some(p)) if self.edge_guard.contains(p) => WinEdge::None,
            (e, Some(p)) if edge_at(rect, p, b) == e => e,
            _ => WinEdge::None,
        };
        self.grip_down = down;
    }

    fn resize_grips(&mut self, ui: &mut egui::Ui, busy: bool) {
        if !self.resizable {
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

    fn stretch(&mut self, p: egui::Pos2, bounds: egui::Rect) {
        let fix = self.rs_fix;
        let min = self.min_size;
        let mut x0 = fix.min.x;
        let mut x1 = fix.max.x;
        let y0 = fix.min.y;
        let mut y1 = fix.max.y;
        if self.rs.left() {
            let want = edge_target(p.x + self.rs_off.x, bounds.min.x, bounds.max.x, EDGE_BAND);
            x0 = clamp_soft(want, bounds.min.x, (x1 - min.x).max(bounds.min.x));
            self.rs_off.x = x0 - p.x;
        } else if self.rs.right() {
            let want = edge_target(p.x + self.rs_off.x, bounds.min.x, bounds.max.x, EDGE_BAND);
            x1 = clamp_soft(want, (x0 + min.x).min(bounds.max.x), bounds.max.x);
            self.rs_off.x = x1 - p.x;
        }
        if self.rs.bottom() {
            let want = edge_target(p.y + self.rs_off.y, bounds.min.y, bounds.max.y, EDGE_BAND);
            y1 = clamp_soft(want, (y0 + min.y).min(bounds.max.y), bounds.max.y);
            self.rs_off.y = y1 - p.y;
        }
        self.pos = egui::pos2(x0, y0);
        self.size = egui::vec2((x1 - x0).max(LINE_W), (y1 - y0).max(LINE_W));
    }
}

#[allow(clippy::too_many_arguments)]
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

    let rects = sys_button_rects(title);
    let mut keys_left = title.max.x;
    for &(_, slot) in keys.shown() {
        keys_left = keys_left.min(rects[slot].min.x);
    }
    let text_slot = title_text_slot(app, title, text);
    let text_slot = egui::Rect::from_min_max(
        text_slot.min,
        egui::pos2(text_slot.max.x.min(keys_left), text_slot.max.y),
    );

    if (text_slot.max.x - keys_left).abs() > 0.5 {
        paint_title_text_edge(painter, text_slot);
    }

    app.text(
        painter,
        egui::pos2(text_left_x(title.min.x, title.min.x), title.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        theme().text,
    );

    let mut hit = None;
    for &(kind, slot) in keys.shown() {
        let rect = rects[slot];

        let resp = ui.interact(
            rect,
            egui::Id::new(("rt_win_key", id, kind)),
            egui::Sense::click(),
        );
        let hot = resp.hovered();
        let (face, symbol) = match kind {
            SysButton::Close => (close_key_face(hot), close_key_symbol(hot)),
            SysButton::Minimize | SysButton::Maximize => (
                if hot {
                    theme().btn_hover
                } else {
                    theme().btn_face
                },
                theme().text,
            ),
        };
        painter.rect_filled(rect, egui::CornerRadius::ZERO, face);
        border_hard(painter, rect, theme().ctrl_border);

        let restore = matches!(keys, WinKeys::MinAndClose) && kind == SysButton::Minimize;
        if restore {
            paint_restore_symbol(
                painter,
                egui::pos2(rect.min.x + CELL_CENTER, rect.min.y + CELL_CENTER),
                symbol,
            );
        } else {
            kind.paint_symbol_max(painter, rect, symbol, face, maximized);
        }

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
