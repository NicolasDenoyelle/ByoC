use crate::private::clone::CloneCell;
use crate::{BuildingBlock, Concurrent, Get};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// [`BuildingBlock`](../trait.BuildingBlock.html) wrapper to collect access,
/// misses, hits and statistics about methods access time.
///
/// Recording statistics is thread safe.
/// If the wrapped container implements the concurrent trait, then
/// the profiler does to.
///
/// # Generics
///
/// * K: The keys type used for the cache.
/// * V: The type of values stored in cache.
/// * C: The type of the cache to profile.
///
/// # Examples
///
/// ```
/// use cache::{BuildingBlock};
/// use cache::container::Vector;
/// use cache::wrapper::Profiler;
///
/// // Build a cache:
/// let c = Vector::new(3);
///
/// // Wrap it into a profiler.
/// let mut c = Profiler::new(c);
///
/// // Populate BuildingBlock
/// c.push(vec![("first", 0), ("second", 1)]);
///
/// // look at statistics
/// assert_eq!(c.write(), 2);
/// assert_eq!(c.read(), 2);
/// assert_eq!(c.hit(), 0);
/// assert_eq!(c.miss(), 0);
///
/// // Counting elements updates reads.
/// let reads = c.read();
/// let count = c.count() as u64;
/// assert_eq!(c.read(), reads + count);
/// assert_eq!(c.read(), 4);
///
/// // Make a request to a non contained key.
/// // Reads are compulsory, writes hits and misses
/// // depends on whether the key was found.
/// assert!(c.take(&"third").is_none());
/// assert_eq!(c.write(), 2);
/// assert_eq!(c.read(), 5);
/// assert_eq!(c.hit(), 0);
/// assert_eq!(c.miss(), 1);
///
/// // Make a request to a contained key:
/// assert!(c.take(&"second").is_some());
/// assert_eq!(c.write(), 3);
/// assert_eq!(c.read(), 6);
/// assert_eq!(c.hit(), 1);
/// assert_eq!(c.miss(), 1);
///
/// // `Get` methods update the same way as `take()` method:
/// // assert!(c.get(&"first").next().is_some());
/// // assert_eq!(c.write(), 4);
/// // assert_eq!(c.read(), 7);
/// // assert_eq!(c.hit(), 2);
/// // assert_eq!(c.miss(), 1);
///
/// // `flush()` updates reads and writes only if the result is iterated.
/// // Reads and writes are incremented at each iteration.
/// c.flush();
/// assert_eq!(c.write(), 3);
///
/// // `contains()` is consider as one read and will update hits and misses.
/// c.contains(&"first");
/// assert_eq!(c.read(), 7);
/// assert_eq!(c.hit(), 1);
/// assert_eq!(c.miss(), 2);
///
/// // pretty print statistics.
/// println!("{}", c);
/// ```

struct Stats {
    read: AtomicU64,
    write: AtomicU64,
    read_ms: AtomicU64,
    write_ms: AtomicU64,
    hit: AtomicU64,
    miss: AtomicU64,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            read: AtomicU64::new(0u64),
            write: AtomicU64::new(0u64),
            read_ms: AtomicU64::new(0u64),
            write_ms: AtomicU64::new(0u64),
            hit: AtomicU64::new(0u64),
            miss: AtomicU64::new(0u64),
        }
    }
}

pub struct Profiler<K, V, C> {
    cache: CloneCell<C>,
    stats: CloneCell<Stats>,
    unused_k: PhantomData<K>,
    unused_v: PhantomData<V>,
}

impl<K, V, C> Profiler<K, V, C> {
    /// Wrap a `cache` into a "cache profiler" cache.
    pub fn new(cache: C) -> Self {
        Profiler {
            cache: CloneCell::new(cache),
            stats: CloneCell::new(Stats::new()),
            unused_k: PhantomData,
            unused_v: PhantomData,
        }
    }

    /// Amount of requests that found a matching key.
    pub fn hit(&self) -> u64 {
        self.stats.hit.load(Ordering::Relaxed)
    }

    /// Amount of requests that did not found a matching key.
    pub fn miss(&self) -> u64 {
        self.stats.miss.load(Ordering::Relaxed)
    }

    /// Ratio of requests that did not find a matching key.
    pub fn miss_ratio(&self) -> f32 {
        let hit = self.hit();
        let miss = self.miss();
        miss as f32 / (hit + miss) as f32
    }

    /// Amount of read cache access
    pub fn read(&self) -> u64 {
        self.stats.read.load(Ordering::Relaxed)
    }

