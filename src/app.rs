#[cfg(test)]
use crate::data;
use crate::ecs;
use crate::font::*;
use crate::grid::*;
use crate::indicator;
use crate::platform::*;
use crate::pointer::*;
use crate::theme::*;
use crate::ui::colorpanel::*;
use crate::ui::console::*;
use crate::ui::contextmenu::*;
use crate::ui::icon::*;
use crate::ui::menu::*;
use crate::ui::panel::*;
use crate::ui::source::*;
pub(crate) use crate::ui::symbol::*;
use crate::ui::titlebar::*;
use crate::ui::winmgr::*;
use eframe::egui;
use raw_window_handle::HasWindowHandle;
use std::ops::{Deref, DerefMut};
use std::time::{Duration, Instant};

pub(crate) const APP_NAME: &str = "圆桌";

pub(crate) const WHEEL_HOLD_S: f64 = 0.3;

pub(crate) const CARET_BLINK_PERIOD_S: f64 = 1.0;

pub(crate) fn caret_blink_visible_at(elapsed: f64) -> bool {
    elapsed.rem_euclid(CARET_BLINK_PERIOD_S) < CARET_BLINK_PERIOD_S * 0.5
}

#[derive(Clone, Copy)]
pub(crate) enum WinCmd {
    AddChart,

    OpenPanel,

    OpenSource(usize),

    OpenConsole,

    CloseConsole,
}

pub(crate) use crate::ecs::{AppWorld, Crosshair, Pane};

#[derive(Clone, Default)]
pub(crate) struct DebugWindowSnapshot {
    pub(crate) windows: Vec<(String, bool, egui::Rect)>,
    pub(crate) chart_count: usize,
    pub(crate) source_open: bool,
    pub(crate) console_open: bool,
}

pub(crate) const PANE_CANDLE: usize = 0;

#[allow(dead_code)]
pub(crate) const PANE_VOLUME: usize = 1;
#[allow(dead_code)]
pub(crate) const PANE_MACD: usize = 2;

impl StockApp {
    pub(crate) fn build_panes(pal: indicator::Palette) -> Vec<Pane> {
        indicator::default_layout()
            .into_iter()
            .map(|mut ind| {
                ind.set_palette(pal);

                Pane::new(ind)
            })
            .collect()
    }

    pub(crate) fn current_palette(&self) -> indicator::Palette {
        let t = theme();
        let rgb = |c: egui::Color32| (c.r(), c.g(), c.b());

        let six = |colors: [egui::Color32; 6]| {
            let mut out = [0u32; 6];
            for (i, c) in colors.iter().enumerate() {
                out[i] = indicator::rgb_u32(rgb(*c));
            }
            out
        };
        indicator::Palette {
            candle_up: indicator::rgb_u32(self.up_color),
            candle_down: indicator::rgb_u32(self.down_color),
            text_up: indicator::rgb_u32(rgb(t.text_up)),
            text_down: indicator::rgb_u32(rgb(t.text_down)),
            dif: indicator::rgb_u32(rgb(t.dif)),
            dea: indicator::rgb_u32(rgb(t.dea)),
            kdj_k: indicator::rgb_u32(rgb(t.kdj_k)),
            kdj_d: indicator::rgb_u32(rgb(t.kdj_d)),
            kdj_j: indicator::rgb_u32(rgb(t.kdj_j)),
            rsi_1: indicator::rgb_u32(rgb(t.rsi_1)),
            rsi_2: indicator::rgb_u32(rgb(t.rsi_2)),
            rsi_3: indicator::rgb_u32(rgb(t.rsi_3)),
            boll_mid: indicator::rgb_u32(rgb(t.boll_mid)),
            boll_up: indicator::rgb_u32(rgb(t.boll_up)),
            boll_down: indicator::rgb_u32(rgb(t.boll_down)),
            obv: indicator::rgb_u32(rgb(t.obv)),
            cci: indicator::rgb_u32(rgb(t.cci)),
            bias: [
                indicator::rgb_u32(rgb(t.bias[0])),
                indicator::rgb_u32(rgb(t.bias[1])),
                indicator::rgb_u32(rgb(t.bias[2])),
            ],
            ma: six(t.ma),
            ema: six(t.ema),
            candle_suspended: indicator::rgb_u32(rgb(t.cross)),
        }
    }

