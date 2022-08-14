use super::Profiler;
use crate::Concurrent;

unsafe impl<C: Send> Send for Profiler<C> {}

unsafe impl<C: Sync> Sync for Profiler<C> {}

impl<C> Concurrent for Profiler<C>
where
    C: Concurrent,
{
    fn clone(&self) -> Self {
        Profiler {
            cache: Concurrent::clone(&self.cache),
            name: self.name.clone(),
            output: self.output.clone(),
            stats: Clone::clone(&self.stats),
        }
    }
}
