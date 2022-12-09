use super::Profiler;
use super::Stats;
use crate::internal::SharedPtr;
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

/// Iterator of flushed elements counting time iterating and number
/// of iterations.
struct ProfilerFlushIter<'a, T> {
    elements: Box<dyn Iterator<Item = T> + 'a>,
    stats: SharedPtr<Stats>,
}

impl<'a, T> Iterator for ProfilerFlushIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let (time, item) = time_it!(self.elements.next());
        Clone::clone(&self.stats).as_mut().flush_iter.add(1, time);
        item
    }
}

impl<'a, K, V, C> BuildingBlock<'a, K, V> for Profiler<C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V>,
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
    ///
    /// This function execution time is profiled and saved into
    /// the container statistics while incrementing the number of calls to this
    /// method.
    fn size(&self) -> usize {
        let (time, size) = time_it!(self.cache.size());
        Clone::clone(&self.stats).as_mut().size.add(1, time);
        size
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
        Clone::clone(&self.stats).as_mut().contains.add(1, time);
        match out {
            true => Clone::clone(&self.stats).as_mut().hit.add(1, time),
            false => Clone::clone(&self.stats).as_mut().miss.add(1, time),
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
        let (time, out) = time_it!(self.cache.take(key));
        self.stats.as_mut().take.add(1, time);
        match out {
            Some(_) => Clone::clone(&self.stats).as_mut().hit.add(1, time),
            None => Clone::clone(&self.stats).as_mut().miss.add(1, time),
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
        let t0 = Instant::now();
        let out = self.cache.take_multiple(keys);
        let time = t0.elapsed().as_nanos();
        self.stats.as_mut().take.add(n, time);
        let hits = out.len();
        let misses = n - hits;
        self.stats.as_mut().hit.add(hits, time);
        self.stats.as_mut().miss.add(misses, time);
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
        let (time, out) = time_it!(self.cache.pop(size));
        let size_after = self.cache.size();
        self.stats.as_mut().pop.add(size_before - size_after, time);
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
        self.stats.as_mut().push.add(n, time);
        out
    }

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
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        let (time, out) = time_it!(self.cache.flush());
        self.stats.as_mut().flush.add(1, time);
        Box::new(ProfilerFlushIter {
            elements: out,
            stats: Clone::clone(&self.stats),
        })
    }
}
