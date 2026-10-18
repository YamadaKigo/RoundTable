use super::{
    ChartCtx, ChartLayout, DrawCmd, Indicator, Palette, axis_ticks, chart_cross, chart_cross_bg,
    chart_text, chart_tick, chart_zero, crosshair, sma, status_field,
};
use crate::data::random_walk::Bar;

pub const BIAS_PERIODS: [usize; 3] = [6, 12, 24];

const STATUS_ROWS: i32 = 2;
const VALUE_TICKS: usize = 3;

pub struct Bias {
    periods: [usize; 3],
    title: String,
    pal: Palette,
}

impl Bias {
    pub fn new(periods: [usize; 3]) -> Self {
        Self {
            periods,
            title: format!("BIAS({},{},{})", periods[0], periods[1], periods[2]),
            pal: Palette::default(),
        }
    }

    fn series(&self, bars: &[Bar], period: usize) -> Vec<Option<f64>> {
        let closes: Vec<f64> = bars.iter().map(|bar| bar.c).collect();
        let ma = sma(&closes, period);
        closes
            .iter()
            .zip(ma.iter())
            .map(|(close, mean)| {
                mean.filter(|m| m.abs() > f64::EPSILON)
                    .map(|m| (close - m) / m * 100.0)
            })
            .collect()
    }
}

impl Default for Bias {
    fn default() -> Self {
        Self::new(BIAS_PERIODS)
    }
}

fn fmt_value(v: Option<f64>) -> String {
    match v {
        Some(v) => format!("{v:+.2}"),
        None => "--".to_owned(),
    }
}

impl Indicator for Bias {
    fn name(&self) -> &str {
        &self.title
    }

    fn value_range(&self, lo: f64, hi: f64) -> (f64, f64) {
        let m = lo.abs().max(hi.abs()).max(1.0);
        (-m, m)
    }

    fn layout(&self) -> ChartLayout {
        ChartLayout::with_status(STATUS_ROWS)
    }

    fn axis(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        axis_ticks(ctx, VALUE_TICKS, chart_text(), chart_tick(), |v| {
            format!("{v:.1}%")
        })
    }

    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        if bars.is_empty() {
            return Vec::new();
        }
        let mut out = vec![DrawCmd::HLine {
            p: 0.0,
            color: chart_zero(),
            w: 1,
        }];
        for (k, period) in self.periods.iter().enumerate() {
            let pts = self
                .series(bars, *period)
                .into_iter()
                .enumerate()
                .filter_map(|(i, v)| v.map(|v| (i, v)))
                .collect();
            out.push(DrawCmd::Polyline {
                pts,
                color: self.pal.bias[k.min(2)],
                w: 1,
            });
        }
        out
    }

    fn status(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        let Some(index) = ctx.focus_bar() else {
            return Vec::new();
        };
        let mut out = vec![DrawCmd::StatusText {
            s: self.name().to_owned(),
            color: chart_text(),
        }];
        for (k, period) in self.periods.iter().enumerate() {
            let value = self.series(ctx.bars, *period).get(index).copied().flatten();
            out.push(status_field(
                &format!("BIAS{period}"),
                fmt_value(value),
                "+000.00%",
                self.pal.bias[k.min(2)],
            ));
        }
        out
    }

    fn overlay(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        crosshair(ctx, chart_cross(), chart_text(), chart_cross_bg(), |v| {
            format!("{v:+.2}%")
        })
    }

    fn set_palette(&mut self, pal: Palette) {
        self.pal = pal;
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
                o: *c,
                h: *c,
                l: *c,
                c: *c,
                vol: 1.0,
                date: DATE_START.add_days(i as i32),
            })
            .collect()
    }

    #[test]
    fn bias_is_close_minus_ma_over_ma() {
        let closes: Vec<f64> = (0..30).map(|i| 10.0 + i as f64).collect();
        let bars = bars_from(&closes);
        let bias = Bias::default();
        let s6 = bias.series(&bars, 6);
        assert!(s6[..5].iter().all(Option::is_none));
        let mean = closes[0..6].iter().sum::<f64>() / 6.0;
        let want = (closes[5] - mean) / mean * 100.0;
        assert!((s6[5].unwrap() - want).abs() < 1e-12);
        assert_eq!(bias.name(), "BIAS(6,12,24)");
    }

    #[test]
    fn bias_range_is_symmetric_around_zero() {
        let bias = Bias::default();
        assert_eq!(bias.value_range(-3.0, 8.0), (-8.0, 8.0));
        assert_eq!(bias.value_range(0.0, 0.0), (-1.0, 1.0));
    }
}
