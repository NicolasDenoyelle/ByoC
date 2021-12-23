use crate::private::bits::log2;
use crate::{BuildingBlock, Concurrent, Get, GetMut, Prefetch};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::marker::Sync;
use std::ops::{Deref, DerefMut};

/// Hasher wrapper that returns a subset of the hash bits shifted to the
/// right.
///
/// The purpose of this structure is to concatenate multiple layer of
/// [`Associative`](struct.Associative.html) building block with one
/// different hasher but different hashes on each level.
/// See how to instanciate such a structure in
/// [builder examples](builder/associative/struct.AssociativeBuilder.html).
///
/// This hasher can be used to create another `MultisetHasher` returning a
/// disjoint set of bits compared to the first one from the hash bits.
/// See [`next()`](struct.MultisetHasher.html#tymethod.next) method.
pub struct MultisetHasher<H: Hasher> {
    hasher: H,
    mask: u64,
    rshift: u8,
    nbits: u8,
}

impl<H: Hasher> Hasher for MultisetHasher<H> {
    fn finish(&self) -> u64 {
        let h = self.hasher.finish();
        (h & self.mask) >> self.rshift
    }

    fn write(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes);
    }
}

impl<H: Hasher + Clone> Clone for MultisetHasher<H> {
    fn clone(&self) -> Self {
        MultisetHasher {
            hasher: self.hasher.clone(),
            mask: self.mask,
            rshift: self.rshift,
            nbits: self.nbits,
        }
    }
}

impl<H: Hasher + Clone> MultisetHasher<H> {
    /// Create a new multiset hasher from another hasher
    /// that returns hash with a maximum value of at least nsets.
    pub fn new(hasher: H, nsets: usize) -> Self {
        let nbits = log2(nsets as u64) + 1;
        MultisetHasher {
            hasher: hasher.clone(),
            mask: (!0u64) >> ((64u8 - nbits) as u64),
            rshift: 0u8,
            nbits,
        }
    }

    /// Create a new multiset hasher that uses a disjoint set of bits
    /// of the hash value generated this hasher, located on the left
    /// of the set of bits used by this hasher.
    /// If there is not enough bits available to count up to `nsets`,
    /// an error is returned instead.
    pub fn next(&self, nsets: usize) -> Result<Self, ()> {
        let nbits = log2(nsets as u64) + 1;
        let rshift = self.nbits + self.rshift;

        if rshift + nbits > 64u8 {
            return Err(());
        }

        let mask = ((!0u64) >> ((64u8 - nbits) as u64)) << rshift;
        Ok(MultisetHasher {
            hasher: self.hasher.clone(),
            mask,
            rshift,
            nbits,
        })
    }
}

//------------------------------------------------------------------------//
// Associative container
//------------------------------------------------------------------------//

/// Associative building block wrapper with multiple sets/buckets.
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
/// # Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::{Array, Associative};
/// use std::collections::hash_map::DefaultHasher;
///
/// // Build a Array cache of 2 sets. Each set hold one element.
/// let mut c = Associative::new([Array::new(2), Array::new(2)],
///                              DefaultHasher::new());
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
pub struct Associative<C, H: Hasher + Clone, const N: usize> {
    containers: [C; N],
    hasher: H,
}

impl<C, H: Hasher + Clone, const N: usize> Associative<C, H, N> {
    /// Construct a new associative container.
    ///
    /// This function builds an associative container using other
    /// containers as sets.
    pub fn new(sets: [C; N], key_hasher: H) -> Self {
        Associative {
            containers: sets,
            hasher: key_hasher,
        }
    }

    fn set<K: Hash>(&self, key: K) -> usize {
        let n_sets = self.containers.len();
        let mut hasher = self.hasher.clone();
        key.hash(&mut hasher);
        let i = hasher.finish();
        (i % (n_sets as u64)) as usize
    }
}

