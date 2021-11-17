use crate::{BuildingBlock, Get};
use std::cmp::Eq;
use std::vec::Vec;

//-------------------------------------------------------------------------
//  Vector struct
//-------------------------------------------------------------------------

/// Unordered [`BuildingBlock`](../trait.BuildingBlock.html).
///
/// Vector holds values in a `Vec<(index, value)>`.
/// It is an unordered container.
/// Any operation on vector (
/// [`push()`](../trait.BuildingBlock.html#tymethod.push),
/// [`take()`](../trait.BuildingBlock.html#tymethod.take),
/// [`pop()`](../trait.BuildingBlock.html#tymethod.pop)
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::building_block::container::Vector;
///
/// // container with only 1 element.
/// let mut c = Vector::new(1);
///
/// // BuildingBlock as room for first element and returns None.
/// assert!(c.push(vec![("first", 4)]).pop().is_none());
///
/// // BuildingBlock is full and pops inserted value.
/// let (key, value) = c.push(vec![("second", 12)]).pop().unwrap();
/// assert!(key == "second");
/// assert!(value == 12);
/// ```
pub struct Vector<K: Eq, V> {
    capacity: usize,
    values: Vec<(K, V)>,
}

impl<K: Eq, V> Vector<K, V> {
    pub fn new(n: usize) -> Self {
        Vector {
            capacity: n,
            values: Vec::with_capacity(n + 1),
        }
    }
}

//------------------------------------------------------------------------//
// Iterator to take elements out.                                         //
//------------------------------------------------------------------------//

struct VectorTakeIterator<'a, K, V> {
    vec: &'a mut Vec<(K, V)>,
    key: &'a K,
    current: usize,
}

impl<'a, K: Eq, V> Iterator for VectorTakeIterator<'a, K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        let n = self.vec.len();
        if n == 0 {
            None
        } else {
            loop {
                if n <= self.current {
                    break None;
                } else if &self.vec[self.current].0 == self.key {
                    break Some(self.vec.swap_remove(self.current));
                } else {
                    self.current += 1;
                }
            }
        }
    }
}

//------------------------------------------------------------------------//
//  BuildingBlock implementation.                                             //
//------------------------------------------------------------------------//

impl<'a, K, V> BuildingBlock<'a, K, V> for Vector<K, V>
where
    K: 'a + Eq,
    V: 'a + Ord,
{
    fn capacity(&self) -> usize {
        return self.capacity;
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(self.values.split_off(0).into_iter())
    }

    fn contains(&self, key: &K) -> bool {
        self.values.iter().any(|(k, _)| k == key)
    }

    fn count(&self) -> usize {
        return self.values.len();
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        self.values.sort_unstable_by(|(_, v1), (_, v2)| v1.cmp(v2));
        let i = self.values.len();
        self.values.split_off(i - std::cmp::min(i, n))
    }

    fn push(&mut self, mut elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let n = std::cmp::min(
            self.capacity - self.values.len(),
            elements.len(),
        );
        let out = elements.split_off(n);

        if n > 0 {
            self.values.append(&mut elements);
        }
        out
    }

    fn take<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = (K, V)> + 'b> {
        Box::new(VectorTakeIterator {
            vec: &mut self.values,
            key: key,
            current: 0usize,
        })
    }
}

impl<'a, K: 'a + Eq, V: 'a + Ord> Get<'a, K, V> for Vector<K, V> {
    fn get<'b>(
        &'b self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = &'b (K, V)> + 'b> {
        Box::new(self.values.iter().filter_map(move |kv| {
            if &kv.0 == key {
                Some(kv)
            } else {
                None
            }
        }))
    }

    fn get_mut<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = &'b mut (K, V)> + 'b> {
        Box::new(self.values.iter_mut().filter_map(move |kv| {
            if &kv.0 == key {
                Some(kv)
            } else {
                None
            }
        }))
    }
}
