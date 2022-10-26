use super::Array;
use crate::builder::Build;
use std::marker::PhantomData;

/// `Array` builder.
///
/// This builder can be consumed later to spawn an
/// [`Array`](../../struct.Array.html) container.
///
/// ## Examples
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::builder::ArrayBuilder;
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

impl<T> Build<Array<T>> for ArrayBuilder<T> {
    fn build(self) -> Array<T> {
        Array::new(self.capacity)
    }
}
