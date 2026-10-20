use crate::app::*;
use crate::grid::*;
use crate::theme::theme;
use crate::ui::button::ButtonContent;
use crate::ui::chart::*;
use crate::ui::colorpanel::{CP_H, CP_W};
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

pub(crate) const TASKBAR_GAP_W: f32 = cells(2);

pub(crate) const WIN_CHART_TITLE: &str = "图表";

pub(crate) const SRC_TITLE: &str = "指标代码";

pub(crate) const TASKBAR_MENU_TEXT: &str = "圆桌";

const _: () = assert!((NEW_WIN_W as i32 - 1) % 9 == 0 && (NEW_WIN_H as i32 - 1) % 9 == 0);
const _: () = assert!((MINI_W as i32 - 1) % 9 == 0);
const _: () = assert!(MINI_W > 2.0 * WIN_CLOSE_W);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum WinKind {
    Chart,
    Panel,

    Source,

    Console,

    Color,
}

pub(crate) struct ManagedWin {
    pub(crate) kind: WinKind,

    pub(crate) title: String,
    pub(crate) minimized: bool,

    pub(crate) restore: (egui::Pos2, egui::Vec2),

    pub(crate) pane: usize,

    pub(crate) pinned: bool,
    pub(crate) exclusive: bool,

    pin_seq: u64,

    mini_seq: u64,

    mini_w: f32,

    pub(crate) w: Win,
}

#[derive(Clone, Copy)]
struct TaskbarDrag {
    id: u64,
    grab: f32,
    start_x: f32,
    x: f32,
    moved: bool,
    page: usize,
}

impl ManagedWin {
    pub(crate) fn rect(&self) -> egui::Rect {
        self.w.rect()
    }

    fn edge_guard(&self) -> egui::Rect {
        match self.kind {
            WinKind::Source => crate::ui::editor::scrollbar_track(self.w.rect()),
            WinKind::Chart | WinKind::Panel | WinKind::Console | WinKind::Color => {
                egui::Rect::NOTHING
            }
        }
    }

    pub(crate) fn body(rect: egui::Rect) -> egui::Rect {
        free_tail(rect.shrink(WIN_FRAME), WIN_TITLE_ROWS)
    }
}

pub(crate) struct WinMgr {
    pub(crate) wins: Vec<ManagedWin>,

    pub(crate) plots: Vec<egui::Rect>,

    pending: Option<u64>,

    pub(crate) work: egui::Rect,

    pub(crate) taskbar: egui::Rect,

    taskbar_gap: egui::Rect,

    taskbar_items: egui::Rect,

    taskbar_pages: Vec<(usize, usize)>,

    taskbar_page: usize,

    taskbar_drag: Option<TaskbarDrag>,

    placed: bool,
    next_id: u64,
    next_mini: u64,
    next_pin: u64,
    focused_id: Option<u64>,
    pending_placement: Vec<u64>,
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
            taskbar: egui::Rect::NOTHING,
            taskbar_gap: egui::Rect::NOTHING,
            taskbar_items: egui::Rect::NOTHING,
            taskbar_pages: Vec::new(),
            taskbar_page: 0,
            taskbar_drag: None,
            placed: false,
            next_id: 1,
            next_mini: 0,
            next_pin: 0,
            focused_id: None,
            pending_placement: Vec::new(),
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
        let frame_size = egui::vec2(2.0 * WIN_FRAME, 2.0 * WIN_FRAME);
        let outer_size = size + frame_size;
        let outer_min = min_size + frame_size;