    pub(crate) fn sync_pane_palette(&mut self) {
        let want = self.current_palette();
        if self.panes_palette == want {
            return;
        }
        for pane in &mut self.panes {
            pane.indicator.set_palette(want);
            pane.dirty = true;
        }
        self.panes_palette = want;
    }

    pub(crate) fn switch_theme(&mut self) {
        let next_id = match theme().id {
            ThemeId::Classic => ThemeId::Dark,
            ThemeId::Dark => ThemeId::Classic,
        };
        set_theme(Theme::load(next_id));
        let next = theme();
        if self.chart_follows_theme {
            self.adopt_theme_candle_colors(next);
        }
        self.mark_panes_dirty();
        self.status_msg = format!(
            "主题：{}（图表配色{}）",
            next.name(),
            if self.chart_follows_theme {
                "跟随系统"
            } else {
                "自定义"
            }
        );
    }

    pub(crate) fn run_context_action(&mut self, ui: &egui::Ui, action: ContextAction) {
        match action {
            ContextAction::AddChart => self.add_chart_window(),
            ContextAction::OpenConsole => self.open_console_window(),
            ContextAction::SwitchTheme => self.switch_theme(),
            ContextAction::DemoOne => self.status_msg = "右键菜单：演示按钮一".to_owned(),
            ContextAction::DemoTwo => self.status_msg = "右键菜单：演示按钮二".to_owned(),
            ContextAction::Cut
            | ContextAction::Copy
            | ContextAction::Paste
            | ContextAction::SelectAll
            | ContextAction::Undo => {
                self.source.editor.apply_context_action(ui, action);
            }
        }
    }

    pub(crate) fn adopt_theme_candle_colors(&mut self, t: Theme) {
        let (u, d) = (t.candle_up, t.candle_down);
        self.up_color = (u.r(), u.g(), u.b());
        self.down_color = (d.r(), d.g(), d.b());
    }

    pub(crate) fn follow_system_chart_colors(&mut self) {
        self.chart_follows_theme = true;
        let t = theme();
        self.adopt_theme_candle_colors(t);
        self.mark_panes_dirty();
        self.status_msg = format!("图表配色已指向系统级配色（{}）", t.name());
    }

    pub(crate) fn ui_drag_active(&self) -> bool {
        self.hsplit_grab.is_some()
            || self.vsplit_grab.is_some()
            || self.win_mgr.drag_active()
            || self.color_panel.drag != ColorDrag::None
            || self.color_panel.win.dragging()
    }

    pub(crate) fn mark_panes_dirty(&mut self) {
        ecs::mark_panes_dirty(&mut self.world);
    }

    #[cfg(test)]
    pub(crate) fn replace_bars(&mut self, bars: Vec<data::random_walk::Bar>) {
        self.world.bars.replace(bars);
    }

    pub(crate) fn ensure_pane_computed(&mut self, pane: usize) {
        ecs::ensure_pane_computed(&mut self.world, pane);
    }

    pub(crate) fn request_win(&mut self, cmd: WinCmd) {
        self.win_cmd = Some(cmd);
    }

    pub(crate) fn run_win_cmd(&mut self) {
        match self.win_cmd.take() {
            None => {}
            Some(WinCmd::AddChart) => self.add_chart_window(),
            Some(WinCmd::OpenPanel) => self.open_panel_window(),
            Some(WinCmd::OpenSource(pane)) => self.open_source_window(pane),
            Some(WinCmd::OpenConsole) => self.open_console_window(),
            Some(WinCmd::CloseConsole) => self.close_console_window(),
        }
    }

