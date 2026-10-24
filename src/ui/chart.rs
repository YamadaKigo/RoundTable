use crate::application::app::*;
use crate::domain::indicator::{ChartCtx, ChartLayout, DrawCmd};
use crate::domain::layout::*;
use crate::ui::menu::*;

use crate::engine::font::HardText;
use crate::engine::kit;
use crate::engine::theme::*;

pub(crate) const CHART_PAD: f32 = GRID;

#[expect(
    clippy::too_many_arguments,
    reason = "hard-edge primitive keeps geometry arguments explicit"
)]
pub(crate) fn paint_candle(
    painter: &kit::Painter,
    left: f32,
    bar_w: f32,
    rising: bool,
    color: kit::Color32,
    y_open: f32,
    y_close: f32,
    y_high: f32,
    y_low: f32,
) {
    paint_candle_w(
        painter, left, bar_w, rising, color, 1, y_open, y_close, y_high, y_low,
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "hard-edge primitive keeps geometry arguments explicit"
)]
pub(crate) fn paint_candle_w(
    painter: &kit::Painter,
    left: f32,
    bar_w: f32,
    rising: bool,
    color: kit::Color32,
    width: u32,
    y_open: f32,
    y_close: f32,
    y_high: f32,
    y_low: f32,
) {
    let x = px_center(left + (bar_w - LINE_W) / 2.0);
    vline_hard_width(painter, x, y_high, y_low, color, width);

    let top = (y_open - 0.5).round().min((y_close - 0.5).round());
    let height = ((y_open - 0.5).round() - (y_close - 0.5).round()).abs() + LINE_W;
    paint_bar(
        painter,
        left,
        bar_w,
        rising,
        width,
        color,
        top,
        top + height,
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "hard-edge primitive keeps geometry arguments explicit"
)]
pub(crate) fn paint_ohlc(
    painter: &kit::Painter,
    left: f32,
    bar_w: f32,
    color: kit::Color32,
    y_open: f32,
    y_close: f32,
    y_high: f32,
    y_low: f32,
) {
    paint_ohlc_w(
        painter, left, bar_w, color, 1, y_open, y_close, y_high, y_low,
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "hard-edge primitive keeps geometry arguments explicit"
)]
pub(crate) fn paint_ohlc_w(
    painter: &kit::Painter,
    left: f32,
    bar_w: f32,
    color: kit::Color32,
    width: u32,
    y_open: f32,
    y_close: f32,
    y_high: f32,
    y_low: f32,
) {
    let center = px_center(left + (bar_w - LINE_W) / 2.0);
    vline_hard_width(painter, center, y_high, y_low, color, width);

    let right = left + bar_w;
    hline_hard_width(painter, center, right, y_close, color, width);
    hline_hard_width(painter, left, center, y_open, color, width);
}

#[expect(
    clippy::too_many_arguments,
    reason = "hard-edge primitive keeps geometry arguments explicit"
)]
pub(crate) fn paint_bar(
    painter: &kit::Painter,
    left: f32,
    bar_w: f32,
    hollow: bool,
    width: u32,
    color: kit::Color32,
    y0: f32,
    y1: f32,
) {
    let rect = kit::Rect::from_min_max(
        kit::pos2(left, y0.min(y1)),
        kit::pos2(left + bar_w, y0.max(y1)),
    );
    painter.rect_filled(rect, kit::CornerRadius::ZERO, color);
    let width = width.max(1) as f32;
    if hollow && rect.width() > 2.0 * width && rect.height() > 2.0 * width {
        let inner = kit::Rect::from_min_max(
            kit::pos2(rect.min.x + width, rect.min.y + width),
            kit::pos2(rect.max.x - width, rect.max.y - width),
        );
        painter.rect_filled(inner, kit::CornerRadius::ZERO, theme().chart_bg);
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "hard-edge primitive keeps geometry arguments explicit"
)]
pub(crate) fn paint_pagoda(
    painter: &kit::Painter,
    left: f32,
    bar_w: f32,
    color: kit::Color32,
    width: u32,
    y_open: f32,
    y_close: f32,
    y_high: f32,
    y_low: f32,
) {
    let center = px_center(left + (bar_w - LINE_W) / 2.0);
    let top = y_open.min(y_close);
    let bottom = y_open.max(y_close);
    hline_hard_width(painter, left, left + bar_w, y_high, color, width);
    hline_hard_width(painter, left, left + bar_w, y_low, color, width);
    vline_hard_width(painter, left, top, bottom, color, width);
    vline_hard_width(painter, left + bar_w - LINE_W, top, bottom, color, width);
    let step = if y_close >= y_open { bottom } else { top };
    hline_hard_width(painter, center, left + bar_w, step, color, width);
}

pub(crate) fn chart_rows(
    area: kit::Rect,
    layout: ChartLayout,
) -> (kit::Rect, Option<kit::Rect>, kit::Rect, Option<kit::Rect>) {
    let title = grid_band(area, 0, PANE_TITLE_ROWS);
    let mut row = PANE_TITLE_ROWS;
    let status = if layout.status_rows > 0 {
        let band = grid_band(area, row, layout.status_rows);
        row += layout.status_rows;
        Some(band)
    } else {
        None
    };

    let main_y = area.min.y + grid_line(row) + LINE_W;

    let tail = if layout.time_rows > 0 {
        cells(layout.time_rows) + CHART_BAR_H - LINE_W
    } else {
        CHART_BAR_H
    };
    let main_h = (area.max.y - main_y - tail).max(0.0);
    let main = kit::Rect::from_min_size(
        kit::pos2(area.min.x, main_y),
        kit::vec2(area.width(), main_h),
    );

    let time = if layout.time_rows > 0 {
        Some(kit::Rect::from_min_size(
            kit::pos2(area.min.x, main_y + main_h),
            kit::vec2(area.width(), cells(layout.time_rows)),
        ))
    } else {
        None
    };
    (title, status, main, time)
}

