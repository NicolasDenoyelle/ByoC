use super::inclusive::InclusiveCell;
use super::Inclusive;
use crate::BuildingBlock;

impl<K, V, L, R> BuildingBlock<K, V> for Inclusive<K, V, L, R>
where
    K: Clone,
    V: Clone,
    L: BuildingBlock<K, InclusiveCell<V>>,
    R: BuildingBlock<K, InclusiveCell<V>>,
{
    /// Get the maximum "size" that elements in the container can fit.
    ///
    /// This is the capacity of the `back` container of this [`Inclusive`]
    /// [`BuildingBlock`] since any element contained in the `front`
    /// container is also contained in the `back` container.
    fn capacity(&self) -> usize {
        self.back.capacity()
    }

    /// Get the size currently occupied by elements in this [`BuildingBlock`].
    ///
    /// This is the size of the `back` container of this [`Inclusive`]
    /// [`BuildingBlock`] since any element contained in the `front`
    /// container is also contained in the `back` container.
    fn size(&self) -> usize {
        self.back.size()
    }

    /// Check if container contains a matching key.
    ///
    /// This method first looks for a matching key in the `front` container.
    /// Only if no matching key is found, then the `back` container is searched
    /// for a matching key too.
    fn contains(&self, key: &K) -> bool {
        self.front.contains(key) || self.back.contains(key)
    }

    /// Take the matching key/value pair out of the container.
    ///
    /// If the key to search is in the container, it will necessarily be in the
    /// back part of the container and will need to be removed there. Therefore,
    /// the back container is searched for the key first using its own
    /// its own [`take()`](trait.BuildingBlock.html#method.take) method.
    /// If it is not found there, then it is not in the container.
    /// If it is found there, it may also be present in the front.
    /// The values in the back container carry a flag that indicates whether
    /// a copy exists in the front container. If the flag of the found
    /// element is set, then it is also taken out of the front container using
    /// the latter [`take()`](trait.BuildingBlock.html#method.take) method.
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.back.take(key) {
            None => None,
            Some((k, c)) => {
                if c.is_cloned() {
                    Some(
                        self.front
                            .take(key)
                            .map(|(k, c)| (k, c.unwrap()))
                            .unwrap(),
                    )
                } else {
                    Some((k, c.unwrap()))
                }
            }
        }
    }

    /// Take multiple keys out of a container at once.
    ///
    /// Similarly to the [`take()`](struct.Inclusive.html#method.take) method,
    /// The back container is searched first using its own
    /// [`take_multiple()`](trait.BuildingBlock.html#method.take_multiple)
    /// method. Any key that was not found there will not be found in the front
    /// container either. Found elements are iterated. All the values from
    /// key/value elements where the value carries the flag indicating it is
    /// cloned in the front are also taken out of the front container using its
    /// own [`take_multiple()`](trait.BuildingBlock.html#method.take_multiple)
    /// method. The found key/value pairs in the front and the not cloned
    /// key/value pairs found in the back are returned.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        // First we remove matching keys from the back.
        let back: Vec<(K, InclusiveCell<V>)> =
            self.back.take_multiple(keys);

        // Then we take clones in the front too.
        let mut front_keys: Vec<K> =
            back.iter()
                .filter_map(|(k, c)| {
                    if c.is_cloned() {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();
        let front = self.front.take_multiple(&mut front_keys);

        // Finally we remove outdated elements from the back and chain the
        // remaining ones to elements from the front.
        back.into_iter()
            .filter(|(_, c)| !c.is_cloned())
            .chain(front.into_iter())
            .map(|(k, c)| (k, c.unwrap()))
            .collect()
    }

    /// Free up to `size` space from the container.
    ///
    /// This method will first attempt to pop `size` from the back container
    /// using its own its own [`pop()`](trait.BuildingBlock.html#method.pop)
    /// method. Key/value pairs popped out are iterated. If the value carries
    /// the flag indicating it is cloned in the front, then it will be taken out
    /// of the front container and will be replaced by the front value in the
    /// returned vector of victims.
    fn pop(&mut self, size: usize) -> Vec<(K, V)> {
        // We only pop values in the back.
        // This is because the meaning of `size` may not be the same at
        // the front and at the back and there is no uniform method to obtain
        // the size of elements in each.
        let out = self.back.pop(size);

        // Values popped in the back need to be removed from the front too.
        // We return the front values rather than the back ones because they
        // might be fresher.
        out.into_iter()
            .map(|(k, c)| {
                if c.is_cloned() {
                    self.front
                        .take(&k)
                        .map(|(k, c)| (k, c.unwrap()))
                        .unwrap()
                } else {
                    (k, c.unwrap())
                }
            })
            .collect()
    }

    /// Insert key/value pairs in the container.
    ///
    /// Elements are always inserted in the `back` container.
    /// Any element popping from the `back` in the operation is also removed
    /// from the `front`. Newly inserted elements are not inserted at the
    /// `front`. They will move there only when they are accessed with the
    /// [`Get`](trait.Get.html) and [`GetMut`](trait.GetMut.html) traits.
    ///
    /// See also [`BuildingBlock`]
    /// [`push()` method](trait.BuildingBlock.html#method.push) documentation.
    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        // Wrap elements into a cell with metadata.
        let elements = elements
            .into_iter()
            .map(|(k, v)| (k, InclusiveCell::new(v)))
            .collect();

        // Push to the back and save what pops out.
        let back_pop = self.back.push(elements);

        // Enumerate matching keys that need to be removed at the front.
        let mut front_keys =
            back_pop
                .iter()
                .filter_map(|(k, c)| {
                    if c.is_cloned() {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();

        // Iterate elements that are removed from the back.
        let back_elements = back_pop.into_iter().filter_map(|(k, c)| {
            if c.is_cloned() {
                None
            } else {
                Some((k, c.unwrap()))
            }
        });

        // Iterate elements that are removed from the front.
        let front_elements = self
            .front
            .take_multiple(&mut front_keys)
            .into_iter()
            .map(|(k, c)| (k, c.unwrap()));

        back_elements.chain(front_elements).collect()
    }

    type FlushIterator = std::iter::Map<
        R::FlushIterator,
        fn((K, InclusiveCell<V>)) -> (K, V),
    >;

    /// Empty the container and retrieve all of its elements.
    ///
    /// This function will flush the front container and replace updated
    /// element in the back with the their updated copies in from front.
    /// Then it will return elements from the back using the back flush method.
    ///
    /// This is conveniently composable with
    /// [`FlushStopper`](struct.FlushStopper.html) building block.
    /// When composed together, the new container `flush()` method empties
    /// the front container and updates elements in the back container if any
    /// of its elements is updated.
    fn flush(&mut self) -> Self::FlushIterator {
        let front: Vec<(K, InclusiveCell<V>)> = self
            .front
            .flush()
            .filter_map(|(k, c)| {
                if c.is_updated() {
                    Some((k, InclusiveCell::new(c.unwrap())))
                } else {
                    None
                }
            })
            .collect();

        let mut keys: Vec<K> =
            front.iter().map(|(k, _)| k.clone()).collect();
        self.back.take_multiple(&mut keys);
        assert_eq!(self.back.push(front).len(), 0);
        self.back.flush().map(|(k, c)| (k, c.unwrap()))
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
            Inclusive::new(Array::new(10), Array::new(100)),
            true,
        );
    }
}
