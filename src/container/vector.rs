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
/// use cache::container::Vector;
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

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.values.iter().enumerate().find_map(|(i, (k, _))| {
            if k == key {
                Some(i.clone())
            } else {
                None
            }
        }) {
            None => None,
            Some(i) => Some(self.values.swap_remove(i)),
        }
    }
}

impl<'a, K: Eq, V: Ord> Get<'a, K, V, &'a V, &'a mut V> for Vector<K, V> {
    fn get(&'a self, key: &K) -> Option<&'a V> {
        self.values.iter().find_map(
            move |(k, v)| {
                if k == key {
                    Some(v)
                } else {
                    None
                }
            },
        )
    }

    fn get_mut(&'a mut self, key: &K) -> Option<&'a mut V> {
        self.values.iter_mut().find_map(move |(k, v)| {
            if k == key {
                Some(v)
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Vector;
    use crate::container::tests::test_container;
    use crate::tests::test_building_block;

    #[test]
    fn building_block() {
        test_building_block(Vector::new(0));
        test_building_block(Vector::new(10));
        test_building_block(Vector::new(100));
    }

    #[test]
    fn container() {
        test_container(Vector::new(0));
        test_container(Vector::new(10));
        test_container(Vector::new(100));
    }
}