pub(crate) fn chart_bar(area: kit::Rect) -> kit::Rect {
    kit::Rect::from_min_size(
        kit::pos2(area.min.x, area.max.y - CHART_BAR_H),
        kit::vec2(area.width(), CHART_BAR_H),
    )
}

pub(crate) fn status_field_metrics(label_w: f32, value_w: f32) -> (f32, f32) {
    let divider_cells = ((TEXT_PAD_X + label_w + GRID) / PITCH).ceil() as i32;
    let divider = grid_line(divider_cells);
    let width = snap_cells_up(divider + GRID + value_w + TEXT_PAD_X);
    (width, divider)
}

#[cfg(test)]
pub(crate) fn frame_rects(area: kit::Rect) -> (kit::Rect, kit::Rect) {
    frame_rects_with_axis(area, true)
}

pub(crate) fn frame_rects_with_axis(area: kit::Rect, show_axis: bool) -> (kit::Rect, kit::Rect) {
    if !show_axis {
        let plot = kit::Rect::from_min_max(
            kit::pos2(area.min.x + LINE_W, area.min.y),
            kit::pos2(area.max.x, area.max.y),
        );
        return (plot, kit::Rect::NOTHING);
    }
    let edge = area.max.x - AXIS_W;
    let plot = kit::Rect::from_min_max(
        kit::pos2(area.min.x + LINE_W, area.min.y),
        kit::pos2(edge, area.max.y),
    );
    let axis = kit::Rect::from_min_max(kit::pos2(edge, area.min.y), area.max);
    (plot, axis)
}

pub(crate) struct CmdMap {
    plot: kit::Rect,

    from: usize,
    to: usize,

    advance: f32,
    bar_w: f32,

    top: f32,
    span_px: f32,
    hi: f64,
    lo: f64,
}

impl CmdMap {
    fn visible(&self, i: usize) -> bool {
        i >= self.from && i < self.to
    }

    fn x_left(&self, i: usize) -> f32 {
        (self.plot.min.x + i.saturating_sub(self.from) as f32 * self.advance).round()
    }

    fn line_x(&self, i: usize) -> f32 {
        self.x_left(i) + (self.bar_w - LINE_W) / 2.0
    }

    fn line_at(&self, i: f64) -> f32 {
        self.plot.min.x + (i - self.from as f64) as f32 * self.advance
    }

    fn span_data(&self) -> f64 {
        (self.hi - self.lo).max(f64::EPSILON)
    }

    fn y(&self, v: f64) -> f32 {
        let t = ((self.hi - v) / self.span_data()).clamp(0.0, 1.0) as f32;
        px_center(self.top + t * self.span_px)
    }
}

pub(crate) struct ChartGeom {
    plot: kit::Rect,
    axis: kit::Rect,
    status: Option<kit::Rect>,
    time: Option<kit::Rect>,
    map: CmdMap,

    bottom: f32,

    origin: kit::Pos2,
}

struct ChartReplayText<'a> {
    engine: Option<&'a mut HardText>,
}

impl ChartReplayText<'_> {
    fn text(
        &mut self,
        painter: &kit::Painter,
        pos: kit::Pos2,
        align: kit::Align2,
        value: &str,
        color: kit::Color32,
    ) {
        if let Some(engine) = &mut self.engine {
            engine.draw(painter, pos, align, value, color);
        }
    }

    fn width(&mut self, value: &str) -> f32 {
        self.engine
            .as_mut()
            .map_or(0.0, |engine| engine.width(value) as f32)
    }
}

