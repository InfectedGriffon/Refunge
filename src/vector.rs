use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::ops::{Add, AddAssign};

/// represents a 2-dimensional vector with integer coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FungeVector(pub i32, pub i32);
impl FungeVector {
    /// negate each dimension
    pub fn invert(&mut self) {
        self.0 *= -1;
        self.1 *= -1;
    }
    /// return the result of rotating this vector 90 degrees counterclockwise
    pub fn left(&self) -> Self {
        FungeVector(self.1, -self.0)
    }
    /// rotate this vector 90 degrees counterclockwise
    pub fn turn_left(&mut self) {
        std::mem::swap(&mut self.0, &mut self.1);
        self.1 *= -1;
    }
    /// return the result of rotating this vector 90 degrees clockwise
    pub fn right(&self) -> Self {
        FungeVector(-self.1, self.0)
    }
    /// rotate this vector 90 degrees clockwise
    pub fn turn_right(&mut self) {
        std::mem::swap(&mut self.0, &mut self.1);
        self.0 *= -1;
    }
    /// returns true if either coordinate is less than zero
    pub fn is_negative(&self) -> bool {
        self.0 < 0 || self.1 < 0
    }
}
impl Distribution<FungeVector> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> FungeVector {
        match rng.gen_range(0..=3) {
            0 => NORTH,
            1 => SOUTH,
            2 => EAST,
            _ => WEST,
        }
    }
}
impl Default for FungeVector {
    fn default() -> Self {
        ORIGIN
    }
}
impl Add<FungeVector> for FungeVector {
    type Output = FungeVector;
    fn add(self, rhs: FungeVector) -> FungeVector {
        FungeVector(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl AddAssign for FungeVector {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

/// (0, 0)
pub const ORIGIN: FungeVector = FungeVector(0, 0);
/// (0, -1)
pub const NORTH: FungeVector = FungeVector(0, -1);
/// (0, 1)
pub const SOUTH: FungeVector = FungeVector(0, 1);
/// (1, 0)
pub const EAST: FungeVector = FungeVector(1, 0);
/// (-1, 0)
pub const WEST: FungeVector = FungeVector(-1, 0);
