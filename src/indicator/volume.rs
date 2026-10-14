// ===========================================================================
//  指标：成交量
// ===========================================================================
//  一块最朴素的「图表」：只要绘图区（不要状态带 / 时间带）+ 右侧成交量刻度 + 十字光标。
//  一根 K 线一根量柱（0 -> 成交量），颜色按涨跌取界面当前的那一对。

use super::{
    axis_ticks, chart_cross, chart_cross_bg, chart_text, chart_tick, crosshair, volume_bars,
    ChartCtx, DrawCmd, Indicator, Palette,
};
use crate::data::random_walk::Bar;

/// 右侧成交量刻度几条。
const VOL_TICKS: usize = 3;

/// 刻度 / 状态条的单位换算：1e12 起步用「兆」、1e8 起步用「亿」、1e4 起步用「万」。
/// 兆 / 亿 保留整数（到了那个量级小数没意义），万 在整数部分 <= 3 位时保留 1 位小数、
/// 到了 4 位（>= 1000 万）也给整数，更小的给整数。
/// 于是最长的一档是 "999.9万"（5 个 ASCII + 1 个汉字 = 7 个内容格 = 63px），
/// 正好卡在格数上限 VALUE_COLS_MAX 上；纵轴留了 9 格 = 63px + 左右各 1 格，标签永远摆得下。
pub const ZHAO: f64 = 1.0e12;
pub const YI: f64 = 1.0e8;
pub const WAN: f64 = 1.0e4;

/// 成交量数值 -> 文字。
pub fn fmt_volume(v: f64) -> String {
    if v >= ZHAO {
        format!("{:.0}兆", v / ZHAO)
    } else if v >= YI {
        format!("{:.0}亿", v / YI)
    } else if v >= WAN {
        // 万：整数部分到 4 位就不留小数了，不然 "9999.9万" 会顶出纵轴那 7 格。
        let n = v / WAN;
        if n < 1000.0 {
            format!("{n:.1}万")
        } else {
            format!("{n:.0}万")
        }
    } else {
        format!("{v:.0}")
    }
}

/// 成交量柱。
pub struct Volume {
    /// 界面灌进来的整套颜色（量柱用其中的 K 线涨跌色）。
    pal: Palette,
}

impl Default for Volume {
    fn default() -> Self {
        Self {
            pal: Palette::default(),
        }
    }
}

impl Indicator for Volume {
    fn name(&self) -> &str {
        "成交量"
    }

    /// 自己的源码：路径相对本文件所在目录解析，所以直接写自己的文件名。
    fn source(&self) -> &'static str {
        include_str!("volume.rs")
    }

    /// 自己的源文件路径：file!() 宏给的就是「相对仓库根」的那个路径，不手写常量。
    fn path(&self) -> &'static str {
        file!()
    }

    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        volume_bars(bars, self.pal.candle_up, self.pal.candle_down)
    }

    /// 右侧成交量刻度：0 到最大量之间均匀 3 条，数值按万 / 亿换算。
    fn axis(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        axis_ticks(ctx, VOL_TICKS, chart_text(), chart_tick(), fmt_volume)
    }

    /// 最上层：十字光标（数值同样按万 / 亿显示）。
    fn overlay(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        crosshair(ctx, chart_cross(), chart_text(), chart_cross_bg(), fmt_volume)
    }

    fn set_palette(&mut self, pal: Palette) {
        self.pal = pal;
    }
}