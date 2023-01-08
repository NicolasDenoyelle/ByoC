/// `BuildingBlock` wrapper disabling the `flush()` method.
///
/// This [`BuildingBlock`](trait.BuildingBlock.html) forwards its methods call
/// to the [`BuildingBlock`](trait.BuildingBlock.html) it
/// wraps except for the [`flush()`](struct.FlushStopper.html#method.flush)
/// method. This method will always return an empty iterator.
///
/// This is useful when used in a multilevel cache such as
/// [`Exclusive`](struct.Exclusive.html) or [`Inclusive`](struct.Inclusive.html)
/// to flush elements to a deeper level while freeing some the first levels
/// without actually taking every elements out of the container.
///
/// ## Examples
///
/// This example shows the effect of a flush stopper with an
/// [`Exclusive`](struct.Exclusive.html)
/// cache.
///
/// ```
/// use byoc::{BuildingBlock};
/// use byoc::{Array, Exclusive, FlushStopper};
///
/// let mut container = Exclusive::new(Array::new(1),
///                                    FlushStopper::new(Array::new(2)));
/// container.push(vec![("key", "value")]);
///
/// assert!(container.front().contains(&"key"));
///
/// // After this call the inserted element is not flushed and is no longer in
/// // the front of the exclusive container.
/// assert!(container.flush().next().is_none());
/// assert!(!container.front().contains(&"key"));
/// assert!(container.back().contains(&"key"));
/// ```
///
/// This example shows the effect of a flush stopper with an
/// [`Inclusive`](struct.Inclusive.html)
/// cache.
///
/// ```
/// use byoc::{BuildingBlock, Get};
/// use byoc::{Array, Inclusive, FlushStopper};
///
/// let mut container = Inclusive::new(Array::new(1),
///                                    FlushStopper::new(Array::new(2)));
/// container.push(vec![("key", "value")]);
///
/// // Bring element to the front with an access.
/// container.get(&"key");
/// assert!(container.front().contains(&"key"));
///
/// // After this call the inserted element is not flushed and is no longer in
/// // the front of the inclusive container.
/// assert!(container.flush().next().is_none());
/// assert!(!container.front().contains(&"key"));
/// assert!(container.back().contains(&"key"));
/// ```
#[derive(Clone, Debug)]
pub struct FlushStopper<C> {
    pub(super) container: C,
}

impl<C> FlushStopper<C> {
    pub fn new(container: C) -> Self {
        FlushStopper { container }
    }
}
