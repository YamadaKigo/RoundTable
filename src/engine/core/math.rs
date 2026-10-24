use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Color32(pub u8, pub u8, pub u8, pub u8);

impl Color32 {
    pub(crate) const TRANSPARENT: Self = Self(0, 0, 0, 0);
    pub(crate) const BLACK: Self = Self(0, 0, 0, 255);
    pub(crate) const WHITE: Self = Self(255, 255, 255, 255);
    #[cfg(test)]
    pub(crate) const RED: Self = Self(255, 0, 0, 255);

    pub(crate) const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b, 255)
    }

    pub(crate) const fn r(self) -> u8 {
        self.0
    }

    pub(crate) const fn g(self) -> u8 {
        self.1
    }

    pub(crate) const fn b(self) -> u8 {
        self.2
    }

    pub(crate) const fn a(self) -> u8 {
        self.3
    }

    pub(crate) const fn to_u32(self) -> u32 {
        ((self.0 as u32) << 16) | ((self.1 as u32) << 8) | self.2 as u32
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub(crate) const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub(crate) const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub(crate) fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    pub(crate) fn length_sq(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub(crate) fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    #[cfg(test)]
    pub(crate) fn round(self) -> Self {
        Self::new(self.x.round(), self.y.round())
    }

    pub(crate) fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Vec2 {
        rhs * self
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Pos2 {
    pub x: f32,
    pub y: f32,
}

impl Pos2 {
    pub(crate) const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub(crate) const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub(crate) fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    #[cfg(test)]
    pub(crate) fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    #[cfg(test)]
    pub(crate) fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }
}

impl Add<Vec2> for Pos2 {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign<Vec2> for Pos2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub<Vec2> for Pos2 {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub for Pos2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Rect {
    pub min: Pos2,
    pub max: Pos2,
}

impl Rect {
    pub(crate) const NOTHING: Self = Self {
        min: Pos2 {
            x: f32::INFINITY,
            y: f32::INFINITY,
        },
        max: Pos2 {
            x: f32::NEG_INFINITY,
            y: f32::NEG_INFINITY,
        },
    };

    pub(crate) const fn from_min_size(min: Pos2, size: Vec2) -> Self {
        Self {
            min,
            max: Pos2::new(min.x + size.x, min.y + size.y),
        }
    }

    pub(crate) const fn from_min_max(min: Pos2, max: Pos2) -> Self {
        Self { min, max }
    }

    pub(crate) fn from_center_size(center: Pos2, size: Vec2) -> Self {
        let half = size * 0.5;
        Self::from_min_max(center - half, center + half)
    }

    pub(crate) fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    pub(crate) fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    pub(crate) fn size(self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    pub(crate) fn center(self) -> Pos2 {
        Pos2::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
        )
    }

    pub(crate) fn contains(self, p: Pos2) -> bool {
        p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y
    }

    #[cfg(test)]
    pub(crate) fn contains_rect(self, other: Self) -> bool {
        other.min.x >= self.min.x
            && other.min.y >= self.min.y
            && other.max.x <= self.max.x
            && other.max.y <= self.max.y
    }

    pub(crate) fn intersect(self, other: Self) -> Self {
        let out = Self::from_min_max(
            Pos2::new(self.min.x.max(other.min.x), self.min.y.max(other.min.y)),
            Pos2::new(self.max.x.min(other.max.x), self.max.y.min(other.max.y)),
        );
        if out.is_positive() {
            out
        } else {
            Self::NOTHING
        }
    }

    pub(crate) fn is_positive(self) -> bool {
        self.max.x > self.min.x && self.max.y > self.min.y
    }

    pub(crate) fn expand(self, amount: f32) -> Self {
        Self::from_min_max(
            Pos2::new(self.min.x - amount, self.min.y - amount),
            Pos2::new(self.max.x + amount, self.max.y + amount),
        )
    }

    pub(crate) fn shrink(self, amount: f32) -> Self {
        self.expand(-amount)
    }

    pub(crate) fn union(self, other: Self) -> Self {
        if !self.is_positive() {
            return other;
        }
        if !other.is_positive() {
            return self;
        }
        Self::from_min_max(
            Pos2::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            Pos2::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        )
    }
}

pub(crate) const fn pos2(x: f32, y: f32) -> Pos2 {
    Pos2::new(x, y)
}

pub(crate) const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}
