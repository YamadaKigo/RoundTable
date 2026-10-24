#[cfg(test)]
use super::{
    ChartCtx, DrawCmd, axis_ticks, chart_cross, chart_cross_bg, chart_text, chart_tick, crosshair,
    format_value, sma, status_field,
};
use super::{ChartLayout, Indicator, Palette};
#[cfg(test)]
use crate::domain::data::random_walk::Bar;

const STATUS_ROWS: i32 = 2;
#[cfg(test)]
const VALUE_TICKS: usize = 3;
#[cfg(test)]
type BandSeries = (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>);

pub struct Boll {
    #[cfg(test)]
    period: usize,
    #[cfg(test)]
    mult: f64,
    title: String,
    pal: Palette,
}

impl Boll {
    pub fn new(period: usize, mult: f64) -> Self {
        let mult_text = if (mult - mult.round()).abs() < 1e-9 {
            format!("{mult:.0}")
        } else {
            format!("{mult}")
        };
        Self {
            #[cfg(test)]
            period,
            #[cfg(test)]
            mult,
            title: format!("BOLL({period},{mult_text})"),
            pal: Palette::default(),
        }
    }

    #[cfg(test)]
    fn band(&self, bars: &[Bar]) -> BandSeries {
        let closes: Vec<f64> = bars.iter().map(|bar| bar.c).collect();
        bands(&closes, self.period, self.mult)
    }
}

#[cfg(test)]
pub fn bands(closes: &[f64], period: usize, mult: f64) -> BandSeries {
    let mid = sma(closes, period);
    let mut upper = vec![None; closes.len()];
    let mut lower = vec![None; closes.len()];
    if period == 0 {
        return (mid, upper, lower);
    }
    for i in 0..closes.len() {
        let Some(m) = mid[i] else {
            continue;
        };
        let from = i + 1 - period;
        let variance = closes[from..=i]
            .iter()
            .map(|v| {
                let d = *v - m;
                d * d
            })
            .sum::<f64>()
            / period as f64;
        let width = variance.max(0.0).sqrt() * mult;
        upper[i] = Some(m + width);
        lower[i] = Some(m - width);
    }
    (mid, upper, lower)
}

impl Default for Boll {
    fn default() -> Self {
        Self::new(20, 2.0)
    }
}

#[cfg(test)]
fn fmt_value(v: Option<f64>) -> String {
    match v {
        Some(v) => format_value(v),
        None => "--".to_owned(),
    }
}

#[cfg(test)]
fn points(values: &[Option<f64>]) -> Vec<(usize, f64)> {
    values
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|v| (i, v)))
        .collect()
}

impl Indicator for Boll {
    fn name(&self) -> &str {
        &self.title
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
        let (mid, upper, lower) = self.band(bars);
        vec![
            DrawCmd::Polyline {
                pts: points(&upper),
                color: self.pal.boll_up,
                w: 1,
            },
            DrawCmd::Polyline {
                pts: points(&mid),
                color: self.pal.boll_mid,
                w: 1,
            },
            DrawCmd::Polyline {
                pts: points(&lower),
                color: self.pal.boll_down,
                w: 1,
            },
        ]
    }

    #[cfg(test)]
    fn status(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        let Some(index) = ctx.focus_bar() else {
            return Vec::new();
        };
        let (mid, upper, lower) = self.band(ctx.bars);
        let at = |v: &[Option<f64>]| v.get(index).copied().flatten();
        vec![
            DrawCmd::StatusText {
                s: self.name().to_owned(),
                color: chart_text(),
            },
            status_field("UP", fmt_value(at(&upper)), self.pal.boll_up),
            status_field("MID", fmt_value(at(&mid)), self.pal.boll_mid),
            status_field("DN", fmt_value(at(&lower)), self.pal.boll_down),
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
    fn boll_uses_close_sma_and_population_deviation() {
        let bars = bars_from(&[10.0, 11.0, 12.0, 13.0, 14.0]);
        let boll = Boll::new(5, 2.0);
        let (mid, upper, lower) = boll.band(&bars);
        assert!(mid[..4].iter().all(Option::is_none));
        assert_eq!(mid[4], Some(12.0));
        assert_eq!(upper[4], Some(12.0 + 2.0 * 2.0f64.sqrt()));
        assert_eq!(lower[4], Some(12.0 - 2.0 * 2.0f64.sqrt()));
        assert_eq!(boll.name(), "BOLL(5,2)");
    }

    #[test]
    fn boll_commands_match_the_three_lines() {
        let closes: Vec<f64> = (0..40).map(|i| 20.0 + (i as f64 * 0.2).sin()).collect();
        let cmds = Boll::default().compute(&bars_from(&closes));
        let pal = Palette::default();
        for color in [pal.boll_up, pal.boll_mid, pal.boll_down] {
            let pts = cmds
                .iter()
                .find_map(|cmd| match cmd {
                    DrawCmd::Polyline { pts, color: c, .. } if *c == color => Some(pts),
                    _ => None,
                })
                .expect("Bollinger Bands少了一条");
            assert_eq!(pts.len(), closes.len() - 19);
            assert_eq!(pts[0].0, 19);
        }
        assert!(Boll::default().compute(&[]).is_empty());
    }
}
