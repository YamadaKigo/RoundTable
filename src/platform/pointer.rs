use crate::engine::kit;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

use crate::ui::icon::CursorKind;

pub(crate) const TRAJECTORY_CAPACITY: usize = 8192;

#[derive(Clone, Copy, Debug)]
pub(crate) struct PointerSample {
    pub(crate) time: f64,
    pub(crate) position: kit::Pos2,
    #[cfg(test)]
    pub(crate) delta: kit::Vec2,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PointerFeel {
    pub(crate) base_speed: f32,

    pub(crate) acceleration: f32,

    pub(crate) accel_start: f32,

    pub(crate) accel_full: f32,
}

impl Default for PointerFeel {
    fn default() -> Self {
        Self {
            base_speed: 1.0,
            acceleration: 0.0,
            accel_start: 300.0,
            accel_full: 3000.0,
        }
    }
}

impl PointerFeel {
    pub(crate) fn from_system(speed: i32, mouse: [i32; 3]) -> Self {
        const SPEED: [f32; 20] = [
            0.03125, 0.0625, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.0, 1.25, 1.5, 1.75,
            2.0, 2.25, 2.5, 2.75, 3.0, 3.25, 3.5,
        ];
        let speed_index = (speed - 1).clamp(0, 19) as usize;
        let start = mouse[0].max(1) as f32 * 120.0;
        let full = mouse[1].max(mouse[0].max(1) + 1) as f32 * 120.0;
        Self {
            base_speed: SPEED[speed_index],
            acceleration: if mouse[2] != 0 { 1.0 } else { 0.0 },
            accel_start: start,
            accel_full: full.max(start + 1.0),
        }
    }
}

pub(crate) struct PointerMotion {
    position: kit::Pos2,

    velocity: kit::Vec2,

    acceleration: kit::Vec2,
    last_time: f64,
    initialized: bool,
    feel: PointerFeel,
    trajectory: VecDeque<PointerSample>,
    record_trail: bool,
    raw_deltas: Vec<kit::Vec2>,
    native_cursor: Option<kit::Pos2>,
}

pub(crate) struct PointerManager {
    pub(crate) motion: PointerMotion,
    pub(crate) system: bool,
    pub(crate) show_trail: bool,
    pub(crate) hint: CursorKind,
    pub(crate) forced_hint: Option<CursorKind>,
    pub(crate) text_hover: bool,
    pub(crate) text_selecting: bool,
    pub(crate) wheel_until: f64,
    pub(crate) live: Option<kit::Pos2>,
    pub(crate) motion_live: Option<kit::Pos2>,
    pub(crate) pinned: Option<kit::Pos2>,
}

impl PointerManager {
    pub(crate) fn new(feel: PointerFeel) -> Self {
        Self {
            motion: PointerMotion::new(feel),
            system: false,
            show_trail: false,
            hint: CursorKind::Normal,
            forced_hint: None,
            text_hover: false,
            text_selecting: false,
            wheel_until: 0.0,
            live: None,
            motion_live: None,
            pinned: None,
        }
    }

    pub(crate) fn begin_frame(&mut self, ctx: &kit::Context) {
        self.motion.set_record_trail(self.show_trail);
        let primary_down = ctx.input(|i| i.pointer.primary_down());
        if !primary_down {
            self.pinned = None;
            self.text_selecting = false;
        }
        self.hint = CursorKind::Normal;
        self.forced_hint = None;
        self.text_hover = false;
    }

    pub(crate) fn wheel_active(&self, ctx: &kit::Context) -> bool {
        self.wheel_until > ctx.input(|i| i.time)
    }

    pub(crate) fn force_hint(&mut self, kind: CursorKind, ui: &kit::Ui) {
        self.forced_hint = Some(kind);
        ui.ctx().set_cursor_icon(kit::CursorIcon::Default);
    }

    pub(crate) fn pinned_if_down(&self, ctx: &kit::Context) -> Option<kit::Pos2> {
        let pin = self.pinned?;
        ctx.input(|i| i.pointer.primary_down()).then_some(pin)
    }

    pub(crate) fn lock_events(&mut self, events: &mut [kit::Event]) {
        let released = events.iter().any(|event| {
            matches!(
                event,
                kit::Event::PointerButton {
                    button: kit::PointerButton::Primary,
                    pressed: false,
                    ..
                } | kit::Event::WindowFocused(false)
            )
        });
        let Some(pin) = self.pinned else {
            return;
        };
        for event in events {
            match event {
                kit::Event::PointerMoved(pos) => *pos = pin,
                kit::Event::PointerButton { pos, .. } => *pos = pin,
                _ => {}
            }
        }
        if released {
            self.pinned = None;
        }
    }
}

impl Deref for PointerManager {
    type Target = PointerMotion;

