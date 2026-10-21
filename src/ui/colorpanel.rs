use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::button::*;
use crate::ui::icon::*;
use crate::ui::textbox::*;
use eframe::egui;

pub(crate) const CP_COLS: i32 = 26;
pub(crate) const CP_ROWS: i32 = 28;
pub(crate) const CP_W: f32 = cells(CP_COLS);
pub(crate) const CP_H: f32 = cells(CP_ROWS);

pub(crate) const CP_CONTENT_W: f32 = CP_W - 2.0 * LINE_W;

pub(crate) const CP_TITLE_ROWS: i32 = 2;
pub(crate) const CP_SV_ROWS: i32 = 16;
pub(crate) const CP_SLIDER_ROWS: i32 = 2;
pub(crate) const CP_HEX_ROWS: i32 = 2;
pub(crate) const CP_BTN_ROWS: i32 = 2;

pub(crate) const CP_ROW_SV: i32 = CP_TITLE_ROWS;
pub(crate) const CP_ROW_RGB: i32 = CP_ROW_SV + CP_SV_ROWS;
pub(crate) const CP_ROW_HEX: i32 = CP_ROW_RGB + 3 * CP_SLIDER_ROWS;
pub(crate) const CP_ROW_BTN: i32 = CP_ROW_HEX + CP_HEX_ROWS;

pub(crate) const CP_Y_SV: f32 = grid_line(CP_ROW_SV);
pub(crate) const CP_Y_RGB: f32 = grid_line(CP_ROW_RGB);
pub(crate) const CP_Y_HEX: f32 = grid_line(CP_ROW_HEX);
pub(crate) const CP_Y_BTN: f32 = grid_line(CP_ROW_BTN);

pub(crate) const CP_CLOSE_W: f32 = cells(2);
const _: () = assert!(CP_CLOSE_W == WIN_CLOSE_W && CP_TITLE_ROWS == WIN_TITLE_ROWS);

pub(crate) fn cp_band(panel: egui::Rect, row: i32, rows: i32) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(panel.min.x, panel.min.y + grid_line(row)),
        egui::vec2(CP_W, cells(rows)),
    )
}

pub(crate) fn cp_inner(panel: egui::Rect, row: i32, rows: i32) -> egui::Rect {
    cp_band(panel, row, rows).shrink(LINE_W)
}

pub(crate) fn hex_box(panel: egui::Rect) -> egui::Rect {
    let band = cp_band(panel, CP_ROW_HEX, CP_HEX_ROWS);
    egui::Rect::from_min_max(
        egui::pos2(band.min.x + grid_line(CP_TRACK_LINE), band.min.y),
        band.max,
    )
}

pub(crate) const SV_COLS: i32 = CP_COLS;
pub(crate) const SV_ROWS: i32 = CP_SV_ROWS;
pub(crate) const SV_BLOCK: f32 = GRID;

pub(crate) const CP_SLIDER_LABEL_COLS: i32 = 2;
pub(crate) const CP_SLIDER_GAP_COLS: i32 = 1;

pub(crate) const CP_SLIDER_VALUE_COLS: i32 = 5;
pub(crate) const CP_TRACK_COLS: i32 =
    CP_COLS - CP_SLIDER_LABEL_COLS - CP_SLIDER_VALUE_COLS - 2 * CP_SLIDER_GAP_COLS;
pub(crate) const CP_TRACK_LINE: i32 = CP_SLIDER_LABEL_COLS + CP_SLIDER_GAP_COLS;
pub(crate) const CP_TRACK_W: f32 = cells(CP_TRACK_COLS);

pub(crate) const CP_TRACK_INNER_W: i32 = 9 * CP_TRACK_COLS - 1;

pub(crate) const CP_HANDLE_W: f32 = cells(1);
pub(crate) const CP_HANDLE_TRAVEL: f32 = CP_TRACK_W - CP_HANDLE_W;

pub(crate) const CP_HALF_COLS: i32 = CP_COLS / 2;
pub(crate) const CP_HALF_W: f32 = cells(CP_HALF_COLS);
pub(crate) const CP_LEFT_W: f32 = CP_HALF_W;

