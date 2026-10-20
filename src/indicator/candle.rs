use super::{
    ChartCtx, ChartLayout, DrawCmd, Indicator, Overlay, Palette, axis_ticks, candles, chart_cross,
    chart_cross_bg, chart_text, chart_tick, crosshair, format_value, overlay_lines, status_field,
    time_labels,
};
use crate::data::random_walk::{Bar, BarState, bar_state};

const STATUS_ROWS: i32 = 2;
const TIME_ROWS: i32 = 2;

const PRICE_TICKS: usize = 5;

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

pub(crate) fn kline_layout() -> ChartLayout {
    ChartLayout::new(STATUS_ROWS, TIME_ROWS)
}

pub(crate) fn kline_axis(ctx: &ChartCtx) -> Vec<DrawCmd> {
    axis_ticks(ctx, PRICE_TICKS, chart_text(), chart_tick(), format_value)
}

pub(crate) fn kline_time(ctx: &ChartCtx) -> Vec<DrawCmd> {
    time_labels(ctx, chart_text())
}

pub(crate) fn kline_status(ctx: &ChartCtx, pal: &Palette) -> Vec<DrawCmd> {
    let Some(index) = ctx.focus_bar() else {
        return Vec::new();
    };
    let Some(bar) = ctx.bar(index) else {
        return Vec::new();
    };

    let prev_close = if index > 0 {
        ctx.bars[index - 1].c
    } else {
        bar.o
    };
    let change = bar.c - prev_close;
    let pct = if prev_close.abs() > f64::EPSILON {
        change / prev_close * 100.0
    } else {
        0.0
    };

    let trend = if change >= 0.0 {
        pal.text_up
    } else {
        pal.text_down
    };
    let t = chart_text();
    let values = [
        (bar.date.full(), t),
        (format_value(bar.o), t),
        (format_value(bar.h), t),
        (format_value(bar.l), t),
        (format_value(bar.c), t),
        (format_value(change), trend),
        (format!("{}%", format_value(pct)), trend),
        (format_value(bar.vol), t),
    ];
    let mut out = Vec::new();
    for (i, (value, color)) in values.into_iter().enumerate() {
        out.push(status_field(INFO_LABELS[i], value, color));
    }
    let state = bar_state(ctx.bars, index);
    if state != BarState::Normal {
        out.push(DrawCmd::StatusText {
            s: state.label().to_owned(),
            color: match state {
                BarState::LimitUp => pal.text_up,
                BarState::LimitDown => pal.text_down,
                BarState::Suspended | BarState::Normal => chart_cross(),
            },
        });
    }
    out
}

pub(crate) fn kline_crosshair(ctx: &ChartCtx) -> Vec<DrawCmd> {
    crosshair(
        ctx,
        chart_cross(),
        chart_text(),
        chart_cross_bg(),
        format_value,
    )
}

pub struct Candle {
    pal: Palette,

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

    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        let mut out = candles(
            bars,
            self.pal.candle_up,
            self.pal.candle_down,
            self.pal.candle_suspended,
        );
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

    #[test]
    fn candle_overlays_the_default_moving_averages() {
        let closes: Vec<f64> = (0..300).map(|i| 20.0 + (i as f64 * 0.1).sin()).collect();
        let bars = bars_from(&closes);
        let candle = Candle::default();
        let cmds = candle.compute(&bars);

        let overlays: Vec<(u32, usize, usize)> = cmds
            .iter()
            .filter_map(|cmd| match cmd {
                DrawCmd::Overlay { pts, color, .. } => Some((*color, pts.len(), pts[0].0)),
                _ => None,
            })
            .collect();
        assert_eq!(overlays.len(), 4, "默认该叠四条 MA");
        let pal = Palette::default();
        for (k, n) in MA_PERIODS.iter().take(4).enumerate() {
            assert_eq!(overlays[k].0, pal.ma[k], "第 {k} 条 MA 的颜色不对");
            assert_eq!(
                overlays[k].1,
                bars.len() - (n - 1),
                "第 {k} 条 MA 的点数不对"
            );
            assert_eq!(overlays[k].2, n - 1, "第 {k} 条 MA 的起点不对");
        }

        let candles = cmds
            .iter()
            .filter(|cmd| matches!(cmd, DrawCmd::Candle { .. }))
            .count();
        assert_eq!(candles, bars.len());
    }

    #[test]
    fn candle_without_overlay_is_only_candles() {
        let bars = bars_from(&[10.0, 11.0, 12.0]);
        let mut candle = Candle::default();
        candle.set_overlay_state(Overlay::NONE);
        let cmds = candle.compute(&bars);
        assert_eq!(cmds.len(), bars.len());
        assert!(cmds.iter().all(|c| matches!(c, DrawCmd::Candle { .. })));
    }

    #[test]
    fn candle_can_overlay_the_bollinger_bands() {
        let closes: Vec<f64> = (0..80).map(|i| 20.0 + (i as f64 * 0.17).sin()).collect();
        let bars = bars_from(&closes);
        let mut candle = Candle::default();
        candle.set_overlay_state(Overlay {
            ma: 0,
            ema: 0,
            boll: true,
        });
        let cmds = candle.compute(&bars);
        let pal = Palette::default();
        for color in [pal.boll_up, pal.boll_mid, pal.boll_down] {
            assert!(
                cmds.iter().any(|cmd| matches!(
                    cmd,
                    DrawCmd::Overlay { color: c, .. } if *c == color
                )),
                "K 线上少了一条布林线"
            );
        }
        let upper = cmds.iter().find_map(|cmd| match cmd {
            DrawCmd::Overlay { pts, color, .. } if *color == pal.boll_up => Some(pts),
            _ => None,
        });
        assert_eq!(upper.expect("没有上轨").len(), closes.len() - 19);
        assert_eq!(upper.expect("没有上轨")[0].0, 19);
    }

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
