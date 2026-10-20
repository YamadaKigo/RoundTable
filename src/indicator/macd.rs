#[allow(unused_imports)]
use super::{
    ChartCtx, ChartLayout, DrawCmd, Indicator, Palette, Rgb, axis_ticks, chart_cross,
    chart_cross_bg, chart_text, chart_tick, chart_zero, crosshair, ema, rgb_u32,
};
use crate::data::random_walk::Bar;

#[allow(dead_code)]
pub const DIF_RGB: Rgb = (0x33, 0x33, 0xCC);
#[allow(dead_code)]
pub const DEA_RGB: Rgb = (0xCC, 0x99, 0x00);

const STATUS_ROWS: i32 = 2;

const VALUE_TICKS: usize = 3;

pub struct Macd {
    fast: usize,
    slow: usize,
    signal: usize,
    title: String,

    pal: Palette,
}

impl Macd {
    pub fn new(fast: usize, slow: usize, signal: usize) -> Self {
        let fast = fast.max(1);
        let slow = slow.max(1);
        let signal = signal.max(1);
        Self {
            fast,
            slow,
            signal,
            title: format!("MACD({fast},{slow},{signal})"),
            pal: Palette::default(),
        }
    }
}

impl Default for Macd {
    fn default() -> Self {
        Self::new(12, 26, 9)
    }
}

fn closes(bars: &[Bar]) -> Vec<f64> {
    bars.iter().map(|bar| bar.c).collect()
}

fn fmt_value(v: Option<f64>) -> String {
    match v {
        Some(v) => format!("{v:.2}"),
        None => "--".to_owned(),
    }
}

impl Indicator for Macd {
    fn name(&self) -> &str {
        &self.title
    }

    fn value_range(&self, lo: f64, hi: f64) -> (f64, f64) {
        let m = lo.abs().max(hi.abs()).max(f64::EPSILON);
        (-m, m)
    }

    fn layout(&self) -> ChartLayout {
        ChartLayout::with_status(STATUS_ROWS)
    }

    fn axis(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        axis_ticks(ctx, VALUE_TICKS, chart_text(), chart_tick(), |v| {
            format!("{v:.2}")
        })
    }

    fn overlay(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        crosshair(ctx, chart_cross(), chart_text(), chart_cross_bg(), |v| {
            format!("{v:.2}")
        })
    }

    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        let closes = closes(bars);
        let len = closes.len();
        if len == 0 {
            return Vec::new();
        }

        let fast = ema(&closes, self.fast);
        let slow = ema(&closes, self.slow);
        let dif: Vec<f64> = (0..len).map(|i| fast[i] - slow[i]).collect();

        let warm = self.slow.saturating_sub(1).min(len);
        let k = 2.0 / (self.signal as f64 + 1.0);
        let mut dea = vec![0.0; len];
        let mut prev = 0.0;
        for i in warm..len {
            prev = if i == warm {
                dif[i]
            } else {
                prev + k * (dif[i] - prev)
            };
            dea[i] = prev;
        }

        let mut cmds = Vec::with_capacity(2 * (len - warm) + 3);

        cmds.push(DrawCmd::HLine {
            p: 0.0,
            color: chart_zero(),
            w: 1,
        });

        for i in warm..len {
            let hist = (dif[i] - dea[i]) * 2.0;
            cmds.push(DrawCmd::Bar {
                i,
                v0: 0.0,
                v1: hist,
                hollow: false,
                color: if hist < 0.0 {
                    self.pal.candle_down
                } else {
                    self.pal.candle_up
                },
            });
        }

