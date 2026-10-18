use crate::app::*;
use crate::grid::*;
use crate::ui::chart::*;
use crate::ui::console::*;
use crate::ui::floatwin::*;
use crate::ui::icon::*;
use crate::ui::panel::*;
use crate::ui::titlebar::*;
use eframe::egui;

pub(crate) const MAX_CHART_WINS: usize = 8;

pub(crate) const CASCADE_STEP: f32 = grid_line(WIN_TITLE_ROWS);

pub(crate) const NEW_WIN_W: f32 = cells(60);
pub(crate) const NEW_WIN_H: f32 = cells(40);

pub(crate) const MINI_W: f32 = cells(10);

pub(crate) const WIN_CHART_TITLE: &str = "图表";

pub(crate) const SRC_TITLE: &str = "指标代码";

const _: () = assert!((NEW_WIN_W as i32 - 1) % 9 == 0 && (NEW_WIN_H as i32 - 1) % 9 == 0);
const _: () = assert!((MINI_W as i32 - 1) % 9 == 0);
const _: () = assert!(MINI_W > 2.0 * WIN_CLOSE_W);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum WinKind {
    Chart,
    Panel,

    Source,

    Console,
}

pub(crate) struct ManagedWin {
    pub(crate) kind: WinKind,

    pub(crate) title: String,
    pub(crate) minimized: bool,

    pub(crate) restore: (egui::Pos2, egui::Vec2),

    pub(crate) pane: usize,

    pub(crate) pinned: bool,

    pin_seq: u64,

    mini_seq: u64,

    mini_w: f32,

    pub(crate) w: Win,
}

impl ManagedWin {
    pub(crate) fn rect(&self) -> egui::Rect {
        self.w.rect()
    }

    fn edge_guard(&self) -> egui::Rect {
        match self.kind {
            WinKind::Source => crate::ui::editor::scrollbar_track(self.w.rect()),
            WinKind::Chart | WinKind::Panel | WinKind::Console => egui::Rect::NOTHING,
        }
    }

    pub(crate) fn body(rect: egui::Rect) -> egui::Rect {
        free_tail(rect, WIN_TITLE_ROWS)
    }
}

pub(crate) struct WinMgr {
    pub(crate) wins: Vec<ManagedWin>,

    pub(crate) plots: Vec<egui::Rect>,

    pending: Option<u64>,

    pub(crate) work: egui::Rect,

    placed: bool,
    next_id: u64,
    next_mini: u64,
    next_pin: u64,
    focused_id: Option<u64>,
    source_needs_placement: bool,
    #[cfg(test)]
    focused_pane_override: Option<usize>,
}

impl WinMgr {
    pub(crate) fn new() -> Self {
        Self {
            wins: Vec::new(),
            plots: Vec::new(),
            pending: None,
            work: egui::Rect::NOTHING,
            placed: false,
            next_id: 1,
            next_mini: 0,
            next_pin: 0,
            focused_id: None,
            source_needs_placement: false,
            #[cfg(test)]
            focused_pane_override: None,
        }
    }

    pub(crate) fn take(&mut self) -> Self {
        std::mem::replace(self, Self::new())
    }

    pub(crate) fn with_default_windows() -> Self {
        Self::with_initial_chart(false)
    }

    #[cfg(test)]
    pub(crate) fn with_default_windows_and_chart() -> Self {
        Self::with_initial_chart(true)
    }

    fn with_initial_chart(chart: bool) -> Self {
        let mut mgr = Self::new();
        let panel = mgr.add(
            WinKind::Panel,
            PANEL_TITLE,
            PANE_CANDLE,
            egui::vec2(PANEL_W, PANEL_CHILD_H),
            egui::vec2(PANEL_W, PANEL_CHILD_H),
        );
        mgr.toggle_pin(panel);
        if chart {
            mgr.add(
                WinKind::Chart,
                WIN_CHART_TITLE,
                PANE_CANDLE,
                egui::vec2(NEW_WIN_W, NEW_WIN_H),
                Self::chart_min_size(),
            );
        }
        mgr
    }

