use super::{
    ChartCtx, ChartLayout, DrawCmd, Indicator, Palette, axis_ticks, chart_cross, chart_cross_bg,
    chart_text, chart_tick, chart_zero, crosshair, format_value, status_field,
};
use crate::data::random_walk::Bar;

const STATUS_ROWS: i32 = 2;

const VALUE_TICKS: usize = 3;

const REF_LINES: [f64; 3] = [20.0, 50.0, 80.0];
const RANGE_LO: f64 = 0.0;
const RANGE_HI: f64 = 100.0;

pub struct Kdj {
    n: usize,
    m1: usize,
    m2: usize,
    title: String,
    pal: Palette,
}

impl Kdj {
    pub fn new(n: usize, m1: usize, m2: usize) -> Self {
        let n = n.max(1);
        let m1 = m1.max(1);
        let m2 = m2.max(1);
        Self {
            n,
            m1,
            m2,
            title: format!("KDJ({n},{m1},{m2})"),
            pal: Palette::default(),
        }
    }

    fn series(&self, bars: &[Bar]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let len = bars.len();
        let (mut k, mut d, mut j) = (vec![0.0; len], vec![0.0; len], vec![0.0; len]);

        let (mut k_prev, mut d_prev) = (50.0, 50.0);
        for i in 0..len {
            let from = (i + 1).saturating_sub(self.n);
            let (mut hi, mut lo) = (f64::MIN, f64::MAX);
            for bar in &bars[from..=i] {
                hi = hi.max(bar.h);
                lo = lo.min(bar.l);
            }
            let span = hi - lo;
            let rsv = if span.abs() > f64::EPSILON {
                (bars[i].c - lo) / span * 100.0
            } else {
                50.0
            };

            let k_now = (rsv + (self.m1.saturating_sub(1)) as f64 * k_prev) / self.m1 as f64;
            let d_now = (k_now + (self.m2.saturating_sub(1)) as f64 * d_prev) / self.m2 as f64;
            k[i] = k_now;
            d[i] = d_now;
            j[i] = 3.0 * k_now - 2.0 * d_now;
            k_prev = k_now;
            d_prev = d_now;
        }
        (k, d, j)
    }
}

impl Default for Kdj {
    fn default() -> Self {
        Self::new(9, 3, 3)
    }
}

fn fmt_value(v: Option<f64>) -> String {
    match v {
        Some(v) => format_value(v),
        None => "--".to_owned(),
    }
}

impl Indicator for Kdj {
    fn name(&self) -> &str {
        &self.title
    }

    fn value_range(&self, lo: f64, hi: f64) -> (f64, f64) {
        (lo.min(RANGE_LO), hi.max(RANGE_HI))
    }

    fn layout(&self) -> ChartLayout {
        ChartLayout::with_status(STATUS_ROWS)
    }

    fn axis(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        axis_ticks(ctx, VALUE_TICKS, chart_text(), chart_tick(), format_value)
    }

    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        if bars.is_empty() {
            return Vec::new();
        }
        let (k, d, j) = self.series(bars);
        let mut out: Vec<DrawCmd> = REF_LINES
            .iter()
            .map(|p| DrawCmd::HLine {
                p: *p,
                color: chart_zero(),
                w: 1,
            })
            .collect();
        for (values, color) in [
            (&k, self.pal.kdj_k),
            (&d, self.pal.kdj_d),
            (&j, self.pal.kdj_j),
        ] {
            let pts: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
            out.push(DrawCmd::Polyline { pts, color, w: 1 });
        }
        out
    }

    fn status(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        let Some(index) = ctx.focus_bar() else {
            return Vec::new();
        };
        let bars = ctx.bars;
        let (k, d, j) = self.series(bars);
        let value = |v: &[f64]| v.get(index).copied();
        vec![
            DrawCmd::StatusText {
                s: self.name().to_owned(),
                color: chart_text(),
            },
            status_field("K", fmt_value(value(&k)), self.pal.kdj_k),
            status_field("D", fmt_value(value(&d)), self.pal.kdj_d),
            status_field("J", fmt_value(value(&j)), self.pal.kdj_j),
        ]
    }

    fn overlay(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        crosshair(
            ctx,
            chart_cross(),
            chart_text(),
            chart_cross_bg(),
            format_value,
        )
    }

    fn set_palette(&mut self, pal: Palette) {
        self.pal = pal;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::random_walk::DATE_START;

    fn bars_from(prices: &[(f64, f64, f64, f64)]) -> Vec<Bar> {
        prices
            .iter()
            .enumerate()
            .map(|(i, (o, h, l, c))| Bar {
                o: *o,
                h: *h,
                l: *l,
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
    fn kdj_shape_is_stable() {
        let bars = bars_from(&[(10.0, 11.0, 9.0, 10.5); 40]);
        let kdj = Kdj::default();
        assert_eq!(kdj.name(), "KDJ(9,3,3)");
        assert_eq!(kdj.value_range(0.0, 0.0), (0.0, 100.0));
        let cmds = kdj.compute(&bars);
        let pal = Palette::default();
        for color in [pal.kdj_k, pal.kdj_d, pal.kdj_j] {
            assert_eq!(polyline(&cmds, color).len(), bars.len());
        }
        for p in REF_LINES {
            assert!(
                cmds.iter()
                    .any(|cmd| matches!(cmd, DrawCmd::HLine { p: q, .. } if *q == p)),
                "少了 {p} 那条参考线"
            );
        }
        assert!(Kdj::default().compute(&[]).is_empty());
    }

    #[test]
    fn kdj_is_high_in_a_steady_uptrend() {
        let bars = bars_from(
            &(0..60)
                .map(|i| {
                    let c = 10.0 + i as f64;
                    (c - 1.0, c, c - 2.0, c)
                })
                .collect::<Vec<_>>(),
        );
        let cmds = Kdj::default().compute(&bars);
        let pal = Palette::default();
        let k = polyline(&cmds, pal.kdj_k);
        let j = polyline(&cmds, pal.kdj_j);
        let last = k.len() - 1;
        assert!(
            k[last].1 > 70.0,
            "一路上涨时 K 该在高位，实际 {}",
            k[last].1
        );
        assert_eq!(j[last].0, k[last].0);

        assert!(j[last].1 > k[last].1);
    }

    #[test]
    fn value_range_keeps_j_above_and_below_the_reference_band() {
        let kdj = Kdj::default();
        assert_eq!(kdj.value_range(-20.0, 130.0), (-20.0, 130.0));
    }
}
