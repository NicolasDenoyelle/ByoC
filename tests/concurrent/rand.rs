use cache::timestamp::{Counter, Timestamp};
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

fn rand(a: u64, b: u64) -> u64 {
    if a >= b {
        panic!("Empty range for random number");
    }
    let n = Counter::new();
    let mut hasher = DefaultHasher::new();
    n.hash(&mut hasher);
    (hasher.finish() % (b - a) + a) as u64
}

pub fn rand_set(n: usize) -> Vec<(u16, u32)> {
    let mut set = BTreeSet::new();
    for _ in 0..n {
        while !set.insert(rand(0, n as u64) as u16) {}
    }

    set.into_iter()
        .map(|k| (k, rand(0, n as u64) as u32))
        .collect()
}

pub fn range_set(n: usize) -> Vec<(u16, u32)> {
    (0..n).map(|i| (i as u16, i as u32)).collect()
}
