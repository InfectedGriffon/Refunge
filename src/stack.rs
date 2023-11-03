use crate::befunge::Frame;
use ratatui::prelude::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::collections::{vec_deque, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

/// a wrapper around VecDeque with queue/invert mode functionality
/// additionality defaults all pops to zero on empty stacks
#[derive(Default, Clone)]
pub struct FungeStack<T> {
    inner: VecDeque<T>,
    /// toggles which end of the stack is popped from
    pub queue_mode: bool,
    /// toggles which end of the stack is pushed to
    pub invert_mode: bool,
}

impl<T> FungeStack<T> {
    /// clear the entire stack
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

    /// returns the number of values in the stack
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    /// returns a bottom-to-top iterator
    pub fn iter(&self) -> vec_deque::Iter<'_, T> {
        self.inner.iter()
    }
    /// push a value onto the bottom of the stack
    pub fn push_front(&mut self, value: T) {
        self.inner.push_front(value)
    }
    /// push a value onto the top of the stack
    pub fn push_back(&mut self, value: T) {
        self.inner.push_back(value)
    }
    /// pop a value from the bottom of the stack
    pub fn pop_front(&mut self) -> Option<T> {
        self.inner.pop_front()
    }
    /// pop a value from the top of the stack
    pub fn pop_back(&mut self) -> Option<T> {
        self.inner.pop_back()
    }
}
impl<T: Default> FungeStack<T> {
    /// pop a value from the stack (0 when empty)
    pub fn pop(&mut self) -> T {
        if self.queue_mode {
            self.inner.pop_front().unwrap_or_default()
        } else {
            self.inner.pop_back().unwrap_or_default()
        }
    }
}
impl<T: Copy + Clone> FungeStack<T> {
    /// rearrange the stack based on a lehmer code
    pub fn permute(&mut self, p: usize) {
        let perm = lehmer::Lehmer::from_decimal(p, self.len()).to_permutation();
        let og = self.inner.clone();
        self.inner = perm.iter().map(|idx| og[*idx as usize]).collect();
    }
}
impl<T: Display> FungeStack<T> {
    /// render to a vertical list, bottom to top
    pub fn render(&self, frame: &mut Frame, area: Rect, max_height: u16, title: impl Into<String>) {
        let widget = Paragraph::new(
            self.inner
                .iter()
                .rev()
                .map(|val| Line::from(val.to_string()))
                .collect::<Vec<Line>>(),
        )
        .block(Block::default().borders(Borders::ALL).title(title.into()));
        let bits = Layout::new()
            .constraints(vec![
                Constraint::Length((self.len() as u16).max(max_height)),
                Constraint::Min(0),
            ])
            .split(area);
        frame.render_widget(widget, bits[0]);
    }
}

impl<T: Debug> Debug for FungeStack<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.inner).finish()
    }
}
impl<T> IntoIterator for FungeStack<T> {
    type Item = T;
    type IntoIter = vec_deque::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
impl<'a, T> IntoIterator for &'a FungeStack<T> {
    type Item = &'a T;
    type IntoIter = vec_deque::Iter<'a, T>;

    fn into_iter(self) -> vec_deque::Iter<'a, T> {
        self.iter()
    }
}
impl<T> Index<usize> for FungeStack<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}
impl<T> IndexMut<usize> for FungeStack<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
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
