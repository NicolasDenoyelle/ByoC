use crate::timestamp::{Counter, Timestamp};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Function to get a random number.
pub fn rand() -> u64 {
    let n = Counter::new();
    let mut hasher = DefaultHasher::new();
    n.hash(&mut hasher);
    hasher.finish()
}

/// Generate a random number in interval
pub fn rand_ab(a: u64, b: u64) -> u64 {
    if a >= b {
        panic!("Empty range for random number");
    }
    rand() % (b - a) + a
}

// /// Generate a range of numbers between a and b and shuffle them.
// pub fn shuffled_range(a: usize, b:usize) -> Vec<usize> {
//     if b <= a {
//         panic!("Cannot create a negative range.");
//     }

//     let n: usize = b-a;
//     let mut index: Vec<usize> = (0..n).collect();
//     let mut sindex: Vec<usize> = (0..n).collect();

//     for i in n..0 {
//         let val = rand_ab(0,i as u64) as usize;
//         sindex[i-1] = index[val] + a;
//         index[val] = index[i-1];
//     }
//     sindex
// }

// /// Shuffle the content of a vector.
// pub fn shuffle<V>(values: &mut Vec<V>){
//     let n: usize = values.len();
//     let index = shuffled_range(0, n);
//     let mut current = 0;
//     let mut next = index[0];

//     for _ in 0..n {
//         values.swap(current, next);
//         current = next;
//         next = index[current];
//     }
// }
