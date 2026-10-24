#[cfg(test)]
use crate::domain::data;
use crate::domain::indicator;
use crate::domain::layout::*;
use crate::engine::font::*;
use crate::engine::kit;
use crate::engine::theme::*;
use crate::platform::pointer::*;
use crate::platform::*;
use crate::ui::colorpanel::*;
use crate::ui::console::*;
use crate::ui::contextmenu::*;
use crate::ui::floatwin::WIN_FRAME;
use crate::ui::icon::*;
use crate::ui::menu::*;
use crate::ui::panel::*;
use crate::ui::source::*;
pub(crate) use crate::ui::symbol::*;
use crate::ui::titlebar::*;
use crate::ui::winmgr::*;
use std::time::Instant;

pub(crate) const APP_NAME: &str = "RoundTable";

pub(crate) const WHEEL_HOLD_S: f64 = 0.3;

pub(crate) const CARET_BLINK_PERIOD_S: f64 = 1.0;
pub(crate) const CARET_BLINK_HALF_S: f64 = CARET_BLINK_PERIOD_S * 0.5;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(crate) enum FrameRateCap {
    Fps60,
    Fps120,
    Fps240,
    #[default]
    Unlimited,
}

impl FrameRateCap {
    pub(crate) fn fps(self) -> Option<u32> {
        match self {
            Self::Fps60 => Some(60),
            Self::Fps120 => Some(120),
            Self::Fps240 => Some(240),
            Self::Unlimited => None,
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            Self::Fps60 => Self::Fps120,
            Self::Fps120 => Self::Fps240,
            Self::Fps240 => Self::Unlimited,
            Self::Unlimited => Self::Fps60,
        }
    }

    pub(crate) fn label(self) -> String {
        match self.fps() {
            Some(fps) => format!("FPS {fps}"),
            None => "FPS Unlimited".to_owned(),
        }
    }
}

pub(crate) fn caret_blink_visible_at(elapsed: f64) -> bool {
    elapsed.rem_euclid(CARET_BLINK_PERIOD_S) < CARET_BLINK_PERIOD_S * 0.5
}

#[derive(Clone, Copy)]
pub(crate) enum WinCmd {
    AddChart,

    OpenPanel,

    OpenColor(ColorTarget),

    OpenSource(usize),

    CloseSource,

    OpenConsole,

    CloseConsole,
}

pub(crate) use crate::domain::chart::{ChartWorld, Crosshair, Pane, PaneCmds};

#[derive(Clone, Default)]
pub(crate) struct DebugWindowSnapshot {
    pub(crate) windows: Vec<(String, bool, kit::Rect)>,
    pub(crate) window_count: usize,
    pub(crate) chart_count: usize,
    pub(crate) source_open: bool,
    pub(crate) console_open: bool,
}

pub(crate) const PANE_CANDLE: usize = 0;

#[cfg(test)]
pub(crate) const PANE_VOLUME: usize = 1;
#[cfg(test)]
pub(crate) const PANE_MACD: usize = 2;

impl AppState {
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
        let rgb = |c: kit::Color32| (c.r(), c.g(), c.b());

        let six = |colors: [kit::Color32; 6]| {
            let mut out = [0u32; 6];
            for (i, c) in colors.iter().enumerate() {
                out[i] = indicator::rgb_u32(rgb(*c));
            }
            out
        };
        indicator::Palette {
            candle_up: indicator::rgb_u32(self.chart_ui.up_color),
            candle_down: indicator::rgb_u32(self.chart_ui.down_color),
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
        if self.chart_world.panes_palette == want {
            return;
        }
        for pane in &mut self.chart_world.panes {
            pane.indicator.set_palette(want);
            pane.dirty = true;
        }
        self.chart_world.panes_palette = want;
    }

    pub(crate) fn run_context_action(&mut self, ui: &kit::Ui, action: ContextAction) {
        match action {
            ContextAction::AddChart => self.request_win(WinCmd::AddChart),
            ContextAction::OpenConsole => self.request_win(WinCmd::OpenConsole),
            ContextAction::DemoOne => {
                self.shell.status_msg = "Context menu: Demo Action One".to_owned()
            }
            ContextAction::DemoTwo => {
                self.shell.status_msg = "Context menu: Demo Action Two".to_owned()
            }
            ContextAction::Cut
            | ContextAction::Copy
            | ContextAction::Paste
            | ContextAction::SelectAll
            | ContextAction::Undo => {
                self.workbench
                    .source
                    .editor
                    .apply_context_action(ui, action);
            }
        }
    }

    pub(crate) fn adopt_theme_candle_colors(&mut self, t: Theme) {
        let (u, d) = (t.candle_up, t.candle_down);
        self.chart_ui.up_color = (u.r(), u.g(), u.b());
        self.chart_ui.down_color = (d.r(), d.g(), d.b());
    }

    pub(crate) fn follow_system_chart_colors(&mut self) {
        self.chart_ui.chart_follows_theme = true;
        let t = theme();
        self.adopt_theme_candle_colors(t);
        self.mark_panes_dirty();
        self.shell.status_msg = format!("Chart colors follow the active theme ({})", t.name());
    }

