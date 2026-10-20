use super::{
    ChartCtx, DrawCmd, Indicator, Palette, axis_ticks, chart_cross, chart_cross_bg, chart_text,
    chart_tick, crosshair, format_value, volume_bars,
};
use crate::data::random_walk::Bar;

const VOL_TICKS: usize = 3;

pub const ZHAO: f64 = 1.0e12;
pub const YI: f64 = 1.0e8;
pub const WAN: f64 = 1.0e4;

pub fn fmt_volume(v: f64) -> String {
    format_value(v)
}

#[derive(Default)]
pub struct Volume {
    pal: Palette,
}

impl Indicator for Volume {
    fn name(&self) -> &str {
        "成交量"
    }

    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        volume_bars(bars, self.pal.candle_up, self.pal.candle_down)
    }

    fn axis(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        axis_ticks(ctx, VOL_TICKS, chart_text(), chart_tick(), format_value)
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
