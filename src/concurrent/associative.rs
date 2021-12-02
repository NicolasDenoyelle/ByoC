use crate::concurrent::{Sequential, SequentialCell};
use crate::private::clone::CloneCell;
use crate::{BuildingBlock, Concurrent, Get};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::marker::Sync;
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// Concurrent implementation of container                                 //
//------------------------------------------------------------------------//

/// Associative [`BuildingBlock`](../trait.BuildingBlock.html) wrapper with
/// multiple sets/buckets.
///
/// This building block is implemented as an array of building blocks.
/// Keys inserted in this container must be hashable to find in which bucket
/// it should be stored/retrieved.
///
/// Since a key can only go in one bucket, the container may refuse
/// insertions before it is actually full if the target buckets are full.
///
/// When [popping](../trait.BuildingBlock.html#tymethod.pop) elements,
/// the policy is to balance buckets element count rather than strictly
/// pop values in descending order. The latter might be "loosely" satisfied
/// if the buckets building block apply such a policy and maximum value are
/// evenly distributed across the buckets with the largest count of
/// elements.
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::container::Array;
/// use cache::concurrent::Associative;
/// use std::collections::hash_map::DefaultHasher;
///
/// // Build a Array cache of 2 sets. Each set hold one element.
/// let mut c = Associative::new(2, 2, |n|{Array::new(n)}, DefaultHasher::new());
///
/// // BuildingBlock as room for first and second element and returns None.
/// assert!(c.push(vec![(0, 4)]).pop().is_none());
/// assert!(c.push(vec![(1, 12)]).pop().is_none());
///
/// // Then we don't know if a set is full. Next insertion may pop:
/// match c.push(vec![(2, 14)]).pop() {
///       None => { println!("Still room for one more"); }
///       Some((key, value)) => {
///             assert!(key == 1);
///             assert!(value == 12);
///       }
/// }
///```
pub struct Associative<C, H: Hasher + Clone> {
    n_sets: usize,
    set_size: usize,
    containers: CloneCell<Vec<Sequential<C>>>,
    hasher: H,
}

impl<C, H: Hasher + Clone> Associative<C, H> {
    /// Construct a new associative container.
    ///
    /// This function builds `n_sets` containers of capacity `set_size`
    /// each using a closure provided by the user to build a container
    /// given its desired capacity. Neither `n_sets` nor `set_size` can
    /// be zero or else this function will panic.
    pub fn new<F: Fn(usize) -> C>(
        n_sets: usize,
        set_size: usize,
        new: F,
        set_hasher: H,
    ) -> Self {
        if n_sets * set_size == 0 {
            panic!("Associative container must contain at least one set with non zero capacity.");
        }

        let mut a = Associative {
            n_sets: n_sets,
            set_size: set_size,
            containers: CloneCell::new(Vec::with_capacity(n_sets)),
            hasher: set_hasher,
        };
        for _ in 0..n_sets {
            a.containers.push(Sequential::new(new(set_size)));
        }
        a
    }

    fn set<K: Hash>(&self, key: K) -> usize {
        let mut hasher = self.hasher.clone();
        key.hash(&mut hasher);
        let i = hasher.finish();
        usize::from((i % (self.n_sets as u64)) as u16)
    }
}

impl<'a, K, V, C, H> BuildingBlock<'a, K, V> for Associative<C, H>
where
    K: 'a + Clone + Hash,
    V: 'a + Ord,
    C: BuildingBlock<'a, K, V>,
    H: Hasher + Clone,
{
    fn capacity(&self) -> usize {
        return self.n_sets * self.set_size;
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        let iterators: Vec<Box<dyn Iterator<Item = (K, V)> + 'a>> =
            self.containers.iter_mut().map(|c| c.flush()).collect();
        Box::new(iterators.into_iter().flat_map(|c| c))
    }

    fn contains(&self, key: &K) -> bool {
        let i = self.set(key.clone());
        self.containers[i].contains(key)
    }

    fn count(&self) -> usize {
        (0..self.n_sets).map(|i| self.containers[i].count()).sum()
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        let i = self.set(key.clone());
        self.containers[i].take(key)
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
        if n == 0 || self.n_sets == 0 {
            return victims;
        }
        victims.reserve(n);

        // Collect all buckets element count.
        // We acquire exclusive lock on buckets in the process.
        let mut counts =
            Vec::<(usize, usize)>::with_capacity(self.n_sets + 1);
        for i in 0..self.n_sets {
            let n = match self.containers[i].lock_mut() {
                Ok(_) => unsafe { self.containers[i].deref_mut().count() },
                Err(_) => 0usize,
            };
            counts.push((n, i));
        }

        let mut total_count: usize = counts.iter().map(|(n, _)| n).sum();

        // If there is more elements to pop than elements available
        // Then we pop everything.
        if total_count <= n {
            for (_, i) in counts.into_iter() {
                unsafe {
                    victims.append(
                        &mut self.containers[i]
                            .deref_mut()
                            .flush()
                            .collect(),
                    )
                }
                self.containers[i].unlock();
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
                    self.containers[bucket_i].unlock();
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
                unsafe {
                    victims.append(
                        &mut self.containers[i].deref_mut().pop(pop_count),
                    );
                }
                popped += pop_count;
            }
            self.containers[i].unlock();
        }

        victims
    }

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, some values are returned.
    /// If a bucket where a value is assign is full, the associated
    /// input key/value pair will be returned, even though this
    /// `Associative` bulding block is not at capacity.
    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let n = elements.len();
        let mut set_elements: Vec<Vec<(K, V)>> =
            Vec::with_capacity(self.n_sets);
        for _ in 0..self.n_sets {
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

unsafe impl<C, H: Hasher + Clone> Send for Associative<C, H> {}

unsafe impl<C, H: Hasher + Clone> Sync for Associative<C, H> {}

impl<C, H: Hasher + Clone> Concurrent for Associative<C, H> {
    fn clone(&self) -> Self {
        Associative {
            n_sets: self.n_sets,
            set_size: self.set_size,
            containers: self.containers.clone(),
            hasher: self.hasher.clone(),
        }
    }
}

//------------------------------------------------------------------------//
// Get Trait Implementation                                               //
//------------------------------------------------------------------------//

impl<K, V, U, W, C, H> Get<K, V, SequentialCell<U>, SequentialCell<W>>
    for Associative<C, H>
where
    K: Hash + Clone,
    U: Deref<Target = V>,
    W: DerefMut<Target = V>,
    H: Hasher + Clone,
    C: Get<K, V, U, W>,
{
    unsafe fn get(&self, key: &K) -> Option<SequentialCell<U>> {
        let i = self.set(key.clone());
        self.containers[i].get(key)
    }

    unsafe fn get_mut(&mut self, key: &K) -> Option<SequentialCell<W>> {
        let i = self.set(key.clone());
        self.containers[i].get_mut(key)
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Associative;
    use crate::concurrent::tests::test_concurrent;
    use crate::container::Array;
    use crate::tests::{test_building_block, test_get};
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn building_block() {
        test_building_block(Associative::new(
            5,
            10,
            |n| Array::new(n),
            DefaultHasher::new(),
        ));
    }

    #[test]
    fn concurrent() {
        test_concurrent(
            Associative::new(
                30,
                30,
                |n| Array::new(n),
                DefaultHasher::new(),
            ),
            64,
        );
    }

    #[test]
    fn get() {
        test_get(Associative::new(
            5,
            10,
            |n| Array::new(n),
            DefaultHasher::new(),
        ));
    }
}