    pub(crate) fn ui_drag_active(&self) -> bool {
        self.chart_ui.hsplit_grab.is_some()
            || self.chart_ui.vsplit_grab.is_some()
            || self.chart_ui.window_drag_active
            || self.shell.color_panel.drag != ColorDrag::None
    }

    pub(crate) fn mark_panes_dirty(&mut self) {
        self.chart_world.mark_panes_dirty();
    }

    #[cfg(test)]
    pub(crate) fn replace_bars(&mut self, bars: Vec<data::random_walk::Bar>) {
        self.chart_world.bars = bars;
    }

    pub(crate) fn ensure_pane_computed(&mut self, pane: usize) {
        self.chart_world.ensure_pane_computed(pane);
    }

    pub(crate) fn request_win(&mut self, cmd: WinCmd) {
        self.shell.win_cmd = Some(cmd);
        self.stats.repaint_requested = true;
    }

    fn report_spawn_blocked(&mut self) {
        let text = "No room left to open another window".to_owned();
        self.shell.status_msg = text.clone();
        self.shell.tooltip = Some((
            kit::Rect::from_min_size(self.chart_ui.work_rect.center(), kit::vec2(LINE_W, LINE_W)),
            text,
        ));
    }
}

impl StockApp {
    pub(crate) fn sync_window_interaction_cache(&mut self) {
        self.state.chart_ui.focused_pane = self.win_mgr.focused_pane();
        self.state.chart_ui.active_pane = self.win_mgr.top_chart_pane();
        self.state.chart_ui.window_drag_active = self.win_mgr.drag_active();
        self.state.chart_ui.color_rect = self.win_mgr.color_rect();
    }

    pub(crate) fn plan_windows(&mut self, ui: &mut kit::Ui, work: kit::Rect, taskbar: kit::Rect) {
        self.win_mgr.plan(&mut self.state, ui, work, taskbar);
    }

    pub(crate) fn draw_windows(&mut self, ui: &mut kit::Ui, work: kit::Rect, taskbar: kit::Rect) {
        self.win_mgr.draw(&mut self.state, ui, work, taskbar);
    }

    pub(crate) fn run_win_cmd(&mut self) {
        match self.state.shell.win_cmd.take() {
            None => return,
            Some(WinCmd::AddChart) => self.add_chart_window(),
            Some(WinCmd::OpenPanel) => self.open_panel_window(),
            Some(WinCmd::OpenColor(target)) => self.open_color_panel(target),
            Some(WinCmd::OpenSource(pane)) => self.open_source_window(pane),
            Some(WinCmd::CloseSource) => self.close_source(),
            Some(WinCmd::OpenConsole) => self.open_console_window(),
            Some(WinCmd::CloseConsole) => self.close_console_window(),
        }
        self.state.stats.repaint_requested = true;
    }

    pub(crate) fn add_chart_window(&mut self) {
        if self.win_mgr.chart_count() >= MAX_CHART_WINS {
            self.state.shell.status_msg = format!("At most {MAX_CHART_WINS} chart windows");
            return;
        }
        let size = kit::vec2(
            NEW_WIN_W.min(self.state.chart_ui.work_rect.width()),
            NEW_WIN_H.min(self.state.chart_ui.work_rect.height()),
        );
        if !self
            .win_mgr
            .can_spawn_window(self.state.chart_ui.work_rect, size)
        {
            self.state.report_spawn_blocked();
            return;
        }
        let pane = self.new_chart_pane();
        if self
            .win_mgr
            .add_chart(self.state.chart_ui.work_rect, pane)
            .is_none()
        {
            self.state.report_spawn_blocked();
            return;
        }
        self.state.shell.status_msg = format!("Chart windows open: {}", self.win_mgr.chart_count());
    }

    pub(crate) fn new_chart_pane(&mut self) -> usize {
        let mut ind = indicator::default_indicator();
        ind.set_palette(self.state.current_palette());
        match self.win_mgr.free_chart_pane() {
            Some(free) => {
                self.state.chart_world.replace_pane(free, Pane::new(ind));
                free
            }
            None => self.state.chart_world.add_pane(Pane::new(ind)),
        }
    }

    pub(crate) fn open_panel_window(&mut self) {
        if self.win_mgr.wins.iter().any(|w| w.kind == WinKind::Panel) {
            self.win_mgr
                .place_panel_at_default(self.state.chart_ui.work_rect);
            self.state.shell.status_msg = "Control Panel restored".to_owned();
        }
    }

    pub(crate) fn open_console_window(&mut self) {
        let size = kit::vec2(
            CONSOLE_W.min(self.state.chart_ui.work_rect.width()),
            CONSOLE_H.min(self.state.chart_ui.work_rect.height()),
        );
        if !self.win_mgr.console_open()
            && !self
                .win_mgr
                .can_spawn_window(self.state.chart_ui.work_rect, size)
        {
            self.state.report_spawn_blocked();
            return;
        }
        if self
            .win_mgr
            .show_console(self.state.chart_ui.work_rect)
            .is_none()
        {
            self.state.report_spawn_blocked();
            return;
        }
        self.state.workbench.console.log("Console window opened.");
        self.state.shell.status_msg = "Debug Console: Opened".to_owned();
    }