    pub(crate) fn add_chart_window(&mut self) {
        if self.win_mgr.chart_count() >= MAX_CHART_WINS {
            self.status_msg = format!("图表窗最多 {MAX_CHART_WINS} 块");
            return;
        }
        let pane = self.new_chart_pane();
        self.win_mgr.add_chart(self.work_rect, pane);
        self.status_msg = format!("图表窗：已新增（共 {} 块）", self.win_mgr.chart_count());
    }

    pub(crate) fn new_chart_pane(&mut self) -> usize {
        let mut ind = indicator::builtin().swap_remove(0);
        ind.set_palette(self.current_palette());
        match self.win_mgr.free_chart_pane() {
            Some(free) => {
                self.world.replace_pane(free, Pane::new(ind));
                free
            }
            None => self.world.add_pane(Pane::new(ind)),
        }
    }

    pub(crate) fn open_panel_window(&mut self) {
        let docked = Self::child_areas(self.work_rect);
        if let Some(i) = self
            .win_mgr
            .wins
            .iter()
            .position(|w| w.kind == WinKind::Panel)
        {
            self.win_mgr.dock(i, docked[1]);
            self.status_msg = "控制面板：已摆回来".to_owned();
        }
    }

    pub(crate) fn open_console_window(&mut self) {
        self.win_mgr.show_console(self.work_rect);
        self.console.log("控制台窗口已打开。");
        self.status_msg = "调试控制台：已打开".to_owned();
    }

    pub(crate) fn close_console_window(&mut self) {
        self.win_mgr.hide_console();
        self.status_msg = "调试控制台：已关闭".to_owned();
    }
}

pub(crate) struct StockApp {
    pub(crate) world: AppWorld,

    pub(crate) hwnd: isize,
    pub(crate) client_cover_ready: bool,

    pub(crate) icons: IconSet,

    pub(crate) symbols: SymbolSet,

    pub(crate) cursors: Cursors,

    pub(crate) pointer: PointerManager,

    pub(crate) tooltip: Option<(egui::Rect, String)>,

    pub(crate) menu: MenuState,
    pub(crate) status_msg: String,
    pub(crate) frames: u64,
    pub(crate) last_frame_time: f64,
    pub(crate) caret_blink_epoch: f64,
    pub(crate) frame_interval_ms: f32,
    pub(crate) frame_ms: f32,
    pub(crate) fps: f32,

    pub(crate) text_engine: Option<HardText>,

    pub(crate) hsplit_grab: Option<f32>,

    pub(crate) vsplit_grab: Option<f32>,

    pub(crate) split_x: f32,

    pub(crate) win_mgr: WinMgr,

    pub(crate) win_cmd: Option<WinCmd>,

    pub(crate) work_rect: egui::Rect,

    pub(crate) panel_buttons: Vec<PanelButton>,

    pub(crate) panel_press: Option<usize>,

    pub(crate) color_panel: ColorPanel,

    pub(crate) win_silent: bool,

    pub(crate) context_menu: ContextMenuState,

    pub(crate) pane_menu: Option<usize>,

    pub(crate) pane_menu_anchor: egui::Rect,

    pub(crate) pane_menu_bar: egui::Rect,

    pub(crate) source: SourceView,

    pub(crate) console: DebugConsole,

    pub(crate) debug_windows: DebugWindowSnapshot,
}

impl Deref for StockApp {
    type Target = AppWorld;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl DerefMut for StockApp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}

impl StockApp {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_tessellation(&cc.egui_ctx);
        configure_zero_spacing(&cc.egui_ctx);
        set_theme(Theme::load(ThemeId::Dark));

        cc.egui_ctx.set_pixels_per_point(1.0);

        let hwnd = cc
            .window_handle()
            .ok()
            .map(|h| native_window_id(h.as_raw()))
            .unwrap_or(0);
        clear_window_frame(hwnd);
        let client_cover_ready = install_client_cover(hwnd);
        let palette = indicator::Palette::default();
        let world = AppWorld::new(Self::build_panes(palette), palette);

