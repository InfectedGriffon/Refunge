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
}