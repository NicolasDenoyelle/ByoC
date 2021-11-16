extern crate rand;

use cache::container::Container;
use rand::random;
use std::vec::Vec;

pub fn rand(a: u64, b: u64) -> u64 {
    a + (random::<u64>() % (b - a))
}

fn test_push<'a, C>(c: &mut C, key: u16, value: u32, packed: bool)
where
    C: Container<'a, u16, u32>,
{
    let count = c.count();
    let capacity = c.capacity();

    // Test insertion
    match c.push(vec![(key, value); 1]).pop() {
        Some((k, v)) => {
            // Insertion popped then count is not updated.
            assert_eq!(c.count(), count);
            if packed {
                assert_eq!(capacity, count);
            }
            // If no capacity container, inserted key and value go out.
            if c.capacity() == 0 {
                assert_eq!(k, key);
                assert_eq!(v, value);
            }
            // When popping different key, inserted element must be present.
            else if k != key {
                assert!(c.contains(&key));
                // Take and reinsert of just inserted key must work.
                let (k, v) = c.take(&key).next().unwrap();
                assert_eq!(key, k);
                assert!(c.push(vec![(k, v); 1]).pop().is_none());
            }
        }
        None => {
            // Cannot insert in full container.
            assert!(count < c.capacity());
            // Insertion updates count.
            assert_eq!(count + 1, c.count());
        }
    }
}

fn test_flush<'a, C>(c: &mut C, pushed: &Vec<(u16, u32)>)
where
    C: Container<'a, u16, u32>,
{
    let mut i = 0;
    let count = c.count();
    for (k, v) in c.flush() {
        assert!(pushed
            .iter()
            .find(|(_k, _v)| { _k == &k && _v == &v })
            .is_some());
        i += 1;
    }
    assert_eq!(i, count);
    assert_eq!(c.count(), 0);
}

fn test_pop<'a, C>(c: &mut C, pushed: &Vec<(u16, u32)>)
where
    C: Container<'a, u16, u32>,
{
    let mut i = 0;
    let count = c.count();
    loop {
        let (k, v) = match c.pop(1).pop() {
            None => break,
            Some(x) => x,
        };
        assert!(pushed
            .iter()
            .find(|(_k, _v)| { _k == &k && _v == &v })
            .is_some());
        i += 1;
    }
    assert_eq!(i, count);
    assert_eq!(c.count(), 0);
}

pub fn test_n<'a, C>(c: &mut C, n: usize, packed: bool)
where
    C: Container<'a, u16, u32>,
{
    let elements: Vec<(u16, u32)> = (0..n as u64)
        .map(|i| (i as u16, rand(0u64, n as u64) as u32))
        .collect();

    for (k, v) in elements.iter() {
        test_push(c, *k, *v, packed);
    }
    test_flush(c, &elements);
    assert_eq!(c.count(), 0);

    for (k, v) in elements.iter().rev() {
        test_push(c, *k, *v, packed);
    }
    test_pop(c, &elements);
    assert_eq!(c.count(), 0);
}

pub fn test_container<'a, C>(mut c: C, is_packed: bool)
where
    C: Container<'a, u16, u32>,
{
    let mut n = 0;
    test_n(&mut c, n, is_packed);
    n = c.capacity() / 2;
    test_n(&mut c, n, is_packed);
    n = c.capacity();
    test_n(&mut c, n, is_packed);
    n = c.capacity() * 2;
    test_n(&mut c, n, is_packed);
}
