use std::{ops::{DerefMut, Deref}, cmp::Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    pub fn new() -> Self {
        Self { line: 1, col: 1 }
    }
}

#[derive(Debug, Clone)]
pub struct Positioned<T> {
    pub inner: T,
    pub start: Position,
    pub end: Position,
}

impl<T> Deref for Positioned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T> DerefMut for Positioned<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
       if self.line != other.line {
           return if self.line > other.line { Ordering::Greater } else { Ordering::Less }
       }
       if self.col != other.col { 
           return if self.col > other.col { Ordering::Greater } else { Ordering::Less }
       }
       Ordering::Equal
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.line.partial_cmp(&other.line) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.col.partial_cmp(&other.col)
    }
}
