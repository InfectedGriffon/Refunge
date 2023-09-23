use std::ops::Add;
use rand::distributions::{Distribution, Standard};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub struct FungeVector(pub i32, pub i32);
impl FungeVector {
    pub fn invert(&mut self) {
        self.0 *= -1;
        self.1 *= -1;
    }
    pub fn left(&self) -> Self {
        FungeVector(self.1, -self.0)
    }
    pub fn turn_left(&mut self) {
        std::mem::swap(&mut self.0, &mut self.1);
        self.1 *= -1;
    }
    pub fn right(&self) -> Self {
        FungeVector(-self.1, self.0)
    }
    pub fn turn_right(&mut self) {
        std::mem::swap(&mut self.0, &mut self.1);
        self.0 *= -1;
    }
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
    fn default() -> Self { ORIGIN }
}
impl Add<FungeVector> for FungeVector {
    type Output = FungeVector;
    fn add(self, rhs: FungeVector) -> FungeVector {
        FungeVector(self.0+rhs.0, self.1+rhs.1)
    }
}

pub const ORIGIN: FungeVector = FungeVector(0,  0);
pub const NORTH: FungeVector = FungeVector( 0, -1);
pub const SOUTH: FungeVector = FungeVector( 0,  1);
pub const EAST:  FungeVector = FungeVector( 1,  0);
pub const WEST:  FungeVector = FungeVector(-1,  0);
