use eframe::egui;
use serde::Deserialize;
use std::cell::Cell;

pub(crate) const C_TITLE_BG: egui::Color32 = egui::Color32::from_rgb(0xBB, 0xBB, 0xBB);

pub(crate) const C_MENU_BG: egui::Color32 = egui::Color32::from_rgb(0xDD, 0xDD, 0xDD);

pub(crate) const C_WORK_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);

pub(crate) const C_DESKTOP_BG: egui::Color32 = egui::Color32::from_rgb(0xEE, 0xEE, 0xEE);
pub(crate) const C_STATUS_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_BTN_FACE: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);

pub(crate) const C_BTN_HOVER: egui::Color32 = egui::Color32::from_rgb(0xDD, 0xDD, 0xDD);
pub(crate) const C_CLOSE_HOVER: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
pub(crate) const C_BORDER_RED: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
pub(crate) const C_TEXT: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00);
pub(crate) const C_TEXT_WHITE: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub(crate) const C_LINE: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);

pub(crate) const C_CTRL_BORDER: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);

pub(crate) const C_GRID_OK: egui::Color32 = egui::Color32::from_rgb(0x00, 0xAA, 0x00);
pub(crate) const C_GRID_BAD: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x88, 0x00);
pub(crate) const C_GRID_CURSOR: egui::Color32 = egui::Color32::from_rgb(0x00, 0xCC, 0xCC);

pub(crate) const C_CHART_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub(crate) const C_AXIS_BG: egui::Color32 = egui::Color32::from_rgb(0xDD, 0xDD, 0xDD);
pub(crate) const C_UP: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
pub(crate) const C_DOWN: egui::Color32 = egui::Color32::from_rgb(0x00, 0x99, 0x00);
pub(crate) const C_TICK: egui::Color32 = egui::Color32::from_rgb(0xDD, 0xDD, 0xDD);

pub(crate) const UP_RGB_DEFAULT: (u8, u8, u8) = (C_UP.r(), C_UP.g(), C_UP.b());
pub(crate) const DOWN_RGB_DEFAULT: (u8, u8, u8) = (C_DOWN.r(), C_DOWN.g(), C_DOWN.b());

pub(crate) const C_CROSS: egui::Color32 = egui::Color32::from_rgb(0x66, 0x66, 0x66);
pub(crate) const C_CROSS_LABEL: egui::Color32 = egui::Color32::from_rgb(0xEE, 0xEE, 0xEE);

pub(crate) const C_SPLITTER: egui::Color32 = egui::Color32::from_rgb(0xAA, 0xAA, 0xAA);

pub(crate) const C_SCROLL_TRACK: egui::Color32 = egui::Color32::from_rgb(0xAA, 0xAA, 0xAA);
pub(crate) const C_SCROLL_THUMB: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_PANEL_BG: egui::Color32 = egui::Color32::from_rgb(0xDD, 0xDD, 0xDD);
pub(crate) const C_BTN_PRESS: egui::Color32 = egui::Color32::from_rgb(0xAA, 0xAA, 0xAA);

pub(crate) const C_SEL_BG: egui::Color32 = egui::Color32::from_rgb(0xDD, 0xDD, 0xDD);

pub(crate) const C_SYN_COMMENT: egui::Color32 = egui::Color32::from_rgb(0x00, 0x99, 0x00);

pub(crate) const C_POINTER_TRAIL: egui::Color32 = egui::Color32::from_rgb(0x00, 0xCC, 0xCC);
pub(crate) const C_POINTER_TRAIL_HOT: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0xFF);

pub(crate) const C_IND_ZERO: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);

pub(crate) const C_DIF: egui::Color32 = egui::Color32::from_rgb(0x33, 0x33, 0xCC);
pub(crate) const C_DEA: egui::Color32 = egui::Color32::from_rgb(0xCC, 0x99, 0x00);

pub(crate) const C_MA_CLASSIC: [egui::Color32; 6] = [
    egui::Color32::from_rgb(0x33, 0x33, 0xCC),
    egui::Color32::from_rgb(0xCC, 0x00, 0x66),
    egui::Color32::from_rgb(0x00, 0x99, 0x33),
    egui::Color32::from_rgb(0x66, 0x00, 0xCC),
    egui::Color32::from_rgb(0x00, 0x99, 0x99),
    egui::Color32::from_rgb(0x66, 0x66, 0x66),
];

