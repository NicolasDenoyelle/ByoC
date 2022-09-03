use crate::policy::{Ordered, Reference, ReferenceFactory};
use std::marker::PhantomData;

//------------------------------------------------------------------------//
// Reference wrapper                                                      //
//------------------------------------------------------------------------//

/// Eviction policy for [`Ordered`](policy/trait.Ordered.html)
/// [`BuildingBlock`](trait.BuildingBlock.html).
///
/// This structure implements a wrapper around building blocks that
/// wraps its values into an orderable cell.
/// As a result, when popping elements out of this building block, when
/// the underlying [`BuildingBlock`](trait.BuildingBlock.html)
/// implements [`Ordered`](../trait.Ordered.html) trait,
/// the policy of this wrapper decides which element is going to be evicted.
///
/// It is critical to note that accessing values wrapped into
/// an order cell might change the order of elements in the container, and
/// therefore, policies should not be used with containers relying on
/// a stable order of their values. Containers that rely on a
/// stable order of values should not allow access to their inner values
/// altogether and should not implement the Ordered trait to avoid this problem.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::{Array, Policy};
/// use byoc::policy::Fifo;
///
/// let mut c = Policy::new(Array::new(3), Fifo::new());
/// c.push(vec![("item1",()), ("item2",()), ("item0",())]);
/// assert_eq!(c.pop(1).pop().unwrap().0, "item1");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item2");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item0");
///```
///
/// Policies can be added to building blocks built with a
/// [builder pattern](builder/trait.Build.html#method.with_policy) or
/// built from a
/// [configuration](config/index.html).
pub struct Policy<C, V, R, F>
where
    C: Ordered<R>,
    R: Reference<V>,
    F: ReferenceFactory<V, R>,
{
    pub(super) container: C,
    pub(super) factory: F,
    pub(super) unused: PhantomData<(R, V)>,
}

impl<C, V, R, F> Policy<C, V, R, F>
where
    C: Ordered<R>,
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
impl<C, V, R, F> Clone for Policy<C, V, R, F>
where
    R: Reference<V>,
    F: ReferenceFactory<V, R> + Clone,
    C: Ordered<R> + Clone,
{
    fn clone(&self) -> Self {
        Policy {
            container: self.container.clone(),
            factory: self.factory.clone(),
            unused: PhantomData,
        }
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Policy;
    use crate::policy::Default;
    use crate::tests::test_ordered;
    use crate::Array;

    #[test]
    fn ordered() {
        for i in [0usize, 10usize, 100usize] {
            test_ordered(Policy::new(Array::new(i), Default {}));
        }
    }
}