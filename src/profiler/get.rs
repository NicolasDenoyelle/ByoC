use super::Profiler;
use crate::{Get, GetMut};
use std::ops::{Deref, DerefMut};
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

impl<K, V, U, C> Get<K, V, U> for Profiler<C>
where
    U: Deref<Target = V>,
    C: Get<K, V, U>,
{
    unsafe fn get(&self, key: &K) -> Option<U> {
        let (time, out) = time_it!(self.cache.get(key));
        Clone::clone(&self.stats).as_mut().get.add(1, time);
        match out {
            Some(_) => Clone::clone(&self.stats).as_mut().hit.add(1, time),
            None => Clone::clone(&self.stats).as_mut().miss.add(1, time),
        };
        out
    }
}

impl<K, V, W, C> GetMut<K, V, W> for Profiler<C>
where
    W: DerefMut<Target = V>,
    C: GetMut<K, V, W>,
{
    unsafe fn get_mut(&mut self, key: &K) -> Option<W> {
        let (time, out) = time_it!(self.cache.get_mut(key));
        self.stats.as_mut().get_mut.add(1, time);
        match out {
            Some(_) => Clone::clone(&self.stats).as_mut().hit.add(1, time),
            None => Clone::clone(&self.stats).as_mut().miss.add(1, time),
        };
        out
    }
}
