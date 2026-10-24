use crate::application::app::*;
use crate::domain::layout::*;
use crate::engine::kit;
use crate::engine::theme::*;
use crate::ui::button::*;
use crate::ui::editor::vertical_scrollbar;
use crate::ui::textbox::*;
use crate::ui::textbuf::char_allowed;
use std::collections::VecDeque;

pub(crate) const CONSOLE_TITLE: &str = "Debug Console";
pub(crate) const CONSOLE_COLS: i32 = 84;
pub(crate) const CONSOLE_ROWS: i32 = 40;
pub(crate) const CONSOLE_W: f32 = cells(CONSOLE_COLS);
pub(crate) const CONSOLE_H: f32 = cells(CONSOLE_ROWS);
pub(crate) const CONSOLE_MIN_COLS: i32 = 48;
pub(crate) const CONSOLE_MIN_ROWS: i32 = 18;
pub(crate) const CONSOLE_MIN_W: f32 = cells(CONSOLE_MIN_COLS);
pub(crate) const CONSOLE_MIN_H: f32 = cells(CONSOLE_MIN_ROWS);

const CONSOLE_MAX_LINES: usize = 2000;
const CONSOLE_INPUT_ID: &str = "rt_debug_console_input";
const CONSOLE_FOOTER_ROWS: i32 = TEXT_ROWS;
pub(crate) const CONSOLE_FOOTER_H: f32 = cells(CONSOLE_FOOTER_ROWS);

#[derive(Clone, Copy, Debug)]
pub(crate) struct ConsoleFooterLayout {
    pub(crate) footer: kit::Rect,
    pub(crate) input: kit::Rect,
    pub(crate) buttons: [kit::Rect; 3],
}

pub(crate) fn console_footer_layout(rect: kit::Rect, widths: [f32; 3]) -> ConsoleFooterLayout {
    let top = (rect.max.y - CONSOLE_FOOTER_H).max(rect.min.y);
    let footer = kit::Rect::from_min_max(kit::pos2(rect.min.x, top), rect.max);

    let mut x = rect.max.x;
    let mut buttons = [kit::Rect::NOTHING; 3];
    for i in (0..3).rev() {
        x -= widths[i];
        buttons[i] =
            kit::Rect::from_min_size(kit::pos2(x, top), kit::vec2(widths[i], CONSOLE_FOOTER_H));

        x += LINE_W;
    }

    let left = rect.min.x + grid_line(1);
    let right = (buttons[0].min.x + LINE_W).max(left + cells(1));
    let input = kit::Rect::from_min_max(
        kit::pos2(left, top),
        kit::pos2(right, top + CONSOLE_FOOTER_H),
    );
    ConsoleFooterLayout {
        footer,
        input,
        buttons,
    }
}

pub(crate) struct DebugConsole {
    lines: VecDeque<String>,
    input: TextInput,

    scroll: usize,
    scroll_grab: Option<kit::Vec2>,
    follow: bool,
    wheel_acc: f32,
    last_status: String,
}

impl Default for DebugConsole {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugConsole {
    pub(crate) fn new() -> Self {
        let mut console = Self {
            lines: VecDeque::new(),
            input: TextInput::new(240),
            scroll: 0,
            scroll_grab: None,
            follow: true,
            wheel_acc: 0.0,
            last_status: String::new(),
        };
        console.log("Debug console ready. Type help for commands.");
        console
    }

    pub(crate) fn log(&mut self, text: impl Into<String>) {
        let text = text.into();
        if text.is_empty() {
            return;
        }
        self.lines.push_back(text);
        while self.lines.len() > CONSOLE_MAX_LINES {
            self.lines.pop_front();
            self.scroll = self.scroll.saturating_sub(1);
        }
        self.follow = true;
    }

    pub(crate) fn observe_status(&mut self, status: &str) {
        if status == self.last_status {
            return;
        }
        self.last_status = status.to_owned();
        if !status.is_empty() {
            self.log(format!("[Status] {status}"));
        }
    }

    pub(crate) fn clear(&mut self) {
        self.lines.clear();
        self.scroll = 0;
        self.follow = true;
    }

    pub(crate) fn dragging(&self) -> bool {
        self.scroll_grab.is_some()
    }

    #[cfg(test)]
    pub(crate) fn log_contains(&self, needle: &str) -> bool {
        self.lines.iter().any(|line| line.contains(needle))
    }

    #[cfg(test)]
    pub(crate) fn scroll(&self) -> usize {
        self.scroll
    }