impl<'a, K, V, C, H, const N: usize> BuildingBlock<'a, K, V>
    for Associative<C, H, N>
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

    fn count(&self) -> usize {
        self.containers.iter().map(|c| c.count()).sum()
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
        if n == 0 || self.capacity() == 0 {
            return victims;
        }
        victims.reserve(n);

        let n_sets = self.containers.len();

        // Collect all buckets element count.
        // We acquire exclusive lock on buckets in the process.
        let mut counts = Vec::<(usize, usize)>::with_capacity(n_sets + 1);
        for i in 0..n_sets {
            let n = self.containers[i].count();
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
    /// `Associative` bulding block is not at capacity.
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

unsafe impl<C: Send, H: Hasher + Clone, const N: usize> Send
    for Associative<C, H, N>
{
}

unsafe impl<C: Sync, H: Hasher + Clone, const N: usize> Sync
    for Associative<C, H, N>
{
}

impl<C: Concurrent, H: Hasher + Clone, const N: usize> Concurrent
    for Associative<C, H, N>
{
    fn clone(&self) -> Self {
        let mut array = [0; N];
        for i in 0..N {
            array[i] = i;
        }
        Associative {
            containers: array
                .map(|i| Concurrent::clone(&self.containers[i])),
            hasher: self.hasher.clone(),
        }
    }
}

//------------------------------------------------------------------------//
// Get Trait Implementation                                               //
//------------------------------------------------------------------------//

impl<K, V, U, C, H, const N: usize> Get<K, V, U> for Associative<C, H, N>
where
    K: Hash + Clone,
    U: Deref<Target = V>,
    H: Hasher + Clone,
    C: Get<K, V, U>,
{
    unsafe fn get(&self, key: &K) -> Option<U> {
        let i = self.set(key.clone());
        self.containers[i].get(key)
    }
}

impl<K, V, W, C, H, const N: usize> GetMut<K, V, W>
    for Associative<C, H, N>
where
    K: Hash + Clone,
    W: DerefMut<Target = V>,
    H: Hasher + Clone,
    C: GetMut<K, V, W>,
{
    unsafe fn get_mut(&mut self, key: &K) -> Option<W> {
        let i = self.set(key.clone());
        self.containers[i].get_mut(key)
    }
}

//------------------------------------------------------------------------//
// Prefetch Trait Implementation
//------------------------------------------------------------------------//

impl<'a, K, V, H, C, const N: usize> Prefetch<'a, K, V>
    for Associative<C, H, N>
where
    K: 'a + Clone + Hash,
    V: 'a + Ord,
    C: BuildingBlock<'a, K, V> + Prefetch<'a, K, V>,
    H: Hasher + Clone,
{
    fn prefetch(&mut self, keys: Vec<K>) {
        let mut set_keys: Vec<Vec<K>> =
            Vec::with_capacity(self.containers.len());
        for _ in 0..self.containers.len() {
            set_keys.push(Vec::with_capacity(keys.len()));
        }

        for k in keys.into_iter() {
            set_keys[self.set(k.clone())].push(k);
        }

        for c in self.containers.iter_mut().rev() {
            c.prefetch(set_keys.pop().unwrap());
        }
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
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    macro_rules! array {
        ($x: expr, $n: literal) => {
            [(); $n].map(|_| $x)
        };
    }

    use super::{Associative, MultisetHasher};
    use crate::tests::{
        test_building_block, test_concurrent, test_get, test_get_mut,
        test_prefetch,
    };
    use crate::{Array, Sequential};
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn multiset_hasher() {
        let mut h = MultisetHasher::new(DefaultHasher::new(), 1);
        for i in 1..64 {
            h = h.next(1).unwrap();
            assert_eq!(h.nbits, 1);
            assert_eq!(h.rshift, i);
            assert_eq!(h.mask >> h.rshift, 1);
        }
        assert!(h.next(1).is_err());

        let mut h = MultisetHasher::new(DefaultHasher::new(), 7);
        for i in 1..21 {
            h = h.next(7).unwrap();
            assert_eq!(h.nbits, 3);
            assert_eq!(h.rshift, i * 3);
            assert_eq!(h.mask >> h.rshift, 7);
        }
        assert!(h.next(7).is_err());
    }

    #[test]
    fn building_block() {
        test_building_block(Associative::new(
            array![Array::new(5), 10],
            DefaultHasher::new(),
        ));
    }

    #[test]
    fn concurrent() {
        test_concurrent(
            Associative::new(
                array![Sequential::new(Array::new(30)), 30],
                DefaultHasher::new(),
            ),
            64,
        );
    }

    #[test]
    fn get() {
        test_get(Associative::new(
            array![Array::new(5), 10],
            DefaultHasher::new(),
        ));
        test_get_mut(Associative::new(
            array![Array::new(5), 10],
            DefaultHasher::new(),
        ));
    }

    #[test]
    fn prefetch() {
        test_prefetch(Associative::new(
            array![Array::new(5), 10],
            DefaultHasher::new(),
        ));
    }
}