pub(crate) const HEX_LEN_MAX: usize = 6;

pub(crate) const HEX_PREFIX: &str = "#";

pub(crate) const CP_APPLY_TEXT: &str = "应用";

pub(crate) const CP_REVERT_TEXT: &str = "撤销";

pub(crate) fn hex_input_char(c: char) -> bool {
    c != '#' && !c.is_control()
}

pub(crate) const CP_DEFAULT_X: f32 = 450.0;
pub(crate) const CP_DEFAULT_Y: f32 = 99.0;

const _: () = assert!(CP_W == 235.0 && CP_H == 253.0);
const _: () = assert!((CP_W as i32 - 1) % 9 == 0 && (CP_H as i32 - 1) % 9 == 0);
const _: () = assert!(CP_ROW_BTN + CP_BTN_ROWS == CP_ROWS);

const _: () = assert!(CP_CONTENT_W == 26.0 * GRID + 25.0 * LINE_W);
const _: () = assert!(SV_COLS as f32 * SV_BLOCK + (SV_COLS as f32 - 1.0) * LINE_W == CP_CONTENT_W);
const _: () = assert!(
    SV_ROWS as f32 * SV_BLOCK + (SV_ROWS as f32 - 1.0) * LINE_W == cells(CP_SV_ROWS) - 2.0 * LINE_W
);

const _: () = assert!((CP_HALF_W as i32 - 1) % 9 == 0);
const _: () = assert!(CP_LEFT_W + CP_HALF_W - LINE_W == CP_W);

const _: () = assert!(
    CP_SLIDER_LABEL_COLS
        + CP_SLIDER_GAP_COLS
        + CP_TRACK_COLS
        + CP_SLIDER_GAP_COLS
        + CP_SLIDER_VALUE_COLS
        == CP_COLS
);
const _: () = assert!(CP_TRACK_W > CP_HANDLE_W);
const _: () = assert!((CP_TRACK_W as i32 - 1) % 9 == 0 && (CP_HANDLE_W as i32 - 1) % 9 == 0);

const _: () = assert!(CP_HANDLE_TRAVEL > 0.0 && CP_HANDLE_TRAVEL % 1.0 == 0.0);
const _: () = assert!((CP_Y_SV as i32) % 9 == 0);
const _: () = assert!((CP_Y_RGB as i32) % 9 == 0 && (CP_Y_HEX as i32) % 9 == 0);
const _: () = assert!((CP_Y_BTN as i32) % 9 == 0);

const _: () = assert!((CP_DEFAULT_X as i32) % 9 == 0 && (CP_DEFAULT_Y as i32) % 9 == 0);

