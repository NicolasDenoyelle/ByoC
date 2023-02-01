use super::Profiler;
use super::Stats;
use crate::utils::SharedPtr;
use crate::BuildingBlock;
use std::time::Instant;

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

/// Get the absolute size difference of a container surrounding a call.
macro_rules! size_it {
    ($container: expr, $call:expr) => {{
        let s0 = $container.size();
        let out = $call;
        let s1 = $container.size();
        let s_tot = if s0 > s1 { s0 - s1 } else { s1 - s0 };
        (s_tot, out)
    }};
}

/// Iterator of flushed elements counting time iterating and number
/// of iterations.
pub struct ProfilerFlushIter<I> {
    elements: I,
    stats: SharedPtr<Stats>,
}

impl<I: Iterator> Iterator for ProfilerFlushIter<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let (time, item) = time_it!(self.elements.next());
        Clone::clone(&self.stats)
            .as_mut()
            .flush_iter
            .add(1, time, 0);
        item
    }
}

impl<K, V, C> BuildingBlock<K, V> for Profiler<C>
where
    C: BuildingBlock<K, V>,
{
    /// Get the maximum storage size of this [`BuildingBlock`].
    ///
    /// This is the size of the container wrapped in this [`Profiler`]
    /// container. At the moment no profiling is performed on this method.
    fn capacity(&self) -> usize {
        self.cache.capacity()
    }

    /// Get the size currently occupied by elements in this [`BuildingBlock`].
    ///
    /// This is the size held in the container wrapped in this [`Profiler`]
    /// container.
    fn size(&self) -> usize {
        self.cache.size()
    }

    /// Check if container contains a matching key.
    ///
    /// This is calls the same method from the wrapped container.
    ///
    /// This function execution time is profiled and saved into
    /// the container statistics while incrementing the number of calls to this
    /// method.
    ///
    /// Additionally, if the key was found, then the "hit" counter is
    /// incremented, otherwise the "miss" counter is incremented.
    fn contains(&self, key: &K) -> bool {
        let (time, out) = time_it!(self.cache.contains(key));
        Clone::clone(&self.stats).as_mut().contains.add(1, time, 0);
        match out {
            true => Clone::clone(&self.stats).as_mut().hit.add(1, time, 0),
            false => {
                Clone::clone(&self.stats).as_mut().miss.add(1, time, 0)
            }
        };
        out
    }

    /// Take the matching key/value pair out of the container.
    ///
    /// This is calls the same method from the wrapped container.
    ///
    /// This function execution time is profiled and saved into
    /// the container statistics while incrementing the number of calls to this
    /// method.
    ///
    /// Additionally, if the key was found, then the "hit" counter is
    /// incremented, otherwise the "miss" counter is incremented.    
    fn take(&mut self, key: &K) -> Option<(K, V)> {
        let (size, (time, out)) =
            size_it!(self.cache, time_it!(self.cache.take(key)));
        self.stats.as_mut().take.add(1, time, size);
        match out {
            Some(_) => {
                Clone::clone(&self.stats).as_mut().hit.add(1, time, 0)
            }
            None => {
                Clone::clone(&self.stats).as_mut().miss.add(1, time, 0)
            }
        };
        out
    }

    /// Take multiple keys out of a container at once.
    ///
    /// This is calls the same method from the wrapped container.
    ///
    /// This function execution time is profiled and saved into
    /// the container statistics. The counter of number of calls to
    /// [`take()`](struct.Profiler.html#method.take) method is incremented
    /// by the number of keys to take.
    ///
    /// Additionally, the counter of "hits" is incremented by the number of
    /// keys that were found while the counter of "misses" is incremented by
    /// the number of keys that were not found.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let n = keys.len();
        let (size, (time, out)) =
            size_it!(self.cache, time_it!(self.cache.take_multiple(keys)));
        self.stats.as_mut().take.add(n, time, size);
        let hits = out.len();
        let misses = n - hits;
        self.stats.as_mut().hit.add(hits, time, 0);
        self.stats.as_mut().miss.add(misses, time, 0);
        out
    }

    /// Free up to `size` space from the container.
    ///
    /// This is calls the same method from the wrapped container.
    ///
    /// This function execution time is profiled and saved into
    /// the container statistics. The counter of number of calls to
    /// [`pop()`](struct.Profiler.html#method.pop) method is incremented
    /// by the size effectively popped out of the container.
    fn pop(&mut self, size: usize) -> Vec<(K, V)> {
        let size_before = self.cache.size();
        let (size, (time, out)) =
            size_it!(self.cache, time_it!(self.cache.pop(size)));
        let size_after = self.cache.size();
        self.stats
            .as_mut()
            .pop
            .add(size_before - size_after, time, size);
        out
    }

    /// Insert key/value pairs in the container.
    ///
    /// This is calls the same method from the wrapped container.
    ///
    /// This function execution time is profiled and saved into
    /// the container statistics. The counter of number of calls to
    /// [`push()`](struct.Profiler.html#method.push) method is incremented
    /// by the number of elements to insert in the container.
    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let n = elements.len();
        let (time, out) = time_it!(self.cache.push(elements));
        self.stats.as_mut().push.add(n, time, 0);
        out
    }

    type FlushIterator = ProfilerFlushIter<C::FlushIterator>;
    /// Empty the container and retrieve all of its elements.
    ///
    /// This is calls the same method from the wrapped container.
    ///
    /// This function execution time is profiled and saved into
    /// the container statistics. The counter of number of calls to this
    /// method is incremented by one.
    ///
    /// The returned iterator, will also profile its iterations over the flushed
    /// elements.
    fn flush(&mut self) -> Self::FlushIterator {
        let (size, (time, out)) =
            size_it!(self.cache, time_it!(self.cache.flush()));
        self.stats.as_mut().flush.add(1, time, size);
        ProfilerFlushIter {
            elements: out,
            stats: Clone::clone(&self.stats),
        }
    }
}
