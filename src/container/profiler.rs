use crate::container::{Buffered, Container, Get};
use crate::marker::Concurrent;
use crate::utils::clone::CloneCell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// [`Container`](../trait.Container.html) wrapper to collect access,
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
/// use cache::container::{Container, Vector, Profiler};
///
/// // Build a cache:
/// let c = Vector::new(3);
///
/// // Wrap it into a profiler.
/// let mut c = Profiler::new(c);
///
/// // Populate Container
/// c.push("first", 0);
/// c.push("second", 1);
///
/// // look at statistics
/// assert_eq!(c.access(), 2);
/// assert_eq!(c.hit(), 0);
/// assert_eq!(c.miss(), 2);
///
/// // Do some access
/// assert!(c.take(&"third").next().is_none());
/// assert_eq!(c.access(), 3);
/// assert_eq!((&c).hit(), 0);
/// assert_eq!((&c).miss(), 3);
/// assert!((&mut c).take(&"first").next().is_some());
/// assert_eq!((&c).access(), 4);
/// assert_eq!((&c).hit(), 1);
/// assert_eq!((&c).miss(), 3);
///
/// // pretty print statistics.
/// println!("{}", c);
/// ```

struct Stats {
    read: AtomicU64,
    write: AtomicU64,
    read_ms: AtomicU64,
    write_ms: AtomicU64,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            read: AtomicU64::new(0u64),
            write: AtomicU64::new(0u64),
            read_ms: AtomicU64::new(0u64),
            write_ms: AtomicU64::new(0u64),
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
        println!("read read_ms write write_ms")
    }

    /// Print the profiler statistic to file.
    /// If file is empty then header is printed.
    /// Profiler statistic are appended at the end of file.
    pub fn print(&self) {
        println!(
            "{read} {read_ms} {write} {write_ms}",
            read = self.stats.read.load(Ordering::Relaxed),
            read_ms = self.stats.read_ms.load(Ordering::Relaxed),
            write = self.stats.write.load(Ordering::Relaxed),
            write_ms = self.stats.write_ms.load(Ordering::Relaxed),
        )
    }
}

impl<K, V, C: Clone> Clone for Profiler<K, V, C> {
    fn clone(&self) -> Self {
        Profiler {
            cache: self.cache.clone(),
            stats: self.stats.clone(),
            unused_k: PhantomData,
            unused_v: PhantomData,
        }
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

struct ProfilerTakeIter<'a, T> {
    elements: Box<dyn Iterator<Item = T> + 'a>,
    stats: CloneCell<Stats>,
}

impl<'a, T> Iterator for ProfilerTakeIter<'a, T> {
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

//------------------------------------------------------------------------//
// Display Implementation                                                 //
//------------------------------------------------------------------------//

impl<'a, K, V, C> std::fmt::Debug for Profiler<K, V, C>
where
    K: 'a,
    V: 'a,
    C: Container<'a, K, V>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Profiler ({}/{}) reads: {} in {}(ms), writes: {} in {}(ms)",
            self.count(),
            self.capacity(),
            self.read(),
            self.read_ms(),
            self.write(),
            self.write_ms(),
        )
    }
}

impl<'a, K, V, C> std::fmt::Display for Profiler<K, V, C>
where
    K: 'a,
    V: 'a,
    C: Container<'a, K, V>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let read_ms = self.read_ms();
        let read = self.read();
        let write_ms = self.write_ms();
        let write = self.write();

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
        )
    }
}

//------------------------------------------------------------------------//
// Container implementation                                               //
//------------------------------------------------------------------------//

impl<'a, K, V, C> Container<'a, K, V> for Profiler<K, V, C>
where
    K: 'a,
    V: 'a,
    C: Container<'a, K, V>,
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

    fn clear(&mut self) {
        let c = self.cache.count();
        let t0 = Instant::now();
        self.cache.clear();
        let tf = t0.elapsed().as_millis();

        self.stats.write_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.write.fetch_add(c as u64, Ordering::SeqCst);
    }

    fn contains(&self, key: &K) -> bool {
        let t0 = Instant::now();
        let out = self.cache.contains(key);
        let tf = t0.elapsed().as_millis();

        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.read.fetch_add(1 as u64, Ordering::SeqCst);
        out
    }

    /// Counts for one cache access.
    /// If key is found, count a hit else count a miss.
    /// See [`take` function](../trait.Container.html)
    fn take<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = (K, V)> + 'b> {
        Box::new(ProfilerTakeIter {
            elements: self.cache.take(key),
            stats: self.stats.clone(),
        })
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(ProfilerFlushIter {
            elements: self.cache.flush(),
            stats: self.stats.clone(),
        })
    }

    /// Counts for one cache access and one hit.
    /// See [`pop` function](../trait.Container.html)
    fn pop(&mut self) -> Option<(K, V)> {
        let t0 = Instant::now();
        let out = self.cache.pop();
        let tf = t0.elapsed().as_millis();
        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.read.fetch_add(1 as u64, Ordering::SeqCst);
        self.stats.write_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.write.fetch_add(1 as u64, Ordering::SeqCst);
        out
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        let t0 = Instant::now();
        let out = self.cache.push(key, reference);
        let tf = t0.elapsed().as_millis();
        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.read.fetch_add(1 as u64, Ordering::SeqCst);
        self.stats.write_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.write.fetch_add(1 as u64, Ordering::SeqCst);
        out
    }
}

impl<'a, K, V, C> Buffered<'a, K, V> for Profiler<K, V, C>
where
    K: 'a,
    V: 'a,
    C: Container<'a, K, V> + Buffered<'a, K, V>,
{
    fn push_buffer(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let n = elements.len();
        let t0 = Instant::now();
        let out = self.cache.push_buffer(elements);
        let tf = t0.elapsed().as_millis();

        self.stats.read_ms.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats
            .read
            .fetch_add(out.len() as u64, Ordering::SeqCst);
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
    C: Container<'a, K, V> + Concurrent<'a, K, V>,
{
}

impl<'a, K, V, C> Get<'a, K, V> for Profiler<K, V, C>
where
    K: 'a,
    V: 'a,
    C: Container<'a, K, V> + Get<'a, K, V>,
{
    fn get<'b>(
        &'b self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = &'b (K, V)> + 'b> {
        Box::new(ProfilerTakeIter {
            elements: self.cache.get(key),
            stats: self.stats.clone(),
        })
    }

    fn get_mut<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = &'b mut (K, V)> + 'b> {
        Box::new(ProfilerTakeIter {
            elements: self.cache.get_mut(key),
            stats: self.stats.clone(),
        })
    }
}
