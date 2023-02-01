use super::Exclusive;
use crate::BuildingBlock;

impl<K, V, L, R> BuildingBlock<K, V> for Exclusive<K, V, L, R>
where
    K: Ord,
    L: BuildingBlock<K, V>,
    R: BuildingBlock<K, V>,
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

    type FlushIterator = std::iter::Chain<
        R::FlushIterator,
        std::iter::Chain<std::vec::IntoIter<(K, V)>, R::FlushIterator>,
    >;

    /// Empty the container and retrieve all of its elements.
    ///
    /// This method moves elements towards the end of the container and flushes
    /// them from the back of the container as follow.
    ///
    /// 1. This method flushes first the back container.
    ///
    /// 2. Then elements from the front container are flushed and pushed in the
    /// back container. Popping element are chained to the flushed elements in
    /// step 1.
    ///
    /// 3. Elements in the back are flushed and chained to elements from the two
    /// former steps.
    ///
    ///
    /// This is conveniently composable with
    /// [`FlushStopper`](struct.FlushStopper.html) building block.
    /// When composed together, the new container `flush()` method pushes
    /// elements from the front container to the back and returns any popping
    /// element.
    fn flush(&mut self) -> Self::FlushIterator {
        let back = self.back.flush();
        let front = self.front.flush().collect();
        let front = self.back.push(front).into_iter();
        let new_back = self.back.flush();
        back.chain(front.chain(new_back))
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