    pub(crate) fn close_console_window(&mut self) {
        self.win_mgr.hide_console();
        self.state.shell.status_msg = "Debug Console: Closed".to_owned();
    }

    pub(crate) fn update_panel_min_size(&mut self) {
        let min = self.state.panel_min_size() + kit::vec2(2.0 * WIN_FRAME, 2.0 * WIN_FRAME);
        for window in &mut self.win_mgr.wins {
            if window.kind == WinKind::Panel {
                window.w.min_size = min;
            }
        }
    }
}

pub(crate) struct AppState {
    pub(crate) chart_world: ChartWorld,
    pub(crate) chart_ui: ChartUiState,
    pub(crate) hwnd: isize,
    pub(crate) assets: AppAssets,
    pub(crate) pointer: PointerManager,
    pub(crate) stats: FrameStats,
    pub(crate) shell: ShellState,
    pub(crate) workbench: WorkbenchState,
}

pub(crate) struct StockApp {
    pub(crate) state: AppState,
    pub(crate) win_mgr: WinMgr,
}

impl std::ops::Deref for StockApp {
    type Target = AppState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl std::ops::DerefMut for StockApp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

pub(crate) struct ChartUiState {
    #[cfg(test)]
    pub(crate) kline_h: f32,
    #[cfg(test)]
    pub(crate) vol_h: f32,
    #[cfg(test)]
    pub(crate) ind_h: f32,
    pub(crate) up_color: (u8, u8, u8),
    pub(crate) down_color: (u8, u8, u8),
    pub(crate) chart_follows_theme: bool,
    pub(crate) hsplit_grab: Option<f32>,
    pub(crate) vsplit_grab: Option<f32>,
    #[cfg(test)]
    pub(crate) split_x: f32,
    pub(crate) work_rect: kit::Rect,
    pub(crate) visible_scratch: Vec<u32>,
    pub(crate) focused_pane: Option<usize>,
    pub(crate) active_pane: Option<usize>,
    pub(crate) window_drag_active: bool,
    pub(crate) color_rect: Option<kit::Rect>,
}

pub(crate) struct FrameStats {
    pub(crate) frames: u64,
    pub(crate) last_frame_time: f64,
    pub(crate) caret_blink_epoch: f64,
    pub(crate) frame_interval_ms: f32,
    pub(crate) frame_ms: f32,
    pub(crate) fps: f32,
    pub(crate) frame_rate_cap: FrameRateCap,
    pub(crate) caret_active: bool,
    pub(crate) repaint_requested: bool,
}

impl Default for FrameStats {
    fn default() -> Self {
        Self {
            frames: 0,
            last_frame_time: 0.0,
            caret_blink_epoch: 0.0,
            frame_interval_ms: 0.0,
            frame_ms: 0.0,
            fps: 0.0,
            frame_rate_cap: FrameRateCap::default(),
            caret_active: false,
            repaint_requested: false,
        }
    }
}

impl FrameStats {
    fn tick(&mut self, ctx: &kit::Context) {
        let now = ctx.input(|input| input.time);
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

    fn reset_caret_blink(&mut self, ctx: &kit::Context) {
        self.caret_blink_epoch = ctx.input(|input| input.time);
    }

    fn caret_visible(&self, ctx: &kit::Context) -> bool {
        let now = ctx.input(|input| input.time);
        let elapsed = (now - self.caret_blink_epoch).max(0.0);
        caret_blink_visible_at(elapsed)
    }

    fn next_caret_change(&self) -> f64 {
        let elapsed = (self.last_frame_time - self.caret_blink_epoch).max(0.0);
        let half = (elapsed / CARET_BLINK_HALF_S).floor() + 1.0;
        self.caret_blink_epoch + half * CARET_BLINK_HALF_S
    }

    fn frame_interval_s(&self) -> Option<f64> {
        self.frame_rate_cap.fps().map(|fps| 1.0 / fps.max(1) as f64)
    }

    fn fps_segment(&self) -> String {
        format!("{:.0} FPS", self.fps)
    }
}

pub(crate) struct AppAssets {
    #[cfg(test)]
    pub(crate) icons: Box<IconSet>,
    pub(crate) symbols: Box<SymbolSet>,
    pub(crate) cursors: Box<Cursors>,
    pub(crate) text_engine: Option<HardText>,
}

pub(crate) struct ShellState {
    pub(crate) tooltip: Option<(kit::Rect, String)>,
    pub(crate) menu: MenuState,
    pub(crate) status_msg: String,
    pub(crate) win_cmd: Option<WinCmd>,
    pub(crate) panel_buttons: Vec<PanelButton>,
    pub(crate) panel_press: Option<usize>,
    pub(crate) color_panel: ColorPanel,
    pub(crate) win_silent: bool,
    pub(crate) context_menu: ContextMenuState,
    pub(crate) pane_menu: Option<usize>,
    pub(crate) pane_menu_tab: usize,
    pub(crate) pane_menu_anchor: kit::Rect,
    pub(crate) pane_menu_bar: kit::Rect,
}

impl Default for ShellState {
    fn default() -> Self {
        Self {
            tooltip: None,
            menu: MenuState::default(),
            status_msg: String::new(),
            win_cmd: None,
            panel_buttons: Vec::new(),
            panel_press: None,
            color_panel: ColorPanel::new(),
            win_silent: false,
            context_menu: ContextMenuState::default(),
            pane_menu: None,
            pane_menu_tab: 0,
            pane_menu_anchor: kit::Rect::NOTHING,
            pane_menu_bar: kit::Rect::NOTHING,
        }
    }
}

pub(crate) struct WorkbenchState {
    pub(crate) source: SourceView,
    pub(crate) console: DebugConsole,
    pub(crate) debug_windows: DebugWindowSnapshot,
}

impl StockApp {
    pub(crate) fn new(ctx: &kit::Context, hwnd: isize) -> Self {
        configure_tessellation(ctx);
        configure_zero_spacing(ctx);

        enter_fullscreen(hwnd);
        let palette = indicator::Palette::default();
        let chart_world = ChartWorld::new(AppState::build_panes(palette), palette);
        let (loaded_theme, symbols, cursors, cjk_font_resource) = std::thread::scope(|scope| {
            let loaded_theme = scope.spawn(|| Theme::load(ThemeId::Classic));
            let symbols = scope.spawn(|| Box::new(SymbolSet::load()));
            let cursors = scope.spawn(|| Box::new(Cursors::load()));
            let fonts = scope.spawn(preload_font_files);
            (
                loaded_theme
                    .join()
                    .unwrap_or_else(|payload| std::panic::resume_unwind(payload)),
                symbols
                    .join()
                    .unwrap_or_else(|payload| std::panic::resume_unwind(payload)),
                cursors
                    .join()
                    .unwrap_or_else(|payload| std::panic::resume_unwind(payload)),
                fonts
                    .join()
                    .unwrap_or_else(|payload| std::panic::resume_unwind(payload)),
            )
        });
        #[cfg(test)]
        let icons = Box::new(IconSet::load_or_create());
        set_theme(loaded_theme);

        let mut state = AppState {
            chart_world,
            chart_ui: ChartUiState {
                #[cfg(test)]
                kline_h: KLINE_H_DEFAULT,
                #[cfg(test)]
                vol_h: VOL_H_DEFAULT,
                #[cfg(test)]
                ind_h: IND_H_DEFAULT,
                up_color: UP_RGB_DEFAULT,
                down_color: DOWN_RGB_DEFAULT,
                chart_follows_theme: true,
                hsplit_grab: None,
                vsplit_grab: None,
                #[cfg(test)]
                split_x: CHART_PANE_W,
                work_rect: kit::Rect::NOTHING,
                visible_scratch: Vec::new(),
                focused_pane: None,
                active_pane: None,
                window_drag_active: false,
                color_rect: None,
            },
            hwnd,
            assets: AppAssets {
                #[cfg(test)]
                icons,
                symbols,
                cursors,
                text_engine: HardText::new_after_font_preload(ctx, cjk_font_resource),
            },
            pointer: PointerManager::new(mouse_feel()),
            stats: FrameStats::default(),
            shell: ShellState::default(),
            workbench: WorkbenchState {
                source: SourceView::new(),
                console: DebugConsole::new(),
                debug_windows: DebugWindowSnapshot::default(),
            },
        };

        if state.chart_ui.chart_follows_theme {
            state.adopt_theme_candle_colors(theme());
        }

        state.sync_pane_palette();

        let mut runtime = Self {
            state,
            win_mgr: WinMgr::with_default_windows(),
        };
        runtime.build_panel_buttons();
        runtime
    }
}

impl AppState {
    pub(crate) fn reset_caret_blink(&mut self, ctx: &kit::Context) {
        self.stats.reset_caret_blink(ctx);
    }

