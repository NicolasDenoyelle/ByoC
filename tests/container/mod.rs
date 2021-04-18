use cache::container::Container;
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

fn test_is_min<'a, C>(c: &mut C, value: &u32)
where
    C: Container<'a, u16, u32>,
{
    let mut elements = Vec::new();
    let count = c.count();
    loop {
        match c.pop() {
            None => break,
            Some((k, v)) => {
                assert!(value <= &v);
                elements.push((k, v));
            }
        }
        assert_eq!(elements.len() + c.count(), count);
    }
    for e in elements {
        c.push(e.0, e.1);
    }
}

fn test_push<'a, C>(c: &mut C, key: u16, value: u32)
where
    C: Container<'a, u16, u32>,
{
    let count = c.count();

    if c.contains(&key) || count == c.capacity() {
        let out = c.push(key, value).unwrap();
        if out.0 != key && out.1 < value {
            test_is_min(c, &out.1);
        }
        assert_eq!(c.count(), count);
    }
}

fn test_n_container<'a, C>(c: &mut C, n: usize)
where
    C: Container<'a, u16, u32>,
{
    let elements: Vec<(u16, u32)> = (0..n as u64)
        .map(|i| (i as u16, rand(0, n as u64) as u32))
        .collect();
    for (k, v) in elements.iter() {
        test_push(c, *k, *v);
    }

    for (k, v) in c.flush() {
        assert!(elements
            .iter()
            .find(|(_k, _v)| { _k == &k && _v == &v })
            .is_some());
    }
}

pub fn test_container<'a, C>(mut c: C)
where
    C: Container<'a, u16, u32>,
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
