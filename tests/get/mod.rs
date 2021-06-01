extern crate rand;

use cache::container::{Container, Get};
use rand::random;
use std::vec::Vec;

pub fn rand(a: u64, b: u64) -> u64 {
    a + (random::<u64>() % (b - a))
}

fn test_push<'a, C>(c: &mut C, key: u16, value: u32)
where
    C: Container<'a, u16, u32> + Get<'a, u16, u32>,
{
    // Test insertion
    match c.push(key, value) {
        Some((k, _)) => {
            if k != key {
                assert!(c.get(&key).any(|(k, _)| k == &key));
            }
        }
        None => {
            assert!(c.get(&key).any(|(k, _)| k == &key));
        }
    }
}

pub fn test_n<'a, C>(c: &mut C, n: usize)
where
    C: Container<'a, u16, u32> + Get<'a, u16, u32>,
{
    let elements: Vec<(u16, u32)> = (0..n as u64)
        .map(|i| (i as u16, rand(0, n as u64) as u32))
        .collect();

    for (k, v) in elements.iter() {
        test_push(c, *k, *v);
    }
}

pub fn test_get<'a, C>(mut c: C)
where
    C: Container<'a, u16, u32> + Get<'a, u16, u32>,
{
    let mut n = 0;
    test_n(&mut c, n);
    n = c.capacity() / 2;
    test_n(&mut c, n);
    n = c.capacity();
    test_n(&mut c, n);
    n = c.capacity() * 2;
    test_n(&mut c, n);
}
