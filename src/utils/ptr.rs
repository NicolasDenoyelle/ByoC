use std::cmp::{Ord, Ordering};
use std::ptr::NonNull;
//------------------------------------------------------------------------------------//
// Unsafe raw pointer implementing Ord.                                     //
//------------------------------------------------------------------------------------//

/// Unsafe raw pointer implementing Ord.
/// The purpose of this struct is to create lightweight orderable
/// pointers to an existing structure.
/// Pointer must not outlive the reference it points to.
/// Reference must point to the same memory location while OrdPtr
/// is in use.
/// The reference must be initialized.
#[derive(Debug)]
pub struct OrdPtr<T: Ord> {
    ptr: NonNull<T>,
}

impl<T: Ord> OrdPtr<T> {
    pub fn new(value: &T) -> OrdPtr<T> {
        OrdPtr {
            ptr: NonNull::<T>::from(value),
        }
    }
}

impl<T: Ord> Copy for OrdPtr<T> {}
impl<T: Ord> Clone for OrdPtr<T> {
    fn clone(&self) -> Self {
        OrdPtr {
            ptr: self.ptr.clone(),
        }
    }
}

impl<T: Ord> Ord for OrdPtr<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { self.ptr.as_ref().cmp(other.ptr.as_ref()) }
    }
}

impl<T: Ord> PartialOrd for OrdPtr<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unsafe { self.ptr.as_ref().partial_cmp(other.ptr.as_ref()) }
    }
}

impl<T: Ord> PartialEq for OrdPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.ptr.as_ref().eq(other.ptr.as_ref()) }
    }
}

impl<T: Ord> Eq for OrdPtr<T> {}

#[cfg(test)]
mod tests {
    use super::OrdPtr;

    // use std::collections::{BTreeSet, BTreeMap};
    // Do not work because references in btm may change.
    // #[test]
    // fn test_btree() {
    //     let mut bts: BTreeSet<OrdPtr<u32>> = BTreeSet::new();
    //     let mut btm = BTreeMap<*mut u32>::new();

    //     for i in 0u32..1000u32 {
    //         assert!(btm.insert(i, &mut i as *mut u32).is_none());
    //         let v = btm.get_mut(&i).unwrap();
    //         assert!(bts.insert(OrdPtr::new(*v)));
    //     };

    //     for i in 0u32..1000u32 {
    //         let v = btm.get_mut(&i).unwrap();
    //         assert!(bts.remove(&OrdPtr::new(*v)));
    //         assert!(btm.remove(&i).is_some());
    //     }
    // }

    #[test]
    fn test_simple() {
        let x = OrdPtr::new(&65u64);
        let y = OrdPtr::new(&66u64);

        assert!(x < y);
        assert!(y > x);
        assert!(x == OrdPtr::new(&65u64));
        assert!(x == x.clone());
    }
}