    pub(crate) fn add(
        &mut self,
        kind: WinKind,
        title: &str,
        pane: usize,
        size: egui::Vec2,
        min_size: egui::Vec2,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let mut w = Win::new(id, 0.0, 0.0, size.x, size.y);
        w.open = true;
        w.min_size = min_size;
        w.resizable = true;
        self.wins.push(ManagedWin {
            kind,
            title: title.to_owned(),
            minimized: false,
            restore: (egui::Pos2::ZERO, size),
            pane,
            pinned: false,
            pin_seq: 0,
            mini_seq: 0,
            mini_w: MINI_W,
            w,
        });
        self.focused_id = Some(id);
        self.wins.len() - 1
    }

    pub(crate) fn chart_count(&self) -> usize {
        self.wins
            .iter()
            .filter(|w| w.kind == WinKind::Chart && w.w.open)
            .count()
    }

    pub(crate) fn source_open(&self) -> bool {
        self.wins
            .iter()
            .any(|w| w.kind == WinKind::Source && w.w.open)
    }

    fn source_index(&self) -> Option<usize> {
        self.wins.iter().position(|w| w.kind == WinKind::Source)
    }

    pub(crate) fn show_source(&mut self, work: egui::Rect) -> usize {
        if let Some(i) = self.source_index() {
            self.wins[i].w.open = true;
            self.wins[i].minimized = false;
            if !self.wins[i].pinned {
                self.toggle_pin(i);
            }
            self.activate(i);
            return i;
        }
        let i = self.add(
            WinKind::Source,
            SRC_TITLE,
            0,
            egui::vec2(SRC_W, SRC_H),
            egui::vec2(SRC_W, SRC_H),
        );
        self.source_needs_placement = !work.is_positive();
        self.wins[i].w.pos = self.source_default_position(work);

        if work.is_positive() {
            self.wins[i].w.clamp_into(work);
        }
        self.wins[i].restore = (self.wins[i].w.pos, self.wins[i].w.size);
        self.toggle_pin(i);
        i
    }

    pub(crate) fn hide_source(&mut self) {
        if let Some(i) = self.source_index() {
            self.wins[i].w.open = false;
            self.wins[i].minimized = false;
            self.wins[i].w.end_sessions();
        }
    }

    fn source_default_position(&self, work: egui::Rect) -> egui::Pos2 {
        if !work.is_positive() {
            return egui::pos2(SRC_DEFAULT_X, SRC_DEFAULT_Y);
        }
        let right_limit = self
            .wins
            .iter()
            .find(|w| w.kind == WinKind::Panel && w.pinned && w.w.open)
            .map_or(work.max.x, |w| w.w.pos.x);
        let x_max = (right_limit - SRC_W).max(work.min.x);
        let y_max = (work.max.y - SRC_H).max(work.min.y);
        egui::pos2(
            SRC_DEFAULT_X.clamp(work.min.x, x_max),
            SRC_DEFAULT_Y.clamp(work.min.y, y_max),
        )
    }

    pub(crate) fn console_open(&self) -> bool {
        self.wins
            .iter()
            .any(|w| w.kind == WinKind::Console && w.w.open)
    }

    pub(crate) fn show_console(&mut self, work: egui::Rect) -> usize {
        if let Some(i) = self.wins.iter().position(|w| w.kind == WinKind::Console) {
            self.wins[i].w.open = true;
            self.wins[i].minimized = false;
            if !self.wins[i].pinned {
                self.toggle_pin(i);
            }
            self.wins[i].w.guard_open_click();
            self.activate(i);
            return i;
        }
        let size = egui::vec2(CONSOLE_W.min(work.width()), CONSOLE_H.min(work.height()));
        let i = self.add(
            WinKind::Console,
            CONSOLE_TITLE,
            0,
            size,
            egui::vec2(CONSOLE_MIN_W, CONSOLE_MIN_H),
        );
        if work.is_positive() {
            self.wins[i].w.pos = egui::pos2(
                (work.center().x - size.x / 2.0).round(),
                (work.center().y - size.y / 2.0).round(),
            );
            self.wins[i].w.clamp_into(work);
        }
        self.wins[i].w.guard_open_click();
        self.wins[i].restore = (self.wins[i].w.pos, self.wins[i].w.size);
        self.toggle_pin(i);
        i
    }

    pub(crate) fn hide_console(&mut self) {
        if let Some(i) = self.wins.iter().position(|w| w.kind == WinKind::Console) {
            self.wins[i].w.open = false;
            self.wins[i].minimized = false;
            self.wins[i].w.end_sessions();
        }
    }

