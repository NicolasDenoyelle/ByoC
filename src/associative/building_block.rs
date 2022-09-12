use super::Associative;
use crate::BuildingBlock;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

impl<'a, K, V, C, H> BuildingBlock<'a, K, V> for Associative<C, H>
where
    K: 'a + Clone + Hash,
    V: 'a + Ord,
    C: BuildingBlock<'a, K, V>,
    H: Hasher + Clone,
{
    fn capacity(&self) -> usize {
        self.containers.iter().map(|c| c.capacity()).sum()
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(
            self.containers
                .iter_mut()
                .map(|c| c.flush())
                .collect::<Vec<Box<dyn Iterator<Item = (K, V)> + 'a>>>()
                .into_iter()
                .flatten(),
        )
    }

    fn contains(&self, key: &K) -> bool {
        let i = self.set(key.clone());
        self.containers[i].contains(key)
    }

    fn size(&self) -> usize {
        self.containers.iter().map(|c| c.size()).sum()
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        let i = self.set(key.clone());
        self.containers[i].take(key)
    }

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
            ret.append(&mut c.take_multiple(keys));
        }

        // Put the remaining keys back in the input keys.
        for mut sk in set_keys.into_iter() {
            keys.append(&mut sk);
        }

        ret
    }

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the container,
    /// the returned vector contains all the container values and
    /// the container is left empty.
    /// This pop method will pop elements from buckets so as to balance
    /// the amount of elements in each bucket. The kind of element popping
    /// out of buckets depends on the implementation of buckets `pop()`
    /// method.
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut victims = Vec::<(K, V)>::new();
        if n == 0 || self.capacity() == 0 {
            return victims;
        }
        victims.reserve(n);

        let n_sets = self.containers.len();

        // Collect all buckets element count.
        // We acquire exclusive lock on buckets in the process.
        let mut counts = Vec::<(usize, usize)>::with_capacity(n_sets + 1);
        for i in 0..n_sets {
            let n = self.containers[i].size();
            counts.push((n, i));
        }

        let mut total_count: usize = counts.iter().map(|(n, _)| n).sum();

        // If there is more elements to pop than elements available
        // Then we pop everything.
        if total_count <= n {
            for (_, i) in counts.into_iter() {
                victims.append(&mut self.containers[i].flush().collect())
            }
            return victims;
        }

        // Sort counts in descending order.
        counts.sort_unstable_by(|(a, _), (b, _)| match a.cmp(b) {
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Less,
        });

        // The amount of elements in each popping bucket after pop.
        let target_count = loop {
            // This the average number of elements per bucket after pop.
            let target_count = (total_count - n) / counts.len();
            // Last popped bucket. If it does not change after below loop,
            // we return above target_count.
            let prev_i = counts[counts.len() - 1].1;
            // Remove smallest bucket if its count is below target.
            loop {
                let (bucket_count, bucket_i) = counts.pop().expect("Unexpected error in pop() method of Associative buildinding block.");
                // If the buckets has more elements than the target
                // count we keep it as a pop bucket.
                if bucket_count >= target_count {
                    counts.push((bucket_count, bucket_i));
                    break;
                } else {
                    total_count -= bucket_count;
                }
            }
            // If we did not remove any bucket, all the buckets have
            // more elements than the target count. Therefore, we can
            // stop and pop.
            if prev_i == counts[counts.len() - 1].1 {
                break target_count;
            }
        };

        // Below is the pop phase.
        // We remove whats above target_count from each bucket.
        // Since target_count is a round number, the total to pop
        // might exceed what was asked. Therefore, we don't keep popping
        // if we reached the amount requested. We but still have to unlock
        // the locked buckets.
        let mut popped = 0;
        for (count, i) in counts.into_iter() {
            let pop_count =
                std::cmp::min(count - target_count, n - popped);
            if pop_count > 0 {
                victims.append(&mut self.containers[i].pop(pop_count));
                popped += pop_count;
            }
        }

        victims
    }

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, some values are returned.
    /// If a bucket where a value is assign is full, the associated
    /// input key/value pair will be returned, even though this
    /// `Associative` building block is not at capacity.
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
        test_building_block(Associative::new(
            vec![Array::new(5); 10],
            DefaultHasher::new(),
        ));
    }
}
