use super::BTree;
use crate::utils::get::LifeTimeGuard;
use crate::{BuildingBlock, GetMut};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// Cell representing a writable value inside a
/// [`BTree`](struct.BTree.html).
///
/// This value inside this cell is taken out of the container and written
/// back in it when the cell is dropped.
pub struct BTreeCell<K: Copy + Ord, V: Ord> {
    kv: Option<(K, V)>,
    set: NonNull<BTree<K, V>>,
}

impl<K: Copy + Ord, V: Ord> Deref for BTreeCell<K, V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.kv.as_ref().unwrap().1
    }
}

impl<K: Copy + Ord, V: Ord> DerefMut for BTreeCell<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kv.as_mut().unwrap().1
    }
}

impl<K: Copy + Ord, V: Ord> Drop for BTreeCell<K, V> {
    fn drop(&mut self) {
        let set = unsafe { self.set.as_mut() };
        let kv = self.kv.take().unwrap();
        assert!(set.push(vec![kv]).pop().is_none());
    }
}

impl<K: Copy + Ord, V: Ord> GetMut<K, V> for BTree<K, V> {
    type Target = BTreeCell<K, V>;

    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        self.take(key).map(|(key, value)| {
            LifeTimeGuard::new(BTreeCell {
                kv: Some((key, value)),
                set: NonNull::new(self).unwrap(),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::BTree;
    use crate::tests::test_get_mut;

    #[test]
    fn get() {
        test_get_mut(BTree::new(0));
        test_get_mut(BTree::new(10));
        test_get_mut(BTree::new(100));
    }
}
