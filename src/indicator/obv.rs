use super::volume::fmt_volume;
use super::{
    ChartCtx, ChartLayout, DrawCmd, Indicator, Palette, axis_ticks, chart_cross, chart_cross_bg,
    chart_text, chart_tick, chart_zero, crosshair, status_field,
};
use crate::data::random_walk::Bar;

const STATUS_ROWS: i32 = 2;
const VALUE_TICKS: usize = 3;

#[derive(Default)]
pub struct Obv {
    pal: Palette,
}

pub const OBV_MA_PERIOD: usize = 30;

fn series(bars: &[Bar]) -> Vec<f64> {
    let mut out = Vec::with_capacity(bars.len());
    let mut total = 0.0;
    for (i, bar) in bars.iter().enumerate() {
        if i > 0 {
            let prev = bars[i - 1].c;
            if bar.c > prev {
                total += bar.vol;
            } else if bar.c < prev {
                total -= bar.vol;
            }
        }
        out.push(total);
    }
    out
}

fn average(values: &[f64], n: usize) -> Vec<Option<f64>> {
    let mut out = vec![None; values.len()];
    if n == 0 || values.len() < n {
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

impl Indicator for Obv {
    fn name(&self) -> &str {
        "OBV"
    }

    fn layout(&self) -> ChartLayout {
        ChartLayout::with_status(STATUS_ROWS)
    }

    fn axis(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        axis_ticks(ctx, VALUE_TICKS, chart_text(), chart_tick(), fmt_volume)
    }

    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        if bars.is_empty() {
            return Vec::new();
        }
        let values = series(bars);
        let average = average(&values, OBV_MA_PERIOD);
        vec![
            DrawCmd::HLine {
                p: 0.0,
                color: chart_zero(),
                w: 1,
            },
            DrawCmd::Polyline {
                pts: values.into_iter().enumerate().collect(),
                color: self.pal.obv,
                w: 1,
            },
            DrawCmd::Polyline {
                pts: average
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, value)| value.map(|value| (i, value)))
                    .collect(),
                color: self.pal.ma[0],
                w: 1,
            },
        ]
    }

    fn status(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        let Some(index) = ctx.focus_bar() else {
            return Vec::new();
        };
        let value = series(ctx.bars).get(index).copied();
        let average = average(&series(ctx.bars), OBV_MA_PERIOD)
            .get(index)
            .copied()
            .flatten();
        vec![
            DrawCmd::StatusText {
                s: self.name().to_owned(),
                color: chart_text(),
            },
            status_field(
                "OBV",
                value.map_or_else(|| "--".to_owned(), fmt_volume),
                "0000.0万",
                self.pal.obv,
            ),
            status_field(
                "OBVMA",
                average.map_or_else(|| "--".to_owned(), fmt_volume),
                "0000.0",
                self.pal.ma[0],
            ),
        ]
    }

    fn overlay(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        crosshair(
            ctx,
            chart_cross(),
            chart_text(),
            chart_cross_bg(),
            fmt_volume,
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

    fn bars_from(prices: &[(f64, f64)]) -> Vec<Bar> {
        prices
            .iter()
            .enumerate()
            .map(|(i, (c, vol))| Bar {
                o: *c,
                h: *c,
                l: *c,
                c: *c,
                vol: *vol,
                date: DATE_START.add_days(i as i32),
            })
            .collect()
    }

    #[test]
    fn obv_adds_volume_on_up_days_and_subtracts_it_on_down_days() {
        let bars = bars_from(&[
            (10.0, 100.0),
            (11.0, 200.0),
            (10.5, 50.0),
            (11.0, 30.0),
            (11.0, 40.0),
        ]);
        assert_eq!(series(&bars), vec![0.0, 200.0, 150.0, 180.0, 180.0]);

        let cmds = Obv::default().compute(&bars);
        assert!(cmds.iter().any(|cmd| matches!(
            cmd,
            DrawCmd::Polyline { pts, .. } if pts.len() == bars.len()
        )));
    }

    #[test]
    fn volume_formatter_handles_signed_obv() {
        assert_eq!(fmt_volume(-2.0e8), "-2亿");
        assert_eq!(fmt_volume(2.0e8), "2亿");
    }
}
