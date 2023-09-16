use rand::distributions::{Distribution, Standard};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub struct Delta {
    pub x: i32,
    pub y: i32
}
macro_rules! delta {
    ($x:expr,$y:expr) => {Delta { x: $x, y: $y }};
}
impl Distribution<Delta> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Delta {
        match rng.gen_range(0..=3) {
            0 => NORTH,
            1 => SOUTH,
            2 => EAST,
            _ => WEST,
        }
    }
}
impl Default for Delta {
    fn default() -> Self {
        delta!(1, 0)
    }
}
impl Delta {
    pub fn new(x: i32, y: i32) -> Delta {
        Delta { x, y }
    }
}

pub const NORTH: Delta = delta!(0, -1);
pub const SOUTH: Delta = delta!(0, 1);
pub const EAST: Delta = delta!(-1, 0);
pub const WEST: Delta = delta!(1, 0);