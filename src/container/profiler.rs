use crate::container::{Container, Get};
use crate::marker::Concurrent;
use crate::utils::{clone::CloneCell, stats::SyncOnlineStats};
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
/// use cache::container::{Container, Get, Vector, Profiler};
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
/// assert!(c.get(&"third").is_none());
/// assert_eq!(c.access(), 3);
/// assert_eq!((&c).hit(), 0);
/// assert_eq!((&c).miss(), 3);
/// assert!((&mut c).get(&"first").is_some());
/// assert_eq!((&c).access(), 4);
/// assert_eq!((&c).hit(), 1);
/// assert_eq!((&c).miss(), 3);
///
/// // pretty print statistics.
/// println!("{}", c);
/// ```

struct Stats {
    access: AtomicU64,
    miss: AtomicU64,
    hit: AtomicU64,
    tot_millis: AtomicU64,
    take_fn: SyncOnlineStats,
    pop_fn: SyncOnlineStats,
    push_fn: SyncOnlineStats,
    flush_fn: SyncOnlineStats,
    get_fn: SyncOnlineStats,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            access: AtomicU64::new(0u64),
            miss: AtomicU64::new(0u64),
            hit: AtomicU64::new(0u64),
            tot_millis: AtomicU64::new(0u64),
            take_fn: SyncOnlineStats::new(),
            pop_fn: SyncOnlineStats::new(),
            push_fn: SyncOnlineStats::new(),
            flush_fn: SyncOnlineStats::new(),
            get_fn: SyncOnlineStats::new(),
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

    /// Amount of cache access
    pub fn access(&self) -> u64 {
        self.stats.access.load(Ordering::Relaxed)
    }
    /// Amount of cache misses
    pub fn miss(&self) -> u64 {
        self.stats.miss.load(Ordering::Relaxed)
    }
    /// Amount of cache hit
    pub fn hit(&self) -> u64 {
        self.stats.hit.load(Ordering::Relaxed)
    }
    /// Amount of cache hit
    pub fn millis(&self) -> u64 {
        self.stats.tot_millis.load(Ordering::Relaxed)
    }

    /// Write profiler header.
    pub fn print_header() {
        println!(
            "{access} {miss} {hit}
{take_fn_mean} {take_fn_var} {take_fn_min} {take_fn_max}
{pop_fn_mean} {pop_fn_var} {pop_fn_min} {pop_fn_max}
{push_fn_mean} {push_fn_var} {push_fn_min} {push_fn_max}
{flush_fn_mean} {flush_fn_var} {flush_fn_min} {flush_fn_max}
{get_fn_mean} {get_fn_var} {get_fn_min} {get_fn_max}",
            access = "access",
            miss = "miss",
            hit = "hit",
            take_fn_mean = "take_fn_mean",
            take_fn_var = "take_fn_var",
            take_fn_min = "take_fn_min",
            take_fn_max = "take_fn_max",
            pop_fn_mean = "pop_fn_mean",
            pop_fn_var = "pop_fn_var",
            pop_fn_min = "pop_fn_min",
            pop_fn_max = "pop_fn_max",
            push_fn_mean = "push_fn_mean",
            push_fn_var = "push_fn_var",
            push_fn_min = "push_fn_min",
            push_fn_max = "push_fn_max",
            flush_fn_mean = "push_fn_mean",
            flush_fn_var = "push_fn_var",
            flush_fn_min = "push_fn_min",
            flush_fn_max = "push_fn_max",
            get_fn_mean = "get_fn_mean",
            get_fn_var = "get_fn_var",
            get_fn_min = "get_fn_min",
            get_fn_max = "get_fn_max"
        )
    }

