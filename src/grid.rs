use ratatui::prelude::{Line, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::delta::Delta;
use crate::pointer::InstructionPointer;

/// 2d array with toroidal looping
#[derive(Debug, Default, Clone)]
pub struct FungeGrid {
    chars: Vec<Vec<char>>,
    og_chars: Vec<Vec<char>>,

    width: usize,
    height: usize
}
impl FungeGrid {
    /// parse some text into the 2d grid of characters
    pub fn new(text: String) -> FungeGrid {
        let width = text.lines().max_by_key(|l| l.len()).expect("empty text").len();
        let height = text.lines().count();
        let chars = text.lines().map(|line|[line.chars().collect::<Vec<char>>(),vec![' ';width-line.len()]].concat()).collect::<Vec<_>>();
        FungeGrid { og_chars: chars.clone(), chars, width, height, ..Default::default() }
    }
    /// reset back to the unmodified grid and return pc to (0,0)
    pub fn reset(&mut self) {
        self.chars = self.og_chars.clone();
        self.width = self.og_chars.iter().max_by_key(|l| l.len()).unwrap().len();
        self.height = self.og_chars.len();
    }
    /// find the top left corner, possibly lower if script mode + hashtag-started lines
    pub fn start_pos(&self, script_mode: bool) -> (usize, usize) {
        let y = if script_mode { self.chars.iter().position(|line| line.get(0) != Some(&'#')).unwrap_or(0) } else { 0 };
        (0, y)
    }

    /// find what character is at (x, y) in the grid
    pub fn char_at(&self, x: usize, y: usize) -> char {
        self.chars[y][x]
    }
    /// copy an area of the grid into a string with line breaks
    pub fn read_from(&self, left: usize, top: usize, right: usize, bottom: usize) -> String {
        if right >= self.width || bottom >= self.height {return String::new()}
        let mut output = String::new();
        for line in &self.chars[top..=bottom] {
            for c in &line[left..=right] {
                output.push(*c);
            }
        }
        output
    }
    /// find the position ahead of an ip in the current direction, including looping
    pub fn cell_ahead_ip(&self, ip: InstructionPointer) -> (usize, usize) {
        (
            (ip.x as i32 + ip.d.x).rem_euclid(self.width as i32) as usize,
            (ip.y as i32 + ip.d.y).rem_euclid(self.height as i32) as usize
        )
    }
    /// find the next runnable character ahead of a location
    pub fn runnable_char_ahead(&self, x: usize, y: usize, d: Delta) -> char {
        let x2 = (x as i32 + d.x).rem_euclid(self.width as i32) as usize;
        let y2 = (y as i32 + d.y).rem_euclid(self.height as i32) as usize;
        match self.chars[y2][x2] {
            ' '|';' => self.runnable_char_ahead(x2, y2, d),
            c => c
        }
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
    /// place some text within the grid
    pub fn place(&mut self, text: String, x: usize, y: usize) {
        for (m, line) in text.lines().enumerate() {
            for (n, c) in line.chars().enumerate() {
                self.set_char_or_expand(x+n, y+m, c);
            }
        }
    }

    /// the full width of the grid
    pub fn width(&self) -> usize {self.width}
    /// the full height of the grid
    pub fn height(&self) -> usize {self.height}

    /// render the grid into a paragraph, styling a selected spot bold
    pub fn render(&self, hx: usize, hy: usize) -> Paragraph {
        Paragraph::new(
            self.chars.iter().enumerate().map(|(y, r)| {
                if y == hy {
                    Line::from(r.iter().enumerate().map(|(x, c)| {
                        if x == hx {
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
}
