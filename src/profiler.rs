use crate::private::clone::CloneCell;
use crate::{BuildingBlock, Concurrent, Get, GetMut, Ordered, Prefetch};
use std::fs::File;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Accumulator of call count and elapsed time.
struct MethodStats {
    count: AtomicU64,
    elapsed: AtomicU64,
}

impl MethodStats {
    /// Create zeroed MethodStats.
    pub fn new() -> Self {
        MethodStats {
            count: AtomicU64::new(0),
            elapsed: AtomicU64::new(0),
        }
    }

    /// Accumulate count and elapsed time.
    pub fn add(&mut self, count: usize, elapsed: u128) {
        self.count.fetch_add(count as u64, Ordering::SeqCst);
        self.elapsed.fetch_add(elapsed as u64, Ordering::SeqCst);
    }

    /// Get (count, time) stats.
    pub fn read(&self) -> (u64, u64) {
        let count = self.count.load(Ordering::Relaxed);
        let elapsed = self.elapsed.load(Ordering::Relaxed);
        (count, elapsed)
    }
}

/// Accumulator of building block method stats.
struct Stats {
    pub count: MethodStats,
    pub contains: MethodStats,
    pub take: MethodStats,
    pub pop: MethodStats,
    pub push: MethodStats,
    pub flush: MethodStats,
    pub flush_iter: MethodStats,
    pub get: MethodStats,
    pub get_mut: MethodStats,
    pub hit: MethodStats,
    pub miss: MethodStats,
}

macro_rules! write_it {
    ($struct:expr, $field:ident, $prefix:ident, $file:ident) => {
        let (n, time) = $struct.$field.read();
        writeln!(
            $file,
            "{}{} {} {}",
            $prefix,
            stringify!($field),
            n,
            time,
        )
        .unwrap();
    };
}

macro_rules! print_it {
    ($struct:expr, $field:ident, $prefix:ident) => {
        let (n, time) = $struct.$field.read();
        println!("{}{} {} {}", $prefix, stringify!($field), n, time);
    };
}

impl Stats {
    /// Initialize zeroed method stats.
    pub fn new() -> Self {
        Stats {
            count: MethodStats::new(),
            contains: MethodStats::new(),
            take: MethodStats::new(),
            pop: MethodStats::new(),
            push: MethodStats::new(),
            flush: MethodStats::new(),
            flush_iter: MethodStats::new(),
            get: MethodStats::new(),
            get_mut: MethodStats::new(),
            hit: MethodStats::new(),
            miss: MethodStats::new(),
        }
    }
}

/// Time a function call and return `(time, output)`
/// where `time` is elapsed time in nanoseconds and
/// `output` is the function call output.
macro_rules! time_it {
    ($call:expr) => {{
        let t0 = Instant::now();
        let out = $call;
        (t0.elapsed().as_nanos(), out)
    }};
}

/// Possible ways of printing output stats when a `Profiler` container
/// is dropped.
#[derive(Clone)]
pub enum ProfilerOutputKind {
    /// No output is printed.
    None,
    /// Output is printed to stdout.
    Stdout,
    /// Output is printed to a file of the given name.
    File(String),
}

/// Building block wrapper to collect
/// access, misses, hits and statistics about methods access time.
///
/// See Profiler `_stats()` methods to learn about what is counted/measured.
///
/// Recording statistics is thread safe.
/// If the wrapped container implements the concurrent trait, then
/// so does the profiler.
///
/// # Examples
///
/// ```
/// use byoc::{BuildingBlock, Get};
/// use byoc::{Array, Profiler, ProfilerOutputKind};
///
/// // Build a cache:
/// let c = Array::new(3);
///
/// // Wrap it into a profiler.
/// let mut c = Profiler::new("example", ProfilerOutputKind::None, c);
///
/// // Populate BuildingBlock
/// c.push(vec![("first", 0), ("second", 1)]);
/// assert_eq!(c.push_stats().0, (2));
///
/// // Check if a key is in the container.
/// c.contains(&"first");
/// assert_eq!(c.hit_stats().0, 1);
/// assert_eq!(c.miss_stats().0, 0);
///
/// // Try to take a non existing key out of the container.
/// c.take(&"third");
/// assert_eq!(c.hit_stats().0, 1);
/// assert_eq!(c.miss_stats().0, 1);
///
/// // Access a value in the container.
/// unsafe { c.get(&"second"); }
/// assert_eq!(c.hit_stats().0, 2);
/// assert_eq!(c.miss_stats().0, 1);
/// ```
pub struct Profiler<C> {
    cache: C,
    name: String,
    output: ProfilerOutputKind,
    stats: CloneCell<Stats>,
}