    fn deref(&self) -> &Self::Target {
        &self.motion
    }
}

impl DerefMut for PointerManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.motion
    }
}

impl Default for PointerMotion {
    fn default() -> Self {
        Self::new(PointerFeel::default())
    }
}

impl PointerMotion {
    pub(crate) fn new(feel: PointerFeel) -> Self {
        Self {
            position: kit::Pos2::ZERO,
            velocity: kit::Vec2::ZERO,
            acceleration: kit::Vec2::ZERO,
            last_time: 0.0,
            initialized: false,
            feel,
            trajectory: VecDeque::new(),
            record_trail: true,
            raw_deltas: Vec::new(),
            native_cursor: None,
        }
    }

    pub(crate) fn position(&self) -> kit::Pos2 {
        self.position
    }

    pub(crate) fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub(crate) fn velocity(&self) -> kit::Vec2 {
        self.velocity
    }

    pub(crate) fn acceleration(&self) -> kit::Vec2 {
        self.acceleration
    }

    pub(crate) fn trajectory(&self) -> &VecDeque<PointerSample> {
        &self.trajectory
    }

    pub(crate) fn set_record_trail(&mut self, record: bool) {
        if self.record_trail == record {
            return;
        }
        self.record_trail = record;
        if !record {
            self.trajectory.clear();
        }
    }

    pub(crate) fn reset(&mut self, position: kit::Pos2, time: f64) {
        self.position = position;
        self.velocity = kit::Vec2::ZERO;
        self.acceleration = kit::Vec2::ZERO;
        self.last_time = time;
        self.initialized = true;
        self.trajectory.clear();
        self.native_cursor = None;
    }

    pub(crate) fn pin(&mut self, position: kit::Pos2, time: f64) {
        #[cfg(test)]
        let previous = self.position;
        self.position = position;
        self.velocity = kit::Vec2::ZERO;
        self.acceleration = kit::Vec2::ZERO;
        self.last_time = time;
        self.initialized = true;
        while self
            .trajectory
            .back()
            .is_some_and(|sample| sample.time >= time)
        {
            self.trajectory.pop_back();
        }
        if self.record_trail {
            self.trajectory.push_back(PointerSample {
                time,
                position,
                #[cfg(test)]
                delta: position - previous,
            });
            if self.trajectory.len() > TRAJECTORY_CAPACITY {
                self.trajectory.pop_front();
            }
        }
    }

    pub(crate) fn sync(&mut self, position: kit::Pos2, time: f64) {
        if !self.initialized || position != self.position {
            self.reset(position, time);
        }
    }

    pub(crate) fn follow(&mut self, position: kit::Pos2, time: f64) {
        if !self.initialized {
            self.reset(position, time);
            return;
        }
        if position == self.position {
            return;
        }
        let delta = position - self.position;
        let dt = (time - self.last_time).clamp(1.0 / 1000.0, 0.05);
        let previous_velocity = self.velocity;
        self.position = position;
        self.velocity = delta / dt as f32;
        self.acceleration = (self.velocity - previous_velocity) / dt as f32;
        self.last_time = time;
        if self.record_trail {
            self.trajectory.push_back(PointerSample {
                time,
                position,
                #[cfg(test)]
                delta,
            });
            if self.trajectory.len() > TRAJECTORY_CAPACITY {
                self.trajectory.pop_front();
            }
        }
    }

    pub(crate) fn follow_events(&mut self, ctx: &kit::Context, position: kit::Pos2, time: f64) {
        self.raw_deltas.clear();
        ctx.input(|input| {
            for event in &input.events {
                if let kit::Event::MouseMoved(delta) = event
                    && delta.is_finite()
                    && *delta != kit::Vec2::ZERO
                {
                    self.raw_deltas.push(*delta);
                }
            }
        });
        let deltas = std::mem::take(&mut self.raw_deltas);
        self.follow_samples(position, time, &deltas);
        self.raw_deltas = deltas;
    }

    pub(crate) fn advance(
        &mut self,
        deltas: &[kit::Vec2],
        time: f64,
        bounds: kit::Rect,
    ) -> kit::Pos2 {
        self.advance_with_trail(deltas, time, bounds, true)
    }

