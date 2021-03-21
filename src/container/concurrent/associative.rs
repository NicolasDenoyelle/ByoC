use crate::container::concurrent::{Sequential, SequentialIter};
use crate::container::{
    Concurrent, Container, Insert, Iter, IterMut, Sequential as Seq,
};
use crate::lock::RWLockGuard;
use crate::reference::{FromValue, Reference};
use crate::utils::clone::CloneCell;
use std::hash::{Hash, Hasher};
use std::marker::Sync;

//----------------------------------------------------------------------------//
// Concurrent implementation of container                                     //
//----------------------------------------------------------------------------//

/// Associative [`container`](../trait.Container.html) wrapper with multiple sets.
///
/// Associative container is an array of containers. Whenever an element is to be
/// insered/looked up, the key is hashed (`key.into() % n_sets`) to choose
/// the set where container [reference](../../reference/trait.Reference.html)
/// will be stored.
/// On insertion, if the target
/// set is full, an element is popped from the same set.
///
/// When invoking `pop()` to evict a container [reference](../reference/trait.Reference.html),
/// `pop()` on all sets. A victim is elected then all elements that are not elected
/// are reinserted inside the container.
///
/// When invoking `push()`, only the container matching the key is affected and only
/// this container may `pop()` a value.
///
/// ## Generics:
///
/// * `K`: The type of key to use. Keys must implement `Clone` trait and `Hash`
/// trait to compute the set index from key.
/// * `V`: Value type stored in [container reference](../../reference/trait.Reference.html).
/// * `R`: A type of container [reference](../../reference/trait.Reference.html).
/// * `C`: A type of [Container](../trait.Container.html).
///
/// ## Examples
///
/// ```
/// use cache::container::{Container, Insert};
/// use cache::container::sequential::Map;
/// use cache::container::concurrent::Associative;
/// use std::collections::hash_map::DefaultHasher;
/// use cache::reference::Default;
///
/// // Build a Map cache of 2 sets. Each set hold one element.
/// let mut c = Associative::<_,Default<_>,_,_>::new(2, 2, |n|{Map::new(n)}, DefaultHasher::new());
///
/// // Container as room for first and second element and returns None.
/// assert!(c.insert(0u16, 4).is_none());
/// assert!(c.insert(1u16, 12).is_none());
///
/// // Then we don't know if a set is full. Next insertion may pop:
/// match c.insert(2u16, 14) {
///       None => { println!("Still room for one more"); }
///       Some((key, value)) => {
///             assert!(key == 2u16);
///             assert!(*value == 14);
///       }
/// }
///```
pub struct Associative<K, V, C, H>
where
    K: Clone + Hash,
    V: Ord,
    C: Container<K, V>,
    H: Hasher + Clone,
{
    n_sets: usize,
    set_size: usize,
    containers: CloneCell<Vec<Sequential<K, V, C>>>,
    hasher: H,
}

impl<K, V, C, H> Associative<K, V, C, H>
where
    K: Clone + Hash,
    V: Ord,
    C: Container<K, V>,
    H: Hasher + Clone,
{
    /// Construct a new associative container from a list of containers.
    ///
    /// The resulting associative container will have as many sets as containers in
    /// input.
    ///
    /// * `n_sets`: The number of sets for this container.
    /// * `set_size`: The capacity of each set. Every set of this container have
    /// the same capacity.
    /// * `new`: A container constructor closure taking the set size as argument
    /// to build a container of the same capacity.
    pub fn new<F>(n_sets: usize, set_size: usize, new: F, set_hasher: H) -> Self
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

    fn set(&self, key: K) -> usize {
        if self.n_sets == 0 || self.set_size == 0 {
            return 0;
        };
        let mut hasher = self.hasher.clone();
        key.hash(&mut hasher);
        let i = hasher.finish();
        usize::from((i % (self.n_sets as u64)) as u16)
    }
}

impl<K, V, R, C, H> Insert<K, V, R> for Associative<K, R, C, H>
where
    K: Clone + Hash,
    R: Reference<V> + FromValue<V>,
    C: Container<K, R>,
    H: Hasher + Clone,
{
}

unsafe impl<K, V, C, H> Send for Associative<K, V, C, H>
where
    K: Clone + Hash,
    V: Ord,
    C: Container<K, V>,
    H: Hasher + Clone,
{
}

unsafe impl<K, V, C, H> Sync for Associative<K, V, C, H>
where
    K: Clone + Hash,
    V: Ord,
    C: Container<K, V>,
    H: Hasher + Clone,
{
}