    pub(crate) fn ui(&mut self, app: &mut AppState, ui: &mut kit::Ui, rect: kit::Rect) {
        let painter = ui.painter().clone().with_clip_rect(rect);
        painter.rect_filled(rect, kit::CornerRadius::ZERO, theme().work_bg);

        let header = free_band(rect, 0, TEXT_ROWS);

        let header_segs = vec![
            format!("Windows: {}", app.workbench.debug_windows.windows.len()),
            if app.pointer.system {
                "Pointer System".to_owned()
            } else {
                "Pointer Software".to_owned()
            },
        ];
        app.paint_status_bar_on_layer(&painter, header, &header_segs, ui.ctx().current_layer());

        let labels = ["Run", "Clear", "Status"];
        let mut widths = [0.0f32; 3];
        for (i, label) in labels.iter().enumerate() {
            widths[i] = app.button_width(&ButtonContent::Text((*label).to_owned()));
        }
        let layout = console_footer_layout(rect, widths);
        let footer = layout.footer;
        let log = kit::Rect::from_min_max(
            kit::pos2(rect.min.x, header.max.y - LINE_W),
            kit::pos2(rect.max.x, footer.min.y),
        );

        painter.rect_filled(log, kit::CornerRadius::ZERO, theme().work_bg);
        self.ui_log(app, ui, &painter, log);
        hline_hard(
            &painter,
            log.min.x,
            log.max.x,
            log.min.y,
            theme().ctrl_border,
        );

        painter.rect_filled(footer, kit::CornerRadius::ZERO, theme().panel_bg);
        hline_hard(
            &painter,
            footer.min.x,
            footer.max.x,
            footer.min.y,
            theme().ctrl_border,
        );

        let mut execute = false;
        if app.draw_button(
            ui,
            &painter,
            layout.buttons[0],
            &ButtonContent::Text("Run".to_owned()),
            false,
            "",
        ) {
            execute = true;
        }
        if app.draw_button(
            ui,
            &painter,
            layout.buttons[1],
            &ButtonContent::Text("Clear".to_owned()),
            false,
            "",
        ) {
            self.clear();
        }
        if app.draw_button(
            ui,
            &painter,
            layout.buttons[2],
            &ButtonContent::Text("Status".to_owned()),
            false,
            "",
        ) {
            self.print_status(app);
        }

        if layout.input.width() > LINE_W {
            self.input.ui(
                app,
                ui,
                layout.input,
                kit::Id::new(CONSOLE_INPUT_ID),
                &char_allowed,
                TextInputStyle {
                    bg: theme().work_bg,
                    text: theme().input_text,
                    edge: Some(theme().input_border),
                },
            );
        }
        if app.shell.win_silent {
            ui.memory_mut(|m| m.surrender_focus(kit::Id::new(CONSOLE_INPUT_ID)));
        }
        let focused =
            !app.shell.win_silent && ui.memory(|m| m.has_focus(kit::Id::new(CONSOLE_INPUT_ID)));
        if focused && ui.input(|i| i.key_pressed(kit::Key::Enter)) {
            execute = true;
        }
        if execute {
            self.submit(app, ui.ctx().input(|i| i.time));
        }
    }

    fn ui_log(
        &mut self,
        app: &mut AppState,
        ui: &mut kit::Ui,
        painter: &kit::Painter,
        log: kit::Rect,
    ) {
        let inner = kit::Rect::from_min_max(
            log.min + kit::vec2(LINE_W, LINE_W),
            kit::pos2(log.max.x - SRC_BAR_W - 2.0 * LINE_W, log.max.y - LINE_W),
        );
        let rows = ((inner.height() / SRC_ROW_PITCH).floor() as usize).max(1);
        let max_scroll = self.lines.len().saturating_sub(rows);

        let hovered = !app.shell.win_silent
            && ui
                .input(|i| i.pointer.hover_pos())
                .is_some_and(|p| log.contains(p));
        if hovered {
            let mut acc = self.wheel_acc;
            ui.input(|i| {
                for event in &i.events {
                    if let kit::Event::MouseWheel { unit, delta, .. } = event {
                        acc += match unit {
                            kit::MouseWheelUnit::Line => delta.y,
                            kit::MouseWheelUnit::Point => delta.y / WHEEL_POINTS_PER_NOTCH,
                        };
                    }
                }
            });
            let whole = acc.trunc();
            self.wheel_acc = acc - whole;
            if whole != 0.0 {
                self.scroll = (self.scroll as f32 - whole)
                    .round()
                    .clamp(0.0, max_scroll as f32) as usize;
                self.follow = self.scroll >= max_scroll;
            }
        }

        self.scroll = self.scroll.min(max_scroll);
        if self.follow {
            self.scroll = max_scroll;
        }

        vertical_scrollbar(
            app,
            ui,
            painter,
            kit::Id::new("rt_console_scroll"),
            log,
            log.min.y,
            log.max.y,
            rows,
            self.lines.len(),
            max_scroll,
            &mut self.scroll,
            &mut self.scroll_grab,
        );
        self.follow = self.scroll >= max_scroll;

        let text_painter = painter.with_clip_rect(inner);
        let text_x = text_left_x(inner.min.x, log.min.x);
        for k in 0..rows {
            let idx = self.scroll + k;
            let Some(line) = self.lines.get(idx) else {
                break;
            };
            let y = inner.min.y + k as f32 * SRC_ROW_PITCH + SRC_LINE_H / 2.0;
            app.text(
                &text_painter,
                kit::pos2(text_x, y),
                kit::Align2::LEFT_CENTER,
                line,
                theme().input_text,
            );
        }
    }

