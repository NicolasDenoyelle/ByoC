extern crate rand;
use crate::{BuildingBlock, Get, GetMut, Prefetch};
use rand::random;
use std::ops::{Deref, DerefMut};

pub type TestElements = Vec<(u16, u32)>;

pub fn rand(a: u64, b: u64) -> u64 {
    a + (random::<u64>() % (b - a))
}

fn test_push<'a, C>(c: &mut C, kv: TestElements)
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
}

pub fn insert<'a, C>(
    c: &mut C,
    elements: TestElements,
) -> (TestElements, TestElements)
where
    C: BuildingBlock<'a, u16, u32>,
{
    let out = c.push(elements.clone());
    let inserted: TestElements = elements
        .iter()
        .filter(|e| out.iter().all(|_e| e != &_e))
        .copied()
        .collect();
    (inserted, out)
}

pub fn test_get<'a, C, U>(mut c: C)
where
    U: Deref<Target = u32>,
    C: 'a + BuildingBlock<'a, u16, u32> + Get<u16, u32, U>,
{
    let elements: TestElements =
        (0u16..10u16).map(|i| (i, i as u32)).collect();
    let (elements, _) = insert(&mut c, elements);

    for (k, _) in elements.iter() {
        assert!(unsafe { c.get(k) }.is_some());
    }
}

pub fn test_get_mut<'a, C, W>(mut c: C)
where
    W: Deref<Target = u32> + DerefMut,
    C: 'a + BuildingBlock<'a, u16, u32> + GetMut<u16, u32, W>,
{
    let elements: TestElements =
        (0u16..10u16).map(|i| (i, i as u32)).collect();
    let (elements, _) = insert(&mut c, elements);

    for (k, _) in elements.iter() {
        let mut v = unsafe { c.get_mut(k).unwrap() };
        *v += 1;
    }

    for (k, v) in elements.iter() {
        assert_eq!(*unsafe { c.get_mut(k).unwrap() } as u32, *v + 1u32);
    }
}

fn test_flush<'a, C>(c: &mut C, elements: TestElements)
where
    C: BuildingBlock<'a, u16, u32>,
{
    #[allow(unused_must_use)]
    {
        c.flush();
    }
    assert_eq!(c.count(), 0);
    let (inserted, _) = insert(c, elements);

    for (k, v) in c.flush() {
        assert!(inserted.iter().any(|(_k, _v)| { _k == &k && _v == &v }));
    }
    assert_eq!(c.count(), 0);
}

fn test_take<'a, C>(c: &mut C, elements: TestElements)
where
    C: BuildingBlock<'a, u16, u32>,
{
    #[allow(unused_must_use)]
    {
        c.flush();
    }
    let (inserted, _) = insert(c, elements);

    let count = c.count();
    for (i, (k, v)) in inserted.iter().enumerate() {
        let out = c.take(k);
        assert!(out.is_some());
        let (_k, _v) = out.unwrap();
        assert_eq!(k, &_k);
        assert_eq!(v, &_v);
        assert_eq!(count - i - 1, c.count());
    }
}

fn test_pop<'a, C>(c: &mut C, n: usize)
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
}

fn test_n<'a, C>(c: &mut C, n: usize)
where
    C: BuildingBlock<'a, u16, u32>,
{
    let elements: TestElements = (0..n as u64)
        .map(|i| (i as u16, rand(0u64, n as u64) as u32))
        .collect();

    // Push Test
    test_push(c, Vec::new());
    if !elements.is_empty() {
        test_push(c, vec![elements[0]]);
        test_push(c, vec![elements[0]]);
    }
    test_push(c, elements.clone());
    if !elements.is_empty() {
        test_push(c, vec![elements[0]]);
    }
    test_push(c, elements.clone());

    // Flush Test
    test_flush(c, elements.clone());

    // Take Test
    test_take(c, elements.clone());

    // Pop Test
    let (inserted, _) = insert(c, elements);
    test_pop(c, 0);
    test_pop(c, 1);
    if !inserted.is_empty() {
        test_pop(c, inserted.len() - 1);
        test_pop(c, 1);
    }
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

pub fn test_prefetch<'a, C>(mut c: C)
where
    C: BuildingBlock<'a, u16, u32> + Prefetch<'a, u16, u32>,
{
    let n = c.capacity();
    let elements: TestElements = (0..n as u64)
        .map(|i| (i as u16, rand(0u64, n as u64) as u32))
        .collect();
    let (mut inserted, _) = insert(&mut c, elements);

    let mut all_keys: Vec<u16> =
        inserted.iter().map(|(k, _)| *k).collect();

    // Make sure prefetch method keeps everything inside the container.
    c.prefetch(all_keys.clone());
    for (k, _) in inserted.iter() {
        assert!(c.contains(k));
    }

    let mut take_all = c.take_multiple(&mut all_keys);

    // Test that input keys contain original input keys minus some removed
    // keys.
    let mut inserted_keys: Vec<u16> =
        inserted.iter().map(|(k, _)| *k).collect();
    inserted_keys.sort_unstable();
    for k in all_keys {
        assert!(inserted_keys.binary_search(&k).is_ok());
    }

    // Make sure that we took all the keys that we inserted.
    inserted.sort_unstable();
    take_all.sort_unstable();
    for (a, b) in inserted.iter().zip(take_all.iter()) {
        assert_eq!(a, b);
    }
}
