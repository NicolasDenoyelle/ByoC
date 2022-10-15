use super::inclusive::InclusiveCell;
use super::Inclusive;
use crate::BuildingBlock;

impl<'a, K, V, L, R> BuildingBlock<'a, K, V> for Inclusive<'a, K, V, L, R>
where
    K: 'a + Clone + Ord,
    V: 'a + Clone,
    L: BuildingBlock<'a, K, InclusiveCell<V>>,
    R: BuildingBlock<'a, K, InclusiveCell<V>>,
{
    /// Get the maximum "size" that elements in the container can fit.
    ///
    /// This is the sum of the capacities of the two containers that this
    /// [`Inclusive`] container is composed of.
    fn capacity(&self) -> usize {
        self.front.capacity() + self.back.capacity()
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        // Flush front and filter out element that are the same in the back.
        let front: Vec<(K, InclusiveCell<V>)> = self
            .front
            .flush()
            .filter_map(|(k, c)| match (c.is_cloned(), c.is_updated()) {
                (true, false) => None,
                _ => Some((k, c)),
            })
            .collect();

        // Remove keys in the back that are also in the front.
        let mut front_keys =
            front
                .iter()
                .filter_map(|(k, c)| {
                    if c.is_cloned() {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();
        drop(self.back.take_multiple(&mut front_keys));
        drop(front_keys);

        // Piggy back back to front.
        let front = front.into_iter().map(|(k, c)| (k, c.unwrap()));
        let back = self.back.flush().map(|(k, c)| (k, c.unwrap()));
        Box::new(front.chain(back))
    }

    /// The front is looked first and if the key is not
    /// found, it is searched in the back.
    fn contains(&self, key: &K) -> bool {
        self.front.contains(key) || self.back.contains(key)
    }

    fn size(&self) -> usize {
        self.front.size() + self.back.size()
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.front.take(key) {
            None => self.back.take(key).map(|(k, c)| (k, c.unwrap())),
            Some((k, c)) => {
                if c.is_cloned() {
                    drop(self.back.take(key));
                }
                Some((k, c.unwrap()))
            }
        }
    }

    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        // First we remove matching keys from the front.
        let mut front: Vec<(K, InclusiveCell<V>)> =
            self.front.take_multiple(keys);

        // We remove matching keys from the front out of the back.
        drop(self.back.take_multiple(
            &mut front.iter().map(|(k, _)| k.clone()).collect(),
        ));

        // We take remaining matching keys from the back.
        front.append(&mut self.back.take_multiple(keys));
        front.into_iter().map(|(k, c)| (k, c.unwrap())).collect()
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        // We pop from the back. If a value has been cloned to the front,
        // we have to make sure it is removed there too. If it is there,
        // we rather return the value there because it might be fresher.
        let old_back_size = self.back.size();
        let mut out: Vec<(K, V)> = self
            .back
            .pop(n)
            .into_iter()
            .filter_map(|(k, c)| {
                if !c.is_cloned() {
                    Some((k, c.unwrap()))
                } else if self.front.contains(&k) {
                    // The room is freed.
                    // However, we don't return the element because a
                    // fresher copy is in the front container.
                    None
                } else {
                    Some((k, c.unwrap()))
                }
            })
            .collect();

        // If we were not able to pop enough elements from the back,
        // we pop remaining requested size from the front.
        let remaining = n - (old_back_size - self.back.size());
        if remaining > 0 {
            let mut front: Vec<(K, V)> = self
                .front
                .pop(remaining)
                .into_iter()
                .map(|(k, c)| (k, c.unwrap()))
                .collect();
            out.append(&mut front);
        }
        out
    }

    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        // Wrap elements into a cell with metadata.
        let elements = elements
            .into_iter()
            .map(|(k, v)| (k, InclusiveCell::new(v)))
            .collect();

        // Insert element at the front and filter evicted elements to keep the
        // one that need to be inserted at the back.
        let mut outdated = Vec::<K>::new();
        let popped = self
            .front
            .push(elements)
            .into_iter()
            .filter_map(|(k, c)| match (c.is_cloned(), c.is_updated()) {
                (true, false) => None,
                (true, true) => {
                    outdated.push(k.clone());
                    Some((k, c))
                }
                _ => Some((k, c)),
            })
            .collect();

        // Remove outdated element in the back.
        drop(self.back.take_multiple(&mut outdated));

        // Push elements evicted from the front and unwrap popped victims.
        self.back
            .push(popped)
            .into_iter()
            .map(|(k, c)| (k, c.unwrap()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Inclusive;
    use crate::tests::test_building_block;
    use crate::Array;

    #[test]
    fn building_block() {
        test_building_block(
            Inclusive::new(Array::new(0), Array::new(0)),
            true,
        );
        test_building_block(
            Inclusive::new(Array::new(0), Array::new(10)),
            true,
        );
        test_building_block(
            Inclusive::new(Array::new(10), Array::new(0)),
            true,
        );
        test_building_block(
            Inclusive::new(Array::new(10), Array::new(100)),
            true,
        );
    }
}
