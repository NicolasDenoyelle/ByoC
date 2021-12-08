use crate::builder::traits::{
    Associative, Builder, Multilevel, Policy, Profiler, Sequential,
};
use crate::container::Array;
use std::marker::PhantomData;

/// [Array](../../container/struct.Array.html)
/// [builder](../traits/trait.Builder.html).
///
/// This builder can be consumed later to spawn an
/// [Array](../../container/struct.Array.html) container.
///
/// ## Examples
/// ```
/// use cache::BuildingBlock;
/// use cache::builder::traits::*;
/// use cache::builder::builders::ArrayBuilder;
///
/// let mut array = ArrayBuilder::new(2).build();
/// array.push(vec![(1, 2)]);
/// ```
pub struct ArrayBuilder<T> {
    capacity: usize,
    unused: PhantomData<T>,
}

impl<T> ArrayBuilder<T> {
    /// The [Array](../../container/array/struct.Array.html) container
    /// spawned by this builder will have a maximum `capacity`.
    pub fn new(capacity: usize) -> Self {
        ArrayBuilder {
            capacity: capacity,
            unused: PhantomData,
        }
    }
}

impl<T> Clone for ArrayBuilder<T> {
    fn clone(&self) -> Self {
        ArrayBuilder {
            capacity: self.capacity,
            unused: PhantomData,
        }
    }
}

impl<T> Policy<Array<T>> for ArrayBuilder<T> {}
impl<T> Associative<Array<T>> for ArrayBuilder<T> {}
impl<T> Sequential<Array<T>> for ArrayBuilder<T> {}
impl<T> Profiler<Array<T>> for ArrayBuilder<T> {}

impl<T, R, RB: Builder<R>> Multilevel<Array<T>, R, RB> for ArrayBuilder<T> {}

impl<T> Builder<Array<T>> for ArrayBuilder<T> {
    fn build(self) -> Array<T> {
        Array::new(self.capacity)
    }
}
