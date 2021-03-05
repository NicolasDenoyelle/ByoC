pub mod clone;

use cache::container::{Concurrent, Container};
use cache::lock::RWLockCell;
use cache::reference::Default;
use cache::timestamp::{Counter, Timestamp};
use clone::CloneMut;
use std::cmp::min;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;
use std::vec::Vec;

type Reference = Default<u32>;

fn rand(a: u64, b: u64) -> u64 {
    if a >= b {
        panic!("Empty range for random number");
    }
    let n = Counter::new();
    let mut hasher = DefaultHasher::new();
    n.hash(&mut hasher);
    (hasher.finish() % (b - a) + a) as u64
}

fn rand_set(n: usize) -> Vec<(u16, Reference)> {
    let mut set = BTreeSet::new();
    for _ in 0..n {
        while !set.insert((
            rand(0, n as u64) as u16,
            Default::new(rand(0, n as u64) as u32),
        )) {}
    }
    set.into_iter().collect()
}

fn range_set(n: usize) -> Vec<(u16, Reference)> {
    (0..n).map(|i| (i as u16, Default::new(i as u32))).collect()
}

fn test_push<C>(c: &mut C, set: Vec<(u16, Reference)>, num_thread: usize)
where
    C: Container<u16, u32, Reference> + Concurrent<u16, u32, Reference>,
{
    let num_thread = min(num_thread, set.len());
    let t_size = set.len() / num_thread;
    let count = Arc::new(AtomicU64::new(0));
    let c = CloneMut::new(c);
    let out = CloneMut::new(RWLockCell::new(Vec::<(u16, Reference)>::new()));
    let mut threads: Vec<JoinHandle<_>> = Vec::with_capacity(num_threads);

    for _ in ..num_threads {
        let mut t_count = count.clone();
        let mut t_container = *c.clone();
        let mut t_out = *out.clone();
        let mut t_set = if set.len() % t_size == 0 {
            set.split_off(set.len() - t_size);
        } else {
            set.split_off(set.len() - (set.len() % t_size));
        };
        threads.push(thread::spawn(move || {}));
    }

    assert!(container.count() == count.load(Ordering::SeqCst) as usize);
}

// pub fn test_concurrent<C>(c: C)
// where
//     C: 'static
//         + Container<u16, u32, Default<u32>>
//         + Concurrent<u16, u32, Default<u32>>,
// {
//     let index_max = c.capacity();
//     let num_threads = 64;
//     let mut threads: Vec<JoinHandle<_>> = Vec::with_capacity(num_threads);
//     let count = Arc::new(AtomicU64::new(0));
//     let container = CloneMut::new(c);

//     for i in 0..num_threads {
//         let begin = i * index_max / num_threads;
//         let mut end = index_max / num_threads;
//         end += if i + 1 == num_threads {
//             index_max % num_threads
//         } else {
//             0
//         };
//         let local_index: Vec<usize> = (begin..end).collect();
//         let local_count = count.clone();
//         let mut local_container = container.clone();
//         threads.push(thread::spawn(move || {
//             for j in local_index.iter().map(|i| i % index_max) {
//                 let r = Default::new(j as u32);
//                 match local_container.push(j as u16, r) {
//                     None => {
//                         local_count.fetch_add(1, Ordering::SeqCst);
//                     }
//                     Some(_) => {}
//                 }
//             }
//         }));
//     }

//     for _ in 0..num_threads {
//         threads.pop().unwrap().join().unwrap();
//     }

//     assert!(container.count() == count.load(Ordering::SeqCst) as usize);
// }
