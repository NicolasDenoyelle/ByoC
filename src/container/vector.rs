use crate::container::{Container, Get};
use crate::marker::Packed;
use std::cmp::Eq;
use std::vec::Vec;

//-------------------------------------------------------------------------
//  Vector struct
//-------------------------------------------------------------------------

/// Unordered [`container`](trait.Container.html).
///
/// Vector holds values in a `Vec<(index, value)>`.
/// It is an unordered container.
/// Any operation on vector (
/// [`push()`](trait.Container.html#tymethod.push),
/// [`take()`](trait.Container.html#tymethod.take),
/// [`pop()`](trait.Container.html#tymethod.pop)
///
/// ## Examples
///
/// ```
/// use cache::container::{Container, Vector};
///
/// // container with only 1 element.
/// let mut c = Vector::new(1);
///
/// // Container as room for first element and returns None.
/// assert!(c.push("first", 4).is_none());
///
/// // Container is full and pops a victim.
/// let (key, value) = c.push("second", 12).unwrap();
///
/// // The victim is the second reference because it has a greater value.
/// assert!(key == "first");
/// assert!(value == 4);
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

//------------------------------------------------------------------------//
//  Container implementation.                                             //
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

impl<'a, K, V> Container<'a, K, V> for Vector<K, V>
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

    fn clear(&mut self) {
        self.values.clear()
    }

    fn pop(&mut self) -> Option<(K, V)> {
        let i = match self
            .values
            .iter()
            .enumerate()
            .min_by(|(_, (_, v1)), (_, (_, v2))| v1.cmp(v2))
        {
            None => return None,
            Some((i, _)) => i,
        };
        Some(self.values.swap_remove(i))
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        if self.capacity == 0 {
            return Some((key, reference));
        }

        let victim = if self.values.len() >= self.capacity {
            self.pop()
        } else {
            None
        };
        self.values.push((key, reference));
        victim
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

impl<'a, K, V> Packed<'a, K, V> for Vector<K, V>
where
    K: 'a + Eq,
    V: 'a + Ord,
{
}
