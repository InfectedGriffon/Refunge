use rand::distributions::{Distribution, Standard};
use rand::Rng;
use crate::direction::Direction::*;

/// one of the four cardinal directions
#[derive(Debug, Default, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    #[default]
    Right,
    Left
}
impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..=3) {
            0 => Up,
            1 => Down,
            2 => Right,
            _ => Left,
        }
    }
}
impl Direction {
    /// the next direction clockwise
    pub fn next(&self) -> Direction {
        match self {
            Up => Right,
            Right => Down,
            Down => Left,
            Left => Up
        }
    }
    /// represent the direction as a delta (using right/down positive)
    pub fn as_delta(&self) -> (i32, i32) {
        match self {
            Up    => (0, -1),
            Down  => (0, 1),
            Right => (1, 0),
            Left  => (-1, 0),
        }
    }
}