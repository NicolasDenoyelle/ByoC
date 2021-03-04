pub mod clone;

use cache::container::{Concurrent, Container};
use cache::reference::Default;
use clone::CloneMut;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;

type Reference = Default<u32>;

fn test_insert_sequential<C>(
    c: &mut C,
    i: u32,
    set: &mut BTreeSet<u32>,
    eviction_check: bool,
) -> bool
where
    C: Container<u16, u32, Default<u32>>,
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
            if !c.contains(&(i as u16)) {
                panic!(
                    "Contains method did not return true on inserted element."
                )
            }
            if !set.insert(i) {
                panic!("Container allow insertion of duplicate keys");
            }
            true
        }
        Some((k, r)) => {
            if c.count() != count {
                panic!("Container poping on insertion does not update count correctly.");
            }
            if k != *r as u16 {
                panic!("Container key and value do not match.")
            }
            if i != *r {
                if !c.contains(&(i as u16)) {
                    panic!("Contains method did not return true on inserted element.")
                }
                if !set.insert(i) {
                    panic!("Container allow insertion of duplicate key and pop other key.")
                }
                if !set.contains(&*r) {
                    panic!("Eviction of an element that does not belong to container");
                }
                if eviction_check {
                    assert_eq!(set.iter().rev().next().unwrap(), &*r);
                }
                assert!(set.remove(&*r));
                true
            } else {
                false
            }
        }
    }
}

// Test the container for the presence of an element and that returned value is correct.
fn test_get<C>(c: &mut C, i: u32)
where
    C: Container<u16, u32, Default<u32>> + Concurrent<u16, u32, Default<u32>>,
{
    match c.get(&(i as u16)) {
        None => {
            panic!("Inserted element could not be found with get method.")
        }
        Some(r) => {
            if **r != i {
                panic!(
                    "Get method of Container does not return expected value."
                )
            }
        }
    }
}

// Pop() count() elements. Test that container is empty after that.
// Reinsert one element and call clear(). Test that container is empty after that.
fn test_clear_sequential<C>(c: &mut C)
where
    C: Container<u16, u32, Default<u32>>,
{
    let num = if c.capacity() < 10 { c.capacity() } else { 10 };
    for i in 0..num {
        c.push(i as u16, Default::new(i as u32));
    }

    for _ in 0..c.count() {
        assert!(c.pop().is_some());
    }
    assert_eq!(c.count(), 0);
    assert!(c.pop().is_none());

    for i in 0..num {
        c.push(i as u16, Default::new(i as u32));
    }
    c.clear();
    assert_eq!(c.count(), 0);
    assert!(c.pop().is_none());
}

// Assert that value can be taken out and reinserted without implying an eviction.
// Also check that count of element remain correct.
fn test_take_sequential<C>(c: &mut C, i: u32)
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

pub fn test_sequential<C>(mut c: C, eviction_check: bool)
where
    C: Container<u16, u32, Default<u32>> + Concurrent<u16, u32, Default<u32>>,
{
    let start = Instant::now();
    let mut set = BTreeSet::new();
    let num = c.capacity() * 4;
    let max = c.capacity() * 2;

    if c.capacity() > 0 {
        test_clear_sequential(&mut c);
    }

    for _ in 0..num {
        let i =
            (Instant::now().duration_since(start).as_nanos() as usize) % max;
        if test_insert_sequential(&mut c, i as u32, &mut set, eviction_check) {
            test_get(&mut c, i as u32);
            test_take_sequential(&mut c, i as u32);
        }
    }
}

pub fn test_concurrent<C>(c: C)
where
    C: 'static
        + Container<u16, u32, Default<u32>>
        + Concurrent<u16, u32, Default<u32>>,
{
    let index_max = c.capacity();
    let num_threads = 64;
    let mut threads: Vec<JoinHandle<_>> = Vec::with_capacity(num_threads);
    let count = Arc::new(AtomicU64::new(0));
    let container = CloneMut::new(c);

    for i in 0..num_threads {
        let begin = i * index_max / num_threads;
        let mut end = index_max / num_threads;
        end += if i + 1 == num_threads {
            index_max % num_threads
        } else {
            0
        };
        let local_index: Vec<usize> = (begin..end).collect();
        let local_count = count.clone();
        let mut local_container = container.clone();
        threads.push(thread::spawn(move || {
            for j in local_index.iter().map(|i| i % index_max) {
                let r = Default::new(j as u32);
                match local_container.push(j as u16, r) {
                    None => {
                        local_count.fetch_add(1, Ordering::SeqCst);
                    }
                    Some(_) => {}
                }
            }
        }));
    }

    for _ in 0..num_threads {
        threads.pop().unwrap().join().unwrap();
    }

    assert!(container.count() == count.load(Ordering::SeqCst) as usize);
}
