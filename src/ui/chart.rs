use crate::app::*;
use crate::grid::*;
use crate::indicator::{ChartCtx, ChartLayout, DrawCmd};
use crate::ui::menu::*;

use crate::theme::*;
use eframe::egui;

pub(crate) const CHART_PAD: f32 = GRID;

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_candle(
    painter: &egui::Painter,
    left: f32,
    bar_w: f32,
    rising: bool,
    color: egui::Color32,
    y_open: f32,
    y_close: f32,
    y_high: f32,
    y_low: f32,
) {
    paint_candle_w(
        painter, left, bar_w, rising, color, 1, y_open, y_close, y_high, y_low,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_candle_w(
    painter: &egui::Painter,
    left: f32,
    bar_w: f32,
    rising: bool,
    color: egui::Color32,
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_ohlc(
    painter: &egui::Painter,
    left: f32,
    bar_w: f32,
    color: egui::Color32,
    y_open: f32,
    y_close: f32,
    y_high: f32,
    y_low: f32,
) {
    paint_ohlc_w(
        painter, left, bar_w, color, 1, y_open, y_close, y_high, y_low,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_ohlc_w(
    painter: &egui::Painter,
    left: f32,
    bar_w: f32,
    color: egui::Color32,
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_bar(
    painter: &egui::Painter,
    left: f32,
    bar_w: f32,
    hollow: bool,
    width: u32,
    color: egui::Color32,
    y0: f32,
    y1: f32,
) {
    let rect = egui::Rect::from_min_max(
        egui::pos2(left, y0.min(y1)),
        egui::pos2(left + bar_w, y0.max(y1)),
    );
    painter.rect_filled(rect, egui::CornerRadius::ZERO, color);
    let width = width.max(1) as f32;
    if hollow && rect.width() > 2.0 * width && rect.height() > 2.0 * width {
        let inner = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + width, rect.min.y + width),
            egui::pos2(rect.max.x - width, rect.max.y - width),
        );
        painter.rect_filled(inner, egui::CornerRadius::ZERO, theme().chart_bg);
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_pagoda(
    painter: &egui::Painter,
    left: f32,
    bar_w: f32,
    color: egui::Color32,
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
    area: egui::Rect,
    layout: ChartLayout,
) -> (
    egui::Rect,
    Option<egui::Rect>,
    egui::Rect,
    Option<egui::Rect>,
) {
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
    let main = egui::Rect::from_min_size(
        egui::pos2(area.min.x, main_y),
        egui::vec2(area.width(), main_h),
    );

    let time = if layout.time_rows > 0 {
        Some(egui::Rect::from_min_size(
            egui::pos2(area.min.x, main_y + main_h),
            egui::vec2(area.width(), cells(layout.time_rows)),
        ))
    } else {
        None
    };
    (title, status, main, time)
}

pub(crate) fn chart_bar(area: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(area.min.x, area.max.y - CHART_BAR_H),
        egui::vec2(area.width(), CHART_BAR_H),
    )
}

pub(crate) fn status_field_metrics(label_w: f32, value_w: f32) -> (f32, f32) {
    let divider_cells = ((TEXT_PAD_X + label_w + GRID) / PITCH).ceil() as i32;
    let divider = grid_line(divider_cells);
    let width = snap_cells_up(divider + GRID + value_w + TEXT_PAD_X);
    (width, divider)
}

#[allow(dead_code)]
pub(crate) fn frame_rects(area: egui::Rect) -> (egui::Rect, egui::Rect) {
    frame_rects_with_axis(area, true)
}

pub(crate) fn frame_rects_with_axis(area: egui::Rect, show_axis: bool) -> (egui::Rect, egui::Rect) {
    if !show_axis {
        let plot = egui::Rect::from_min_max(
            egui::pos2(area.min.x + LINE_W, area.min.y),
            egui::pos2(area.max.x, area.max.y),
        );
        return (plot, egui::Rect::NOTHING);
    }
    let edge = area.max.x - AXIS_W;
    let plot = egui::Rect::from_min_max(
        egui::pos2(area.min.x + LINE_W, area.min.y),
        egui::pos2(edge, area.max.y),
    );
    let axis = egui::Rect::from_min_max(egui::pos2(edge, area.min.y), area.max);
    (plot, axis)
}

pub(crate) struct CmdMap {
    plot: egui::Rect,

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
    plot: egui::Rect,
    axis: egui::Rect,
    status: Option<egui::Rect>,
    time: Option<egui::Rect>,
    map: CmdMap,

    bottom: f32,

    origin: egui::Pos2,
}

fn color32(c: u32) -> egui::Color32 {
    egui::Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

pub(crate) fn pane_range(cmds: &[DrawCmd], from: usize, to: usize) -> (f64, f64) {
    let vis = |i: usize| i >= from && i < to;
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for cmd in cmds {
        match cmd {
            DrawCmd::Candle { i, h, l, .. } | DrawCmd::StyledCandle { i, h, l, .. } => {
                if vis(*i) {
                    lo = lo.min(*l);
                    hi = hi.max(*h);
                }
            }
            DrawCmd::Bar { i, v0, v1, .. } | DrawCmd::StyledBar { i, v0, v1, .. } => {
                if vis(*i) {
                    lo = lo.min(*v0).min(*v1);
                    hi = hi.max(*v0).max(*v1);
                }
            }
            DrawCmd::Ohlc { i, h, l, .. } | DrawCmd::StyledOhlc { i, h, l, .. } => {
                if vis(*i) {
                    lo = lo.min(*l);
                    hi = hi.max(*h);
                }
            }
            DrawCmd::CloseLine { i, p0, p1, .. } => {
                if vis(*i) {
                    lo = lo.min(*p0).min(*p1);
                    hi = hi.max(*p0).max(*p1);
                }
            }
            DrawCmd::Pagoda { i, o, h, l, c, .. } => {
                if vis(*i) {
                    lo = lo.min(*l).min(*o).min(*c);
                    hi = hi.max(*h).max(*o).max(*c);
                }
            }
            DrawCmd::Polyline { pts, .. } => {
                for (i, p) in pts {
                    if vis(*i) {
                        lo = lo.min(*p);
                        hi = hi.max(*p);
                    }
                }
            }
            DrawCmd::Line { i1, p1, i2, p2, .. } => {
                if vis(*i1) {
                    lo = lo.min(*p1);
                    hi = hi.max(*p1);
                }
                if vis(*i2) {
                    lo = lo.min(*p2);
                    hi = hi.max(*p2);
                }
            }

            DrawCmd::Overlay { .. }
            | DrawCmd::HLine { .. }
            | DrawCmd::VLine { .. }
            | DrawCmd::Text { .. }
            | DrawCmd::AxisText { .. }
            | DrawCmd::TimeText { .. }
            | DrawCmd::CrossTimeText { .. }
            | DrawCmd::StatusText { .. }
            | DrawCmd::StatusField { .. } => {}
        }
    }
    if lo > hi { (0.0, 0.0) } else { (lo, hi) }
}

impl StockApp {
    pub(crate) fn pane_view_range(&self, pane: usize, from: usize, to: usize) -> (f64, f64) {
        let (lo, hi) = pane_range(&self.panes[pane].cmds, from, to);
        self.panes[pane].chart_value_range(lo, hi)
    }

    pub(crate) fn chart_interact(
        &mut self,
        ui: &egui::Ui,
        plots: &[egui::Rect],
        bars: &[egui::Rect],
    ) {
        if self.ui_drag_active() {
            for pane in &mut self.panes {
                pane.view.pan = None;
            }
            return;
        }

        let Some(focused) = self.win_mgr.focused_pane() else {
            for pane in &mut self.panes {
                pane.view.pan = None;
            }
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
                            let offset = self.panes[pane].view.offset;
                            self.panes[pane].view.pan = Some((p.x, offset));
                        }
                    }
                }
            }
        }
        if down {
            if let Some(p) = latest {
                let bar_count = self.bars.len();
                for pane in &mut self.panes {
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
            for pane in &mut self.panes {
                pane.view.pan = None;
            }
        }
    }

    pub(crate) fn update_crosshair(&mut self, ui: &egui::Ui, plots: &[egui::Rect]) {
        if self.ui_drag_active() {
            for pane in &mut self.panes {
                pane.view.crosshair = None;
            }
            return;
        }
        let Some(pane) = self.win_mgr.focused_pane() else {
            for pane in &mut self.panes {
                pane.view.crosshair = None;
            }
            return;
        };
        let hover = ui.input(|i| i.pointer.hover_pos());
        let dragging = self.panes[pane].view.pan.is_some();
        let point = hover.or_else(|| {
            if dragging {
                ui.input(|i| i.pointer.latest_pos())
            } else {
                None
            }
        });
        let Some(p) = point else {
            for pane in &mut self.panes {
                pane.view.crosshair = None;
            }
            return;
        };

        if !dragging && !plots[pane].contains(p) {
            for pane in &mut self.panes {
                pane.view.crosshair = None;
            }
            return;
        }

        if !dragging && self.floating_at(p, ui.max_rect().max.y) {
            for pane in &mut self.panes {
                pane.view.crosshair = None;
            }
            return;
        }
        if self.panes[pane].view.visible == 0 {
            self.panes[pane].view.crosshair = None;
            return;
        }

        let plot = plots[pane];
        let from = self.panes[pane].view.offset;
        let to = (from + self.panes[pane].view.visible).min(self.bars.len());
        let advance = bar_advance(self.panes[pane].view.scale);
        let top = plot.min.y + CHART_PAD;
        let span_px = (plot.max.y - CHART_PAD - top)
            .max(LINE_W)
            .min((self.panes[pane].height as f32).max(LINE_W));
        let (lo, hi) = self.pane_view_range(pane, from, to);
        let span = (hi - lo).max(f64::EPSILON);

        let x = px_center(p.x.clamp(plot.min.x, plot.max.x - LINE_W));
        let y = px_center(p.y.clamp(top, top + span_px));
        let fpos = (from as f32 + (x - plot.min.x) / advance).max(0.0);
        let index = (fpos.floor() as usize).min(self.bars.len().saturating_sub(1));

        let t = ((y - top) / span_px).clamp(0.0, 1.0) as f64;
        let crosshair = Crosshair {
            value: hi - span * t,
            index,
            fpos: fpos as f64,
            pane,
        };
        for (i, slot) in self.panes.iter_mut().enumerate() {
            slot.view.crosshair = (i == pane).then_some(crosshair);
        }
    }

    pub(crate) fn ui_chart(&mut self, ui: &mut egui::Ui, pane: usize, area: egui::Rect) {
        let painter = ui.painter().clone();
        if self.panes[pane].dirty {
            self.sync_pane_palette();
        }
        self.ensure_pane_computed(pane);
        let layout = self.panes[pane].chart_layout();
        let (title, status, main, time) = chart_rows(area, layout);
        let show_axis = self.panes[pane].show_axis();
        let (plot, axis) = frame_rects_with_axis(main, show_axis);

        painter.rect_filled(area, egui::CornerRadius::ZERO, theme().work_bg);
        painter.rect_filled(plot, egui::CornerRadius::ZERO, theme().chart_bg);
        if show_axis {
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(axis.min.x, main.min.y - LINE_W),
                    egui::pos2(axis.max.x, main.max.y),
                ),
                egui::CornerRadius::ZERO,
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
            theme().chart_status_bg,
            theme().chart_status_label,
            theme().chart_status_divider,
        );

        self.panes[pane].height = main.height() as i32;
        self.clamp_view(pane, plot.width());
        let visible = self.panes[pane].view.visible;
        if visible == 0 {
            return;
        }

        let from = self.panes[pane].view.offset;
        let to = (from + visible).min(self.bars.len());
        let advance = bar_advance(self.panes[pane].view.scale);
        let bar_w = snap_scale(self.panes[pane].view.scale) as f32;
        let (lo, hi) = self.pane_view_range(pane, from, to);
        let top = plot.min.y + CHART_PAD;
        let span_px = (plot.max.y - CHART_PAD - top)
            .max(LINE_W)
            .min((self.panes[pane].height as f32).max(LINE_W));
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

            bottom: time.map_or(main.max.y - LINE_W, |t| t.min.y),

            origin: area.min,
        };

        let data = std::mem::take(&mut self.panes[pane].cmds);
        let crosshair = self.panes[pane].view.crosshair;
        let (axis_cmds, time_cmds, status_cmds, overlay_cmds) = {
            let ctx = ChartCtx {
                bars: &self.bars,
                cmds: &data,
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
                self.panes[pane].chart_axis(&ctx),
                self.panes[pane].chart_time(&ctx),
                self.panes[pane].chart_status(&ctx),
                self.panes[pane].chart_crosshair(&ctx),
            )
        };

        let mut run = Vec::new();
        self.replay_cmds(&painter, &geom, &axis_cmds, &mut run);
        self.replay_cmds(&painter, &geom, &data, &mut run);
        if show_axis {
            vline_hard(
                &painter,
                axis.min.x,
                axis.min.y,
                axis.max.y,
                theme().chart_line,
            );
        }
        self.replay_cmds(&painter, &geom, &time_cmds, &mut run);
        self.replay_cmds(&painter, &geom, &status_cmds, &mut run);
        self.replay_cmds(&painter, &geom, &overlay_cmds, &mut run);
        self.panes[pane].cmds = data;
    }

    fn replay_cmds(
        &mut self,
        painter: &egui::Painter,
        geom: &ChartGeom,
        cmds: &[DrawCmd],
        run: &mut Vec<(f32, f32)>,
    ) {
        let map = &geom.map;

        let mut sx = geom.status.map_or(0.0, |r| r.min.x);
        let mut first = true;
        for cmd in cmds {
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
                    let rect = egui::Rect::from_min_max(
                        egui::pos2(left, a.min(b)),
                        egui::pos2(left + map.bar_w, a.max(b)),
                    );
                    painter.rect_filled(rect, egui::CornerRadius::ZERO, color32(*color));

                    if *hollow && map.bar_w >= 3.0 && rect.height() >= 3.0 {
                        let inner = egui::Rect::from_min_max(
                            egui::pos2(rect.min.x + LINE_W, rect.min.y + LINE_W),
                            egui::pos2(rect.max.x - LINE_W, rect.max.y - LINE_W),
                        );
                        painter.rect_filled(inner, egui::CornerRadius::ZERO, theme().chart_bg);
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
                    debug_assert!(*w >= 1, "折线宽度至少 1px");
                    run.clear();
                    for (i, p) in pts {
                        if map.visible(*i) {
                            run.push((map.line_x(*i), map.y(*p)));
                        }
                    }

                    if run.len() > 1 {
                        polyline_hard_width(painter, &run, color32(*color), *w);
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
                    debug_assert!(*w >= 1, "直线宽度至少 1px");
                    painter.line_segment(
                        [
                            egui::pos2(map.line_x(*i1), map.y(*p1)),
                            egui::pos2(map.line_x(*i2), map.y(*p2)),
                        ],
                        egui::Stroke::new(*w as f32, color32(*color)),
                    );
                }
                DrawCmd::HLine { p, color, w } => {
                    debug_assert!(*w >= 1, "参考线宽度至少 1px");

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
                    debug_assert!(*w >= 1, "竖线宽度至少 1px");
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
                    let pos = egui::pos2(map.line_x(*i), map.y(*p));
                    self.text(
                        painter,
                        pos,
                        egui::Align2::CENTER_CENTER,
                        s,
                        color32(*color),
                    );
                }
                DrawCmd::AxisText { p, s, fg, bg } => {
                    let y = map.y(*p);
                    if *bg == 0 {
                        if !geom.axis.is_positive() {
                            continue;
                        }
                        self.text(
                            painter,
                            egui::pos2(Self::axis_tick_x(geom.axis), y),
                            egui::Align2::RIGHT_CENTER,
                            s,
                            color32(*fg),
                        );
                    } else {
                        if geom.plot.height() < CROSS_LABEL_H {
                            continue;
                        }
                        let w = self.cross_label_w(s);
                        let x0 = if geom.axis.is_positive() {
                            Self::cross_axis_label_x(geom.axis, w)
                        } else {
                            (geom.plot.max.x - w - LINE_W).max(geom.plot.min.x)
                        };

                        let y0 = floating_label_top(
                            y,
                            CROSS_LABEL_ROWS,
                            geom.plot.min.y - LINE_W,
                            geom.bottom,
                            geom.origin.y,
                        );
                        self.axis_label_box(
                            painter,
                            s,
                            egui::pos2(x0, y0),
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
                        let w = self.text_width(s);
                        if cx - w / 2.0 < geom.plot.min.x || cx + w / 2.0 > geom.plot.max.x - LINE_W
                        {
                            continue;
                        }
                        self.text(
                            painter,
                            egui::pos2(cx, band.center().y),
                            egui::Align2::CENTER_CENTER,
                            s,
                            color32(*fg),
                        );
                    } else {
                        let w = self.cross_label_w(s);
                        let x0 =
                            Self::cross_time_label_x(geom.plot, map.line_x(*i), w, geom.origin.x);

                        self.label_box(
                            painter,
                            s,
                            egui::pos2(x0, geom.bottom - PITCH * CROSS_LABEL_ROWS as f32),
                            w,
                            color32(*bg),
                            theme().cross_label_text,
                        );
                    }
                }
                DrawCmd::CrossTimeText { i, s, bg } => {
                    let w = self.cross_label_w(s);
                    let x0 = Self::cross_time_label_x(geom.plot, map.line_at(*i), w, geom.origin.x);
                    self.label_box(
                        painter,
                        s,
                        egui::pos2(x0, geom.bottom - PITCH * CROSS_LABEL_ROWS as f32),
                        w,
                        color32(*bg),
                        theme().cross_label_text,
                    );
                }
                DrawCmd::StatusText { s, color } => {
                    let Some(band) = geom.status else {
                        continue;
                    };
                    let w = snap_cells_up(self.text_width(s) + 2.0 * TEXT_PAD_X);
                    if sx + w > band.max.x {
                        break;
                    }
                    self.status_divider(painter, band, sx, first);
                    self.text(
                        painter,
                        egui::pos2(text_left_x(sx, geom.origin.x), band.center().y),
                        egui::Align2::LEFT_CENTER,
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
                    let label_w = self.text_width(label);
                    let value_w = self.text_width(value);
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
                    self.text(
                        &clipped,
                        egui::pos2(text_left_x(sx, geom.origin.x), band.center().y),
                        egui::Align2::LEFT_CENTER,
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
                    self.text(
                        &clipped,
                        egui::pos2(
                            text_right_x(sx + w - LINE_W, geom.origin.x),
                            band.center().y,
                        ),
                        egui::Align2::RIGHT_CENTER,
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

    fn status_divider(&self, painter: &egui::Painter, band: egui::Rect, x: f32, first: bool) {
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
        &mut self,
        painter: &egui::Painter,
        text: &str,
        pos: egui::Pos2,
        w: f32,
        bg: egui::Color32,
        fg: egui::Color32,
    ) {
        let rect = egui::Rect::from_min_size(pos, egui::vec2(w, CROSS_LABEL_H));
        tag_box(painter, rect, bg, theme().cross_label_border);
        self.text(
            painter,
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            fg,
        );
    }

    fn axis_label_box(
        &mut self,
        painter: &egui::Painter,
        text: &str,
        pos: egui::Pos2,
        w: f32,
        bg: egui::Color32,
        fg: egui::Color32,
    ) {
        let rect = egui::Rect::from_min_size(pos, egui::vec2(w, CROSS_LABEL_H));
        tag_box(painter, rect, bg, theme().cross_label_border);
        self.text(
            painter,
            egui::pos2(rect.min.x + PITCH, rect.center().y),
            egui::Align2::LEFT_CENTER,
            text,
            fg,
        );
    }

    pub(crate) const PANE_TABS: [&'static str; 3] = ["圆桌", "设置", "切换"];

    #[allow(dead_code)]
    pub(crate) fn pane_button_rects(title: egui::Rect, widths: &[f32]) -> Vec<egui::Rect> {
        menu_item_rects(title, widths, MenuAlign::Left)
    }

    pub(crate) fn ui_pane_title(
        &mut self,
        ui: &mut egui::Ui,
        pane: usize,
        title: egui::Rect,
    ) -> Vec<egui::Rect> {
        let open = if self.pane_menu == Some(pane) {
            Some(self.pane_menu_tab)
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
            self.pane_menu = Some(pane);
            self.pane_menu_tab = tab;
        } else if self.pane_menu == Some(pane) {
            self.pane_menu = None;
        }

        if self.pane_menu == Some(pane) {
            self.pane_menu_anchor = out.rects[self.pane_menu_tab];
            self.pane_menu_bar = title;
        }
        out.rects
    }

    pub(crate) fn take_wheel_notches(
        &mut self,
        ui: &egui::Ui,
        hover: Option<egui::Pos2>,
        pane: usize,
    ) -> i32 {
        let bottom = ui.max_rect().max.y;
        if hover.is_none()
            || self.ui_drag_active()
            || hover.is_some_and(|p| self.floating_at(p, bottom))
        {
            self.panes[pane].view.zoom_acc = 0.0;
            return 0;
        }
        let mut acc = self.panes[pane].view.zoom_acc;
        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::MouseWheel { unit, delta, .. } = event {
                    acc += match unit {
                        egui::MouseWheelUnit::Line => delta.y,
                        egui::MouseWheelUnit::Point => delta.y / WHEEL_POINTS_PER_NOTCH,
                        egui::MouseWheelUnit::Page => delta.y,
                    };
                }
            }
        });
        let whole = acc.trunc();
        self.panes[pane].view.zoom_acc = acc - whole;
        whole as i32
    }

    pub(crate) fn zoom_around(&mut self, pane: usize, x_rel: f32, notches: i32) {
        let old = snap_scale(self.panes[pane].view.scale);
        let new = scale_after(old, notches);
        if new == old {
            return;
        }
        let mouse_bar = self.panes[pane].view.offset as f32 + x_rel / bar_advance(old);
        self.panes[pane].view.scale = new;
        self.panes[pane].view.offset =
            (mouse_bar - x_rel / bar_advance(new)).round().max(0.0) as usize;
    }

    pub(crate) fn clamp_view(&mut self, pane: usize, plot_w: f32) {
        let bar_count = self.bars.len();
        let view = &mut self.panes[pane].view;
        view.visible = visible_bars(plot_w, view.scale).min(bar_count);
        view.offset = view.offset.min(bar_count.saturating_sub(view.visible));
    }

    pub(crate) fn time_step(&mut self, advance: f32) -> usize {
        let label_w = self.text_width("00-00").max(self.text_width("00-00-00"));
        let need = (label_w + GRID).max(TIME_LABEL_GAP_MIN);
        ((need / advance).ceil() as usize).max(1)
    }

    pub(crate) fn cross_label_w(&mut self, _text: &str) -> f32 {
        cells(AXIS_COLS - 2)
    }

    pub(crate) fn axis_tick_x(axis: egui::Rect) -> f32 {
        axis.max.x - LINE_W - GRID
    }

    pub(crate) fn cross_axis_label_x(axis: egui::Rect, w: f32) -> f32 {
        let right = Self::axis_tick_x(axis) - w;
        clamp_soft(right, axis.min.x, right)
    }

    pub(crate) fn cross_time_label_x(plot: egui::Rect, cx: f32, w: f32, _origin: f32) -> f32 {
        clamp_soft(
            (cx - w / 2.0).round(),
            plot.min.x - LINE_W,
            plot.max.x - w - GRID,
        )
    }
}