        let mut app = Self {
            world,
            hwnd,
            client_cover_ready,
            icons: IconSet::load_or_create(),
            symbols: SymbolSet::load(),
            cursors: Cursors::load(),
            pointer: PointerManager::new(mouse_feel()),
            tooltip: None,
            menu: MenuState::default(),
            status_msg: String::new(),
            frames: 0,
            last_frame_time: 0.0,
            caret_blink_epoch: 0.0,
            frame_interval_ms: 0.0,
            frame_ms: 0.0,
            fps: 0.0,
            text_engine: HardText::new(&cc.egui_ctx),
            hsplit_grab: None,
            vsplit_grab: None,

            split_x: CHART_PANE_W,

            win_mgr: WinMgr::with_default_windows(),
            win_cmd: None,
            work_rect: egui::Rect::NOTHING,
            panel_buttons: Vec::new(),
            panel_press: None,
            color_panel: ColorPanel::new(),
            win_silent: false,
            context_menu: ContextMenuState::default(),
            pane_menu: None,
            pane_menu_anchor: egui::Rect::NOTHING,
            pane_menu_bar: egui::Rect::NOTHING,
            source: SourceView::new(),
            console: DebugConsole::new(),
            debug_windows: DebugWindowSnapshot::default(),
        };

        if app.chart_follows_theme {
            app.adopt_theme_candle_colors(theme());
        }

        app.sync_pane_palette();

