use crate::timestamp::{Counter, Timestamp};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct Rand {}

impl Rand {
    pub fn rand() -> u64 {
        let n = Counter::new();
        let mut hasher = DefaultHasher::new();
        n.hash(&mut hasher);
        hasher.finish()
    }

    pub fn range(a: u64, b: u64) -> u64 {
        if a >= b {
            panic!("Empty range for random number");
        }
        Rand::rand() % (b - a) + a
    }
}
