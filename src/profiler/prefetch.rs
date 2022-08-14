use super::Profiler;
use crate::{BuildingBlock, Prefetch};
use std::time::Instant;

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
}
