use std::collections::VecDeque;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

#[derive(Debug, Default)]
pub struct FungeStack {
    inner: VecDeque<i32>,
    queue_mode: bool,
    invert_mode: bool,
}
impl FungeStack {
    /// clear all data
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    /// reset modes and clear data
    pub fn reset(&mut self) {
        self.clear();
        self.queue_mode = false;
        self.invert_mode = false;
    }
    /// pop a value from the stack
    pub fn pop(&mut self) -> i32 {
        if self.queue_mode {
            self.inner.pop_front().unwrap_or(0)
        } else {
            self.inner.pop_back().unwrap_or(0)
        }
    }
    /// push a value onto the stack
    pub fn push(&mut self, n: i32) {
        if self.invert_mode {
            self.inner.push_front(n)
        } else {
            self.inner.push_back(n)
        }
    }
    /// the number of elements stored
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    /// rearrange data via lehmer codes
    pub fn permute(&mut self, p: usize) {
        let perm = lehmer::Lehmer::from_decimal(p, self.len()).to_permutation();
        let og = self.inner.clone();
        self.inner = perm.iter().map(|idx| og[*idx as usize]).collect();
    }
    /// render to a vertical list, bottom to top
    pub fn render(&self) -> Paragraph {
        let name = if self.queue_mode {"Stack".to_string()} else {"Queue".to_string()};

        Paragraph::new(self.inner.iter().rev().map(|n| Line::from(n.to_string())).collect::<Vec<Line>>())
            .block(Block::default().borders(Borders::ALL).title(name))
    }
}