        let mut w = Win::new(id, 0.0, 0.0, outer_size.x, outer_size.y);
        w.open = true;
        w.min_size = outer_min;
        w.resizable = true;
        self.wins.push(ManagedWin {
            kind,
            title: title.to_owned(),
            minimized: false,
            restore: (egui::Pos2::ZERO, outer_size),
            pane,
            pinned: false,
            exclusive: false,
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

    pub(crate) fn show_source(&mut self, work: egui::Rect) -> Option<usize> {
        if let Some(i) = self.source_index() {
            let was_open = self.wins[i].w.open;
            self.wins[i].w.open = true;
            self.wins[i].minimized = false;
            if !self.wins[i].pinned {
                self.toggle_pin(i);
            }
            if !was_open && !self.place_default(i, work) {
                self.wins[i].w.open = false;
                return None;
            }
            self.activate(i);
            return Some(i);
        }
        let i = self.add(
            WinKind::Source,
            SRC_TITLE,
            0,
            egui::vec2(SRC_W, SRC_H),
            egui::vec2(SRC_W, SRC_H),
        );
        if !self.place_default(i, work) {
            self.wins[i].w.open = false;
            return None;
        }
        self.toggle_pin(i);
        Some(i)
    }

    pub(crate) fn hide_source(&mut self) {
        if let Some(i) = self.source_index() {
            self.wins[i].w.open = false;
            self.wins[i].minimized = false;
            self.wins[i].w.end_sessions();
        }
    }

    pub(crate) fn console_open(&self) -> bool {
        self.wins
            .iter()
            .any(|w| w.kind == WinKind::Console && w.w.open)
    }

    pub(crate) fn show_console(&mut self, work: egui::Rect) -> Option<usize> {
        if let Some(i) = self.wins.iter().position(|w| w.kind == WinKind::Console) {
            let was_open = self.wins[i].w.open;
            self.wins[i].w.open = true;
            self.wins[i].minimized = false;
            if !self.wins[i].pinned {
                self.toggle_pin(i);
            }
            if !was_open && !self.place_default(i, work) {
                self.wins[i].w.open = false;
                return None;
            }
            self.wins[i].w.guard_open_click();
            self.activate(i);
            return Some(i);
        }
        let size = egui::vec2(CONSOLE_W.min(work.width()), CONSOLE_H.min(work.height()));
        let i = self.add(
            WinKind::Console,
            CONSOLE_TITLE,
            0,
            size,
            egui::vec2(CONSOLE_MIN_W, CONSOLE_MIN_H),
        );
        if !self.place_default(i, work) {
            self.wins[i].w.open = false;
            return None;
        }
        self.wins[i].w.guard_open_click();
        self.toggle_pin(i);
        Some(i)
    }

    pub(crate) fn hide_console(&mut self) {
        if let Some(i) = self.wins.iter().position(|w| w.kind == WinKind::Console) {
            self.wins[i].w.open = false;
            self.wins[i].minimized = false;
            self.wins[i].w.end_sessions();
        }
    }

    pub(crate) fn show_color(&mut self, work: egui::Rect) -> Option<usize> {
        if let Some(i) = self.wins.iter().position(|w| w.kind == WinKind::Color) {
            let was_open = self.wins[i].w.open;
            self.wins[i].w.open = true;
            self.wins[i].minimized = false;
            if !self.wins[i].pinned {
                self.set_pin(i, true);
            }
            if !was_open && !self.place_default(i, work) {
                self.wins[i].w.open = false;
                return None;
            }
            self.wins[i].w.guard_open_click();
            self.activate(i);
            return Some(i);
        }
        let size = egui::vec2(CP_W, CP_H);
        let i = self.add(WinKind::Color, "颜色设置", 0, size, size);
        self.wins[i].w.resizable = false;
        if !self.place_default(i, work) {
            self.wins[i].w.open = false;
            return None;
        }
        self.wins[i].w.guard_open_click();
        self.set_pin(i, true);
        Some(i)
    }

    pub(crate) fn hide_color(&mut self) {
        if let Some(i) = self.wins.iter().position(|w| w.kind == WinKind::Color) {
            self.wins[i].w.open = false;
            self.wins[i].minimized = false;
            self.wins[i].w.end_sessions();
        }
    }

    pub(crate) fn color_index(&self) -> Option<usize> {
        self.wins
            .iter()
            .position(|w| w.kind == WinKind::Color && w.w.open)
    }

    pub(crate) fn color_rect(&self) -> Option<egui::Rect> {
        self.color_index().map(|i| self.rect_of(i))
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

    fn spawn_position_for(
        &self,
        work: egui::Rect,
        size: egui::Vec2,
        skip: Option<usize>,
    ) -> Option<egui::Pos2> {
        if !work.is_positive() {
            return None;
        }
        let occupied = |pos: egui::Pos2| {
            self.wins.iter().enumerate().any(|(i, w)| {
                Some(i) != skip && w.w.open && !w.minimized && (w.w.pos - pos).length_sq() < 0.5
            })
        };
        let centered = egui::pos2(
            (work.center().x - size.x / 2.0).round(),
            (work.center().y - size.y / 2.0).round(),
        );
        if !occupied(centered) {
            return Some(centered);
        }

        let mut y = work.min.y;
        loop {
            if y + size.y > work.max.y + 0.01 {
                return None;
            }
            let pos = egui::pos2(work.min.x, y);
            if !occupied(pos) {
                return Some(pos);
            }
            y += WIN_TITLE_H;
            if y + size.y > work.max.y + 0.01 {
                return None;
            }
        }
    }

    fn place_default(&mut self, i: usize, work: egui::Rect) -> bool {
        if !work.is_positive() {
            let id = self.wins[i].w.id;
            if !self.pending_placement.contains(&id) {
                self.pending_placement.push(id);
            }
            return true;
        }
        let size = self.wins[i].w.size;
        let Some(pos) = self.spawn_position_for(work, size, Some(i)) else {
            let id = self.wins[i].w.id;
            self.pending_placement.retain(|pending| *pending != id);
            return false;
        };
        let id = self.wins[i].w.id;
        self.pending_placement.retain(|pending| *pending != id);
        self.wins[i].w.pos = pos;
        self.wins[i].w.clamp_into(work);
        self.wins[i].restore = (self.wins[i].w.pos, self.wins[i].w.size);
        true
    }

    fn place_panel(&mut self, i: usize, work: egui::Rect) {
        if !work.is_positive() {
            let id = self.wins[i].w.id;
            if !self.pending_placement.contains(&id) {
                self.pending_placement.push(id);
            }
            return;
        }
        let size = self.wins[i].w.size;
        let pos = egui::pos2(work.max.x - size.x, work.max.y - size.y);
        self.wins[i].w.pos = pos;
        self.wins[i].w.clamp_into(work);
        self.wins[i].restore = (self.wins[i].w.pos, self.wins[i].w.size);
    }

    pub(crate) fn place_panel_at_default(&mut self, work: egui::Rect) {
        if let Some(i) = self.wins.iter().position(|w| w.kind == WinKind::Panel) {
            self.wins[i].w.open = true;
            self.wins[i].minimized = false;
            self.place_panel(i, work);
            self.wins[i].w.end_sessions();
        }
    }

    pub(crate) fn activate(&mut self, i: usize) {
        if i + 1 >= self.wins.len() {
            return;
        }
        let w = self.wins.remove(i);
        self.wins.push(w);
    }

    #[allow(dead_code)]
    pub(crate) fn activate_id(&mut self, id: u64) {
        self.pending = Some(id);
    }

    fn place_defaults(&mut self, work: egui::Rect) {
        if self.placed {
            return;
        }
        self.placed = true;
        for i in 0..self.wins.len() {
            match self.wins[i].kind {
                WinKind::Panel => self.place_panel(i, work),
                WinKind::Chart => {
                    self.place_default(i, work);
                }
                WinKind::Source | WinKind::Console | WinKind::Color => {}
            }
        }
    }

    fn clamp_all(&mut self, work: egui::Rect) {
        for w in self.wins.iter_mut() {
            if w.w.open {
                w.w.clamp_into(work);
            }
        }
    }

    fn update_move_guards(&mut self) {
        let live: Vec<(u64, egui::Rect, bool)> = self
            .wins
            .iter()
            .filter(|w| w.w.open && !w.minimized && !w.w.maximized && !w.pinned)
            .map(|w| (w.w.id, w.w.rect(), w.exclusive))
            .collect();
        for i in 0..self.wins.len() {
            let window = &self.wins[i];
            let guards = if window.minimized || window.w.maximized || window.pinned {
                Vec::new()
            } else if window.exclusive {
                live.iter()
                    .filter(|(id, _, _)| *id != window.w.id)
                    .map(|(id, rect, exclusive)| (*id, *rect, *exclusive))
                    .collect()
            } else {
                live.iter()
                    .filter(|(id, _, exclusive)| *id != window.w.id && *exclusive)
                    .map(|(id, rect, exclusive)| (*id, *rect, *exclusive))
                    .collect()
            };
            self.wins[i].w.set_move_guards(guards);
        }
    }

    fn overlap_blocks_exclusive(&self, id: u64, exclusive: bool, rect: egui::Rect) -> bool {
        self.wins.iter().any(|window| {
            window.w.id != id
                && window.w.open
                && !window.minimized
                && !window.w.maximized
                && !window.pinned
                && (exclusive || window.exclusive)
                && interior_overlap(rect, window.w.rect())
        })
    }

    #[cfg(test)]
    fn exclusive_blocks(&self, id: u64, rect: egui::Rect) -> bool {
        self.wins.iter().any(|window| {
            window.w.id != id
                && window.w.open
                && !window.minimized
                && !window.w.maximized
                && !window.pinned
                && window.exclusive
                && interior_overlap(rect, window.w.rect())
        })
    }

    fn free_position_as(
        &self,
        i: usize,
        size: egui::Vec2,
        work: egui::Rect,
        exclusive: bool,
    ) -> egui::Pos2 {
        if !work.is_positive() {
            return work.min;
        }
        for k in 0..=self.wins.len() + 32 {
            let step = CASCADE_STEP * k as f32;
            let want = egui::pos2(work.min.x + step, work.min.y + step);
            let pos = egui::pos2(
                clamp_soft(want.x, work.min.x, work.max.x - size.x),
                clamp_soft(want.y, work.min.y, work.max.y - size.y),
            );
            let rect = egui::Rect::from_min_size(pos, size);
            let id = self.wins[i].w.id;
            if !self.overlap_blocks_exclusive(id, exclusive, rect) && !self.occupied_except(pos, i)
            {
                return pos;
            }
        }
        work.min
    }

    fn set_exclusive(&mut self, i: usize, exclusive: bool, work: egui::Rect) {
        if exclusive && self.wins[i].pinned {
            self.wins[i].exclusive = false;
            self.wins[i].w.end_sessions();
            return;
        }
        if exclusive {
            let id = self.wins[i].w.id;
            let rect = self.wins[i].w.rect();
            if self.overlap_blocks_exclusive(id, true, rect) {
                let size = self.wins[i].w.size;
                let pos = self.free_position_as(i, size, work, true);
                self.wins[i].w.pos = pos;
                self.wins[i].restore.0 = pos;
            }
        }
        self.wins[i].exclusive = exclusive;
        self.wins[i].w.end_sessions();
    }

    pub(crate) fn mini_width(app: &mut StockApp, title: &str) -> f32 {
        let need = app.text_width(title) + 2.0 * TEXT_PAD_X + SYS_BTN_W - LINE_W;
        cells(((need - LINE_W) / PITCH).ceil() as i32).max(MINI_W)
    }

    fn taskbar_menu_rect(&self, app: &mut StockApp) -> egui::Rect {
        let text = ButtonContent::Text(TASKBAR_MENU_TEXT.to_owned());
        let width = app.button_width(&text).min(self.taskbar.width());
        egui::Rect::from_min_size(self.taskbar.min, egui::vec2(width, self.taskbar.height()))
    }

    fn taskbar_pager_rects(&self) -> [egui::Rect; 2] {
        let down = egui::Rect::from_min_size(
            egui::pos2(self.taskbar.max.x - SYS_BTN_W, self.taskbar.min.y),
            egui::vec2(SYS_BTN_W, self.taskbar.height()),
        );
        let up = egui::Rect::from_min_size(
            egui::pos2(down.min.x - SYS_BTN_W + LINE_W, self.taskbar.min.y),
            egui::vec2(SYS_BTN_W, self.taskbar.height()),
        );
        [up, down]
    }

    fn taskbar_gap_rect(&self, app: &mut StockApp) -> egui::Rect {
        let menu = self.taskbar_menu_rect(app);
        egui::Rect::from_min_size(
            egui::pos2(menu.max.x - LINE_W, self.taskbar.min.y),
            egui::vec2(TASKBAR_GAP_W, self.taskbar.height()),
        )
    }

    fn taskbar_order(&self) -> Vec<usize> {
        let mut items: Vec<usize> = (0..self.wins.len())
            .filter(|&i| self.wins[i].w.open && self.wins[i].minimized)
            .collect();
        items.sort_by_key(|&i| self.wins[i].mini_seq);
        items
    }

    fn taskbar_page_range(&self, len: usize) -> (usize, usize) {
        if self.taskbar_pages.is_empty() {
            (0, len)
        } else {
            self.taskbar_pages[self.taskbar_page.min(self.taskbar_pages.len() - 1)]
        }
    }

    fn begin_taskbar_drag(&mut self, p: egui::Pos2) {
        if self.taskbar_drag.is_some() || !self.taskbar_items.contains(p) {
            return;
        }
        let Some(id) = self.taskbar_item_at(p) else {
            return;
        };
        let Some(i) = self.wins.iter().position(|w| w.w.id == id) else {
            return;
        };
        let rect = self.rect_of(i);
        let title = free_band(rect, 0, WIN_TITLE_ROWS);
        let keys = sys_button_rects_n(title, 1);
        let handle = egui::Rect::from_min_max(title.min, egui::pos2(keys[0].min.x, title.max.y));
        if !handle.contains(p) {
            return;
        }
        self.focused_id = Some(id);
        self.taskbar_drag = Some(TaskbarDrag {
            id,
            grab: p.x - rect.min.x,
            start_x: p.x,
            x: rect.min.x,
            moved: false,
            page: self.taskbar_page,
        });
    }

    fn update_taskbar_drag(&mut self, ui: &mut egui::Ui) -> bool {
        let (pressed, down, latest) = ui.input(|i| {
            (
                i.pointer.primary_pressed(),
                i.pointer.primary_down(),
                i.pointer.latest_pos(),
            )
        });
        if pressed
            && let Some(p) = ui.input(|i| {
                i.pointer
                    .interact_pos()
                    .or_else(|| i.pointer.press_origin())
                    .or_else(|| i.pointer.latest_pos())
            })
        {
            self.begin_taskbar_drag(p);
        }
        let Some(mut drag) = self.taskbar_drag else {
            return false;
        };
        if down {
            if let Some(p) = latest {
                drag.x = clamp_soft(
                    p.x - drag.grab,
                    self.taskbar_items.min.x,
                    (self.taskbar_items.max.x - self.taskbar_item_width(drag.id))
                        .max(self.taskbar_items.min.x),
                );
                drag.moved |= (p.x - drag.start_x).abs() >= 3.0;
            }
            self.taskbar_drag = Some(drag);
            false
        } else {
            if drag.moved
                && let Some(p) = latest
                && self.taskbar_items.contains(p)
            {
                self.reorder_taskbar(drag.id, drag.x, drag.page);
            }
            let suppress_click = drag.moved;
            self.taskbar_drag = None;
            suppress_click
        }
    }

    fn taskbar_item_width(&self, id: u64) -> f32 {
        self.wins
            .iter()
            .find(|w| w.w.id == id)
            .map_or(MINI_W, |w| w.mini_w.min(self.taskbar_items.width()))
    }

    fn reorder_taskbar(&mut self, id: u64, x: f32, page: usize) {
        let mut order = self.taskbar_order();
        let Some(position) = order.iter().position(|&i| self.wins[i].w.id == id) else {
            return;
        };
        let (start, end) = if self.taskbar_pages.is_empty() {
            (0, order.len())
        } else {
            let page = page.min(self.taskbar_pages.len() - 1);
            self.taskbar_pages[page]
        };
        if position < start || position >= end {
            return;
        }
        let dragged_index = order[position];
        let area = self.taskbar_items;
        let mut target = 0usize;
        let mut cursor = area.min.x;
        for &item in &order[start..end] {
            if item == dragged_index {
                continue;
            }
            let w = self.wins[item].mini_w.min(area.width());
            if x > cursor + w / 2.0 {
                target += 1;
            }
            cursor += (w - LINE_W).max(LINE_W);
        }
        let dragged = order.remove(position);
        let insert = (start + target).min(order.len());
        order.insert(insert, dragged);
        for (seq, &i) in order.iter().enumerate() {
            self.wins[i].mini_seq = seq as u64;
        }
        self.next_mini = order.len() as u64;
    }

    fn update_taskbar_layout(&mut self, app: &mut StockApp) {
        let gap = self.taskbar_gap_rect(app);
        self.taskbar_gap = gap;
        let [up, _] = self.taskbar_pager_rects();
        self.taskbar_items = egui::Rect::from_min_max(
            egui::pos2(gap.max.x - LINE_W, self.taskbar.min.y),
            egui::pos2(up.min.x, self.taskbar.max.y),
        );
        let order = self.taskbar_order();
        let mut pages = Vec::new();
        let mut start = 0usize;
        while start < order.len() {
            let mut end = start;
            let mut width = 0.0;
            while end < order.len() {
                let item_w = self.wins[order[end]].mini_w.min(self.taskbar_items.width());
                let next = if end == start {
                    item_w
                } else {
                    width + item_w - LINE_W
                };
                if next > self.taskbar_items.width() && end > start {
                    break;
                }
                width = next;
                end += 1;
            }
            if end == start {
                end += 1;
            }
            pages.push((start, end));
            start = end;
        }
        if pages.is_empty() {
            pages.push((0, 0));
        }
        self.taskbar_page = self.taskbar_page.min(pages.len() - 1);
        self.taskbar_pages = pages;
    }

    fn mini_rect(&self, i: usize, _k: usize) -> egui::Rect {
        let order = self.taskbar_order();
        let Some(k) = order.iter().position(|&index| index == i) else {
            return egui::Rect::NOTHING;
        };
        let area = if self.taskbar_items.is_positive() {
            self.taskbar_items
        } else {
            self.taskbar
        };
        let (start, end) = self.taskbar_page_range(order.len());
        if k < start || k >= end {
            return egui::Rect::NOTHING;
        }
        let w = self.wins[i].mini_w.min(area.width());
        if self
            .taskbar_drag
            .is_some_and(|drag| drag.id == self.wins[i].w.id)
        {
            let x = self.taskbar_drag.map_or(area.min.x, |drag| drag.x);
            return egui::Rect::from_min_size(
                egui::pos2(
                    clamp_soft(x, area.min.x, (area.max.x - w).max(area.min.x)),
                    area.min.y,
                ),
                egui::vec2(w, area.height()),
            );
        }
        let mut x = area.min.x;
        for &item in &order[start..k] {
            x += (self.wins[item].mini_w.min(area.width()) - LINE_W).max(LINE_W);
        }
        egui::Rect::from_min_size(
            egui::pos2(
                clamp_soft(x, area.min.x, (area.max.x - w).max(area.min.x)),
                area.min.y,
            ),
            egui::vec2(w, area.height()),
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

    fn taskbar_item_at(&self, p: egui::Pos2) -> Option<u64> {
        self.draw_order()
            .into_iter()
            .filter(|&i| self.wins[i].minimized)
            .find(|&i| {
                let rect = self.rect_of(i);
                rect.is_positive() && rect.contains(p)
            })
            .map(|i| self.wins[i].w.id)
    }

    fn draw_order(&self) -> Vec<usize> {
        let live = |w: &ManagedWin, mini: bool| w.w.open && w.minimized == mini;
        let mut out: Vec<usize> = (0..self.wins.len())
            .filter(|&i| {
                live(&self.wins[i], false) && !self.wins[i].pinned && self.wins[i].exclusive
            })
            .collect();
        out.extend((0..self.wins.len()).filter(|&i| {
            live(&self.wins[i], false) && !self.wins[i].pinned && !self.wins[i].exclusive
        }));
        let mut pinned: Vec<usize> = (0..self.wins.len())
            .filter(|&i| live(&self.wins[i], false) && self.wins[i].pinned)
            .collect();
        pinned.sort_by_key(|&i| self.wins[i].pin_seq);
        out.extend(pinned);
        let mut minimized: Vec<usize> = (0..self.wins.len())
            .filter(|&i| live(&self.wins[i], true))
            .collect();
        minimized.sort_by_key(|&i| {
            let dragging = self
                .taskbar_drag
                .is_some_and(|drag| drag.id == self.wins[i].w.id);
            (dragging, self.wins[i].mini_seq)
        });
        out.extend(minimized);
        out
    }

    pub(crate) fn toggle_pin(&mut self, i: usize) {
        self.set_pin(i, !self.wins[i].pinned);
    }

    pub(crate) fn set_pin(&mut self, i: usize, pinned: bool) {
        if pinned {
            self.wins[i].exclusive = false;
        }
        self.wins[i].pinned = pinned;
        if self.wins[i].pinned {
            self.wins[i].pin_seq = self.next_pin;
            self.next_pin += 1;
        } else {
            self.wins[i].pin_seq = 0;
        }
        self.wins[i].w.end_sessions();
    }

    #[allow(dead_code)]
    pub(crate) fn toggle_exclusive(&mut self, i: usize) {
        let exclusive = !self.wins[i].exclusive;
        let work = self.work;
        self.set_exclusive(i, exclusive, work);
    }

    #[cfg(test)]
    pub(crate) fn order_ids(&self) -> Vec<u64> {
        self.draw_order()
            .into_iter()
            .map(|i| self.wins[i].w.id)
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn taskbar_items_rect(&self) -> egui::Rect {
        self.taskbar_items
    }

    #[cfg(test)]
    pub(crate) fn taskbar_gap_rect_for_test(&self) -> egui::Rect {
        self.taskbar_gap
    }

    #[cfg(test)]
    pub(crate) fn taskbar_ids(&self) -> Vec<u64> {
        self.taskbar_order()
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
        if let Some(grip) = self.grip_owner_at(p) {
            return Some(grip);
        }
        self.top_win_at(p)
    }

    fn edge_at_any(&self, p: egui::Pos2) -> bool {
        self.draw_order().into_iter().any(|i| {
            let w = &self.wins[i];
            w.w.open
                && !w.minimized
                && !w.w.maximized
                && w.w.resizable
                && edge_at(w.w.rect(), p, EDGE_BAND).any()
        })
    }

    fn grip_owner_at(&self, p: egui::Pos2) -> Option<usize> {
        self.draw_order().into_iter().rev().find(|&i| {
            let w = &self.wins[i];
            self.focused_id == Some(w.w.id)
                && w.w.open
                && !w.minimized
                && !w.w.maximized
                && w.w.resizable
                && edge_at(w.rect(), p, EDGE_BAND).any()
        })
    }

    pub(crate) fn can_spawn_window(&self, work: egui::Rect, size: egui::Vec2) -> bool {
        if !work.is_positive() {
            return true;
        }
        let outer = size + egui::vec2(2.0 * WIN_FRAME, 2.0 * WIN_FRAME);
        self.spawn_position_for(work, outer, None).is_some()
    }

    pub(crate) fn add_chart(&mut self, work: egui::Rect, pane: usize) -> Option<usize> {
        let size = egui::vec2(NEW_WIN_W.min(work.width()), NEW_WIN_H.min(work.height()));
        if !self.can_spawn_window(work, size) {
            return None;
        }
        let i = self.add(
            WinKind::Chart,
            WIN_CHART_TITLE,
            pane,
            size,
            Self::chart_min_size(),
        );
        if !self.place_default(i, work) {
            self.wins[i].w.open = false;
            return None;
        }

        Some(i)
    }

    pub(crate) fn chart_min_size() -> egui::Vec2 {
        egui::vec2(PANE_MIN_W, PANE_H_MIN)
    }

    fn occupied_except(&self, p: egui::Pos2, skip: usize) -> bool {
        self.wins
            .iter()
            .enumerate()
            .any(|(i, w)| i != skip && w.w.open && !w.minimized && (w.w.pos - p).length_sq() < 0.5)
    }

    pub(crate) fn drag_active(&self) -> bool {
        self.taskbar_drag.is_some() || self.wins.iter().any(|w| w.w.dragging() || w.w.resizing())
    }

    pub(crate) fn session_kind(&self) -> Option<CursorKind> {
        if self.taskbar_drag.is_some() {
            return Some(CursorKind::Move);
        }
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

    #[cfg(test)]
    pub(crate) fn dock(&mut self, i: usize, r: egui::Rect) {
        if self.exclusive_blocks(self.wins[i].w.id, r) {
            return;
        }
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
        self.taskbar_pages.clear();
        if self.wins[i].kind == WinKind::Chart && app.pane_menu == Some(pane) {
            app.pane_menu = None;
        }
        if self.wins[i].kind == WinKind::Color {
            app.color_panel.close_cancelled();
            app.status_msg = "颜色面板：已取消，颜色没变".to_owned();
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
        self.wins[i].restore = self.wins[i].w.restore_rect();
        self.wins[i].mini_w = mini_w;
        self.wins[i].minimized = true;
        self.wins[i].w.maximized = false;
        self.taskbar_pages.clear();
        self.next_mini += 1;
        self.wins[i].mini_seq = self.next_mini;
        self.wins[i].w.end_sessions();
    }

    pub(crate) fn restore(&mut self, i: usize) {
        let (pos, size) = self.wins[i].restore;
        self.wins[i].w.pos = pos;
        self.wins[i].w.size = size;
        self.wins[i].w.maximized = false;
        self.wins[i].minimized = false;
        self.taskbar_pages.clear();
    }

    #[allow(dead_code)]
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

    pub(crate) fn plan(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        work: egui::Rect,
        taskbar: egui::Rect,
    ) {
        self.work = work;
        self.taskbar = taskbar;
        self.update_taskbar_layout(app);
        self.place_defaults(work);
        if work.is_positive() && !self.pending_placement.is_empty() {
            for id in std::mem::take(&mut self.pending_placement) {
                if let Some(i) = self.wins.iter().position(|w| w.w.id == id) {
                    match self.wins[i].kind {
                        WinKind::Panel => self.place_panel(i, work),
                        _ => {
                            self.place_default(i, work);
                        }
                    }
                }
            }
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
        self.update_move_guards();

        if !self.drag_active()
            && let Some(p) = ui.input(|i| i.pointer.latest_pos())
            && work.contains(p)
            && !app.popup_at(p, ui.max_rect().max.y)
            && !self.edge_at_any(p)
            && let Some(i) = self.top_win_at(p)
        {
            let id = self.wins[i].w.id;
            if self.focused_id != Some(id) {
                self.focused_id = Some(id);
            }
        }

        let (primary_pressed, pointer, press_origin, interact_pos) = ui.input(|i| {
            (
                i.pointer.primary_pressed(),
                i.pointer.latest_pos(),
                i.pointer.press_origin(),
                i.pointer.interact_pos(),
            )
        });
        if primary_pressed {
            let p = interact_pos.or(press_origin).or(pointer);
            if let Some(p) = p
                && !app.popup_at(p, ui.max_rect().max.y)
            {
                if let Some(i) = self.press_owner_at(p) {
                    let id = self.wins[i].w.id;
                    self.focused_id = Some(id);
                    self.activate(i);
                }
            }
        }

        if primary_pressed
            && let Some(p) = press_origin.or(interact_pos)
            && !app.popup_at(p, ui.max_rect().max.y)
            && let Some(i) = self.press_owner_at(p)
            && self.focused_id == Some(self.wins[i].w.id)
            && edge_at(self.wins[i].rect(), p, EDGE_BAND).any()
            && !self.wins[i].w.edge_guard.contains(p)
        {
            self.wins[i].w.begin_resize(p);
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
            let layout = app.panes[pane].chart_layout();
            let (_, _, main, _) = chart_rows(body, layout);
            self.plots[pane] = frame_rects_with_axis(main, app.panes[pane].show_axis()).0;
        }
    }

    pub(crate) fn draw(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        work: egui::Rect,
        taskbar: egui::Rect,
    ) {
        self.work = work;
        self.taskbar = taskbar;
        let taskbar_drag_released = self.update_taskbar_drag(ui);
        for i in self.draw_order() {
            if !self.wins[i].w.open {
                continue;
            }
            if self.wins[i].minimized && !self.rect_of(i).is_positive() {
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
        let taskbar_click = ui.input(|input| {
            input
                .pointer
                .primary_clicked()
                .then(|| input.pointer.interact_pos())
                .flatten()
        });
        if !taskbar_drag_released
            && let Some(p) = taskbar_click
            && let Some(id) = self.taskbar_item_at(p)
        {
            self.run_taskbar_click(app, p, id);
        }
        self.draw_taskbar_controls(app, ui);
        app.win_silent = false;
    }

    fn draw_taskbar_controls(&mut self, app: &mut StockApp, ui: &mut egui::Ui) {
        let painter = ui.painter().clone();
        let menu = self.taskbar_menu_rect(app);
        let menu_text = ButtonContent::Text(TASKBAR_MENU_TEXT.to_owned());
        app.draw_button(ui, &painter, menu, &menu_text, false, "");

        let gap = self.taskbar_gap;
        painter.rect_filled(gap, egui::CornerRadius::ZERO, theme().status_bg);
        border_hard(&painter, gap, theme().ctrl_border);
        vline_hard(
            &painter,
            gap.center().x,
            gap.min.y,
            gap.max.y,
            theme().ctrl_border,
        );

        let [up, down] = self.taskbar_pager_rects();
        let up_content = ButtonContent::Symbol(SymbolKind::PageUp);
        let down_content = ButtonContent::Symbol(SymbolKind::PageDown);
        if app.draw_button(ui, &painter, up, &up_content, false, "上一页") {
            self.taskbar_page = self.taskbar_page.saturating_sub(1);
        }
        if app.draw_button(ui, &painter, down, &down_content, false, "下一页") {
            self.taskbar_page =
                (self.taskbar_page + 1).min(self.taskbar_pages.len().saturating_sub(1));
        }
    }

    fn run_taskbar_click(&mut self, app: &mut StockApp, p: egui::Pos2, id: u64) {
        let Some(i) = self.wins.iter().position(|window| window.w.id == id) else {
            return;
        };
        let rect = self.rect_of(i);
        let title = free_band(rect, 0, WIN_TITLE_ROWS);
        let keys = sys_button_rects_n(title, 1);
        let handle = egui::Rect::from_min_max(title.min, egui::pos2(keys[0].min.x, title.max.y));
        if keys[0].contains(p) {
            self.close_win(app, i);
        } else if handle.contains(p) {
            self.restore(i);
            if let Some(j) = self.wins.iter().position(|window| window.w.id == id) {
                self.focused_id = Some(id);
                self.activate(j);
                if let Some(j) = self.wins.iter().position(|window| window.w.id == id) {
                    self.wins[j].w.guard_open_click();
                }
            }
        }
    }

    fn draw_content(&mut self, app: &mut StockApp, ui: &mut egui::Ui, i: usize, body: egui::Rect) {
        match self.wins[i].kind {
            WinKind::Panel => app.ui_panel(ui, body),
            WinKind::Chart => {
                let pane = self.wins[i].pane;
                let layout = app.panes[pane].chart_layout();
                let (_, _, main, _) = chart_rows(body, layout);
                self.plots[pane] = frame_rects_with_axis(main, app.panes[pane].show_axis()).0;
                if app.win_silent {
                    app.panes[pane].view.crosshair = None;
                    app.panes[pane].view.pan = None;
                }
                app.ui_chart(ui, pane, body);
            }
            WinKind::Source => {
                self.wins[i].title = app.source_title();

                let rect = self.wins[i].w.inner_rect();
                let cmd = app.ui_source_body(ui, rect);
                app.run_editor_cmd(cmd);
            }
            WinKind::Console => {
                let mut console = std::mem::take(&mut app.console);
                console.ui(app, ui, body);
                app.console = console;
            }
            WinKind::Color => {
                let panel = egui::Rect::from_min_max(
                    egui::pos2(body.min.x, body.min.y - grid_line(WIN_TITLE_ROWS)),
                    body.max,
                );
                app.ui_color_panel_content(ui, panel);
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
        let work = self.work;
        if self.wins[i].minimized {
            let rect = self.rect_of(i);
            let title = free_band(rect, 0, WIN_TITLE_ROWS);
            let painter = ui.painter().clone();
            paint_title_bar(
                app,
                ui,
                &painter,
                title,
                self.wins[i].w.id,
                &name,
                WinKeys::CloseOnly,
                false,
                self.wins[i].pinned,
                true,
                self.wins[i].exclusive,
            );
            return None;
        }

        let pinned = self.wins[i].pinned;
        let exclusive = self.wins[i].exclusive;
        let active = self.focused_id == Some(self.wins[i].w.id);
        let keys = if self.wins[i].kind == WinKind::Color {
            WinKeys::MinAndClose
        } else {
            WinKeys::MinMaxClose
        };
        let out = self.wins[i].w.begin(
            app, ui, work, &name, keys, busy, active, pinned, false, exclusive,
        );
        if out.exclusive {
            let exclusive = !self.wins[i].exclusive;
            self.set_exclusive(i, exclusive, work);
        } else if out.pin {
            let pinned = !self.wins[i].pinned;
            self.set_pin(i, pinned);
        } else if out.close {
            self.close_win(app, i);
        } else if out.maximize || out.restore_maximized {
            self.wins[i].w.toggle_maximize(work);
        } else if out.minimize && !self.wins[i].minimized {
            let width = Self::mini_width(app, &name);
            self.minimize(i, width);
            app.tooltip = None;
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
