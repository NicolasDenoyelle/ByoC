use crate::container::{Container, Get, Packed};
use std::marker::PhantomData;

//----------------------------------------------------------------------------//
// Top K Container                                                            //
//----------------------------------------------------------------------------//

/// [`Container`](../trait.Container.html) wrapper with persistent references.
///
/// Top K Container is a container wrapper that will keep its references intact.
/// Whenever a `push` occures, value inside victim reference is swapped with the
/// value to insert. Returned reference is a clone of victim reference embedding
/// victim value.
///
/// ## Generics:
///
/// * `K`: The type of key to use. Keys must implement `Clone` trait.
/// * `V`: The type of element inside container references.
/// * `R`: A type of cache [reference](../../reference/trait.Reference.html).
/// * `C`: A type of cache [container](../trait.Container.html).
///
/// ## Examples
///
/// ```
/// use cache::container::{Container, Map, TopK};
/// use cache::reference::{Reference, LRU};
///
/// // Build a Map cache of one element.
/// let mut c = TopK::new(Map::<_,LRU<_>>::new(1));
///
/// // Container as room an element and returns None.
/// assert!(c.push(0u16, LRU::new(4)).is_none());
///
/// // Container is full, next insertion will pop an element:
/// let (key, popped) = c.push(2u16, LRU::new(3)).unwrap();
///
/// // The victim, is the greatest element inserted.
/// assert!(key == 0u16);
/// assert!(*popped == 4);
///
/// // The victim popped as the same reference order as the one taking its place in cache.
/// let current = c.take(&2u16).unwrap();
/// assert!(popped == current);
///```
pub struct TopK<K, V, R, C>
where
    K: Clone,
    R: Reference<V>,
    C: Container<K, R>,
{
    container: C,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
    unused_r: PhantomData<R>,
}

impl<K, V, R, C> TopK<K, V, R, C>
where
    K: Clone,
    R: Reference<V>,
    C: Container<K, R>,
{
    /// Construct a new TopK container from another container.
    pub fn new(container: C) -> TopK<K, V, R, C> {
        TopK {
            container: container,
            unused_k: PhantomData,
            unused_v: PhantomData,
            unused_r: PhantomData,
        }
    }
}

impl<K, V, R, C> Packed<K, R> for TopK<K, V, R, C>
where
    K: Clone,
    R: Reference<V>,
    C: Container<K, R> + Packed<K, R>,
{
}

impl<K, V, R, C> Container<K, R> for TopK<K, V, R, C>
where
    K: Clone,
    R: Reference<V>,
    C: Container<K, R>,
{
    fn capacity(&self) -> usize {
        self.container.capacity()
    }

    fn count(&self) -> usize {
        self.container.count()
    }

    fn flush(&mut self) -> Vec<(K, R)> {
        self.container.flush()
    }

    fn clear(&mut self) {
        self.container.clear()
    }

    fn contains(&self, key: &K) -> bool {
        self.container.contains(key)
    }

    fn take(&mut self, key: &K) -> Option<R> {
        self.container.take(key)
    }

    fn pop(&mut self) -> Option<(K, R)> {
        self.container.pop()
    }

    fn push(&mut self, key: K, reference: R) -> Option<(K, R)> {
        match self.container.push(key.clone(), reference) {
            None => None,
            Some((old_key, mut r)) => match self.container.take(&key) {
                None => Some((old_key, r)),
                Some(new_ref) => {
                    let old_val = r.replace(new_ref.unwrap());
                    let old_ref = R::from_ref(old_val, &r);
                    assert!(self.container.push(key, r).is_none());
                    Some((old_key, old_ref))
                }
            },
        }
    }
}

impl<K, V, R, C> Get<K, V> for TopK<K, V, R, C>
where
    K: Clone,
    R: Reference<V>,
    C: Container<K, V> + Get<K, V>,
{
    fn get(&mut self, key: &K) -> Option<&V> {
        self.container.get(key)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.container.get_mut(key)
    }
}
