use crate::policy::{Reference, ReferenceFactory};
use std::cmp::Ord;

//----------------------------------------------------------------------------//
// Default reference                                                          //
//----------------------------------------------------------------------------//

/// Implementation of [`Reference`](trait.Reference.html) as its inner value.
///
/// ## Examples
///
/// ```
/// use byoc::container::Array;
/// use byoc::policy::{Policy, Default};
///
/// let c = Policy::new(Array::new(3), Default::new());
/// c.push(vec!["item1", "item2", "item0"]);
/// assert_eq!(c.pop(1).pop().unwrap(), "item2");
/// assert_eq!(c.pop(1).pop().unwrap(), "item1");
/// assert_eq!(c.pop(1).pop().unwrap(), "item0");
/// ```
#[derive(Debug, PartialOrd, Ord, Eq, PartialEq)]
pub struct DefaultCell<V: Ord> {
    value: V,
}

impl<V: Ord> DefaultCell<V> {
    pub fn new(v: V) -> Self {
        DefaultCell { value: v }
    }
}

impl<V: Ord> Reference<V> for DefaultCell<V> {
    fn unwrap(self) -> V {
        self.value
    }
    fn get(&self) -> &V {
        &self.value
    }
    fn get_mut(&mut self) -> &mut V {
        &mut self.value
    }
}

#[derive(Clone)]
pub struct Default {}

impl<V: Ord> ReferenceFactory<V> for Default {
    type Item = DefaultCell<V>;

    fn wrap(&mut self, v: V) -> Self::Item {
        DefaultCell { value: v }
    }
}

unsafe impl Send for Default {}
unsafe impl Sync for Default {}

#[cfg(test)]
mod tests {
    use super::DefaultCell;

    #[test]
    fn test_default_ref() {
        let p0 = DefaultCell::new("item0");
        let p1 = DefaultCell::new("item1");
        assert!(p0 < p1);
        assert!(p1 > p0);
        // assert!(p0 == p0);
        // assert!(p1 == p1);
    }
}
