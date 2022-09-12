use super::BTree;
use crate::BuildingBlock;
use std::rc::Rc;

impl<'a, K, V> BuildingBlock<'a, K, V> for BTree<K, V>
where
    K: 'a + Copy + Ord,
    V: 'a + Ord,
{
    fn capacity(&self) -> usize {
        self.capacity * Self::element_size()
    }

    fn size(&self) -> usize {
        self.references.len() * Self::element_size()
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        self.map.clear();
        self.set.clear();
        Box::new(
            self.references
                .drain(..)
                .map(|(k, v)| {
                    (k, Rc::try_unwrap(v).map_err(|_| panic!("")).unwrap())
                })
                .collect::<Vec<(K, V)>>()
                .into_iter(),
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
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)> {
        let mut out = Vec::new();
        let mut n = self.references.len();

        for (key, value) in values.into_iter() {
            if n >= self.capacity {
                out.push((key, value));
            } else if self.map.get(&key).is_some() {
                out.push((key, value))
            } else {
                let value = Rc::new(value);
                let _value = Rc::clone(&value);
                self.references.push((key, value));
                assert!(self.map.insert(key, n).is_none());
                assert!(self.set.insert((_value, key)));
                n += 1;
            }
        }
        out
    }

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the container,
    /// the returned vector contains all the container values and
    /// the container is left empty.
    /// This building block implements the trait
    /// [`Ordered`](../trait.Ordered.html), which means that
    /// the highest values are popped out. This is implemented by
    /// retrieving the last values stored in a binary tree.
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut ret = Vec::new();

        for _ in 0..n {
            let k = match self.set.iter().rev().next() {
                None => break,
                Some((_, k)) => *k,
            };

            let n = self.references.len() - 1;
            let j = self.map.remove(&k).unwrap();
            assert!(self
                .set
                .remove(&(Rc::clone(&self.references[j].1), k)));

            // If we don't remove the last element, we need to update the
            // position of the element it is swapped with.
            if j != n {
                let (k, _) = self
                    .references
                    .iter()
                    .rev()
                    .map(|(k, r)| (*k, Rc::clone(r)))
                    .next()
                    .unwrap();
                self.map.insert(k, j);
            }

            let (k, r) = self.references.swap_remove(j);
            ret.push((
                k,
                Rc::try_unwrap(r).map_err(|_| panic!("")).unwrap(),
            ));
        }
        ret
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.map.remove(key) {
            None => None,
            Some(i) => {
                let n = self.references.len() - 1;
                assert!(self
                    .set
                    .remove(&(Rc::clone(&self.references[i].1), *key)));
                if i != n {
                    let (k, _) = self
                        .references
                        .iter()
                        .rev()
                        .map(|(k, r)| (*k, Rc::clone(r)))
                        .next()
                        .unwrap();
                    self.map.insert(k, i);
                }
                let (k, r) = self.references.swap_remove(i);
                Some((
                    k,
                    Rc::try_unwrap(r).map_err(|_| panic!("")).unwrap(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BTree;
    use crate::tests::test_building_block;

    #[test]
    fn building_block() {
        test_building_block(BTree::new(0));
        test_building_block(BTree::new(10));
        test_building_block(BTree::new(100));
    }
}
