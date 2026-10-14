// ===========================================================================
//  指标层：图表
// ===========================================================================
//  一个「图表」= 一个指标：喂给它整段 K 线，它还给一串 DrawCmd（数据坐标的绘图指令）。
//  界面那边不认识任何具体指标 - 加新指标只要在这里再写一个 impl，绘图一行都不用改；
//  用户自己写的可编程图表走的是同一个接口（source() 就是它的源码，「代码」按钮拿它开窗）。
//
//  【没有主图 / 副图】：每一块都是「图表」，同一个指标挂到哪一块上画出来都一样 -
//  界面只提供几何（标题栏 / 状态带 / 绘图区 / 时间带 / 右侧纵轴）和一组默认绘制函数，
//  画什么、怎么画、要不要十字光标 / 刻度 / 状态条，全在指标这边（也就是用户代码）。
//
//  指标一共挂六个钩子，产出的都是数据坐标的 DrawCmd：
//    compute - 行情数据本身（蜡烛 / 柱子 / 折线）：只跟 bars 有关，整段算一次缓存起来；
//    layout  - 声明这块图表要哪几条带（状态带 / 时间带；绘图区永远有）；
//    axis    - 右侧纵轴带：刻度参考线 + 数值（画在数据下面）；
//    time    - 时间带：日期；
//    status  - 顶部状态带：鼠标那一根的数据 / 图例；
//    overlay - 最上层：十字光标之类（画在数据之上）。
//  后五个每帧现算（只产出装饰、不碰行情数据，开销可以忽略）；缩放 / 平移 / 鼠标移动
//  只重放缓存，绝不重算。
//
//  为什么 DrawCmd 只存数据坐标（K 线索引 + 价格 / 数值）、不存屏幕坐标：
//    - compute() 只跟 bars 有关，所以能整段算一次缓存起来（数据换代 / 换指标才算）；
//    - 缩放 / 平移 / 鼠标移动 / 每帧重绘只重放缓存 - 屏幕坐标在重放时才现算；
//    - 以后把同一份输入输出搬进脚本，或者把 DrawCmd 存下来做回放 / 回测，都跟屏幕无关。
//
//  颜色一律 0xRRGGBB（u32）。图表部件的默认配色也在这儿，theme.rs 末尾逐个核对，
//  保证和界面那套灰度是同一批值。

use crate::data::random_walk::Bar;

pub mod bars;
pub mod candle;
pub mod kdj;
pub mod macd;
pub mod rsi;
pub mod volume;

/// 图表区部件的颜色（0xRRGGBB）：一律从当前皮肤取 - 界面切皮肤，图表装饰
/// （刻度参考线 / 时间轴 / 状态带 / 十字光标）下一帧就跟着变。
/// 数值型数据（蜡烛 / 柱子 / 折线）的颜色由界面通过 set_palette 灌进来（见 Palette）。
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
/// 指标 0 轴那条参考线的颜色（也走皮肤）。
pub fn chart_zero() -> u32 {
    crate::theme::rgb32_u32(crate::theme::theme().ind_zero)
}

/// 颜色一律 (u8, u8, u8)，和主程序那一套一致；进 DrawCmd 时转成 0xRRGGBB。
pub type Rgb = (u8, u8, u8);

/// (u8, u8, u8) -> 0xRRGGBB。
pub const fn rgb_u32(c: Rgb) -> u32 {
    ((c.0 as u32) << 16) | ((c.1 as u32) << 8) | c.2 as u32
}

/// 一整套颜色（都出自当前皮肤），界面每帧灌给每个指标：K 线 / 柱子涨跌色、
/// 数字（状态带里的涨跌）涨跌色、MACD 两条线的颜色。
/// 「K 线跌色变青、数字跌色还是绿」就靠这里两套分开。
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub candle_up: u32,
    pub candle_down: u32,
    pub text_up: u32,
    pub text_down: u32,
    pub dif: u32,
    pub dea: u32,
    /// KDJ 的 K / D / J 三条线。
    pub kdj_k: u32,
    pub kdj_d: u32,
    pub kdj_j: u32,
    /// RSI 的三条线（周期见 RSI_PERIODS）。
    pub rsi_1: u32,
    pub rsi_2: u32,
    pub rsi_3: u32,
    /// K 线上的叠加均线：MA / EMA 各 6 条，和 MA_PERIODS / EMA_PERIODS 一一对应。
    pub ma: [u32; 6],
    pub ema: [u32; 6],
}

