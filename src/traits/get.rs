use crate::utils::get::LifeTimeGuard;
use std::ops::{Deref, DerefMut};

/// This is a companion trait for
/// [`BuildingBlock`](trait.BuildingBlock.html)
/// trait to have read-only access to values inside of a building block.
///
/// When a building block implements this trait, it provides access to
/// values inside itself.
/// Values are wrapped in a Cell that can be dereferenced to obtain
/// a shared reference to the value matching the key in the building block.
/// Accesses, even read-only may have a side effect
/// of modifying the container internal state. Therefore, the `get()`
/// method requires exclusive access to the container. Otherwise, it might
/// be possible to hold invalid container references.
/// Compared to using [`GetMut`] trait, this method might have less overhead
/// since there is no need to update the container as a result of a modified
/// value.
pub trait Get<K, V> {
    type Target: Deref<Target = V>;
    /// Get a read-only smart pointer to a value inside the container.
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
    fn get(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>>;
}

/// This is a companion trait for
/// [`BuildingBlock`](trait.BuildingBlock.html)
/// trait to have access to a mutable values inside of a building block.
///
/// When a building block implements this trait, it provides access to
/// mutable values inside itself.
/// Values are wrapped in a Cell that can be dereferenced to obtain
/// a reference to the value matching the key in the building block.
pub trait GetMut<K, V> {
    type Target: Deref<Target = V> + DerefMut;

    /// Get a smart pointer to a mutable value inside the container.
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
    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>>;
}
