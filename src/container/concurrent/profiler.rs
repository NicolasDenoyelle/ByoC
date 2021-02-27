use crate::container::{
    Concurrent, Container, Insert, Iter, IterMut, Sequential,
};
use crate::reference::{FromValue, Reference};
use crate::utils::lock::RWLockGuard;
use crate::utils::stats::SyncOnlineStats;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
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
/// // Let's bring container methods into scope. We also bring
/// // Insert methods for container to make the code easier to read.
/// // We bring sequential get methods into scope.
/// use cache::container::{Container, Insert, Sequential};
/// // The container will be a Map.
/// use cache::container::sequential::Map;
/// // We wrap into a Profiler. Though it can support concurrent acces,
/// // We only use sequential access in this example.
/// use cache::container::concurrent::Profiler;
/// // We will insert default references in the container.
/// use cache::reference::Default;
///
/// // Build a cache:
/// let map = Map::<_,_,Default<_>>::new(2);
///
/// // Wrap it into a profiler.
/// let mut p = &mut Profiler::new(map);
///
/// // Populate Container
/// p.insert("first", 0);
/// p.insert("second", 1);
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
pub struct Profiler<K, V, R, C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R>,
{
    cache: C,
    _access: AtomicU64,
    _miss: AtomicU64,
    _hit: AtomicU64,
    _tot_millis: AtomicU64,
    take_fn: SyncOnlineStats,
    pop_fn: SyncOnlineStats,
    push_fn: SyncOnlineStats,
    get_fn: SyncOnlineStats,
    path: Option<PathBuf>,
    unused_k: PhantomData<K>,
    unused_r: PhantomData<R>,
    unused_v: PhantomData<V>,
}

impl<K, V, R, C> Profiler<K, V, R, C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R>,
{
    /// Wrap a `cache` into a "cache profiler" cache.
    pub fn new(cache: C) -> Self {
        Profiler {
            cache: cache,
            _access: AtomicU64::new(0u64),
            _miss: AtomicU64::new(0u64),
            _hit: AtomicU64::new(0u64),
            _tot_millis: AtomicU64::new(0u64),
            take_fn: SyncOnlineStats::new(),
            pop_fn: SyncOnlineStats::new(),
            push_fn: SyncOnlineStats::new(),
            get_fn: SyncOnlineStats::new(),
            path: None,
            unused_k: PhantomData,
            unused_r: PhantomData,
            unused_v: PhantomData,
        }
    }

    /// Wrap a `cache` into a "Cache profiler" and write results to
    /// a file pointed by `path` when it is dropped.
    pub fn with_path(cache: C, path: &Path) -> Self {
        Profiler {
            cache: cache,
            _access: AtomicU64::new(0u64),
            _miss: AtomicU64::new(0u64),
            _hit: AtomicU64::new(0u64),
            _tot_millis: AtomicU64::new(0u64),
            take_fn: SyncOnlineStats::new(),
            pop_fn: SyncOnlineStats::new(),
            push_fn: SyncOnlineStats::new(),
            get_fn: SyncOnlineStats::new(),
            path: Some(path.to_path_buf()),
            unused_k: PhantomData,
            unused_r: PhantomData,
            unused_v: PhantomData,
        }
    }

    /// Amount of cache access
    pub fn access(&self) -> u64 {
        self._access.load(Ordering::Relaxed)
    }
    /// Amount of cache misses
    pub fn miss(&self) -> u64 {
        self._miss.load(Ordering::Relaxed)
    }
    /// Amount of cache hit
    pub fn hit(&self) -> u64 {
        self._hit.load(Ordering::Relaxed)
    }
    /// Amount of cache hit
    pub fn millis(&self) -> u64 {
        self._tot_millis.load(Ordering::Relaxed)
    }