    pub(crate) fn note_caret(&mut self) {
        self.stats.caret_active = true;
    }

    pub(crate) fn caret_visible(&self, ctx: &kit::Context) -> bool {
        self.stats.caret_visible(ctx)
    }

    pub(crate) fn status_segments(&self) -> Vec<String> {
        let mut segs = vec![current_time_text()];

        if !self.shell.status_msg.is_empty() {
            segs.push(self.shell.status_msg.clone());
        }
        segs
    }

    pub(crate) fn fps_segment(&self) -> String {
        self.stats.fps_segment()
    }

    fn dismiss_popup_menus_on_keyboard(&mut self, ctx: &kit::Context) {
        let pressed = ctx.input(|i| {
            i.events
                .iter()
                .any(|event| matches!(event, kit::Event::Key { pressed: true, .. }))
        });
        if !pressed {
            return;
        }
        self.shell.menu = MenuState::default();
        self.shell.pane_menu = None;
        self.workbench.source.editor.menu = MenuState::default();
        self.shell.context_menu.close();
    }

    pub(crate) fn chart_status_segments(&self, pane: usize) -> Vec<String> {
        let view = &self.chart_world.panes[pane].view;
        vec![
            format!("Zoom {}", view.scale),
            format!("Pan {}", view.offset),
            format!("Visible {} / {}", view.visible, self.chart_world.bars.len()),
        ]
    }

