use crate::data::random_walk::Bar;
use crate::indicator;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy)]
pub(crate) struct Crosshair {
    pub(crate) value: f64,
    pub(crate) index: usize,
    pub(crate) fpos: f64,
    pub(crate) pane: usize,
}

pub(crate) struct ChartView {
    pub(crate) scale: u32,
    pub(crate) offset: usize,
    pub(crate) visible: usize,
    pub(crate) pan: Option<(f32, usize)>,
    pub(crate) zoom_acc: f32,
    pub(crate) crosshair: Option<Crosshair>,
}

impl ChartView {
    pub(crate) fn new() -> Self {
        Self {
            scale: crate::grid::BAR_SCALE_DEFAULT,
            offset: 0,
            visible: 0,
            pan: None,
            zoom_acc: 0.0,
            crosshair: None,
        }
    }
}

pub(crate) struct Pane {
    pub(crate) indicator: Box<dyn indicator::Indicator>,
    pub(crate) cmds: Vec<indicator::DrawCmd>,
    pub(crate) script_render: Option<crate::script::ScriptRender>,
    pub(crate) height: i32,
    pub(crate) dirty: bool,
    pub(crate) view: ChartView,
    pub(crate) script: Option<crate::script::ScriptProgram>,
    pub(crate) script_source: String,
    pub(crate) script_error: Option<String>,
}

impl Pane {
    pub(crate) fn new(indicator: Box<dyn indicator::Indicator>) -> Self {
        let script_source = indicator::script_source(indicator.name());
        let script = crate::script::ScriptProgram::parse(&script_source).ok();
        Self {
            indicator,
            cmds: Vec::new(),
            script_render: None,
            height: 0,
            dirty: true,
            view: ChartView::new(),
            script,
            script_source,
            script_error: None,
        }
    }

    pub(crate) fn chart_layout(&self) -> indicator::ChartLayout {
        self.script_render.as_ref().map_or_else(
            || self.indicator.layout(),
            |render| render.layout(self.indicator.layout()),
        )
    }

    pub(crate) fn show_axis(&self) -> bool {
        self.script_render
            .as_ref()
            .and_then(|render| render.settings.show_axis)
            .unwrap_or(true)
    }

    pub(crate) fn chart_axis(&self, ctx: &indicator::ChartCtx) -> Vec<indicator::DrawCmd> {
        self.script_render
            .as_ref()
            .and_then(|render| render.axis(ctx))
            .unwrap_or_else(|| self.indicator.axis(ctx))
    }

    pub(crate) fn chart_time(&self, ctx: &indicator::ChartCtx) -> Vec<indicator::DrawCmd> {
        self.script_render
            .as_ref()
            .and_then(|render| render.time(ctx))
            .unwrap_or_else(|| self.indicator.time(ctx))
    }

    pub(crate) fn chart_status(&self, ctx: &indicator::ChartCtx) -> Vec<indicator::DrawCmd> {
        self.script_render
            .as_ref()
            .and_then(|render| render.status(ctx))
            .unwrap_or_else(|| self.indicator.status(ctx))
    }

    pub(crate) fn chart_value_range(&self, lo: f64, hi: f64) -> (f64, f64) {
        self.script_render.as_ref().map_or_else(
            || self.indicator.value_range(lo, hi),
            |render| render.value_range(lo, hi),
        )
    }
}

pub(crate) struct MarketBars {
    bars: Vec<Bar>,
}

impl MarketBars {
    pub(crate) fn new(bars: Vec<Bar>) -> Self {
        Self { bars }
    }

    pub(crate) fn replace(&mut self, bars: Vec<Bar>) {
        self.bars = bars;
    }
}

impl Deref for MarketBars {
    type Target = [Bar];

    fn deref(&self) -> &Self::Target {
        &self.bars
    }
}

impl DerefMut for MarketBars {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bars
    }
}
