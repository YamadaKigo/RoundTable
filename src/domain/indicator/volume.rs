#[cfg(test)]
use super::{
    ChartCtx, DrawCmd, axis_ticks, chart_cross, chart_cross_bg, chart_text, chart_tick, crosshair,
    format_value, volume_bars,
};
use super::{Indicator, Palette};
#[cfg(test)]
use crate::domain::data::random_walk::Bar;

#[cfg(test)]
const VOL_TICKS: usize = 3;

#[cfg(test)]
pub const YI: f64 = 1.0e8;
#[cfg(test)]
pub const WAN: f64 = 1.0e4;

#[cfg(test)]
pub fn fmt_volume(v: f64) -> String {
    format_value(v)
}

#[derive(Default)]
pub struct Volume {
    pal: Palette,
}

impl Indicator for Volume {
    fn name(&self) -> &str {
        "Volume"
    }

    #[cfg(test)]
    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        volume_bars(bars, self.pal.candle_up, self.pal.candle_down)
    }

    #[cfg(test)]
    fn axis(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        axis_ticks(ctx, VOL_TICKS, chart_text(), chart_tick(), format_value)
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