    /// Write profiler header.
    fn write_header<W: Write>(output: &mut W) -> std::io::Result<()> {
        write!(
            output,
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
    pub fn fprint(&self, out: &mut File) -> std::io::Result<()> {
        let (begin, end) = {
            let pos = out.seek(SeekFrom::Current(0))?;
            let begin = out.seek(SeekFrom::Start(0))?;
            let end = out.seek(SeekFrom::End(0))?;
            out.seek(SeekFrom::Start(pos))?;
            (begin, end)
        };

        if end - begin == 0 {
            Self::write_header(out)?;
        } else {
            out.seek(SeekFrom::End(0))?;
        }

        write!(
            out,
            "{access} {miss} {hit} 
{take_fn_mean} {take_fn_var} {take_fn_min} {take_fn_max} 
{pop_fn_mean} {pop_fn_var} {pop_fn_min} {pop_fn_max} 
{push_fn_mean} {push_fn_var} {push_fn_min} {push_fn_max} 
{get_fn_mean} {get_fn_var} {get_fn_min} {get_fn_max}",
            access = self._access.load(Ordering::Relaxed),
            miss = self._miss.load(Ordering::Relaxed),
            hit = self._hit.load(Ordering::Relaxed),
            take_fn_mean = self.take_fn.mean(),
            take_fn_var = self.take_fn.var(),
            take_fn_min = self.take_fn.min(),
            take_fn_max = self.take_fn.max(),
            pop_fn_mean = self.pop_fn.mean(),
            pop_fn_var = self.pop_fn.var(),
            pop_fn_min = self.pop_fn.min(),
            pop_fn_max = self.pop_fn.max(),
            push_fn_mean = self.push_fn.mean(),
            push_fn_var = self.push_fn.var(),
            push_fn_min = self.push_fn.min(),
            push_fn_max = self.push_fn.max(),
            get_fn_mean = self.get_fn.mean(),
            get_fn_var = self.get_fn.var(),
            get_fn_min = self.get_fn.min(),
            get_fn_max = self.get_fn.max()
        )
    }
}

impl<K, V, R, C> Drop for Profiler<K, V, R, C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R>,
{
    fn drop(&mut self) {
        if let Some(pathbuf) = &self.path {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(pathbuf)
                .expect("Profiler cache open output:");
            self.fprint(&mut file).expect("Profiler cache dump output:");
        }
    }
}

//----------------------------------------------------------------------------//
// Display Implementation                                                     //
//----------------------------------------------------------------------------//

impl<K, V, R, C> std::fmt::Debug for Profiler<K, V, R, C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R>,
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

impl<K, V, R, C> std::fmt::Display for Profiler<K, V, R, C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R>,
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
            self.take_fn.mean(),
            self.take_fn.var(),
            self.take_fn.min(),
            self.take_fn.max()
        )?;

        write!(
            f,
            "* fn pop (ns):   {:.2} (mean) | {:.2} (var) | {:.2} (min) | {:.2} (max)",
            self.pop_fn.mean(),
            self.pop_fn.var(),
            self.pop_fn.min(),
            self.pop_fn.max()
        )?;

        write!(
            f,
            "* fn push (ns):  {:.2} (mean) | {:.2} (var) | {:.2} (min) | {:.2} (max),",
            self.push_fn.mean(),
            self.push_fn.var(),
            self.push_fn.min(),
            self.push_fn.max()
        )?;

        write!(
            f,
            "* fn get (ns):   {:.2} (mean) | {:.2} (var) | {:.2} (min) | {:.2} (max)",
            self.get_fn.mean(),
            self.get_fn.var(),
            self.get_fn.min(),
            self.get_fn.max()
        )
    }
}

//----------------------------------------------------------------------------//
// Container implementation                                                   //
//----------------------------------------------------------------------------//

impl<K, V, R, C> Insert<K, V, R> for Profiler<K, V, R, C>
where
    K: Ord,
    R: Reference<V> + FromValue<V>,
    C: Container<K, V, R>,
{
}

