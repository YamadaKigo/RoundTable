use crate::data::random_walk::Bar;

pub mod bars;
pub mod bias;
pub mod boll;
pub mod candle;
pub mod cci;
pub mod kdj;
pub mod macd;
pub mod obv;
pub mod rsi;
pub mod volume;

pub fn chart_text() -> u32 {
    crate::theme::rgb32_u32(crate::theme::theme().text)
}
pub fn chart_tick() -> u32 {
    crate::theme::rgb32_u32(crate::theme::theme().tick)
}
pub fn chart_cross() -> u32 {
    crate::theme::rgb32_u32(crate::theme::theme().cross)
}
pub fn chart_cross_bg() -> u32 {
    crate::theme::rgb32_u32(crate::theme::theme().cross_label)
}

pub fn chart_zero() -> u32 {
    crate::theme::rgb32_u32(crate::theme::theme().ind_zero)
}

pub type Rgb = (u8, u8, u8);

pub const fn rgb_u32(c: Rgb) -> u32 {
    ((c.0 as u32) << 16) | ((c.1 as u32) << 8) | c.2 as u32
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub candle_up: u32,
    pub candle_down: u32,
    pub text_up: u32,
    pub text_down: u32,
    pub dif: u32,
    pub dea: u32,

    pub kdj_k: u32,
    pub kdj_d: u32,
    pub kdj_j: u32,

    pub rsi_1: u32,
    pub rsi_2: u32,
    pub rsi_3: u32,

    pub boll_mid: u32,
    pub boll_up: u32,
    pub boll_down: u32,
    pub obv: u32,
    pub cci: u32,
    pub bias: [u32; 3],

    pub ma: [u32; 6],
    pub ema: [u32; 6],

    pub candle_suspended: u32,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            candle_up: 0xFF0000,
            candle_down: 0x009900,
            text_up: 0xFF0000,
            text_down: 0x009900,
            dif: 0x3333CC,
            dea: 0xCC9900,
            kdj_k: 0x3333CC,
            kdj_d: 0xCC9900,
            kdj_j: 0xFF00FF,
            rsi_1: 0x3333CC,
            rsi_2: 0xCC9900,
            rsi_3: 0xFF00FF,
            boll_mid: 0x3366CC,
            boll_up: 0xCC0066,
            boll_down: 0x009999,
            obv: 0xCC6600,
            cci: 0x6600CC,
            bias: [0x3333CC, 0xCC9900, 0xFF00FF],
            ma: [0x3333CC, 0xCC0066, 0x009933, 0x6600CC, 0x009999, 0x666666],
            ema: [0xFF00FF, 0xFF8800, 0x00CCCC, 0x88AA00, 0x8888FF, 0xAA00AA],
            candle_suspended: 0x999999,
        }
    }
}

pub const MA_PERIODS: [usize; 6] = [5, 10, 20, 60, 120, 250];
pub const EMA_PERIODS: [usize; 6] = MA_PERIODS;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Overlay {
    pub ma: u8,
    pub ema: u8,
    pub boll: bool,
}

#[allow(dead_code)]
impl Overlay {
    pub const NONE: Self = Self {
        ma: 0,
        ema: 0,
        boll: false,
    };

    pub const DEFAULT: Self = Self {
        ma: 0b0000_1111,
        ema: 0,
        boll: false,
    };

    pub fn ma_on(self, i: usize) -> bool {
        i < MA_PERIODS.len() && (self.ma >> i) & 1 == 1
    }
    pub fn ema_on(self, i: usize) -> bool {
        i < EMA_PERIODS.len() && (self.ema >> i) & 1 == 1
    }

    pub fn toggle_ma(&mut self, i: usize) {
        if i < MA_PERIODS.len() {
            self.ma ^= 1 << i;
        }
    }
    pub fn toggle_ema(&mut self, i: usize) {
        if i < EMA_PERIODS.len() {
            self.ema ^= 1 << i;
        }
    }

    pub fn toggle_boll(&mut self) {
        self.boll = !self.boll;
    }
}

#[derive(Clone)]
pub enum DrawCmd {
    Candle {
        i: usize,
        o: f64,
        h: f64,
        l: f64,
        c: f64,
        color: u32,
    },

    StyledCandle {
        i: usize,
        o: f64,
        h: f64,
        l: f64,
        c: f64,
        color: u32,
        w: u32,
    },

    Bar {
        i: usize,
        v0: f64,
        v1: f64,
        color: u32,
        hollow: bool,
    },