        app.build_panel_buttons();
        app
    }

    pub(crate) fn tick_frame(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if self.last_frame_time > 0.0 {
            let dt = (now - self.last_frame_time) as f32;
            if dt > 0.0 {
                self.frame_interval_ms = dt * 1000.0;
                let instant = 1.0 / dt;

                self.fps = if self.fps > 0.0 {
                    self.fps * 0.85 + instant * 0.15
                } else {
                    instant
                };
            }
        }
        self.last_frame_time = now;
        self.frames += 1;
    }

    pub(crate) fn reset_caret_blink(&mut self, ctx: &egui::Context) {
        self.caret_blink_epoch = ctx.input(|i| i.time);
    }

    pub(crate) fn caret_visible(&self, ctx: &egui::Context) -> bool {
        let now = ctx.input(|i| i.time);
        let elapsed = (now - self.caret_blink_epoch).max(0.0);
        let phase = elapsed.rem_euclid(CARET_BLINK_PERIOD_S);
        let visible = caret_blink_visible_at(elapsed);
        let half = CARET_BLINK_PERIOD_S * 0.5;
        let wait = if visible {
            half - phase
        } else {
            CARET_BLINK_PERIOD_S - phase
        };
        ctx.request_repaint_after(Duration::from_secs_f64(wait.max(0.001)));
        visible
    }

    pub(crate) fn status_segments(&self) -> Vec<String> {
        let mut segs = vec![format!("{:.0} FPS", self.fps)];

        if !self.status_msg.is_empty() {
            segs.push(self.status_msg.clone());
        }
        segs.push(current_time_text());
        segs
    }

    fn dismiss_popup_menus_on_keyboard(&mut self, ctx: &egui::Context) {
        let pressed = ctx.input(|i| {
            i.events
                .iter()
                .any(|event| matches!(event, egui::Event::Key { pressed: true, .. }))
        });
        if !pressed {
            return;
        }
        self.menu = MenuState::default();
        self.pane_menu = None;
        self.source.editor.menu = MenuState::default();
        self.context_menu.close();
    }

    pub(crate) fn chart_status_segments(&self, pane: usize) -> Vec<String> {
        let view = &self.panes[pane].view;
        vec![
            format!("缩放 {}", view.scale),
            format!("平移 {}", view.offset),
            format!("可见 {} / 共 {} 根", view.visible, self.bars.len()),
        ]
    }

    pub(crate) fn active_chart_pane(&self) -> Option<usize> {
        self.win_mgr.top_chart_pane()
    }

    pub(crate) fn paint_status_bar(
        &mut self,
        painter: &egui::Painter,
        band: egui::Rect,
        segs: &[String],
    ) {
        painter.rect_filled(band, egui::CornerRadius::ZERO, theme().status_bg);
        hline_hard(painter, band.min.x, band.max.x, band.min.y, theme().line);
        let mut x = band.min.x;
        let mut first = true;
        for seg in segs {
            let w = snap_cells_up(self.text_width(seg) + 2.0 * TEXT_PAD_X);
            if x + w > band.max.x {
                break;
            }
            if !first {
                vline_hard(
                    painter,
                    x,
                    band.min.y + LINE_W,
                    band.max.y - LINE_W,
                    theme().line,
                );
            }
            self.text(
                painter,
                egui::pos2(text_left_x(x, band.min.x), band.center().y),
                egui::Align2::LEFT_CENTER,
                seg,
                theme().text,
            );
            x += w - LINE_W;
            first = false;
        }
        if !first {
            vline_hard(
                painter,
                x,
                band.min.y + LINE_W,
                band.max.y - LINE_W,
                theme().line,
            );
            let right = band.max.x - LINE_W;
            if right > x {
                vline_hard(
                    painter,
                    right,
                    band.min.y + LINE_W,
                    band.max.y - LINE_W,
                    theme().line,
                );
            }
        }
    }

    pub(crate) fn text(
        &mut self,
        painter: &egui::Painter,
        pos: egui::Pos2,
        align: egui::Align2,
        s: &str,
        color: egui::Color32,
    ) {
        if let Some(engine) = &mut self.text_engine {
            engine.draw(painter, pos, align, s, color);
        }
    }

    pub(crate) fn text_width(&mut self, s: &str) -> f32 {
        self.text_engine
            .as_mut()
            .map_or(0.0, |engine| engine.width(s) as f32)
    }

    pub(crate) fn paint_skeleton(
        &mut self,
        painter: &egui::Painter,
        _layout: egui::Rect,
        areas: [egui::Rect; 4],
    ) {
        let [title, menu, work, status] = areas;
        painter.rect_filled(title, egui::CornerRadius::ZERO, theme().title_bg);
        let name_slot = title_text_slot(self, title, APP_NAME);
        paint_title_text_edge(painter, name_slot);
        painter.rect_filled(menu, egui::CornerRadius::ZERO, theme().menu_bg);

        painter.rect_filled(work, egui::CornerRadius::ZERO, theme().desktop_bg);
        painter.rect_filled(status, egui::CornerRadius::ZERO, theme().status_bg);

        let segs = self.status_segments();
        self.paint_status_bar(painter, status, &segs);
    }

    pub(crate) fn paint_separators(
        &self,
        painter: &egui::Painter,
        layout: egui::Rect,
        areas: [egui::Rect; 4],
    ) {
        for y in [areas[1].min.y, areas[2].min.y, areas[3].min.y] {
            hline_hard(painter, layout.min.x, layout.max.x, y, theme().line);
        }
    }

    pub(crate) fn floating_at(&self, p: egui::Pos2, window_bottom: f32) -> bool {
        if self.color_panel.win.open && self.color_panel.win.rect().contains(p) {
            return true;
        }
        if self.pane_menu.is_some() {
            let count = indicator::builtin_names().len();
            if self
                .pane_menu_rect(count, window_bottom)
                .is_some_and(|r| r.contains(p))
            {
                return true;
            }
        }
        false
    }

    fn advance_software_pointer(&mut self, ctx: &egui::Context, layout: egui::Rect) {
        let now = ctx.input(|i| i.time);

        if self.pointer.system {
            let screen = (self.hwnd != 0)
                .then(cursor_pos)
                .flatten()
                .map(|(x, y)| egui::pos2(x as f32, y as f32))
                .or_else(|| ctx.input(|i| i.pointer.latest_pos()))
                .unwrap_or(layout.center());
            self.pointer.sync(screen, now);
            self.pointer.live = Some(match client_origin(self.hwnd) {
                Some((ox, oy)) => egui::pos2(screen.x - ox as f32, screen.y - oy as f32),
                None => screen,
            });
            return;
        }
        let mut deltas = Vec::new();
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::MouseMoved(delta) = event {
                    deltas.push(*delta);
                }
            }
        });

        let screen_bounds = screen_area()
            .map(|(x, y, w, h)| {
                egui::Rect::from_min_max(
                    egui::pos2(x as f32, y as f32),
                    egui::pos2((x + w - 1) as f32, (y + h - 1) as f32),
                )
            })
            .unwrap_or(layout);
        if !self.pointer.is_initialized() {
            let start = match (cursor_pos(), client_origin(self.hwnd)) {
                (Some((x, y)), _) if self.hwnd != 0 => egui::pos2(x as f32, y as f32),
                _ => ctx
                    .input(|i| i.pointer.latest_pos())
                    .unwrap_or(layout.center()),
            };
            self.pointer.reset(start, now);
        } else if self.hwnd != 0
            && let Some((x, y)) = cursor_pos()
        {
            let physical = egui::pos2(x as f32, y as f32);
            if physical.distance(self.pointer.position()) > 32.0 {
                self.pointer.reset(physical, now);
            }
        }

        let moving = !deltas.is_empty();
        let at = if deltas.is_empty() {
            if self.hwnd == 0 {
                if let Some(p) = ctx.input(|i| i.pointer.latest_pos()) {
                    self.pointer.sync(p, now);
                }
            }
            self.pointer.position()
        } else {
            self.pointer.advance(&deltas, now, screen_bounds)
        };

        if moving {
            ctx.request_repaint();
        }

        let shown = if self.hwnd != 0 {
            egui::pos2(at.x.round(), at.y.round())
        } else {
            at
        };
        if self.hwnd != 0 {
            set_cursor_pos(shown.x as i32, shown.y as i32);
        }
        self.pointer.live = Some(match client_origin(self.hwnd) {
            Some((ox, oy)) => egui::pos2(shown.x - ox as f32, shown.y - oy as f32),
            None => shown,
        });
    }

    pub(crate) fn apply_cursor(&mut self, ctx: &egui::Context) {
        if self.pointer.system {
            ctx.set_cursor_image(None);
            ctx.set_cursor_icon(egui::CursorIcon::Default);
            return;
        }
        let now = ctx.input(|i| i.time);
        let kind = self.cursor_kind(ctx);
        let pin = self.pinned_pointer(ctx);
        let at = pin.or(self.pointer.live);
        let Some(at) = at else {
            return;
        };

        if let Some(pin) = pin {
            if let Some((ox, oy)) = client_origin(self.hwnd) {
                let screen = egui::pos2(ox as f32 + pin.x, oy as f32 + pin.y);
                self.pointer.pin(screen, now);
                set_cursor_pos(screen.x.round() as i32, screen.y.round() as i32);
            }
        }

        self.paint_pointer_debug(ctx, at);

        if self.cursors.is_blank() {
            return;
        }
        let layer = egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("rt_pointer"));
        self.cursors.paint(&ctx.layer_painter(layer), at, kind);

        ctx.set_cursor_image(None);
        ctx.set_cursor_icon(egui::CursorIcon::None);
    }

    pub(crate) fn cursor_kind(&self, ctx: &egui::Context) -> CursorKind {
        if let Some(kind) = self.pointer.forced_hint {
            return kind;
        }
        if let Some(kind) = self.drag_session_kind() {
            return kind;
        }
        let (left, right) = ctx.input(|i| (i.pointer.primary_down(), i.pointer.secondary_down()));
        if right {
            return CursorKind::Right;
        }
        if left {
            return CursorKind::Left;
        }
        if self.wheel_active(ctx) {
            return CursorKind::Wheel;
        }
        self.hover_kind()
    }

    fn drag_session_kind(&self) -> Option<CursorKind> {
        if let Some(kind) = self.win_mgr.session_kind() {
            return Some(kind);
        }

        if self.color_panel.win.dragging() || self.panes.iter().any(|pane| pane.view.pan.is_some())
        {
            return Some(CursorKind::Move);
        }
        if self.color_panel.drag != ColorDrag::None || self.source.editor.scroll_grab.is_some() {
            return Some(CursorKind::Slider);
        }

        if self.hsplit_grab.is_some() || self.vsplit_grab.is_some() {
            return Some(CursorKind::ResizeV);
        }
        None
    }

    fn wheel_active(&self, ctx: &egui::Context) -> bool {
        self.pointer.wheel_active(ctx)
    }

    fn hover_kind(&self) -> CursorKind {
        if let Some(kind) = self.win_mgr.hover_kind() {
            return kind;
        }
        if let Some(kind) = self.color_panel.win.hover_kind() {
            return kind;
        }
        self.pointer.hint
    }

    pub(crate) fn pinned_pointer(&self, ctx: &egui::Context) -> Option<egui::Pos2> {
        self.pointer.pinned_if_down(ctx)
    }
}

