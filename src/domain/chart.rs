use crate::domain::data::random_walk::Bar;
use crate::domain::indicator;
use crate::domain::layout::{BAR_COUNT, BAR_SCALE_DEFAULT, BAR_SEED, BAR_START};

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

impl Default for ChartView {
    fn default() -> Self {
        Self {
            scale: BAR_SCALE_DEFAULT,
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
    pub(crate) cmds: PaneCmds,
    pub(crate) script_render: Option<crate::domain::script::ScriptRender>,
    pub(crate) height: i32,
    pub(crate) dirty: bool,
    pub(crate) view: ChartView,
    pub(crate) script: Option<crate::domain::script::ScriptProgram>,
    pub(crate) script_source: String,
    pub(crate) script_error: Option<String>,
}

impl Pane {
    pub(crate) fn new(indicator: Box<dyn indicator::Indicator>) -> Self {
        let script_source = indicator::script_source(indicator.name());
        let script = crate::domain::script::ScriptProgram::parse(&script_source).ok();
        Self {
            indicator,
            cmds: PaneCmds::default(),
            script_render: None,
            height: 0,
            dirty: true,
            view: ChartView::default(),
            script,
            script_source,
            script_error: None,
        }
    }

    pub(crate) fn chart_layout(&self) -> indicator::ChartLayout {
        self.script_render
            .as_ref()
            .map_or(indicator::ChartLayout::NONE, |render| {
                render.layout(self.indicator.layout())
            })
    }

    pub(crate) fn show_axis(&self) -> bool {
        self.script_render
            .as_ref()
            .and_then(|render| render.settings.show_axis)
            .unwrap_or(false)
    }

    pub(crate) fn chart_axis(&self, ctx: &indicator::ChartCtx) -> Vec<indicator::DrawCmd> {
        self.script_render
            .as_ref()
            .and_then(|render| render.axis(ctx))
            .unwrap_or_default()
    }

    pub(crate) fn chart_time(&self, ctx: &indicator::ChartCtx) -> Vec<indicator::DrawCmd> {
        self.script_render
            .as_ref()
            .and_then(|render| render.time(ctx))
            .unwrap_or_default()
    }

    pub(crate) fn chart_status(&self, ctx: &indicator::ChartCtx) -> Vec<indicator::DrawCmd> {
        self.script_render
            .as_ref()
            .and_then(|render| render.status(ctx))
            .unwrap_or_default()
    }

    pub(crate) fn chart_crosshair(&self, ctx: &indicator::ChartCtx) -> Vec<indicator::DrawCmd> {
        self.script_render
            .as_ref()
            .map_or_else(Vec::new, |render| render.crosshair(ctx))
    }

    pub(crate) fn chart_value_range(&self, lo: f64, hi: f64) -> (f64, f64) {
        self.script_render
            .as_ref()
            .map_or((lo, hi), |render| render.value_range(lo, hi))
    }
}

#[derive(Clone, Default)]
pub(crate) struct PaneCmds {
    raw: Vec<indicator::DrawCmd>,
    by_bar: Vec<Vec<u32>>,
    global: Vec<u32>,
}

impl PaneCmds {
    pub(crate) fn set(&mut self, raw: Vec<indicator::DrawCmd>) {
        self.raw = raw;
        self.by_bar.clear();
        self.global.clear();
        let mut indices = Vec::new();
        for (position, cmd) in self.raw.iter().enumerate() {
            command_indices_into(cmd, &mut indices);
            if indices.is_empty() {
                self.global.push(position as u32);
                continue;
            }
            let max_index = indices.iter().copied().max().unwrap_or(0);
            if self.by_bar.len() <= max_index {
                self.by_bar.resize_with(max_index + 1, Vec::new);
            }
            for &index in &indices {
                self.by_bar[index].push(position as u32);
            }
        }
    }

    pub(crate) fn clear(&mut self) {
        self.raw.clear();
        self.by_bar.clear();
        self.global.clear();
    }

    #[cfg(test)]
    pub(crate) fn visible_indices(&self, from: usize, to: usize) -> Vec<usize> {
        let mut positions = Vec::new();
        self.visible_indices_into(from, to, &mut positions);
        positions.into_iter().map(|i| i as usize).collect()
    }

    pub(crate) fn visible_indices_into(&self, from: usize, to: usize, positions: &mut Vec<u32>) {
        positions.clear();
        positions.extend_from_slice(&self.global);
        let start = from.min(self.by_bar.len());
        let end = to.min(self.by_bar.len());
        if start < end {
            for bucket in &self.by_bar[start..end] {
                positions.extend_from_slice(bucket);
            }
        }
        positions.sort_unstable();
        positions.dedup();
    }
}

impl std::ops::Deref for PaneCmds {
    type Target = [indicator::DrawCmd];

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl<'a> IntoIterator for &'a PaneCmds {
    type Item = &'a indicator::DrawCmd;
    type IntoIter = std::slice::Iter<'a, indicator::DrawCmd>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter()
    }
}

fn command_indices_into(cmd: &indicator::DrawCmd, indices: &mut Vec<usize>) {
    use indicator::DrawCmd;
    indices.clear();
    match cmd {
        DrawCmd::Candle { i, .. }
        | DrawCmd::StyledCandle { i, .. }
        | DrawCmd::Bar { i, .. }
        | DrawCmd::StyledBar { i, .. }
        | DrawCmd::Ohlc { i, .. }
        | DrawCmd::StyledOhlc { i, .. }
        | DrawCmd::CloseLine { i, .. }
        | DrawCmd::Pagoda { i, .. } => indices.push(*i),
        DrawCmd::Polyline { pts, .. } | DrawCmd::Overlay { pts, .. } => {
            indices.extend(pts.iter().map(|(index, _)| *index));
        }
        DrawCmd::Line { i1, i2, .. } => {
            indices.push(*i1);
            indices.push(*i2);
        }
        DrawCmd::HLine { .. }
        | DrawCmd::VLine { .. }
        | DrawCmd::Text { .. }
        | DrawCmd::AxisText { .. }
        | DrawCmd::TimeText { .. }
        | DrawCmd::CrossTimeText { .. }
        | DrawCmd::StatusText { .. }
        | DrawCmd::StatusField { .. } => {}
    }
}

pub(crate) struct ChartWorld {
    pub(crate) bars: Vec<Bar>,
    pub(crate) bar_seed: u64,
    pub(crate) panes: Vec<Pane>,
    pub(crate) panes_palette: indicator::Palette,
}

impl ChartWorld {
    pub(crate) fn new(panes: Vec<Pane>, palette: indicator::Palette) -> Self {
        Self {
            bars: crate::domain::data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START),
            bar_seed: BAR_SEED,
            panes,
            panes_palette: palette,
        }
    }

    pub(crate) fn mark_panes_dirty(&mut self) {
        for pane in &mut self.panes {
            pane.dirty = true;
        }
    }

    pub(crate) fn clear_pan(&mut self) {
        for pane in &mut self.panes {
            pane.view.pan = None;
        }
    }

    pub(crate) fn clear_crosshair(&mut self) {
        for pane in &mut self.panes {
            pane.view.crosshair = None;
        }
    }

    pub(crate) fn clear_navigation(&mut self) {
        for pane in &mut self.panes {
            pane.view.crosshair = None;
            pane.view.pan = None;
        }
    }

    pub(crate) fn clear_pane_navigation(&mut self, pane: usize) {
        self.panes[pane].view.crosshair = None;
        self.panes[pane].view.pan = None;
    }

    pub(crate) fn ensure_pane_computed(&mut self, pane: usize) {
        if !self.panes[pane].dirty {
            return;
        }
        let rendered = self.panes[pane]
            .script
            .as_ref()
            .map(|script| script.render(&self.bars, &self.panes_palette));
        let pane = &mut self.panes[pane];
        match rendered {
            Some(Ok(mut render)) => {
                pane.cmds.set(std::mem::take(&mut render.cmds));
                pane.script_render = Some(render);
                pane.script_error = None;
            }
            Some(Err(error)) => {
                pane.cmds.clear();
                pane.script_error = Some(error);
            }
            None => {
                pane.cmds.clear();
                pane.script_render = None;
                pane.script_error = None;
            }
        }
        pane.dirty = false;
    }

    pub(crate) fn regenerate_market(&mut self) {
        self.bars = crate::domain::data::random_walk::generate(self.bar_seed, BAR_COUNT, BAR_START);
        self.mark_panes_dirty();
    }

    pub(crate) fn add_pane(&mut self, pane: Pane) -> usize {
        self.panes.push(pane);
        self.panes.len() - 1
    }

    pub(crate) fn replace_pane(&mut self, index: usize, pane: Pane) {
        self.panes[index] = pane;
    }
}
