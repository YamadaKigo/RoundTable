use crate::engine::kit;
use std::cell::Cell;

pub(crate) const C_TITLE_BG: kit::Color32 = kit::Color32::from_rgb(0xBB, 0xBB, 0xBB);

pub(crate) const C_TITLE_FILL: kit::Color32 = kit::Color32::from_rgb(0x99, 0x99, 0x99);

pub(crate) const C_MENU_BG: kit::Color32 = kit::Color32::from_rgb(0xDD, 0xDD, 0xDD);

pub(crate) const C_WORK_BG: kit::Color32 = kit::Color32::from_rgb(0xFF, 0xFF, 0xFF);

pub(crate) const C_DESKTOP_BG: kit::Color32 = kit::Color32::from_rgb(0xEE, 0xEE, 0xEE);
pub(crate) const C_STATUS_BG: kit::Color32 = kit::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_BTN_FACE: kit::Color32 = kit::Color32::from_rgb(0xCC, 0xCC, 0xCC);

pub(crate) const C_BTN_HOVER: kit::Color32 = kit::Color32::from_rgb(0xDD, 0xDD, 0xDD);
pub(crate) const C_TEXT: kit::Color32 = kit::Color32::from_rgb(0x00, 0x00, 0x00);
pub(crate) const C_TEXT_WHITE: kit::Color32 = kit::Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub(crate) const C_LINE: kit::Color32 = kit::Color32::from_rgb(0x99, 0x99, 0x99);

pub(crate) const C_CTRL_BORDER: kit::Color32 = kit::Color32::from_rgb(0x99, 0x99, 0x99);

pub(crate) const C_CHART_BG: kit::Color32 = kit::Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub(crate) const C_AXIS_BG: kit::Color32 = kit::Color32::from_rgb(0xDD, 0xDD, 0xDD);
pub(crate) const C_UP: kit::Color32 = kit::Color32::from_rgb(0xFF, 0x00, 0x00);
pub(crate) const C_DOWN: kit::Color32 = kit::Color32::from_rgb(0x00, 0x99, 0x00);
pub(crate) const C_TICK: kit::Color32 = kit::Color32::from_rgb(0xDD, 0xDD, 0xDD);

pub(crate) const UP_RGB_DEFAULT: (u8, u8, u8) = (C_UP.r(), C_UP.g(), C_UP.b());
pub(crate) const DOWN_RGB_DEFAULT: (u8, u8, u8) = (C_DOWN.r(), C_DOWN.g(), C_DOWN.b());

pub(crate) const C_CROSS: kit::Color32 = kit::Color32::from_rgb(0x66, 0x66, 0x66);
pub(crate) const C_CROSS_LABEL: kit::Color32 = kit::Color32::from_rgb(0xEE, 0xEE, 0xEE);

pub(crate) const C_SPLITTER: kit::Color32 = kit::Color32::from_rgb(0xAA, 0xAA, 0xAA);

pub(crate) const C_SCROLL_TRACK: kit::Color32 = kit::Color32::from_rgb(0xAA, 0xAA, 0xAA);
pub(crate) const C_SCROLL_THUMB: kit::Color32 = kit::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_PANEL_BG: kit::Color32 = kit::Color32::from_rgb(0xDD, 0xDD, 0xDD);
pub(crate) const C_BTN_PRESS: kit::Color32 = kit::Color32::from_rgb(0xAA, 0xAA, 0xAA);

pub(crate) const C_SEL_BG: kit::Color32 = kit::Color32::from_rgb(0xDD, 0xDD, 0xDD);

pub(crate) const C_SYN_COMMENT: kit::Color32 = kit::Color32::from_rgb(0x00, 0x99, 0x00);
pub(crate) const C_SYN_KEYWORD: kit::Color32 = kit::Color32::from_rgb(0x00, 0x00, 0xFF);

pub(crate) const C_POINTER_TRAIL: kit::Color32 = kit::Color32::from_rgb(0x00, 0xCC, 0xCC);
pub(crate) const C_POINTER_TRAIL_HOT: kit::Color32 = kit::Color32::from_rgb(0xFF, 0x00, 0xFF);

pub(crate) const C_IND_ZERO: kit::Color32 = kit::Color32::from_rgb(0x99, 0x99, 0x99);

pub(crate) const C_DIF: kit::Color32 = kit::Color32::from_rgb(0x33, 0x33, 0xCC);
pub(crate) const C_DEA: kit::Color32 = kit::Color32::from_rgb(0xCC, 0x99, 0x00);

