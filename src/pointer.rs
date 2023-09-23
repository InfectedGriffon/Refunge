use crate::vector;
use crate::vector::FungeVector;

/// a befunge ip, with a 2d coordinate and direction
#[derive(Debug, Default, Clone, Copy)]
pub struct InstructionPointer {
    pub pos: FungeVector,
    pub delta: FungeVector,
    pub offset: FungeVector,
    pub original: FungeVector
}
impl InstructionPointer {
    /// create a new instruction pointer facing right at specified coordinates
    pub fn new(pos: FungeVector) -> InstructionPointer {
        Self { pos, original: pos, delta: vector::EAST, ..Default::default() }
    }
    /// reset ip to original y facing right
    pub fn reset(&mut self) {
        self.pos = self.original;
        self.delta = vector::EAST;
        self.offset = vector::ORIGIN;
    }

    /// move one space forwards, wrapping around if needed
    pub fn walk(&mut self, max_x: usize, max_y: usize) {
        self.pos.0 = (self.pos.0 + self.delta.0).rem_euclid(max_x as i32);
        self.pos.1 = (self.pos.1 + self.delta.1).rem_euclid(max_y as i32);
    }
    /// move one space backwards, wrapping around if needed
    pub fn walk_reverse(&mut self, max_x: usize, max_y: usize) {
        self.pos.0 = (self.pos.0 - self.delta.0).rem_euclid(max_x as i32);
        self.pos.1 = (self.pos.1 - self.delta.1).rem_euclid(max_y as i32);
    }
}