pub(crate) fn hsv_to_rgb(h: u16, s: u8, v: u8) -> (u8, u8, u8) {
    let hf = (h % 360) as f32;
    let sf = s as f32 / 255.0;
    let vf = v as f32 / 255.0;
    let c = vf * sf;
    let x = c * (1.0 - ((hf / 60.0) % 2.0 - 1.0).abs());

    let (r, g, b) = match (hf / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = vf - c;
    let to = |f: f32| ((f + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (to(r), to(g), to(b))
}

pub(crate) fn selection_cross_rects(center: egui::Pos2) -> [egui::Rect; 2] {
    let cx = center.x.round();
    let cy = center.y.round();
    [
        egui::Rect::from_min_size(
            egui::pos2(cx - 3.0 * LINE_W, cy - LINE_W),
            egui::vec2(6.0 * LINE_W, 2.0 * LINE_W),
        ),
        egui::Rect::from_min_size(
            egui::pos2(cx - LINE_W, cy - 3.0 * LINE_W),
            egui::vec2(2.0 * LINE_W, 6.0 * LINE_W),
        ),
    ]
}

pub(crate) fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let d = max - min;

    let h = if d == 0.0 {
        0.0
    } else if max == rf {
        60.0 * (((gf - bf) / d) % 6.0)
    } else if max == gf {
        60.0 * ((bf - rf) / d + 2.0)
    } else {
        60.0 * ((rf - gf) / d + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    let s = if max == 0.0 { 0.0 } else { d / max };
    (
        (h.round() as u16).min(360),
        (s * 255.0).round() as u8,
        (max * 255.0).round() as u8,
    )
}

pub(crate) fn contrast_on(rgb: (u8, u8, u8)) -> egui::Color32 {
    let luma = (299 * rgb.0 as u32 + 587 * rgb.1 as u32 + 114 * rgb.2 as u32) / 1000;
    if luma >= 128 {
        egui::Color32::BLACK
    } else {
        egui::Color32::WHITE
    }
}

pub(crate) fn rgb32(rgb: (u8, u8, u8)) -> egui::Color32 {
    egui::Color32::from_rgb(rgb.0, rgb.1, rgb.2)
}

pub(crate) fn hex_of(rgb: (u8, u8, u8)) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb.0, rgb.1, rgb.2)
}

pub(crate) fn hex_digits(rgb: (u8, u8, u8)) -> String {
    format!("{:02X}{:02X}{:02X}", rgb.0, rgb.1, rgb.2)
}

pub(crate) fn hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn parse_hex(s: &str) -> Option<(u8, u8, u8)> {
    let bytes = s.as_bytes();
    if bytes.len() != HEX_LEN_MAX {
        return None;
    }
    let mut rgb = [0u8; 3];
    for i in 0..3 {
        let hi = hex_digit(bytes[i * 2])?;
        let lo = hex_digit(bytes[1 + i * 2])?;
        rgb[i] = hi * 16 + lo;
    }
    Some((rgb[0], rgb[1], rgb[2]))
}

pub(crate) fn bar_value(i: i32, max: i32) -> i32 {
    (i as f32 * max as f32 / (CP_TRACK_INNER_W - 1) as f32).round() as i32
}

pub(crate) fn handle_left(track_min_x: f32, ratio: f32) -> f32 {
    track_min_x + (ratio.clamp(0.0, 1.0) * CP_HANDLE_TRAVEL).round()
}

pub(crate) fn s_of_col(col: i32) -> u8 {
    (col as f32 * 255.0 / (SV_COLS - 1) as f32).round() as u8
}

pub(crate) fn v_of_row(row: i32) -> u8 {
    ((SV_ROWS - 1 - row) as f32 * 255.0 / (SV_ROWS - 1) as f32).round() as u8
}

pub(crate) fn col_of_s(s: u8) -> i32 {
    (s as f32 * (SV_COLS - 1) as f32 / 255.0).round() as i32
}

pub(crate) fn row_of_v(v: u8) -> i32 {
    SV_ROWS - 1 - (v as f32 * (SV_ROWS - 1) as f32 / 255.0).round() as i32
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ColorTarget {
    Up,
    Down,
}

impl ColorTarget {
    pub(crate) fn name(self) -> &'static str {
        match self {
            ColorTarget::Up => "涨",
            ColorTarget::Down => "跌",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ColorDrag {
    None,
    Sv,

    Slider(u8),
}

#[derive(Clone)]
pub(crate) struct ColorPanel {
    pub(crate) target: ColorTarget,

    pub(crate) h: u16,
    pub(crate) s: u8,
    pub(crate) v: u8,

    pub(crate) rgb: (u8, u8, u8),

    pub(crate) pending_up: (u8, u8, u8),
    pub(crate) pending_down: (u8, u8, u8),

    pub(crate) initial_up: (u8, u8, u8),
    pub(crate) initial_down: (u8, u8, u8),

    pub(crate) hex: TextInput,
    pub(crate) hex_valid: bool,

    pub(crate) drag: ColorDrag,

    pub(crate) drag_off: egui::Vec2,

    pub(crate) close_requested: bool,
}

impl ColorPanel {
    pub(crate) fn new() -> Self {
        Self {
            target: ColorTarget::Up,
            h: 0,
            s: 0,
            v: 0,
            rgb: (0, 0, 0),
            pending_up: UP_RGB_DEFAULT,
            pending_down: DOWN_RGB_DEFAULT,
            initial_up: UP_RGB_DEFAULT,
            initial_down: DOWN_RGB_DEFAULT,
            hex: TextInput::new(HEX_LEN_MAX),
            hex_valid: true,
            drag: ColorDrag::None,
            drag_off: egui::Vec2::ZERO,
            close_requested: false,
        }
    }

    pub(crate) fn set_hsv(&mut self, h: u16, s: u8, v: u8) {
        self.h = h;
        self.s = s;
        self.v = v;
        self.rgb = hsv_to_rgb(h, s, v);
    }

    pub(crate) fn set_rgb(&mut self, rgb: (u8, u8, u8)) {
        self.rgb = rgb;
        let (h, s, v) = rgb_to_hsv(rgb.0, rgb.1, rgb.2);
        self.h = h;
        self.s = s;
        self.v = v;
    }

    pub(crate) fn load(&mut self, rgb: (u8, u8, u8)) {
        self.set_rgb(rgb);

        self.hex.set_text(&hex_digits(rgb));
        self.hex_valid = true;
    }

    pub(crate) fn pending_for(&self, target: ColorTarget) -> (u8, u8, u8) {
        match target {
            ColorTarget::Up => self.pending_up,
            ColorTarget::Down => self.pending_down,
        }
    }

    pub(crate) fn save_pending(&mut self) {
        let rgb = self.rgb;
        match self.target {
            ColorTarget::Up => self.pending_up = rgb,
            ColorTarget::Down => self.pending_down = rgb,
        }
    }

    pub(crate) fn close_session(&mut self) {
        self.drag = ColorDrag::None;
    }

    pub(crate) fn revert_initial(&mut self) {
        self.pending_up = self.initial_up;
        self.pending_down = self.initial_down;
        let rgb = self.pending_for(self.target);
        self.load(rgb);
        self.close_session();
    }
}

impl StockApp {
    pub(crate) fn open_color_panel(&mut self, target: ColorTarget) {
        let (up, down) = (self.up_color, self.down_color);
        self.color_panel.pending_up = up;
        self.color_panel.pending_down = down;
        self.color_panel.initial_up = up;
        self.color_panel.initial_down = down;
        self.color_panel.target = target;
        self.color_panel.drag = ColorDrag::None;
        self.color_panel.close_requested = false;
        let rgb = self.color_panel.pending_for(target);
        self.color_panel.load(rgb);
        if self.win_mgr.show_color(self.work_rect).is_none() {
            let text = "窗口出生位置已触底，不能继续添加".to_owned();
            self.status_msg = text.clone();
            self.tooltip = Some((
                egui::Rect::from_min_size(self.work_rect.center(), egui::vec2(LINE_W, LINE_W)),
                text,
            ));
            return;
        }
        self.status_msg = format!("颜色面板：正在编辑{}色", target.name());
    }

    pub(crate) fn ui_color_panel_content(&mut self, ui: &mut egui::Ui, panel: egui::Rect) {
        let mut cp = self.color_panel.clone();

        let painter = ui.painter().clone();
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(panel.min.x, panel.min.y + grid_line(CP_TITLE_ROWS)),
                panel.max,
            ),
            egui::CornerRadius::ZERO,
            theme().color_picker_bg,
        );

        let pointer = self
            .pointer
            .live
            .or_else(|| ui.input(|i| i.pointer.latest_pos()));
        let down = ui.input(|i| i.pointer.primary_down());

        let rgb_at_start = cp.rgb;

        let sv = cp_inner(panel, CP_ROW_SV, CP_SV_ROWS);
        for row in 0..SV_ROWS {
            for col in 0..SV_COLS {
                let block = egui::Rect::from_min_size(
                    egui::pos2(sv.min.x + col as f32 * PITCH, sv.min.y + row as f32 * PITCH),
                    egui::vec2(SV_BLOCK, SV_BLOCK),
                );
                painter.rect_filled(
                    block,
                    egui::CornerRadius::ZERO,
                    rgb32(hsv_to_rgb(cp.h, s_of_col(col), v_of_row(row))),
                );
            }
        }

        let sv_band = cp_band(panel, CP_ROW_SV, CP_SV_ROWS);
        for col in 0..=SV_COLS {
            vline_hard(
                &painter,
                sv.min.x - LINE_W + grid_line(col),
                sv_band.min.y,
                sv_band.max.y,
                theme().color_picker_border,
            );
        }
        for row in 0..=SV_ROWS {
            hline_hard(
                &painter,
                sv_band.min.x,
                sv_band.max.x,
                sv.min.y - LINE_W + grid_line(row),
                theme().color_picker_border,
            );
        }
        let sv_resp = ui.allocate_rect(sv, egui::Sense::click_and_drag());
        if sv_resp.hovered() || sv_resp.is_pointer_button_down_on() {
            self.pointer.hint = CursorKind::Slider;
        }
        if sv_resp.is_pointer_button_down_on() && cp.drag == ColorDrag::None {
            cp.drag = ColorDrag::Sv;
        }
        if cp.drag == ColorDrag::Sv {
            if let Some(p) = pointer {
                let col = (((p.x - sv.min.x) / PITCH).floor() as i32).clamp(0, SV_COLS - 1);
                let row = (((p.y - sv.min.y) / PITCH).floor() as i32).clamp(0, SV_ROWS - 1);
                cp.set_hsv(cp.h, s_of_col(col), v_of_row(row));
            }
        }

        let sel_center = egui::pos2(
            sv.min.x + col_of_s(cp.s) as f32 * PITCH + SV_BLOCK / 2.0,
            sv.min.y + row_of_v(cp.v) as f32 * PITCH + SV_BLOCK / 2.0,
        );

        let (mut cr, mut cg, mut cb) = cp.rgb;
        if let Some(x) = self.color_slider(
            ui,
            &painter,
            0,
            &mut cp.drag,
            &mut cp.drag_off,
            cp_band(panel, CP_ROW_RGB, CP_SLIDER_ROWS),
            "R",
            &cr.to_string(),
            cr as f32 / 255.0,
            255,
            |i| (bar_value(i, 255) as u8, cg, cb),
        ) {
            cp.set_rgb((x as u8, cg, cb));
        }
        (cr, cg, cb) = cp.rgb;
        if let Some(x) = self.color_slider(
            ui,
            &painter,
            1,
            &mut cp.drag,
            &mut cp.drag_off,
            cp_band(panel, CP_ROW_RGB + CP_SLIDER_ROWS, CP_SLIDER_ROWS),
            "G",
            &cg.to_string(),
            cg as f32 / 255.0,
            255,
            |i| (cr, bar_value(i, 255) as u8, cb),
        ) {
            cp.set_rgb((cr, x as u8, cb));
        }
        (cr, cg, cb) = cp.rgb;
        if let Some(x) = self.color_slider(
            ui,
            &painter,
            2,
            &mut cp.drag,
            &mut cp.drag_off,
            cp_band(panel, CP_ROW_RGB + 2 * CP_SLIDER_ROWS, CP_SLIDER_ROWS),
            "B",
            &cb.to_string(),
            cb as f32 / 255.0,
            255,
            |i| (cr, cg, bar_value(i, 255) as u8),
        ) {
            cp.set_rgb((cr, cg, x as u8));
        }

        let hex_band = cp_band(panel, CP_ROW_HEX, CP_HEX_ROWS);
        let hex_box = hex_box(panel);
        let hex_inner = hex_box.shrink(LINE_W);

        self.text(
            &painter,
            egui::pos2(
                text_left_x(hex_band.min.x, panel.min.x),
                hex_band.center().y,
            ),
            egui::Align2::LEFT_CENTER,
            HEX_PREFIX,
            theme().input_text,
        );
        let hex_typed = cp.hex.ui(
            self,
            ui,
            hex_box,
            egui::Id::new("rt_color_hex"),
            &hex_input_char,
            TextInputStyle {
                bg: theme().input_bg,

                text: if cp.hex_valid {
                    theme().input_text
                } else {
                    theme().input_invalid
                },
                edge: Some(theme().input_border),
            },
        );

        match parse_hex(&cp.hex.text()) {
            Some(rgb) => {
                cp.hex_valid = true;
                if hex_typed {
                    cp.set_rgb(rgb);
                }
            }
            None => cp.hex_valid = false,
        }

        let btn_band = cp_band(panel, CP_ROW_BTN, CP_BTN_ROWS);
        let apply_rect =
            egui::Rect::from_min_size(btn_band.min, egui::vec2(CP_LEFT_W, btn_band.height()));
        let cancel_rect = egui::Rect::from_min_size(
            egui::pos2(btn_band.min.x + CP_HALF_W - LINE_W, btn_band.min.y),
            egui::vec2(CP_HALF_W, btn_band.height()),
        );
        let mut close_after = false;
        let apply_text = ButtonContent::Text(CP_APPLY_TEXT.to_owned());
        if self.draw_button(ui, &painter, apply_rect, &apply_text, false, "") {
            cp.save_pending();
            let rgb = cp.pending_for(cp.target);
            match cp.target {
                ColorTarget::Up => self.up_color = rgb,
                ColorTarget::Down => self.down_color = rgb,
            }
            self.mark_panes_dirty();
            self.chart_follows_theme = false;
            cp.close_session();
            self.status_msg = format!(
                "已应用：{}色 {}（图表配色改为自定义）",
                cp.target.name(),
                hex_of(rgb)
            );
            close_after = true;
        }
        let cancel_text = ButtonContent::Text(CP_REVERT_TEXT.to_owned());
        if self.draw_button(ui, &painter, cancel_rect, &cancel_text, false, "") {
            cp.revert_initial();
            self.status_msg = "颜色面板：已撤销到打开时的颜色".to_owned();
        }

        for row in [
            CP_ROW_SV,
            CP_ROW_RGB,
            CP_ROW_RGB + CP_SLIDER_ROWS,
            CP_ROW_RGB + 2 * CP_SLIDER_ROWS,
            CP_ROW_HEX,
            CP_ROW_BTN,
        ] {
            hline_hard(
                &painter,
                panel.min.x,
                panel.max.x,
                panel.min.y + grid_line(row),
                theme().color_picker_border,
            );
        }

        let mark = contrast_on(cp.rgb);
        let [horizontal, vertical] = selection_cross_rects(sel_center);
        painter.rect_filled(horizontal, egui::CornerRadius::ZERO, mark);
        painter.rect_filled(vertical, egui::CornerRadius::ZERO, mark);

        let hex_edge = theme().color_picker_border;
        vline_hard(
            &painter,
            hex_box.min.x,
            hex_band.min.y + LINE_W,
            hex_band.max.y - LINE_W,
            hex_edge,
        );
        hline_hard(
            &painter,
            hex_inner.min.x,
            hex_inner.max.x,
            hex_band.min.y,
            hex_edge,
        );
        hline_hard(
            &painter,
            hex_inner.min.x,
            hex_inner.max.x,
            hex_band.max.y - LINE_W,
            hex_edge,
        );

        if !down {
            cp.drag = ColorDrag::None;
        }
        if cp.rgb != rgb_at_start && !hex_typed {
            cp.hex.set_text(&hex_digits(cp.rgb));
            cp.hex_valid = true;
        }
        self.color_panel = cp;
        if close_after {
            self.color_panel.close_requested = true;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn color_slider(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        slot: u8,
        drag: &mut ColorDrag,
        drag_off: &mut egui::Vec2,
        row: egui::Rect,
        label: &str,
        value_text: &str,
        ratio: f32,
        max: i32,
        grad: impl Fn(i32) -> (u8, u8, u8),
    ) -> Option<i32> {
        let pointer = self
            .pointer
            .live
            .or_else(|| ui.input(|i| i.pointer.latest_pos()));
        let track = egui::Rect::from_min_size(
            egui::pos2(row.min.x + grid_line(CP_TRACK_LINE), row.min.y),
            egui::vec2(CP_TRACK_W, row.height()),
        );
        self.text(
            painter,
            egui::pos2(text_left_x(row.min.x, row.min.x), row.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            theme().input_text,
        );

        let value_rect = egui::Rect::from_min_size(
            egui::pos2(
                row.min.x + grid_line(CP_TRACK_LINE + CP_TRACK_COLS + CP_SLIDER_GAP_COLS),
                row.min.y,
            ),
            egui::vec2(cells(CP_SLIDER_VALUE_COLS), row.height()),
        );
        self.text(
            painter,
            value_rect.center(),
            egui::Align2::CENTER_CENTER,
            value_text,
            theme().input_text,
        );

        let inner = track.shrink(LINE_W);
        for i in 0..CP_TRACK_INNER_W {
            let bar = egui::Rect::from_min_size(
                egui::pos2(inner.min.x + i as f32, inner.min.y),
                egui::vec2(LINE_W, inner.height()),
            );
            painter.rect_filled(bar, egui::CornerRadius::ZERO, rgb32(grad(i)));
        }

        let rest_x = handle_left(track.min.x, ratio);
        let rest = egui::Rect::from_min_size(
            egui::pos2(rest_x, row.min.y),
            egui::vec2(CP_HANDLE_W, row.height()),
        );

        let track_resp = ui.allocate_rect(track, egui::Sense::click_and_drag());
        let over_track = track_resp.hovered();
        let wheel_value = if over_track && *drag == ColorDrag::None {
            let notches = color_wheel_notches(ui);
            (notches != 0).then(|| {
                let current = (ratio.clamp(0.0, 1.0) * max as f32).round() as i32;
                (current + notches).clamp(0, max)
            })
        } else {
            None
        };

        let knob = ui.interact(
            rest,
            egui::Id::new(("rt_cp_slider_knob", slot)),
            egui::Sense::click_and_drag(),
        );

        if knob.hovered() || over_track || *drag == ColorDrag::Slider(slot) {
            self.pointer.hint = CursorKind::Slider;
        }
        if (knob.is_pointer_button_down_on() || track_resp.is_pointer_button_down_on())
            && *drag == ColorDrag::None
        {
            *drag = ColorDrag::Slider(slot);

            if let Some(p) = pointer {
                drag_off.x = if p.x >= rest_x && p.x < rest_x + CP_HANDLE_W {
                    (p.x - rest_x).clamp(0.0, CP_HANDLE_W - LINE_W)
                } else {
                    CP_HANDLE_W / 2.0
                };
            }
        }
        let drag_x = if *drag == ColorDrag::Slider(slot) {
            pointer.map(|p| {
                let raw = p.x - track.min.x - drag_off.x;
                let x = raw.clamp(0.0, CP_HANDLE_TRAVEL);
                if raw != x {
                    drag_off.x = p.x - track.min.x - x;
                }
                x.round()
            })
        } else {
            None
        };

        let hx = match (drag_x, wheel_value) {
            (Some(x), _) => track.min.x + x,
            (None, Some(value)) => handle_left(track.min.x, value as f32 / max as f32),
            (None, None) => rest_x,
        };
        let handle = egui::Rect::from_min_size(
            egui::pos2(hx, row.min.y),
            egui::vec2(CP_HANDLE_W, row.height()),
        );

        painter.rect_filled(
            handle,
            egui::CornerRadius::ZERO,
            theme().color_picker_handle,
        );
        border_hard(painter, handle, theme().color_picker_border);

        vline_hard(
            painter,
            track.min.x,
            track.min.y,
            track.max.y,
            theme().color_picker_border,
        );
        vline_hard(
            painter,
            track.max.x - LINE_W,
            track.min.y,
            track.max.y,
            theme().color_picker_border,
        );

        drag_x
            .map(|x| (x / CP_HANDLE_TRAVEL * max as f32).round() as i32)
            .or(wheel_value)
    }
}

fn color_wheel_notches(ui: &egui::Ui) -> i32 {
    let mut acc = 0.0f32;
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
    acc.round() as i32
}