    pub(crate) fn active_chart_pane(&self) -> Option<usize> {
        self.chart_ui.active_pane
    }

    pub(crate) fn paint_status_bar(
        &mut self,
        painter: &kit::Painter,
        band: kit::Rect,
        segs: &[String],
    ) {
        self.paint_status_bar_on_layer(painter, band, segs, kit::PaintLayer::Dynamic);
    }

    pub(crate) fn paint_status_bar_on_layer(
        &mut self,
        painter: &kit::Painter,
        band: kit::Rect,
        segs: &[String],
        layer: kit::PaintLayer,
    ) {
        let background = theme().status_bg;
        let text = theme().status_text;
        let border = theme().status_border;
        painter.rect_filled(band, kit::CornerRadius::ZERO, background);
        hline_hard(painter, band.min.x, band.max.x, band.min.y, border);

        let fps = self.fps_segment();
        let fps_w = snap_cells_up(self.text_width(&fps) + 2.0 * TEXT_PAD_X);
        let fps_min = (band.max.x - fps_w).max(band.min.x);
        let mut x = band.min.x;
        painter.with_layer(layer, || {
            for (i, seg) in segs.iter().enumerate() {
                let w = snap_cells_up(self.text_width(seg) + 2.0 * TEXT_PAD_X);
                if x + w > fps_min {
                    break;
                }
                if i > 0 {
                    vline_hard(painter, x, band.min.y + LINE_W, band.max.y - LINE_W, border);
                }
                self.text(
                    painter,
                    kit::pos2(text_left_x(x, band.min.x), band.center().y),
                    kit::Align2::LEFT_CENTER,
                    seg,
                    text,
                );
                x += w - LINE_W;
            }
            if fps_min > band.min.x {
                vline_hard(
                    painter,
                    fps_min,
                    band.min.y + LINE_W,
                    band.max.y - LINE_W,
                    border,
                );
            }
            self.text(
                painter,
                kit::pos2(fps_min + fps_w / 2.0, band.center().y),
                kit::Align2::CENTER_CENTER,
                &fps,
                text,
            );
        });
    }

    pub(crate) fn paint_status_bar_colors(
        &mut self,
        painter: &kit::Painter,
        band: kit::Rect,
        segs: &[String],
        colors: (kit::Color32, kit::Color32, kit::Color32),
        text_layer: kit::PaintLayer,
    ) {
        let (background, text, border) = colors;
        painter.rect_filled(band, kit::CornerRadius::ZERO, background);
        hline_hard(painter, band.min.x, band.max.x, band.min.y, border);
        let mut x = band.min.x;
        let mut first = true;
        painter.with_layer(text_layer, || {
            for seg in segs {
                let w = snap_cells_up(self.text_width(seg) + 2.0 * TEXT_PAD_X);
                if x + w > band.max.x {
                    break;
                }
                if !first {
                    vline_hard(painter, x, band.min.y + LINE_W, band.max.y - LINE_W, border);
                }
                self.text(
                    painter,
                    kit::pos2(text_left_x(x, band.min.x), band.center().y),
                    kit::Align2::LEFT_CENTER,
                    seg,
                    text,
                );
                x += w - LINE_W;
                first = false;
            }
        });
        if !first {
            vline_hard(painter, x, band.min.y + LINE_W, band.max.y - LINE_W, border);
            let right = band.max.x - LINE_W;
            if right > x {
                vline_hard(
                    painter,
                    right,
                    band.min.y + LINE_W,
                    band.max.y - LINE_W,
                    border,
                );
            }
        }
    }

    pub(crate) fn text(
        &mut self,
        painter: &kit::Painter,
        pos: kit::Pos2,
        align: kit::Align2,
        s: &str,
        color: kit::Color32,
    ) {
        if let Some(engine) = &mut self.assets.text_engine {
            engine.draw(painter, pos, align, s, color);
        }
    }

    pub(crate) fn text_width(&mut self, s: &str) -> f32 {
        self.assets
            .text_engine
            .as_mut()
            .map_or(0.0, |engine| engine.width(s) as f32)
    }

    pub(crate) fn paint_skeleton(
        &mut self,
        painter: &kit::Painter,
        layout: kit::Rect,
        areas: [kit::Rect; 5],
    ) {
        painter.rect_filled(layout, kit::CornerRadius::ZERO, theme().work_bg);
        let [title, menu, work, taskbar, status] = areas;
        painter.rect_filled(title, kit::CornerRadius::ZERO, theme().title_bg);
        let name_slot = title_text_slot(self, title, APP_NAME);
        paint_title_text_edge(painter, name_slot);
        painter.rect_filled(menu, kit::CornerRadius::ZERO, theme().menu_bg);

        painter.rect_filled(work, kit::CornerRadius::ZERO, theme().desktop_bg);
        painter.rect_filled(taskbar, kit::CornerRadius::ZERO, theme().taskbar_bg);
        painter.rect_filled(status, kit::CornerRadius::ZERO, theme().status_bg);

        let segs = self.status_segments();
        self.paint_status_bar(painter, status, &segs);
    }

