#[cfg(test)]
use super::{
    ChartCtx, DrawCmd, axis_ticks, chart_cross, chart_cross_bg, chart_text, chart_tick, chart_zero,
    crosshair, format_value, status_field,
};
use super::{ChartLayout, Indicator, Palette};
#[cfg(test)]
use crate::domain::data::random_walk::Bar;

const STATUS_ROWS: i32 = 2;
#[cfg(test)]
const VALUE_TICKS: usize = 3;
#[cfg(test)]
const REF_LINES: [f64; 3] = [-100.0, 0.0, 100.0];

pub struct Cci {
    #[cfg(test)]
    period: usize,
    title: String,
    pal: Palette,
}

impl Cci {
    pub fn new(period: usize) -> Self {
        Self {
            #[cfg(test)]
            period,
            title: format!("CCI({period})"),
            pal: Palette::default(),
        }
    }

    #[cfg(test)]
    fn series(&self, bars: &[Bar]) -> Vec<Option<f64>> {
        let mut out = vec![None; bars.len()];
        if self.period == 0 || bars.len() < self.period {
            return out;
        }
        let tp: Vec<f64> = bars
            .iter()
            .map(|bar| (bar.h + bar.l + bar.c) / 3.0)
            .collect();
        for i in (self.period - 1)..bars.len() {
            let from = i + 1 - self.period;
            let window = &tp[from..=i];
            let mean = window.iter().sum::<f64>() / self.period as f64;
            let deviation =
                window.iter().map(|v| (v - mean).abs()).sum::<f64>() / self.period as f64;
            out[i] = Some(if deviation > f64::EPSILON {
                (tp[i] - mean) / (0.015 * deviation)
            } else {
                0.0
            });
        }
        out
    }
}

impl Default for Cci {
    fn default() -> Self {
        Self::new(14)
    }
}

#[cfg(test)]
fn fmt_value(v: Option<f64>) -> String {
    match v {
        Some(v) => format_value(v),
        None => "--".to_owned(),
    }
}

impl Indicator for Cci {
    fn name(&self) -> &str {
        &self.title
    }

    #[cfg(test)]
    fn value_range(&self, lo: f64, hi: f64) -> (f64, f64) {
        (lo.min(-100.0), hi.max(100.0))
    }

    fn layout(&self) -> ChartLayout {
        ChartLayout::with_status(STATUS_ROWS)
    }

    #[cfg(test)]
    fn axis(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        axis_ticks(ctx, VALUE_TICKS, chart_text(), chart_tick(), format_value)
    }

    #[cfg(test)]
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
        let pts = self
            .series(bars)
            .into_iter()
            .enumerate()
            .filter_map(|(i, v)| v.map(|v| (i, v)))
            .collect();
        out.push(DrawCmd::Polyline {
            pts,
            color: self.pal.cci,
            w: 1,
        });
        out
    }

    #[cfg(test)]
    fn status(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        let Some(index) = ctx.focus_bar() else {
            return Vec::new();
        };
        let value = self.series(ctx.bars).get(index).copied().flatten();
        vec![
            DrawCmd::StatusText {
                s: self.name().to_owned(),
                color: chart_text(),
            },
            status_field("CCI", fmt_value(value), self.pal.cci),
        ]
    }

    #[cfg(test)]
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
    use crate::domain::data::random_walk::DATE_START;

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

    #[test]
    fn cci_keeps_ref_lines_and_warm_up() {
        let bars = bars_from(&[(10.0, 11.0, 9.0, 10.0); 20]);
        let cci = Cci::new(14);
        let cmds = cci.compute(&bars);
        for p in REF_LINES {
            assert!(cmds.iter().any(|cmd| matches!(
                cmd,
                DrawCmd::HLine { p: q, .. } if *q == p
            )));
        }
        let pts = cmds
            .iter()
            .find_map(|cmd| match cmd {
                DrawCmd::Polyline { pts, .. } => Some(pts),
                _ => None,
            })
            .expect("CCI 折线不见了");
        assert_eq!(pts.len(), 7);
        assert_eq!(pts[0].0, 13);
        assert!(pts.iter().all(|(_, v)| v.abs() < 1e-9));
        assert_eq!(cci.value_range(-25.0, 25.0), (-100.0, 100.0));
    }
}