impl Default for Palette {
    /// 经典皮肤那套（指标还没被灌色时的兜底）。
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
            ma: [0x3333CC, 0xCC0066, 0x009933, 0x6600CC, 0x009999, 0x666666],
            ema: [0xFF00FF, 0xFF8800, 0x00CCCC, 0x88AA00, 0x8888FF, 0xAA00AA],
        }
    }
}

/// K 线上那两条叠加均线的周期：MA / EMA 各 6 条，和 Palette 的 ma / ema 两张色表一一对应。
pub const MA_PERIODS: [usize; 6] = [5, 10, 20, 60, 120, 250];
pub const EMA_PERIODS: [usize; 6] = MA_PERIODS;

/// K 线上叠不叠均线：两个位掩码，第 i 位 = 第 i 条周期（MA_PERIODS[i] / EMA_PERIODS[i]）开不开。
/// 「叠加」这个菜单里勾的就是它 - 只有 K 线图（蜡烛图 / 美国线）认这份配置。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Overlay {
    pub ma: u8,
    pub ema: u8,
}

impl Overlay {
    /// 一条都不叠。
    pub const NONE: Self = Self { ma: 0, ema: 0 };

    /// 默认：主图常用的那四条 MA（5 / 10 / 20 / 60）开着，EMA 全关。
    pub const DEFAULT: Self = Self { ma: 0b0000_1111, ema: 0 };

    /// 第 i 条 MA / EMA 开着吗（i 越界算关）。
    pub fn ma_on(self, i: usize) -> bool {
        i < MA_PERIODS.len() && (self.ma >> i) & 1 == 1
    }
    pub fn ema_on(self, i: usize) -> bool {
        i < EMA_PERIODS.len() && (self.ema >> i) & 1 == 1
    }

    /// 第 i 条 MA / EMA 开关翻转（i 越界什么都不做）。
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
}

/// 绘图指令：只说「画什么」，坐标全是数据坐标（i = 全局 K 线索引，p = 价格 / 数值）。
/// 屏幕坐标在重放时才算（见 ui::chart 的重放），所以缩放平移不会让缓存失效。
pub enum DrawCmd {
    // ---- 行情数据：compute() 产出，整段缓存 ----
    /// 一根蜡烛：四价 + 颜色。涨跌由 c 与 o 的大小决定（上涨空心、下跌实心，重放时判）。
    Candle { i: usize, o: f64, h: f64, l: f64, c: f64, color: u32 },
    /// 一根柱子：从 v0 长到 v1（成交量柱是 0 -> 量，MACD 柱是 0 -> 值）。
    /// hollow = true 时画空心柱（四边各 1px 的框、中间掏成绘图区底色）。
    Bar { i: usize, v0: f64, v1: f64, color: u32, hollow: bool },
    /// 美国线的一根：中间一竖 = 最高到最低，左边那横（中点到左边）= 开盘，
    /// 右边那横（中点到右边）= 收盘 - 左开右收，国外那种 bar chart。
    Ohlc { i: usize, o: f64, h: f64, l: f64, c: f64, color: u32 },
    /// 叠加线（K 线上的 MA / EMA）：画法和 Polyline 一模一样，但**不参与纵轴值域扫描** -
    /// 均线只是叠在 K 线上看的，250 日均线常常跑在可见区间之外，不能让它把纵轴掰歪。
    Overlay { pts: Vec<(usize, f64)>, color: u32, w: u32 },
    /// 折线：pts 是 (K 线下标, 值)；中间断档就拆成多段，每段一条 Polyline。
    Polyline { pts: Vec<(usize, f64)>, color: u32, w: u32 },
    /// 两点直线：用户在图表里画趋势线之类用（内置指标暂时不用）。
    #[allow(dead_code)]
    Line { i1: usize, p1: f64, i2: usize, p2: f64, color: u32, w: u32 },
    /// 一条横贯绘图区的水平线：p 是它的值（MACD 的 0 轴、用户的水平位）。
    HLine { p: f64, color: u32, w: u32 },

    // ---- 图表部件：四个钩子产出，每帧现算 ----
    /// 一条竖线：i 是（可以带小数的）K 线下标 - 十字光标的竖线就落在鼠标那一列上。
    VLine { i: f64, color: u32, w: u32 },
    /// 一个锚在 (i, p) 那个数据位置上的文字标签（居中）。内置指标暂时不用。
    #[allow(dead_code)]
    Text { i: usize, p: f64, s: String, color: u32 },
    /// 右侧纵轴带里的一行：值 p 对应的那一行，数值右对齐。
    /// bg 非 0 时底下衬一块带 1px 边框的标签（十字光标的价格标签就是它）。
    AxisText { p: f64, s: String, fg: u32, bg: u32 },
    /// 时间带里的一格：下标 i 那一列，日期居中。bg 非 0 时同样衬一块标签框
    /// （十字光标的索引标签就是它，画在绘图区下沿）。
    TimeText { i: usize, s: String, fg: u32, bg: u32 },
    /// 状态带里的一项：从左往右接着排，项间留 1 格（宽度按文字算）。
    StatusText { s: String, color: u32 },
    /// 状态带里的一项：标签左对齐、数值右对齐，宽度按 tmpl 量 -
    /// 数值位数变化时项宽不变，后面的项不会跟着抖。
    StatusField { label: String, value: String, tmpl: String, color: u32 },
}