impl<K, V, C, H> Container<K, V> for Associative<K, V, C, H>
where
    K: Clone + Hash,
    V: Ord,
    C: Container<K, V>,
    H: Hasher + Clone,
{
    fn capacity(&self) -> usize {
        return self.n_sets * self.set_size;
    }

    fn flush(&mut self) -> Vec<(K, V)> {
        let mut v = Vec::with_capacity(self.capacity());
        for i in 0..self.n_sets {
            v.append(&mut self.containers[i].flush());
        }
        v
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

    fn take(&mut self, key: &K) -> Option<V> {
        let i = self.set(key.clone());
        self.containers[i].take(key)
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

impl<K, V, R, C, H> Concurrent<K, V, R> for Associative<K, R, C, H>
where
    K: Clone + Hash,
    R: Reference<V>,
    C: Container<K, R> + Seq<K, V, R>,
    H: Hasher + Clone,
{
    fn get(&mut self, key: &K) -> Option<RWLockGuard<&V>> {
        if self.n_sets == 0 || self.set_size == 0 {
            return None;
        };
        let i = self.set(key.clone());
        self.containers[i].get(key)
    }

    fn get_mut(&mut self, key: &K) -> Option<RWLockGuard<&mut V>> {
        if self.n_sets == 0 || self.set_size == 0 {
            return None;
        };
        let i = self.set(key.clone());
        self.containers[i].get_mut(key)
    }
}

impl<K, V, C, H> Clone for Associative<K, V, C, H>
where
    K: Clone + Hash,
    V: Ord,
    C: Container<K, V>,
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

//----------------------------------------------------------------------------//
// iterator for associative concurrent cache                                  //
//----------------------------------------------------------------------------//

pub struct AssociativeIter<'a, K, V, R, C, I>
where
    K: 'a + Clone + Hash,
    V: 'a,
    R: 'a + Reference<V>,
    C: 'a + Container<K, R> + Iter<'a, K, V, R, Iterator = I>,
    I: Iterator<Item = (&'a K, &'a V)>,
{
    containers: std::slice::IterMut<'a, Sequential<K, R, C>>,
    it: Option<SequentialIter<'a, I>>,
}

impl<'a, K, V, R, C, I> Iterator for AssociativeIter<'a, K, V, R, C, I>
where
    K: 'a + Clone + Hash,
    V: 'a,
    R: 'a + Reference<V>,
    C: 'a + Container<K, R> + Iter<'a, K, V, R, Iterator = I>,
    I: Iterator<Item = (&'a K, &'a V)>,
{
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.it {
            None => match self.containers.next() {
                None => None,
                Some(c) => {
                    self.it = Some(c.iter());
                    self.next()
                }
            },
            Some(it) => match it.next() {
                Some(v) => Some(v),
                None => {
                    self.it = None;
                    self.next()
                }
            },
        }
    }
}

impl<'a, K, V, R, C, I, H> Iter<'a, K, V, R> for Associative<K, R, C, H>
where
    K: 'a + Clone + Hash,
    V: 'a,
    R: 'a + Reference<V>,
    C: 'a + Container<K, R> + Seq<K, V, R> + Iter<'a, K, V, R, Iterator = I>,
    I: Iterator<Item = (&'a K, &'a V)>,
    H: Hasher + Clone,
{
    type Iterator = AssociativeIter<'a, K, V, R, C, I>;

    fn iter(&'a mut self) -> Self::Iterator {
        AssociativeIter::<'a, K, V, R, C, I> {
            containers: self.containers.iter_mut(),
            it: None,
        }
    }
}

pub struct AssociativeIterMut<'a, K, V, R, C, I>
where
    K: 'a + Clone + Hash,
    V: 'a,
    R: 'a + Reference<V>,
    C: 'a + Container<K, R> + IterMut<'a, K, V, R, Iterator = I>,
    I: Iterator<Item = (&'a K, &'a mut V)>,
{
    containers: std::slice::IterMut<'a, Sequential<K, R, C>>,
    it: Option<SequentialIter<'a, I>>,
}

impl<'a, K, V, R, C, I> Iterator for AssociativeIterMut<'a, K, V, R, C, I>
where
    K: 'a + Clone + Hash,
    V: 'a,
    R: 'a + Reference<V>,
    C: 'a + Container<K, R> + IterMut<'a, K, V, R, Iterator = I>,
    I: Iterator<Item = (&'a K, &'a mut V)>,
{
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.it {
            None => match self.containers.next() {
                None => None,
                Some(c) => {
                    self.it = Some(c.iter_mut());
                    self.next()
                }
            },
            Some(it) => match it.next() {
                Some(v) => Some(v),
                None => {
                    self.it = None;
                    self.next()
                }
            },
        }
    }
}

impl<'a, K, V, R, C, I, H> IterMut<'a, K, V, R> for Associative<K, R, C, H>
where
    K: 'a + Ord + Clone + Hash,
    V: 'a,
    R: 'a + Reference<V>,
    C: 'a + Container<K, R> + Seq<K, V, R> + IterMut<'a, K, V, R, Iterator = I>,
    I: Iterator<Item = (&'a K, &'a mut V)>,
    H: Hasher + Clone,
{
    type Iterator = AssociativeIterMut<'a, K, V, R, C, I>;

    fn iter_mut(&'a mut self) -> Self::Iterator {
        AssociativeIterMut::<'a, K, V, R, C, I> {
            containers: self.containers.iter_mut(),
            it: None,
        }
    }
}
