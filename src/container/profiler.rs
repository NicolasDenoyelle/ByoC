use crate::container::{Container, Get};
use crate::marker::Concurrent;
use crate::utils::{clone::CloneCell, stats::SyncOnlineStats};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// [`Container`](../trait.Container.html) wrapper to collect access, misses, hits and
/// statistics about methods access time.
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
/// use cache::container::{Container, Get, Map, Profiler};
/// use cache::reference::Default;
///
/// // Build a cache:
/// let map = Map::<_,Default<_>>::new(2);
///
/// // Wrap it into a profiler.
/// let mut p = &mut Profiler::new(map);
///
/// // Populate Container
/// p.push("first", 0);
/// p.push("second", 1);
///
/// // look at statistics
/// assert_eq!(p.access(), 2);
/// assert_eq!(p.hit(), 0);
/// assert_eq!(p.miss(), 2);
///
/// // Do some access
/// assert!(p.get(&"third").is_none());
/// assert_eq!(p.access(), 3);
/// assert_eq!((&p).hit(), 0);
/// assert_eq!((&p).miss(), 3);
/// assert!((&mut p).get(&"first").is_some());
/// assert_eq!((&p).access(), 4);
/// assert_eq!((&p).hit(), 1);
/// assert_eq!((&p).miss(), 3);
///
/// // pretty print statistics.
/// println!("{}", p);
/// ```

struct Stats {
    access: AtomicU64,
    miss: AtomicU64,
    hit: AtomicU64,
    tot_millis: AtomicU64,
    take_fn: SyncOnlineStats,
    pop_fn: SyncOnlineStats,
    push_fn: SyncOnlineStats,
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
            get_fn: SyncOnlineStats::new(),
        }
    }
}

pub struct Profiler<C> {
    cache: CloneCell<C>,
    stats: CloneCell<Stats>,
}

impl<C> Profiler<C> {
    /// Wrap a `cache` into a "cache profiler" cache.
    pub fn new(cache: C) -> Self {
        Profiler {
            cache: CloneCell::new(cache),
            stats: CloneCell::new(Stats::new()),
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
            get_fn_mean = self.stats.get_fn.mean(),
            get_fn_var = self.stats.get_fn.var(),
            get_fn_min = self.stats.get_fn.min(),
            get_fn_max = self.stats.get_fn.max()
        )
    }
}

impl<C: Clone> Clone for Profiler<C> {
    fn clone(&self) -> Self {
        Profiler {
            cache: self.cache.clone(),
            stats: self.stats.clone(),
        }
    }
}

//------------------------------------------------------------------------//
// Display Implementation                                                 //
//------------------------------------------------------------------------//

impl<C: Container<u32, u32>> std::fmt::Debug for Profiler<C> {
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

impl<C: Container<u32, u32>> std::fmt::Display for Profiler<C> {
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

impl<K, V, C> Container<K, V> for Profiler<C>
where
    C: Container<K, V>,
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
    fn take(&mut self, key: &K) -> Option<V> {
        self.stats.access.fetch_add(1, Ordering::SeqCst);
        let t0 = Instant::now();
        let out = self.cache.take(key);
        let tf = t0.elapsed().as_millis();
        self.stats.tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.take_fn.push(tf as f64);

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

    fn flush(&mut self) -> Vec<(K, V)> {
        let t0 = Instant::now();
        let v = self.cache.flush();
        let tf = t0.elapsed().as_millis();
        self.stats.hit.fetch_add(v.len() as u64, Ordering::SeqCst);
        self.stats.tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.stats.pop_fn.push(tf as f64);
        v
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

impl<'a, K, V, C, T> Get<'a, K, V> for Profiler<C>
where
    C: Get<'a, K, V, Item = T>,
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

unsafe impl<C: Send> Send for Profiler<C> {}

unsafe impl<C: Sync> Sync for Profiler<C> {}

impl<K, V, C: Concurrent<K, V>> Concurrent<K, V> for Profiler<C> {}
