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
    pub(crate) height: i32,
    pub(crate) dirty: bool,
    pub(crate) view: ChartView,
}

impl Pane {
    pub(crate) fn new(indicator: Box<dyn indicator::Indicator>) -> Self {
        Self {
            indicator,
            cmds: Vec::new(),
            height: 0,
            dirty: true,
            view: ChartView::new(),
        }
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