    /// Time spent in read cache access in milliseconds.
    pub fn read_ms(&self) -> u64 {
        self.stats.read_ms.load(Ordering::Relaxed)
    }

    /// Amount of write cache access
    pub fn write(&self) -> u64 {
        self.stats.write.load(Ordering::Relaxed)
    }

    /// Time spent in write cache access in milliseconds.
    pub fn write_ms(&self) -> u64 {
        self.stats.write_ms.load(Ordering::Relaxed)
    }

    /// Write profiler header.
    pub fn print_header() {
        println!("read read_ms write write_ms hit miss")
    }

    /// Print the profiler statistic to file.
    /// If file is empty then header is printed.
    /// Profiler statistic are appended at the end of file.
    pub fn print(&self) {
        println!(
            "{read} {read_ms} {write} {write_ms} {hit} {miss}",
            read = self.stats.read.load(Ordering::Relaxed),
            read_ms = self.stats.read_ms.load(Ordering::Relaxed),
            write = self.stats.write.load(Ordering::Relaxed),
            write_ms = self.stats.write_ms.load(Ordering::Relaxed),
            hit = self.stats.hit.load(Ordering::Relaxed),
            miss = self.stats.miss.load(Ordering::Relaxed),
        )
    }
}

//------------------------------------------------------------------------//
// Flush iterator
//------------------------------------------------------------------------//

struct ProfilerFlushIter<'a, T> {
    elements: Box<dyn Iterator<Item = T> + 'a>,
    stats: CloneCell<Stats>,
}

impl<'a, T> Iterator for ProfilerFlushIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let t0 = Instant::now();
        let item = self.elements.next();
        let tf = t0.elapsed().as_millis();
        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.write_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.read.fetch_add(1 as u64, Ordering::SeqCst);
        self.stats.write.fetch_add(1 as u64, Ordering::SeqCst);
        item
    }
}

struct ProfilerReqIter<'a, T> {
    elements: Box<dyn Iterator<Item = T> + 'a>,
    stats: CloneCell<Stats>,
}

impl<'a, T> Iterator for ProfilerReqIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let t0 = Instant::now();
        let item = self.elements.next();
        let tf = t0.elapsed().as_millis();

        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.read.fetch_add(1 as u64, Ordering::SeqCst);
        if item.is_some() {
            self.stats.write_ms.fetch_add(tf as u64, Ordering::SeqCst);
            self.stats.write.fetch_add(1 as u64, Ordering::SeqCst);
            self.stats.hit.fetch_add(1 as u64, Ordering::SeqCst);
        } else {
            self.stats.miss.fetch_add(1 as u64, Ordering::SeqCst);
        }
        item
    }
}

//------------------------------------------------------------------------//
// Display Implementation                                                 //
//------------------------------------------------------------------------//

impl<'a, K, V, C> std::fmt::Debug for Profiler<K, V, C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Profiler ({}/{}) reads: {} in {}(ms), writes: {} in {}(ms), miss ratio: {}",
            self.count(),
            self.capacity(),
            self.read(),
            self.read_ms(),
            self.write(),
            self.write_ms(),
						self.miss_ratio(),
        )
    }
}

impl<'a, K, V, C> std::fmt::Display for Profiler<K, V, C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let read_ms = self.read_ms();
        let read = self.read();
        let write_ms = self.write_ms();
        let write = self.write();
        let hit = self.hit();
        let miss = self.miss();
        let miss_ratio = miss as f32 / (hit + miss) as f32;

        write!(f, "---------------------------------------------------")?;
        write!(f, " Cache profile summary")?;
        write!(f, "---------------------------------------------------")?;
        write!(
            f,
            "Cache capacity usage: {}/{}",
            self.count(),
            self.capacity()
        )?;
        write!(
            f,
            "Total profiled time: {:.2} seconds",
            (read_ms + write_ms) as f32 * 1e6
        )?;
        write!(
            f,
            "Reads: {} in {:.2} seconds",
            read,
            read_ms as f32 * 1e6
        )?;
        write!(
            f,
            "Writes: {} in {:.2} seconds",
            write,
            write_ms as f32 * 1e6
        )?;
        write!(
            f,
            "Requests: hits {}, misses {}, ratio: {}%",
            hit,
            miss,
            100f32 * miss_ratio
        )
    }
}

//------------------------------------------------------------------------//
// BuildingBlock implementation                                               //
//------------------------------------------------------------------------//

