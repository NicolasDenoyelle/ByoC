use crate::builder::traits::{
    Associative, Builder, Multilevel, Policy, Profiler, Sequential,
};
use crate::Array;
use std::marker::PhantomData;

/// `Array` builder.
///
/// This builder can be consumed later to spawn an
/// [`Array`](../../struct.Array.html) container.
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
    /// The [Array](../../struct.Array.html) container
    /// spawned by this builder will have a maximum `capacity`.
    pub fn new(capacity: usize) -> Self {
        ArrayBuilder {
            capacity,
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

impl<T, R, RB: Builder<R>> Multilevel<Array<T>, R, RB>
    for ArrayBuilder<T>
{
}

impl<T> Builder<Array<T>> for ArrayBuilder<T> {
    fn build(self) -> Array<T> {
        Array::new(self.capacity)
    }
}