    pub(crate) fn paint_separators(
        &self,
        painter: &kit::Painter,
        _layout: kit::Rect,
        areas: [kit::Rect; 5],
    ) {
        let left = areas[0].min.x;
        let right = areas[0].max.x;
        for y in [
            areas[1].min.y,
            areas[2].min.y,
            areas[3].min.y,
            areas[4].min.y,
        ] {
            hline_hard(painter, left, right, y, theme().menu_separator);
        }
    }

    pub(crate) fn floating_at(&self, p: kit::Pos2, window_bottom: f32) -> bool {
        if self.popup_at(p, window_bottom) {
            return true;
        }
        self.chart_ui
            .color_rect
            .is_some_and(|rect| rect.contains(p))
    }

    pub(crate) fn popup_at(&self, p: kit::Pos2, window_bottom: f32) -> bool {
        if self.shell.pane_menu.is_some() {
            let count = self.pane_menu_count();
            if self
                .pane_menu_rect(count, window_bottom)
                .is_some_and(|r| r.expand(grid_line(1)).contains(p))
            {
                return true;
            }
        }
        false
    }

    fn advance_software_pointer(&mut self, ctx: &kit::Context, layout: kit::Rect) {
        let now = ctx.input(|i| i.time);
        let locked = self.pointer.pinned_if_down(ctx);

        if self.pointer.system {
            let live = ctx
                .input(|input| input.pointer.latest_pos())
                .unwrap_or(layout.center());
            self.pointer.sync(live, now);
            self.pointer.live = Some(live);
            self.pointer.motion_live = Some(live);
            return;
        }
        if !self.pointer.is_initialized() {
            let start = ctx
                .input(|input| input.pointer.latest_pos())
                .unwrap_or(layout.center());
            self.pointer.reset(start, now);
        }

        let (_moving, at) = if locked.is_some() {
            let (raw_moving, at) = self.pointer.advance_events(ctx, now, layout, true);
            if raw_moving {
                (true, at)
            } else if self.hwnd != 0 {
                let before = self.pointer.position();
                let delta = cursor_pos()
                    .map(|(x, y)| {
                        self.pointer
                            .take_native_motion(kit::pos2(x as f32, y as f32))
                    })
                    .unwrap_or(kit::Vec2::ZERO);
                if delta == kit::Vec2::ZERO {
                    (false, before)
                } else {
                    let at = self.pointer.advance_locked(&[delta], now, layout);
                    (self.pointer.position() != before, at)
                }
            } else {
                let before = self.pointer.position();
                if let Some(p) = ctx.input(|i| i.pointer.latest_pos()) {
                    self.pointer.sync(p, now);
                }
                (self.pointer.position() != before, self.pointer.position())
            }
        } else {
            let before = self.pointer.position();
            if let Some(position) = ctx.input(|input| input.pointer.latest_pos()) {
                self.pointer.follow_events(ctx, position, now);
            }
            (self.pointer.position() != before, self.pointer.position())
        };

        self.pointer.motion_live = Some(if self.hwnd != 0 {
            kit::pos2(at.x.round(), at.y.round())
        } else {
            at
        });
        if let Some(pin) = locked
            && let Some((ox, oy)) = client_origin(self.hwnd)
        {
            let screen = kit::pos2(ox as f32 + pin.x, oy as f32 + pin.y);
            if self.pointer.native_cursor_needs_move(screen)
                && set_cursor_pos(screen.x as i32, screen.y as i32)
            {
                self.pointer.mark_native_cursor(screen);
            }
        }
        self.pointer.live = self.pointer.motion_live;
    }

    pub(crate) fn paint_cursor(&mut self, ctx: &kit::Context, kind: CursorKind) {
        let now = ctx.input(|i| i.time);
        let pin = self.pinned_pointer(ctx);
        let at = pin.or(self.pointer.live);
        let Some(at) = at else {
            return;
        };

        if let Some(pin) = pin {
            if let Some((ox, oy)) = client_origin(self.hwnd) {
                let screen = kit::pos2(ox as f32 + pin.x, oy as f32 + pin.y);
                self.pointer.pin(screen, now);
                if self.pointer.native_cursor_needs_move(screen)
                    && set_cursor_pos(screen.x.round() as i32, screen.y.round() as i32)
                {
                    self.pointer.mark_native_cursor(screen);
                }
            }
        }

        ctx.with_layer(kit::PaintLayer::Dynamic, || {
            self.paint_pointer_debug(ctx, at)
        });

        if pin.is_none() || self.pointer.system || self.assets.cursors.is_blank() {
            ctx.set_cursor_icon(kind.system_icon());
            return;
        }
        let layer = kit::LayerId::new(kit::Order::Tooltip, kit::Id::new("rt_pointer"));
        ctx.with_layer(kit::PaintLayer::Dynamic, || {
            self.assets
                .cursors
                .paint(&ctx.layer_painter(layer), at, kind);
        });

        ctx.set_cursor_icon(kit::CursorIcon::None);
    }