/// 这块图表要哪几条带（绘图区永远有，不在这儿声明）。格数是内容格：
/// 2 格 = 一个文本行盒（17px 内容 + 上下两条 1px 线格，和标题栏一个口径）。
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ChartLayout {
    /// 顶部状态带（鼠标那一根的数据 / 图例）。0 = 不要。
    pub status_rows: i32,
    /// 底部时间带（日期）。0 = 不要。
    pub time_rows: i32,
}

impl ChartLayout {
    /// 只有绘图区：一个纯粹的图，什么带都不要。
    pub const NONE: Self = Self { status_rows: 0, time_rows: 0 };

    /// 状态带 + 时间带，各若干格。
    pub const fn new(status_rows: i32, time_rows: i32) -> Self {
        Self { status_rows, time_rows }
    }

    /// 只要状态带。
    pub const fn with_status(rows: i32) -> Self {
        Self { status_rows: rows, time_rows: 0 }
    }

    /// 只要时间带。
    #[allow(dead_code)]
    pub const fn with_time(rows: i32) -> Self {
        Self { status_rows: 0, time_rows: rows }
    }
}

/// 每帧现算的视图上下文：给指标代码看的那一份「现在什么情况」。
/// 只带数据和视图状态，一个像素都不带 - 像素换算全在重放那一侧。
pub struct ChartCtx<'a> {
    /// 整段行情（不用管可见不可见，按全局下标取）。
    pub bars: &'a [Bar],
    /// 这块图表自己那串缓存好的指令（状态条 / 图例可以直接从里面读值，不必重算）。
    pub cmds: &'a [DrawCmd],
    /// 可见区间（全局索引，右开）：[from, to)
    pub from: usize,
    pub to: usize,
    /// 一屏几根。
    #[allow(dead_code)]
    pub visible: usize,
    /// 一根 K 线在屏幕上占的像素步进（自己算刻度间距之类用得上）。
    #[allow(dead_code)]
    pub advance: f32,
    /// 界面按「相邻时间标签中心至少隔 6 格」算好的标注间隔（根）。
    pub time_step: usize,
    /// 当前纵轴值域（已过指标自己的 value_range）。
    pub lo: f64,
    pub hi: f64,
    /// 十字光标竖线指着的那一根；鼠标不在图上就是 None。
    pub cursor_bar: Option<usize>,
    /// 十字光标竖线的精确位置（带小数的下标，跟着鼠标走，不吸附到 K 线中心）。
    pub cursor_index: Option<f64>,
    /// 鼠标压在这块图表里时，按它自己的值域反算出来的那个数值；别的图表里是 None。
    pub cursor_value: Option<f64>,
}

impl<'a> ChartCtx<'a> {
    /// 按全局下标取一根 K 线（越界给 None）。
    pub fn bar(&self, i: usize) -> Option<&'a Bar> {
        self.bars.get(i)
    }

    /// 值域里均匀取 n 条刻度值（从上到下）。n < 2 或者值域塌了就返回空。
    pub fn ticks(&self, n: usize) -> Vec<f64> {
        if n < 2 || !(self.hi > self.lo) {
            return Vec::new();
        }
        (0..n)
            .map(|k| self.hi - (self.hi - self.lo) * k as f64 / (n - 1) as f64)
            .collect()
    }

    /// 状态条 / 图例跟着走的那一根：鼠标指着的那根；鼠标不在图上就是最后一根（没数据给 None）。
    pub fn focus_bar(&self) -> Option<usize> {
        match self.cursor_bar {
            Some(i) => Some(i),
            None => self.bars.len().checked_sub(1),
        }
    }
}

// ===========================================================================
//  默认绘制函数：内置指标就是拿这些拼出来的，用户也可以直接调 / 改
// ===========================================================================

/// 默认行情图之一：蜡烛 - 一根 K 线一条 Candle 指令，涨跌上色。
pub fn candles(bars: &[Bar], up: u32, down: u32) -> Vec<DrawCmd> {
    bars.iter()
        .enumerate()
        .map(|(i, bar)| DrawCmd::Candle {
            i,
            o: bar.o,
            h: bar.h,
            l: bar.l,
            c: bar.c,
            color: if bar.c >= bar.o { up } else { down },
        })
        .collect()
}

