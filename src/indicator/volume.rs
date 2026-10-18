use super::{
    ChartCtx, DrawCmd, Indicator, Palette, axis_ticks, chart_cross, chart_cross_bg, chart_text,
    chart_tick, crosshair, volume_bars,
};
use crate::data::random_walk::Bar;

const VOL_TICKS: usize = 3;

pub const ZHAO: f64 = 1.0e12;
pub const YI: f64 = 1.0e8;
pub const WAN: f64 = 1.0e4;

pub fn fmt_volume(v: f64) -> String {
    let sign = if v < 0.0 { "-" } else { "" };
    let v = v.abs();
    if v >= ZHAO {
        format!("{sign}{:.0}兆", v / ZHAO)
    } else if v >= YI {
        format!("{sign}{:.0}亿", v / YI)
    } else if v >= WAN {
        let n = v / WAN;
        if n < 1000.0 {
            format!("{sign}{n:.1}万")
        } else {
            format!("{sign}{n:.0}万")
        }
    } else {
        format!("{sign}{v:.0}")
    }
}

#[derive(Default)]
pub struct Volume {
    pal: Palette,
}

impl Indicator for Volume {
    fn name(&self) -> &str {
        "成交量"
    }

    fn source(&self) -> &'static str {
        include_str!("volume.rs")
    }

    fn path(&self) -> &'static str {
        file!()
    }

    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        volume_bars(bars, self.pal.candle_up, self.pal.candle_down)
    }

    fn axis(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        axis_ticks(ctx, VOL_TICKS, chart_text(), chart_tick(), fmt_volume)
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
