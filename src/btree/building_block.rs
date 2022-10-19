use super::BTree;
use crate::BuildingBlock;
use std::collections::BTreeMap;
use std::rc::Rc;

impl<'a, K, V> BuildingBlock<'a, K, V> for BTree<K, V>
where
    K: 'a + Copy + Ord,
    V: 'a + Ord,
{
    /// Get the maximum "size" that elements in the container can fit.
    ///
    /// This is the size set by the constructor
    /// [`BTree::new()`](struct.BTree.html#method.new).
    /// The meaning of this methods depends on the meaning of the
    /// elements size that can be set with the method
    /// [`with_element_size()`](struct.struct..html#method.with_element_size).
    /// For instance, capacity can be the number of elements in the BTree
    /// when all elements size is one, or it can be the maximum stack
    /// size when elements size is the size of the element on the stack.
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn size(&self) -> usize {
        self.total_size
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        let mut elements = BTreeMap::new();
        std::mem::swap(&mut elements, &mut self.map);
        self.set.clear();
        self.total_size = 0;

        Box::new(
            elements.into_iter().map(|(k, rc)| (k, Self::as_value(rc))),
        )
    }

    fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, the last input values not fitting in are
    /// returned. This container does not accept keys matching keys
    /// already inside the container and will return the corresponding
    /// key/value pairs.
    fn push(&mut self, mut elements: Vec<(K, V)>) -> Vec<(K, V)> {
        // Find out if we have too many elements and where to split them.
        let mut split = 0;
        let mut size = 0;
        for (i, s) in elements
            .iter()
            .map(|(k, v)| (self.element_size)(k, v))
            .enumerate()
        {
            if size + s > self.capacity {
                break;
            }
            split = i + 1;
            size += s;
        }

        // Remove extra elements from the container.
        let mut out: Vec<(K, V)> = if split < elements.len() {
            // Here there is more elements to insert than the container
            // capacity.

            // Remove extra elements from input.
            let mut out = elements.split_off(split);
            // Clear container.
            let mut map = BTreeMap::<K, Rc<V>>::new();
            std::mem::swap(&mut self.map, &mut map);
            self.set.clear();

            // Move all elements that were in the container in the returned
            // vector.
            for (k, rc) in map.into_iter() {
                match Rc::<V>::try_unwrap(rc) {
                    Ok(v) => out.push((k, v)),
                    Err(_) => panic!(
                        "Looks like we forgot to delete an Rc value."
                    ),
                }
            }

            // The container is cleared entirely.
            self.total_size = 0;
            out
        } else if size + self.total_size > self.capacity {
            // Pop out the extra size. The container size is updated.
            self.pop(size + self.total_size - self.capacity)
        } else {
            // Nothing to pop or remove from input elements.
            Vec::new()
        };

        // Insert new elements in the container.
        for (k, v) in elements.into_iter() {
            let value = Rc::new(v);
            if self.set.insert((Rc::clone(&value), k)) {
                self.map.insert(k, value);
            } else {
                match Rc::<V>::try_unwrap(value) {
                    Ok(v) => out.push((k, v)),
                    Err(_) => panic!(
                        "Looks like we forgot to delete an Rc value."
                    ),
                }
            }
        }

        // Update size.
        self.total_size += size;
        out
    }

    /// Remove up to `n` size values from the container.
    /// If less than `n` size is occupied by values in the container,
    /// the returned vector contains all the container values and
    /// the container is left empty.
    /// This building block implements the trait
    /// [`Ordered`](../trait.Ordered.html), which means that
    /// the highest values are popped out. This is implemented by
    /// retrieving the last values stored in a binary tree.
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        // List keys to evict.
        let mut size = 0;
        let mut split: Option<(Rc<V>, K)> = None;
        for (rc, k) in self.set.iter().rev() {
            size += (self.element_size)(k, rc.as_ref());
            if size >= n {
                split = Some((rc.clone(), *k));
                break;
            }
        }

        match split {
            None => self.flush().collect(),
            Some(s) => {
                self.total_size -= size;
                let out_set = self.set.split_off(&s);
                // Drop Rc<V> clone to avoid panicking when unwrapping the value
                drop(s);
                let mut out = Vec::with_capacity(out_set.len());
                for (rc, k) in out_set.into_iter() {
                    // Drop Rc<V> clone to avoid panicking when unwrapping the
                    // value below
                    drop(rc);
                    let rc = self.map.remove(&k).unwrap();
                    out.push((k, Self::as_value(rc)));
                }
                out
            }
        }
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.map.remove(key) {
            None => None,
            Some(rc) => {
                self.total_size -= (self.element_size)(key, rc.as_ref());
                self.set.remove(&(rc.clone(), *key));
                Some((*key, Self::as_value(rc)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BTree;
    use crate::tests::{test_building_block, TestElement};

    #[test]
    fn building_block_default() {
        test_building_block(BTree::new(0), true);
        test_building_block(BTree::new(10), true);
        test_building_block(BTree::new(100), true);
    }

    #[test]
    fn building_block_stack_size() {
        test_building_block(
            BTree::new(0).with_element_size(|_, _| {
                std::mem::size_of::<TestElement>()
            }),
            true,
        );
        test_building_block(
            BTree::new(10).with_element_size(|_, _| {
                std::mem::size_of::<TestElement>()
            }),
            true,
        );
        test_building_block(
            BTree::new(100).with_element_size(|_, _| {
                std::mem::size_of::<TestElement>()
            }),
            true,
        );
    }
}
