use std::collections::VecDeque;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

#[allow(unused)]
#[derive(PartialEq, Debug, Default)]
enum DataMode { #[default] Stack, Queue }

#[derive(Debug, Default)]
pub struct FungeData {
    inner: VecDeque<i32>,
    mode: DataMode
}
impl FungeData {
    /// reset and clear all data
    pub fn clear(&mut self) {
        self.inner.clear();
        self.mode = Default::default();
    }
    /// pop a value from the top of the stack / start of the queue
    pub fn pop(&mut self) -> i32 {
        match self.mode {
            DataMode::Stack => self.inner.pop_back().unwrap_or(0),
            DataMode::Queue => self.inner.pop_front().unwrap_or(0)
        }
    }
    /// push a value onto top of stack / end of queue
    pub fn push(&mut self, n: i32) {
        self.inner.push_back(n)
    }
    /// the number of elements stored
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    /// first in, last out
    #[allow(unused)]
    pub fn stack_mode(&mut self) {
        self.mode = DataMode::Stack;
    }
    /// first in, first out
    #[allow(unused)]
    pub fn queue_mode(&mut self) {
        self.mode = DataMode::Queue;
    }
    /// rearrange data via lehmer codes
    pub fn permute(&mut self, p: usize) {
        let perm = lehmer::Lehmer::from_decimal(p, self.len()).to_permutation();
        let og = self.inner.clone();
        self.inner = perm.iter().map(|idx| og[*idx as usize]).collect();
    }
    /// render to a vertical list, bottom to top
    pub fn render(&self) -> Paragraph {
        let name = match self.mode {
            DataMode::Stack => "Stack".to_string(),
            DataMode::Queue => "Queue".to_string()
        };

        Paragraph::new(self.inner.iter().rev().map(|n| Line::from(n.to_string())).collect::<Vec<Line>>())
            .block(Block::default().borders(Borders::ALL).title(name))
    }
}