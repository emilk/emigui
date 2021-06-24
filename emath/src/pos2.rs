use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::*;

/// A position on screen.
///
/// Normally given in points (logical pixels).
///
/// Mathematically this is known as a "point", but the term position was chosen so not to
/// conflict with the unit (one point = X physical pixels).
#[derive(Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Pos2 {
    pub x: f32,
    pub y: f32,
    // implicit w = 1
}

/// `pos2(x,y) == Pos2::new(x, y)`
#[inline(always)]
pub const fn pos2(x: f32, y: f32) -> Pos2 {
    Pos2 { x, y }
}

// ----------------------------------------------------------------------------
// Compatibility and convenience conversions to and from [f32; 2]:

impl From<[f32; 2]> for Pos2 {
    #[inline(always)]
    fn from(v: [f32; 2]) -> Self {
        Self { x: v[0], y: v[1] }
    }
}

impl From<&[f32; 2]> for Pos2 {
    #[inline(always)]
    fn from(v: &[f32; 2]) -> Self {
        Self { x: v[0], y: v[1] }
    }
}

impl From<Pos2> for [f32; 2] {
    #[inline(always)]
    fn from(v: Pos2) -> Self {
        [v.x, v.y]
    }
}

impl From<&Pos2> for [f32; 2] {
    #[inline(always)]
    fn from(v: &Pos2) -> Self {
        [v.x, v.y]
    }
}

// ----------------------------------------------------------------------------
// Compatibility and convenience conversions to and from (f32, f32):

impl From<(f32, f32)> for Pos2 {
    #[inline(always)]
    fn from(v: (f32, f32)) -> Self {
        Self { x: v.0, y: v.1 }
    }
}

impl From<&(f32, f32)> for Pos2 {
    #[inline(always)]
    fn from(v: &(f32, f32)) -> Self {
        Self { x: v.0, y: v.1 }
    }
}

impl From<Pos2> for (f32, f32) {
    #[inline(always)]
    fn from(v: Pos2) -> Self {
        (v.x, v.y)
    }
}

impl From<&Pos2> for (f32, f32) {
    #[inline(always)]
    fn from(v: &Pos2) -> Self {
        (v.x, v.y)
    }
}

// ----------------------------------------------------------------------------
// Mint compatibility and convenience conversions

#[cfg(feature = "mint")]
impl From<mint::Point2<f32>> for Pos2 {
    fn from(v: mint::Point2<f32>) -> Self {
        Self::new(v.x, v.y)
    }
}

#[cfg(feature = "mint")]
impl From<Pos2> for mint::Point2<f32> {
    fn from(v: Pos2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

// ----------------------------------------------------------------------------

impl Pos2 {
    /// The zero position, the origin.
    /// The top left corner in a GUI.
    /// Same as `Pos2::default()`.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[deprecated = "Use Pos2::ZERO instead"]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// The vector from origin to this position.
    /// `p.to_vec2()` is equivalent to `p - Pos2::default()`.
    #[inline(always)]
    pub fn to_vec2(self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    #[inline]
    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    #[inline]
    pub fn distance_sq(self, other: Self) -> f32 {
        (self - other).length_sq()
    }

    #[inline]
    pub fn distance_from_rect_sq(self, rect: Rect) -> f32 {
        let dx = if rect.min.x > self.x {
            rect.min.x - self.x
        } else if self.x > rect.max.x {
            self.x - rect.max.x
        } else {
            0.0
        };
        let dy = if rect.min.y > self.y {
            rect.min.y - self.y
        } else if self.y > rect.max.y {
            self.y - rect.max.y
        } else {
            0.0
        };
        dx * dx + dy * dy
    }

    #[inline(always)]
    pub fn floor(self) -> Self {
        pos2(self.x.floor(), self.y.floor())
    }

    #[inline(always)]
    pub fn round(self) -> Self {
        pos2(self.x.round(), self.y.round())
    }

    #[inline(always)]
    pub fn ceil(self) -> Self {
        pos2(self.x.ceil(), self.y.ceil())
    }

    /// True if all members are also finite.
    #[inline(always)]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// True if any member is NaN.
    #[inline(always)]
    pub fn any_nan(self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    #[must_use]
    #[inline]
    pub fn min(self, other: Self) -> Self {
        pos2(self.x.min(other.x), self.y.min(other.y))
    }

    #[must_use]
    #[inline]
    pub fn max(self, other: Self) -> Self {
        pos2(self.x.max(other.x), self.y.max(other.y))
    }

    #[must_use]
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
        }
    }
}

impl std::ops::Index<usize> for Pos2 {
    type Output = f32;

    #[inline(always)]
    fn index(&self, index: usize) -> &f32 {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Pos2 index out of bounds: {}", index),
        }
    }
}

impl std::ops::IndexMut<usize> for Pos2 {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Pos2 index out of bounds: {}", index),
        }
    }
}

impl Eq for Pos2 {}

impl AddAssign<Vec2> for Pos2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Vec2) {
        *self = Pos2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        };
    }
}

impl SubAssign<Vec2> for Pos2 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Vec2) {
        *self = Pos2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        };
    }
}

impl Add<Vec2> for Pos2 {
    type Output = Pos2;

    #[inline(always)]
    fn add(self, rhs: Vec2) -> Pos2 {
        Pos2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Pos2 {
    type Output = Vec2;

    #[inline(always)]
    fn sub(self, rhs: Pos2) -> Vec2 {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<Vec2> for Pos2 {
    type Output = Pos2;

    #[inline(always)]
    fn sub(self, rhs: Vec2) -> Pos2 {
        Pos2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::fmt::Debug for Pos2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:.1} {:.1}]", self.x, self.y)
    }
}
