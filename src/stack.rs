use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

#[derive(Default, Clone)]
pub struct FungeStack<T> {
    inner: VecDeque<T>,
    pub queue_mode: bool,
    pub invert_mode: bool,
}

impl<T> FungeStack<T> {
    /// clear all data
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    /// push a value onto the stack
    pub fn push(&mut self, val: T) {
        if self.invert_mode {
            self.inner.push_front(val)
        } else {
            self.inner.push_back(val)
        }
    }
}
impl<T: Default> FungeStack<T> {
    /// pop a value from the stack
    pub fn pop(&mut self) -> T {
        if self.queue_mode {
            self.inner.pop_front().unwrap_or_default()
        } else {
            self.inner.pop_back().unwrap_or_default()
        }
    }
}
impl<T: Copy + Clone> FungeStack<T>
{
    /// rearrange data via lehmer codes
    pub fn permute(&mut self, p: usize) {
        let perm = lehmer::Lehmer::from_decimal(p, self.len()).to_permutation();
        let og = self.inner.clone();
        self.inner = perm.iter().map(|idx| og[*idx as usize]).collect();
    }
}
impl<T: Display> FungeStack<T> {
    /// render to a vertical list, bottom to top
    pub fn render(&self, title: impl Into<String>) -> Paragraph {
        Paragraph::new(self.inner.iter().rev().map(|val| Line::from(val.to_string())).collect::<Vec<Line>>())
            .block(Block::default().borders(Borders::ALL).title(title.into()))
    }
}

impl<T: Debug> Debug for FungeStack<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.inner).finish()
    }
}

impl<T> Deref for FungeStack<T> {
    type Target = VecDeque<T>;
    fn deref(&self) -> &Self::Target { &self.inner }
}
impl<T> DerefMut for FungeStack<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
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