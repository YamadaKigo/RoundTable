use eframe::egui;
use serde::Deserialize;
use std::cell::Cell;

pub(crate) const C_TITLE_BG: egui::Color32 = egui::Color32::from_rgb(0xBB, 0xBB, 0xBB);

pub(crate) const C_TITLE_FILL: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);

pub(crate) const C_MENU_BG: egui::Color32 = egui::Color32::from_rgb(0xDD, 0xDD, 0xDD);

pub(crate) const C_WORK_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);

pub(crate) const C_DESKTOP_BG: egui::Color32 = egui::Color32::from_rgb(0xEE, 0xEE, 0xEE);
pub(crate) const C_STATUS_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_BTN_FACE: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);

pub(crate) const C_BTN_HOVER: egui::Color32 = egui::Color32::from_rgb(0xDD, 0xDD, 0xDD);
pub(crate) const C_BORDER_RED: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
pub(crate) const C_TEXT: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00);
pub(crate) const C_TEXT_WHITE: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub(crate) const C_LINE: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);

pub(crate) const C_CTRL_BORDER: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);

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
pub(crate) const C_SYN_KEYWORD: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0xFF);

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

    pub(crate) title_text: egui::Color32,

    pub(crate) title_fill: egui::Color32,

    pub(crate) title_button_bg: egui::Color32,

    pub(crate) title_button_hover: egui::Color32,

    pub(crate) title_button_symbol: egui::Color32,

    pub(crate) title_button_border: egui::Color32,

    pub(crate) menu_bg: egui::Color32,

    pub(crate) menu_text: egui::Color32,

    pub(crate) menu_hover: egui::Color32,

    pub(crate) menu_border: egui::Color32,

    pub(crate) menu_separator: egui::Color32,

    pub(crate) menu_popup_bg: egui::Color32,

    pub(crate) menu_popup_frame_bg: egui::Color32,

    pub(crate) menu_popup_text: egui::Color32,

    pub(crate) menu_popup_hover: egui::Color32,

    pub(crate) menu_popup_border: egui::Color32,

    pub(crate) menu_popup_frame_border: egui::Color32,

    pub(crate) menu_popup_separator: egui::Color32,

    pub(crate) work_bg: egui::Color32,

    pub(crate) desktop_bg: egui::Color32,

    pub(crate) status_bg: egui::Color32,

    pub(crate) status_text: egui::Color32,

    pub(crate) status_border: egui::Color32,

    pub(crate) taskbar_bg: egui::Color32,

    pub(crate) taskbar_text: egui::Color32,

    pub(crate) taskbar_border: egui::Color32,

    pub(crate) panel_bg: egui::Color32,

    pub(crate) panel_border: egui::Color32,

    pub(crate) splitter: egui::Color32,

    pub(crate) scroll_track: egui::Color32,

    pub(crate) scroll_thumb: egui::Color32,

    pub(crate) scroll_border: egui::Color32,

    pub(crate) btn_face: egui::Color32,

    pub(crate) btn_hover: egui::Color32,

    pub(crate) btn_press: egui::Color32,

    pub(crate) button_text: egui::Color32,

    pub(crate) button_border: egui::Color32,

    pub(crate) sel_bg: egui::Color32,

    pub(crate) symbol: egui::Color32,

    pub(crate) ctrl_border: egui::Color32,

    pub(crate) window_border: egui::Color32,

    pub(crate) line: egui::Color32,

    pub(crate) chart_line: egui::Color32,

    pub(crate) text: egui::Color32,

    pub(crate) text_white: egui::Color32,

    pub(crate) syn_comment: egui::Color32,

    pub(crate) syn_keyword: egui::Color32,

    pub(crate) editor_bg: egui::Color32,

    pub(crate) editor_gutter_bg: egui::Color32,

    pub(crate) editor_text: egui::Color32,

    pub(crate) editor_line_number: egui::Color32,

    pub(crate) editor_border: egui::Color32,

    pub(crate) chart_bg: egui::Color32,

    pub(crate) axis_bg: egui::Color32,

    pub(crate) tick: egui::Color32,

    pub(crate) cross: egui::Color32,

    pub(crate) cross_label: egui::Color32,

    pub(crate) cross_label_text: egui::Color32,

    pub(crate) cross_label_border: egui::Color32,

    pub(crate) tooltip_bg: egui::Color32,

    pub(crate) tooltip_text: egui::Color32,

    pub(crate) tooltip_border: egui::Color32,

    pub(crate) input_bg: egui::Color32,

    pub(crate) input_text: egui::Color32,

    pub(crate) input_invalid: egui::Color32,

    pub(crate) input_caret: egui::Color32,

    pub(crate) input_border: egui::Color32,

    pub(crate) color_picker_bg: egui::Color32,

    pub(crate) color_picker_border: egui::Color32,

    pub(crate) color_picker_handle: egui::Color32,

    pub(crate) chart_status_bg: egui::Color32,

    pub(crate) chart_status_label: egui::Color32,

    pub(crate) chart_status_divider: egui::Color32,

    pub(crate) chart_pane_bg: egui::Color32,

    pub(crate) chart_pane_text: egui::Color32,

    pub(crate) chart_pane_hover: egui::Color32,

    pub(crate) chart_pane_border: egui::Color32,

    pub(crate) chart_pane_separator: egui::Color32,

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

    pub(crate) pointer_trail: egui::Color32,

    pub(crate) pointer_trail_hot: egui::Color32,
}

