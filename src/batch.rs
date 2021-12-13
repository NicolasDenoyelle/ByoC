use crate::{BuildingBlock, Get, GetMut, Prefetch};
use std::collections::LinkedList;
use std::ops::{Deref, DerefMut};

pub struct Batch<C> {
    bb: LinkedList<C>,
}

impl<C> Batch<C> {
    pub fn new() -> Self {
        Batch {
            bb: LinkedList::new(),
        }
    }

    pub fn append(mut self, c: C) -> Self {
        self.bb.push_back(c);
        self
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
    V: 'a,
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
        let mut out = Vec::new();
        let mut bb = LinkedList::new();

        loop {
            let n = n - out.len();
            if n == 0 {
                break;
            }
            let mut c = match self.bb.pop_back() {
                None => break,
                Some(c) => c,
            };
            out.append(&mut c.pop(n));
            bb.push_front(c);
        }

        bb.append(&mut self.bb);
        std::mem::swap(&mut self.bb, &mut bb);
        out
    }

    fn push(&mut self, mut values: Vec<(K, V)>) -> Vec<(K, V)> {
        for c in self.bb.iter_mut() {
            if values.is_empty() {
                break;
            }
            values = c.push(values);
        }

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
    V: 'a,
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
        test_building_block(Batch::new().append(Array::new(0)));
        test_building_block(
            Batch::new().append(Array::new(0)).append(Array::new(0)),
        );
        test_building_block(
            Batch::new().append(Array::new(0)).append(Array::new(10)),
        );
        test_building_block(
            Batch::new().append(Array::new(10)).append(Array::new(0)),
        );
        test_building_block(
            Batch::new().append(Array::new(10)).append(Array::new(10)),
        );
    }

    #[test]
    fn get() {
        test_get(Batch::<Array<(u16, u32)>>::new());
        test_get(Batch::new().append(Array::new(0)));
        test_get(Batch::new().append(Array::new(0)).append(Array::new(0)));
        test_get(
            Batch::new().append(Array::new(0)).append(Array::new(10)),
        );
        test_get(
            Batch::new().append(Array::new(10)).append(Array::new(0)),
        );
        test_get(
            Batch::new().append(Array::new(10)).append(Array::new(10)),
        );
    }

    #[test]
    fn get_mut() {
        test_get_mut(Batch::<Array<(u16, u32)>>::new());
        test_get_mut(Batch::new().append(Array::new(0)));
        test_get_mut(
            Batch::new().append(Array::new(0)).append(Array::new(0)),
        );
        test_get_mut(
            Batch::new().append(Array::new(0)).append(Array::new(10)),
        );
        test_get_mut(
            Batch::new().append(Array::new(10)).append(Array::new(0)),
        );
        test_get_mut(
            Batch::new().append(Array::new(10)).append(Array::new(10)),
        );
    }

    #[test]
    fn prefetch() {
        test_prefetch(Batch::<Array<(u16, u32)>>::new());
        test_prefetch(Batch::new().append(Array::new(0)));
        test_prefetch(
            Batch::new().append(Array::new(0)).append(Array::new(0)),
        );
        test_prefetch(
            Batch::new().append(Array::new(0)).append(Array::new(10)),
        );
        test_prefetch(
            Batch::new().append(Array::new(10)).append(Array::new(0)),
        );
        test_prefetch(
            Batch::new().append(Array::new(10)).append(Array::new(10)),
        );
    }
}