    #[allow(dead_code)]
    pub(crate) fn chart_panes(&self) -> Vec<usize> {
        self.wins
            .iter()
            .filter(|w| w.kind == WinKind::Chart)
            .map(|w| w.pane)
            .collect()
    }

    pub(crate) fn free_chart_pane(&self) -> Option<usize> {
        self.wins
            .iter()
            .find(|w| w.kind == WinKind::Chart && !w.w.open)
            .map(|w| w.pane)
    }

    pub(crate) fn activate(&mut self, i: usize) {
        if i + 1 >= self.wins.len() {
            return;
        }
        let w = self.wins.remove(i);
        self.wins.push(w);
    }

    pub(crate) fn activate_id(&mut self, id: u64) {
        self.pending = Some(id);
    }

    fn place_defaults(&mut self, work: egui::Rect) {
        if self.placed {
            return;
        }
        self.placed = true;
        let docked = StockApp::child_areas(work);
        let mut chart_done = false;
        for w in self.wins.iter_mut() {
            let r = match w.kind {
                WinKind::Chart if !chart_done => {
                    chart_done = true;
                    docked[0]
                }
                WinKind::Panel => docked[1],

                WinKind::Chart | WinKind::Source | WinKind::Console => continue,
            };
            w.w.pos = r.min;
            w.w.size = r.size();
            w.restore = (r.min, r.size());
        }
    }

    fn clamp_all(&mut self, work: egui::Rect) {
        for w in self.wins.iter_mut() {
            if w.w.open {
                w.w.clamp_into(work);
            }
        }
    }

    pub(crate) fn mini_width(app: &mut StockApp, title: &str) -> f32 {
        let need = app.text_width(title) + 2.0 * TEXT_PAD_X + 3.0 * SYS_BTN_W - LINE_W;
        cells(((need - LINE_W) / PITCH).ceil() as i32).max(MINI_W)
    }

    fn mini_rect(&self, i: usize, k: usize) -> egui::Rect {
        let work = self.work;
        let w = self.wins[i].mini_w.min(work.width());
        let h = WIN_TITLE_H.min(work.height());
        egui::Rect::from_min_size(
            egui::pos2(
                work.min.x,
                work.min.y + grid_line(k as i32 * WIN_TITLE_ROWS),
            ),
            egui::vec2(w, h),
        )
    }

    fn mini_index(&self, i: usize) -> usize {
        let seq = self.wins[i].mini_seq;
        self.wins
            .iter()
            .filter(|w| w.w.open && w.minimized && w.mini_seq < seq)
            .count()
    }

    pub(crate) fn rect_of(&self, i: usize) -> egui::Rect {
        if self.wins[i].minimized {
            let k = self.mini_index(i);
            self.mini_rect(i, k)
        } else {
            self.wins[i].rect()
        }
    }

    fn draw_order(&self) -> Vec<usize> {
        let live = |w: &ManagedWin, mini: bool| w.w.open && w.minimized == mini;
        let mut out: Vec<usize> = (0..self.wins.len())
            .filter(|&i| live(&self.wins[i], false) && !self.wins[i].pinned)
            .collect();
        let mut pinned: Vec<usize> = (0..self.wins.len())
            .filter(|&i| live(&self.wins[i], false) && self.wins[i].pinned)
            .collect();
        pinned.sort_by_key(|&i| self.wins[i].pin_seq);
        out.extend(pinned);
        out.extend((0..self.wins.len()).filter(|&i| live(&self.wins[i], true)));
        out
    }

    pub(crate) fn toggle_pin(&mut self, i: usize) {
        self.wins[i].pinned = !self.wins[i].pinned;
        if self.wins[i].pinned {
            self.wins[i].pin_seq = self.next_pin;
            self.next_pin += 1;
        } else {
            self.wins[i].pin_seq = 0;
        }
        self.wins[i].w.end_sessions();
    }

    #[cfg(test)]
    pub(crate) fn order_ids(&self) -> Vec<u64> {
        self.draw_order()
            .into_iter()
            .map(|i| self.wins[i].w.id)
            .collect()
    }

    fn top_win_at(&self, p: egui::Pos2) -> Option<usize> {
        self.draw_order()
            .into_iter()
            .rev()
            .find(|&i| self.rect_of(i).contains(p))
    }

