use crate::container::{Container, Get, Sequential};
use crate::lock::RWLockGuard;
use crate::marker::Concurrent;
use crate::utils::{clone::CloneCell, flush::VecFlushIterator};
use std::hash::{Hash, Hasher};
use std::marker::Sync;

//------------------------------------------------------------------------//
// Concurrent implementation of container                                 //
//------------------------------------------------------------------------//

/// Associative [`container`](../trait.Container.html) wrapper with
/// multiple sets.
///
/// Associative container is an array of containers. Whenever an element
/// is to be insered/looked up, the key is hashed to choose the set where
/// container key/value pair will be stored.
/// On insertion, if the target set is full, an element is popped from the
/// same set. Therefore, the container may pop while not being fulled.
///
/// When invoking `pop()` to evict a container element `pop()` is called
/// on all sets. A victim is elected then all elements that are not elected
/// are reinserted inside the container.
///
/// ## Generics:
///
/// * `H`: The hasher type to hash keys.
///
/// ## Examples
///
/// ```
/// use cache::container::{Container, Vector, Associative};
/// use std::collections::hash_map::DefaultHasher;
///
/// // Build a Vector cache of 2 sets. Each set hold one element.
/// let mut c = Associative::new(2, 2, |n|{Vector::new(n)}, DefaultHasher::new());
///
/// // Container as room for first and second element and returns None.
/// assert!(c.push(0, 4).is_none());
/// assert!(c.push(1, 12).is_none());
///
/// // Then we don't know if a set is full. Next insertion may pop:
/// match c.push(2, 14) {
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
    /// Construct a new associative container from a list of containers.
    ///
    /// The resulting associative container will have as many sets as
    /// containers in input.
    ///
    /// * `n_sets`: The number of sets for this container.
    /// * `set_size`: The capacity of each set. Every set of this
    /// container have the same capacity.
    /// * `new`: A container constructor closure taking the set size as
    /// argument to build a container of the same capacity.
    pub fn new<F>(
        n_sets: usize,
        set_size: usize,
        new: F,
        set_hasher: H,
    ) -> Self
    where
        F: Fn(usize) -> C,
    {
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
        if self.n_sets == 0 || self.set_size == 0 {
            return 0;
        };
        let mut hasher = self.hasher.clone();
        key.hash(&mut hasher);
        let i = hasher.finish();
        usize::from((i % (self.n_sets as u64)) as u16)
    }
}

impl<'a, K, V, C, H> Container<'a, K, V> for Associative<C, H>
where
    K: 'a + Clone + Hash,
    V: 'a + Ord,
    C: Container<'a, K, V>,
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

    fn clear(&mut self) {
        for i in 0..self.n_sets {
            self.containers[i].clear();
        }
    }

    fn take(
        &'a mut self,
        key: &'a K,
    ) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        if self.n_sets * self.set_size == 0 {
            Box::new(Vec::<(K, V)>::new().into_iter())
        } else {
            let i = self.set(key.clone());
            self.containers[i].take(key)
        }
    }

    fn pop(&mut self) -> Option<(K, V)> {
        let mut victims: Vec<Option<(K, V)>> = (0..self.n_sets)
            .map(|i| {
                self.containers[i].lock_mut();
                unsafe { self.containers[i].deref_mut().pop() }
            })
            .collect();

        let n = victims.len();

        let mut v = 0;
        for i in 1..n {
            v = match (&victims[i], &victims[v]) {
                (None, None) => 0,
                (None, Some(_)) => v,
                (Some(_), None) => i,
                (Some((_, vi)), Some((_, vv))) => {
                    if vi >= vv {
                        i
                    } else {
                        v
                    }
                }
            }
        }

        for i in (v + 1..n).rev() {
            match victims.pop().unwrap() {
                Some((k, r)) => {
                    assert!(unsafe {
                        self.containers[i].deref_mut().push(k, r).is_none()
                    });
                }
                None => {}
            }
            self.containers[i].unlock();
        }

        let ret = victims.pop().unwrap();
        self.containers[v].unlock();

        for i in (0..v).rev() {
            match victims.pop().unwrap() {
                Some((k, r)) => {
                    assert!(unsafe {
                        self.containers[i].deref_mut().push(k, r).is_none()
                    });
                }
                None => {}
            }
            self.containers[i].unlock();
        }

        ret
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        if self.n_sets == 0 || self.set_size == 0 {
            return Some((key, reference));
        };

        let i = self.set(key.clone());
        self.containers[i].push(key, reference)
    }
}

unsafe impl<C, H: Hasher + Clone> Send for Associative<C, H> {}

unsafe impl<C, H: Hasher + Clone> Sync for Associative<C, H> {}

impl<'a, K, V, C, H> Concurrent<'a, K, V> for Associative<C, H>
where
    K: 'a + Hash + Clone,
    V: 'a + Ord,
    C: Container<'a, K, V>,
    H: Hasher + Clone,
{
}

impl<'a, K, V, C, H, T> Get<'a, K, V> for Associative<C, H>
where
    K: 'a + Clone + Hash,
    V: 'a + Ord,
    C: Get<'a, K, V, Item = T>,
    H: Hasher + Clone,
    T: 'a,
{
    type Item = RWLockGuard<'a, T>;
    fn get(&'a mut self, key: &K) -> Option<Self::Item> {
        if self.n_sets == 0 || self.set_size == 0 {
            return None;
        };
        let i = self.set(key.clone());
        self.containers[i].lock_mut();
        Get::get(&mut self.containers[i], key)
    }
}

impl<C, H: Hasher + Clone> Clone for Associative<C, H> {
    fn clone(&self) -> Self {
        Associative {
            n_sets: self.n_sets,
            set_size: self.set_size,
            containers: self.containers.clone(),
            hasher: self.hasher.clone(),
        }
    }
}
