use crate::direction::Direction;

/// a befunge ip, with a 2d coordinate and direction
#[derive(Debug, Default, Clone, Copy)]
pub struct InstructionPointer {
    pub x: usize,
    pub y: usize,
    pub dir: Direction,

    original: (usize, usize),
}
impl InstructionPointer {
    /// create a new instruction pointer facing right at specified coordinates
    pub fn new(x: usize, y: usize) -> InstructionPointer {
        Self { x, y, original: (x, y), dir: Direction::Right }
    }
    /// reset ip to original y facing right
    pub fn reset(&mut self) {
        self.x = self.original.0;
        self.y = self.original.1;
        self.dir = Direction::Right;
    }

    /// change direction
    pub fn face(&mut self, dir: Direction) {
        self.dir = dir
    }
    /// rotate by 180 degrees
    pub fn turn_reverse(&mut self) {
        self.dir = self.dir.next().next();
    }
    /// rotate by 90 degrees anticlockwise
    pub fn turn_left(&mut self) {
        self.dir = self.dir.next().next().next();
    }
    /// rotate by 90 degrees clockwise
    pub fn turn_right(&mut self) {
        self.dir = self.dir.next();
    }

    /// move one space forwards, wrapping around if needed
    pub fn walk(&mut self, max_x: usize, max_y: usize) {
        match self.dir {
            Direction::Up    => self.y = if self.y == 0 {max_y} else {self.y-1},
            Direction::Down  => self.y = if self.y == max_y {0} else {self.y+1},
            Direction::Right => self.x = if self.x == max_x {0} else {self.x+1},
            Direction::Left  => self.x = if self.x == 0 {max_x} else {self.x-1},
        }
    }
    /// move one space backwards, wrapping around if needed
    pub fn walk_reverse(&mut self, max_x: usize, max_y: usize) {
        match self.dir.next().next() {
            Direction::Up    => self.y = if self.y == 0 {max_y} else {self.y-1},
            Direction::Down  => self.y = if self.y == max_y {0} else {self.y+1},
            Direction::Right => self.x = if self.x == max_x {0} else {self.x+1},
            Direction::Left  => self.x = if self.x == 0 {max_x} else {self.x-1},
        }
    }

    /// current position of the ip (x, y)
    pub fn pos(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}