impl eframe::App for StockApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        theme().title_bg.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let t0 = Instant::now();

        self.tick_frame(&ctx);

        self.tooltip = None;

        if self.hwnd == 0 {
            if let Ok(h) = frame.window_handle() {
                self.hwnd = native_window_id(h.as_raw());
            }
        }
        if !self.client_cover_ready && install_client_cover(self.hwnd) {
            self.client_cover_ready = true;
        }

        self.draw_frame(ui);

        clear_window_frame(self.hwnd);

        self.frame_ms = t0.elapsed().as_secs_f32() * 1000.0;

        ctx.request_repaint_after(Duration::from_millis(IDLE_REPAINT_MS));

        if self.win_mgr.drag_active() || self.color_panel.win.dragging() {
            ctx.request_repaint();
        }
    }
}

impl StockApp {
    pub(crate) fn draw_frame(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();

        self.pointer.begin_frame();
        self.dismiss_popup_menus_on_keyboard(&ctx);

        let (scrolled, now) = ctx.input(|i| (i.time_since_last_scroll() < 0.01, i.time));
        if scrolled {
            self.pointer.wheel_until = now + WHEEL_HOLD_S;

            ctx.request_repaint_after(Duration::from_secs_f64(WHEEL_HOLD_S));
        }

        self.console.observe_status(&self.status_msg);

        let win_rect = ui.max_rect();
        let layout = egui::Rect::from_min_size(
            win_rect.min,
            win_rect.size().max(egui::vec2(WINDOW_MIN_W, WINDOW_MIN_H)),
        );

        self.advance_software_pointer(&ctx, layout);

        let areas = split_areas(layout);

        self.paint_skeleton(&ui.painter().clone(), layout, areas);
        self.ui_titlebar(ui, areas[0]);
        let menubar_state = self.menu.clone();
        let (open_anchor, menubar_state) =
            ui_menu_bar(self, ui, areas[1], &TOP_TABS, menubar_state);
        self.menu = menubar_state;

        self.work_rect = areas[2];
        let mut mgr = self.win_mgr.take();
        mgr.plan(self, ui, areas[2]);
        self.win_mgr = mgr;

        let plots = self.win_mgr.plots.clone();

        self.chart_interact(ui, &plots, &[]);

        for (pane, plot) in plots.iter().enumerate() {
            if plot.is_positive() {
                self.clamp_view(pane, plot.width());
            }
        }
        self.update_crosshair(ui, &plots);

        let mut mgr = self.win_mgr.take();
        mgr.draw(self, ui, areas[2]);
        self.win_mgr = mgr;

        let blank_right_click = ctx.input(|i| {
            (
                i.pointer.secondary_pressed(),
                i.pointer
                    .press_origin()
                    .or_else(|| i.pointer.interact_pos())
                    .or_else(|| i.pointer.latest_pos()),
            )
        });
        let context_hit = blank_right_click.1.is_some_and(|p| {
            self.context_menu
                .open()
                .is_some_and(|m| menu_rect(self, m.kind, m.anchor, layout).contains(p))
        });
        let floating_hit = self.color_panel.win.open
            && blank_right_click
                .1
                .is_some_and(|p| self.color_panel.win.rect().contains(p));
        if blank_right_click.0
            && let Some(p) = blank_right_click.1
            && self.work_rect.contains(p)
            && !context_hit
            && !floating_hit
            && !self
                .win_mgr
                .wins
                .iter()
                .enumerate()
                .any(|(i, w)| w.w.open && self.win_mgr.rect_of(i).contains(p))
        {
            self.context_menu.open_main(p);
        }

        self.paint_separators(&ui.painter().clone(), layout, areas);

        if let Some((idx, anchor)) = open_anchor {
            let items = TOP_TABS[idx].items;
            let mut state = self.menu.clone();

            if let Some(label) =
                ui_menu_popup(self, ui, anchor, items, &mut state, layout, &[areas[1]])
            {
                if TOP_TABS[idx].title == "布局" {
                    match label {
                        "控制面板" => self.request_win(WinCmd::OpenPanel),
                        "新增图表" => self.request_win(WinCmd::AddChart),
                        _ => self.status_msg = format!("你点击了：{label}"),
                    }
                } else {
                    self.status_msg = format!("你点击了：{label}");
                }
            }
            self.menu = state;
        }

        border_hard(&ui.painter().clone(), layout, theme().ctrl_border);

        self.ui_color_panel(ui, areas[2]);

        self.ui_pane_menu(ui);
        self.run_win_cmd();

        self.paint_tooltip(&ui.painter().clone(), layout);

        if let Some(action) = ui_context_menu(self, ui, layout) {
            self.run_context_action(ui, action);
        }

        self.apply_cursor(&ctx);
    }
}