impl<K, V, R, C> Container<K, V, R> for Profiler<K, V, R, C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R>,
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
    fn take(&mut self, key: &K) -> Option<R> {
        self._access.fetch_add(1, Ordering::SeqCst);
        let t0 = Instant::now();
        let out = self.cache.take(key);
        let tf = t0.elapsed().as_millis();
        self._tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.take_fn.push(tf as f64);

        match out {
            None => {
                self._miss.fetch_add(1, Ordering::SeqCst);
                None
            }
            Some(v) => {
                self._hit.fetch_add(1, Ordering::SeqCst);
                Some(v)
            }
        }
    }

    /// Counts for one cache access and one hit.
    /// See [`pop` function](../trait.Container.html)
    fn pop(&mut self) -> Option<(K, R)> {
        self._access.fetch_add(1, Ordering::SeqCst);
        self._hit.fetch_add(1, Ordering::SeqCst);
        let t0 = Instant::now();
        let out = self.cache.pop();
        let tf = t0.elapsed().as_millis();
        self._tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.pop_fn.push(tf as f64);
        out
    }

    fn push(&mut self, key: K, reference: R) -> Option<(K, R)> {
        self._access.fetch_add(1, Ordering::SeqCst);
        let t0 = Instant::now();
        let out = self.cache.push(key, reference);
        let tf = t0.elapsed().as_millis();
        self._tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.push_fn.push(tf as f64);

        match out {
            None => {
                self._miss.fetch_add(1, Ordering::SeqCst);
                None
            }
            Some(v) => {
                self._hit.fetch_add(1, Ordering::SeqCst);
                Some(v)
            }
        }
    }
}

//----------------------------------------------------------------------------//
// Sequential Trait                                                           //
//----------------------------------------------------------------------------//

impl<K, V, R, C> Sequential<K, V, R> for Profiler<K, V, R, C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R> + Sequential<K, V, R>,
{
    fn get(&mut self, key: &K) -> Option<&V> {
        self._access.fetch_add(1, Ordering::SeqCst);
        let t0 = Instant::now();

        let out = self.cache.get(key);
        let tf = t0.elapsed().as_millis();
        self._tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.get_fn.push(tf as f64);
        match out {
            None => {
                self._miss.fetch_add(1, Ordering::SeqCst);
                None
            }
            Some(v) => {
                self._hit.fetch_add(1, Ordering::SeqCst);
                Some(v)
            }
        }
    }

    /// Counts for one cache access.
    /// If key is found, count a hit else count a miss.
    /// See [`get_mut` function](../trait.Container.html)
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self._access.fetch_add(1, Ordering::SeqCst);
        let t0 = Instant::now();
        let out = self.cache.get_mut(key);
        let tf = t0.elapsed().as_millis();
        self._tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.get_fn.push(tf as f64);
        match out {
            None => {
                self._miss.fetch_add(1, Ordering::SeqCst);
                None
            }
            Some(v) => {
                self._hit.fetch_add(1, Ordering::SeqCst);
                Some(v)
            }
        }
    }
}

//----------------------------------------------------------------------------//
// Concurrent trait                                                           //
//----------------------------------------------------------------------------//

unsafe impl<K, V, R, C> Send for Profiler<K, V, R, C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R> + Concurrent<K, V, R>,
{
}

unsafe impl<K, V, R, C> Sync for Profiler<K, V, R, C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R> + Concurrent<K, V, R>,
{
}

