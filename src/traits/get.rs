use crate::utils::get::LifeTimeGuard;
use std::ops::{Deref, DerefMut};

/// This is a companion trait for
/// [`BuildingBlock`](trait.BuildingBlock.html)
/// trait to have access to values inside of a building block.
///
/// When a building block implements this trait, it provides access to
/// values inside itself.
/// Values are wrapped in a Cell that can be dereferenced to obtain
/// a reference to the value matching the key in the building block.
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
    /// let element_size = Array::<(u32,u32)>::element_size();
    /// let mut v = Array::new(element_size);
    /// v.push(vec![(1,1)]);
    ///
    /// // Get the value inside the array.
    /// assert_eq!(*v.get(&1).unwrap(), 1);
    /// ```
    fn get(&self, key: &K) -> Option<LifeTimeGuard<Self::Target>>;
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
/// but not `get()`.
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
    /// let element_size = Array::<(u32,u32)>::element_size();
    /// let mut v = Array::new(element_size);
    /// v.push(vec![(1,1)]);
    ///
    /// // Modify a value inside the array.
    /// *v.get_mut(&1).unwrap() = 2;
    /// assert_eq!(v.take(&1).unwrap().1, 2);
    /// ```
    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>>;
}
