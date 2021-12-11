use crate::policies::{Reference, ReferenceFactory};
use crate::{BuildingBlock, Concurrent, Get, GetMut, Ordered, Prefetch};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// Reference wrapper                                                      //
//------------------------------------------------------------------------//

/// Eviction policy for `Ordered` building blocks.
///
/// This structure implements a wrapper around a building blocks that
/// wraps values (from key/value pairs) into an orderable cell.
/// As a result, when popping elements out of a building blocks
/// implementing [`Ordered`](../trait.Ordered.html) trait,
/// this wrapper decides which element is going to be evicted.
///
/// Users beware that accessing values wrapped into
/// an order cell might change the order of elements in the container, and
/// therefore, policies should not be used with containers relying on
/// a stable order of its values. Note that containers that rely on a
/// stable order of values should not allow access to their inner values
/// alltogether to avoid this problem.
///
/// # Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::{Array, Policy};
/// use cache::policies::FIFO;
///
/// let mut c = Policy::new(Array::new(3), FIFO::new());
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
            container,
            factory,
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
        self.container.take(key).map(|(k, r)| (k, r.unwrap()))
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

impl<'a, V, C, R, F> Concurrent for Policy<C, V, R, F>
where
    V: 'a,
    R: 'a + Reference<V>,
    F: ReferenceFactory<V, R> + Clone + Send + Sync,
    C: Concurrent,
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

/// A Cell wrapping elements borrowed from inside a `Policy` building block.
///
/// It can be dereferenced through the wrapped building block element cell
/// to obtain original value.
///
/// # Safety:
///
/// The safety of using this cell depends on the safety of using the wrapped
/// element cell.
pub struct PolicyCell<V, R, U>
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

impl<K, V, R, U, F, C> Get<K, V, PolicyCell<V, R, U>>
    for Policy<C, V, R, F>
where
    R: Reference<V>,
    U: Deref<Target = R>,
    F: ReferenceFactory<V, R> + Clone + Send + Sync,
    C: Get<K, R, U> + Ordered<R>,
{
    unsafe fn get(&self, key: &K) -> Option<PolicyCell<V, R, U>> {
        self.container.get(key).map(|x| PolicyCell {
            item: x,
            unused: PhantomData,
        })
    }
}

impl<K, V, R, W, F, C> GetMut<K, V, PolicyCell<V, R, W>>
    for Policy<C, V, R, F>
where
    R: Reference<V>,
    W: DerefMut<Target = R>,
    F: ReferenceFactory<V, R> + Clone + Send + Sync,
    C: GetMut<K, R, W> + Ordered<R>,
{
    unsafe fn get_mut(&mut self, key: &K) -> Option<PolicyCell<V, R, W>> {
        self.container.get_mut(key).map(|x| PolicyCell {
            item: x,
            unused: PhantomData,
        })
    }
}

//------------------------------------------------------------------------//
// Prefetch Trait Implementation
//------------------------------------------------------------------------//

impl<'a, K, V, C, R, F> Prefetch<'a, K, V> for Policy<C, V, R, F>
where
    K: 'a,
    V: 'a,
    R: 'a + Reference<V>,
    C: BuildingBlock<'a, K, R> + Prefetch<'a, K, R>,
    F: ReferenceFactory<V, R>,
{
    fn prefetch(&mut self, keys: Vec<K>) {
        self.container.prefetch(keys)
    }

    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        self.container
            .take_multiple(keys)
            .into_iter()
            .map(|(k, r)| (k, r.unwrap()))
            .collect()
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Policy;
    use crate::policies::Default;
    use crate::tests::{
        test_building_block, test_get, test_get_mut, test_ordered,
        test_prefetch,
    };
    use crate::Array;

    #[test]
    fn building_block() {
        for i in [0usize, 10usize, 100usize] {
            test_building_block(Policy::new(Array::new(i), Default {}));
        }
    }

    #[test]
    fn get() {
        for i in [0usize, 10usize, 100usize] {
            test_get(Policy::new(Array::new(i), Default {}));
            test_get_mut(Policy::new(Array::new(i), Default {}));
        }
    }

    #[test]
    fn prefetch() {
        for i in [0usize, 10usize, 100usize] {
            test_prefetch(Policy::new(Array::new(i), Default {}));
        }
    }

    #[test]
    fn ordered() {
        for i in [0usize, 10usize, 100usize] {
            test_ordered(Policy::new(Array::new(i), Default {}));
        }
    }
}
