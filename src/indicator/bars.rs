use super::candle::{kline_axis, kline_crosshair, kline_layout, kline_status, kline_time};
use super::{
    ChartCtx, ChartLayout, DrawCmd, Indicator, Overlay, Palette, ohlc_bars, overlay_lines,
};
use crate::data::random_walk::Bar;

pub struct Bars {
    pal: Palette,

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

    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        let mut out = ohlc_bars(bars, self.pal.candle_up, self.pal.candle_down);
        out.extend(overlay_lines(bars, self.ov, &self.pal));
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

    #[test]
    fn ohlc_bars_carry_the_four_prices() {
        let bars = bars_from(&[10.0, 11.0, 9.0]);
        let cmds = Bars::default().compute(&bars);
        let ohlc: Vec<(usize, f64, f64, f64, f64, u32)> = cmds
            .iter()
            .filter_map(|cmd| match cmd {
                DrawCmd::Ohlc {
                    i,
                    o,
                    h,
                    l,
                    c,
                    color,
                } => Some((*i, *o, *h, *l, *c, *color)),
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
            let want = if bar.c >= bar.o {
                pal.candle_up
            } else {
                pal.candle_down
            };
            assert_eq!(ohlc[k].5, want, "第 {k} 根的涨跌色不对");
        }
    }

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