pub(crate) const C_MA_CLASSIC: [kit::Color32; 6] = [
    kit::Color32::from_rgb(0x33, 0x33, 0xCC),
    kit::Color32::from_rgb(0xCC, 0x00, 0x66),
    kit::Color32::from_rgb(0x00, 0x99, 0x33),
    kit::Color32::from_rgb(0x66, 0x00, 0xCC),
    kit::Color32::from_rgb(0x00, 0x99, 0x99),
    kit::Color32::from_rgb(0x66, 0x66, 0x66),
];

pub(crate) const C_EMA_CLASSIC: [kit::Color32; 6] = [
    kit::Color32::from_rgb(0xFF, 0x00, 0xFF),
    kit::Color32::from_rgb(0xFF, 0x88, 0x00),
    kit::Color32::from_rgb(0x00, 0xCC, 0xCC),
    kit::Color32::from_rgb(0x88, 0xAA, 0x00),
    kit::Color32::from_rgb(0x88, 0x88, 0xFF),
    kit::Color32::from_rgb(0xAA, 0x00, 0xAA),
];

pub(crate) const C_TEXT_UP: kit::Color32 = kit::Color32::from_rgb(0xFF, 0x00, 0x00);
pub(crate) const C_TEXT_DOWN: kit::Color32 = kit::Color32::from_rgb(0x00, 0x99, 0x00);

pub(crate) const C_KDJ_K: kit::Color32 = kit::Color32::from_rgb(0x33, 0x33, 0xCC);
pub(crate) const C_KDJ_D: kit::Color32 = kit::Color32::from_rgb(0xCC, 0x99, 0x00);
pub(crate) const C_KDJ_J: kit::Color32 = kit::Color32::from_rgb(0xFF, 0x00, 0xFF);
pub(crate) const C_RSI_1: kit::Color32 = kit::Color32::from_rgb(0x33, 0x33, 0xCC);
pub(crate) const C_RSI_2: kit::Color32 = kit::Color32::from_rgb(0xCC, 0x99, 0x00);
pub(crate) const C_RSI_3: kit::Color32 = kit::Color32::from_rgb(0xFF, 0x00, 0xFF);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ThemeId {
    Classic,
}

trait ThemeColorValue: Copy {
    fn read(colors: &crate::engine::core::json::Json, paths: &[&str]) -> Result<Self, String>;
}

impl ThemeColorValue for kit::Color32 {
    fn read(colors: &crate::engine::core::json::Json, paths: &[&str]) -> Result<Self, String> {
        colors.color_at(paths[0])
    }
}

impl<const N: usize> ThemeColorValue for [kit::Color32; N] {
    fn read(colors: &crate::engine::core::json::Json, paths: &[&str]) -> Result<Self, String> {
        if paths.len() != N {
            return Err(format!(
                "theme color path count mismatch: {} != {N}",
                paths.len()
            ));
        }
        let mut out = [kit::Color32::BLACK; N];
        for (slot, path) in out.iter_mut().zip(paths) {
            *slot = colors.color_at(path)?;
        }
        Ok(out)
    }
}

macro_rules! define_theme {
    ($($field:ident: $ty:ty = $paths:expr => $default:expr;)+) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub(crate) struct Theme {
            pub(crate) id: ThemeId,
            $(pub(crate) $field: $ty,)+
        }

        impl Theme {
            pub(crate) const CLASSIC: Self = Self {
                id: ThemeId::Classic,
                $($field: $default,)+
            };

            pub(crate) const fn of(id: ThemeId) -> Self {
                match id {
                    ThemeId::Classic => Self::CLASSIC,
                }
            }

            pub(crate) fn name(self) -> &'static str {
                match self.id {
                    ThemeId::Classic => "Light",
                }
            }

            pub(crate) fn load(id: ThemeId) -> Self {
                let path = crate::platform::assets::theme_file(id.file_name());
                let text = match std::fs::read_to_string(&path) {
                    Ok(text) => text,
                    Err(error) => {
                        eprintln!(
                            "[theme] Cannot read {}: {error}; using built-in colors",
                            path.display()
                        );
                        return Self::of(id);
                    }
                };
                match Self::from_json(&text, id) {
                    Ok(theme) => theme,
                    Err(error) => {
                        eprintln!(
                            "[theme] Invalid {}: {error}; using built-in colors",
                            path.display()
                        );
                        Self::of(id)
                    }
                }
            }

            pub(crate) fn from_json(text: &str, expected: ThemeId) -> Result<Self, String> {
                let file = crate::engine::core::json::Json::parse(text)?;
                if file.string_at("format")? != "roundtable-theme" {
                    return Err(format!(
                        "format must be roundtable-theme, got {}",
                        file.string_at("format")?
                    ));
                }
                if file.u32_at("version")? != 4 {
                    return Err(format!("version must be 4, got {}", file.u32_at("version")?));
                }
                if file.string_at("id")? != expected.file_name() {
                    return Err(format!(
                        "id must be {}, got {}",
                        expected.file_name(),
                        file.string_at("id")?
                    ));
                }
                if file.string_at("name")?.trim().is_empty() {
                    return Err("name must not be empty".to_owned());
                }
                let colors = file.get("colors").ok_or_else(|| "missing colors".to_owned())?;
                Ok(Self {
                    id: expected,
                    $($field: <$ty as ThemeColorValue>::read(colors, $paths)?,)+
                })
            }

        }

        define_theme! {
            @fields
            $($field: $ty = $paths => $default;)+
        }
    };
    (@fields $($tail:tt)*) => {};
}