    fn submit(&mut self, app: &mut AppState, time: f64) {
        let command = self.input.text().trim().to_owned();
        self.input.set_text("");
        if command.is_empty() {
            return;
        }
        self.log(format!("> {command}"));
        self.run_command(app, &command, time);
        self.follow = true;
    }

    pub(crate) fn run_command(&mut self, app: &mut AppState, command: &str, time: f64) {
        let mut parts = command.splitn(2, char::is_whitespace);
        let head = parts.next().unwrap_or("").to_ascii_lowercase();
        let rest = parts.next().unwrap_or("").trim();
        match head.as_str() {
            "" => {}
            "help" | "?" => {
                self.log("help / ?        show commands");
                self.log("status          print runtime state");
                self.log("pointer         print pointer position and speed");
                self.log("trail on|off    toggle the pointer trail overlay");
                self.log("data            print bar count and view state");
                self.log("windows         list open windows");
                self.log("regen           generate new market data");
                self.log("echo <text>     print text verbatim");
                self.log("clear / cls     clear the log");
                self.log("close / quit    close the console window");
            }
            "clear" | "cls" => self.clear(),
            "status" => self.print_status(app),
            "fps" => self.log(format!(
                "fps={:.1} frame={:.2}ms interval={:.2}ms frames={} uptime={:.1}s",
                app.stats.fps,
                app.stats.frame_ms,
                app.stats.frame_interval_ms,
                app.stats.frames,
                time
            )),
            "pointer" => {
                let p = app.pointer.position();
                let v = app.pointer.velocity();
                self.log(format!(
                    "pointer pos=({:.1},{:.1}) v=({:.1},{:.1}) |v|={:.1}px/s",
                    p.x,
                    p.y,
                    v.x,
                    v.y,
                    v.length()
                ));
            }
            "trail" => {
                set_toggle(&mut app.pointer.show_trail, rest);
                self.log(format!(
                    "trail={}",
                    if app.pointer.show_trail { "on" } else { "off" }
                ));
            }
            "data" => {
                let view = app
                    .active_chart_pane()
                    .map(|pane| &app.chart_world.panes[pane].view);
                self.log(format!(
                    "bars={} seed={} scale={} offset={} visible={}",
                    app.chart_world.bars.len(),
                    app.chart_world.bar_seed,
                    view.map_or(0, |view| view.scale),
                    view.map_or(0, |view| view.offset),
                    view.map_or(0, |view| view.visible)
                ));
            }
            "windows" => {
                let open = app.workbench.debug_windows.windows.len();
                self.log(format!("windows={open}"));
                for (title, minimized, r) in &app.workbench.debug_windows.windows {
                    self.log(format!(
                        "{} {} ({:.0},{:.0}) {:.0}x{:.0}",
                        title,
                        if *minimized { "min" } else { "normal" },
                        r.min.x,
                        r.min.y,
                        r.width(),
                        r.height()
                    ));
                }
            }
            "regen" => {
                app.regenerate_random_walk();
            }
            "echo" => self.log(rest),
            "close" | "quit" | "exit" => {
                app.request_win(WinCmd::CloseConsole);
            }
            _ => self.log(format!(
                "Unknown command: {command}. Type help for commands."
            )),
        }
    }

    fn print_status(&mut self, app: &AppState) {
        let p = app.pointer.position();
        self.log(format!(
            "frame={} fps={:.1} frame_ms={:.2} interval_ms={:.2} windows={}",
            app.stats.frames,
            app.stats.fps,
            app.stats.frame_ms,
            app.stats.frame_interval_ms,
            app.workbench.debug_windows.windows.len()
        ));
        let v = app.pointer.velocity();
        self.log(format!(
            "pointer pos=({:.1},{:.1}) v=({:.1},{:.1}) trail={}",
            p.x,
            p.y,
            v.x,
            v.y,
            if app.pointer.show_trail { "on" } else { "off" }
        ));
        let view = app
            .active_chart_pane()
            .map(|pane| &app.chart_world.panes[pane].view);
        self.log(format!(
            "data bars={} seed={} scale={} offset={} visible={}",
            app.chart_world.bars.len(),
            app.chart_world.bar_seed,
            view.map_or(0, |view| view.scale),
            view.map_or(0, |view| view.offset),
            view.map_or(0, |view| view.visible)
        ));
        self.log(format!(
            "theme={} charts={} source={} console={}",
            theme().name(),
            app.workbench.debug_windows.chart_count,
            app.workbench.debug_windows.source_open,
            app.workbench.debug_windows.console_open
        ));
    }
}

fn set_toggle(flag: &mut bool, arg: &str) {
    match arg.trim().to_ascii_lowercase().as_str() {
        "" | "toggle" => *flag = !*flag,
        "on" | "1" | "true" => *flag = true,
        "off" | "0" | "false" => *flag = false,
        _ => {}
    }
}