    #[cfg(test)]
    pub(crate) fn live_index(&self) -> Option<usize> {
        if let Some(id) = self.focused_id
            && let Some(i) = self
                .wins
                .iter()
                .position(|w| w.w.id == id && w.w.open && !w.minimized)
        {
            return Some(i);
        }
        self.draw_order()
            .into_iter()
            .rev()
            .find(|&i| !self.wins[i].minimized)
    }

    pub(crate) fn focused_index(&self) -> Option<usize> {
        let id = self.focused_id?;
        self.wins.iter().position(|w| w.w.id == id && w.w.open)
    }

    pub(crate) fn focused_pane(&self) -> Option<usize> {
        #[cfg(test)]
        if let Some(pane) = self.focused_pane_override {
            return Some(pane);
        }
        let i = self.focused_index()?;
        (self.wins[i].kind == WinKind::Chart).then_some(self.wins[i].pane)
    }

    #[cfg(test)]
    pub(crate) fn focus_pane(&mut self, pane: usize) -> bool {
        self.focused_pane_override = Some(pane);
        true
    }

    pub(crate) fn top_chart_pane(&self) -> Option<usize> {
        self.draw_order()
            .into_iter()
            .rev()
            .find(|&i| self.wins[i].kind == WinKind::Chart && self.wins[i].w.open)
            .map(|i| self.wins[i].pane)
    }

    fn press_owner_at(&self, p: egui::Pos2) -> Option<usize> {
        match (self.grip_owner_at(p), self.top_win_at(p)) {
            (Some(g), Some(t)) if g != t => {
                if self.z_of(g) > self.z_of(t) {
                    Some(g)
                } else {
                    Some(t)
                }
            }
            (Some(g), _) => Some(g),
            (None, t) => t,
        }
    }

    fn z_of(&self, i: usize) -> usize {
        self.draw_order().iter().position(|&k| k == i).unwrap_or(0)
    }

    fn grip_owner_at(&self, p: egui::Pos2) -> Option<usize> {
        self.draw_order().into_iter().rev().find(|&i| {
            let w = &self.wins[i];
            w.w.open
                && !w.minimized
                && !w.w.maximized
                && w.w.resizable
                && edge_at(w.rect(), p, EDGE_BAND).any()
        })
    }

    pub(crate) fn add_chart(&mut self, work: egui::Rect, pane: usize) -> usize {
        let size = egui::vec2(NEW_WIN_W.min(work.width()), NEW_WIN_H.min(work.height()));
        let mut pos = work.min;
        for k in 0..=self.wins.len() {
            let step = CASCADE_STEP * k as f32;
            let want = egui::pos2(work.min.x + step, work.min.y + step);
            pos = egui::pos2(
                clamp_soft(want.x, work.min.x, work.max.x - size.x),
                clamp_soft(want.y, work.min.y, work.max.y - size.y),
            );
            if !self.occupied(pos) {
                break;
            }
        }
        let i = self.add(
            WinKind::Chart,
            WIN_CHART_TITLE,
            pane,
            size,
            Self::chart_min_size(),
        );
        self.wins[i].w.pos = pos;
        self.wins[i].restore = (pos, size);

        i
    }

    pub(crate) fn chart_min_size() -> egui::Vec2 {
        egui::vec2(PANE_MIN_W, PANE_H_MIN)
    }

    fn occupied(&self, p: egui::Pos2) -> bool {
        self.occupied_except(p, usize::MAX)
    }

    fn occupied_except(&self, p: egui::Pos2, skip: usize) -> bool {
        self.wins
            .iter()
            .enumerate()
            .any(|(i, w)| i != skip && w.w.open && !w.minimized && (w.w.pos - p).length_sq() < 0.5)
    }

    pub(crate) fn drag_active(&self) -> bool {
        self.wins.iter().any(|w| w.w.dragging() || w.w.resizing())
    }

    pub(crate) fn session_kind(&self) -> Option<CursorKind> {
        if let Some(edge) = self.wins.iter().find_map(|w| w.w.resize_edge()) {
            return Some(edge.cursor_kind());
        }
        if self.wins.iter().any(|w| w.w.dragging()) {
            return Some(CursorKind::Move);
        }
        None
    }

    pub(crate) fn hover_kind(&self) -> Option<CursorKind> {
        self.draw_order().into_iter().rev().find_map(|i| {
            let w = &self.wins[i];
            if w.w.open { w.w.hover_kind() } else { None }
        })
    }