    pub(crate) fn settle_pointer_pin(&mut self, pin: kit::Pos2, time: f64) {
        self.pointer.pinned = Some(pin);
        let screen = match client_origin(self.hwnd) {
            Some((ox, oy)) => kit::pos2(ox as f32 + pin.x, oy as f32 + pin.y),
            None => pin,
        };
        self.pointer.pin(screen, time);
    }

    fn drag_session_kind(&self) -> Option<CursorKind> {
        if self
            .chart_world
            .panes
            .iter()
            .any(|pane| pane.view.pan.is_some())
        {
            return Some(CursorKind::Move);
        }
        if self.pointer.text_selecting || self.workbench.source.editor.selecting {
            return Some(CursorKind::Text);
        }
        if self.shell.color_panel.drag != ColorDrag::None
            || self.workbench.source.editor.scroll_grab.is_some()
        {
            return Some(CursorKind::Slider);
        }

        if self.chart_ui.hsplit_grab.is_some() || self.chart_ui.vsplit_grab.is_some() {
            return Some(CursorKind::ResizeV);
        }
        None
    }

    pub(crate) fn pinned_pointer(&self, ctx: &kit::Context) -> Option<kit::Pos2> {
        self.pointer.pinned_if_down(ctx)
    }
}

impl StockApp {
    pub(crate) fn cursor_kind(&self, ctx: &kit::Context) -> CursorKind {
        if let Some(kind) = self.state.pointer.forced_hint {
            return kind;
        }
        let (left, right) = ctx.input(|i| (i.pointer.primary_down(), i.pointer.secondary_down()));
        if right {
            return CursorKind::Right;
        }
        if self.state.pointer.wheel_active(ctx) {
            return CursorKind::Wheel;
        }
        if self.state.pointer.text_hover {
            return CursorKind::Text;
        }
        if let Some(kind) = self.win_mgr.session_kind() {
            return kind;
        }
        if let Some(kind) = self.state.drag_session_kind() {
            return kind;
        }
        if left {
            return CursorKind::Left;
        }
        self.win_mgr.hover_kind().unwrap_or(self.state.pointer.hint)
    }

    pub(crate) fn apply_cursor(&mut self, ctx: &kit::Context) {
        let kind = self.cursor_kind(ctx);
        self.state.paint_cursor(ctx, kind);
    }

    pub(crate) fn lock_pointer_events(&mut self, raw_input: &mut kit::RawInput) {
        self.state.pointer.lock_events(&mut raw_input.events);
    }