pub(crate) const C_EMA_CLASSIC: [egui::Color32; 6] = [
    egui::Color32::from_rgb(0xFF, 0x00, 0xFF),
    egui::Color32::from_rgb(0xFF, 0x88, 0x00),
    egui::Color32::from_rgb(0x00, 0xCC, 0xCC),
    egui::Color32::from_rgb(0x88, 0xAA, 0x00),
    egui::Color32::from_rgb(0x88, 0x88, 0xFF),
    egui::Color32::from_rgb(0xAA, 0x00, 0xAA),
];

pub(crate) const C_TEXT_UP: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
pub(crate) const C_TEXT_DOWN: egui::Color32 = egui::Color32::from_rgb(0x00, 0x99, 0x00);

pub(crate) const C_KDJ_K: egui::Color32 = egui::Color32::from_rgb(0x33, 0x33, 0xCC);
pub(crate) const C_KDJ_D: egui::Color32 = egui::Color32::from_rgb(0xCC, 0x99, 0x00);
pub(crate) const C_KDJ_J: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0xFF);
pub(crate) const C_RSI_1: egui::Color32 = egui::Color32::from_rgb(0x33, 0x33, 0xCC);
pub(crate) const C_RSI_2: egui::Color32 = egui::Color32::from_rgb(0xCC, 0x99, 0x00);
pub(crate) const C_RSI_3: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0xFF);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ThemeId {
    Classic,
    Dark,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Theme {
    pub(crate) id: ThemeId,

    pub(crate) title_bg: egui::Color32,

    pub(crate) menu_bg: egui::Color32,

    pub(crate) work_bg: egui::Color32,

    pub(crate) desktop_bg: egui::Color32,

    pub(crate) status_bg: egui::Color32,

    pub(crate) panel_bg: egui::Color32,

    pub(crate) splitter: egui::Color32,

    pub(crate) scroll_track: egui::Color32,

    pub(crate) scroll_thumb: egui::Color32,

    pub(crate) btn_face: egui::Color32,

    pub(crate) btn_hover: egui::Color32,

    pub(crate) btn_press: egui::Color32,

    pub(crate) sel_bg: egui::Color32,

    pub(crate) close_hover: egui::Color32,

    pub(crate) ctrl_border: egui::Color32,

    pub(crate) line: egui::Color32,

    pub(crate) chart_line: egui::Color32,

    pub(crate) text: egui::Color32,

    pub(crate) text_white: egui::Color32,

    pub(crate) syn_comment: egui::Color32,

    pub(crate) chart_bg: egui::Color32,

    pub(crate) axis_bg: egui::Color32,

    pub(crate) tick: egui::Color32,

    pub(crate) cross: egui::Color32,

    pub(crate) cross_label: egui::Color32,

    pub(crate) ind_zero: egui::Color32,

    pub(crate) candle_up: egui::Color32,

    pub(crate) candle_down: egui::Color32,

    pub(crate) text_up: egui::Color32,

    pub(crate) text_down: egui::Color32,

    pub(crate) dif: egui::Color32,

    pub(crate) dea: egui::Color32,

    pub(crate) kdj_k: egui::Color32,
    pub(crate) kdj_d: egui::Color32,
    pub(crate) kdj_j: egui::Color32,

    pub(crate) rsi_1: egui::Color32,
    pub(crate) rsi_2: egui::Color32,
    pub(crate) rsi_3: egui::Color32,

    pub(crate) boll_mid: egui::Color32,
    pub(crate) boll_up: egui::Color32,
    pub(crate) boll_down: egui::Color32,
    pub(crate) obv: egui::Color32,
    pub(crate) cci: egui::Color32,
    pub(crate) bias: [egui::Color32; 3],

    pub(crate) ma: [egui::Color32; 6],
    pub(crate) ema: [egui::Color32; 6],

    pub(crate) grid_ok: egui::Color32,

    pub(crate) grid_bad: egui::Color32,

    pub(crate) grid_cursor: egui::Color32,

    pub(crate) pointer_trail: egui::Color32,

    pub(crate) pointer_trail_hot: egui::Color32,
}

impl Theme {
    pub(crate) const CLASSIC: Self = Self {
        id: ThemeId::Classic,
        title_bg: C_TITLE_BG,
        menu_bg: C_MENU_BG,
        work_bg: C_WORK_BG,
        desktop_bg: C_DESKTOP_BG,
        status_bg: C_STATUS_BG,
        panel_bg: C_PANEL_BG,
        splitter: C_SPLITTER,
        scroll_track: C_SCROLL_TRACK,
        scroll_thumb: C_SCROLL_THUMB,
        btn_face: C_BTN_FACE,
        btn_hover: C_BTN_HOVER,
        btn_press: C_BTN_PRESS,
        sel_bg: C_SEL_BG,
        close_hover: C_CLOSE_HOVER,
        ctrl_border: C_CTRL_BORDER,
        line: C_LINE,
        chart_line: C_LINE,
        text: C_TEXT,
        text_white: C_TEXT_WHITE,
        syn_comment: C_SYN_COMMENT,
        chart_bg: C_CHART_BG,
        axis_bg: C_AXIS_BG,
        tick: C_TICK,
        cross: C_CROSS,
        cross_label: C_CROSS_LABEL,
        ind_zero: C_IND_ZERO,
        candle_up: C_UP,
        candle_down: C_DOWN,
        text_up: C_TEXT_UP,
        text_down: C_TEXT_DOWN,
        dif: C_DIF,
        dea: C_DEA,
        kdj_k: C_KDJ_K,
        kdj_d: C_KDJ_D,
        kdj_j: C_KDJ_J,
        rsi_1: C_RSI_1,
        rsi_2: C_RSI_2,
        rsi_3: C_RSI_3,
        boll_mid: egui::Color32::from_rgb(0x33, 0x66, 0xCC),
        boll_up: egui::Color32::from_rgb(0xCC, 0x00, 0x66),
        boll_down: egui::Color32::from_rgb(0x00, 0x99, 0x99),
        obv: egui::Color32::from_rgb(0xCC, 0x66, 0x00),
        cci: egui::Color32::from_rgb(0x66, 0x00, 0xCC),
        bias: [
            egui::Color32::from_rgb(0x33, 0x33, 0xCC),
            egui::Color32::from_rgb(0xCC, 0x99, 0x00),
            egui::Color32::from_rgb(0xFF, 0x00, 0xFF),
        ],
        ma: C_MA_CLASSIC,
        ema: C_EMA_CLASSIC,
        grid_ok: C_GRID_OK,
        grid_bad: C_GRID_BAD,
        grid_cursor: C_GRID_CURSOR,
        pointer_trail: C_POINTER_TRAIL,
        pointer_trail_hot: C_POINTER_TRAIL_HOT,
    };

    pub(crate) const DARK: Self = Self::dark_mirror();

    const fn dark_mirror() -> Self {
        let l = Self::CLASSIC;
        let mut t = l;
        t.id = ThemeId::Dark;
        t.title_bg = invert_color(l.title_bg);
        t.menu_bg = invert_color(l.menu_bg);
        t.work_bg = invert_color(l.work_bg);
        t.desktop_bg = invert_color(l.desktop_bg);
        t.status_bg = invert_color(l.status_bg);
        t.panel_bg = invert_color(l.panel_bg);
        t.splitter = invert_color(l.splitter);
        t.scroll_track = invert_color(l.scroll_track);
        t.scroll_thumb = invert_color(l.scroll_thumb);
        t.btn_face = invert_color(l.btn_face);
        t.btn_hover = invert_color(l.btn_hover);
        t.btn_press = invert_color(l.btn_press);
        t.sel_bg = invert_color(l.sel_bg);
        t.text = invert_color(l.text);
        t.syn_comment = invert_color(l.syn_comment);
        t.grid_ok = invert_color(l.grid_ok);
        t.grid_bad = invert_color(l.grid_bad);
        t.grid_cursor = invert_color(l.grid_cursor);
        t.pointer_trail = invert_color(l.pointer_trail);
        t.pointer_trail_hot = invert_color(l.pointer_trail_hot);
        t.chart_bg = egui::Color32::from_rgb(0x00, 0x00, 0x00);
        t.chart_line = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
        t.axis_bg = egui::Color32::from_rgb(0x22, 0x22, 0x22);
        t.tick = egui::Color32::from_rgb(0x22, 0x22, 0x22);
        t.cross_label = egui::Color32::from_rgb(0x11, 0x11, 0x11);
        t.ind_zero = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
        t.candle_down = egui::Color32::from_rgb(0x00, 0xFF, 0xFF);
        t.ctrl_border = C_BORDER_RED;
        t.line = C_BORDER_RED;
        t.close_hover = C_CLOSE_HOVER;
        t.text_white = C_TEXT_WHITE;
        t
    }

    pub(crate) const fn of(id: ThemeId) -> Self {
        match id {
            ThemeId::Classic => Self::CLASSIC,
            ThemeId::Dark => Self::DARK,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self.id {
            ThemeId::Classic => "浅色",
            ThemeId::Dark => "深色",
        }
    }

    pub(crate) fn load(id: ThemeId) -> Self {
        let path = crate::assets::theme_file(id.file_name());
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("[theme] 读不到 {}：{error}，使用内置配色", path.display());
                return Self::of(id);
            }
        };
        match Self::from_json(&text, id) {
            Ok(theme) => theme,
            Err(error) => {
                eprintln!("[theme] {} 无效：{error}，使用内置配色", path.display());
                Self::of(id)
            }
        }
    }

    pub(crate) fn from_json(text: &str, expected: ThemeId) -> Result<Self, String> {
        let file: ThemeFile =
            serde_json::from_str(text).map_err(|error| format!("JSON 解析失败：{error}"))?;
        if file.format != "roundtable-theme" {
            return Err(format!(
                "format 必须是 roundtable-theme，当前是 {}",
                file.format
            ));
        }
        if file.version != 1 {
            return Err(format!("version 必须是 1，当前是 {}", file.version));
        }
        if file.id != expected.file_name() {
            return Err(format!(
                "id 必须是 {}，当前是 {}",
                expected.file_name(),
                file.id
            ));
        }
        if file.name.trim().is_empty() {
            return Err("name 不能为空".to_owned());
        }

        let colors = file.colors;
        Ok(Self {
            id: expected,
            title_bg: colors.window.title_bg.0,
            menu_bg: colors.window.menu_bg.0,
            work_bg: colors.window.work_bg.0,
            desktop_bg: colors.window.desktop_bg.0,
            status_bg: colors.window.status_bg.0,
            panel_bg: colors.window.panel_bg.0,
            splitter: colors.window.splitter.0,
            scroll_track: colors.window.scroll_track.0,
            scroll_thumb: colors.window.scroll_thumb.0,
            btn_face: colors.window.btn_face.0,
            btn_hover: colors.window.btn_hover.0,
            btn_press: colors.window.btn_press.0,
            sel_bg: colors.window.sel_bg.0,
            close_hover: colors.window.close_hover.0,
            ctrl_border: colors.window.ctrl_border.0,
            line: colors.window.line.0,
            chart_line: colors.chart.chart_line.0,
            text: colors.window.text.0,
            text_white: colors.window.text_white.0,
            syn_comment: colors.window.syn_comment.0,
            chart_bg: colors.chart.chart_bg.0,
            axis_bg: colors.chart.axis_bg.0,
            tick: colors.chart.tick.0,
            cross: colors.chart.cross.0,
            cross_label: colors.chart.cross_label.0,
            ind_zero: colors.chart.ind_zero.0,
            candle_up: colors.chart.candle_up.0,
            candle_down: colors.chart.candle_down.0,
            text_up: colors.chart.text_up.0,
            text_down: colors.chart.text_down.0,
            dif: colors.indicators.dif.0,
            dea: colors.indicators.dea.0,
            kdj_k: colors.indicators.kdj_k.0,
            kdj_d: colors.indicators.kdj_d.0,
            kdj_j: colors.indicators.kdj_j.0,
            rsi_1: colors.indicators.rsi_1.0,
            rsi_2: colors.indicators.rsi_2.0,
            rsi_3: colors.indicators.rsi_3.0,
            boll_mid: colors.indicators.boll_mid.0,
            boll_up: colors.indicators.boll_up.0,
            boll_down: colors.indicators.boll_down.0,
            obv: colors.indicators.obv.0,
            cci: colors.indicators.cci.0,
            bias: [
                colors.indicators.bias[0].0,
                colors.indicators.bias[1].0,
                colors.indicators.bias[2].0,
            ],
            ma: [
                colors.indicators.ma[0].0,
                colors.indicators.ma[1].0,
                colors.indicators.ma[2].0,
                colors.indicators.ma[3].0,
                colors.indicators.ma[4].0,
                colors.indicators.ma[5].0,
            ],
            ema: [
                colors.indicators.ema[0].0,
                colors.indicators.ema[1].0,
                colors.indicators.ema[2].0,
                colors.indicators.ema[3].0,
                colors.indicators.ema[4].0,
                colors.indicators.ema[5].0,
            ],
            grid_ok: colors.debug.grid_ok.0,
            grid_bad: colors.debug.grid_bad.0,
            grid_cursor: colors.debug.grid_cursor.0,
            pointer_trail: colors.pointer.pointer_trail.0,
            pointer_trail_hot: colors.pointer.pointer_trail_hot.0,
        })
    }

    pub(crate) const fn colors(&self) -> [egui::Color32; 63] {
        [
            self.title_bg,
            self.menu_bg,
            self.work_bg,
            self.desktop_bg,
            self.status_bg,
            self.panel_bg,
            self.splitter,
            self.scroll_track,
            self.scroll_thumb,
            self.btn_face,
            self.btn_hover,
            self.btn_press,
            self.sel_bg,
            self.close_hover,
            self.ctrl_border,
            self.line,
            self.chart_line,
            self.text,
            self.text_white,
            self.syn_comment,
            self.chart_bg,
            self.axis_bg,
            self.tick,
            self.cross,
            self.cross_label,
            self.ind_zero,
            self.candle_up,
            self.candle_down,
            self.text_up,
            self.text_down,
            self.dif,
            self.dea,
            self.kdj_k,
            self.kdj_d,
            self.kdj_j,
            self.rsi_1,
            self.rsi_2,
            self.rsi_3,
            self.boll_mid,
            self.boll_up,
            self.boll_down,
            self.obv,
            self.cci,
            self.bias[0],
            self.bias[1],
            self.bias[2],
            self.ma[0],
            self.ma[1],
            self.ma[2],
            self.ma[3],
            self.ma[4],
            self.ma[5],
            self.ema[0],
            self.ema[1],
            self.ema[2],
            self.ema[3],
            self.ema[4],
            self.ema[5],
            self.grid_ok,
            self.grid_bad,
            self.grid_cursor,
            self.pointer_trail,
            self.pointer_trail_hot,
        ]
    }
}