    pub(crate) fn dock(&mut self, i: usize, r: egui::Rect) {
        self.wins[i].w.pos = r.min;
        self.wins[i].w.size = r.size();
        self.wins[i].restore = (r.min, r.size());
        self.wins[i].w.open = true;
        self.wins[i].minimized = false;
        self.wins[i].w.end_sessions();
    }

    fn close_win(&mut self, app: &mut StockApp, i: usize) {
        let closed_id = self.wins[i].w.id;
        let pane = self.wins[i].pane;
        let title = self.wins[i].title.clone();
        self.wins[i].w.open = false;
        self.wins[i].w.end_sessions();
        self.wins[i].minimized = false;
        if self.wins[i].kind == WinKind::Chart && app.pane_menu == Some(pane) {
            app.pane_menu = None;
        }
        if self.focused_id == Some(closed_id) {
            self.focused_id = self
                .draw_order()
                .into_iter()
                .rev()
                .map(|i| self.wins[i].w.id)
                .next();
        }
        app.status_msg = format!("{title}：已关闭");
    }

    pub(crate) fn minimize(&mut self, i: usize, mini_w: f32) {
        self.wins[i].restore = (self.wins[i].w.pos, self.wins[i].w.size);
        self.wins[i].mini_w = mini_w;
        self.wins[i].minimized = true;
        self.next_mini += 1;
        self.wins[i].mini_seq = self.next_mini;
        self.wins[i].w.end_sessions();
    }

    pub(crate) fn restore(&mut self, i: usize) {
        let (pos, size) = self.wins[i].restore;
        self.wins[i].w.pos = pos;
        self.wins[i].w.size = size;
        self.wins[i].minimized = false;
    }

    fn toggle_minimize(&mut self, app: &mut StockApp, i: usize) {
        let id = self.wins[i].w.id;
        if self.wins[i].minimized {
            self.restore(i);
        } else {
            let title = self.wins[i].title.clone();
            let w = Self::mini_width(app, &title);
            self.minimize(i, w);
        }
        self.activate_id(id);
    }

    pub(crate) fn plan(&mut self, app: &mut StockApp, ui: &mut egui::Ui, work: egui::Rect) {
        self.work = work;
        if self.source_needs_placement
            && work.is_positive()
            && let Some(i) = self.source_index()
        {
            self.wins[i].w.pos = self.source_default_position(work);
            self.wins[i].restore.0 = self.wins[i].w.pos;
            self.source_needs_placement = false;
        }
        app.debug_windows.windows = self
            .wins
            .iter()
            .filter(|w| w.w.open)
            .map(|w| (w.title.clone(), w.minimized, w.rect()))
            .collect();
        app.debug_windows.chart_count = self.chart_count();
        app.debug_windows.source_open = self.source_open();
        app.debug_windows.console_open = self.console_open();
        self.place_defaults(work);

        for i in 0..self.wins.len() {
            let guard = self.wins[i].edge_guard();
            self.wins[i].w.edge_guard = guard;
        }

        if let Some(id) = self.pending.take() {
            if let Some(i) = self.wins.iter().position(|w| w.w.id == id) {
                self.activate(i);
            }
        }

        if !ui.input(|i| i.pointer.primary_down()) {
            for w in self.wins.iter_mut() {
                w.w.end_sessions();
            }
        }
        self.clamp_all(work);

        if !self.drag_active()
            && let Some(p) = ui.input(|i| i.pointer.latest_pos())
            && work.contains(p)
            && !app.floating_at(p, ui.max_rect().max.y)
            && let Some(i) = self.top_win_at(p)
        {
            let id = self.wins[i].w.id;
            if self.focused_id != Some(id) {
                self.focused_id = Some(id);
                self.activate(i);
            }
        }

        if ui.input(|i| i.pointer.primary_pressed()) {
            if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
                if !app.floating_at(p, ui.max_rect().max.y) {
                    if let Some(i) = self.press_owner_at(p) {
                        self.activate(i);
                    }
                }
            }

            if let Some(p) = ui.input(|i| i.pointer.press_origin()) {
                if !app.floating_at(p, ui.max_rect().max.y) {
                    if let Some(i) = self.press_owner_at(p) {
                        if edge_at(self.wins[i].rect(), p, EDGE_BAND).any()
                            && !self.wins[i].w.edge_guard.contains(p)
                        {
                            self.wins[i].w.begin_resize(p);
                        }
                    }
                }
            }
        }