impl Theme {
    pub(crate) const CLASSIC: Self = Self {
        id: ThemeId::Classic,
        title_bg: C_TITLE_BG,
        title_text: C_TEXT,
        title_fill: C_TITLE_FILL,
        title_button_bg: C_BTN_FACE,
        title_button_hover: C_BTN_HOVER,
        title_button_symbol: C_TEXT,
        title_button_border: C_CTRL_BORDER,
        menu_bg: C_MENU_BG,
        menu_text: C_TEXT,
        menu_hover: C_BTN_PRESS,
        menu_border: C_CTRL_BORDER,
        menu_separator: C_LINE,
        menu_popup_bg: C_MENU_BG,
        menu_popup_frame_bg: C_MENU_BG,
        menu_popup_text: C_TEXT,
        menu_popup_hover: C_BTN_PRESS,
        menu_popup_border: C_CTRL_BORDER,
        menu_popup_frame_border: C_CTRL_BORDER,
        menu_popup_separator: C_LINE,
        work_bg: C_WORK_BG,
        desktop_bg: C_DESKTOP_BG,
        status_bg: C_STATUS_BG,
        status_text: C_TEXT,
        status_border: C_CTRL_BORDER,
        taskbar_bg: C_STATUS_BG,
        taskbar_text: C_TEXT,
        taskbar_border: C_CTRL_BORDER,
        panel_bg: C_PANEL_BG,
        panel_border: C_CTRL_BORDER,
        splitter: C_SPLITTER,
        scroll_track: C_SCROLL_TRACK,
        scroll_thumb: C_SCROLL_THUMB,
        scroll_border: C_CTRL_BORDER,
        btn_face: C_BTN_FACE,
        btn_hover: C_BTN_HOVER,
        btn_press: C_BTN_PRESS,
        button_text: C_TEXT,
        button_border: C_CTRL_BORDER,
        sel_bg: C_SEL_BG,
        symbol: C_TEXT,
        ctrl_border: C_CTRL_BORDER,
        window_border: C_CTRL_BORDER,
        line: C_LINE,
        chart_line: C_LINE,
        text: C_TEXT,
        text_white: C_TEXT_WHITE,
        syn_comment: C_SYN_COMMENT,
        syn_keyword: C_SYN_KEYWORD,
        editor_bg: C_WORK_BG,
        editor_gutter_bg: C_PANEL_BG,
        editor_text: C_TEXT,
        editor_line_number: C_CROSS,
        editor_border: C_CTRL_BORDER,
        chart_bg: C_CHART_BG,
        axis_bg: C_AXIS_BG,
        tick: C_TICK,
        cross: C_CROSS,
        cross_label: C_CROSS_LABEL,
        cross_label_text: C_TEXT,
        cross_label_border: C_LINE,
        tooltip_bg: C_CROSS_LABEL,
        tooltip_text: C_TEXT,
        tooltip_border: C_CTRL_BORDER,
        input_bg: C_WORK_BG,
        input_text: C_TEXT,
        input_invalid: C_UP,
        input_caret: C_TEXT,
        input_border: C_CTRL_BORDER,
        color_picker_bg: C_WORK_BG,
        color_picker_border: C_CTRL_BORDER,
        color_picker_handle: C_TEXT_WHITE,
        chart_status_bg: C_STATUS_BG,
        chart_status_label: C_TEXT,
        chart_status_divider: C_LINE,
        chart_pane_bg: C_MENU_BG,
        chart_pane_text: C_TEXT,
        chart_pane_hover: C_BTN_PRESS,
        chart_pane_border: C_CTRL_BORDER,
        chart_pane_separator: C_LINE,
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
        pointer_trail: C_POINTER_TRAIL,
        pointer_trail_hot: C_POINTER_TRAIL_HOT,
    };

