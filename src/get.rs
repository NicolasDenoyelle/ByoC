use std::ops::{Deref, DerefMut};

/// This is a companion trait for
/// [`BuildingBlock`](trait.BuildingBlock.html)
/// trait to have access to values inside of a building block.
///
/// When a building block implements this trait, it provides access to
/// values inside itself.
/// Values are wrapped in a Cell that can be dereferenced to obtain
/// a reference to the value matching the key in the building block.
pub trait Get<K, V, U>
where
    U: Deref<Target = V>,
{
    /// Get a read-only smart pointer to a value inside the container.
    ///
    /// ## Safety
    ///
    /// At this time, it does not seam feasible to return a trait object
    /// with the same lifetime as the function call. Therefore, any lifetime
    /// inference on the returned structure would require it to have the
    /// same lifetime as the building block instance which would for
    /// instance prevent to call this trait method in a loop. As a result,
    /// this trait implementation maybe `unsafe`, because the returned
    /// guard lifetime may outlive the borrowing lifetime of the container
    /// where the inner value originates from.
    unsafe fn get(&self, key: &K) -> Option<U>;
}
/// This is a companion trait for
/// [`BuildingBlock`](trait.BuildingBlock.html)
/// trait to have access to a mutable values inside of a building block.
///
/// When a building block implements this trait, it provides access to
/// mutable values inside itself.
/// Values are wrapped in a Cell that can be dereferenced to obtain
/// a reference to the value matching the key in the building block.
/// This trait is separated from [`Get`](trait.Get.html) because
/// some containers ([BTree](struct.BTree.html)) have to
/// be mutated when they are accessed, hence they can implement `get_mut()`
/// but not `get()`. These two traits may also require different trait
/// bounds because, for instance int the former the value can be moved
/// from a building block not implementing `GetMut` to one implementing
/// it and returning the value from there
/// (See [`Exclusive`](struct.Exclusive.html)).
pub trait GetMut<K, V, W>
where
    W: Deref<Target = V> + DerefMut,
{
    /// Get a smart pointer to a mutable value inside the container.
    ///
    /// ## Safety
    ///
    /// At this time, it does not seam feasible to return a trait object
    /// with the same lifetime as the function call. Therefore, any lifetime
    /// inference on the returned structure would require it to have the
    /// same lifetime as the building block instance which would for
    /// instance prevent to call this trait method in a loop. As a result,
    /// this trait implementation maybe `unsafe`, because the returned
    /// guard lifetime may outlive the borrowing lifetime of the container
    /// where the inner value originates from.
    unsafe fn get_mut(&mut self, key: &K) -> Option<W>;
}
