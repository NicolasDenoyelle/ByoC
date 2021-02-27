#[cfg(test)]
use crate::container::{Container, Sequential};
#[cfg(test)]
use crate::reference::Default;
#[cfg(test)]
use crate::utils::rand::rand_ab;
/// This module is for testing container.
/// Users that wish to test a container implementation should call the function
/// `test_container()` on an initialized container.

#[cfg(test)]
use std::collections::BTreeSet;

// Assert container contains a specific value.
#[cfg(test)]
fn test_contains<C>(c: &mut C, i: u32)
where
    C: Container<u16, u32, Default<u32>> + Sequential<u16, u32, Default<u32>>,
{
    if !c.contains(&(i as u16)) {
        panic!("Contains method did not return true on inserted element.")
    }

    match c.get(&(i as u16)) {
        None => {
            panic!("Inserted element could not be found with get method.")
        }
        Some(r) => {
            if *r != i {
                panic!(
                    "Get method of Container does not return expected value."
                )
            }
        }
    }
}

// Assert that value can be taken out and reinserted without implying an eviction.
// Also check that count of element remain correct.
#[cfg(test)]
fn test_take<C>(c: &mut C, i: u32)
where
    C: Container<u16, u32, Default<u32>>,
{
    let count = c.count();

    match c.take(&(i as u16)) {
        None => {
            panic!("Inserted element cannot be taken.")
        }
        Some(r) => {
            if c.count() != count - 1 {
                panic!("Taking in container does not update count correclty.")
            }
            if c.push(i as u16, r).is_some() {
                panic!("Insertion of non existing key/value in non full container pops an element.")
            }
        }
    }
}

// Assert that insertion keep key/values association working and that count update
// remain valid whether the container pops or not. Also check that the container does
// not overflow.
#[cfg(test)]
fn test_insert<C>(c: &mut C, i: u32)
where
    C: Container<u16, u32, Default<u32>> + Sequential<u16, u32, Default<u32>>,
{
    let count = c.count();
    match c.push(i as u16, Default::new(i)) {
        None => {
            if c.count() != count + 1 {
                panic!("Container does not update count on insertions.")
            }
            if c.capacity() <= count {
                panic!("Container overflows.")
            }
            test_contains(c, i);
            test_take(c, i);
        }
        Some((k, r)) => {
            if c.count() != count {
                panic!("Container poping on insertion does not update count correctly.");
            }
            if i == *r {
                if k != i as u16 {
                    panic!("Container key and value do not match.")
                }
            }
        }
    }
}

// Same as test_insert + check from a BTreeSet that popped victims are
// the expected ones. Some container like Associative do not respect exactly
// victims order. In that case test_insert() should be used.
#[cfg(test)]
fn test_insert_order<C>(c: &mut C, i: u32, set: &mut BTreeSet<u32>)
where
    C: Container<u16, u32, Default<u32>> + Sequential<u16, u32, Default<u32>>,
{
    let count = c.count();
    match c.push(i as u16, Default::new(i)) {
        None => {
            if !set.insert(i) {
                panic!("Container allow insertion of duplicate keys");
            }
            if c.count() != count + 1 {
                panic!("Container does not update count on insertions.")
            }
            if c.capacity() <= count {
                panic!("Container overflows.")
            }
            test_contains(c, i);
            test_take(c, i);
        }
        Some((k, r)) => {
            if c.count() != count {
                panic!("Container poping on insertion does not update count correctly.");
            }
            if i == *r && set.contains(&*r) {
                assert_eq!(k, i as u16);
            } else {
                if *r != i {
                    set.insert(i);
                    test_contains(c, i);
                } else {
                    assert_eq!(k, i as u16);
                }
                if set.contains(&*r) {
                    assert_eq!(set.iter().rev().next().unwrap(), &*r);
                    assert!(set.remove(&*r));
                }
            }
        }
    }
}

#[cfg(test)]
fn test_clear<C>(c: &mut C)
where
    C: Container<u16, u32, Default<u32>>,
{
    for _ in 0..c.count() {
        assert!(c.pop().is_some());
    }
    assert_eq!(c.count(), 0);
    assert!(c.pop().is_none());

    match c.push(5u16, Default::new(342u32)) {
        None => {}
        Some((k, r)) => {
            panic!("Push in empty container yields ({},{:?})", k, r);
        }
    }
    assert!(c.count() > 0);
    c.clear();
    assert_eq!(c.count(), 0);
    assert!(c.pop().is_none());
}

#[cfg(test)]
pub fn test_container<C>(mut c: C, test_order: bool)
where
    C: Container<u16, u32, Default<u32>> + Sequential<u16, u32, Default<u32>>,
{
    let mut set = BTreeSet::new();
    let num = c.capacity() * 4;
    let max = c.capacity() * 2;

    for _ in 0..num {
        let i = rand_ab(0, max as u64);
        if test_order {
            test_insert_order(&mut c, i as u32, &mut set);
        } else {
            test_insert(&mut c, i as u32);
        }
    }

    if c.capacity() > 0 {
        test_clear(&mut c);
    }
}