fn color32(c: u32) -> kit::Color32 {
    kit::Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

#[cfg(test)]
pub(crate) fn pane_range(cmds: &[DrawCmd], from: usize, to: usize) -> (f64, f64) {
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for cmd in cmds {
        include_range(cmd, from, to, &mut lo, &mut hi);
    }
    if lo > hi { (0.0, 0.0) } else { (lo, hi) }
}

pub(crate) fn pane_range_indexed(
    cmds: &PaneCmds,
    from: usize,
    to: usize,
    positions: &mut Vec<u32>,
) -> (f64, f64) {
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    cmds.visible_indices_into(from, to, positions);
    for &position in positions.iter() {
        include_range(&cmds[position as usize], from, to, &mut lo, &mut hi);
    }
    if lo > hi { (0.0, 0.0) } else { (lo, hi) }
}

fn include_range(cmd: &DrawCmd, from: usize, to: usize, lo: &mut f64, hi: &mut f64) {
    let visible = |index: usize| index >= from && index < to;
    match cmd {
        DrawCmd::Candle { i, h, l, .. } | DrawCmd::StyledCandle { i, h, l, .. } => {
            if visible(*i) {
                *lo = lo.min(*l);
                *hi = hi.max(*h);
            }
        }
        DrawCmd::Bar { i, v0, v1, .. } | DrawCmd::StyledBar { i, v0, v1, .. } => {
            if visible(*i) {
                *lo = lo.min(*v0).min(*v1);
                *hi = hi.max(*v0).max(*v1);
            }
        }
        DrawCmd::Ohlc { i, h, l, .. } | DrawCmd::StyledOhlc { i, h, l, .. } => {
            if visible(*i) {
                *lo = lo.min(*l);
                *hi = hi.max(*h);
            }
        }
        DrawCmd::CloseLine { i, p0, p1, .. } => {
            if visible(*i) {
                *lo = lo.min(*p0).min(*p1);
                *hi = hi.max(*p0).max(*p1);
            }
        }
        DrawCmd::Pagoda { i, o, h, l, c, .. } => {
            if visible(*i) {
                *lo = lo.min(*l).min(*o).min(*c);
                *hi = hi.max(*h).max(*o).max(*c);
            }
        }
        DrawCmd::Polyline { pts, .. } | DrawCmd::Overlay { pts, .. } => {
            for (_, point) in points_range(pts, from, to) {
                *lo = lo.min(*point);
                *hi = hi.max(*point);
            }
        }
        DrawCmd::Line { i1, p1, i2, p2, .. } => {
            if visible(*i1) {
                *lo = lo.min(*p1);
                *hi = hi.max(*p1);
            }
            if visible(*i2) {
                *lo = lo.min(*p2);
                *hi = hi.max(*p2);
            }
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

fn points_range(pts: &[(usize, f64)], from: usize, to: usize) -> &[(usize, f64)] {
    let start = pts.partition_point(|(index, _)| *index < from);
    let end = pts.partition_point(|(index, _)| *index < to);
    &pts[start..end.max(start)]
}

impl AppState {
    fn pane_view_range_with_scratch(
        &self,
        pane: usize,
        from: usize,
        to: usize,
        positions: &mut Vec<u32>,
    ) -> (f64, f64) {
        let (lo, hi) = pane_range_indexed(&self.chart_world.panes[pane].cmds, from, to, positions);
        self.chart_world.panes[pane].chart_value_range(lo, hi)
    }

    #[cfg(test)]
    pub(crate) fn pane_view_range(&self, pane: usize, from: usize, to: usize) -> (f64, f64) {
        self.pane_view_range_with_scratch(pane, from, to, &mut Vec::new())
    }

    pub(crate) fn chart_interact(&mut self, ui: &kit::Ui, plots: &[kit::Rect], bars: &[kit::Rect]) {
        if self.ui_drag_active() {
            self.chart_world.clear_pan();
            return;
        }

        let Some(focused) = self.chart_ui.focused_pane else {
            self.chart_world.clear_pan();
            return;
        };
        let hover = ui.input(|i| i.pointer.hover_pos());
        let over_plot = hover
            .filter(|p| plots[focused].contains(*p))
            .map(|p| (focused, p));

        if let Some((pane, p)) = over_plot {
            let notches = self.take_wheel_notches(ui, Some(p), pane);
            if notches != 0 {
                self.zoom_around(pane, p.x - plots[pane].min.x, notches);
            }
        }

        let bottom = ui.max_rect().max.y;
        let (pressed, down, press_pos, latest) = ui.input(|i| {
            (
                i.pointer.primary_pressed(),
                i.pointer.primary_down(),
                i.pointer.interact_pos(),
                i.pointer.latest_pos(),
            )
        });

        if pressed {
            if let Some(p) = press_pos {
                if !bars.iter().any(|r| r.contains(p)) && !self.floating_at(p, bottom) {
                    for (pane, plot) in plots.iter().enumerate() {
                        if plot.contains(p) {
                            let offset = self.chart_world.panes[pane].view.offset;
                            self.chart_world.panes[pane].view.pan = Some((p.x, offset));
                        }
                    }
                }
            }
        }
        if down {
            if let Some(p) = latest {
                let bar_count = self.chart_world.bars.len();
                for pane in &mut self.chart_world.panes {
                    let Some((start_x, start_offset)) = pane.view.pan else {
                        continue;
                    };
                    let bars = (((p.x - start_x) / bar_advance(pane.view.scale)) as f64).round();
                    let want = (start_offset as f64 - bars).max(0.0) as usize;
                    pane.view.offset = want.min(bar_count.saturating_sub(pane.view.visible));
                    if pane.view.offset as f64 != start_offset as f64 - bars {
                        pane.view.pan = Some((p.x, pane.view.offset));
                    }
                }
            }
        } else {
            self.chart_world.clear_pan();
        }
    }

    pub(crate) fn update_crosshair(&mut self, ui: &kit::Ui, plots: &[kit::Rect]) {
        if self.ui_drag_active() {
            self.chart_world.clear_crosshair();
            return;
        }
        let Some(pane) = self.chart_ui.focused_pane else {
            self.chart_world.clear_crosshair();
            return;
        };
        let hover = ui.input(|i| i.pointer.hover_pos());
        let dragging = self.chart_world.panes[pane].view.pan.is_some();
        let point = hover.or_else(|| {
            if dragging {
                ui.input(|i| i.pointer.latest_pos())
            } else {
                None
            }
        });
        let Some(p) = point else {
            self.chart_world.clear_crosshair();
            return;
        };

        if !dragging && !plots[pane].contains(p) {
            self.chart_world.clear_crosshair();
            return;
        }

        if !dragging && self.floating_at(p, ui.max_rect().max.y) {
            self.chart_world.clear_crosshair();
            return;
        }
        if self.chart_world.panes[pane].view.visible == 0 {
            self.chart_world.panes[pane].view.crosshair = None;
            return;
        }

        let plot = plots[pane];
        let from = self.chart_world.panes[pane].view.offset;
        let to =
            (from + self.chart_world.panes[pane].view.visible).min(self.chart_world.bars.len());
        let advance = bar_advance(self.chart_world.panes[pane].view.scale);
        let top = plot.min.y + CHART_PAD;
        let span_px = (plot.max.y - CHART_PAD - top)
            .max(LINE_W)
            .min((self.chart_world.panes[pane].height as f32).max(LINE_W));
        let mut positions = std::mem::take(&mut self.chart_ui.visible_scratch);
        let (lo, hi) = self.pane_view_range_with_scratch(pane, from, to, &mut positions);
        self.chart_ui.visible_scratch = positions;
        let span = (hi - lo).max(f64::EPSILON);

        let x = px_center(p.x.clamp(plot.min.x, plot.max.x - LINE_W));
        let y = px_center(p.y.clamp(top, top + span_px));
        let fpos = (from as f32 + (x - plot.min.x) / advance).max(0.0);
        let index = (fpos.floor() as usize).min(self.chart_world.bars.len().saturating_sub(1));

        let t = ((y - top) / span_px).clamp(0.0, 1.0) as f64;
        let crosshair = Crosshair {
            value: hi - span * t,
            index,
            fpos: fpos as f64,
            pane,
        };
        for (i, slot) in self.chart_world.panes.iter_mut().enumerate() {
            slot.view.crosshair = (i == pane).then_some(crosshair);
        }
    }

    pub(crate) fn ui_chart(&mut self, ui: &mut kit::Ui, pane: usize, area: kit::Rect) {
        let painter = ui.painter().clone();
        if self.chart_world.panes[pane].dirty {
            self.sync_pane_palette();
        }
        self.ensure_pane_computed(pane);
        let layout = self.chart_world.panes[pane].chart_layout();
        let (title, status, main, time) = chart_rows(area, layout);
        let show_axis = self.chart_world.panes[pane].show_axis();
        let chart_layer = ui.ctx().current_layer();
        let dynamic_overlay = chart_layer == kit::PaintLayer::Chart;
        let (plot, axis) = frame_rects_with_axis(main, show_axis);

        painter.rect_filled(area, kit::CornerRadius::ZERO, theme().work_bg);
        painter.rect_filled(plot, kit::CornerRadius::ZERO, theme().chart_bg);
        if show_axis {
            painter.rect_filled(
                kit::Rect::from_min_max(
                    kit::pos2(axis.min.x, main.min.y - LINE_W),
                    kit::pos2(axis.max.x, main.max.y),
                ),
                kit::CornerRadius::ZERO,
                theme().axis_bg,
            );
        }

        hline_hard(
            &painter,
            area.min.x,
            area.max.x,
            main.min.y - LINE_W,
            theme().chart_line,
        );
        if let Some(t) = time {
            hline_hard(
                &painter,
                area.min.x,
                area.max.x,
                t.min.y,
                theme().chart_line,
            );
        }

        self.ui_pane_title(ui, pane, title);

        let bar_segs = self.chart_status_segments(pane);
        let bar = chart_bar(area);
        self.paint_status_bar_colors(
            &painter,
            bar,
            &bar_segs,
            (
                theme().chart_status_bg,
                theme().chart_status_label,
                theme().chart_status_divider,
            ),
            if dynamic_overlay {
                kit::PaintLayer::Dynamic
            } else {
                chart_layer
            },
        );

        self.chart_world.panes[pane].height = main.height() as i32;
        self.clamp_view(pane, plot.width());
        let visible = self.chart_world.panes[pane].view.visible;
        if visible == 0 {
            return;
        }

        let from = self.chart_world.panes[pane].view.offset;
        let to = (from + visible).min(self.chart_world.bars.len());
        let advance = bar_advance(self.chart_world.panes[pane].view.scale);
        let bar_w = snap_scale(self.chart_world.panes[pane].view.scale) as f32;
        let mut positions = std::mem::take(&mut self.chart_ui.visible_scratch);
        let (lo, hi) = self.pane_view_range_with_scratch(pane, from, to, &mut positions);
        let top = plot.min.y + CHART_PAD;
        let span_px = (plot.max.y - CHART_PAD - top)
            .max(LINE_W)
            .min((self.chart_world.panes[pane].height as f32).max(LINE_W));
        let time_step = self.time_step(advance);
        let map = CmdMap {
            plot,
            from,
            to,
            advance,
            bar_w,
            top,
            span_px,
            hi,
            lo,
        };
        let geom = ChartGeom {
            plot,
            axis,
            status,
            time,
            map,

            bottom: main.max.y,

            origin: area.min,
        };

        let data = &self.chart_world.panes[pane].cmds;
        let crosshair = self.chart_world.panes[pane].view.crosshair;
        let (axis_cmds, time_cmds, status_cmds, overlay_cmds) = {
            let ctx = ChartCtx {
                bars: &self.chart_world.bars,
                cmds: data,
                from,
                to,
                visible,
                advance,
                time_step,
                lo,
                hi,
                cursor_bar: crosshair.map(|c| c.index),
                cursor_index: crosshair.map(|c| c.fpos),
                cursor_value: crosshair.filter(|c| c.pane == pane).map(|c| c.value),
            };
            (
                self.chart_world.panes[pane].chart_axis(&ctx),
                self.chart_world.panes[pane].chart_time(&ctx),
                self.chart_world.panes[pane].chart_status(&ctx),
                self.chart_world.panes[pane].chart_crosshair(&ctx),
            )
        };

        let mut text = ChartReplayText {
            engine: self.assets.text_engine.as_mut(),
        };
        let mut run = Vec::new();
        Self::replay_cmds(
            &mut text,
            &painter,
            &geom,
            &axis_cmds,
            chart_layer,
            dynamic_overlay,
            &mut run,
        );
        data.visible_indices_into(from, to, &mut positions);
        for &position in &positions {
            Self::replay_cmds(
                &mut text,
                &painter,
                &geom,
                std::slice::from_ref(&data[position as usize]),
                chart_layer,
                dynamic_overlay,
                &mut run,
            );
        }
        if show_axis {
            vline_hard(
                &painter,
                axis.min.x,
                axis.min.y,
                axis.max.y,
                theme().chart_line,
            );
        }
        Self::replay_cmds(
            &mut text,
            &painter,
            &geom,
            &time_cmds,
            chart_layer,
            dynamic_overlay,
            &mut run,
        );
        Self::replay_cmds(
            &mut text,
            &painter,
            &geom,
            &status_cmds,
            if dynamic_overlay {
                kit::PaintLayer::Dynamic
            } else {
                chart_layer
            },
            dynamic_overlay,
            &mut run,
        );
        Self::replay_cmds(
            &mut text,
            &painter,
            &geom,
            &overlay_cmds,
            if dynamic_overlay {
                kit::PaintLayer::Dynamic
            } else {
                chart_layer
            },
            dynamic_overlay,
            &mut run,
        );
        self.chart_ui.visible_scratch = positions;
    }

    fn replay_cmds(
        text: &mut ChartReplayText<'_>,
        painter: &kit::Painter,
        geom: &ChartGeom,
        cmds: &[DrawCmd],
        default_layer: kit::PaintLayer,
        dynamic_enabled: bool,
        run: &mut Vec<(f32, f32)>,
    ) {
        let map = &geom.map;

        let mut sx = geom.status.map_or(0.0, |r| r.min.x);
        let mut first = true;
        for cmd in cmds {
            let layer = match cmd {
                _ if !dynamic_enabled => default_layer,
                DrawCmd::AxisText { bg, .. } if *bg != 0 => kit::PaintLayer::Dynamic,
                DrawCmd::TimeText { bg, .. } if *bg != 0 => kit::PaintLayer::Dynamic,
                DrawCmd::CrossTimeText { .. }
                | DrawCmd::StatusText { .. }
                | DrawCmd::StatusField { .. } => kit::PaintLayer::Dynamic,
                _ => default_layer,
            };
            let painter = &painter.paint_layer(layer);
            match cmd {
                DrawCmd::Candle {
                    i,
                    o,
                    h,
                    l,
                    c,
                    color,
                } => {
                    if !map.visible(*i) {
                        continue;
                    }
                    paint_candle(
                        painter,
                        map.x_left(*i),
                        map.bar_w,
                        c >= o,
                        color32(*color),
                        map.y(*o),
                        map.y(*c),
                        map.y(*h),
                        map.y(*l),
                    );
                }
                DrawCmd::StyledCandle {
                    i,
                    o,
                    h,
                    l,
                    c,
                    color,
                    w,
                } => {
                    if !map.visible(*i) {
                        continue;
                    }
                    paint_candle_w(
                        painter,
                        map.x_left(*i),
                        map.bar_w,
                        c >= o,
                        color32(*color),
                        *w,
                        map.y(*o),
                        map.y(*c),
                        map.y(*h),
                        map.y(*l),
                    );
                }
                DrawCmd::Bar {
                    i,
                    v0,
                    v1,
                    color,
                    hollow,
                } => {
                    if !map.visible(*i) {
                        continue;
                    }
                    let left = map.x_left(*i);
                    let (a, b) = (map.y(*v0), map.y(*v1));
                    let rect = kit::Rect::from_min_max(
                        kit::pos2(left, a.min(b)),
                        kit::pos2(left + map.bar_w, a.max(b)),
                    );
                    painter.rect_filled(rect, kit::CornerRadius::ZERO, color32(*color));

                    if *hollow && map.bar_w >= 3.0 && rect.height() >= 3.0 {
                        let inner = kit::Rect::from_min_max(
                            kit::pos2(rect.min.x + LINE_W, rect.min.y + LINE_W),
                            kit::pos2(rect.max.x - LINE_W, rect.max.y - LINE_W),
                        );
                        painter.rect_filled(inner, kit::CornerRadius::ZERO, theme().chart_bg);
                    }
                }
                DrawCmd::StyledBar {
                    i,
                    v0,
                    v1,
                    color,
                    hollow,
                    w,
                } => {
                    if !map.visible(*i) {
                        continue;
                    }
                    paint_bar(
                        painter,
                        map.x_left(*i),
                        map.bar_w,
                        *hollow,
                        *w,
                        color32(*color),
                        map.y(*v0),
                        map.y(*v1),
                    );
                }
                DrawCmd::Ohlc {
                    i,
                    o,
                    h,
                    l,
                    c,
                    color,
                } => {
                    if !map.visible(*i) {
                        continue;
                    }
                    paint_ohlc(
                        painter,
                        map.x_left(*i),
                        map.bar_w,
                        color32(*color),
                        map.y(*o),
                        map.y(*c),
                        map.y(*h),
                        map.y(*l),
                    );
                }
                DrawCmd::StyledOhlc {
                    i,
                    o,
                    h,
                    l,
                    c,
                    color,
                    w,
                } => {
                    if !map.visible(*i) {
                        continue;
                    }
                    paint_ohlc_w(
                        painter,
                        map.x_left(*i),
                        map.bar_w,
                        color32(*color),
                        *w,
                        map.y(*o),
                        map.y(*c),
                        map.y(*h),
                        map.y(*l),
                    );
                }
                DrawCmd::CloseLine {
                    i,
                    p0,
                    p1,
                    color,
                    w,
                } => {
                    if !map.visible(*i) {
                        continue;
                    }
                    let x = map.line_x(*i);
                    polyline_hard_width(
                        painter,
                        &[(x, map.y(*p0)), (x - map.advance, map.y(*p1))],
                        color32(*color),
                        *w,
                    );
                }
                DrawCmd::Pagoda {
                    i,
                    o,
                    h,
                    l,
                    c,
                    color,
                    w,
                } => {
                    if !map.visible(*i) {
                        continue;
                    }
                    paint_pagoda(
                        painter,
                        map.x_left(*i),
                        map.bar_w,
                        color32(*color),
                        *w,
                        map.y(*o),
                        map.y(*c),
                        map.y(*h),
                        map.y(*l),
                    );
                }
                DrawCmd::Polyline { pts, color, w } | DrawCmd::Overlay { pts, color, w } => {
                    debug_assert!(*w >= 1, "Polyline width must be at least 1px");
                    run.clear();
                    for (i, p) in points_range(pts, map.from, map.to) {
                        run.push((map.line_x(*i), map.y(*p)));
                    }

                    if run.len() > 1 {
                        polyline_hard_width(painter, run, color32(*color), *w);
                    }
                }
                DrawCmd::Line {
                    i1,
                    p1,
                    i2,
                    p2,
                    color,
                    w,
                } => {
                    debug_assert!(*w >= 1, "Line width must be at least 1px");
                    painter.line_segment(
                        [
                            kit::pos2(map.line_x(*i1), map.y(*p1)),
                            kit::pos2(map.line_x(*i2), map.y(*p2)),
                        ],
                        kit::Stroke::new(*w as f32, color32(*color)),
                    );
                }
                DrawCmd::HLine { p, color, w } => {
                    debug_assert!(*w >= 1, "Reference line width must be at least 1px");

                    if *p >= map.lo && *p <= map.hi {
                        hline_hard_width(
                            painter,
                            geom.plot.min.x,
                            geom.plot.max.x,
                            map.y(*p),
                            color32(*color),
                            *w,
                        );
                    }
                }
                DrawCmd::VLine { i, color, w } => {
                    debug_assert!(*w >= 1, "Vertical line width must be at least 1px");
                    vline_hard_width(
                        painter,
                        map.line_at(*i),
                        geom.plot.min.y,
                        geom.plot.max.y,
                        color32(*color),
                        *w,
                    );
                }
                DrawCmd::Text { i, p, s, color } => {
                    let pos = kit::pos2(map.line_x(*i), map.y(*p));
                    text.text(painter, pos, kit::Align2::CENTER_CENTER, s, color32(*color));
                }
                DrawCmd::AxisText { p, s, fg, bg } => {
                    let y = map.y(*p);
                    if *bg == 0 {
                        if !geom.axis.is_positive() {
                            continue;
                        }
                        text.text(
                            painter,
                            kit::pos2(Self::axis_tick_x(geom.axis), y),
                            kit::Align2::RIGHT_CENTER,
                            s,
                            color32(*fg),
                        );
                    } else {
                        if !geom.axis.is_positive() {
                            continue;
                        }
                        if geom.plot.height() < CROSS_LABEL_H {
                            continue;
                        }
                        let w = cells(AXIS_COLS - 2);
                        let x0 = Self::cross_axis_label_x(geom.axis, w);

                        let y0 = floating_label_top(
                            y,
                            CROSS_LABEL_ROWS,
                            geom.plot.min.y - LINE_W,
                            geom.bottom,
                            geom.origin.y,
                        );
                        Self::axis_label_box(
                            text,
                            painter,
                            s,
                            kit::pos2(x0, y0),
                            w,
                            color32(*bg),
                            theme().cross_label_text,
                        );
                    }
                }
                DrawCmd::TimeText { i, s, fg, bg } => {
                    if *bg == 0 {
                        let Some(band) = geom.time else {
                            continue;
                        };
                        let cx = map.line_x(*i);
                        let w = text.width(s);
                        if cx - w / 2.0 < geom.plot.min.x || cx + w / 2.0 > geom.plot.max.x - LINE_W
                        {
                            continue;
                        }
                        text.text(
                            painter,
                            kit::pos2(cx, band.center().y),
                            kit::Align2::CENTER_CENTER,
                            s,
                            color32(*fg),
                        );
                    } else {
                        if geom.time.is_none() {
                            continue;
                        }
                        let w = cells(AXIS_COLS - 2);
                        let x0 = Self::cross_time_label_x(
                            geom.plot,
                            map.line_x(*i),
                            w,
                            Self::cross_time_label_right(geom.plot, geom.axis),
                        );

                        Self::label_box(
                            text,
                            painter,
                            s,
                            kit::pos2(x0, geom.bottom - PITCH * CROSS_LABEL_ROWS as f32),
                            w,
                            color32(*bg),
                            theme().cross_label_text,
                        );
                    }
                }
                DrawCmd::CrossTimeText { i, s, bg } => {
                    if geom.time.is_none() {
                        continue;
                    }
                    let w = cells(AXIS_COLS - 2);
                    let x0 = Self::cross_time_label_x(
                        geom.plot,
                        map.line_at(*i),
                        w,
                        Self::cross_time_label_right(geom.plot, geom.axis),
                    );
                    Self::label_box(
                        text,
                        painter,
                        s,
                        kit::pos2(x0, geom.bottom - PITCH * CROSS_LABEL_ROWS as f32),
                        w,
                        color32(*bg),
                        theme().cross_label_text,
                    );
                }
                DrawCmd::StatusText { s, color } => {
                    let Some(band) = geom.status else {
                        continue;
                    };
                    let w = snap_cells_up(text.width(s) + 2.0 * TEXT_PAD_X);
                    if sx + w > band.max.x {
                        break;
                    }
                    Self::status_divider(painter, band, sx, first);
                    text.text(
                        painter,
                        kit::pos2(text_left_x(sx, geom.origin.x), band.center().y),
                        kit::Align2::LEFT_CENTER,
                        s,
                        color32(*color),
                    );
                    sx += w - LINE_W;
                    first = false;
                }
                DrawCmd::StatusField {
                    label,
                    value,
                    color,
                } => {
                    let Some(band) = geom.status else {
                        continue;
                    };
                    if sx >= band.max.x - LINE_W {
                        break;
                    }
                    let label_w = text.width(label);
                    let value_w = text.width(value);
                    let (w, divider) = status_field_metrics(label_w, value_w);
                    let clipped = painter.with_clip_rect(band);
                    if sx < band.max.x {
                        vline_hard(
                            &clipped,
                            sx,
                            band.min.y + LINE_W,
                            band.max.y - LINE_W,
                            theme().chart_status_divider,
                        );
                    }
                    text.text(
                        &clipped,
                        kit::pos2(text_left_x(sx, geom.origin.x), band.center().y),
                        kit::Align2::LEFT_CENTER,
                        label,
                        theme().chart_status_label,
                    );
                    if sx + divider < band.max.x {
                        vline_hard(
                            &clipped,
                            sx + divider,
                            band.min.y + LINE_W,
                            band.max.y - LINE_W,
                            theme().chart_status_divider,
                        );
                    }
                    text.text(
                        &clipped,
                        kit::pos2(
                            text_right_x(sx + w - LINE_W, geom.origin.x),
                            band.center().y,
                        ),
                        kit::Align2::RIGHT_CENTER,
                        value,
                        color32(*color),
                    );
                    sx += w - LINE_W;
                    first = false;
                }
            }
        }
        if let Some(band) = geom.status {
            if !first && sx < band.max.x {
                vline_hard(
                    painter,
                    sx,
                    band.min.y + LINE_W,
                    band.max.y - LINE_W,
                    theme().chart_status_divider,
                );
                let right = band.max.x - LINE_W;
                if right > sx {
                    vline_hard(
                        painter,
                        right,
                        band.min.y + LINE_W,
                        band.max.y - LINE_W,
                        theme().chart_status_divider,
                    );
                }
            }
        }
    }

    fn status_divider(painter: &kit::Painter, band: kit::Rect, x: f32, first: bool) {
        if first {
            return;
        }
        vline_hard(
            painter,
            x,
            band.min.y + LINE_W,
            band.max.y - LINE_W,
            theme().chart_status_divider,
        );
    }

    fn label_box(
        text_state: &mut ChartReplayText<'_>,
        painter: &kit::Painter,
        text: &str,
        pos: kit::Pos2,
        w: f32,
        bg: kit::Color32,
        fg: kit::Color32,
    ) {
        let rect = kit::Rect::from_min_size(pos, kit::vec2(w, CROSS_LABEL_H));
        tag_box(painter, rect, bg, theme().cross_label_border);
        text_state.text(painter, rect.center(), kit::Align2::CENTER_CENTER, text, fg);
    }

    fn axis_label_box(
        text_state: &mut ChartReplayText<'_>,
        painter: &kit::Painter,
        text: &str,
        pos: kit::Pos2,
        w: f32,
        bg: kit::Color32,
        fg: kit::Color32,
    ) {
        let rect = kit::Rect::from_min_size(pos, kit::vec2(w, CROSS_LABEL_H));
        tag_box(painter, rect, bg, theme().cross_label_border);
        text_state.text(
            painter,
            kit::pos2(rect.min.x + PITCH, rect.center().y),
            kit::Align2::LEFT_CENTER,
            text,
            fg,
        );
    }

    pub(crate) const PANE_TABS: [&'static str; 3] = ["Main", "View", "Menu"];

    #[cfg(test)]
    pub(crate) fn pane_button_rects(title: kit::Rect, widths: &[f32]) -> Vec<kit::Rect> {
        menu_item_rects(title, widths, MenuAlign::Left)
    }

    pub(crate) fn ui_pane_title(
        &mut self,
        ui: &mut kit::Ui,
        pane: usize,
        title: kit::Rect,
    ) -> Vec<kit::Rect> {
        let open = if self.shell.pane_menu == Some(pane) {
            Some(self.shell.pane_menu_tab)
        } else {
            None
        };
        let out = menu_bar_ui(
            self,
            ui,
            title,
            &Self::PANE_TABS,
            MenuAlign::Left,
            open,
            MenuBarStyle::PANE_BAR,
        );

        if let Some(tab @ 0..=2) = out.open {
            self.shell.pane_menu = Some(pane);
            self.shell.pane_menu_tab = tab;
        } else if self.shell.pane_menu == Some(pane) {
            self.shell.pane_menu = None;
        }

        if self.shell.pane_menu == Some(pane) {
            self.shell.pane_menu_anchor = out.rects[self.shell.pane_menu_tab];
            self.shell.pane_menu_bar = title;
        }
        out.rects
    }

    pub(crate) fn take_wheel_notches(
        &mut self,
        ui: &kit::Ui,
        hover: Option<kit::Pos2>,
        pane: usize,
    ) -> i32 {
        let bottom = ui.max_rect().max.y;
        if hover.is_none()
            || self.ui_drag_active()
            || hover.is_some_and(|p| self.floating_at(p, bottom))
        {
            self.chart_world.panes[pane].view.zoom_acc = 0.0;
            return 0;
        }
        let mut acc = self.chart_world.panes[pane].view.zoom_acc;
        ui.input(|i| {
            for event in &i.events {
                if let kit::Event::MouseWheel { unit, delta, .. } = event {
                    acc += match unit {
                        kit::MouseWheelUnit::Line => delta.y,
                        kit::MouseWheelUnit::Point => delta.y / WHEEL_POINTS_PER_NOTCH,
                    };
                }
            }
        });
        let whole = acc.trunc();
        self.chart_world.panes[pane].view.zoom_acc = acc - whole;
        whole as i32
    }

    pub(crate) fn zoom_around(&mut self, pane: usize, x_rel: f32, notches: i32) {
        let old = snap_scale(self.chart_world.panes[pane].view.scale);
        let new = scale_after(old, notches);
        if new == old {
            return;
        }
        let mouse_bar = self.chart_world.panes[pane].view.offset as f32 + x_rel / bar_advance(old);
        self.chart_world.panes[pane].view.scale = new;
        self.chart_world.panes[pane].view.offset =
            (mouse_bar - x_rel / bar_advance(new)).round().max(0.0) as usize;
    }

    pub(crate) fn clamp_view(&mut self, pane: usize, plot_w: f32) {
        let bar_count = self.chart_world.bars.len();
        let view = &mut self.chart_world.panes[pane].view;
        view.visible = visible_bars(plot_w, view.scale).min(bar_count);
        view.offset = view.offset.min(bar_count.saturating_sub(view.visible));
    }

    pub(crate) fn time_step(&mut self, advance: f32) -> usize {
        let label_w = self.text_width("00-00").max(self.text_width("00-00-00"));
        let need = (label_w + GRID).max(TIME_LABEL_GAP_MIN);
        ((need / advance).ceil() as usize).max(1)
    }

    #[cfg(test)]
    pub(crate) fn cross_label_w(&mut self, _text: &str) -> f32 {
        cells(AXIS_COLS - 2)
    }

    pub(crate) fn axis_tick_x(axis: kit::Rect) -> f32 {
        axis.max.x - LINE_W - GRID
    }

    pub(crate) fn cross_axis_label_x(axis: kit::Rect, w: f32) -> f32 {
        let right = Self::axis_tick_x(axis) - w;
        clamp_soft(right, axis.min.x, right)
    }

    #[cfg(test)]
    pub(crate) fn cross_plot_label_x(plot: kit::Rect, w: f32) -> f32 {
        (plot.max.x - w).max(plot.min.x)
    }

    pub(crate) fn cross_time_label_right(plot: kit::Rect, axis: kit::Rect) -> f32 {
        if axis.is_positive() {
            plot.max.x
        } else {
            plot.max.x - LINE_W
        }
    }

    pub(crate) fn cross_time_label_x(plot: kit::Rect, cx: f32, w: f32, right: f32) -> f32 {
        clamp_soft(
            (cx - w / 2.0).round(),
            plot.min.x - LINE_W,
            right - w + LINE_W,
        )
    }
}

impl StockApp {
    pub(crate) fn chart_interact(&mut self, ui: &kit::Ui, plots: &[kit::Rect], bars: &[kit::Rect]) {
        self.sync_window_interaction_cache();
        self.state.chart_interact(ui, plots, bars);
    }

    pub(crate) fn update_crosshair(&mut self, ui: &kit::Ui, plots: &[kit::Rect]) {
        self.sync_window_interaction_cache();
        self.state.update_crosshair(ui, plots);
    }
}