    /// Print the profiler statistic to file.
    /// If file is empty then header is printed.
    /// Profiler statistic are appended at the end of file.
    pub fn print(&self) {
        println!(
            "{access} {miss} {hit}
{take_fn_mean} {take_fn_var} {take_fn_min} {take_fn_max}
{pop_fn_mean} {pop_fn_var} {pop_fn_min} {pop_fn_max}
{push_fn_mean} {push_fn_var} {push_fn_min} {push_fn_max}
{flush_fn_mean} {flush_fn_var} {flush_fn_min} {flush_fn_max}
{get_fn_mean} {get_fn_var} {get_fn_min} {get_fn_max}",
            access = self.stats.access.load(Ordering::Relaxed),
            miss = self.stats.miss.load(Ordering::Relaxed),
            hit = self.stats.hit.load(Ordering::Relaxed),
            take_fn_mean = self.stats.take_fn.mean(),
            take_fn_var = self.stats.take_fn.var(),
            take_fn_min = self.stats.take_fn.min(),
            take_fn_max = self.stats.take_fn.max(),
            pop_fn_mean = self.stats.pop_fn.mean(),
            pop_fn_var = self.stats.pop_fn.var(),
            pop_fn_min = self.stats.pop_fn.min(),
            pop_fn_max = self.stats.pop_fn.max(),
            push_fn_mean = self.stats.push_fn.mean(),
            push_fn_var = self.stats.push_fn.var(),
            push_fn_min = self.stats.push_fn.min(),
            push_fn_max = self.stats.push_fn.max(),
            flush_fn_mean = self.stats.flush_fn.mean(),
            flush_fn_var = self.stats.flush_fn.var(),
            flush_fn_min = self.stats.flush_fn.min(),
            flush_fn_max = self.stats.flush_fn.max(),
            get_fn_mean = self.stats.get_fn.mean(),
            get_fn_var = self.stats.get_fn.var(),
            get_fn_min = self.stats.get_fn.min(),
            get_fn_max = self.stats.get_fn.max()
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
        let item = self.elements.next();
        match item {
            Some(v) => {
                self.stats.hit.fetch_add(1 as u64, Ordering::SeqCst);
                Some(v)
            }
            None => {
                self.stats.miss.fetch_add(1, Ordering::SeqCst);
                None
            }
        }
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

        let out = match item {
            Some(v) => {
                self.stats.hit.fetch_add(1 as u64, Ordering::SeqCst);
                Some(v)
            }
            None => {
                self.stats.miss.fetch_add(1, Ordering::SeqCst);
                None
            }
        };

        self.stats.tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.take_fn.push(tf as f64);
        out
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
            "Profiler ({}/{}) {} access in {:.2} (s), {}% hits",
            self.count(),
            self.capacity(),
            self.access(),
            self.millis() as f32 * 1e6,
            100f32 * (self.hit() as f32) / (self.access() as f32)
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
        write!(f, "---------------------------------------------------")?;
        write!(f, "Cache profile summary")?;
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
            self.millis() as f32 * 1e6
        )?;
        write!(
            f,
            "Number of access: {} ({}% miss / {}% hits)",
            self.access(),
            100f32 * (self.miss() as f32) / (self.access() as f32),
            100f32 * (self.hit() as f32) / (self.access() as f32)
        )?;
        write!(f, "Methods call timings")?;

        write!(
            f,
            "* fn take (ns):  {:.2} (mean) | {:.2} (var) | {:.2} (min) | {:.2} (max)",
            self.stats.take_fn.mean(),
            self.stats.take_fn.var(),
            self.stats.take_fn.min(),
            self.stats.take_fn.max()
        )?;

        write!(
            f,
            "* fn pop (ns):   {:.2} (mean) | {:.2} (var) | {:.2} (min) | {:.2} (max)",
            self.stats.pop_fn.mean(),
            self.stats.pop_fn.var(),
            self.stats.pop_fn.min(),
            self.stats.pop_fn.max()
        )?;

        write!(
            f,
            "* fn push (ns):  {:.2} (mean) | {:.2} (var) | {:.2} (min) | {:.2} (max),",
            self.stats.push_fn.mean(),
            self.stats.push_fn.var(),
            self.stats.push_fn.min(),
            self.stats.push_fn.max()
        )?;

        write!(
            f,
            "* fn flush (ns):  {:.2} (mean) | {:.2} (var) | {:.2} (min) | {:.2} (max),",
            self.stats.flush_fn.mean(),
            self.stats.flush_fn.var(),
            self.stats.flush_fn.min(),
            self.stats.flush_fn.max()
        )?;

        write!(
            f,
            "* fn get (ns):   {:.2} (mean) | {:.2} (var) | {:.2} (min) | {:.2} (max)",
            self.stats.get_fn.mean(),
            self.stats.get_fn.var(),
            self.stats.get_fn.min(),
            self.stats.get_fn.max()
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
        self.cache.count()
    }
    fn clear(&mut self) {
        self.cache.clear()
    }
    fn contains(&self, key: &K) -> bool {
        self.cache.contains(key)
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
        let t0 = Instant::now();
        let it = Box::new(ProfilerFlushIter {
            elements: self.cache.flush(),
            stats: self.stats.clone(),
        });
        let tf = t0.elapsed().as_millis();

        self.stats.tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.flush_fn.push(tf as f64);
        it
    }

    /// Counts for one cache access and one hit.
    /// See [`pop` function](../trait.Container.html)
    fn pop(&mut self) -> Option<(K, V)> {
        self.stats.access.fetch_add(1, Ordering::SeqCst);
        self.stats.hit.fetch_add(1, Ordering::SeqCst);
        let t0 = Instant::now();
        let out = self.cache.pop();
        let tf = t0.elapsed().as_millis();
        self.stats.tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.pop_fn.push(tf as f64);
        out
    }

    fn push(&mut self, key: K, reference: V) -> Option<(K, V)> {
        self.stats.access.fetch_add(1, Ordering::SeqCst);
        let t0 = Instant::now();
        let out = self.cache.push(key, reference);
        let tf = t0.elapsed().as_millis();
        self.stats.tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.push_fn.push(tf as f64);

        match out {
            None => {
                self.stats.miss.fetch_add(1, Ordering::SeqCst);
                None
            }
            Some(v) => {
                self.stats.hit.fetch_add(1, Ordering::SeqCst);
                Some(v)
            }
        }
    }
}

//------------------------------------------------------------------------//
// Get Trait                                                              //
//------------------------------------------------------------------------//

impl<'a, K, V, C, T> Get<'a, K, V> for Profiler<K, V, C>
where
    K: 'a,
    V: 'a,
    C: Get<'a, K, V, Item = T>,
    T: 'a,
{
    type Item = T;
    fn get(&'a mut self, key: &K) -> Option<T> {
        self.stats.access.fetch_add(1, Ordering::SeqCst);
        let t0 = Instant::now();

        let out = self.cache.get(key);
        let tf = t0.elapsed().as_millis();
        self.stats.tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.get_fn.push(tf as f64);
        match out {
            None => {
                self.stats.miss.fetch_add(1, Ordering::SeqCst);
                None
            }
            Some(v) => {
                self.stats.hit.fetch_add(1, Ordering::SeqCst);
                Some(v)
            }
        }
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
