use super::Batch;
use crate::BuildingBlock;
use std::collections::LinkedList;

impl<K, V, C> BuildingBlock<K, V> for Batch<C>
where
    V: Ord,
    C: BuildingBlock<K, V>,
{
    /// Get the maximum storage size of this [`BuildingBlock`].
    ///
    /// This is the sum of the capacities of the containers that this
    /// [`Batch`] container is composed of.
    fn capacity(&self) -> usize {
        self.bb.iter().map(|c| c.capacity()).sum()
    }

    /// Get the size currently occupied by elements in this [`BuildingBlock`].
    ///    
    /// This is the sum of the sizes of the containers that this
    /// [`Batch`] container is composed of.
    fn size(&self) -> usize {
        self.bb.iter().map(|c| c.size()).sum()
    }

    fn contains(&self, key: &K) -> bool {
        self.bb.iter().any(|c| c.contains(key))
    }

    /// Take the matching key/value pair out of the container.
    ///
    /// This method iterates through all batches and stop at the first batch
    /// with a matching key. If multiple batches contain a matching key, the
    /// first match is returned.
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        self.bb.iter_mut().find_map(|c| c.take(key))
    }

    /// Take multiple keys out of a container at once.
    ///
    /// This method will iterate batches and call each batch
    /// [`take_multiple()`](trait.BuildingBlock.html#tymethod.take_multiple)
    /// method. This method expect that matched keys will be taken out of the
    /// input `keys` vector. Therefore, if at one point during the iteration
    /// the input `keys` vector is emptied, the method will stop and return
    /// all the matched keys.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let mut out = Vec::with_capacity(keys.len());
        for c in self.bb.iter_mut() {
            if keys.is_empty() {
                break;
            }
            out.append(&mut c.take_multiple(keys))
        }
        out
    }

    /// Free up to `size` space from the container.
    ///
    /// The method will call
    /// [`pop(size)`](trait.BuildingBlock.html#tymethod.pop)
    /// method to free up to `size` space from batches in a round-robin fahsion.
    /// If the popped size from a batch is less than the size to pop, it is
    /// subtracted from the size to pop and the method is applied to the next
    /// batch with the new `size` value until either `size` is 0 or all the
    /// batches have been emptied.
    fn pop(&mut self, mut size: usize) -> Vec<(K, V)> {
        let mut out = Vec::new();

        // We iterate in reverse order because the push() method puts
        // the most recently filled batches in the back.
        for bb in self.bb.iter_mut().rev() {
            let bb_size = bb.size();
            out.append(&mut bb.pop(size));
            let new_bb_size = bb.size();
            let popped_size = bb_size - new_bb_size;
            if popped_size >= size {
                break;
            }
            size -= popped_size;
        }
        out
    }

    /// Insert key/value pairs in the container.
    ///
    /// This method will try to [push](trait.BuildingBlock.html#tymethod.push)
    /// `values` in all batches in a round-robin fashion. Elements evicted by
    /// a batch are attempted to be
    /// [pushed](trait.BuildingBlock.html#tymethod.push) to the next batch.
    /// If at one point during the iteration no element is evicted, the
    /// iteration stops and an empty `Vec` is returned. Else, the last evicted
    /// elements are returned.
    ///
    /// Since batches are filled from the front, newly filled batches are
    /// rotated to the back of the batch list such that next pushes will
    /// likely start with non-full batches.
    fn push(&mut self, mut values: Vec<(K, V)>) -> Vec<(K, V)> {
        let mut bb = LinkedList::new();
        loop {
            if values.is_empty() {
                break;
            }
            let mut c = match self.bb.pop_front() {
                None => break,
                Some(c) => c,
            };
            values = c.push(values);
            bb.push_back(c);
        }
        self.bb.append(&mut bb);
        values
    }

    type FlushIterator =
        std::iter::Flatten<std::vec::IntoIter<C::FlushIterator>>;
    fn flush(&mut self) -> Self::FlushIterator {
        self.bb
            .iter_mut()
            .map(|c| c.flush())
            .collect::<Vec<C::FlushIterator>>()
            .into_iter()
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::Batch;
    use crate::tests::test_building_block;
    use crate::Array;

    #[test]
    fn building_block() {
        test_building_block(Batch::<Array<(u16, u32)>>::new(), true);
        test_building_block(Batch::from([Array::new(0)]), true);
        test_building_block(
            Batch::from([Array::new(0), Array::new(0)]),
            true,
        );
        test_building_block(
            Batch::from([Array::new(0), Array::new(10)]),
            true,
        );
        test_building_block(
            Batch::from([Array::new(10), Array::new(0)]),
            true,
        );
        test_building_block(
            Batch::from([Array::new(10), Array::new(10)]),
            true,
        );
    }
}
