use crate::reference::Reference;
use std::cmp::Ord;
use std::ops::{Deref, DerefMut};

//----------------------------------------------------------------------------//
// Default reference                                                          //
//----------------------------------------------------------------------------//

/// Implementation of [`Reference`](trait.Reference.html) as its inner value.
///
/// ## Examples
///
/// ```
/// use cache::reference::{Reference, Default};
///
/// let p0 = Default::new("item0");
/// let p1 = Default::new("item1");
/// assert!( p0 < p1 );
/// assert!( p1 > p0 );
/// assert!( p0 == p0 );
/// assert!( p1 == p1 );
/// ```
#[derive(Debug, PartialOrd, Ord, Eq, PartialEq)]
pub struct Default<V: Ord> {
    value: V,
}

impl<V: Ord> Default<V> {
    pub fn new(v: V) -> Self {
        Default { value: v }
    }
}

impl<V: Ord> Deref for Default<V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V: Ord> DerefMut for Default<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<V: Ord> Reference<V> for Default<V> {}
