extern crate rand;
use crate::{BuildingBlock, Get, GetMut};
use rand::random;

pub type TestKey = u16;
pub type TestValue = u32;
pub type TestElement = (TestKey, TestValue);
pub type TestElements = Vec<TestElement>;

pub fn rand(a: u64, b: u64) -> u64 {
    a + (random::<u64>() % (b - a))
}

fn test_push<'a, C>(c: &mut C, kv: TestElements, check_capacity: bool)
where
    C: BuildingBlock<'a, TestKey, TestValue>,
{
    let input = kv.clone();
    let size_before_insertion = c.size();
    let capacity = c.capacity();
    let output = c.push(kv);
    let size_after_insertion = c.size();

    if check_capacity {
        // Size before insertion is less than capacity.
        assert!(size_before_insertion <= capacity);

        // Size after insertion is less than capacity.
        assert!(size_after_insertion <= capacity);

        // There is not more space cleared than needed.
        // assert!(output.len() <= input.len());

        // If nothing needs to be cleared, there is less room in the container
        // after adding elements to it.
        if output.is_empty() && !input.is_empty() {
            assert!(size_after_insertion > size_before_insertion);
        }
    }

    // Elements inserted can be found in the container.
    for &(k, v) in input.iter() {
        // If not in extra, must be in container
        if !output.iter().any(|(_k, _v)| (_k, _v) == (&k, &v)) {
            assert!(c.contains(&k));
        }
    }
}

pub fn insert<'a, C>(
    c: &mut C,
    elements: TestElements,
) -> (TestElements, TestElements)
where
    C: BuildingBlock<'a, TestKey, TestValue>,
{
    let out = c.push(elements.clone());
    let inserted: TestElements = elements
        .iter()
        .filter(|e| out.iter().all(|_e| e != &_e))
        .copied()
        .collect();
    (inserted, out)
}

pub fn test_get<'a, C>(mut c: C)
where
    C: 'a
        + BuildingBlock<'a, TestKey, TestValue>
        + Get<TestKey, TestValue>,
{
    let elements: TestElements =
        (0u16..10u16).map(|i| (i, i as TestValue)).collect();
    let (elements, _) = insert(&mut c, elements);

    for (k, _) in elements.iter() {
        assert!(c.get(k).is_some());
    }
}

pub fn test_get_mut<'a, C>(mut c: C)
where
    C: 'a
        + BuildingBlock<'a, TestKey, TestValue>
        + GetMut<TestKey, TestValue>,
{
    let elements: TestElements =
        (0u16..10u16).map(|i| (i, i as TestValue)).collect();
    let (elements, _) = insert(&mut c, elements);

    for (k, _) in elements.iter() {
        let mut v = c.get_mut(k).unwrap();
        *v += 1;
    }

    for (k, v) in elements.iter() {
        assert_eq!(*c.get_mut(k).unwrap() as TestValue, *v + 1u32);
    }
}

fn test_flush<'a, C>(c: &mut C, elements: TestElements)
where
    C: BuildingBlock<'a, TestKey, TestValue>,
{
    #[allow(unused_must_use)]
    {
        c.flush();
    }
    assert_eq!(c.size(), 0);
    let (inserted, _) = insert(c, elements);

    for (k, v) in c.flush() {
        assert!(inserted.iter().any(|(_k, _v)| { _k == &k && _v == &v }));
    }
    assert_eq!(c.size(), 0);
}

fn test_take<'a, C>(
    c: &mut C,
    elements: TestElements,
    check_capacity: bool,
) where
    C: BuildingBlock<'a, TestKey, TestValue>,
{
    #[allow(unused_must_use)]
    {
        c.flush();
    }
    let (inserted, _) = insert(c, elements);

    let size = c.size();
    for (k, v) in inserted.iter() {
        let out = c.take(k);
        assert!(out.is_some());
        let (_k, _v) = out.unwrap();
        assert_eq!(k, &_k);
        assert_eq!(v, &_v);
        if check_capacity {
            assert!(size > c.size());
        }
    }
}

fn test_pop<'a, C>(c: &mut C, n: usize, check_capacity: bool)
where
    C: BuildingBlock<'a, TestKey, TestValue>,
{
    let old_size = c.size();
    c.pop(n);
    let new_size = c.size();

    if check_capacity {
        // Popping should not increase container size.
        assert!(old_size >= new_size);

        // If we popped less than requested, then the container
        // must be empty.
        let popped_size = old_size - new_size;
        if popped_size < n {
            assert_eq!(new_size, 0);
        }
    }
}

fn test_n<'a, C>(c: &mut C, n: usize, check_capacity: bool)
where
    C: BuildingBlock<'a, TestKey, TestValue>,
{
    let elements: TestElements = (0..n as u64)
        .map(|i| (i as TestKey, rand(0u64, n as u64) as TestValue))
        .collect();

    // Push Test
    test_push(c, Vec::new(), check_capacity);
    if !elements.is_empty() {
        test_push(c, vec![elements[0]], check_capacity);
        test_push(c, vec![elements[0]], check_capacity);
    }
    test_push(c, elements.clone(), check_capacity);
    if !elements.is_empty() {
        test_push(c, vec![elements[0]], check_capacity);
    }
    test_push(c, elements.clone(), check_capacity);

    // Flush Test
    test_flush(c, elements.clone());

    // Take Test
    test_take(c, elements.clone(), check_capacity);

    // Pop Test
    let (inserted, _) = insert(c, elements);
    test_pop(c, 0, check_capacity);
    test_pop(c, 1, check_capacity);
    if !inserted.is_empty() {
        test_pop(c, inserted.len() - 1, check_capacity);
        test_pop(c, 1, check_capacity);
    }

    // Flush Test
    drop(c.flush());
    assert_eq!(c.size(), 0);

    // take_multiple() Test
    let elements: TestElements = (0..n as u64)
        .map(|i| (i as TestKey, rand(0u64, n as u64) as TestValue))
        .collect();
    let (mut inserted, _) = insert(c, elements);

    let mut all_keys: Vec<TestKey> =
        inserted.iter().map(|(k, _)| *k).collect();

    let mut take_all = c.take_multiple(&mut all_keys);

    // Test that input keys contain original input keys minus some removed
    // keys.
    let mut inserted_keys: Vec<TestKey> =
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

pub fn test_building_block<'a, C>(mut c: C, check_capacity: bool)
where
    C: BuildingBlock<'a, TestKey, TestValue>,
{
    let capacity = c.capacity();
    test_n(&mut c, 0, check_capacity);
    test_n(&mut c, capacity / 2, check_capacity);
    test_n(&mut c, capacity, check_capacity);
    test_n(&mut c, capacity * 2, check_capacity);
}
