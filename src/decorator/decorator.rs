use crate::decorator::DecorationFactory;
use std::marker::PhantomData;

//------------------------------------------------------------------------//
// Decoration wrapper                                                      //
//------------------------------------------------------------------------//

/// Decorator for `BuildingBlock` values.
///
/// ## [`BuildingBlock`](trait.BuildingBlock.html) Implementation
///
/// TODO
///
/// ## [`Get`](trait.Get.html) Implementation
///
/// It is critical to note that accessing values wrapped into
/// an order cell might change the order of elements in the container, and
/// therefore, policies should not be used with containers relying on
/// a stable order of their values. Containers that rely on a
/// stable order of values should not allow access to their inner values
/// altogether and should not implement the Ordering trait to avoid this problem.
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::{Array, Decorator};
/// use byoc::decorator::Fifo;
///
/// let mut c = Decorator::new(Array::new(3), Fifo::new());
/// c.push(vec![("item1",1u16), ("item2",2u16), ("item0",3u16)]);
/// assert_eq!(c.pop(1).pop().unwrap().0, "item1");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item2");
/// assert_eq!(c.pop(1).pop().unwrap().0, "item0");
///```
///
/// Policies can be added to building blocks built with a
/// [builder pattern](builder/trait.Build.html#method.with_policy) or
/// built from a
/// [configuration](config/index.html).
pub struct Decorator<C, V, F>
where
    F: DecorationFactory<V>,
{
    pub(super) container: C,
    pub(super) factory: F,
    pub(super) unused: PhantomData<V>,
}

impl<C, V, F> Decorator<C, V, F>
where
    F: DecorationFactory<V>,
{
    /// Construct a new decorator wrapper.
    pub fn new(container: C, factory: F) -> Self {
        Decorator {
            container,
            factory,
            unused: PhantomData,
        }
    }
}
impl<C, V, F> Clone for Decorator<C, V, F>
where
    F: DecorationFactory<V> + Clone,
    C: Clone,
{
    fn clone(&self) -> Self {
        Decorator {
            container: self.container.clone(),
            factory: self.factory.clone(),
            unused: PhantomData,
        }
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

// TODO