/// 默认行情图之一：成交量柱 - 从 0 长到成交量，涨跌上色。
/// 上涨那一档是空心柱（四边 1px 框、中间掏空），下跌是实心柱 - 一眼就能看出涨还是跌。
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

/// 默认行情图之一：美国线 - 一根 K 线一条 Ohlc 指令（最高 / 最低 / 开盘 / 收盘），涨跌上色。
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

/// 简单移动平均：前 n-1 根还没凑够 n 个收价，没有值（None）。
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

/// 指数移动平均：第一根就取它自己的值，之后按 k = 2/(n+1) 递推（前面不留一段空白）。
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

/// 一串 (下标, 值) 里把没有值的点丢掉：均线开头那几根（还没凑够周期）不画，
/// 后面的点连着画，不会在图上留一段假线。
fn pairs(values: &[Option<f64>]) -> Vec<(usize, f64)> {
    values
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|v| (i, v)))
        .collect()
}

/// K 线上的叠加均线（MA / EMA）：按 ov 里开着的位各产出一条 Overlay 指令。
/// closes 是收盘价序列；周期见 MA_PERIODS / EMA_PERIODS，上色走 pal.ma / pal.ema。
pub fn overlay_lines(closes: &[f64], ov: Overlay, pal: &Palette) -> Vec<DrawCmd> {
    let mut out = Vec::new();
    for (i, n) in MA_PERIODS.iter().enumerate() {
        if ov.ma_on(i) {
            let pts = pairs(&sma(closes, *n));
            if pts.len() > 1 {
                out.push(DrawCmd::Overlay { pts, color: pal.ma[i], w: 1 });
            }
        }
    }
    for (i, n) in EMA_PERIODS.iter().enumerate() {
        if ov.ema_on(i) {
            let values: Vec<Option<f64>> = ema(closes, *n).into_iter().map(Some).collect();
            let pts = pairs(&values);
            if pts.len() > 1 {
                out.push(DrawCmd::Overlay { pts, color: pal.ema[i], w: 1 });
            }
        }
    }
    out
}

/// 默认纵轴刻度：值域里均匀取 n 条 - 每条一条贯穿绘图区的参考线 + 右侧一个数值。
pub fn axis_ticks(
    ctx: &ChartCtx,
    n: usize,
    fg: u32,
    grid: u32,
    fmt: impl Fn(f64) -> String,
) -> Vec<DrawCmd> {
    let mut out = Vec::new();
    for p in ctx.ticks(n) {
        out.push(DrawCmd::HLine { p, color: grid, w: 1 });
        out.push(DrawCmd::AxisText { p, s: fmt(p), fg, bg: 0 });
    }
    out
}

/// 默认时间刻度：每 time_step 根标一个日期；跨年那一根换成 YY-MM-DD。
/// 时间带里放不下的（会压到绘图区边缘的）由重放那边丢掉，不用自己算。
pub fn time_labels(ctx: &ChartCtx, fg: u32) -> Vec<DrawCmd> {
    let step = ctx.time_step.max(1);
    let mut out = Vec::new();
    let mut i = ctx.from - ctx.from % step;
    let end = ctx.to.min(ctx.bars.len());
    while i < end {
        let bar = ctx.bars[i];
        let cross_year = i < step || ctx.bars[i - step].date.y != bar.date.y;
        let s = if cross_year { bar.date.ymd_short() } else { bar.date.md() };
        out.push(DrawCmd::TimeText { i, s, fg, bg: 0 });
        i += step;
    }
    out
}

/// 默认十字光标：竖线跟着鼠标走 - 三块图表共用同一套 x 映射，连起来就是一条贯穿的竖线，
/// 所以每一块都画；横线、纵轴数值标签、底部索引标签只在鼠标压着的那一块画
/// （别的图表那根鼠标 y 根本不在它值域里，各画一条索引标签又会重复三份）。
pub fn crosshair(
    ctx: &ChartCtx,
    line: u32,
    fg: u32,
    bg: u32,
    fmt: impl Fn(f64) -> String,
) -> Vec<DrawCmd> {
    let Some(index) = ctx.cursor_bar else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if let Some(x) = ctx.cursor_index {
        out.push(DrawCmd::VLine { i: x, color: line, w: 1 });
    }
    if let Some(v) = ctx.cursor_value {
        out.push(DrawCmd::HLine { p: v, color: line, w: 1 });
        out.push(DrawCmd::AxisText { p: v, s: fmt(v), fg, bg });
        out.push(DrawCmd::TimeText { i: index, s: index.to_string(), fg, bg });
    }
    out
}

