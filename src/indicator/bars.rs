// ===========================================================================
//  指标：美国线（OHLC bar chart）
// ===========================================================================
//  和蜡烛图并列的另一种 K 线画法：一根 K 线一个「竖线 + 左右两横」-
//  中间那竖是最低价到最高价，左边那横向左伸到中点 = 开盘价，右边那横从中点伸到右边 = 收盘价
//  （左开右收，国外那种 bar chart）。涨跌用同一套涨跌色上色。
//
//  「K 线图」的其它部分（右侧价格刻度 / 底部时间轴 / 顶部状态条 / 十字光标 / 叠加均线）
//  和蜡烛图完全一样 - 那些都抽在 candle.rs 里，这里直接调，一个字都不重复。

use super::candle::{
    kline_axis, kline_crosshair, kline_layout, kline_status, kline_time,
};
use super::{
    ohlc_bars, overlay_lines, ChartCtx, ChartLayout, DrawCmd, Indicator, Overlay, Palette,
};
use crate::data::random_walk::Bar;

/// 美国线。
pub struct Bars {
    /// 界面灌进来的整套颜色（K 线涨跌 + 数字涨跌 + 叠加均线）。
    pal: Palette,
    /// 叠加在 K 线上的均线（界面的「叠加」菜单改它）。
    ov: Overlay,
}

impl Default for Bars {
    fn default() -> Self {
        Self {
            pal: Palette::default(),
            ov: Overlay::DEFAULT,
        }
    }
}

impl Indicator for Bars {
    fn name(&self) -> &str {
        "美国线"
    }

    /// 自己的源码：路径相对本文件所在目录解析，所以直接写自己的文件名。
    fn source(&self) -> &'static str {
        include_str!("bars.rs")
    }

    /// 自己的源文件路径：file!() 宏给的就是「相对仓库根」的那个路径，不手写常量。
    fn path(&self) -> &'static str {
        file!()
    }

    /// 美国线 + 叠加的那几条均线（均线是 Overlay，不参与纵轴值域扫描）。
    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        let mut out = ohlc_bars(bars, self.pal.candle_up, self.pal.candle_down);
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

    fn bars_from(closes: &[f64]) -> Vec<Bar> {
        closes
            .iter()
            .enumerate()
            .map(|(i, c)| Bar {
                o: *c - 0.5,
                h: *c + 1.0,
                l: *c - 1.0,
                c: *c,
                vol: 1.0,
                date: DATE_START.add_days(i as i32),
            })
            .collect()
    }

    /// 一根 K 线一条 Ohlc 指令，四价原样带出来；涨的那根用涨色、跌的那根用跌色。
    #[test]
    fn ohlc_bars_carry_the_four_prices() {
        let bars = bars_from(&[10.0, 11.0, 9.0]);
        let cmds = Bars::default().compute(&bars);
        let ohlc: Vec<(usize, f64, f64, f64, f64, u32)> = cmds
            .iter()
            .filter_map(|cmd| match cmd {
                DrawCmd::Ohlc { i, o, h, l, c, color } => Some((*i, *o, *h, *l, *c, *color)),
                _ => None,
            })
            .collect();
        assert_eq!(ohlc.len(), bars.len());
        let pal = Palette::default();
        for (k, bar) in bars.iter().enumerate() {
            assert_eq!(ohlc[k].0, k);
            assert_eq!(ohlc[k].1, bar.o);
            assert_eq!(ohlc[k].2, bar.h);
            assert_eq!(ohlc[k].3, bar.l);
            assert_eq!(ohlc[k].4, bar.c);
            let want = if bar.c >= bar.o { pal.candle_up } else { pal.candle_down };
            assert_eq!(ohlc[k].5, want, "第 {k} 根的涨跌色不对");
        }
    }

    /// 美国线也认叠加均线：默认四条 MA，指令里就该有四条 Overlay。
    #[test]
    fn american_bars_accept_the_same_overlay() {
        let closes: Vec<f64> = (0..300).map(|i| 20.0 + (i as f64 * 0.07).cos()).collect();
        let cmds = Bars::default().compute(&bars_from(&closes));
        let overlays = cmds
            .iter()
            .filter(|cmd| matches!(cmd, DrawCmd::Overlay { .. }))
            .count();
        assert_eq!(overlays, 4);
    }
}