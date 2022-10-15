use super::Profiler;
use crate::utils::get::LifeTimeGuard;
use crate::{Get, GetMut};
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

impl<K, V, C> Get<K, V> for Profiler<C>
where
    C: Get<K, V>,
{
    type Target = C::Target;

    fn get(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        let (time, out) = time_it!(self.cache.get(key));
        Clone::clone(&self.stats).as_mut().get.add(1, time);
        match out {
            Some(_) => Clone::clone(&self.stats).as_mut().hit.add(1, time),
            None => Clone::clone(&self.stats).as_mut().miss.add(1, time),
        };
        out
    }
}

impl<K, V, C> GetMut<K, V> for Profiler<C>
where
    C: GetMut<K, V>,
{
    type Target = C::Target;

    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        let (time, out) = time_it!(self.cache.get_mut(key));
        self.stats.as_mut().get_mut.add(1, time);
        match out {
            Some(_) => Clone::clone(&self.stats).as_mut().hit.add(1, time),
            None => Clone::clone(&self.stats).as_mut().miss.add(1, time),
        };
        out
    }
}
