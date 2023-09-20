use crate::delta::Delta;

/// a befunge ip, with a 2d coordinate and direction
#[derive(Debug, Default, Clone, Copy)]
pub struct InstructionPointer {
    pub x: usize,
    pub y: usize,
    pub d: Delta,
    pub offset: (i32, i32),
    original: (usize, usize),
}
impl InstructionPointer {
    /// create a new instruction pointer facing right at specified coordinates
    pub fn new(x: usize, y: usize) -> InstructionPointer {
        Self { x, y, original: (x, y), ..Default::default() }
    }
    /// reset ip to original y facing right
    pub fn reset(&mut self) {
        self.x = self.original.0;
        self.y = self.original.1;
        self.d = Default::default();
    }

    /// rotate by 180 degrees
    pub fn turn_reverse(&mut self) {
        self.d = Delta::new(-self.d.x, -self.d.y);
    }
    /// rotate by 90 degrees anticlockwise
    pub fn turn_left(&mut self) {
        self.d = Delta::new(self.d.y, -self.d.x);
    }
    /// rotate by 90 degrees clockwise
    pub fn turn_right(&mut self) {
        self.d = Delta::new(-self.d.y, self.d.x)
    }

    /// move one space forwards, wrapping around if needed
    pub fn walk(&mut self, max_x: usize, max_y: usize) {
        self.x = (self.x as i32 + self.d.x).rem_euclid(max_x as i32) as usize;
        self.y = (self.y as i32 + self.d.y).rem_euclid(max_y as i32) as usize;
    }
    /// move one space backwards, wrapping around if needed
    pub fn walk_reverse(&mut self, max_x: usize, max_y: usize) {
        self.x = (self.x as i32 - self.d.x).rem_euclid(max_x as i32) as usize;
        self.y = (self.y as i32 - self.d.y).rem_euclid(max_y as i32) as usize;
    }
}