    StyledBar {
        i: usize,
        v0: f64,
        v1: f64,
        color: u32,
        hollow: bool,
        w: u32,
    },

    Ohlc {
        i: usize,
        o: f64,
        h: f64,
        l: f64,
        c: f64,
        color: u32,
    },

    StyledOhlc {
        i: usize,
        o: f64,
        h: f64,
        l: f64,
        c: f64,
        color: u32,
        w: u32,
    },

    CloseLine {
        i: usize,
        p0: f64,
        p1: f64,
        color: u32,
        w: u32,
    },

    Pagoda {
        i: usize,
        o: f64,
        h: f64,
        l: f64,
        c: f64,
        color: u32,
        w: u32,
    },

    Overlay {
        pts: Vec<(usize, f64)>,
        color: u32,
        w: u32,
    },

    Polyline {
        pts: Vec<(usize, f64)>,
        color: u32,
        w: u32,
    },

    #[allow(dead_code)]
    Line {
        i1: usize,
        p1: f64,
        i2: usize,
        p2: f64,
        color: u32,
        w: u32,
    },

    HLine {
        p: f64,
        color: u32,
        w: u32,
    },

    VLine {
        i: f64,
        color: u32,
        w: u32,
    },

    #[allow(dead_code)]
    Text {
        i: usize,
        p: f64,
        s: String,
        color: u32,
    },

    AxisText {
        p: f64,
        s: String,
        fg: u32,
        bg: u32,
    },

    TimeText {
        i: usize,
        s: String,
        fg: u32,
        bg: u32,
    },

    StatusText {
        s: String,
        color: u32,
    },

    StatusField {
        label: String,
        value: String,
        tmpl: String,
        color: u32,
    },
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ChartLayout {
    pub status_rows: i32,

    pub time_rows: i32,
}

impl ChartLayout {
    pub const NONE: Self = Self {
        status_rows: 0,
        time_rows: 0,
    };

    pub const fn new(status_rows: i32, time_rows: i32) -> Self {
        Self {
            status_rows,
            time_rows,
        }
    }

    pub const fn with_status(rows: i32) -> Self {
        Self {
            status_rows: rows,
            time_rows: 0,
        }
    }

    #[allow(dead_code)]
    pub const fn with_time(rows: i32) -> Self {
        Self {
            status_rows: 0,
            time_rows: rows,
        }
    }
}

pub struct ChartCtx<'a> {
    pub bars: &'a [Bar],

    pub cmds: &'a [DrawCmd],

    pub from: usize,
    pub to: usize,

    #[allow(dead_code)]
    pub visible: usize,

    #[allow(dead_code)]
    pub advance: f32,

    pub time_step: usize,

    pub lo: f64,
    pub hi: f64,

    pub cursor_bar: Option<usize>,

    pub cursor_index: Option<f64>,

    pub cursor_value: Option<f64>,
}

impl<'a> ChartCtx<'a> {
    pub fn bar(&self, i: usize) -> Option<&'a Bar> {
        self.bars.get(i)
    }

    pub fn ticks(&self, n: usize) -> Vec<f64> {
        if n < 2 || !self.hi.is_finite() || !self.lo.is_finite() || self.hi <= self.lo {
            return Vec::new();
        }
        (0..n)
            .map(|k| self.hi - (self.hi - self.lo) * k as f64 / (n - 1) as f64)
            .collect()
    }

    pub fn focus_bar(&self) -> Option<usize> {
        match self.cursor_bar {
            Some(i) => Some(i),
            None => self.bars.len().checked_sub(1),
        }
    }
}

pub fn candles(bars: &[Bar], up: u32, down: u32, suspended: u32) -> Vec<DrawCmd> {
    bars.iter()
        .enumerate()
        .map(|(i, bar)| DrawCmd::Candle {
            i,
            o: bar.o,
            h: bar.h,
            l: bar.l,
            c: bar.c,
            color: if bar.vol <= 0.0 {
                suspended
            } else if bar.c >= bar.o {
                up
            } else {
                down
            },
        })
        .collect()
}

pub fn volume_bars(bars: &[Bar], up: u32, down: u32) -> Vec<DrawCmd> {
    bars.iter()
        .enumerate()
        .map(|(i, bar)| {
            let rising = bar.c >= bar.o;
            DrawCmd::Bar {
                i,
                v0: 0.0,
                v1: bar.vol,
                color: if rising { up } else { down },
                hollow: rising,
            }
        })
        .collect()
}