impl ThemeId {
    pub(crate) const fn file_name(self) -> &'static str {
        match self {
            ThemeId::Classic => "light",
            ThemeId::Dark => "dark",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeFile {
    format: String,
    version: u32,
    id: String,
    name: String,
    colors: ThemeColors,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeColors {
    window: WindowColors,
    chart: ChartColors,
    indicators: IndicatorColors,
    debug: DebugColors,
    pointer: PointerColors,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WindowColors {
    title_bg: HexColor,
    menu_bg: HexColor,
    work_bg: HexColor,
    desktop_bg: HexColor,
    status_bg: HexColor,
    panel_bg: HexColor,
    splitter: HexColor,
    scroll_track: HexColor,
    scroll_thumb: HexColor,
    btn_face: HexColor,
    btn_hover: HexColor,
    btn_press: HexColor,
    sel_bg: HexColor,
    close_hover: HexColor,
    ctrl_border: HexColor,
    line: HexColor,
    text: HexColor,
    text_white: HexColor,
    syn_comment: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChartColors {
    chart_bg: HexColor,
    chart_line: HexColor,
    axis_bg: HexColor,
    tick: HexColor,
    cross: HexColor,
    cross_label: HexColor,
    ind_zero: HexColor,
    candle_up: HexColor,
    candle_down: HexColor,
    text_up: HexColor,
    text_down: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IndicatorColors {
    dif: HexColor,
    dea: HexColor,
    kdj_k: HexColor,
    kdj_d: HexColor,
    kdj_j: HexColor,
    rsi_1: HexColor,
    rsi_2: HexColor,
    rsi_3: HexColor,
    boll_mid: HexColor,
    boll_up: HexColor,
    boll_down: HexColor,
    obv: HexColor,
    cci: HexColor,
    bias: [HexColor; 3],
    ma: [HexColor; 6],
    ema: [HexColor; 6],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DebugColors {
    grid_ok: HexColor,
    grid_bad: HexColor,
    grid_cursor: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointerColors {
    pointer_trail: HexColor,
    pointer_trail_hot: HexColor,
}

struct HexColor(egui::Color32);

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        parse_hex_color(&text)
            .map(HexColor)
            .map_err(serde::de::Error::custom)
    }
}

fn parse_hex_color(text: &str) -> Result<egui::Color32, String> {
    let value = text
        .strip_prefix('#')
        .ok_or_else(|| format!("颜色 {text} 必须以 # 开头"))?;
    if value.len() != 6 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("颜色 {text} 必须是 #RRGGBB"));
    }
    let channel = |range: std::ops::Range<usize>| {
        u8::from_str_radix(&value[range], 16).map_err(|error| error.to_string())
    };
    Ok(egui::Color32::from_rgb(
        channel(0..2)?,
        channel(2..4)?,
        channel(4..6)?,
    ))
}

thread_local! {
    static CURRENT: Cell<Theme> = const { Cell::new(Theme::CLASSIC) };
}

pub(crate) fn theme() -> Theme {
    CURRENT.with(|c| c.get())
}

pub(crate) fn set_theme(t: Theme) {
    CURRENT.with(|c| c.set(t));
}

pub(crate) const fn rgb32_u32(c: egui::Color32) -> u32 {
    ((c.r() as u32) << 16) | ((c.g() as u32) << 8) | c.b() as u32
}

pub(crate) const fn invert_color(c: egui::Color32) -> egui::Color32 {
    egui::Color32::from_rgb(0xFF - c.r(), 0xFF - c.g(), 0xFF - c.b())
}

pub(crate) const fn color_is_aabbcc(c: egui::Color32) -> bool {
    c.r() % 17 == 0 && c.g() % 17 == 0 && c.b() % 17 == 0
}

const fn theme_is_aabbcc(t: Theme) -> bool {
    let cs = t.colors();
    let mut i = 0;
    while i < cs.len() {
        if !color_is_aabbcc(cs[i]) {
            return false;
        }
        i += 1;
    }
    true
}
const _: () = assert!(theme_is_aabbcc(Theme::CLASSIC));
const _: () = assert!(theme_is_aabbcc(Theme::DARK));

const _: () = assert!(
    UP_RGB_DEFAULT.0 == 0xFF
        && UP_RGB_DEFAULT.1 == 0x00
        && UP_RGB_DEFAULT.2 == 0x00
        && DOWN_RGB_DEFAULT.0 == 0x00
        && DOWN_RGB_DEFAULT.1 == 0x99
        && DOWN_RGB_DEFAULT.2 == 0x00
);

const _: () = assert!(
    Theme::DARK.ctrl_border.r() == 0xFF
        && Theme::DARK.ctrl_border.g() == 0x00
        && Theme::DARK.ctrl_border.b() == 0x00
);
const _: () = assert!(
    Theme::DARK.line.r() == 0xFF && Theme::DARK.line.g() == 0x00 && Theme::DARK.line.b() == 0x00
);
const _: () = assert!(
    Theme::DARK.text.r() == 0xFF && Theme::DARK.text.g() == 0xFF && Theme::DARK.text.b() == 0xFF
);
const _: () = assert!(
    Theme::DARK.candle_down.r() == 0x00
        && Theme::DARK.candle_down.g() == 0xFF
        && Theme::DARK.candle_down.b() == 0xFF
);
const _: () = assert!(
    Theme::DARK.close_hover.r() == 0xFF
        && Theme::DARK.close_hover.g() == 0x00
        && Theme::DARK.close_hover.b() == 0x00
);
const _: () = assert!(
    Theme::DARK.chart_line.r() == 0xFF
        && Theme::DARK.chart_line.g() == 0x00
        && Theme::DARK.chart_line.b() == 0x00
);
