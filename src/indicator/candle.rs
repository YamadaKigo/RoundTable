// ===========================================================================
//  指标：蜡烛图
// ===========================================================================
//  默认的一块「图表」：蜡烛 + 右侧价格刻度 + 底部时间轴 + 顶部状态条 + 十字光标。
//  这几样全是本文件自己拼出来的 - 界面只给几何（哪条带、多宽多高），别的什么都不管；
//  换个指标就是换一套画法，而同一份代码挂到任何一块图表上画出来都是一样的。
//
//  「K 线图」有两种画法：蜡烛图（本文件的 Candle）和美国线（bars.rs）。
//  两者除了行情那一层（蜡烛 / 美国线），其余（价格刻度 / 时间轴 / 状态条 / 十字光标 /
//  叠加均线）完全一样 - 所以那些都抽成本文件里的一组 pub(crate) 函数，两边共用。
//
//  叠加均线（MA / EMA）：只有 K 线图认（supports_overlay）。周期表和颜色表在
//  indicator/mod.rs 的 MA_PERIODS / Palette.ma 里；勾选状态由界面的「叠加」菜单灌进来。

use super::volume::fmt_volume;
use super::{
    axis_ticks, candles, chart_cross, chart_cross_bg, chart_text, chart_tick, crosshair,
    overlay_lines, status_field, time_labels, ChartCtx, ChartLayout, DrawCmd, Indicator, Overlay,
    Palette,
};
use crate::data::random_walk::Bar;

/// 状态带 / 时间带各 2 格（一个文本行盒：17px 内容 + 上下两条 1px 线格，和标题栏一个口径）。
/// 状态带是图表之间通用的那条带，文字必须和网格对齐，所以固定 2 格。
const STATUS_ROWS: i32 = 2;
const TIME_ROWS: i32 = 2;
/// 右侧价格刻度几条。
const PRICE_TICKS: usize = 5;

/// 状态带里的八项：标签 + 量宽度用的模板值（模板只为了算宽度，不参与显示）。
/// 模板比实际数值宽一点点没关系 - 数值是右对齐的，多出来的宽度全在标签那侧当空隙。
pub const INFO_LABELS: [&str; 8] = ["日期", "开", "高", "低", "收", "涨跌", "涨跌幅", "成交量"];
pub const INFO_TEMPLATES: [&str; 8] = [
    "0000-00-00",
    "0000.00",
    "0000.00",
    "0000.00",
    "0000.00",
    "-0000.00",
    "-0000.00%",
    "0000.0万",
];

/// K 线图要哪几条带：状态带 2 格 + 时间带 2 格。
pub(crate) fn kline_layout() -> ChartLayout {
    ChartLayout::new(STATUS_ROWS, TIME_ROWS)
}

/// K 线图的右侧价格刻度：值域里均匀 5 条参考线 + 数值。
pub(crate) fn kline_axis(ctx: &ChartCtx) -> Vec<DrawCmd> {
    axis_ticks(ctx, PRICE_TICKS, chart_text(), chart_tick(), |v| {
        format!("{v:.2}")
    })
}

/// K 线图的底部时间轴：日期。
pub(crate) fn kline_time(ctx: &ChartCtx) -> Vec<DrawCmd> {
    time_labels(ctx, chart_text())
}

/// K 线图的顶部状态条：鼠标指着的那一根（鼠标不在图上就是最后一根）的日期 / 四价 /
/// 涨跌 / 涨跌幅 / 成交量。涨跌和涨跌幅按涨跌色上色 - 涨用涨色、跌用跌色。
pub(crate) fn kline_status(ctx: &ChartCtx, pal: &Palette) -> Vec<DrawCmd> {
    let Some(index) = ctx.focus_bar() else {
        return Vec::new();
    };
    let Some(bar) = ctx.bar(index) else {
        return Vec::new();
    };
    // 涨跌 = 收盘 - 昨收（第一根没有昨收，就拿它自己的开盘当基准）。
    let prev_close = if index > 0 { ctx.bars[index - 1].c } else { bar.o };
    let change = bar.c - prev_close;
    let pct = if prev_close.abs() > f64::EPSILON {
        change / prev_close * 100.0
    } else {
        0.0
    };
    // 涨跌 / 涨跌幅用「数字涨跌色」（红涨绿跌，和 K 线跌色分开：暗色皮肤下 K 线跌色是青的）。
    let trend = if change >= 0.0 {
        pal.text_up
    } else {
        pal.text_down
    };
    let t = chart_text();
    let values = [
        (bar.date.full(), t),
        (format!("{:.2}", bar.o), t),
        (format!("{:.2}", bar.h), t),
        (format!("{:.2}", bar.l), t),
        (format!("{:.2}", bar.c), t),
        (format!("{change:+.2}"), trend),
        (format!("{pct:+.2}%"), trend),
        (fmt_volume(bar.vol), t),
    ];
    let mut out = Vec::new();
    for (i, (value, color)) in values.into_iter().enumerate() {
        out.push(status_field(INFO_LABELS[i], value, INFO_TEMPLATES[i], color));
    }
    out
}

/// K 线图的十字光标：竖线 + 横线 + 纵轴上的价格标签 + 底部的索引标签。
pub(crate) fn kline_crosshair(ctx: &ChartCtx) -> Vec<DrawCmd> {
    crosshair(ctx, chart_cross(), chart_text(), chart_cross_bg(), |v| {
        format!("{v:.2}")
    })
}