pub fn ohlc_bars(bars: &[Bar], up: u32, down: u32) -> Vec<DrawCmd> {
    bars.iter()
        .enumerate()
        .map(|(i, bar)| DrawCmd::Ohlc {
            i,
            o: bar.o,
            h: bar.h,
            l: bar.l,
            c: bar.c,
            color: if bar.c >= bar.o { up } else { down },
        })
        .collect()
}

pub fn sma(values: &[f64], n: usize) -> Vec<Option<f64>> {
    let mut out = vec![None; values.len()];
    if n == 0 || n > values.len() {
        return out;
    }
    let mut sum: f64 = values[..n].iter().sum();
    out[n - 1] = Some(sum / n as f64);
    for i in n..values.len() {
        sum += values[i] - values[i - n];
        out[i] = Some(sum / n as f64);
    }
    out
}

pub fn ema(values: &[f64], n: usize) -> Vec<f64> {
    let k = 2.0 / (n as f64 + 1.0);
    let mut out = Vec::with_capacity(values.len());
    let mut prev = 0.0;
    for (i, v) in values.iter().enumerate() {
        prev = if i == 0 { *v } else { prev + k * (*v - prev) };
        out.push(prev);
    }
    out
}

fn pairs(values: &[Option<f64>]) -> Vec<(usize, f64)> {
    values
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|v| (i, v)))
        .collect()
}

pub fn overlay_lines(bars: &[Bar], ov: Overlay, pal: &Palette) -> Vec<DrawCmd> {
    let mut out = Vec::new();
    let closes: Vec<f64> = bars.iter().map(|bar| bar.c).collect();
    for (i, n) in MA_PERIODS.iter().enumerate() {
        if ov.ma_on(i) {
            let pts = pairs(&sma(&closes, *n));
            if pts.len() > 1 {
                out.push(DrawCmd::Overlay {
                    pts,
                    color: pal.ma[i],
                    w: 1,
                });
            }
        }
    }
    for (i, n) in EMA_PERIODS.iter().enumerate() {
        if ov.ema_on(i) {
            let values: Vec<Option<f64>> = ema(&closes, *n).into_iter().map(Some).collect();
            let pts = pairs(&values);
            if pts.len() > 1 {
                out.push(DrawCmd::Overlay {
                    pts,
                    color: pal.ema[i],
                    w: 1,
                });
            }
        }
    }
    if ov.boll {
        let (upper, mid, lower) = boll::bands(&closes, 20, 2.0);
        for (values, color) in [
            (&upper, pal.boll_up),
            (&mid, pal.boll_mid),
            (&lower, pal.boll_down),
        ] {
            let pts = pairs(values);
            if pts.len() > 1 {
                out.push(DrawCmd::Overlay { pts, color, w: 1 });
            }
        }
    }
    out
}

pub fn axis_ticks(
    ctx: &ChartCtx,
    n: usize,
    fg: u32,
    grid: u32,
    fmt: impl Fn(f64) -> String,
) -> Vec<DrawCmd> {
    let mut out = Vec::new();
    for p in ctx.ticks(n) {
        out.push(DrawCmd::HLine {
            p,
            color: grid,
            w: 1,
        });
        out.push(DrawCmd::AxisText {
            p,
            s: fmt(p),
            fg,
            bg: 0,
        });
    }
    out
}

pub fn time_labels(ctx: &ChartCtx, fg: u32) -> Vec<DrawCmd> {
    let step = ctx.time_step.max(1);
    let mut out = Vec::new();
    let mut i = ctx.from - ctx.from % step;
    let end = ctx.to.min(ctx.bars.len());
    while i < end {
        let bar = ctx.bars[i];
        let cross_year = i < step || ctx.bars[i - step].date.y != bar.date.y;
        let s = if cross_year {
            bar.date.ymd_short()
        } else {
            bar.date.md()
        };
        out.push(DrawCmd::TimeText { i, s, fg, bg: 0 });
        i += step;
    }
    out
}

pub fn crosshair(
    ctx: &ChartCtx,
    line: u32,
    fg: u32,
    bg: u32,
    fmt: impl Fn(f64) -> String,
) -> Vec<DrawCmd> {
    let Some(v) = ctx.cursor_value else {
        return Vec::new();
    };
    let Some(index) = ctx.cursor_bar else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if let Some(x) = ctx.cursor_index {
        out.push(DrawCmd::VLine {
            i: x,
            color: line,
            w: 1,
        });
    }
    out.push(DrawCmd::HLine {
        p: v,
        color: line,
        w: 1,
    });
    out.push(DrawCmd::AxisText {
        p: v,
        s: fmt(v),
        fg,
        bg,
    });
    out.push(DrawCmd::TimeText {
        i: index,
        s: index.to_string(),
        fg,
        bg,
    });
    out
}

