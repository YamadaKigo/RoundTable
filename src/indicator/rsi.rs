use super::{
    ChartCtx, ChartLayout, DrawCmd, Indicator, Palette, axis_ticks, chart_cross, chart_cross_bg,
    chart_text, chart_tick, chart_zero, crosshair, status_field,
};
use crate::data::random_walk::Bar;

pub const RSI_PERIODS: [usize; 3] = [6, 12, 24];

const STATUS_ROWS: i32 = 2;

const VALUE_TICKS: usize = 3;

const REF_LINES: [f64; 3] = [20.0, 50.0, 80.0];
const RANGE_LO: f64 = 0.0;
const RANGE_HI: f64 = 100.0;

pub struct Rsi {
    periods: [usize; 3],
    title: String,
    pal: Palette,
}

impl Rsi {
    pub fn new(periods: [usize; 3]) -> Self {
        Self {
            periods,
            title: format!("RSI({},{},{})", periods[0], periods[1], periods[2]),
            pal: Palette::default(),
        }
    }

    fn series(&self, bars: &[Bar], n: usize) -> Vec<Option<f64>> {
        let mut out = vec![None; bars.len()];
        if bars.len() < 2 || n == 0 {
            return out;
        }
        let (mut up, mut dn) = (0.0, 0.0);
        for i in 1..bars.len() {
            let diff = bars[i].c - bars[i - 1].c;
            let u = diff.max(0.0);
            let d = (-diff).max(0.0);

            up = (u + (n - 1) as f64 * up) / n as f64;
            dn = (d + (n - 1) as f64 * dn) / n as f64;
            let sum = up + dn;
            out[i] = Some(if sum.abs() > f64::EPSILON {
                up / sum * 100.0
            } else {
                0.0
            });
        }
        out
    }
}

impl Default for Rsi {
    fn default() -> Self {
        Self::new(RSI_PERIODS)
    }
}

fn fmt_value(v: Option<f64>) -> String {
    match v {
        Some(v) => format!("{v:.2}"),
        None => "--".to_owned(),
    }
}

impl Indicator for Rsi {
    fn name(&self) -> &str {
        &self.title
    }

    fn source(&self) -> &'static str {
        include_str!("rsi.rs")
    }

    fn path(&self) -> &'static str {
        file!()
    }

    fn value_range(&self, _lo: f64, _hi: f64) -> (f64, f64) {
        (RANGE_LO, RANGE_HI)
    }

    fn layout(&self) -> ChartLayout {
        ChartLayout::with_status(STATUS_ROWS)
    }

    fn axis(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        axis_ticks(ctx, VALUE_TICKS, chart_text(), chart_tick(), |v| {
            format!("{v:.0}")
        })
    }

    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        if bars.is_empty() {
            return Vec::new();
        }
        let mut out: Vec<DrawCmd> = REF_LINES
            .iter()
            .map(|p| DrawCmd::HLine {
                p: *p,
                color: chart_zero(),
                w: 1,
            })
            .collect();
        for (k, n) in self.periods.iter().enumerate() {
            let values = self.series(bars, *n);
            let pts: Vec<(usize, f64)> = values
                .iter()
                .enumerate()
                .filter_map(|(i, v)| v.map(|v| (i, v)))
                .collect();
            let color = [self.pal.rsi_1, self.pal.rsi_2, self.pal.rsi_3][k.min(2)];
            out.push(DrawCmd::Polyline { pts, color, w: 1 });
        }
        out
    }

    fn status(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        let Some(index) = ctx.focus_bar() else {
            return Vec::new();
        };
        let colors = [self.pal.rsi_1, self.pal.rsi_2, self.pal.rsi_3];
        let mut out = vec![DrawCmd::StatusText {
            s: self.name().to_owned(),
            color: chart_text(),
        }];
        for (k, n) in self.periods.iter().enumerate() {
            let v = self.series(ctx.bars, *n).get(index).copied().flatten();
            out.push(status_field(
                &format!("RSI{}", n),
                fmt_value(v),
                "000.00",
                colors[k.min(2)],
            ));
        }
        out
    }

    fn overlay(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        crosshair(ctx, chart_cross(), chart_text(), chart_cross_bg(), |v| {
            format!("{v:.2}")
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
                h: *c + 1.0,
                l: *c - 1.0,
                c: *c,
                vol: 1.0,
                date: DATE_START.add_days(i as i32),
            })
            .collect()
    }

    fn polyline(cmds: &[DrawCmd], color: u32) -> Vec<(usize, f64)> {
        cmds.iter()
            .find_map(|cmd| match cmd {
                DrawCmd::Polyline { pts, color: c, .. } if *c == color => Some(pts.clone()),
                _ => None,
            })
            .expect("没找到这条折线")
    }

    #[test]
    fn rsi_shape_is_stable() {
        let closes: Vec<f64> = (0..80).map(|i| 20.0 + (i as f64 * 0.3).sin()).collect();
        let bars = bars_from(&closes);
        let rsi = Rsi::default();
        assert_eq!(rsi.name(), "RSI(6,12,24)");
        assert_eq!(rsi.value_range(0.0, 0.0), (0.0, 100.0));
        let cmds = rsi.compute(&bars);
        let pal = Palette::default();
        for color in [pal.rsi_1, pal.rsi_2, pal.rsi_3] {
            let pts = polyline(&cmds, color);
            assert_eq!(pts.len(), bars.len() - 1);
            assert_eq!(pts[0].0, 1, "第一根没有昨收，RSI 该从第 2 根起");
        }
        for p in REF_LINES {
            assert!(
                cmds.iter()
                    .any(|cmd| matches!(cmd, DrawCmd::HLine { p: q, .. } if *q == p))
            );
        }
        assert!(Rsi::default().compute(&[]).is_empty());
    }

    #[test]
    fn rsi_is_one_hundred_in_a_steady_uptrend() {
        let closes: Vec<f64> = (0..40).map(|i| 10.0 + i as f64).collect();
        let cmds = Rsi::default().compute(&bars_from(&closes));
        let pts = polyline(&cmds, Palette::default().rsi_1);
        let last = pts[pts.len() - 1].1;
        assert!(
            (last - 100.0).abs() < 1e-9,
            "一路上涨 RSI 该是 100，实际 {last}"
        );
    }

    #[test]
    fn rsi_is_zero_in_a_steady_downtrend() {
        let closes: Vec<f64> = (0..40).map(|i| 100.0 - i as f64).collect();
        let cmds = Rsi::default().compute(&bars_from(&closes));
        let pts = polyline(&cmds, Palette::default().rsi_1);
        assert!(pts[pts.len() - 1].1.abs() < 1e-9);
    }
}
