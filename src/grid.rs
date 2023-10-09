use std::collections::VecDeque;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use crate::pointer::InstructionPointer;
use crate::vector::FungeVector;

/// 2d array with toroidal looping
#[derive(Debug, Default, Clone)]
pub struct FungeGrid {
    chars: Vec<Vec<char>>,
    og_chars: Vec<Vec<char>>,
    width: usize,
    height: usize,
    highlights: Vec<FungeVector>
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
    pub fn start_pos(&self, script_mode: bool) -> FungeVector {
        let y = if script_mode { self.chars.iter().position(|line| line.get(0) != Some(&'#')).unwrap_or(0) as i32 } else { 0 };
        FungeVector(0, y)
    }

    /// find what character is at (x, y) in the grid
    pub fn char_at(&self, pos: FungeVector) -> char {
        if pos.is_negative() { return ' ' }
        self.chars[pos.1 as usize][pos.0 as usize]
    }
    /// copy an area of the grid into a string with line breaks
    pub fn read_from(&self, start: FungeVector, end: FungeVector) -> String {
        if start.is_negative() || end.is_negative() {return String::new()}
        let (left, right, top, bottom) = (start.0 as usize, end.0 as usize, start.1 as usize, end.1 as usize);
        if right >= self.width || bottom >= self.height {return String::new()}
        let mut output = String::new();
        for line in &self.chars[top..=bottom] {
            for c in &line[left..=right] {
                output.push(*c);
            }
            output.push('\n');
        }
        output
    }
    /// find the position ahead of an ip in the current direction, including looping
    pub fn cell_ahead_ip(&self, ip: &InstructionPointer) -> FungeVector {
        FungeVector(
            (ip.pos.0 + ip.delta.0).rem_euclid(self.width as i32),
            (ip.pos.1 + ip.delta.1).rem_euclid(self.height as i32)
        )
    }
    /// find the next runnable character ahead of a location
    pub fn runnable_char_ahead(&self, pos: FungeVector, delta: FungeVector) -> char {
        let pos2 = FungeVector(
            (pos.0 + delta.0).rem_euclid(self.width as i32),
            (pos.1 + delta.1).rem_euclid(self.height as i32)
        );
        match self.chars[pos2.1 as usize][pos2.0 as usize] {
            ' '|';' => self.runnable_char_ahead(pos2, delta),
            c => c
        }
    }

    /// set a character in the grid, panics if outside the grid area
    pub fn set_char(&mut self, pos: FungeVector, c: char) {
        if pos.is_negative() { return }
        let (x, y) = (pos.0 as usize, pos.1 as usize);
        if x < self.width && y < self.height {
            self.chars[y][x] = c;
        } else {
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
    }
    /// place some text within the grid
    pub fn place(&mut self, text: String, pos: FungeVector, binary: bool) {
        if binary {
            for (n, c) in text.chars().enumerate() {
                self.set_char(pos + FungeVector(n as i32, 0), c);
            }
        } else {
            for (y, line) in text.lines().enumerate() {
                for (x, c) in line.chars().enumerate() {
                    self.set_char(pos + FungeVector(x as i32, y as i32), c);
                }
            }
        }
    }

    /// the full width of the grid
    pub fn width(&self) -> usize {self.width}
    /// the full height of the grid
    pub fn height(&self) -> usize {self.height}

    pub fn highlights(mut self, selections: VecDeque<InstructionPointer>) -> Self {
        self.highlights = selections.iter().map(|ip|ip.pos).collect();
        self
    }
}
impl Widget for FungeGrid {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(
            self.chars.iter().enumerate().map(|(y, r)| {
                Line::from(r.iter().enumerate().map(|(x, c)| {
                    if self.highlights.contains(&FungeVector(x as i32, y as i32)) {
                        Span::styled(c.to_string(), Style::default()
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::UNDERLINED))
                    } else {
                        Span::raw(c.to_string())
                    }
                }).collect::<Vec<Span>>())
            }).collect::<Vec<Line>>()
        ).block(Block::default().borders(Borders::ALL).title("Grid"))
         .render(area, buf)
    }
}