pub fn status_field(label: &str, value: String, tmpl: &str, color: u32) -> DrawCmd {
    DrawCmd::StatusField {
        label: label.to_owned(),
        value,
        tmpl: tmpl.to_owned(),
        color,
    }
}

pub trait Indicator {
    fn name(&self) -> &str;

    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd>;

    fn layout(&self) -> ChartLayout {
        ChartLayout::NONE
    }

    fn axis(&self, _ctx: &ChartCtx) -> Vec<DrawCmd> {
        Vec::new()
    }

    fn time(&self, _ctx: &ChartCtx) -> Vec<DrawCmd> {
        Vec::new()
    }

    fn status(&self, _ctx: &ChartCtx) -> Vec<DrawCmd> {
        Vec::new()
    }

    fn overlay(&self, _ctx: &ChartCtx) -> Vec<DrawCmd> {
        Vec::new()
    }

    fn value_range(&self, lo: f64, hi: f64) -> (f64, f64) {
        (lo, hi)
    }

    fn set_palette(&mut self, _pal: Palette) {}

    #[allow(dead_code)]
    fn supports_overlay(&self) -> bool {
        false
    }

    #[allow(dead_code)]
    fn overlay_state(&self) -> Overlay {
        Overlay::NONE
    }

    #[allow(dead_code)]
    fn set_overlay_state(&mut self, _ov: Overlay) {}
}

pub fn builtin() -> Vec<Box<dyn Indicator>> {
    vec![
        Box::new(candle::Candle::default()),
        Box::new(bars::Bars::default()),
        Box::new(volume::Volume::default()),
        Box::new(macd::Macd::default()),
        Box::new(kdj::Kdj::default()),
        Box::new(rsi::Rsi::default()),
        Box::new(boll::Boll::default()),
        Box::new(obv::Obv::default()),
        Box::new(cci::Cci::default()),
        Box::new(bias::Bias::default()),
    ]
}

pub fn default_layout() -> Vec<Box<dyn Indicator>> {
    vec![
        Box::new(candle::Candle::default()),
        Box::new(volume::Volume::default()),
        Box::new(macd::Macd::default()),
    ]
}

pub fn builtin_names() -> Vec<String> {
    builtin().iter().map(|ind| ind.name().to_owned()).collect()
}

pub(crate) fn script_file(name: &str) -> &'static str {
    if name == "蜡烛图" {
        "candle.rti"
    } else if name == "美国线" {
        "bars.rti"
    } else if name == "成交量" {
        "volume.rti"
    } else if name.starts_with("MACD") {
        "macd.rti"
    } else if name.starts_with("KDJ") {
        "kdj.rti"
    } else if name.starts_with("RSI") {
        "rsi.rti"
    } else if name.starts_with("BOLL") {
        "boll.rti"
    } else if name.starts_with("OBV") {
        "obv.rti"
    } else if name.starts_with("CCI") {
        "cci.rti"
    } else {
        "bias.rti"
    }
}

pub(crate) fn script_source(name: &str) -> String {
    let file = script_file(name);
    std::fs::read_to_string(crate::assets::scripts_dir().join(file))
        .unwrap_or_else(|_| script_default(file).to_owned())
}

#[cfg(test)]
pub(crate) fn script_path(name: &str) -> std::path::PathBuf {
    crate::assets::scripts_dir().join(script_file(name))
}

pub(crate) fn user_script_path(pane: usize, name: &str) -> std::path::PathBuf {
    crate::assets::user_indicators_dir().join(format!("pane{pane}_{}", script_file(name)))
}

fn script_default(file: &str) -> &'static str {
    match file {
        "candle.rti" => include_str!("../../assets/scripts/candle.rti"),
        "bars.rti" => include_str!("../../assets/scripts/bars.rti"),
        "volume.rti" => include_str!("../../assets/scripts/volume.rti"),
        "macd.rti" => include_str!("../../assets/scripts/macd.rti"),
        "kdj.rti" => include_str!("../../assets/scripts/kdj.rti"),
        "rsi.rti" => include_str!("../../assets/scripts/rsi.rti"),
        "boll.rti" => include_str!("../../assets/scripts/boll.rti"),
        "obv.rti" => include_str!("../../assets/scripts/obv.rti"),
        "cci.rti" => include_str!("../../assets/scripts/cci.rti"),
        _ => include_str!("../../assets/scripts/bias.rti"),
    }
}