pub(crate) fn configure_zero_spacing(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::Vec2::ZERO;
        style.spacing.window_margin = egui::Margin::ZERO;
        style.spacing.menu_margin = egui::Margin::ZERO;
        style.spacing.button_padding = egui::Vec2::ZERO;
        style.spacing.indent = 0.0;
        style.spacing.interact_size = egui::Vec2::ZERO;
        style.spacing.slider_width = 0.0;
        style.spacing.slider_rail_height = 0.0;
        style.spacing.combo_width = 0.0;
        style.spacing.combo_height = 0.0;
        style.spacing.text_edit_width = 0.0;
        style.spacing.menu_width = 0.0;
        style.spacing.menu_spacing = 0.0;
        style.spacing.tooltip_width = 0.0;
        style.spacing.extra_text_line_spacing = 0.0;
        style.spacing.icon_width = 0.0;
        style.spacing.icon_width_inner = 0.0;
        style.spacing.icon_spacing = 0.0;
        style.spacing.default_area_size = egui::Vec2::ZERO;

        style.visuals.window_corner_radius = egui::CornerRadius::ZERO;
        style.visuals.menu_corner_radius = egui::CornerRadius::ZERO;
        style.visuals.window_shadow = egui::Shadow::NONE;

        style.visuals.disabled_alpha = 1.0;
    });
}

pub(crate) fn configure_tessellation(ctx: &egui::Context) {
    ctx.tessellation_options_mut(|opt| {
        opt.feathering = false;
        opt.round_text_to_pixels = true;
        opt.round_line_segments_to_pixels = true;
        opt.round_rects_to_pixels = true;
    });
}