/// 默认状态条里的一格：标签左对齐、数值右对齐，宽度按 tmpl 量（数值位数变化时项宽不变）。
pub fn status_field(label: &str, value: String, tmpl: &str, color: u32) -> DrawCmd {
    DrawCmd::StatusField {
        label: label.to_owned(),
        value,
        tmpl: tmpl.to_owned(),
        color,
    }
}

/// 指标统一接口（见本模块开头那段说明）。
pub trait Indicator {
    /// 名字（带参数，比如 "MACD(12,26,9)"）：图表标题栏、指标列表都用它。
    fn name(&self) -> &str;
    /// 这个指标自己的源码：include_str! 编译期就把整份文件嵌进来了（「代码」按钮看的就是它）。
    fn source(&self) -> &'static str;
    /// 这个指标自己的源文件路径（相对仓库根，由 file!() 宏给出，不手写字符串常量）：
    /// 可编辑源码窗「保存」时写回的就是它。
    fn path(&self) -> &'static str;
    /// 行情数据本身：只依赖 bars，不依赖界面状态；结果全是数据坐标。
    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd>;

    /// 这块图表要哪几条带。默认只有绘图区。
    fn layout(&self) -> ChartLayout {
        ChartLayout::NONE
    }
    /// 右侧纵轴带（画在数据下面）：刻度参考线 + 数值。默认空。
    fn axis(&self, _ctx: &ChartCtx) -> Vec<DrawCmd> {
        Vec::new()
    }
    /// 时间带：日期。默认空。
    fn time(&self, _ctx: &ChartCtx) -> Vec<DrawCmd> {
        Vec::new()
    }
    /// 顶部状态带：鼠标那一根的数据 / 图例。默认空。
    fn status(&self, _ctx: &ChartCtx) -> Vec<DrawCmd> {
        Vec::new()
    }
    /// 最上层（画在数据之上）：十字光标之类。默认空。
    fn overlay(&self, _ctx: &ChartCtx) -> Vec<DrawCmd> {
        Vec::new()
    }
    /// 纵轴值域：默认照扫描出来的可见区间值域用；指标可以自己改（MACD 就把它掰成以 0 对称）。
    fn value_range(&self, lo: f64, hi: f64) -> (f64, f64) {
        (lo, hi)
    }
    /// 灌入界面当前那整套颜色（来自皮肤）。默认什么都不做（不按颜色上色的指标用不着）。
    fn set_palette(&mut self, _pal: Palette) {}

    /// 这块图表认不认「叠加均线」（K 线图才认：蜡烛图 / 美国线）。默认不认 - 不认的图表
    /// 标题栏上就不会出现「叠加」按钮。
    fn supports_overlay(&self) -> bool {
        false
    }
    /// 当前的叠加配置（界面画「叠加」菜单里那几行勾选用）。
    fn overlay_state(&self) -> Overlay {
        Overlay::NONE
    }
    /// 灌入叠加配置：认它的指标要置脏重算（均线是 compute 时产出的）。
    fn set_overlay_state(&mut self, _ov: Overlay) {}
}

/// 内置指标表（标题栏「切换」菜单就按这个顺序摆）：K 线图两种、成交量、MACD、KDJ、RSI。
/// 比左栏那三个槽位多 - 槽位默认摆哪三个见 default_layout()，其余随时能从菜单换上去。
pub fn builtin() -> Vec<Box<dyn Indicator>> {
    vec![
        Box::new(candle::Candle::default()),
        Box::new(bars::Bars::default()),
        Box::new(volume::Volume::default()),
        Box::new(macd::Macd::default()),
        Box::new(kdj::Kdj::default()),
        Box::new(rsi::Rsi::default()),
    ]
}

/// 左栏三块图表的默认指标（槽位 0 / 1 / 2 = 蜡烛图 / 成交量 / MACD，见 app.rs 的 PANE_*）。
pub fn default_layout() -> Vec<Box<dyn Indicator>> {
    vec![
        Box::new(candle::Candle::default()),
        Box::new(volume::Volume::default()),
        Box::new(macd::Macd::default()),
    ]
}

/// 内置指标的名字（顺序和 builtin() 一模一样）：指标下拉面板只为了显示名字，
/// 不必真去构造一遍指标 - 但名字还是从各指标自己那儿取，免得两处各写一份。
pub fn builtin_names() -> Vec<String> {
    builtin().iter().map(|ind| ind.name().to_owned()).collect()
}