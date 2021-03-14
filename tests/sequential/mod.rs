use cache::container::{Container, Sequential};
use cache::reference::Default;
use cache::timestamp::{Counter, Timestamp};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::vec::Vec;

pub fn rand(a: u64, b: u64) -> u64 {
    if a >= b {
        panic!("Empty range for random number");
    }
    let n = Counter::new();
    let mut hasher = DefaultHasher::new();
    n.hash(&mut hasher);
    hasher.finish() % (b - a) + a
}

type Reference = Default<u32>;

fn test_push<C>(c: &mut C, key: u16, value: u32)
where
    C: Container<u16, Reference> + Sequential<u16, u32, Reference>,
{
    let reference = Default::new(value);

    match c.push(key.clone(), reference.clone()) {
        None => (),
        Some((k, v)) => {
            if k != key || v != reference {
                assert!(c.get(&k).is_none());
                assert!(c.contains(&key));
                assert!(c.get(&key).is_some());
                assert!(c.get_mut(&key).is_some());
            }
        }
    };
}

fn test_n_sequential<C>(c: &mut C, n: usize)
where
    C: Container<u16, Reference> + Sequential<u16, u32, Reference>,
{
    let elements: Vec<(u16, u32)> = (0..n as u64)
        .map(|i| (i as u16, rand(0, n as u64) as u32))
        .collect();
    for (k, v) in elements {
        test_push(c, k, v);
    }
    c.clear();
}

pub fn test_sequential<C>(mut c: C)
where
    C: Container<u16, Reference> + Sequential<u16, u32, Reference>,
{
    let mut n = 0;
    test_n_sequential(&mut c, n);
    n = c.capacity() / 2;
    test_n_sequential(&mut c, n);
    n = c.capacity();
    test_n_sequential(&mut c, n);
    n = c.capacity() * 2;
    test_n_sequential(&mut c, n);
}
