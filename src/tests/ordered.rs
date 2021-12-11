use crate::tests::{insert, rand, TestElements};
use crate::BuildingBlock;

fn test_pop<'a, C>(c: &mut C, n: usize, victims: TestElements)
where
    C: BuildingBlock<'a, u16, u32>,
{
    let popped = c.pop(n);

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
    let elements: TestElements = (0..n as u64)
        .map(|i| (i as u16, rand(0u64, n as u64) as u32))
        .collect();

    let (mut victims, _) = insert(c, elements);
    victims.sort_by(|(_, v1), (_, v2)| v1.cmp(v2));

    if !victims.is_empty() {
        let first = vec![victims.pop().unwrap()];
        test_pop(c, 1, first);
    }
    test_pop(c, victims.len(), victims);
}

pub fn test_ordered<'a, C>(mut c: C)
where
    C: BuildingBlock<'a, u16, u32>,
{
    let capacity = c.capacity();
    test_n(&mut c, 0);
    test_n(&mut c, capacity / 2);
    test_n(&mut c, capacity);
    test_n(&mut c, capacity * 2);
}
