use crate::utils::get::LifeTimeGuard;
use std::ops::{Deref, DerefMut};

/// Get exclusive read-only access to a value inside of a `BuildingBlock`.
///
/// When a building block implements this trait, it provides access to
/// values inside itself.
/// The [`get()`](trait.Get.html#tymehod.get) returns an option.
/// If the key is not found in the [`BuildingBlock`](trait.BuildingBlock.html)
/// it returns `None`, otherwise it returns value are wrapped inside of a cell
/// that can be dereferenced read it.
///
/// Accesses, even read-only may have a side effect
/// of modifying the container internal state. Therefore, the `get()`
/// method requires exclusive access to the container.
/// Compared to using [`GetMut`] trait, this method might have less overhead
/// since there is no need to update the container as a result of a modified
/// value, such as writing a modified value back to disk.
///
/// ## Example:
///
/// ```
/// use byoc::{BuildingBlock, Get};
/// use byoc::Array;
/// use std::ops::Deref;
///
/// // Make a array and populate it.
/// let mut v = Array::new(1);
/// v.push(vec![(1,1)]);
///
/// // Get the value inside the array.
/// assert_eq!(*v.get(&1).unwrap(), 1);
/// ```
pub trait Get<K, V> {
    type Target: Deref<Target = V>;
    /// Get a read-only smart pointer to a value inside the container.
    fn get(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>>;
}

/// Get exclusive write access to a value inside of a `BuildingBlock`.
///
/// When a building block implements this trait, it provides mutable and
/// exclusive access to values inside itself.
/// This is a sibling trait to [`Get`] trait. See [`Get`] trait documentation
/// for more details.
///
/// ## Example:
///
/// ```
/// use byoc::{BuildingBlock, GetMut};
/// use byoc::Array;
///
/// // Make a array and populate it.
/// let mut v = Array::new(1);
/// v.push(vec![(1,1)]);
///
/// // Modify a value inside the array.
/// *v.get_mut(&1).unwrap() = 2;
/// assert_eq!(v.take(&1).unwrap().1, 2);
/// ```
pub trait GetMut<K, V> {
    type Target: Deref<Target = V> + DerefMut;

    /// Get a smart pointer to a mutable value inside the container.
    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>>;
}