impl<C> Drop for Profiler<C> {
    fn drop(&mut self) {
        let mut prefix = self.name.clone();
        prefix.push(' ');

        match &self.output {
            ProfilerOutputKind::None => {}
            ProfilerOutputKind::Stdout => {
                print_it!(self.stats, count, prefix);
                print_it!(self.stats, contains, prefix);
                print_it!(self.stats, take, prefix);
                print_it!(self.stats, pop, prefix);
                print_it!(self.stats, push, prefix);
                print_it!(self.stats, flush, prefix);
                print_it!(self.stats, flush_iter, prefix);
                print_it!(self.stats, get, prefix);
                print_it!(self.stats, get_mut, prefix);
                print_it!(self.stats, hit, prefix);
                print_it!(self.stats, miss, prefix);
            }
            ProfilerOutputKind::File(s) => match File::create(s) {
                Ok(mut f) => {
                    write_it!(self.stats, count, prefix, f);
                    write_it!(self.stats, contains, prefix, f);
                    write_it!(self.stats, take, prefix, f);
                    write_it!(self.stats, pop, prefix, f);
                    write_it!(self.stats, push, prefix, f);
                    write_it!(self.stats, flush, prefix, f);
                    write_it!(self.stats, flush_iter, prefix, f);
                    write_it!(self.stats, get, prefix, f);
                    write_it!(self.stats, get_mut, prefix, f);
                    write_it!(self.stats, hit, prefix, f);
                    write_it!(self.stats, miss, prefix, f);
                }
                Err(e) => {
                    println!(
                        "Failed to open file for writing: {}.\n{:?}",
                        s, e
                    )
                }
            },
        }
    }
}

impl<C> Profiler<C> {
    /// Wrap a building block into a `Profiler`.
    pub fn new(name: &str, output: ProfilerOutputKind, cache: C) -> Self {
        Profiler {
            cache,
            name: String::from(name),
            output: output,
            stats: CloneCell::new(Stats::new()),
        }
    }

    /// Get a summary of (0) the number of
    /// [`count()`](trait.BuildingBlock.html#tymethod.count) method call
    /// and (1) the total time spent in nanoseconds in these calls.
    pub fn count_stats(&self) -> (u64, u64) {
        self.stats.count.read()
    }
    /// Get a summary of (0) the number of
    /// [`contain()`](trait.BuildingBlock.html#tymethod.contain) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn contains_stats(&self) -> (u64, u64) {
        self.stats.contains.read()
    }
    /// Get a summary of (0) the number of
    /// [`take()`](trait.BuildingBlock.html#tymethod.take) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn take_stats(&self) -> (u64, u64) {
        self.stats.take.read()
    }
    /// Get a summary of (0) the number of
    /// [`pop()`](trait.BuildingBlock.html#tymethod.pop) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn pop_stats(&self) -> (u64, u64) {
        self.stats.pop.read()
    }
    /// Get a summary of (0) the number of
    /// [`push()`](trait.BuildingBlock.html#tymethod.push) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn push_stats(&self) -> (u64, u64) {
        self.stats.push.read()
    }
    /// Get a summary of (0) the number of
    /// [`flush()`](trait.BuildingBlock.html#tymethod.flush) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn flush_stats(&self) -> (u64, u64) {
        self.stats.flush.read()
    }
    /// Get a summary of (0) the number of iterations performed on an
    /// iterator obtained through
    /// [`flush()`](trait.BuildingBlock.html#tymethod.flush) method
    /// and (1) the total time spent in nanoseconds on iterations.
    pub fn flush_iter_stats(&self) -> (u64, u64) {
        self.stats.flush_iter.read()
    }
    /// Get a summary of (0) the number of
    /// [`get()`](trait.Get.html#tymethod.get) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn get_stats(&self) -> (u64, u64) {
        self.stats.get.read()
    }
    /// Get a summary of (0) the number of
    /// [`get_mut()`](trait.Get.html#tymethod.get_mut) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn get_mut_stats(&self) -> (u64, u64) {
        self.stats.get_mut.read()
    }
    /// Get the total amount of time user key query was matched with a key
    /// in the container when calling
    /// [`contains()`](trait.BuildingBlock.html#tymethod.contains),
    /// [`take()`](trait.BuildingBlock.html#tymethod.take),
    /// [`get()`](trait.Get.html#tymethod.get) or
    /// [`get_mut()`](trait.Get.html#tymethod.get_mut) methods.
    pub fn hit_stats(&self) -> (u64, u64) {
        self.stats.hit.read()
    }
    /// Get the total amount of time user key query was not matched with a
    /// key in the container when calling
    /// [`contains()`](trait.BuildingBlock.html#tymethod.contains),
    /// [`take()`](trait.BuildingBlock.html#tymethod.take),
    /// [`get()`](trait.Get.html#tymethod.get) or
    /// [`get_mut()`](trait.Get.html#tymethod.get_mut) methods.
    pub fn miss_stats(&self) -> (u64, u64) {
        self.stats.miss.read()
    }

    /// Get the total time spent in methods call so far.
    pub fn time_stats(&self) -> u64 {
        let count_time = self.stats.count.read().1;
        let contains_time = self.stats.contains.read().1;
        let take_time = self.stats.take.read().1;
        let pop_time = self.stats.pop.read().1;
        let push_time = self.stats.push.read().1;
        let flush_time = self.stats.flush.read().1;
        let flush_iter_time = self.stats.flush_iter.read().1;
        let get_time = self.stats.get.read().1;
        let get_mut_time = self.stats.get_mut.read().1;
        count_time
            + contains_time
            + take_time
            + pop_time
            + push_time
            + flush_time
            + flush_iter_time
            + get_time
            + get_mut_time
    }
}