    pub(crate) fn advance_locked(
        &mut self,
        deltas: &[kit::Vec2],
        time: f64,
        bounds: kit::Rect,
    ) -> kit::Pos2 {
        if !self.initialized {
            self.reset(self.position, time);
        }
        let dt = (time - self.last_time).clamp(1.0 / 1000.0, 0.05) as f32;
        let before = self.position;
        let mut next = before;
        for delta in deltas {
            if delta.is_finite() {
                next += *delta;
            }
        }
        self.position = kit::pos2(
            next.x.clamp(bounds.min.x, bounds.max.x),
            next.y.clamp(bounds.min.y, bounds.max.y),
        );
        let velocity = (self.position - before) / dt;
        self.acceleration = (velocity - self.velocity) / dt;
        self.velocity = velocity;
        self.last_time = time;
        self.position
    }

    pub(crate) fn advance_events(
        &mut self,
        ctx: &kit::Context,
        time: f64,
        bounds: kit::Rect,
        locked: bool,
    ) -> (bool, kit::Pos2) {
        self.raw_deltas.clear();
        ctx.input(|input| {
            for event in &input.events {
                if let kit::Event::MouseMoved(delta) = event {
                    self.raw_deltas.push(*delta);
                }
            }
        });
        if self.raw_deltas.is_empty() {
            self.velocity = kit::Vec2::ZERO;
            self.acceleration = kit::Vec2::ZERO;
            self.last_time = time;
            return (false, self.position);
        }
        let deltas = std::mem::take(&mut self.raw_deltas);
        let at = if locked {
            self.advance_locked(&deltas, time, bounds)
        } else {
            self.advance(&deltas, time, bounds)
        };
        self.raw_deltas = deltas;
        (true, at)
    }

    pub(crate) fn native_cursor_needs_move(&self, position: kit::Pos2) -> bool {
        let target = kit::pos2(position.x.round(), position.y.round());
        self.native_cursor != Some(target)
    }

    pub(crate) fn mark_native_cursor(&mut self, position: kit::Pos2) {
        self.native_cursor = Some(kit::pos2(position.x.round(), position.y.round()));
    }

    pub(crate) fn take_native_motion(&mut self, position: kit::Pos2) -> kit::Vec2 {
        let rounded = kit::pos2(position.x.round(), position.y.round());
        let delta = self
            .native_cursor
            .map_or(kit::Vec2::ZERO, |previous| rounded - previous);
        self.native_cursor = Some(rounded);
        delta
    }

    fn advance_with_trail(
        &mut self,
        deltas: &[kit::Vec2],
        time: f64,
        bounds: kit::Rect,
        record: bool,
    ) -> kit::Pos2 {
        if !self.initialized {
            self.reset(self.position, time);
        }
        let mut raw = kit::Vec2::ZERO;
        for d in deltas {
            raw += *d;
        }
        if raw == kit::Vec2::ZERO || !raw.is_finite() {
            let dt = (time - self.last_time).clamp(1.0 / 1000.0, 0.05) as f32;
            let previous = self.velocity;
            self.velocity = kit::Vec2::ZERO;
            self.acceleration = -previous / dt;
            self.last_time = time;
            return self.position;
        }

        let dt = (time - self.last_time).clamp(1.0 / 1000.0, 0.05);
        let dt_f = dt as f32;
        let n = deltas.len().max(1);
        let event_dt = (dt_f / n as f32).max(0.000_25);
        let base = self.feel.base_speed.max(0.0);
        let span = (self.feel.accel_full - self.feel.accel_start).max(1.0);

        let before = self.position;
        let mut next = before;
        for d in deltas {
            if !d.is_finite() {
                continue;
            }

            let speed = d.length() / event_dt;
            let t = ((speed - self.feel.accel_start) / span).clamp(0.0, 1.0);
            let curve = t * t * (3.0 - 2.0 * t);
            let gain = base * (1.0 + self.feel.acceleration.max(0.0) * curve);
            let step = *d * gain;
            next += step;
        }
        let clamped = kit::pos2(
            next.x.clamp(bounds.min.x, bounds.max.x),
            next.y.clamp(bounds.min.y, bounds.max.y),
        );
        self.position = clamped;

        let new_velocity = (clamped - before) / dt as f32;
        self.acceleration = (new_velocity - self.velocity) / dt_f;
        self.velocity = new_velocity;

        let total = raw.length();
        let mut sample_walked = 0.0f32;
        for (i, d) in deltas.iter().enumerate() {
            sample_walked += d.length();
            let f = if total > 0.0 {
                sample_walked / total
            } else {
                1.0
            };
            let at = before + (clamped - before) * f.clamp(0.0, 1.0);
            let back = (n - 1 - i) as f64 / n as f64;
            if record {
                self.trajectory.push_back(PointerSample {
                    time: time - dt * back,
                    position: at,
                    #[cfg(test)]
                    delta: *d,
                });
                if self.trajectory.len() > TRAJECTORY_CAPACITY {
                    self.trajectory.pop_front();
                }
            }
        }
        self.last_time = time;
        clamped
    }

