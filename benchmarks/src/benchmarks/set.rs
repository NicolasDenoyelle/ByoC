use std::collections::BTreeSet;
use std::iter::Iterator;

pub struct SetIter<T: Ord + Copy, I: Iterator<Item = T>> {
    visited: BTreeSet<T>,
    it: I,
}

impl<T: Ord + Copy, I: Iterator<Item = T>> SetIter<T, I> {
    pub fn new(it: I) -> Self {
        Self {
            visited: BTreeSet::new(),
            it,
        }
    }
}

impl<T: Ord + Copy, I: Iterator<Item = T>> Iterator for SetIter<T, I> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = match self.it.next() {
                None => {
                    return None;
                }
                Some(i) => i,
            };
            if self.visited.insert(item) {
                return Some(item);
            }
        }
    }
}

pub trait Set<T: Ord + Copy, I: Iterator<Item = T>> {
    fn set(self) -> SetIter<T, I>;
}

impl<T: Ord + Copy, I: Iterator<Item = T>> Set<T, Self> for I {
    fn set(self) -> SetIter<T, Self> {
        SetIter::new(self)
    }
}
