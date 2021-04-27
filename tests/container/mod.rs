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

fn test_push<'a, C>(c: &'a mut C, key: u16, value: u32)
where
    C: Container<'a, u16, u32>,
{
    let count = c.count();

    // Test insertion
    match c.push(key, value) {
        Some((k, v)) => {
            // Insertion popped then count is not updated.
            assert_eq!(c.count(), count);
            // If no capacity container, inserted key and value go out.
            if c.capacity() == 0 {
                assert_eq!(k, key);
                assert_eq!(v, value);
            }
            // When popping different key, inserted element must be present.
            else if k != key {
                assert!(c.contains(&key));
                // Take and reinsert of just inserted key must work.
                let (k, v) = c.take(key).next().unwrap();
                assert_eq!(key, k);
                assert!(c.push(k, v).is_none());
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

// fn test_flush<'a, C>(c: &'a mut C, pushed: &Vec<(u16, u32)>)
// where
//     C: 'a + Container<u16, u32>,
// {
//     let mut i = 0;
//     let count = c.count();
//     for (k, v) in c.flush() {
//         assert!(pushed
//             .iter()
//             .find(|(_k, _v)| { _k == &k && _v == &v })
//             .is_some());
//         i += 1;
//     }
//     assert_eq!(i, count);
//     assert_eq!(c.count(), 0);
// }

fn test_pop<'a, C>(c: &mut C, pushed: &Vec<(u16, u32)>)
where
    C: Container<'a, u16, u32>,
{
    let mut i = 0;
    let count = c.count();
    while let Some((k, v)) = c.pop() {
        assert!(pushed
            .iter()
            .find(|(_k, _v)| { _k == &k && _v == &v })
            .is_some());
        i += 1;
    }
    assert_eq!(i, count);
    assert_eq!(c.count(), 0);
}

fn test_n_container<'a, C>(c: &'a mut C, n: usize)
where
    C: Container<'a, u16, u32>,
{
    let elements: Vec<(u16, u32)> = (0..n as u64)
        .map(|i| (i as u16, rand(0, n as u64) as u32))
        .collect();

    // for (k, v) in elements.iter() {
    //     test_push(c, *k, *v);
    // }
    // test_flush(c, &elements);
    // assert_eq!(c.count(), 0);

    for (k, v) in elements.iter().rev() {
        test_push(c, *k, *v);
    }
    test_pop(c, &elements);
    assert_eq!(c.count(), 0);
}

pub fn test_container<'a, 'b: 'a, C>(mut c: C)
where
    C: 'b + Container<'a, u16, u32>,
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
