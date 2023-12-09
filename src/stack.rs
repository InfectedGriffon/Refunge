use ratatui::prelude::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use std::collections::{vec_deque, VecDeque};
use std::fmt::{Debug, Formatter};
use std::ops::{Index, IndexMut};

/// a wrapper around VecDeque with queue/invert mode functionality
/// additionality defaults all pops to zero on empty stacks
#[derive(Default, Clone)]
pub struct FungeStack {
    inner: VecDeque<i32>,
    /// toggles which end of the stack is popped from
    pub queue_mode: bool,
    /// toggles which end of the stack is pushed to
    pub invert_mode: bool,
}

impl FungeStack {
    /// clear the entire stack
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    /// push a value onto the stack
    pub fn push(&mut self, val: i32) {
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
    pub fn iter(&self) -> vec_deque::Iter<'_, i32> {
        self.inner.iter()
    }

    /// pop a value from the stack (0 when empty)
    pub fn pop(&mut self) -> i32 {
        if self.queue_mode {
            self.inner.pop_front().unwrap_or_default()
        } else {
            self.inner.pop_back().unwrap_or_default()
        }
    }

    /// rearrange the stack based on a lehmer code
    pub fn permute(&mut self, p: usize) {
        let perm = lehmer::Lehmer::from_decimal(p, self.len()).to_permutation();
        let og = self.inner.clone();
        self.inner = perm.iter().map(|idx| og[*idx as usize]).collect();
    }

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

impl Debug for FungeStack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.inner).finish()
    }
}
impl IntoIterator for FungeStack {
    type Item = i32;
    type IntoIter = vec_deque::IntoIter<i32>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
impl<'a> IntoIterator for &'a FungeStack {
    type Item = &'a i32;
    type IntoIter = vec_deque::Iter<'a, i32>;

    fn into_iter(self) -> vec_deque::Iter<'a, i32> {
        self.iter()
    }
}
impl Index<usize> for FungeStack {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}
impl IndexMut<usize> for FungeStack {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}
impl From<Vec<i32>> for FungeStack {
    fn from(value: Vec<i32>) -> Self {
        FungeStack {
            inner: value.into(),
            queue_mode: false,
            invert_mode: false,
        }
    }
}
impl<const N: usize> From<[i32; N]> for FungeStack {
    fn from(value: [i32; N]) -> Self {
        FungeStack {
            inner: value.into(),
            queue_mode: false,
            invert_mode: false,
        }
    }
}
