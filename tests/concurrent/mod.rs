pub mod rand;

use cache::{
    container::{Concurrent, Container},
    reference::Default,
};
use std::{cmp::min, sync::mpsc::channel, thread, vec::Vec};

type Reference = Default<u32>;

fn test_after_push<C>(
    mut c: C,
    count: usize,
    keys: Vec<u16>,
    popped_keys: Vec<u16>,
) where
    C: 'static
        + Container<u16, u32, Default<u32>>
        + Concurrent<u16, u32, Default<u32>>,
{
    // Test container count is the incremented count.
    assert!(c.count() == count);
    // Test popped keys plus inserted keys is the number of keys.
    assert!(keys.len() == c.count() + popped_keys.len());

    // Test popped keys and inside keys do not overlap.
    for key in keys {
        match c.get_mut(&key) {
            None => {
                assert!(popped_keys.contains(&key));
            }
            Some(_) => {
                assert!(!popped_keys.contains(&key));
            }
        }
    }

    // Test container count does not exceed capacity:
    assert!(c.count() <= c.capacity());
}

pub fn push_concurrent<C>(c: C, mut set: Vec<(u16, Reference)>, num_thread: u8)
where
    C: 'static
        + Container<u16, u32, Default<u32>>
        + Concurrent<u16, u32, Default<u32>>,
{
    // Not more threads than elements.
    let num_thread = min(num_thread as usize, set.len()) as u8;
    // The total number of elements to push in the container c.
    let keys: Vec<u16> = set.iter().map(|(k, _)| k.clone()).collect();

    // The base set size for each thread.
    let t_size = set.len() / num_thread as usize;
    // Elements popped out.
    let (count, counted) = channel();
    // Elements popped out.
    let (pop, popped) = channel();

    // Parallel push.
    let handles = (0..num_thread).map(|i| {
        let count = count.clone();
        let mut container = c.clone();
        let pop = pop.clone();
        let set = if i == num_thread - 1 {
            set.split_off(0)
        } else {
            set.split_off(set.len() - t_size)
        };
        thread::spawn(move || {
            for (k, v) in set.into_iter() {
                match container.push(k, v) {
                    None => {
                        count.send(1usize).unwrap();
                    }
                    Some((k, _)) => {
                        pop.send(k).unwrap();
                    }
                }
            }
        })
    });

    for h in handles {
        h.join().unwrap()
    }

    test_after_push(
        c,
        counted.try_iter().sum(),
        keys,
        popped.try_iter().collect(),
    );
}

pub fn test_concurrent<C>(c: C, set: Vec<(u16, Reference)>, num_thread: u8)
where
    C: 'static
        + Container<u16, u32, Default<u32>>
        + Concurrent<u16, u32, Default<u32>>,
{
    push_concurrent(c, set, num_thread);
}