impl<'a, K, V, C> BuildingBlock<'a, K, V> for Profiler<K, V, C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V>,
{
    fn capacity(&self) -> usize {
        self.cache.capacity()
    }
    fn count(&self) -> usize {
        let t0 = Instant::now();
        let out = self.cache.count();
        let tf = t0.elapsed().as_millis();

        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.read.fetch_add(out as u64, Ordering::SeqCst);
        out
    }

    fn contains(&self, key: &K) -> bool {
        let t0 = Instant::now();
        let out = self.cache.contains(key);
        let tf = t0.elapsed().as_millis();

        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.read.fetch_add(1 as u64, Ordering::SeqCst);
        if out {
            self.stats.hit.fetch_add(1 as u64, Ordering::SeqCst);
        } else {
            self.stats.miss.fetch_add(1 as u64, Ordering::SeqCst);
        }
        out
    }

    /// Counts for one cache access.
    /// If key is found, count a hit else count a miss.
    /// See [`take` function](../trait.BuildingBlock.html#tymethod.take)
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        let t0 = Instant::now();
        let item = self.cache.take(key);
        let tf = t0.elapsed().as_millis();

        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.read.fetch_add(1 as u64, Ordering::SeqCst);
        if item.is_some() {
            self.stats.write_ms.fetch_add(tf as u64, Ordering::SeqCst);
            self.stats.write.fetch_add(1 as u64, Ordering::SeqCst);
            self.stats.hit.fetch_add(1 as u64, Ordering::SeqCst);
        } else {
            self.stats.miss.fetch_add(1 as u64, Ordering::SeqCst);
        }
        item
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(ProfilerFlushIter {
            elements: self.cache.flush(),
            stats: self.stats.clone(),
        })
    }

    /// Counts for one cache access and one hit.
    /// See [`pop` function](../trait.BuildingBlock.html#tymethod.pop)
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let t0 = Instant::now();
        let out = self.cache.pop(n);
        let tf = t0.elapsed().as_millis();
        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.read.fetch_add(n as u64, Ordering::SeqCst);
        self.stats.write_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.write.fetch_add(n as u64, Ordering::SeqCst);
        out
    }

    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let n = elements.len();
        let t0 = Instant::now();
        let out = self.cache.push(elements);
        let tf = t0.elapsed().as_millis();

        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.read.fetch_add(n as u64, Ordering::SeqCst);
        self.stats.write_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.write.fetch_add(n as u64, Ordering::SeqCst);
        out
    }
}

//------------------------------------------------------------------------//
// Concurrent trait                                                       //
//------------------------------------------------------------------------//

unsafe impl<K, V, C: Send> Send for Profiler<K, V, C> {}

unsafe impl<K, V, C: Sync> Sync for Profiler<K, V, C> {}

impl<'a, K, V, C> Concurrent<'a, K, V> for Profiler<K, V, C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V> + Concurrent<'a, K, V>,
{
    fn clone(&self) -> Self {
        Profiler {
            cache: Concurrent::clone(&self.cache),
            stats: self.stats.clone(),
            unused_k: PhantomData,
            unused_v: PhantomData,
        }
    }
}

//------------------------------------------------------------------------//
// Get trait implementation
//------------------------------------------------------------------------//

impl<'a, K, V, U, W, C> Get<'a, K, V, U, W> for Profiler<K, V, C>
where
    U: Deref<Target = V>,
    W: DerefMut<Target = V>,
    C: Get<'a, K, V, U, W>,
{
    fn get(&'a self, key: &K) -> Option<U> {
        let t0 = Instant::now();
        let item = self.cache.get(key);
        let tf = t0.elapsed().as_millis();

        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.read.fetch_add(1 as u64, Ordering::SeqCst);
        if item.is_some() {
            self.stats.hit.fetch_add(1 as u64, Ordering::SeqCst);
        } else {
            self.stats.miss.fetch_add(1 as u64, Ordering::SeqCst);
        }
        item
    }

    fn get_mut(&'a mut self, key: &K) -> Option<W> {
        let t0 = Instant::now();
        let item = self.cache.get_mut(key);
        let tf = t0.elapsed().as_millis();

        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.read.fetch_add(1 as u64, Ordering::SeqCst);
        if item.is_some() {
            self.stats.write_ms.fetch_add(tf as u64, Ordering::SeqCst);
            self.stats.write.fetch_add(1 as u64, Ordering::SeqCst);
            self.stats.hit.fetch_add(1 as u64, Ordering::SeqCst);
        } else {
            self.stats.miss.fetch_add(1 as u64, Ordering::SeqCst);
        }
        item
    }
}