//------------------------------------------------------------------------//
// Flush iterator
//------------------------------------------------------------------------//

/// Iterator of flushed elements counting time iterating and number
/// of iterations.
struct ProfilerFlushIter<'a, T> {
    elements: Box<dyn Iterator<Item = T> + 'a>,
    stats: CloneCell<Stats>,
}

impl<'a, T> Iterator for ProfilerFlushIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let (time, item) = time_it!(self.elements.next());
        self.stats.clone().flush_iter.add(1, time);
        item
    }
}

//------------------------------------------------------------------------//
// BuildingBlock implementation
//------------------------------------------------------------------------//

impl<'a, K, V, C> BuildingBlock<'a, K, V> for Profiler<C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V>,
{
    fn capacity(&self) -> usize {
        self.cache.capacity()
    }

    fn count(&self) -> usize {
        let (time, count) = time_it!(self.count());
        self.stats.clone().count.add(1, time);
        count
    }

    fn contains(&self, key: &K) -> bool {
        let (time, out) = time_it!(self.cache.contains(key));
        self.stats.clone().contains.add(1, time);
        match out {
            true => self.stats.clone().hit.add(1, time),
            false => self.stats.clone().miss.add(1, time),
        };
        out
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        let (time, out) = time_it!(self.cache.take(key));
        self.stats.take.add(1, time);
        match out {
            Some(_) => self.stats.clone().hit.add(1, time),
            None => self.stats.clone().miss.add(1, time),
        };
        out
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        let (time, out) = time_it!(self.cache.flush());
        self.stats.flush.add(1, time);
        Box::new(ProfilerFlushIter {
            elements: out,
            stats: self.stats.clone(),
        })
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let (time, out) = time_it!(self.cache.pop(n));
        self.stats.pop.add(n, time);
        out
    }

    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let n = elements.len();
        let (time, out) = time_it!(self.cache.push(elements));
        self.stats.push.add(n, time);
        out
    }
}

impl<V, C> Ordered<V> for Profiler<C>
where
    V: Ord,
    C: Ordered<V>,
{
}

//------------------------------------------------------------------------//
// Concurrent trait                                                       //
//------------------------------------------------------------------------//

unsafe impl<C: Send> Send for Profiler<C> {}

unsafe impl<C: Sync> Sync for Profiler<C> {}

impl<C> Concurrent for Profiler<C>
where
    C: Concurrent,
{
    fn clone(&self) -> Self {
        Profiler {
            cache: Concurrent::clone(&self.cache),
            name: self.name.clone(),
            output: self.output.clone(),
            stats: self.stats.clone(),
        }
    }
}

//------------------------------------------------------------------------//
// Get trait implementation
//------------------------------------------------------------------------//

impl<K, V, U, C> Get<K, V, U> for Profiler<C>
where
    U: Deref<Target = V>,
    C: Get<K, V, U>,
{
    unsafe fn get(&self, key: &K) -> Option<U> {
        let (time, out) = time_it!(self.cache.get(key));
        self.stats.clone().get.add(1, time);
        match out {
            Some(_) => self.stats.clone().hit.add(1, time),
            None => self.stats.clone().miss.add(1, time),
        };
        out
    }
}

impl<K, V, W, C> GetMut<K, V, W> for Profiler<C>
where
    W: DerefMut<Target = V>,
    C: GetMut<K, V, W>,
{
    unsafe fn get_mut(&mut self, key: &K) -> Option<W> {
        let (time, out) = time_it!(self.cache.get_mut(key));
        self.stats.get_mut.add(1, time);
        match out {
            Some(_) => self.stats.clone().hit.add(1, time),
            None => self.stats.clone().miss.add(1, time),
        };
        out
    }
}

//------------------------------------------------------------------------//
// Prefetch Trait Implementation
//------------------------------------------------------------------------//

impl<'a, K, V, C> Prefetch<'a, K, V> for Profiler<C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V> + Prefetch<'a, K, V>,
{
    fn prefetch(&mut self, keys: Vec<K>) {
        self.cache.prefetch(keys)
    }

    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let n = keys.len();
        let (time, out) = time_it!(self.cache.take_multiple(keys));
        self.stats.take.add(n, time);
        let hits = out.len();
        let misses = n - hits;
        self.stats.hit.add(hits, time);
        self.stats.miss.add(misses, time);
        out
    }
}
