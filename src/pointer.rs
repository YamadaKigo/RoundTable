use eframe::egui;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

use crate::ui::icon::CursorKind;

pub(crate) const TRAJECTORY_CAPACITY: usize = 8192;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub(crate) struct PointerSample {
    pub(crate) time: f64,
    pub(crate) position: egui::Pos2,
    pub(crate) delta: egui::Vec2,
    pub(crate) velocity: egui::Vec2,
    pub(crate) acceleration: egui::Vec2,
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
            acceleration: 0.3,
            accel_start: 250.0,
            accel_full: 3000.0,
        }
    }
}

pub(crate) struct PointerMotion {
    position: egui::Pos2,

    velocity: egui::Vec2,

    acceleration: egui::Vec2,
    last_time: f64,
    initialized: bool,
    feel: PointerFeel,
    trajectory: VecDeque<PointerSample>,
}

pub(crate) struct PointerManager {
    pub(crate) motion: PointerMotion,
    pub(crate) system: bool,
    pub(crate) show_trail: bool,
    pub(crate) hint: CursorKind,
    pub(crate) forced_hint: Option<CursorKind>,
    pub(crate) text_hover: bool,
    pub(crate) wheel_until: f64,
    pub(crate) live: Option<egui::Pos2>,
    pub(crate) pinned: Option<egui::Pos2>,
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
            wheel_until: 0.0,
            live: None,
            pinned: None,
        }
    }

    pub(crate) fn begin_frame(&mut self) {
        self.pinned = None;
        self.hint = CursorKind::Normal;
        self.forced_hint = None;
        self.text_hover = false;
    }

    pub(crate) fn wheel_active(&self, ctx: &egui::Context) -> bool {
        self.wheel_until > ctx.input(|i| i.time)
    }

    pub(crate) fn force_hint(&mut self, kind: CursorKind, ui: &egui::Ui) {
        self.forced_hint = Some(kind);
        ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
    }

    pub(crate) fn pinned_if_down(&self, ctx: &egui::Context) -> Option<egui::Pos2> {
        let pin = self.pinned?;
        ctx.input(|i| i.pointer.primary_down()).then_some(pin)
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
            position: egui::Pos2::ZERO,
            velocity: egui::Vec2::ZERO,
            acceleration: egui::Vec2::ZERO,
            last_time: 0.0,
            initialized: false,
            feel,
            trajectory: VecDeque::with_capacity(TRAJECTORY_CAPACITY),
        }
    }

    pub(crate) fn position(&self) -> egui::Pos2 {
        self.position
    }

    pub(crate) fn is_initialized(&self) -> bool {
        self.initialized
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn velocity(&self) -> egui::Vec2 {
        self.velocity
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn acceleration(&self) -> egui::Vec2 {
        self.acceleration
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn trajectory(&self) -> &VecDeque<PointerSample> {
        &self.trajectory
    }

    pub(crate) fn reset(&mut self, position: egui::Pos2, time: f64) {
        self.position = position;
        self.velocity = egui::Vec2::ZERO;
        self.acceleration = egui::Vec2::ZERO;
        self.last_time = time;
        self.initialized = true;
        self.trajectory.clear();
    }

    pub(crate) fn pin(&mut self, position: egui::Pos2, time: f64) {
        self.position = position;
        self.velocity = egui::Vec2::ZERO;
        self.acceleration = egui::Vec2::ZERO;
        self.last_time = time;
        self.initialized = true;
    }

    pub(crate) fn sync(&mut self, position: egui::Pos2, time: f64) {
        if !self.initialized || position != self.position {
            self.reset(position, time);
        }
    }

    pub(crate) fn advance(
        &mut self,
        deltas: &[egui::Vec2],
        time: f64,
        bounds: egui::Rect,
    ) -> egui::Pos2 {
        if !self.initialized {
            self.reset(self.position, time);
        }
        let mut raw = egui::Vec2::ZERO;
        for d in deltas {
            raw += *d;
        }
        if raw == egui::Vec2::ZERO || !raw.is_finite() {
            let dt = (time - self.last_time).clamp(1.0 / 1000.0, 0.05) as f32;
            let previous = self.velocity;
            self.velocity = egui::Vec2::ZERO;
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
        let clamped = egui::pos2(
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
            self.trajectory.push_back(PointerSample {
                time: time - dt * back,
                position: at,
                delta: *d,
                velocity: new_velocity,
                acceleration: self.acceleration,
            });
            if self.trajectory.len() > TRAJECTORY_CAPACITY {
                self.trajectory.pop_front();
            }
        }
        self.last_time = time;
        clamped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> egui::Rect {
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1920.0, 1080.0))
    }

    #[test]
    fn every_raw_delta_is_recorded_and_fast_motion_gets_acceleration() {
        let mut slow = PointerMotion::new(PointerFeel {
            base_speed: 1.0,
            acceleration: 0.5,
            accel_start: 100.0,
            accel_full: 3000.0,
        });
        slow.reset(egui::pos2(100.0, 100.0), 0.0);
        let at = slow.advance(&[egui::vec2(1.0, 0.0)], 0.05, bounds());
        assert_eq!(at, egui::pos2(101.0, 100.0), "慢速移动不该被加速曲线放大");
        assert_eq!(slow.trajectory().len(), 1, "原始移动样本没有进轨迹记录");
        assert_eq!(slow.trajectory()[0].delta, egui::vec2(1.0, 0.0));

        let mut fast = PointerMotion::new(PointerFeel {
            base_speed: 1.0,
            acceleration: 0.5,
            accel_start: 100.0,
            accel_full: 3000.0,
        });
        fast.reset(egui::pos2(100.0, 100.0), 0.0);
        let at = fast.advance(&[egui::vec2(200.0, 0.0)], 0.016, bounds());
        assert!(
            at.x - 100.0 > 200.0,
            "高速移动没有获得加速度增益：{}",
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
        slow.reset(egui::pos2(100.0, 100.0), 0.0);
        fast.reset(egui::pos2(100.0, 100.0), 0.0);

        let a = slow.advance(&[egui::vec2(2.0, 0.0)], 0.02, bounds());
        let b = fast.advance(&[egui::vec2(2.0, 0.0)], 0.02, bounds());
        assert_eq!(a.x - 100.0, 1.0);
        assert_eq!(b.x - 100.0, 4.0);

        slow.reset(egui::pos2(100.0, 100.0), 0.0);
        fast.reset(egui::pos2(100.0, 100.0), 0.0);
        let a = slow.advance(&[egui::vec2(100.0, 0.0)], 0.02, bounds());
        let b = fast.advance(&[egui::vec2(100.0, 0.0)], 0.02, bounds());
        let ga = (a.x - 100.0) / 100.0;
        let gb = (b.x - 100.0) / 100.0;
        assert!(
            ga > 0.5 * 1.2 && gb > 2.0 * 1.2,
            "高速段没有进入加速区：{ga} / {gb}"
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
        p.reset(egui::pos2(100.0, 100.0), 0.0);
        let mut t = 0.0;
        for _ in 0..10 {
            t += 0.001;
            let _ = p.advance(&[egui::vec2(0.25, 0.0)], t, bounds());
        }
        assert!(
            (p.position().x - 102.5).abs() < 1e-6,
            "小位移被取整丢掉了：{}",
            p.position().x
        );

        p.reset(egui::pos2(100.0, 100.0), 0.0);
        let at = p.advance(&[egui::vec2(37.0, -12.0)], 0.016, bounds());
        assert_eq!(
            at,
            egui::pos2(137.0, 88.0),
            "速度 1x、加速度 0 时不该改变原始位移"
        );
    }

    #[test]
    fn acceleration_reacts_to_the_current_sample_without_a_tail() {
        let mut p = PointerMotion::new(PointerFeel {
            base_speed: 1.0,
            acceleration: 0.2,
            accel_start: 100.0,
            accel_full: 1000.0,
        });
        p.reset(egui::pos2(100.0, 100.0), 0.0);
        let fast = p.advance(&[egui::vec2(100.0, 0.0)], 0.016, bounds());
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
        p.reset(egui::pos2(10.0, 10.0), 0.0);
        let deltas = [
            egui::vec2(2.0, 1.0),
            egui::vec2(3.0, -1.0),
            egui::vec2(4.0, 2.0),
        ];
        let at = p.advance(&deltas, 0.016, bounds());
        assert_eq!(p.trajectory().len(), deltas.len());
        assert_eq!(at, egui::pos2(19.0, 12.0));
        let times: Vec<f64> = p.trajectory().iter().map(|s| s.time).collect();
        assert!(times.windows(2).all(|w| w[0] < w[1]), "轨迹时间没有铺开");
        assert_eq!(times.last().copied(), Some(0.016));
    }
}
