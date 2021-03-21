use crate::container::{Buffered, Container, Get};
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

//------------------------------------------------------------------------//
//  Container implementation.                                             //
//------------------------------------------------------------------------//

impl<'a, K, V> Container<'a, K, V> for Vector<(K, V)>
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

impl<'a, K: 'a + Eq, V: 'a + Ord> Buffered<'a, K, V> for Vector<(K, V)> {
    fn push_buffer(&mut self, mut elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let mut out = Vec::<(K, V)>::with_capacity(elements.len());
        let mut duplicate = Vec::<(K, V)>::with_capacity(elements.len());

        // First pass removes duplicate keys
        let get_key = |elements: &Vec<(K, V)>, k: &K| {
            elements.iter().enumerate().find_map(|(i, (kk, _))| {
                if k == kk {
                    Some(i)
                } else {
                    None
                }
            })
        };
        for i in 0..self.values.len() {
            match get_key(&elements, &self.values[i].0) {
                None => (),
                Some(j) => {
                    duplicate.push(elements.swap_remove(j));
                    out.push(self.values.swap_remove(i));
                }
            }
        }

        // Second pass remove elements popping.
        self.values.sort_by(|(_, a), (_, b)| a.cmp(b));
        let n = elements.len() + self.values.len() + duplicate.len();
        if n > self.capacity {
            out.append(&mut self.values.split_off(n - self.capacity))
        }
        self.values.append(&mut elements);
        self.values.append(&mut duplicate);

        out
    }
}
