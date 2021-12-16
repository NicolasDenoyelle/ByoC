use crate::tests::{TestElement, TestKey, TestValue};
use crate::{BuildingBlock, Concurrent};
use std::{sync::mpsc::channel, thread};

fn test_after_push<C>(
    c: C,
    count: usize,
    keys: Vec<TestKey>,
    popped_keys: Vec<TestKey>,
) where
    C: 'static + BuildingBlock<'static, TestKey, TestValue> + Concurrent,
{
    // Test container count is the incremented count.
    assert!(c.count() == count);
    // Test popped keys plus inserted keys is the number of keys.
    assert!(keys.len() == c.count() + popped_keys.len());

    // Test popped keys and inside keys do not overlap,
    // All keys are distinct. They are either in or out.
    for key in keys {
        if c.contains(&key) {
            assert!(!popped_keys.contains(&key));
        } else {
            assert!(popped_keys.contains(&key));
        }
    }

    // Test container count does not exceed capacity:
    assert!(c.count() <= c.capacity());
}

fn push_concurrent<C>(c: C, num_thread: u8)
where
    C: 'static + BuildingBlock<'static, TestKey, TestValue> + Concurrent,
{
    let capacity = c.capacity();
    let mut set: Vec<TestElement> = (0..capacity * 2)
        .map(|i| (i as TestKey, i as TestValue))
        .collect();
    // The total number of elements to push in the container c.
    let keys: Vec<TestKey> = set.iter().map(|(k, _)| *k).collect();

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
                match container.push(vec![(k, v); 1]).pop() {
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

pub fn test_concurrent<C>(c: C, num_thread: u8)
where
    C: 'static + BuildingBlock<'static, TestKey, TestValue> + Concurrent,
{
    push_concurrent(c, num_thread);
}
