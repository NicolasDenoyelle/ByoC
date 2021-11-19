extern crate rand;
use crate::BuildingBlock;
use rand::random;
use std::vec::Vec;

pub fn rand(a: u64, b: u64) -> u64 {
    a + (random::<u64>() % (b - a))
}

fn test_push<'a, C>(c: &mut C, kv: Vec<(u16, u32)>)
where
    C: BuildingBlock<'a, u16, u32>,
{
    let num_before_insertion = c.count();
    let max_capacity = c.capacity();
    let num_insertion = kv.len();
    let extra = c.push(kv.clone());
    let num_extra = extra.len();
    let num_inserted = num_insertion - num_extra;

    // There is less elements not inserted than elements to insert.
    assert!(num_extra <= num_insertion);
    // The count in countainer does not exceed capacity.
    assert!(num_inserted + num_before_insertion <= max_capacity);
    // The count is updated correctly.
    assert_eq!(c.count(), num_inserted + num_before_insertion);

    // Elements inserted can be found in the container.
    for &(k, v) in kv.iter() {
        // If not in extra, must be in container
        if !extra.iter().any(|(_k, _v)| (_k, _v) == (&k, &v)) {
            assert!(c.contains(&k));
        }
    }

    // All elements not inserted are elements from kv
    for e in extra.iter() {
        assert!(kv.iter().any(|_e| e == e));
    }
}

fn test_flush<'a, C>(c: &mut C, pushed: &Vec<(u16, u32)>)
where
    C: BuildingBlock<'a, u16, u32>,
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

fn test_pop<'a, C>(c: &mut C, n: usize, victims: &Vec<(u16, u32)>)
where
    C: BuildingBlock<'a, u16, u32>,
{
    let count = c.count();
    let popped = c.pop(n);

    // Less elements are popped than requested
    assert!(popped.len() <= n);
    // Less elements are popped than present in the container.
    assert!(popped.len() <= count);
    // New count is the difference between old count and popped.
    assert_eq!(c.count(), count - popped.len());
    // All popped elements are expected victims
    if n > 0 {
        for (_, v) in popped {
            assert!(victims.iter().any(|(_, _v)| &v == _v))
        }
    }
}

fn test_n<'a, C>(c: &mut C, n: usize)
where
    C: BuildingBlock<'a, u16, u32>,
{
    let elements: Vec<(u16, u32)> = (0..n as u64)
        .map(|i| (i as u16, rand(0u64, n as u64) as u32))
        .collect();
    test_push(c, Vec::new());

    if elements.len() > 0 {
        test_push(c, vec![elements[0]]);
        test_push(c, vec![elements[0]]);
    }
    test_push(c, elements.clone());
    if elements.len() > 0 {
        test_push(c, vec![elements[0]]);
    }
    test_push(c, elements.clone());

    test_flush(c, &elements);
    assert_eq!(c.count(), 0);

    let out = c.push(elements.clone());
    let mut victims: Vec<(u16, u32)> = elements
        .iter()
        .filter(|e| out.iter().all(|_e| e != &_e))
        .map(|e| e.clone())
        .collect();
    victims.sort_by(|(_, v1), (_, v2)| v1.cmp(v2));

    if victims.len() > 0 {
        let first = vec![victims.pop().unwrap()];
        let count = c.count();
        test_pop(c, 1, &first);
        assert_eq!(c.count(), count - 1);
    }

    test_pop(c, victims.len(), &victims);
    assert_eq!(c.count(), 0);
}

pub fn test_building_block<'a, C>(mut c: C)
where
    C: BuildingBlock<'a, u16, u32>,
{
    let capacity = c.capacity();
    test_n(&mut c, 0);
    test_n(&mut c, capacity / 2);
    test_n(&mut c, capacity);
    test_n(&mut c, capacity * 2);
}
