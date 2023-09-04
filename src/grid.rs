use ratatui::prelude::{Line, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::direction::Direction::{self, *};

/// 2d vec of chars + the pc
#[derive(Debug, Default, Clone)]
pub struct FungeGrid {
    chars: Vec<Vec<char>>,
    og_chars: Vec<Vec<char>>,

    x: usize,
    y: usize,
    dir: Direction,

    width: usize,
    height: usize
}
impl FungeGrid {
    /// parse some text into the 2d grid of characters
    pub fn new(text: String, scriptmode: bool) -> FungeGrid {
        let width = text.lines().max_by_key(|l| l.len()).expect("empty text").len();
        let height = text.lines().count();
        let chars: Vec<Vec<char>> = text.lines().map(|line| {
            let mut row = line.chars().collect::<Vec<char>>();
            row.extend(vec![' '; width - line.len()]);
            row
        }).collect();

        let y = if scriptmode { chars.iter().position(|line| line.get(0) != Some(&'#')).unwrap_or(0) } else { 0 };
        FungeGrid { og_chars: chars.clone(), chars, y, width, height, ..Default::default() }
    }
    /// reset back to the unmodified grid and return pc to (0,0)
    pub fn reset(&mut self) {
        self.chars = self.og_chars.clone();
        self.x = 0;
        self.y = 0;
        self.dir = Right;
    }

    /// find what character is at (x, y) in the grid
    pub fn char_at(&self, x: usize, y: usize) -> char {
        self.chars[y][x]
    }
    /// find what character is at the pointer's location in the grid
    pub fn current_char(&self) -> char {
        self.chars[self.y][self.x]
    }
    /// set a character in the grid, panics if outside the grid area
    pub fn set_char(&mut self, x: usize, y: usize, c: char) {
        if x < self.width && y < self.height {
            self.chars[y][x] = c;
        } else {
            panic!("trying to access area outside of grid")
        }
    }
    /// set a character in the grid, expanding the grid area if needed
    pub fn set_char_or_expand(&mut self, x: usize, y: usize, c: char) {
        while x >= self.width {
            for row in &mut self.chars {
                (*row).push(' ');
            }
            self.width += 1;
        }
        while y >= self.height {
            self.chars.push(vec![' '; self.width]);
            self.height += 1;
        }
        self.chars[y][x] = c;
    }

    /// the full width of the grid
    pub fn width(&self) -> usize {self.width}
    /// the full height of the grid
    pub fn height(&self) -> usize {self.height}

    /// change the pc's direction
    pub fn face(&mut self, dir: Direction) {
        self.dir = dir
    }
    /// rotate the pc by 180 degrees
    pub fn turn_reverse(&mut self) {
        self.dir = self.dir.next().next();
    }
    /// rotate the pc by 90 degrees anticlockwise
    pub fn turn_left(&mut self) {
        self.dir = self.dir.next().next().next();
    }
    /// rotate the pc by 90 degrees clockwise
    pub fn turn_right(&mut self) {
        self.dir = self.dir.next();
    }

    /// move one space forwards, wrapping around if needed
    pub fn walk(&mut self) {
        self.step(self.dir)
    }
    /// move one space backwards, wrapping around if needed
    pub fn walk_reverse(&mut self) {
        self.step(self.dir.next().next())
    }
    fn step(&mut self, dir: Direction) {
        match dir {
            Up    => self.y = if self.y == 0 {self.height-1} else {self.y-1},
            Down  => self.y = if self.y == self.height-1 {0} else {self.y+1},
            Right => self.x = if self.x == self.width-1 {0} else {self.x+1},
            Left  => self.x = if self.x == 0 {self.width-1} else {self.x-1},
        }
    }

    /// render the grid into a paragraph, styling the pc's selected spot bold
    pub fn render(&self) -> Paragraph {
        Paragraph::new(
            self.chars.iter().enumerate().map(|(y, r)| {
                if y == self.y {
                    Line::from(r.iter().enumerate().map(|(x, c)| {
                        if x == self.x {
                            Span::styled(c.to_string(), Style::default()
                                .add_modifier(Modifier::BOLD)
                                .add_modifier(Modifier::UNDERLINED))
                        } else {
                            Span::raw(c.to_string())
                        }
                    }).collect::<Vec<Span>>())
                } else {
                    Line::from(r.iter().collect::<String>())
                }
            }).collect::<Vec<Line>>()
        ).block(Block::default().borders(Borders::ALL).title("Grid"))
    }
    pub fn pos(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}
