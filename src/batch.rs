use crate::{BuildingBlock, Get, GetMut, Prefetch};
use std::collections::LinkedList;
use std::ops::{Deref, DerefMut};

/// A list of building blocks in a building block.
///
/// `Batch` stores multiple building blocks of the same kind in a list.
/// The goal of this building block is to cut the cost of accessing a
/// building block into small pieces. Specifically, when the underlying
/// building block is a [`Compressor`](struct.Compressor.html), only small
/// part of a building block are decompressed into memory, thus reducing
/// the memory footprint.
pub struct Batch<C> {
    bb: LinkedList<C>,
}

impl<C> Batch<C> {
    /// Build an empty batch of building blocks.
    pub fn new() -> Self {
        Batch {
            bb: LinkedList::new(),
        }
    }

    /// Append a building block to the batch.
    pub fn append(&mut self, c: C) {
        self.bb.push_back(c);
    }
}

impl<T, const N: usize> From<[T; N]> for Batch<T> {
    fn from(arr: [T; N]) -> Self {
        Batch {
            bb: LinkedList::from(arr),
        }
    }
}

impl<C> Default for Batch<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, K, V, C> BuildingBlock<'a, K, V> for Batch<C>
where
    K: 'a,
    V: 'a + Ord,
    C: BuildingBlock<'a, K, V>,
{
    fn capacity(&self) -> usize {
        self.bb.iter().map(|c| c.capacity()).sum()
    }

    fn count(&self) -> usize {
        self.bb.iter().map(|c| c.count()).sum()
    }

    fn contains(&self, key: &K) -> bool {
        self.bb.iter().any(|c| c.contains(key))
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        self.bb.iter_mut().find_map(|c| c.take(key))
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut out = Vec::with_capacity(n * self.bb.len());

        for bb in self.bb.iter_mut() {
            out.append(&mut bb.pop(n));
        }

        let len = out.len();
        if n > len {
            out
        } else {
            out.sort_unstable_by(|(_, v1), (_, v2)| v1.cmp(v2));
            let victims = out.split_off(len - n);
            assert!(self.push(out).pop().is_none());
            victims
        }
    }

    fn push(&mut self, mut values: Vec<(K, V)>) -> Vec<(K, V)> {
        let mut bb = LinkedList::new();
        loop {
            if values.is_empty() {
                break;
            }
            let mut c = match self.bb.pop_front() {
                None => break,
                Some(c) => c,
            };
            values = c.push(values);
            bb.push_back(c);
        }
        self.bb.append(&mut bb);
        values
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(
            self.bb
                .iter_mut()
                .map(|c| c.flush())
                .collect::<Vec<Box<dyn Iterator<Item = (K, V)> + 'a>>>()
                .into_iter()
                .flatten(),
        )
    }
}

//------------------------------------------------------------------------//
// Get Trait Implementation                                               //
//------------------------------------------------------------------------//

impl<K, V, U, C> Get<K, V, U> for Batch<C>
where
    U: Deref<Target = V>,
    C: Get<K, V, U>,
{
    unsafe fn get(&self, key: &K) -> Option<U> {
        self.bb.iter().find_map(|c| c.get(key))
    }
}

impl<K, V, W, C> GetMut<K, V, W> for Batch<C>
where
    W: DerefMut<Target = V>,
    C: GetMut<K, V, W>,
{
    unsafe fn get_mut(&mut self, key: &K) -> Option<W> {
        self.bb.iter_mut().find_map(|c| c.get_mut(key))
    }
}

//------------------------------------------------------------------------//
// Prefetch Trait Implementation
//------------------------------------------------------------------------//

impl<'a, K, V, C> Prefetch<'a, K, V> for Batch<C>
where
    K: 'a,
    V: 'a + Ord,
    C: Prefetch<'a, K, V>,
{
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let mut out = Vec::with_capacity(keys.len());
        for c in self.bb.iter_mut() {
            if keys.is_empty() {
                break;
            }
            out.append(&mut c.take_multiple(keys))
        }
        out
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Batch;
    use crate::tests::{
        test_building_block, test_get, test_get_mut, test_prefetch,
    };
    use crate::Array;

    #[test]
    fn building_block() {
        test_building_block(Batch::<Array<(u16, u32)>>::new());
        test_building_block(Batch::from([Array::new(0)]));
        test_building_block(Batch::from([Array::new(0), Array::new(0)]));
        test_building_block(Batch::from([Array::new(0), Array::new(10)]));
        test_building_block(Batch::from([Array::new(10), Array::new(0)]));
        test_building_block(Batch::from([Array::new(10), Array::new(10)]));
    }

    #[test]
    fn get() {
        test_get(Batch::<Array<(u16, u32)>>::new());
        test_get(Batch::from([Array::new(0)]));
        test_get(Batch::from([Array::new(0), Array::new(0)]));
        test_get(Batch::from([Array::new(0), Array::new(10)]));
        test_get(Batch::from([Array::new(10), Array::new(0)]));
        test_get(Batch::from([Array::new(10), Array::new(10)]));
    }

    #[test]
    fn get_mut() {
        test_get_mut(Batch::<Array<(u16, u32)>>::new());
        test_get_mut(Batch::from([Array::new(0)]));
        test_get_mut(Batch::from([Array::new(0), Array::new(0)]));
        test_get_mut(Batch::from([Array::new(0), Array::new(10)]));
        test_get_mut(Batch::from([Array::new(10), Array::new(0)]));
        test_get_mut(Batch::from([Array::new(10), Array::new(10)]));
    }

    #[test]
    fn prefetch() {
        test_prefetch(Batch::<Array<(u16, u32)>>::new());
        test_prefetch(Batch::from([Array::new(0)]));
        test_prefetch(Batch::from([Array::new(0), Array::new(0)]));
        test_prefetch(Batch::from([Array::new(0), Array::new(10)]));
        test_prefetch(Batch::from([Array::new(10), Array::new(0)]));
        test_prefetch(Batch::from([Array::new(10), Array::new(10)]));
    }
}