    pub(crate) const DARK: Self = Self::dark_mirror();

    const fn dark_mirror() -> Self {
        let l = Self::CLASSIC;
        let mut t = l;
        t.id = ThemeId::Dark;
        t.title_bg = invert_color(l.title_bg);
        t.title_fill = egui::Color32::from_rgb(0x66, 0x66, 0x66);
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
        t.syn_comment = C_SYN_COMMENT;
        t.syn_keyword = egui::Color32::from_rgb(0x00, 0xFF, 0xFF);
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
        t.symbol = C_TEXT_WHITE;
        t.text_white = C_TEXT_WHITE;
        t.title_text = t.text;
        t.title_button_bg = t.btn_face;
        t.title_button_hover = t.btn_hover;
        t.title_button_symbol = t.symbol;
        t.title_button_border = t.ctrl_border;
        t.menu_text = t.text;
        t.menu_hover = t.btn_press;
        t.menu_border = t.ctrl_border;
        t.menu_separator = t.line;
        t.menu_popup_bg = t.menu_bg;
        t.menu_popup_frame_bg = t.menu_bg;
        t.menu_popup_text = t.menu_text;
        t.menu_popup_hover = t.menu_hover;
        t.menu_popup_border = t.menu_border;
        t.menu_popup_frame_border = t.menu_border;
        t.menu_popup_separator = t.menu_separator;
        t.status_text = t.text;
        t.status_border = t.ctrl_border;
        t.taskbar_bg = t.status_bg;
        t.taskbar_text = t.text;
        t.taskbar_border = t.ctrl_border;
        t.panel_border = t.ctrl_border;
        t.scroll_border = t.ctrl_border;
        t.button_text = t.text;
        t.button_border = t.ctrl_border;
        t.window_border = t.ctrl_border;
        t.editor_bg = t.work_bg;
        t.editor_gutter_bg = t.panel_bg;
        t.editor_text = t.text;
        t.editor_line_number = t.cross;
        t.editor_border = t.ctrl_border;
        t.cross_label_text = t.text;
        t.cross_label_border = t.line;
        t.tooltip_text = t.text;
        t.tooltip_border = t.ctrl_border;
        t.input_bg = t.work_bg;
        t.input_text = t.text;
        t.input_invalid = t.candle_up;
        t.input_caret = t.text;
        t.input_border = t.ctrl_border;
        t.color_picker_bg = t.work_bg;
        t.color_picker_border = t.ctrl_border;
        t.color_picker_handle = C_TEXT_WHITE;
        t.chart_status_bg = t.status_bg;
        t.chart_status_label = t.text;
        t.chart_status_divider = t.line;
        t.chart_pane_bg = t.menu_bg;
        t.chart_pane_text = t.menu_text;
        t.chart_pane_hover = t.menu_hover;
        t.chart_pane_border = t.menu_border;
        t.chart_pane_separator = t.menu_separator;
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
        if file.version != 4 {
            return Err(format!("version 必须是 4，当前是 {}", file.version));
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
            title_bg: colors.application.title_bar.background.0,
            title_text: colors.application.title_bar.text.0,
            title_fill: colors.application.title_bar.pattern.0,
            title_button_bg: colors.application.title_bar.button.background.0,
            title_button_hover: colors.application.title_bar.button.hover.0,
            title_button_symbol: colors.application.title_bar.button.symbol.0,
            title_button_border: colors.application.title_bar.button.border.0,
            menu_bg: colors.application.menu_bar.background.0,
            menu_text: colors.application.menu_bar.text.0,
            menu_hover: colors.application.menu_bar.hover.0,
            menu_border: colors.application.menu_bar.border.0,
            menu_separator: colors.application.menu_bar.separator.0,
            menu_popup_bg: colors.application.menu_bar.popup.background.0,
            menu_popup_frame_bg: colors.application.menu_bar.popup.frame_background.0,
            menu_popup_text: colors.application.menu_bar.popup.text.0,
            menu_popup_hover: colors.application.menu_bar.popup.hover.0,
            menu_popup_border: colors.application.menu_bar.popup.border.0,
            menu_popup_frame_border: colors.application.menu_bar.popup.frame_border.0,
            menu_popup_separator: colors.application.menu_bar.popup.separator.0,
            work_bg: colors.window.background.0,
            desktop_bg: colors.application.workspace.background.0,
            status_bg: colors.application.status_bar.background.0,
            status_text: colors.application.status_bar.text.0,
            status_border: colors.application.status_bar.border.0,
            taskbar_bg: colors.application.taskbar.background.0,
            taskbar_text: colors.application.taskbar.text.0,
            taskbar_border: colors.application.taskbar.border.0,
            panel_bg: colors.window.panel.background.0,
            panel_border: colors.window.panel.border.0,
            splitter: colors.window.splitter.0,
            scroll_track: colors.controls.scrollbar.track.0,
            scroll_thumb: colors.controls.scrollbar.thumb.0,
            scroll_border: colors.controls.scrollbar.border.0,
            btn_face: colors.controls.button.background.0,
            btn_hover: colors.controls.button.hover.0,
            btn_press: colors.controls.button.pressed.0,
            button_text: colors.controls.button.text.0,
            button_border: colors.controls.button.border.0,
            sel_bg: colors.controls.selection.background.0,
            symbol: colors.window.symbol.0,
            ctrl_border: colors.controls.border.0,
            window_border: colors.window.border.0,
            line: colors.application.menu_bar.separator.0,
            chart_line: colors.chart.border.0,
            text: colors.window.text.0,
            text_white: colors.controls.color_picker.handle.0,
            syn_comment: colors.editor.syntax.comment.0,
            syn_keyword: colors.editor.syntax.custom_name.0,
            editor_bg: colors.editor.background.0,
            editor_gutter_bg: colors.editor.gutter.background.0,
            editor_text: colors.editor.text.0,
            editor_line_number: colors.editor.gutter.line_number.0,
            editor_border: colors.editor.border.0,
            chart_bg: colors.chart.background.0,
            axis_bg: colors.chart.axis.background.0,
            tick: colors.chart.axis.tick.0,
            cross: colors.chart.crosshair.line.0,
            cross_label: colors.chart.crosshair.label.background.0,
            cross_label_text: colors.chart.crosshair.label.text.0,
            cross_label_border: colors.chart.crosshair.label.border.0,
            tooltip_bg: colors.controls.tooltip.background.0,
            tooltip_text: colors.controls.tooltip.text.0,
            tooltip_border: colors.controls.tooltip.border.0,
            input_bg: colors.controls.text_input.background.0,
            input_text: colors.controls.text_input.text.0,
            input_invalid: colors.controls.text_input.invalid_text.0,
            input_caret: colors.controls.text_input.caret.0,
            input_border: colors.controls.text_input.border.0,
            color_picker_bg: colors.controls.color_picker.background.0,
            color_picker_border: colors.controls.color_picker.border.0,
            color_picker_handle: colors.controls.color_picker.handle.0,
            chart_status_bg: colors.chart.status.background.0,
            chart_status_label: colors.chart.status.label.0,
            chart_status_divider: colors.chart.status.divider.0,
            chart_pane_bg: colors.chart.pane_menu.background.0,
            chart_pane_text: colors.chart.pane_menu.text.0,
            chart_pane_hover: colors.chart.pane_menu.hover.0,
            chart_pane_border: colors.chart.pane_menu.border.0,
            chart_pane_separator: colors.chart.pane_menu.separator.0,
            ind_zero: colors.chart.zero_line.0,
            candle_up: colors.chart.candle.up.0,
            candle_down: colors.chart.candle.down.0,
            text_up: colors.chart.price_text.up.0,
            text_down: colors.chart.price_text.down.0,
            dif: colors.indicators.macd.dif.0,
            dea: colors.indicators.macd.dea.0,
            kdj_k: colors.indicators.kdj.k.0,
            kdj_d: colors.indicators.kdj.d.0,
            kdj_j: colors.indicators.kdj.j.0,
            rsi_1: colors.indicators.rsi.line_1.0,
            rsi_2: colors.indicators.rsi.line_2.0,
            rsi_3: colors.indicators.rsi.line_3.0,
            boll_mid: colors.indicators.boll.mid.0,
            boll_up: colors.indicators.boll.upper.0,
            boll_down: colors.indicators.boll.lower.0,
            obv: colors.indicators.obv.line.0,
            cci: colors.indicators.cci.line.0,
            bias: [
                colors.indicators.bias.line_1.0,
                colors.indicators.bias.line_2.0,
                colors.indicators.bias.line_3.0,
            ],
            ma: [
                colors.indicators.ma.line_1.0,
                colors.indicators.ma.line_2.0,
                colors.indicators.ma.line_3.0,
                colors.indicators.ma.line_4.0,
                colors.indicators.ma.line_5.0,
                colors.indicators.ma.line_6.0,
            ],
            ema: [
                colors.indicators.ema.line_1.0,
                colors.indicators.ema.line_2.0,
                colors.indicators.ema.line_3.0,
                colors.indicators.ema.line_4.0,
                colors.indicators.ema.line_5.0,
                colors.indicators.ema.line_6.0,
            ],
            pointer_trail: colors.pointer.trail.normal.0,
            pointer_trail_hot: colors.pointer.trail.hot.0,
        })
    }

    pub(crate) const fn colors(&self) -> [egui::Color32; 114] {
        [
            self.title_bg,
            self.title_text,
            self.title_fill,
            self.title_button_bg,
            self.title_button_hover,
            self.title_button_symbol,
            self.title_button_border,
            self.menu_bg,
            self.menu_text,
            self.menu_hover,
            self.menu_border,
            self.menu_separator,
            self.menu_popup_bg,
            self.menu_popup_frame_bg,
            self.menu_popup_text,
            self.menu_popup_hover,
            self.menu_popup_border,
            self.menu_popup_frame_border,
            self.menu_popup_separator,
            self.work_bg,
            self.desktop_bg,
            self.status_bg,
            self.status_text,
            self.status_border,
            self.taskbar_bg,
            self.taskbar_text,
            self.taskbar_border,
            self.panel_bg,
            self.panel_border,
            self.splitter,
            self.scroll_track,
            self.scroll_thumb,
            self.scroll_border,
            self.btn_face,
            self.btn_hover,
            self.btn_press,
            self.button_text,
            self.button_border,
            self.sel_bg,
            self.symbol,
            self.ctrl_border,
            self.window_border,
            self.line,
            self.chart_line,
            self.text,
            self.text_white,
            self.syn_comment,
            self.syn_keyword,
            self.editor_bg,
            self.editor_gutter_bg,
            self.editor_text,
            self.editor_line_number,
            self.editor_border,
            self.chart_bg,
            self.axis_bg,
            self.tick,
            self.cross,
            self.cross_label,
            self.cross_label_text,
            self.cross_label_border,
            self.tooltip_bg,
            self.tooltip_text,
            self.tooltip_border,
            self.input_bg,
            self.input_text,
            self.input_invalid,
            self.input_caret,
            self.input_border,
            self.color_picker_bg,
            self.color_picker_border,
            self.color_picker_handle,
            self.chart_status_bg,
            self.chart_status_label,
            self.chart_status_divider,
            self.chart_pane_bg,
            self.chart_pane_text,
            self.chart_pane_hover,
            self.chart_pane_border,
            self.chart_pane_separator,
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
    application: ApplicationColors,
    window: WindowColors,
    controls: ControlColors,
    editor: EditorColors,
    chart: ChartColors,
    indicators: IndicatorColors,
    pointer: PointerColors,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplicationColors {
    title_bar: TitleBarColors,
    menu_bar: MenuBarColors,
    workspace: WorkspaceColors,
    taskbar: BarColors,
    status_bar: BarColors,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TitleBarColors {
    background: HexColor,
    pattern: HexColor,
    text: HexColor,
    button: TitleButtonColors,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TitleButtonColors {
    background: HexColor,
    hover: HexColor,
    symbol: HexColor,
    border: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MenuBarColors {
    background: HexColor,
    text: HexColor,
    hover: HexColor,
    border: HexColor,
    separator: HexColor,
    popup: MenuPopupColors,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MenuPopupColors {
    background: HexColor,
    frame_background: HexColor,
    text: HexColor,
    hover: HexColor,
    border: HexColor,
    frame_border: HexColor,
    separator: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceColors {
    background: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BarColors {
    background: HexColor,
    text: HexColor,
    border: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WindowColors {
    background: HexColor,
    panel: PanelColors,
    border: HexColor,
    splitter: HexColor,
    symbol: HexColor,
    text: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelColors {
    background: HexColor,
    border: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlColors {
    button: ButtonColors,
    border: HexColor,
    selection: SelectionColors,
    text_input: TextInputColors,
    scrollbar: ScrollbarColors,
    tooltip: TooltipColors,
    color_picker: ColorPickerColors,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ButtonColors {
    background: HexColor,
    hover: HexColor,
    pressed: HexColor,
    text: HexColor,
    border: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectionColors {
    background: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextInputColors {
    background: HexColor,
    text: HexColor,
    invalid_text: HexColor,
    caret: HexColor,
    border: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScrollbarColors {
    track: HexColor,
    thumb: HexColor,
    border: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TooltipColors {
    background: HexColor,
    text: HexColor,
    border: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ColorPickerColors {
    background: HexColor,
    border: HexColor,
    handle: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditorColors {
    background: HexColor,
    text: HexColor,
    border: HexColor,
    gutter: GutterColors,
    syntax: SyntaxColors,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GutterColors {
    background: HexColor,
    line_number: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SyntaxColors {
    comment: HexColor,
    custom_name: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChartColors {
    background: HexColor,
    border: HexColor,
    axis: AxisColors,
    crosshair: CrosshairColors,
    status: StatusFieldColors,
    zero_line: HexColor,
    candle: CandleColors,
    price_text: PriceTextColors,
    pane_menu: ChartPaneMenuColors,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChartPaneMenuColors {
    background: HexColor,
    text: HexColor,
    hover: HexColor,
    border: HexColor,
    separator: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AxisColors {
    background: HexColor,
    tick: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CrosshairColors {
    line: HexColor,
    label: LabelColors,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LabelColors {
    background: HexColor,
    text: HexColor,
    border: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusFieldColors {
    background: HexColor,
    label: HexColor,
    divider: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CandleColors {
    up: HexColor,
    down: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PriceTextColors {
    up: HexColor,
    down: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IndicatorColors {
    macd: MacdColors,
    kdj: KdjColors,
    rsi: RsiColors,
    boll: BollColors,
    obv: LineColor,
    cci: LineColor,
    bias: BiasColors,
    ma: MaColors,
    ema: EmaColors,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MacdColors {
    dif: HexColor,
    dea: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KdjColors {
    k: HexColor,
    d: HexColor,
    j: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RsiColors {
    line_1: HexColor,
    line_2: HexColor,
    line_3: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BollColors {
    mid: HexColor,
    upper: HexColor,
    lower: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LineColor {
    line: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BiasColors {
    line_1: HexColor,
    line_2: HexColor,
    line_3: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaColors {
    line_1: HexColor,
    line_2: HexColor,
    line_3: HexColor,
    line_4: HexColor,
    line_5: HexColor,
    line_6: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmaColors {
    line_1: HexColor,
    line_2: HexColor,
    line_3: HexColor,
    line_4: HexColor,
    line_5: HexColor,
    line_6: HexColor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointerColors {
    trail: PointerTrailColors,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointerTrailColors {
    normal: HexColor,
    hot: HexColor,
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
    Theme::DARK.symbol.r() == 0xFF
        && Theme::DARK.symbol.g() == 0xFF
        && Theme::DARK.symbol.b() == 0xFF
);
const _: () = assert!(
    Theme::DARK.chart_line.r() == 0xFF
        && Theme::DARK.chart_line.g() == 0x00
        && Theme::DARK.chart_line.b() == 0x00
);
