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

    /// Get the size currently occupied by elements in this [`BuildingBlock`].
    ///
    /// This is the sum of the sizes held in the two containers that this
    /// [`Exclusive`] container is composed of.
    fn size(&self) -> usize {
        self.front.size() + self.back.size()
    }

    /// Check if container contains a matching key.
    ///
    /// This method will lookup the front container first.
    /// If the the key is not found, then only it is searched in the back
    /// container.
    fn contains(&self, key: &K) -> bool {
        self.front.contains(key) || self.back.contains(key)
    }

    /// Take the matching key/value pair out of the container.
    ///
    /// This method will lookup the front container first.
    /// If the the key is not found, then only it is searched in the back
    /// container.
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.front.take(key) {
            Some(x) => Some(x),
            None => self.back.take(key),
        }
    }

    /// Take multiple keys out of a container at once.
    ///
    /// Input `keys` are first sorted before matching keys are taken from the
    /// front container. The keys taken out are then searched and removed from
    /// the input `keys`. If some keys are left in the input `keys` vector, they
    /// are attemptively taken out of the back container. Keys that were
    /// successfully taken out of the back container are also removed from the
    /// input `keys`. Finally, all the key/value pairs that were found either
    /// in the front or the back container are returned in a vector.
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

    /// Free up to `size` space from the container.
    ///
    /// This function first attempts to free `size` space from the back
    /// container with its own [`pop()`](trait.BuildingBlock.html#method.pop)
    /// method. If less than `size` space was successfully freed, then
    /// the remaining size to free is popped from the front container also with
    /// its own [`pop()`](trait.BuildingBlock.html#method.pop) method.
    ///
    /// If less than `size` elements were stored in the container,
    /// the returned vector will contain all the container values and
    /// the container will be left empty.
    fn pop(&mut self, size: usize) -> Vec<(K, V)> {
        let old_size = self.back.size();
        let mut v = self.back.pop(size);
        let evicted_size = old_size - self.back.size();

        if evicted_size < size {
            v.append(&mut self.front.pop(size - evicted_size));
        }
        v
    }

    /// Insert key/value pairs in the container.
    ///
    /// This function pushes the input `elements` to the front container with
    /// its own [`push()`](trait.BuildingBlock.html#method.push) method and will
    /// push all returned elements to the back container, also using its own
    /// [`push()`](trait.BuildingBlock.html#method.push) method.
    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        self.back.push(self.front.push(elements))
    }

    /// Empty the container and retrieve all of its elements.
    ///
    /// This method flushes first the front container and then the back
    /// container into a chained iterator.
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(self.front.flush().chain(self.back.flush()))
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
