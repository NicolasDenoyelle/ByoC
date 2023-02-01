use super::Associative;
use crate::BuildingBlock;
use std::hash::{Hash, Hasher};

impl<K, V, C, H> BuildingBlock<K, V> for Associative<C, H>
where
    K: Clone + Hash,
    V: Ord,
    C: BuildingBlock<K, V>,
    H: Hasher + Clone,
{
    /// Get the maximum storage size of this [`BuildingBlock`].
    ///
    /// This is the sum of the capacities of the containers that this
    /// [`Associative`] container is composed of.
    ///
    /// Note that this container may refuse new elements before being having
    /// `size` close to its `capacity` if their respective buckets/sets are
    /// full.
    fn capacity(&self) -> usize {
        self.containers.iter().map(|c| c.capacity()).sum()
    }

    type FlushIterator =
        std::iter::Flatten<std::vec::IntoIter<C::FlushIterator>>;
    fn flush(&mut self) -> Self::FlushIterator {
        self.containers
            .iter_mut()
            .map(|c| c.flush())
            .collect::<Vec<C::FlushIterator>>()
            .into_iter()
            .flatten()
    }

    /// Check if container contains a matching key.
    ///
    /// The key is first hashed to find out which bucket may contain the key.
    /// Then the method returns whether the matching container actually contains
    /// the key.
    fn contains(&self, key: &K) -> bool {
        let i = self.set(key.clone());
        self.containers[i].contains(key)
    }

    /// Get the size currently occupied by elements in this [`BuildingBlock`].
    ///
    /// This is the sum of the sizes of the containers that this
    /// [`Associative`] container is composed of.
    ///
    /// Note that this container may refuse new elements before being having
    /// `size` close to its `capacity` if their respective buckets/sets are
    /// full.
    fn size(&self) -> usize {
        self.containers.iter().map(|c| c.size()).sum()
    }

    /// Take the matching key/value pair out of the container.
    ///
    /// The key is first hashed to find out which bucket may contain the key.
    /// Then the method returns the result of
    /// [`take()`](trait.BuildingBlock.html#method.take) method on the
    /// matching container.
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        let i = self.set(key.clone());
        self.containers[i].take(key)
    }

    /// Take multiple keys out of a container at once.
    ///
    /// This method will first hash all the keys and sort them by index of their
    /// matching bucket.
    /// Then, for each bucket in sequential order, the result of
    /// [`take_multiple()`](trait.BuildingBlock.html#method.take_multiple)
    /// with the matching bucket keys is returned.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let mut ret = Vec::with_capacity(keys.len());

        // Rearrange keys per set.
        let mut set_keys: Vec<Vec<K>> =
            Vec::with_capacity(self.containers.len());
        for _ in 0..self.containers.len() {
            set_keys.push(Vec::with_capacity(keys.len()));
        }
        for k in keys.drain(0..keys.len()) {
            set_keys[self.set(k.clone())].push(k);
        }

        // Take from each bucket.
        for (c, keys) in
            self.containers.iter_mut().zip(set_keys.iter_mut())
        {
            if !keys.is_empty() {
                ret.append(&mut c.take_multiple(keys));
            }
        }

        // Put the remaining keys back in the input keys.
        for mut sk in set_keys.into_iter() {
            keys.append(&mut sk);
        }

        ret
    }

    /// Free up to `size` space from the container.
    ///
    /// If the container is empty or the requested size is `0`, an empty
    /// vector is returned.
    ///
    /// If less than `size` can be evicted out,
    /// the returned vector contains all the container values and
    /// the container is left empty.
    ///
    /// Else, this [`pop()`](trait.BuildingBlock.html#method.pop)
    /// method will pop elements from buckets with the goal of balancing
    /// the size of all the buckets. The kind of element popping
    /// out of buckets depends on the implementation of individual buckets
    /// [`pop()`](trait.BuildingBlock.html#method.pop) method.
    ///
    /// If the container is concurrently accessed this method will still
    /// attempt to pop the requested size elements. However, the method can't
    /// guarantee to achieve optimal bucket balancing.
    fn pop(&mut self, size: usize) -> Vec<(K, V)> {
        let mut victims = Vec::<(K, V)>::new();

        // Collect all buckets sizes and indexes.
        // In this variable we keep the buckets we intend to pop from and
        // their size.
        let mut popped_buckets: Vec<(usize, usize)> = self
            .containers
            .iter()
            .map(|c| c.size())
            .enumerate()
            .collect();

        // The total size of popped buckets.
        let mut popped_buckets_size =
            popped_buckets.iter().map(|(_, s)| s).sum();

        // Easy path, we don't need to go further if there is nothing to return.
        if size == 0 || popped_buckets_size == 0 {
            return victims;
        }

        // Easy path, we don't need to go further we need to return everything.
        if size >= popped_buckets_size {
            return self.flush().collect();
        }

        // Generic pop(size) scenario:
        //
        //  1   3   2   0  -- sorted buckets
        //             +-+
        //         +-+ | |
        //     +-+ | | | |
        // ------------------ average bucket size
        // ------------------ target average bucket size after pop.
        //     | | | | | |
        // +-+ | | | | | |
        // | | | | | | | |
        // We cannot pop from bucket `1` without increasing imbalance.
        // Instead we will need to pop below the
        // target average bucket size after pop` in other buckets.
        // Consequently, we need to recompute our goal of:
        // `target average bucket size after pop` without the buckets below
        // that value. We can process iteratively removing the smallest
        // bucket with a size below the target value at every step.

        // First, sort ! in reverse order to have the small buckets last.
        popped_buckets.sort_unstable_by(|(a, _), (b, _)| b.cmp(a));

        // Then loop through the buckets starting from the smallest (tail) and:
        let target_average_bucket_size = loop {
            // Compute the goal for buckets size:
            let target_average_bucket_size =
                (popped_buckets_size - size) / (popped_buckets.len() + 1);

            // Look if the smaller bucket size is greater than the goal.
            let (bucket_size, bucket_index) = popped_buckets
                .pop()
                .expect("Associative container pop() error.");

            // If it is greater, then we can stop and go to the next step.
            if bucket_size >= target_average_bucket_size {
                popped_buckets.push((bucket_size, bucket_index));
                break target_average_bucket_size;
            }

            // Else, we loop on what's left of the buckets.
            // The new `popped_buckets_size` of remaining bucket is the current
            // one minus the discarded bucket.
            popped_buckets_size -= bucket_size;

            // Since bucket_size < target_average_bucket_size,
            // there must be more than requested pop size in other buckets.
            // If this is not true, the next iteration of the loop will panic
            // on computing `popped_buckets_size - size`.
            assert!(
                popped_buckets_size > size,
                "Associative container pop() error."
            );
        };

        // Now all the buckets in `popped_buckets` vector have a size that is
        // greater than the `target_average_bucket_size`. We just have to pop()
        // the difference between their size and the target size from them.
        //
        //  3   2   0  -- remaining sorted buckets.
        //         +-+     +
        //     +-+ | |     |
        // +-+ | | | |     | size to pop      +
        // | | | | | |     | in last bucket.  | size to pop
        // | | | | | |     |                  | in first bucket.
        // | | | | | |     +                  +
        // -------------- average bucket size after pop
        // | | | | | |
        for (bucket_size, bucket_index) in popped_buckets.into_iter().rev()
        {
            let bucket = &mut self.containers[bucket_index];
            let pop_size = bucket_size - target_average_bucket_size;
            victims.append(&mut bucket.pop(pop_size));
        }

        victims
    }

    /// Insert key/value pairs in the container.
    ///
    /// Each key to insert is hashed and assigned to a bucket.
    /// Then for each bucket where a there is at least one key to insert,
    /// the bucket [`push()`](trait.BuildingBlock.html#method.push) invoked
    /// with the associated keys and values to insert. It is up to this
    /// container to choose what will be inserted and what may be evicted.
    ///
    /// Note that this container may refuse new elements before being having
    /// its `size` close to its `capacity` if for instance most key are assigned
    /// to a full bucket while other buckets still have room.
    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let n = elements.len();
        let n_sets = self.containers.len();
        let mut set_elements: Vec<Vec<(K, V)>> =
            Vec::with_capacity(n_sets);
        for _ in 0..n_sets {
            set_elements.push(Vec::with_capacity(n));
        }
        for e in elements.into_iter() {
            set_elements[self.set(e.0.clone())].push(e);
        }

        let mut out = Vec::with_capacity(n);
        for (i, v) in set_elements.into_iter().enumerate() {
            out.append(&mut (self.containers[i].push(v)));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::Associative;
    use crate::tests::test_building_block;
    use crate::Array;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn building_block() {
        test_building_block(
            Associative::new(
                vec![Array::new(5); 10],
                DefaultHasher::new(),
            ),
            true,
        );
    }
}
