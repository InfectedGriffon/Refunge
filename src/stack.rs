use std::collections::VecDeque;
use std::fmt::Display;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

#[derive(Debug, Default)]
pub struct FungeStack<T> {
    inner: VecDeque<T>,
    pub queue_mode: bool,
    pub invert_mode: bool,
}
impl<T> FungeStack<T>
where
    T: Copy + Clone + Default + Display
{
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
    pub fn pop(&mut self) -> T {
        if self.queue_mode {
            self.inner.pop_front().unwrap_or_default()
        } else {
            self.inner.pop_back().unwrap_or_default()
        }
    }
    /// push a value onto the stack
    pub fn push(&mut self, val: T) {
        if self.invert_mode {
            self.inner.push_front(val)
        } else {
            self.inner.push_back(val)
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
        let name = if self.queue_mode {"Queue".to_string()} else {"Stack".to_string()};

        Paragraph::new(self.inner.iter().rev().map(|val| Line::from(val.to_string())).collect::<Vec<Line>>())
            .block(Block::default().borders(Borders::ALL).title(name))
    }
}
impl<T> From<Vec<T>> for FungeStack<T> {
    fn from(value: Vec<T>) -> Self {
        FungeStack {
            inner: value.into(),
            queue_mode: false,
            invert_mode: false,
        }
    }
}

impl<T, const N: usize> From<[T; N]> for FungeStack<T> {
    fn from(value: [T; N]) -> Self {
        FungeStack {
            inner: value.into(),
            queue_mode: false,
            invert_mode: false,
        }
    }
}
impl<T: PartialEq<T>> PartialEq for FungeStack<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<T: PartialEq<T>> Eq for FungeStack<T> {}