impl<K, V, R, C> Concurrent<K, V, R> for Profiler<K, V, R, C>
where
    K: Ord,
    R: Reference<V>,
    C: Container<K, V, R> + Concurrent<K, V, R>,
{
    fn get(&mut self, key: &K) -> Option<RWLockGuard<&V>> {
        self._access.fetch_add(1, Ordering::SeqCst);
        let t0 = Instant::now();

        let out = self.cache.get(key);
        let tf = t0.elapsed().as_millis();
        self._tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.get_fn.push(tf as f64);
        match out {
            None => {
                self._miss.fetch_add(1, Ordering::SeqCst);
                None
            }
            Some(v) => {
                self._hit.fetch_add(1, Ordering::SeqCst);
                Some(v)
            }
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<RWLockGuard<&mut V>> {
        self._access.fetch_add(1, Ordering::SeqCst);
        let t0 = Instant::now();
        let out = self.cache.get_mut(key);
        let tf = t0.elapsed().as_millis();
        self._tot_millis.fetch_add(tf as u64, Ordering::SeqCst);
        self.get_fn.push(tf as f64);
        match out {
            None => {
                self._miss.fetch_add(1, Ordering::SeqCst);
                None
            }
            Some(v) => {
                self._hit.fetch_add(1, Ordering::SeqCst);
                Some(v)
            }
        }
    }
}

//----------------------------------------------------------------------------//
// Iterator trait implementation                                              //
//----------------------------------------------------------------------------//

// This is incompatible with Drop trait that dumps the container when it gets out
// of scope. Building this iterator drops the container profiler. Thus, there is
// no point trying to record statistics.
//
// impl<'a,K,V,R,C,I> IntoIterator for Profiler<'a,K,V,R,C>
// where K: Ord,
//       R: Reference<V>,
//       I: Iterator<Item=(K, V)>,
//       C: Container<K,V,R> + IntoIterator<Item=(K, V), IntoIter=I>,
// {
//     type Item=(K, V);
//     type IntoIter=I;

//     fn into_iter(self) -> Self::IntoIter { self.cache.into_iter() }
// }

/// Iterator of [Profiler](struct.Profiler.html) [container](../trait.Container.html).
pub struct ProfilerIter<'a, I>
where
    I: Iterator,
{
    access: &'a mut AtomicU64,
    hit: &'a mut AtomicU64,
    container_iterator: I,
}

impl<'a, I> Iterator for ProfilerIter<'a, I>
where
    I: Iterator,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self.container_iterator.next() {
            None => None,
            Some(item) => {
                self.access.fetch_add(1, Ordering::SeqCst);
                self.hit.fetch_add(1, Ordering::SeqCst);
                Some(item)
            }
        }
    }
}

impl<'a, K, V, R, C, I> IterMut<'a, K, V, R> for Profiler<K, V, R, C>
where
    K: 'a + Ord,
    V: 'a,
    R: 'a + Reference<V>,
    I: Iterator<Item = (&'a K, &'a mut V)>,
    C: Container<K, V, R> + IterMut<'a, K, V, R, Iterator = I>,
{
    type Iterator = ProfilerIter<'a, I>;

    fn iter_mut(&'a mut self) -> Self::Iterator {
        ProfilerIter {
            access: &mut self._access,
            hit: &mut self._hit,
            container_iterator: self.cache.iter_mut(),
        }
    }
}

impl<'a, K, V, R, C, I> Iter<'a, K, V, R> for Profiler<K, V, R, C>
where
    K: 'a + Ord,
    V: 'a,
    R: 'a + Reference<V>,
    I: Iterator<Item = (&'a K, &'a V)>,
    C: Container<K, V, R> + Iter<'a, K, V, R, Iterator = I>,
{
    type Iterator = ProfilerIter<'a, I>;

    fn iter(&'a mut self) -> Self::Iterator {
        ProfilerIter {
            access: &mut self._access,
            hit: &mut self._hit,
            container_iterator: self.cache.iter(),
        }
    }
}

//------------------------------------------------------------------------------------//
//                                        Tests                                       //
//------------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Profiler;
    use crate::container::concurrent::{tests, Sequential as SeqLock};
    use crate::container::sequential::Map;
    use crate::reference::Default;

    #[test]
    fn test_associative() {
        tests::test_sequential(
            Profiler::new(SeqLock::new(Map::<_, _, Default<_>>::new(1))),
            true,
        );
        tests::test_sequential(
            Profiler::new(SeqLock::new(Map::<_, _, Default<_>>::new(100))),
            true,
        );
        tests::test_concurrent(Profiler::new(SeqLock::new(Map::<
            _,
            _,
            Default<_>,
        >::new(100))));
    }
}