    pub(crate) fn ui(&mut self, ui: &mut kit::Ui) {
        let ctx = ui.ctx().clone();
        let t0 = Instant::now();

        self.state.stats.tick(&ctx);
        self.state.shell.tooltip = None;

        self.draw_frame(ui);

        self.state.stats.frame_ms = t0.elapsed().as_secs_f32() * 1000.0;

        let _ = &ctx;
    }
}

impl StockApp {
    pub(crate) fn draw_frame(&mut self, ui: &mut kit::Ui) {
        let ctx = ui.ctx().clone();
        self.stats.caret_active = false;
        self.stats.repaint_requested = false;

        self.pointer.begin_frame(&ctx);
        self.dismiss_popup_menus_on_keyboard(&ctx);
        if ctx.input(|i| i.pointer.secondary_pressed()) {
            self.shell.menu = MenuState::default();
            self.shell.pane_menu = None;
            self.workbench.source.editor.menu = MenuState::default();
        }

        let (scrolled, now) = ctx.input(|i| (i.time_since_last_scroll() < 0.01, i.time));
        if scrolled {
            self.pointer.wheel_until = now + WHEEL_HOLD_S;
        }

        let status = self.shell.status_msg.clone();
        self.workbench.console.observe_status(&status);

        let win_rect = ui.max_rect();
        let layout = kit::Rect::from_min_size(
            win_rect.min,
            win_rect.size().max(kit::vec2(WINDOW_MIN_W, WINDOW_MIN_H)),
        );

        self.advance_software_pointer(&ctx, layout);

        let areas = split_areas(layout);
        let content = kit::Rect::from_min_max(areas[0].min, areas[4].max);

        self.paint_skeleton(&ui.painter().clone(), layout, areas);
        self.ui_titlebar(ui, areas[0]);
        let menubar_state = std::mem::take(&mut self.shell.menu);
        let (open_anchor, menubar_state) =
            ui_menu_bar(self, ui, areas[1], &TOP_TABS, menubar_state);
        self.shell.menu = menubar_state;

        self.chart_ui.work_rect = areas[2];
        self.plan_windows(ui, areas[2], areas[3]);

        let plots = std::mem::take(&mut self.win_mgr.plots);

        self.chart_interact(ui, &plots, &[]);

        for (pane, plot) in plots.iter().enumerate() {
            if plot.is_positive() {
                self.clamp_view(pane, plot.width());
            }
        }
        self.update_crosshair(ui, &plots);
        self.win_mgr.plots = plots;

        self.draw_windows(ui, areas[2], areas[3]);

        let blank_right_click = ctx.input(|i| {
            (
                i.pointer.secondary_pressed(),
                i.pointer.interact_pos().or_else(|| i.pointer.latest_pos()),
            )
        });
        let context_hit = blank_right_click.1.is_some_and(|p| {
            self.shell
                .context_menu
                .open()
                .is_some_and(|m| menu_rect(self, m.kind, m.anchor, layout).contains(p))
        });
        let floating_hit = blank_right_click.1.is_some_and(|p| {
            self.win_mgr
                .color_rect()
                .is_some_and(|rect| rect.contains(p))
        });
        if blank_right_click.0
            && let Some(p) = blank_right_click.1
            && self.chart_ui.work_rect.contains(p)
            && !context_hit
            && !floating_hit
            && !self
                .win_mgr
                .wins
                .iter()
                .enumerate()
                .any(|(i, w)| w.w.open && self.win_mgr.rect_of(i).contains(p))
        {
            self.shell.context_menu.open_main(p);
        }

        self.paint_separators(&ui.painter().clone(), layout, areas);

        if let Some((idx, anchor)) = open_anchor {
            let items = TOP_TABS[idx].items;
            let mut state = std::mem::take(&mut self.shell.menu);

            let ctx = ui.ctx().clone();
            let selected = ctx.with_layer(kit::PaintLayer::Dynamic, || {
                ui_menu_popup(self, ui, anchor, items, &mut state, content, &[areas[1]])
            });
            if let Some(label) = selected {
                if TOP_TABS[idx].title == "Layout" {
                    match label {
                        "Control Panel" => self.request_win(WinCmd::OpenPanel),
                        "New Chart" => self.request_win(WinCmd::AddChart),
                        _ => self.shell.status_msg = format!("Selected: {label}"),
                    }
                } else {
                    self.shell.status_msg = format!("Selected: {label}");
                }
            }
            self.shell.menu = state;
        }

        let ctx = ui.ctx().clone();
        ctx.with_layer(kit::PaintLayer::Dynamic, || self.ui_pane_menu(ui));
        self.run_win_cmd();

        self.paint_tooltip(&ctx, layout);

        let ctx = ui.ctx().clone();
        let context_action = ctx.with_layer(kit::PaintLayer::Dynamic, || {
            ui_context_menu(self, ui, content)
        });
        if let Some(action) = context_action {
            self.run_context_action(ui, action);
        }
        if ctx.input(|i| i.pointer.primary_pressed() || i.pointer.secondary_pressed()) {
            self.stats.repaint_requested = true;
        }

        border_hard(&ui.painter().clone(), content, theme().window_border);
        border_hard(&ui.painter().clone(), layout, theme().window_border);

        self.apply_cursor(&ctx);
    }

    pub(crate) fn next_repaint_at(&self, now: f64) -> f64 {
        let mut next = now + seconds_to_next_wall_second();
        if self.stats.caret_active {
            next = next.min(self.stats.next_caret_change());
        }
        if self.pointer.wheel_until > now {
            next = next.min(self.pointer.wheel_until);
        }
        if self.stats.repaint_requested || self.continuous_repaint() {
            let paced = self
                .stats
                .frame_interval_s()
                .map_or(now + 0.001, |interval| {
                    (self.stats.last_frame_time + interval).max(now + 0.001)
                });
            next = next.min(paced);
        }
        next.max(now + 0.001)
    }

    pub(crate) fn next_input_repaint_at(&self, now: f64) -> f64 {
        self.stats.frame_interval_s().map_or(now, |interval| {
            (self.stats.last_frame_time + interval).max(now)
        })
    }

    fn continuous_repaint(&self) -> bool {
        self.ui_drag_active()
            || self.pointer.text_selecting
            || self.workbench.source.editor.selecting
            || self.workbench.source.editor.scroll_grab.is_some()
            || self.workbench.console.dragging()
            || self
                .chart_world
                .panes
                .iter()
                .any(|pane| pane.view.pan.is_some())
    }
}

pub(crate) fn configure_zero_spacing(ctx: &kit::Context) {
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = kit::Vec2::ZERO;
        style.spacing.window_margin = kit::Margin::ZERO;
        style.spacing.menu_margin = kit::Margin::ZERO;
        style.spacing.button_padding = kit::Vec2::ZERO;
        style.spacing.indent = 0.0;
        style.spacing.interact_size = kit::Vec2::ZERO;
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
        style.spacing.default_area_size = kit::Vec2::ZERO;
        style.interaction.interact_radius = 0.0;

        style.visuals.window_corner_radius = kit::CornerRadius::ZERO;
        style.visuals.menu_corner_radius = kit::CornerRadius::ZERO;
        style.visuals.window_shadow = kit::Shadow::NONE;

        style.visuals.disabled_alpha = 1.0;
    });
}

pub(crate) fn configure_tessellation(ctx: &kit::Context) {
    ctx.tessellation_options_mut(|opt| {
        opt.feathering = false;
        opt.round_text_to_pixels = true;
        opt.round_line_segments_to_pixels = true;
        opt.round_rects_to_pixels = true;
    });
}
