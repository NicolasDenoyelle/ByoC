use crate::policy::{Reference, ReferenceFactory};
use crate::{BuildingBlock, Concurrent, Get};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// Reference wrapper                                                      //
//------------------------------------------------------------------------//

/// Eviction policy for [`BuildingBlock`](../trait.BuildingBlock.html).
///
/// This structure implements a wrapper around a building blocks that
/// wraps values (from key/value pairs) into an orderable cell.
/// As a result, when popping elements out a building blocks,
/// this wrapper decides which element is going to be evicted.
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::container::Vector;
/// use cache::policy::{Policy, FIFO};
///
/// let mut c = Policy::new(Vector::new(3), FIFO::new());
/// c.push(vec![("item1",()), ("item2",()), ("item0",())]);
/// assert_eq!(c.pop(1).pop().unwrap().0, "item1");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item2");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item0");
///```
pub struct Policy<C, V, R, F>
where
    R: Reference<V>,
    F: ReferenceFactory<V, R>,
{
    container: C,
    factory: F,
    unused: PhantomData<(R, V)>,
}

impl<C, V, R, F> Policy<C, V, R, F>
where
    R: Reference<V>,
    F: ReferenceFactory<V, R>,
{
    /// Construct a new policy wrapper.
    pub fn new(container: C, factory: F) -> Self {
        Policy {
            container: container,
            factory: factory,
            unused: PhantomData,
        }
    }
}

impl<'a, K, V, C, R, F> BuildingBlock<'a, K, V> for Policy<C, V, R, F>
where
    K: 'a,
    V: 'a,
    R: 'a + Reference<V>,
    C: BuildingBlock<'a, K, R>,
    F: ReferenceFactory<V, R>,
{
    fn capacity(&self) -> usize {
        self.container.capacity()
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(self.container.flush().map(|(k, r)| (k, r.unwrap())))
    }

    fn count(&self) -> usize {
        self.container.count()
    }

    fn contains(&self, key: &K) -> bool {
        self.container.contains(key)
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        match self.container.take(key) {
            None => None,
            Some((k, r)) => Some((k, r.unwrap())),
        }
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        self.container
            .pop(n)
            .into_iter()
            .map(|(k, r)| (k, r.unwrap()))
            .collect()
    }

    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let (container, factory) =
            (&mut self.container, &mut self.factory);
        container
            .push(
                elements
                    .into_iter()
                    .map(|(k, v)| (k, factory.wrap(v)))
                    .collect(),
            )
            .into_iter()
            .map(|(k, r)| (k, r.unwrap()))
            .collect()
    }
}

unsafe impl<C, V, R, F> Send for Policy<C, V, R, F>
where
    R: Reference<V>,
    F: ReferenceFactory<V, R> + Send,
    C: Send,
{
}

unsafe impl<C, V, R, F> Sync for Policy<C, V, R, F>
where
    R: Reference<V>,
    F: ReferenceFactory<V, R> + Sync,
    C: Sync,
{
}

impl<C, V, R, F> Clone for Policy<C, V, R, F>
where
    R: Reference<V>,
    F: ReferenceFactory<V, R> + Clone,
    C: Clone,
{
    fn clone(&self) -> Self {
        Policy {
            container: self.container.clone(),
            factory: self.factory.clone(),
            unused: PhantomData,
        }
    }
}

impl<'a, K, V, C, R, F> Concurrent<'a, K, V> for Policy<C, V, R, F>
where
    K: 'a,
    V: 'a,
    R: 'a + Reference<V>,
    F: ReferenceFactory<V, R> + Clone + Send + Sync,
    C: BuildingBlock<'a, K, R> + Concurrent<'a, K, R>,
{
    fn clone(&self) -> Self {
        Policy {
            container: Concurrent::clone(&self.container),
            factory: self.factory.clone(),
            unused: PhantomData,
        }
    }
}

impl<'a, K, V, U, W, R, F, C> Get<'a, K, V, U, W> for Policy<C, V, R, F>
where
    R: 'a + Reference<V>,
    F: ReferenceFactory<V, R> + Clone + Send + Sync,
    U: 'a + Deref<Target = V>,
    W: 'a + DerefMut<Target = V>,
    C: Get<'a, K, V, U, W>,
{
    fn get(&'a self, key: &K) -> Option<U> {
        self.container.get(key)
    }

    fn get_mut(&'a mut self, key: &K) -> Option<W> {
        self.container.get_mut(key)
    }
}
