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
    fn capacity(&self) -> usize {
        self.cache.capacity()
    }

    fn size(&self) -> usize {
        let (time, count) = time_it!(self.cache.size());
        Clone::clone(&self.stats).as_mut().count.add(1, time);
        count
    }

    fn contains(&self, key: &K) -> bool {
        let (time, out) = time_it!(self.cache.contains(key));
        Clone::clone(&self.stats).as_mut().contains.add(1, time);
        match out {
            true => Clone::clone(&self.stats).as_mut().hit.add(1, time),
            false => Clone::clone(&self.stats).as_mut().miss.add(1, time),
        };
        out
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        let (time, out) = time_it!(self.cache.take(key));
        self.stats.as_mut().take.add(1, time);
        match out {
            Some(_) => Clone::clone(&self.stats).as_mut().hit.add(1, time),
            None => Clone::clone(&self.stats).as_mut().miss.add(1, time),
        };
        out
    }

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

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        let (time, out) = time_it!(self.cache.flush());
        self.stats.as_mut().flush.add(1, time);
        Box::new(ProfilerFlushIter {
            elements: out,
            stats: Clone::clone(&self.stats),
        })
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let (time, out) = time_it!(self.cache.pop(n));
        self.stats.as_mut().pop.add(n, time);
        out
    }

    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        let n = elements.len();
        let (time, out) = time_it!(self.cache.push(elements));
        self.stats.as_mut().push.add(n, time);
        out
    }
}