/// 蜡烛图。
pub struct Candle {
    /// 界面灌进来的整套颜色（K 线涨跌 + 数字涨跌 + 叠加均线）。
    pal: Palette,
    /// 叠加在 K 线上的均线（界面的「叠加」菜单改它）。
    ov: Overlay,
}

impl Default for Candle {
    fn default() -> Self {
        Self {
            pal: Palette::default(),
            ov: Overlay::DEFAULT,
        }
    }
}

impl Indicator for Candle {
    fn name(&self) -> &str {
        "蜡烛图"
    }

    /// 自己的源码：路径相对本文件所在目录解析，所以直接写自己的文件名。
    fn source(&self) -> &'static str {
        include_str!("candle.rs")
    }

    /// 自己的源文件路径：file!() 宏给的就是「相对仓库根」的那个路径，不手写常量。
    fn path(&self) -> &'static str {
        file!()
    }

    /// 蜡烛 + 叠加的那几条均线（均线是 Overlay，不参与纵轴值域扫描）。
    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        let mut out = candles(bars, self.pal.candle_up, self.pal.candle_down);
        let closes: Vec<f64> = bars.iter().map(|bar| bar.c).collect();
        out.extend(overlay_lines(&closes, self.ov, &self.pal));
        out
    }

    fn layout(&self) -> ChartLayout {
        kline_layout()
    }

    fn axis(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        kline_axis(ctx)
    }

    fn time(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        kline_time(ctx)
    }

    fn status(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        kline_status(ctx, &self.pal)
    }

    fn overlay(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        kline_crosshair(ctx)
    }

    fn set_palette(&mut self, pal: Palette) {
        self.pal = pal;
    }

    /// K 线图认「叠加均线」：标题栏上会出现「叠加」按钮。
    fn supports_overlay(&self) -> bool {
        true
    }

    fn overlay_state(&self) -> Overlay {
        self.ov
    }

    fn set_overlay_state(&mut self, ov: Overlay) {
        self.ov = ov;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::random_walk::DATE_START;
    use crate::indicator::MA_PERIODS;

    fn bars_from(closes: &[f64]) -> Vec<Bar> {
        closes
            .iter()
            .enumerate()
            .map(|(i, c)| Bar {
                o: *c,
                h: *c + 1.0,
                l: *c - 1.0,
                c: *c,
                vol: 1.0,
                date: DATE_START.add_days(i as i32),
            })
            .collect()
    }

    /// 默认叠四条 MA（5 / 10 / 20 / 60）：指令里就该正好有四条 Overlay 线，
    /// 颜色是 Palette.ma 里那四个；周期没凑够的开头那几根不画。
    #[test]
    fn candle_overlays_the_default_moving_averages() {
        let closes: Vec<f64> = (0..300).map(|i| 20.0 + (i as f64 * 0.1).sin()).collect();
        let bars = bars_from(&closes);
        let candle = Candle::default();
        let cmds = candle.compute(&bars);

        let overlays: Vec<(u32, usize, usize)> = cmds
            .iter()
            .filter_map(|cmd| match cmd {
                DrawCmd::Overlay { pts, color, .. } => {
                    Some((*color, pts.len(), pts[0].0))
                }
                _ => None,
            })
            .collect();
        assert_eq!(overlays.len(), 4, "默认该叠四条 MA");
        let pal = Palette::default();
        for (k, n) in MA_PERIODS.iter().take(4).enumerate() {
            assert_eq!(overlays[k].0, pal.ma[k], "第 {k} 条 MA 的颜色不对");
            assert_eq!(overlays[k].1, bars.len() - (n - 1), "第 {k} 条 MA 的点数不对");
            assert_eq!(overlays[k].2, n - 1, "第 {k} 条 MA 的起点不对");
        }
        // 蜡烛本身一根不少（Overlay 是叠加上去的，不占蜡烛）。
        let candles = cmds
            .iter()
            .filter(|cmd| matches!(cmd, DrawCmd::Candle { .. }))
            .count();
        assert_eq!(candles, bars.len());
    }

    /// 一条 MA 都不开时，compute 只剩蜡烛（界面把叠加全关掉之后就该是干净的 K 线）。
    #[test]
    fn candle_without_overlay_is_only_candles() {
        let bars = bars_from(&[10.0, 11.0, 12.0]);
        let mut candle = Candle::default();
        candle.set_overlay_state(Overlay::NONE);
        let cmds = candle.compute(&bars);
        assert_eq!(cmds.len(), bars.len());
        assert!(cmds.iter().all(|c| matches!(c, DrawCmd::Candle { .. })));
    }

    /// 均线不该把纵轴掰歪：值域扫描（pane_range 那一套）只看蜡烛，Overlay 一条都不算。
    #[test]
    fn overlay_lines_stay_out_of_the_value_range() {
        let closes: Vec<f64> = (0..300).map(|i| 100.0 - i as f64 * 0.2).collect();
        let bars = bars_from(&closes);
        let cmds = Candle::default().compute(&bars);
        let (lo, hi) = crate::ui::chart::pane_range(&cmds, 0, bars.len());
        let lo_want = bars.iter().map(|b| b.l).fold(f64::MAX, f64::min);
        let hi_want = bars.iter().map(|b| b.h).fold(f64::MIN, f64::max);
        assert!((lo - lo_want).abs() < 1e-9 && (hi - hi_want).abs() < 1e-9);
    }
}