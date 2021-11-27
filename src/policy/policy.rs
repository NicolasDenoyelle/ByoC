use crate::concurrent::Concurrent;
use crate::policy::{Ordered, Reference, ReferenceFactory};
use crate::{BuildingBlock, Get};
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
    C: Ordered<R>,
{
    container: C,
    factory: F,
    unused: PhantomData<(R, V)>,
}

impl<C, V, R, F> Policy<C, V, R, F>
where
    R: Reference<V>,
    F: ReferenceFactory<V, R>,
    C: Ordered<R>,
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

//------------------------------------------------------------------------//
// BuildingBlock trait implementation
//------------------------------------------------------------------------//

impl<'a, K, V, C, R, F> BuildingBlock<'a, K, V> for Policy<C, V, R, F>
where
    K: 'a,
    V: 'a,
    R: 'a + Reference<V>,
    C: BuildingBlock<'a, K, R> + Ordered<R>,
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
    C: Send + Ordered<R>,
{
}

unsafe impl<C, V, R, F> Sync for Policy<C, V, R, F>
where
    R: Reference<V>,
    F: ReferenceFactory<V, R> + Sync,
    C: Sync + Ordered<R>,
{
}

impl<C, V, R, F> Clone for Policy<C, V, R, F>
where
    R: Reference<V>,
    F: ReferenceFactory<V, R> + Clone,
    C: Clone + Ordered<R>,
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
    C: BuildingBlock<'a, K, R> + Concurrent<'a, K, R> + Ordered<R>,
{
    fn clone(&self) -> Self {
        Policy {
            container: Concurrent::clone(&self.container),
            factory: self.factory.clone(),
            unused: PhantomData,
        }
    }
}

//------------------------------------------------------------------------//
// Get trait implementation
//------------------------------------------------------------------------//

struct PolicyCell<V, R, U>
where
    R: Reference<V>,
    U: Deref<Target = R>,
{
    item: U,
    unused: PhantomData<V>,
}

impl<V, R, U> Deref for PolicyCell<V, R, U>
where
    R: Reference<V>,
    U: Deref<Target = R>,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        self.item.deref().get()
    }
}

impl<V, R, W> DerefMut for PolicyCell<V, R, W>
where
    R: Reference<V>,
    W: Deref<Target = R> + DerefMut,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item.deref_mut().get_mut()
    }
}

impl<K, V, R, U, W, F, C>
    Get<K, V, PolicyCell<V, R, U>, PolicyCell<V, R, W>>
    for Policy<C, V, R, F>
where
    R: Reference<V>,
    U: Deref<Target = R>,
    W: DerefMut<Target = R>,
    F: ReferenceFactory<V, R> + Clone + Send + Sync,
    C: Get<K, R, U, W> + Ordered<R>,
{
    fn get<'a>(&'a self, key: &K) -> Option<PolicyCell<V, R, U>> {
        match self.container.get(key) {
            None => None,
            Some(x) => Some(PolicyCell {
                item: x,
                unused: PhantomData,
            }),
        }
    }

    fn get_mut<'a>(&'a mut self, key: &K) -> Option<PolicyCell<V, R, W>> {
        match self.container.get_mut(key) {
            None => None,
            Some(x) => Some(PolicyCell {
                item: x,
                unused: PhantomData,
            }),
        }
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Policy;
    use crate::container::Vector;
    use crate::policy::default::Default;
    use crate::policy::tests::test_ordered;
    use crate::tests::{test_building_block, test_get};

    #[test]
    fn building_block() {
        for i in vec![0, 10, 100] {
            test_building_block(Policy::new(Vector::new(i), Default {}));
        }
    }

    #[test]
    fn get() {
        for i in vec![0, 10, 100] {
            test_get(Policy::new(Vector::new(i), Default {}));
        }
    }

    #[test]
    fn ordered() {
        for i in vec![0, 10, 100] {
            test_ordered(Policy::new(Vector::new(i), Default {}));
        }
    }
}