        let panes = app.panes.len();
        self.plots.clear();
        self.plots.resize(panes, egui::Rect::NOTHING);
        for i in self.draw_order() {
            if !self.wins[i].w.open || self.wins[i].kind != WinKind::Chart || self.wins[i].minimized
            {
                continue;
            }
            let pane = self.wins[i].pane;
            let body = ManagedWin::body(self.rect_of(i));
            let layout = app.panes[pane].indicator.layout();
            let (_, _, main, _) = chart_rows(body, layout);
            self.plots[pane] = frame_rects(main).0;
        }
    }

    pub(crate) fn draw(&mut self, app: &mut StockApp, ui: &mut egui::Ui, work: egui::Rect) {
        self.work = work;
        for i in self.draw_order() {
            if !self.wins[i].w.open {
                continue;
            }

            let busy = self
                .wins
                .iter()
                .enumerate()
                .any(|(k, w)| k != i && (w.w.dragging() || w.w.resizing()));

            let focused = self.focused_id == Some(self.wins[i].w.id);
            app.win_silent = !focused;
            ui.scope(|ui| {
                if !focused {
                    ui.disable();
                }
                let body = self.begin_win(app, ui, i, busy);
                if let Some(body) = body {
                    self.draw_content(app, ui, i, body);
                }
                self.end_win(ui, i, busy);
            });
        }
        app.win_silent = false;
    }

    fn draw_content(&mut self, app: &mut StockApp, ui: &mut egui::Ui, i: usize, body: egui::Rect) {
        match self.wins[i].kind {
            WinKind::Panel => app.ui_panel(ui, body),
            WinKind::Chart => {
                let pane = self.wins[i].pane;
                let layout = app.panes[pane].indicator.layout();
                let (_, _, main, _) = chart_rows(body, layout);
                self.plots[pane] = frame_rects(main).0;
                if app.win_silent {
                    app.panes[pane].view.crosshair = None;
                    app.panes[pane].view.pan = None;
                }
                app.ui_chart(ui, pane, body);
            }
            WinKind::Source => {
                self.wins[i].title = app.source_title();

                let rect = self.wins[i].w.rect();
                let cmd = app.ui_source_body(ui, rect);
                app.run_editor_cmd(cmd);
            }
            WinKind::Console => {
                let mut console = std::mem::take(&mut app.console);
                console.ui(app, ui, body);
                app.console = console;
            }
        }
    }

    fn begin_win(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        i: usize,
        busy: bool,
    ) -> Option<egui::Rect> {
        let name = self.wins[i].title.clone();
        if self.wins[i].minimized {
            let rect = self.rect_of(i);
            let title = free_band(rect, 0, WIN_TITLE_ROWS);
            let painter = ui.painter().clone();
            interact_rect(ui, rect, egui::Sense::click_and_drag());
            let hit = paint_title_bar(
                app,
                ui,
                &painter,
                title,
                self.wins[i].w.id,
                &name,
                WinKeys::MinAndClose,
                false,
                self.wins[i].pinned,
                true,
            );
            match hit {
                Some(SysButton::Pin) => self.toggle_pin(i),
                Some(SysButton::Minimize) => self.toggle_minimize(app, i),
                Some(SysButton::Close) => self.close_win(app, i),

                Some(SysButton::Maximize) => {}
                None => {}
            }
            return None;
        }

        let work = self.work;
        let pinned = self.wins[i].pinned;
        let out = self.wins[i].w.begin(
            app,
            ui,
            work,
            &name,
            WinKeys::MinMaxClose,
            busy,
            pinned,
            false,
        );
        if out.pin {
            self.toggle_pin(i);
        } else if out.close {
            self.close_win(app, i);
        } else if out.maximize {
            self.wins[i].w.toggle_maximize(work);
        } else if out.minimize {
            self.toggle_minimize(app, i);
        }

        out.panel.map(ManagedWin::body)
    }

    fn end_win(&mut self, ui: &mut egui::Ui, i: usize, busy: bool) {
        if self.wins[i].minimized {
            let rect = self.rect_of(i);
            paint_shell_border(&ui.painter().clone(), rect);
        } else {
            self.wins[i].w.end(ui, busy);
        }
    }
}
