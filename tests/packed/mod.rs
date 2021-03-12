use cache::container::{Container, Packed};
use cache::reference::Default;
use cache::timestamp::{Counter, Timestamp};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::vec::Vec;

pub fn rand(a: u64, b: u64) -> u64 {
    assert!(a < b);
    let n = Counter::new();
    let mut hasher = DefaultHasher::new();
    n.hash(&mut hasher);
    hasher.finish() % (b - a) + a
}

type Reference = Default<u32>;

fn test_is_max<C>(c: &mut C, value: &Default<u32>, inserted_key: u16)
where
    C: Container<u16, Reference> + Packed<u16, Reference>,
{
    let mut elements = Vec::new();
    let count = c.count();
    loop {
        match c.pop() {
            None => break,
            Some((k, v)) => {
                // If k is the key that just got inserted,
                // then v might greater than popped reference.
                if k != inserted_key {
                    assert!(value >= &v);
                }
                elements.push((k, v));
            }
        }
        assert_eq!(elements.len() + c.count(), count);
    }
    for e in elements {
        c.push(e.0, e.1);
    }
}

fn test_push<C>(c: &mut C, key: u16, value: u32)
where
    C: Container<u16, Reference> + Packed<u16, Reference>,
{
    let count = c.count();
    let reference = Default::new(value);

    if c.contains(&key) {
        match c.push(key, reference) {
            None => panic!("Pushing existing key does not pop."),
            Some((k, _)) => {
                if k != key {
                    panic!("Pushing existing key does not pop same key.")
                }
            }
        }
        assert_eq!(c.count(), count);
    } else if count == c.capacity() {
        match c.push(key, reference) {
            None => panic!("Pushing in full container does not pop."),
            Some((k, v)) => {
                if k != key {
                    test_is_max(c, &v, key);
                }
            }
        }
        assert_eq!(c.count(), count);
    } else {
        assert!(c.push(key, reference).is_none());
        assert_eq!(c.count(), count + 1);
        assert!(c.contains(&key));
    }
}

fn test_n_container<C>(c: &mut C, n: usize)
where
    C: Container<u16, Reference> + Packed<u16, Reference>,
{
    let elements: Vec<(u16, u32)> = (0..n as u64)
        .map(|i| (i as u16, rand(0, n as u64) as u32))
        .collect();
    for (k, v) in elements {
        test_push(c, k, v);
    }
    c.clear();
}

pub fn test_container<C>(mut c: C)
where
    C: Container<u16, Reference> + Packed<u16, Reference>,
{
    let mut n = 0;
    test_n_container(&mut c, n);
    n = c.capacity() / 2;
    test_n_container(&mut c, n);
    n = c.capacity();
    test_n_container(&mut c, n);
    n = c.capacity() * 2;
    test_n_container(&mut c, n);
}