define_theme! {
    title_bg: kit::Color32 = &["application.title_bar.background"] => C_TITLE_BG;
    title_text: kit::Color32 = &["application.title_bar.text"] => C_TEXT;
    title_fill: kit::Color32 = &["application.title_bar.pattern"] => C_TITLE_FILL;
    title_button_bg: kit::Color32 = &["application.title_bar.button.background"] => C_BTN_FACE;
    title_button_hover: kit::Color32 = &["application.title_bar.button.hover"] => C_BTN_HOVER;
    title_button_symbol: kit::Color32 = &["application.title_bar.button.symbol"] => C_TEXT;
    title_button_border: kit::Color32 = &["application.title_bar.button.border"] => C_CTRL_BORDER;
    menu_bg: kit::Color32 = &["application.menu_bar.background"] => C_MENU_BG;
    menu_text: kit::Color32 = &["application.menu_bar.text"] => C_TEXT;
    menu_hover: kit::Color32 = &["application.menu_bar.hover"] => C_BTN_PRESS;
    menu_border: kit::Color32 = &["application.menu_bar.border"] => C_CTRL_BORDER;
    menu_separator: kit::Color32 = &["application.menu_bar.separator"] => C_LINE;
    menu_popup_bg: kit::Color32 = &["application.menu_bar.popup.background"] => C_MENU_BG;
    menu_popup_frame_bg: kit::Color32 = &["application.menu_bar.popup.frame_background"] => C_MENU_BG;
    menu_popup_text: kit::Color32 = &["application.menu_bar.popup.text"] => C_TEXT;
    menu_popup_hover: kit::Color32 = &["application.menu_bar.popup.hover"] => C_BTN_PRESS;
    menu_popup_border: kit::Color32 = &["application.menu_bar.popup.border"] => C_CTRL_BORDER;
    menu_popup_frame_border: kit::Color32 = &["application.menu_bar.popup.frame_border"] => C_CTRL_BORDER;
    menu_popup_separator: kit::Color32 = &["application.menu_bar.popup.separator"] => C_LINE;
    work_bg: kit::Color32 = &["window.background"] => C_WORK_BG;
    desktop_bg: kit::Color32 = &["application.workspace.background"] => C_DESKTOP_BG;
    status_bg: kit::Color32 = &["application.status_bar.background"] => C_STATUS_BG;
    status_text: kit::Color32 = &["application.status_bar.text"] => C_TEXT;
    status_border: kit::Color32 = &["application.status_bar.border"] => C_CTRL_BORDER;
    taskbar_bg: kit::Color32 = &["application.taskbar.background"] => C_STATUS_BG;
    taskbar_text: kit::Color32 = &["application.taskbar.text"] => C_TEXT;
    taskbar_border: kit::Color32 = &["application.taskbar.border"] => C_CTRL_BORDER;
    panel_bg: kit::Color32 = &["window.panel.background"] => C_PANEL_BG;
    panel_border: kit::Color32 = &["window.panel.border"] => C_CTRL_BORDER;
    splitter: kit::Color32 = &["window.splitter"] => C_SPLITTER;
    scroll_track: kit::Color32 = &["controls.scrollbar.track"] => C_SCROLL_TRACK;
    scroll_thumb: kit::Color32 = &["controls.scrollbar.thumb"] => C_SCROLL_THUMB;
    scroll_border: kit::Color32 = &["controls.scrollbar.border"] => C_CTRL_BORDER;
    btn_face: kit::Color32 = &["controls.button.background"] => C_BTN_FACE;
    btn_hover: kit::Color32 = &["controls.button.hover"] => C_BTN_HOVER;
    btn_press: kit::Color32 = &["controls.button.pressed"] => C_BTN_PRESS;
    button_text: kit::Color32 = &["controls.button.text"] => C_TEXT;
    button_border: kit::Color32 = &["controls.button.border"] => C_CTRL_BORDER;
    sel_bg: kit::Color32 = &["controls.selection.background"] => C_SEL_BG;
    symbol: kit::Color32 = &["window.symbol"] => C_TEXT;
    ctrl_border: kit::Color32 = &["controls.border"] => C_CTRL_BORDER;
    window_border: kit::Color32 = &["window.border"] => C_CTRL_BORDER;
    line: kit::Color32 = &["application.menu_bar.separator"] => C_LINE;
    chart_line: kit::Color32 = &["chart.border"] => C_LINE;
    text: kit::Color32 = &["window.text"] => C_TEXT;
    text_white: kit::Color32 = &["controls.color_picker.handle"] => C_TEXT_WHITE;
    syn_comment: kit::Color32 = &["editor.syntax.comment"] => C_SYN_COMMENT;
    syn_keyword: kit::Color32 = &["editor.syntax.custom_name"] => C_SYN_KEYWORD;
    editor_bg: kit::Color32 = &["editor.background"] => C_WORK_BG;
    editor_gutter_bg: kit::Color32 = &["editor.gutter.background"] => C_PANEL_BG;
    editor_text: kit::Color32 = &["editor.text"] => C_TEXT;
    editor_line_number: kit::Color32 = &["editor.gutter.line_number"] => C_CROSS;
    editor_border: kit::Color32 = &["editor.border"] => C_CTRL_BORDER;
    chart_bg: kit::Color32 = &["chart.background"] => C_CHART_BG;
    axis_bg: kit::Color32 = &["chart.axis.background"] => C_AXIS_BG;
    tick: kit::Color32 = &["chart.axis.tick"] => C_TICK;
    cross: kit::Color32 = &["chart.crosshair.line"] => C_CROSS;
    cross_label: kit::Color32 = &["chart.crosshair.label.background"] => C_CROSS_LABEL;
    cross_label_text: kit::Color32 = &["chart.crosshair.label.text"] => C_TEXT;
    cross_label_border: kit::Color32 = &["chart.crosshair.label.border"] => C_LINE;
    tooltip_bg: kit::Color32 = &["controls.tooltip.background"] => C_CROSS_LABEL;
    tooltip_text: kit::Color32 = &["controls.tooltip.text"] => C_TEXT;
    tooltip_border: kit::Color32 = &["controls.tooltip.border"] => C_CTRL_BORDER;
    input_bg: kit::Color32 = &["controls.text_input.background"] => C_WORK_BG;
    input_text: kit::Color32 = &["controls.text_input.text"] => C_TEXT;
    input_invalid: kit::Color32 = &["controls.text_input.invalid_text"] => C_UP;
    input_caret: kit::Color32 = &["controls.text_input.caret"] => C_TEXT;
    input_border: kit::Color32 = &["controls.text_input.border"] => C_CTRL_BORDER;
    color_picker_bg: kit::Color32 = &["controls.color_picker.background"] => C_WORK_BG;
    color_picker_border: kit::Color32 = &["controls.color_picker.border"] => C_CTRL_BORDER;
    color_picker_handle: kit::Color32 = &["controls.color_picker.handle"] => C_TEXT_WHITE;
    chart_status_bg: kit::Color32 = &["chart.status.background"] => C_STATUS_BG;
    chart_status_label: kit::Color32 = &["chart.status.label"] => C_TEXT;
    chart_status_divider: kit::Color32 = &["chart.status.divider"] => C_LINE;
    chart_pane_bg: kit::Color32 = &["chart.pane_menu.background"] => C_MENU_BG;
    chart_pane_text: kit::Color32 = &["chart.pane_menu.text"] => C_TEXT;
    chart_pane_hover: kit::Color32 = &["chart.pane_menu.hover"] => C_BTN_PRESS;
    chart_pane_border: kit::Color32 = &["chart.pane_menu.border"] => C_CTRL_BORDER;
    chart_pane_separator: kit::Color32 = &["chart.pane_menu.separator"] => C_LINE;
    ind_zero: kit::Color32 = &["chart.zero_line"] => C_IND_ZERO;
    candle_up: kit::Color32 = &["chart.candle.up"] => C_UP;
    candle_down: kit::Color32 = &["chart.candle.down"] => C_DOWN;
    text_up: kit::Color32 = &["chart.price_text.up"] => C_TEXT_UP;
    text_down: kit::Color32 = &["chart.price_text.down"] => C_TEXT_DOWN;
    dif: kit::Color32 = &["indicators.macd.dif"] => C_DIF;
    dea: kit::Color32 = &["indicators.macd.dea"] => C_DEA;
    kdj_k: kit::Color32 = &["indicators.kdj.k"] => C_KDJ_K;
    kdj_d: kit::Color32 = &["indicators.kdj.d"] => C_KDJ_D;
    kdj_j: kit::Color32 = &["indicators.kdj.j"] => C_KDJ_J;
    rsi_1: kit::Color32 = &["indicators.rsi.line_1"] => C_RSI_1;
    rsi_2: kit::Color32 = &["indicators.rsi.line_2"] => C_RSI_2;
    rsi_3: kit::Color32 = &["indicators.rsi.line_3"] => C_RSI_3;
    boll_mid: kit::Color32 = &["indicators.boll.mid"] => kit::Color32::from_rgb(0x33, 0x66, 0xCC);
    boll_up: kit::Color32 = &["indicators.boll.upper"] => kit::Color32::from_rgb(0xCC, 0x00, 0x66);
    boll_down: kit::Color32 = &["indicators.boll.lower"] => kit::Color32::from_rgb(0x00, 0x99, 0x99);
    obv: kit::Color32 = &["indicators.obv.line"] => kit::Color32::from_rgb(0xCC, 0x66, 0x00);
    cci: kit::Color32 = &["indicators.cci.line"] => kit::Color32::from_rgb(0x66, 0x00, 0xCC);
    bias: [kit::Color32; 3] = &[
        "indicators.bias.line_1",
        "indicators.bias.line_2",
        "indicators.bias.line_3",
    ] => [
        kit::Color32::from_rgb(0x33, 0x33, 0xCC),
        kit::Color32::from_rgb(0xCC, 0x99, 0x00),
        kit::Color32::from_rgb(0xFF, 0x00, 0xFF),
    ];
    ma: [kit::Color32; 6] = &[
        "indicators.ma.line_1",
        "indicators.ma.line_2",
        "indicators.ma.line_3",
        "indicators.ma.line_4",
        "indicators.ma.line_5",
        "indicators.ma.line_6",
    ] => C_MA_CLASSIC;
    ema: [kit::Color32; 6] = &[
        "indicators.ema.line_1",
        "indicators.ema.line_2",
        "indicators.ema.line_3",
        "indicators.ema.line_4",
        "indicators.ema.line_5",
        "indicators.ema.line_6",
    ] => C_EMA_CLASSIC;
    pointer_trail: kit::Color32 = &["pointer.trail.normal"] => C_POINTER_TRAIL;
    pointer_trail_hot: kit::Color32 = &["pointer.trail.hot"] => C_POINTER_TRAIL_HOT;
}
impl ThemeId {
    pub(crate) const fn file_name(self) -> &'static str {
        match self {
            ThemeId::Classic => "light",
        }
    }
}

pub(crate) fn parse_hex_color(text: &str) -> Result<kit::Color32, String> {
    let value = text
        .strip_prefix('#')
        .ok_or_else(|| format!("Color {text} must start with #"))?;
    if value.len() != 6 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("color {text} must be #RRGGBB"));
    }
    let channel = |range: std::ops::Range<usize>| {
        u8::from_str_radix(&value[range], 16).map_err(|error| error.to_string())
    };
    Ok(kit::Color32::from_rgb(
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

pub(crate) const fn rgb32_u32(c: kit::Color32) -> u32 {
    ((c.r() as u32) << 16) | ((c.g() as u32) << 8) | c.b() as u32
}

const _: () = assert!(
    UP_RGB_DEFAULT.0 == 0xFF
        && UP_RGB_DEFAULT.1 == 0x00
        && UP_RGB_DEFAULT.2 == 0x00
        && DOWN_RGB_DEFAULT.0 == 0x00
        && DOWN_RGB_DEFAULT.1 == 0x99
        && DOWN_RGB_DEFAULT.2 == 0x00
);
