use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::button::*;
use crate::ui::textbox::*;
use crate::ui::textbuf::char_allowed;
use eframe::egui;
use std::collections::VecDeque;

pub(crate) const CONSOLE_TITLE: &str = "调试控制台";
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
const CONSOLE_FOOTER_H: f32 = cells(CONSOLE_FOOTER_ROWS);

#[derive(Clone, Copy, Debug)]
pub(crate) struct ConsoleFooterLayout {
    pub(crate) footer: egui::Rect,
    pub(crate) input: egui::Rect,
    pub(crate) buttons: [egui::Rect; 3],
}

pub(crate) fn console_footer_layout(rect: egui::Rect, widths: [f32; 3]) -> ConsoleFooterLayout {
    let top = (rect.max.y - CONSOLE_FOOTER_H).max(rect.min.y);
    let footer = egui::Rect::from_min_max(egui::pos2(rect.min.x, top), rect.max);

    let mut x = rect.max.x;
    let mut buttons = [egui::Rect::NOTHING; 3];
    for i in (0..3).rev() {
        x -= widths[i];
        buttons[i] =
            egui::Rect::from_min_size(egui::pos2(x, top), egui::vec2(widths[i], CONSOLE_FOOTER_H));

        x += LINE_W;
    }

    let left = rect.min.x + grid_line(1);
    let right = (buttons[0].min.x + LINE_W).max(left + cells(1));
    let input = egui::Rect::from_min_max(
        egui::pos2(left, top),
        egui::pos2(right, top + CONSOLE_FOOTER_H),
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
            follow: true,
            wheel_acc: 0.0,
            last_status: String::new(),
        };
        console.log("调试控制台已就绪。输入 help 查看命令。");
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
            self.log(format!("[状态] {status}"));
        }
    }

    pub(crate) fn clear(&mut self) {
        self.lines.clear();
        self.scroll = 0;
        self.follow = true;
    }

    #[cfg(test)]
    pub(crate) fn log_contains(&self, needle: &str) -> bool {
        self.lines.iter().any(|line| line.contains(needle))
    }

    pub(crate) fn ui(&mut self, app: &mut StockApp, ui: &mut egui::Ui, rect: egui::Rect) {
        let painter = ui.painter().clone().with_clip_rect(rect);
        painter.rect_filled(rect, egui::CornerRadius::ZERO, theme().work_bg);

        let header = free_band(rect, 0, TEXT_ROWS);

        let header_segs = vec![
            format!("窗口 {}", app.debug_windows.windows.len()),
            if app.pointer.system {
                "鼠标 系统".to_owned()
            } else {
                format!(
                    "速度 {:.2}x 加速度 {:.2}",
                    app.pointer.feel().base_speed,
                    app.pointer.feel().acceleration
                )
            },
        ];
        app.paint_status_bar(&painter, header, &header_segs);

        let labels = ["执行", "清空", "状态"];
        let mut widths = [0.0f32; 3];
        for (i, label) in labels.iter().enumerate() {
            widths[i] = app.button_width(&ButtonContent::Text((*label).to_owned()));
        }
        let layout = console_footer_layout(rect, widths);
        let footer = layout.footer;
        let log = egui::Rect::from_min_max(
            egui::pos2(rect.min.x, header.max.y - LINE_W),
            egui::pos2(rect.max.x, footer.min.y),
        );

        painter.rect_filled(log, egui::CornerRadius::ZERO, theme().work_bg);
        self.ui_log(app, ui, &painter, log);
        hline_hard(
            &painter,
            log.min.x,
            log.max.x,
            log.min.y,
            theme().ctrl_border,
        );

        painter.rect_filled(footer, egui::CornerRadius::ZERO, theme().panel_bg);
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
            &ButtonContent::Text("执行".to_owned()),
            false,
            "",
        ) {
            execute = true;
        }
        if app.draw_button(
            ui,
            &painter,
            layout.buttons[1],
            &ButtonContent::Text("清空".to_owned()),
            false,
            "",
        ) {
            self.clear();
        }
        if app.draw_button(
            ui,
            &painter,
            layout.buttons[2],
            &ButtonContent::Text("状态".to_owned()),
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
                egui::Id::new(CONSOLE_INPUT_ID),
                &char_allowed,
                TextInputStyle {
                    bg: theme().work_bg,
                    text: theme().text,
                    edge: Some(theme().ctrl_border),
                },
            );
        }
        let focused = ui.memory(|m| m.has_focus(egui::Id::new(CONSOLE_INPUT_ID)));
        if focused && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            execute = true;
        }
        if execute {
            self.submit(app, ui.ctx().input(|i| i.time));
        }
    }

    fn ui_log(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        log: egui::Rect,
    ) {
        let inner = log.shrink(LINE_W);
        let rows = ((inner.height() / SRC_ROW_PITCH).floor() as usize).max(1);
        let max_scroll = self.lines.len().saturating_sub(rows);

        let hovered = !app.win_silent
            && ui
                .input(|i| i.pointer.hover_pos())
                .is_some_and(|p| log.contains(p));
        if hovered {
            let mut acc = self.wheel_acc;
            ui.input(|i| {
                for event in &i.events {
                    if let egui::Event::MouseWheel { unit, delta, .. } = event {
                        acc += match unit {
                            egui::MouseWheelUnit::Line => delta.y,
                            egui::MouseWheelUnit::Point => delta.y / WHEEL_POINTS_PER_NOTCH,
                            egui::MouseWheelUnit::Page => delta.y,
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
                egui::pos2(text_x, y),
                egui::Align2::LEFT_CENTER,
                line,
                theme().text,
            );
        }
    }

    fn submit(&mut self, app: &mut StockApp, time: f64) {
        let command = self.input.text().trim().to_owned();
        self.input.set_text("");
        if command.is_empty() {
            return;
        }
        self.log(format!("> {command}"));
        self.run_command(app, &command, time);
        self.follow = true;
    }

    pub(crate) fn run_command(&mut self, app: &mut StockApp, command: &str, time: f64) {
        let mut parts = command.splitn(2, char::is_whitespace);
        let head = parts.next().unwrap_or("").to_ascii_lowercase();
        let rest = parts.next().unwrap_or("").trim();
        match head.as_str() {
            "" => {}
            "help" | "?" => {
                self.log("help / ?        显示命令列表");
                self.log("status          输出运行状态快照");
                self.log("pointer         指针速度 / 加速度 / 当前位置");
                self.log("speed <数值>    设置鼠标速度，例如 speed 1.0");
                self.log("accel <数值>    设置鼠标加速度，例如 accel 0.1");
                self.log("trail on|off    开关鼠标轨迹叠层");
                self.log("grid on|off     开关 UI 工程测试网格");
                self.log("theme           切换到下一套主题");
                self.log("data            行情数量和视图状态");
                self.log("windows         列出当前打开的窗口");
                self.log("regen           重新生成一段行情");
                self.log("echo <文本>     原样输出文本");
                self.log("clear / cls     清空日志");
                self.log("close / quit    关闭控制台窗口");
            }
            "clear" | "cls" => self.clear(),
            "status" => self.print_status(app),
            "fps" => self.log(format!(
                "fps={:.1} frame={:.2}ms interval={:.2}ms frames={} uptime={:.1}s",
                app.fps, app.frame_ms, app.frame_interval_ms, app.frames, time
            )),
            "pointer" => {
                let p = app.pointer.position();
                let v = app.pointer.velocity();
                self.log(format!(
                    "pointer speed={:.2}x accel={:.2} pos=({:.1},{:.1}) v=({:.1},{:.1}) |v|={:.1}px/s",
                    app.pointer.feel().base_speed,
                    app.pointer.feel().acceleration,
                    p.x,
                    p.y,
                    v.x,
                    v.y,
                    v.length()
                ));
            }
            "speed" => {
                if rest.is_empty() {
                    self.log(format!("speed={:.2}x", app.pointer.feel().base_speed));
                } else {
                    match rest.parse::<f32>() {
                        Ok(value) => {
                            let actual = app.pointer.set_speed(value);
                            self.log(format!("speed={actual:.2}x"));
                        }
                        Err(_) => self.log("speed 需要一个数值，例如 speed 1.0"),
                    }
                }
            }
            "accel" => {
                if rest.is_empty() {
                    self.log(format!("accel={:.2}", app.pointer.feel().acceleration));
                } else {
                    match rest.parse::<f32>() {
                        Ok(value) => {
                            let actual = app.pointer.set_acceleration(value);
                            self.log(format!("accel={actual:.2}"));
                        }
                        Err(_) => self.log("accel 需要一个数值，例如 accel 0.1"),
                    }
                }
            }
            "trail" => {
                set_toggle(&mut app.pointer.show_trail, rest);
                self.log(format!(
                    "trail={}",
                    if app.pointer.show_trail { "on" } else { "off" }
                ));
            }
            "grid" => {
                set_toggle(&mut app.show_grid, rest);
                self.log(format!("grid={}", if app.show_grid { "on" } else { "off" }));
            }
            "theme" => {
                app.switch_theme();
                self.log(format!("theme={}", theme().name()));
            }
            "data" => {
                let view = app.active_chart_pane().map(|pane| &app.panes[pane].view);
                self.log(format!(
                    "bars={} seed={} scale={} offset={} visible={}",
                    app.bars.len(),
                    app.bar_seed,
                    view.map_or(0, |view| view.scale),
                    view.map_or(0, |view| view.offset),
                    view.map_or(0, |view| view.visible)
                ));
            }
            "windows" => {
                let open = app.debug_windows.windows.len();
                self.log(format!("windows={open}"));
                for (title, minimized, r) in &app.debug_windows.windows {
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
            _ => self.log(format!("未知命令：{command}。输入 help 查看命令列表。")),
        }
    }

    fn print_status(&mut self, app: &StockApp) {
        let p = app.pointer.position();
        self.log(format!(
            "frame={} fps={:.1} frame_ms={:.2} interval_ms={:.2} windows={}",
            app.frames,
            app.fps,
            app.frame_ms,
            app.frame_interval_ms,
            app.debug_windows.windows.len()
        ));
        self.log(format!(
            "pointer speed={:.2}x accel={:.2} pos=({:.1},{:.1}) trail={}",
            app.pointer.feel().base_speed,
            app.pointer.feel().acceleration,
            p.x,
            p.y,
            if app.pointer.show_trail { "on" } else { "off" }
        ));
        let view = app.active_chart_pane().map(|pane| &app.panes[pane].view);
        self.log(format!(
            "data bars={} seed={} scale={} offset={} visible={}",
            app.bars.len(),
            app.bar_seed,
            view.map_or(0, |view| view.scale),
            view.map_or(0, |view| view.offset),
            view.map_or(0, |view| view.visible)
        ));
        self.log(format!(
            "theme={} grid={} charts={} source={} console={}",
            theme().name(),
            if app.show_grid { "on" } else { "off" },
            app.debug_windows.chart_count,
            app.debug_windows.source_open,
            app.debug_windows.console_open
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
