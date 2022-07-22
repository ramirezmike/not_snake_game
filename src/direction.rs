use bevy::math::const_vec2;
use bevy::prelude::*;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

#[derive(Component, Clone, Copy, Debug, PartialEq, Default)]
pub struct Direction {
    unit_vector: Vec2,
}

impl Direction {
    #[inline]
    pub fn new(vec2: Vec2) -> Self {
        Self {
            unit_vector: vec2.normalize_or_zero(),
        }
    }

    pub const NEUTRAL: Direction = Direction {
        unit_vector: Vec2::ZERO,
    };

    pub const UP: Direction = Direction {
        unit_vector: const_vec2!([1.0, 0.0]),
    };

    pub const DOWN: Direction = Direction {
        unit_vector: const_vec2!([-1.0, 0.0]),
    };

    pub const RIGHT: Direction = Direction {
        unit_vector: const_vec2!([0.0, 1.0]),
    };

    pub const LEFT: Direction = Direction {
        unit_vector: const_vec2!([0.0, -1.0]),
    };
}

impl Add for Direction {
    type Output = Direction;
    fn add(self, other: Direction) -> Direction {
        Self {
            unit_vector: (self.unit_vector + other.unit_vector).normalize_or_zero(),
        }
    }
}

impl AddAssign for Direction {
    fn add_assign(&mut self, other: Direction) {
        *self = *self + other;
    }
}

impl Sub for Direction {
    type Output = Direction;

    fn sub(self, rhs: Direction) -> Direction {
        Self {
            unit_vector: (self.unit_vector - rhs.unit_vector).normalize_or_zero(),
        }
    }
}

impl SubAssign for Direction {
    fn sub_assign(&mut self, other: Direction) {
        *self = *self - other;
    }
}

impl Mul<f32> for Direction {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec2::new(self.unit_vector.x * rhs, self.unit_vector.y * rhs)
    }
}

impl Mul<Direction> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: Direction) -> Self::Output {
        Vec2::new(self * rhs.unit_vector.x, self * rhs.unit_vector.y)
    }
}

impl From<Direction> for Vec3 {
    fn from(direction: Direction) -> Vec3 {
        Vec3::new(direction.unit_vector.x, 0.0, direction.unit_vector.y)
    }
}

impl Neg for Direction {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            unit_vector: -self.unit_vector,
        }
    }
}
