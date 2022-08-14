use super::Multilevel;
use crate::BuildingBlock;

impl<'a, K, V, L, R> BuildingBlock<'a, K, V> for Multilevel<K, V, L, R>
where
    K: 'a,
    V: 'a,
    L: BuildingBlock<'a, K, V>,
    R: BuildingBlock<'a, K, V>,
{
    fn capacity(&self) -> usize {
        self.left.capacity() + self.right.capacity()
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(self.left.flush().chain(self.right.flush()))
    }

    /// The left side is looked first and if the key is not
    /// found, it is searched in the right side.
    fn contains(&self, key: &K) -> bool {
        if self.left.contains(key) {
            true
        } else {
            self.right.contains(key)
        }
    }

    fn count(&self) -> usize {
        self.left.count() + self.right.count()
    }

    /// Take the matching key/value pair out of the container.
    /// The left side is looked first and if the key is not
    /// found, it is searched in the right side.
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.left.take(key) {
            Some(x) => Some(x),
            None => self.right.take(key),
        }
    }

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the container,
    /// the returned vector contains all the container values and
    /// the container is left empty.
    /// Pop will remove values from the right side all it can.
    /// If there were less than `n` values in the right side,
    /// then more values from the left side are popped.
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut v = self.right.pop(n);

        if v.len() < n {
            v.append(&mut self.left.pop(n - v.len()));
        }
        v
    }

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, some values are returned.
    ///
    /// Push will make room (pop) from the left side to the right side to
    /// fit as many new `elements` as possible. If there is more elements
    /// than capacity in the left side, the left side is flushed to the
    /// right side. At this point, everything that overflows the right side
    /// will be returned.
    /// Once room has been made, `elements` are inserted to the left.
    /// If new elements pop in the process, they are inserted to the
    /// right side. If elements pop again on the right side, they are
    /// also returned.
    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let left_capacity = self.left.capacity();
        let left_count = self.left.count();

        let mut left_pop = if elements.len()
            <= (left_capacity - left_count)
        {
            Vec::new()
        } else if elements.len() <= left_capacity {
            let pop_count = elements.len() + left_count - left_capacity;
            self.left.pop(pop_count)
        } else {
            self.left.flush().collect()
        };
        let mut elements = self.left.push(elements);
        elements.append(&mut left_pop);

        if elements.is_empty() {
            return elements;
        }

        let right_capacity = self.right.capacity();
        let right_count = self.right.count();
        let mut right_pop = if elements.len()
            <= (right_capacity - right_count)
        {
            Vec::new()
        } else if elements.len() <= right_capacity {
            let pop_count = elements.len() + right_count - right_capacity;
            self.right.pop(pop_count)
        } else {
            self.right.flush().collect()
        };
        let mut elements = self.right.push(elements);
        elements.append(&mut right_pop);

        elements
    }
}

#[cfg(test)]
mod tests {
    use super::Multilevel;
    use crate::tests::test_building_block;
    use crate::Array;

    #[test]
    fn building_block() {
        test_building_block(Multilevel::new(Array::new(0), Array::new(0)));
        test_building_block(Multilevel::new(
            Array::new(0),
            Array::new(10),
        ));
        test_building_block(Multilevel::new(
            Array::new(10),
            Array::new(0),
        ));
        test_building_block(Multilevel::new(
            Array::new(10),
            Array::new(100),
        ));
    }
}