    fn follow_samples(&mut self, position: kit::Pos2, time: f64, deltas: &[kit::Vec2]) {
        if !self.initialized {
            self.reset(position, time);
            return;
        }
        if position == self.position {
            return;
        }
        let delta = position - self.position;
        let total = deltas.iter().map(|sample| sample.length()).sum::<f32>();
        if total <= 0.0 || !total.is_finite() {
            self.follow(position, time);
            return;
        }

        let before = self.position;
        let start_time = self.last_time;
        let dt = (time - start_time).clamp(1.0 / 1000.0, 0.05);
        let previous_velocity = self.velocity;
        self.position = position;
        self.velocity = delta / dt as f32;
        self.acceleration = (self.velocity - previous_velocity) / dt as f32;
        self.last_time = time;

        if self.record_trail {
            let mut walked = 0.0f32;
            for sample in deltas {
                walked += sample.length();
                let progress = (walked / total).clamp(0.0, 1.0);
                self.trajectory.push_back(PointerSample {
                    time: start_time + dt * f64::from(progress),
                    position: before + delta * progress,
                    #[cfg(test)]
                    delta: *sample,
                });
                if self.trajectory.len() > TRAJECTORY_CAPACITY {
                    self.trajectory.pop_front();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> kit::Rect {
        kit::Rect::from_min_max(kit::pos2(0.0, 0.0), kit::pos2(1920.0, 1080.0))
    }

    #[test]
    fn default_pointer_feel_keeps_system_style_one_to_one_motion() {
        let feel = PointerFeel::default();
        assert_eq!(feel.base_speed, 1.0);
        assert_eq!(feel.acceleration, 0.0);
    }

    #[test]
    fn pointer_manager_starts_with_the_builtin_pointer() {
        assert!(!PointerManager::new(PointerFeel::default()).system);
    }

    #[test]
    fn every_raw_delta_is_recorded_and_fast_motion_gets_acceleration() {
        let mut slow = PointerMotion::new(PointerFeel {
            base_speed: 1.0,
            acceleration: 0.5,
            accel_start: 100.0,
            accel_full: 3000.0,
        });
        slow.reset(kit::pos2(100.0, 100.0), 0.0);
        let at = slow.advance(&[kit::vec2(1.0, 0.0)], 0.05, bounds());
        assert_eq!(at, kit::pos2(101.0, 100.0), "慢速移动不该被加速曲线放大");
        assert_eq!(slow.trajectory().len(), 1, "原始移动样本没有进轨迹记录");
        assert_eq!(slow.trajectory()[0].delta, kit::vec2(1.0, 0.0));

        let mut fast = PointerMotion::new(PointerFeel {
            base_speed: 1.0,
            acceleration: 0.5,
            accel_start: 100.0,
            accel_full: 3000.0,
        });
        fast.reset(kit::pos2(100.0, 100.0), 0.0);
        let at = fast.advance(&[kit::vec2(200.0, 0.0)], 0.016, bounds());
        assert!(
            at.x - 100.0 > 200.0,
            "High速移动没有获得加速度增益：{}",
            at.x - 100.0
        );
        assert!(fast.velocity().x > 0.0);
        assert!(fast.acceleration().x != 0.0);
        assert_eq!(fast.trajectory().len(), 1);
    }

    #[test]
    fn speed_scales_the_linear_part_without_moving_the_acceleration_curve() {
        let feel = PointerFeel {
            acceleration: 0.5,
            accel_start: 500.0,
            accel_full: 2500.0,
            ..PointerFeel::default()
        };
        let mut slow = PointerMotion::new(PointerFeel {
            base_speed: 0.5,
            ..feel
        });
        let mut fast = PointerMotion::new(PointerFeel {
            base_speed: 2.0,
            ..feel
        });
        slow.reset(kit::pos2(100.0, 100.0), 0.0);
        fast.reset(kit::pos2(100.0, 100.0), 0.0);

        let a = slow.advance(&[kit::vec2(2.0, 0.0)], 0.02, bounds());
        let b = fast.advance(&[kit::vec2(2.0, 0.0)], 0.02, bounds());
        assert_eq!(a.x - 100.0, 1.0);
        assert_eq!(b.x - 100.0, 4.0);

        slow.reset(kit::pos2(100.0, 100.0), 0.0);
        fast.reset(kit::pos2(100.0, 100.0), 0.0);
        let a = slow.advance(&[kit::vec2(100.0, 0.0)], 0.02, bounds());
        let b = fast.advance(&[kit::vec2(100.0, 0.0)], 0.02, bounds());
        let ga = (a.x - 100.0) / 100.0;
        let gb = (b.x - 100.0) / 100.0;
        assert!(
            ga > 0.5 * 1.2 && gb > 2.0 * 1.2,
            "High速段没有进入加速区：{ga} / {gb}"
        );
        assert!((gb / ga - 4.0).abs() < 0.05, "速度档位改变了加速曲线");
    }

    #[test]
    fn a_linear_pointer_is_exact_and_keeps_subpixel_motion() {
        let mut p = PointerMotion::new(PointerFeel {
            base_speed: 1.0,
            acceleration: 0.0,
            ..PointerFeel::default()
        });
        p.reset(kit::pos2(100.0, 100.0), 0.0);
        let mut t = 0.0;
        for _ in 0..10 {
            t += 0.001;
            let _ = p.advance(&[kit::vec2(0.25, 0.0)], t, bounds());
        }
        assert!(
            (p.position().x - 102.5).abs() < 1e-6,
            "小位移被取整丢掉了：{}",
            p.position().x
        );

        p.reset(kit::pos2(100.0, 100.0), 0.0);
        let at = p.advance(&[kit::vec2(37.0, -12.0)], 0.016, bounds());
        assert_eq!(
            at,
            kit::pos2(137.0, 88.0),
            "速度 1x、加速度 0 时不该改变原始位移"
        );
    }

    #[test]
    fn absolute_follow_tracks_the_system_pointer_without_reapplying_ballistics() {
        let mut pointer = PointerMotion::new(PointerFeel {
            base_speed: 0.5,
            acceleration: 1.0,
            accel_start: 1.0,
            accel_full: 2.0,
        });
        pointer.reset(kit::pos2(100.0, 100.0), 0.0);
        pointer.follow(kit::pos2(137.0, 88.0), 0.016);
        assert_eq!(pointer.position(), kit::pos2(137.0, 88.0));
        assert_eq!(pointer.trajectory().len(), 1);
        assert_eq!(
            pointer.trajectory().back().map(|sample| sample.delta),
            Some(kit::vec2(37.0, -12.0))
        );
    }

    #[test]
    fn raw_motion_samples_keep_the_intermediate_pointer_path() {
        let mut pointer = PointerMotion::new(PointerFeel::default());
        pointer.reset(kit::pos2(0.0, 0.0), 0.0);
        pointer.follow_samples(
            kit::pos2(6.0, 3.0),
            0.016,
            &[
                kit::vec2(2.0, 1.0),
                kit::vec2(2.0, 1.0),
                kit::vec2(2.0, 1.0),
            ],
        );
        let positions: Vec<kit::Pos2> = pointer
            .trajectory()
            .iter()
            .map(|sample| sample.position)
            .collect();
        let expected = [
            kit::pos2(2.0, 1.0),
            kit::pos2(4.0, 2.0),
            kit::pos2(6.0, 3.0),
        ];
        for (actual, wanted) in positions.into_iter().zip(expected) {
            assert!((actual - wanted).length() < 1e-6);
        }
        assert_eq!(pointer.position(), kit::pos2(6.0, 3.0));
    }

    #[test]
    fn acceleration_reacts_to_the_current_sample_without_a_tail() {
        let mut p = PointerMotion::new(PointerFeel {
            base_speed: 1.0,
            acceleration: 0.2,
            accel_start: 100.0,
            accel_full: 1000.0,
        });
        p.reset(kit::pos2(100.0, 100.0), 0.0);
        let fast = p.advance(&[kit::vec2(100.0, 0.0)], 0.016, bounds());
        assert!(fast.x > 200.0, "快速样本没有立刻获得加速");

        let before = p.position();
        let still = p.advance(&[], 0.017, bounds());
        assert_eq!(still, before, "松手后指针还在自己追自己");
    }

    #[test]
    fn multiple_events_are_all_kept_even_when_consumed_in_one_frame() {
        let mut p = PointerMotion::new(PointerFeel {
            base_speed: 1.0,
            acceleration: 0.0,
            ..PointerFeel::default()
        });
        p.reset(kit::pos2(10.0, 10.0), 0.0);
        let deltas = [
            kit::vec2(2.0, 1.0),
            kit::vec2(3.0, -1.0),
            kit::vec2(4.0, 2.0),
        ];
        let at = p.advance(&deltas, 0.016, bounds());
        assert_eq!(p.trajectory().len(), deltas.len());
        assert_eq!(at, kit::pos2(19.0, 12.0));
        let times: Vec<f64> = p.trajectory().iter().map(|s| s.time).collect();
        assert!(times.windows(2).all(|w| w[0] < w[1]), "轨迹时间没有铺Open");
        assert_eq!(times.last().copied(), Some(0.016));
    }

    #[test]
    fn system_mouse_settings_map_to_speed_and_ballistics() {
        let neutral = PointerFeel::from_system(10, [6, 10, 0]);
        let faster = PointerFeel::from_system(12, [6, 10, 0]);
        let accelerated = PointerFeel::from_system(12, [4, 8, 1]);
        assert_eq!(neutral.base_speed, 1.0);
        assert_eq!(neutral.acceleration, 0.0);
        assert!(faster.base_speed > neutral.base_speed);
        assert_eq!(accelerated.acceleration, 1.0);
        assert_eq!(accelerated.accel_start, 480.0);
        assert_eq!(accelerated.accel_full, 960.0);
    }

    #[test]
    fn a_locked_pointer_swallows_absolute_motion_but_keeps_raw_delta() {
        let mut pointer = PointerManager::new(PointerFeel::default());
        pointer.pinned = Some(kit::pos2(40.0, 50.0));
        let mut events = vec![
            kit::Event::PointerMoved(kit::pos2(400.0, 300.0)),
            kit::Event::MouseMoved(kit::vec2(9.0, -3.0)),
        ];
        pointer.lock_events(&mut events);
        assert_eq!(events[0], kit::Event::PointerMoved(kit::pos2(40.0, 50.0)));
        assert_eq!(events[1], kit::Event::MouseMoved(kit::vec2(9.0, -3.0)));

        let mut release = vec![kit::Event::PointerButton {
            pos: kit::pos2(900.0, 700.0),
            button: kit::PointerButton::Primary,
            pressed: false,
            modifiers: kit::Modifiers::NONE,
        }];
        pointer.lock_events(&mut release);
        assert_eq!(
            release[0],
            kit::Event::PointerButton {
                pos: kit::pos2(40.0, 50.0),
                button: kit::PointerButton::Primary,
                pressed: false,
                modifiers: kit::Modifiers::NONE,
            }
        );
        assert!(pointer.pinned.is_none());
    }

    #[test]
    fn locked_motion_does_not_leave_an_overshoot_trail() {
        let mut pointer = PointerMotion::new(PointerFeel {
            acceleration: 0.0,
            ..PointerFeel::default()
        });
        pointer.reset(kit::pos2(100.0, 100.0), 0.0);
        pointer.advance_locked(&[kit::vec2(500.0, 0.0)], 0.016, bounds());
        assert!(pointer.trajectory().is_empty());
        pointer.pin(kit::pos2(160.0, 100.0), 0.016);
        assert_eq!(pointer.trajectory().len(), 1);
        assert_eq!(
            pointer.trajectory().back().map(|sample| sample.position),
            Some(kit::pos2(160.0, 100.0))
        );
    }

    #[test]
    fn locked_motion_is_exact_and_native_displacement_is_consumed_once() {
        let mut pointer = PointerMotion::new(PointerFeel {
            base_speed: 1.0,
            acceleration: 1.0,
            accel_start: 1.0,
            accel_full: 2.0,
        });
        pointer.reset(kit::pos2(100.0, 100.0), 0.0);
        pointer.mark_native_cursor(kit::pos2(100.0, 100.0));
        let at = pointer.advance_locked(&[kit::vec2(37.0, -12.0)], 0.016, bounds());
        assert_eq!(at, kit::pos2(137.0, 88.0));
        assert!(pointer.trajectory().is_empty());

        let delta = pointer.take_native_motion(kit::pos2(105.0, 100.0));
        assert_eq!(delta, kit::vec2(5.0, 0.0));
        assert_eq!(
            pointer.take_native_motion(kit::pos2(105.0, 100.0)),
            kit::Vec2::ZERO,
            "位置没有再次变化时不能重复累计旧位移"
        );
    }
}
