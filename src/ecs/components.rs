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

pub(crate) struct Pane {
    pub(crate) indicator: Box<dyn indicator::Indicator>,
    pub(crate) cmds: Vec<indicator::DrawCmd>,
    pub(crate) height: i32,
    pub(crate) dirty: bool,
}

impl Pane {
    pub(crate) fn new(indicator: Box<dyn indicator::Indicator>) -> Self {
        Self {
            indicator,
            cmds: Vec::new(),
            height: 0,
            dirty: true,
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
