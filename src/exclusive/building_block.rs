use super::Exclusive;
use crate::BuildingBlock;

impl<'a, K, V, L, R> BuildingBlock<'a, K, V> for Exclusive<'a, K, V, L, R>
where
    K: 'a + Ord,
    V: 'a,
    L: BuildingBlock<'a, K, V>,
    R: BuildingBlock<'a, K, V>,
{
    /// Get the maximum "size" that elements in the container can fit.
    ///
    /// This is the sum of the capacities of the two containers that this
    /// [`Exclusive`] container is composed of.
    fn capacity(&self) -> usize {
        self.front.capacity() + self.back.capacity()
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(self.front.flush().chain(self.back.flush()))
    }

    /// The front is looked first and if the key is not
    /// found, it is searched in the back.
    fn contains(&self, key: &K) -> bool {
        self.front.contains(key) || self.back.contains(key)
    }

    fn size(&self) -> usize {
        self.front.size() + self.back.size()
    }

    /// Take the matching key/value pair out of the container.
    /// The front is looked first and if the key is not
    /// found, it is searched in the back.
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.front.take(key) {
            Some(x) => Some(x),
            None => self.back.take(key),
        }
    }

    /// This method will take matching keys on the front then on
    /// the back.
    /// Matching keys found on the front are not searched on the back
    /// side.
    /// Input `keys` is updated as a side effect to contain
    /// only non matching keys.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        keys.sort();

        let mut front = self.front.take_multiple(keys);

        // Remove matches from keys before querying on the back.
        for (k, _) in front.iter() {
            if let Ok(i) = keys.binary_search(k) {
                keys.remove(i);
            }
        }

        let mut back = self.back.take_multiple(keys);

        // Remove matching keys in case these keys are used in other
        // calls to take_multiple.
        for (k, _) in back.iter() {
            if let Ok(i) = keys.binary_search(k) {
                keys.remove(i);
            }
        }

        // Return final matches.
        front.append(&mut back);
        front
    }

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the container,
    /// the returned vector contains all the container values and
    /// the container is front empty.
    /// Pop will remove values from the back all it can.
    /// If there were less than `n` values in the back,
    /// then more values from the front are popped.
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut v = self.back.pop(n);

        if v.len() < n {
            v.append(&mut self.front.pop(n - v.len()));
        }
        v
    }

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, some values are returned.
    ///
    /// Push will make room (pop) from the front to the back to
    /// fit as many new `elements` as possible. If there is more elements
    /// than capacity in the front, the front is flushed to the
    /// back. At this point, everything that overflows the back
    /// will be returned.
    /// Once room has been made, `elements` are inserted to the front.
    /// If new elements pop in the process, they are inserted to the
    /// back. If elements pop again on the back, they are
    /// also returned.
    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let front_capacity = self.front.capacity();
        let front_count = self.front.size();

        let mut front_pop = if elements.len()
            <= (front_capacity - front_count)
        {
            Vec::new()
        } else if elements.len() <= front_capacity {
            let pop_count = elements.len() + front_count - front_capacity;
            self.front.pop(pop_count)
        } else {
            self.front.flush().collect()
        };
        let mut elements = self.front.push(elements);
        elements.append(&mut front_pop);

        if elements.is_empty() {
            return elements;
        }

        let back_capacity = self.back.capacity();
        let back_count = self.back.size();
        let mut back_pop = if elements.len()
            <= (back_capacity - back_count)
        {
            Vec::new()
        } else if elements.len() <= back_capacity {
            let pop_count = elements.len() + back_count - back_capacity;
            self.back.pop(pop_count)
        } else {
            self.back.flush().collect()
        };
        let mut elements = self.back.push(elements);
        elements.append(&mut back_pop);

        elements
    }
}

#[cfg(test)]
mod tests {
    use super::Exclusive;
    use crate::tests::test_building_block;
    use crate::Array;

    #[test]
    fn building_block() {
        test_building_block(
            Exclusive::new(Array::new(0), Array::new(0)),
            true,
        );
        test_building_block(
            Exclusive::new(Array::new(0), Array::new(10)),
            true,
        );
        test_building_block(
            Exclusive::new(Array::new(10), Array::new(0)),
            true,
        );
        test_building_block(
            Exclusive::new(Array::new(10), Array::new(100)),
            true,
        );
    }
}
