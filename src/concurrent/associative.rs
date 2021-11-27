use crate::concurrent::Concurrent;
use crate::concurrent::{LockedItem, Sequential};
use crate::private::clone::CloneCell;
use crate::{BuildingBlock, Get};
use std::hash::{Hash, Hasher};
use std::marker::Sync;
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// Concurrent implementation of container                                 //
//------------------------------------------------------------------------//

/// Associative [`BuildingBlock`](../trait.BuildingBlock.html) wrapper with
/// multiple sets.
///
/// Associative container is an array of containers. Whenever an element
/// is to be inserted/looked up, the key is hashed to choose the set where
/// container key/value pair will be stored.  
/// On insertion, if the target set is full, an element is popped from the
/// same set. Therefore, the container may pop while not being full.
/// When invoking [`pop()`](../trait.BuildingBlock.html#tymethod.pop) to evict a
/// container element, the method is called on all sets. A victim is elected
/// and then all elements that are not elected are reinserted inside the
/// container.
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::container::Vector;
/// use cache::concurrent::Associative;
/// use std::collections::hash_map::DefaultHasher;
///
/// // Build a Vector cache of 2 sets. Each set hold one element.
/// let mut c = Associative::new(2, 2, |n|{Vector::new(n)}, DefaultHasher::new());
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
    pub fn new<F>(
        n_sets: usize,
        set_size: usize,
        new: F,
        set_hasher: H,
    ) -> Self
    where
        F: Fn(usize) -> C,
    {
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

/// `Vec` of flush iterators flushing elements sequentially,
/// starting from last iterator until empty.
pub struct VecFlushIterator<'a, K, V>
where
    K: 'a,
    V: 'a,
{
    pub it: Vec<Box<dyn Iterator<Item = (K, V)> + 'a>>,
}

impl<'a, K, V> Iterator for VecFlushIterator<'a, K, V>
where
    K: 'a,
    V: 'a,
{
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.it.pop() {
                None => {
                    return None;
                }
                Some(mut it) => {
                    if let Some(e) = it.next() {
                        self.it.push(it);
                        return Some(e);
                    }
                }
            }
        }
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
        Box::new(VecFlushIterator {
            it: self.containers.iter_mut().map(|c| c.flush()).collect(),
        })
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

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut victims = Vec::<(K, V)>::new();
        if n == 0 {
            return victims;
        }
        victims.reserve(n);

        // Collect all buckets element count.
        // We acquire exclusive lock on buckets in the process.
        let mut lengths =
            Vec::<(usize, usize)>::with_capacity(self.n_sets + 1);
        for i in 0..self.n_sets {
            let n = match self.containers[i].lock_mut() {
                Ok(_) => unsafe { self.containers[i].deref_mut().count() },
                Err(_) => 0usize,
            };
            lengths.push((n, i));
        }
        lengths.sort();
        lengths.insert(0, (0usize, 0usize));

        // Compute the number of elements to pop from each bucket.
        let mut tot = 0usize;
        let mut count = vec![0usize; self.n_sets];
        let n_sets = lengths.len();
        for i in 1..n_sets {
            if tot >= n {
                break;
            }
            for j in (i..n_sets).rev() {
                let (lj, cj) = lengths[j];
                let (li, _) = lengths[j - 1];
                let l = lj - li;
                if tot + l >= n {
                    let l = n - tot;
                    count[cj] += l;
                    lengths[j].0 -= l;
                    tot += l;
                    break;
                } else {
                    count[cj] += l;
                    lengths[j].0 -= l;
                    tot += l;
                }
            }
        }

        // Append elements to victims vector.
        // Buckets are unlocked in the process.
        for (i, n) in count.iter().enumerate() {
            if n > &0 {
                victims.append(
                    &mut unsafe { self.containers[i].deref_mut() }.pop(*n),
                );
            }
            self.containers[i].unlock();
        }
        victims
    }

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

impl<'a, K, V, C, H> Concurrent<'a, K, V> for Associative<C, H>
where
    K: 'a + Hash + Clone,
    V: 'a + Ord,
    C: 'a + BuildingBlock<'a, K, V>,
    H: Hasher + Clone,
{
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

impl<K, V, U, W, C, H> Get<K, V, LockedItem<U>, LockedItem<W>>
    for Associative<C, H>
where
    K: Hash + Clone,
    U: Deref<Target = V>,
    W: DerefMut<Target = V>,
    H: Hasher + Clone,
    C: Get<K, V, U, W>,
{
    fn get<'a>(&'a self, key: &K) -> Option<LockedItem<U>> {
        let i = self.set(key.clone());
        self.containers[i].get(key)
    }

    fn get_mut<'a>(&'a mut self, key: &K) -> Option<LockedItem<W>> {
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
    use crate::container::Vector;
    use crate::tests::{test_building_block, test_get};
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn building_block() {
        test_building_block(Associative::new(
            5,
            10,
            |n| Vector::new(n),
            DefaultHasher::new(),
        ));
    }

    #[test]
    fn concurrent() {
        test_concurrent(
            Associative::new(
                30,
                30,
                |n| Vector::new(n),
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
            |n| Vector::new(n),
            DefaultHasher::new(),
        ));
    }
}