        cmds.push(DrawCmd::Polyline {
            pts: (warm..len).map(|i| (i, dif[i])).collect(),
            color: self.pal.dif,
            w: 1,
        });
        cmds.push(DrawCmd::Polyline {
            pts: (warm..len).map(|i| (i, dea[i])).collect(),
            color: self.pal.dea,
            w: 1,
        });
        cmds
    }

    fn status(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        let Some(index) = ctx.focus_bar() else {
            return Vec::new();
        };
        let mut dif = None;
        let mut dea = None;
        let mut macd = None;
        for cmd in ctx.cmds {
            match cmd {
                DrawCmd::Polyline { pts, color, .. } => {
                    let v = pts.iter().find(|(i, _)| *i == index).map(|(_, p)| *p);
                    if *color == self.pal.dif {
                        dif = v;
                    } else if *color == self.pal.dea {
                        dea = v;
                    }
                }
                DrawCmd::Bar { i, v1, .. } if *i == index => macd = Some(*v1),
                _ => {}
            }
        }
        let macd_color = if macd.is_some_and(|v| v < 0.0) {
            self.pal.candle_down
        } else {
            self.pal.candle_up
        };
        vec![
            DrawCmd::StatusText {
                s: self.name().to_owned(),
                color: chart_text(),
            },
            DrawCmd::StatusText {
                s: format!("DIF {}", fmt_value(dif)),
                color: self.pal.dif,
            },
            DrawCmd::StatusText {
                s: format!("DEA {}", fmt_value(dea)),
                color: self.pal.dea,
            },
            DrawCmd::StatusText {
                s: format!("MACD {}", fmt_value(macd)),
                color: macd_color,
            },
        ]
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

    fn bars_of(cmds: &[DrawCmd]) -> Vec<(usize, f64)> {
        cmds.iter()
            .filter_map(|cmd| match cmd {
                DrawCmd::Bar { i, v1, .. } => Some((*i, *v1)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn ema_starts_at_the_first_value() {
        let e = ema(&[10.0, 20.0, 20.0], 9);
        let k = 2.0 / 10.0;
        assert_eq!(e[0], 10.0);
        assert!((e[1] - (10.0 + k * 10.0)).abs() < 1e-12);
        assert!((e[2] - (e[1] + k * (20.0 - e[1]))).abs() < 1e-12);
        assert!(e[2] > e[1] && e[2] < 20.0);
    }

    #[test]
    fn macd_cmds_align_with_bars_and_warm_up() {
        let closes: Vec<f64> = (0..120).map(|i| 20.0 + (i as f64 * 0.1).sin()).collect();
        let bars = bars_from(&closes);
        let macd = Macd::default();
        let cmds = macd.compute(&bars);

        assert_eq!(macd.name(), "MACD(12,26,9)");
        assert_eq!(macd.value_range(1.0, 2.0), (-2.0, 2.0), "纵轴要以 0 对称");
        let dif = polyline(&cmds, rgb_u32(DIF_RGB));
        let dea = polyline(&cmds, rgb_u32(DEA_RGB));
        let hist = bars_of(&cmds);
        assert_eq!(dif.len(), bars.len() - 25);
        assert_eq!(dea.len(), bars.len() - 25);
        assert_eq!(hist.len(), bars.len() - 25);

        assert_eq!(dif[0].0, 25);
        assert_eq!(dea[0].0, 25);
        assert_eq!(hist[0].0, 25);
        assert_eq!(dif[dif.len() - 1].0, bars.len() - 1);

        assert!(cmds.iter().any(|cmd| matches!(
            cmd,
            DrawCmd::HLine { p, .. } if *p == 0.0
        )));
    }

    #[test]
    fn macd_hist_is_twice_the_gap() {
        let closes: Vec<f64> = (0..80).map(|i| 20.0 + (i as f64 * 0.37).cos()).collect();
        let bars = bars_from(&closes);
        let cmds = Macd::default().compute(&bars);
        let dif = polyline(&cmds, rgb_u32(DIF_RGB));
        let dea = polyline(&cmds, rgb_u32(DEA_RGB));
        for (k, (i, h)) in bars_of(&cmds).into_iter().enumerate() {
            let d = dif.iter().find(|(j, _)| *j == i).unwrap().1;
            let s = dea.iter().find(|(j, _)| *j == i).unwrap().1;
            assert!((h - (d - s) * 2.0).abs() < 1e-12, "第 {k} 根柱子不对");
        }
    }

    #[test]
    fn macd_is_positive_in_a_steady_uptrend() {
        let closes: Vec<f64> = (0..60).map(|i| 20.0 + i as f64).collect();
        let cmds = Macd::default().compute(&bars_from(&closes));
        let dif = polyline(&cmds, rgb_u32(DIF_RGB));
        assert!(dif[dif.len() - 1].1 > 0.0, "一路上涨时 DIF 应该为正");
        let hist = bars_of(&cmds);
        assert!(hist[hist.len() - 1].1 > 0.0);

        assert!(Macd::default().compute(&[]).is_empty());
    }
}
