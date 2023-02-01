use super::Stats;
use crate::utils::SharedPtr;

/// `BuildingBlock` wrapper to collect accesses, misses, hits and statistics
/// about methods access time.
///
/// [`Profiler`] is a building block wrapper that inherits the root traits from
/// [this crate](index.html), of the
/// [`BuildingBlock`](trait.BuildingBlock.html) it wraps.
///
/// This building block can also be built with a
/// [builder](builder/trait.Build.html#method.profile) pattern or from a
/// [configuration](config/configs/struct.ProfilerConfig.html).
/// Everything is counted in an atomic type so that it is safe to use the
/// [`Concurrent`](trait.Concurrent.html) trait if the wrapped container
/// implements it.
///
/// ## [`BuildingBlock`](trait.BuildingBlock.html) Implementation
///
/// This is a simple wrapper calling into the wrapped container methods while
/// counting statistics in each method:
/// * [`size()`](trait.BuildingBlock.html#tymethod.size):
/// Time spent in the method,
/// * [`contains()`](trait.BuildingBlock.html#tymethod.contains):
/// Time spent in the method, plus a miss if the key was not found, else a hit
/// * [`take()`](trait.BuildingBlock.html#tymethod.take):
/// Time spent in the method, plus a miss if the key was not found, else a hit
/// * [`pop()`](trait.BuildingBlock.html#tymethod.pop),
/// Time spent in the method,
/// * [`push()`](trait.BuildingBlock.html#tymethod.push):
/// Time spent in the method,
/// * [`flush()`](trait.BuildingBlock.html#tymethod.contains),
/// Time spent in the method, plus for each flushed item iterated, the iteration
/// time.
///
/// ## [`Get`](trait.Get.html) Implementation
///
/// This is a simple wrapper calling into the wrapped container methods while
/// counting statistics in each method:
/// [`get()`](trait.Get.html#tymethod.get):
/// Time spent in the method, plus a miss if the key was not found, else a hit
/// [`get_mut()`](trait.GetMut.html#tymethod.get_mut):
/// Time spent in the method, plus a miss if the key was not found, else a hit
///
/// ## Examples
///
/// ```
/// use byoc::{BuildingBlock, Get};
/// use byoc::{Array, Profiler};
///
/// // Build a cache:
/// let c = Array::new(3);
///
/// // Wrap it into a profiler.
/// let mut c = Profiler::new(c);
///
/// // Populate BuildingBlock
/// c.push(vec![("first", 0), ("second", 1)]);
/// assert_eq!(c.push_stats().0, (2));
///
/// // Check if a key is in the container.
/// c.contains(&"first");
/// assert_eq!(c.hit_stats().0, 1);
/// assert_eq!(c.miss_stats().0, 0);
///
/// // Try to take a non existing key out of the container.
/// c.take(&"third");
/// assert_eq!(c.hit_stats().0, 1);
/// assert_eq!(c.miss_stats().0, 1);
///
/// // Access a value in the container.
/// c.get(&"second");
/// assert_eq!(c.hit_stats().0, 2);
/// assert_eq!(c.miss_stats().0, 1);
/// ```
pub struct Profiler<C> {
    pub(super) cache: C,
    pub(super) stats: SharedPtr<Stats>,
}

impl<C> Profiler<C> {
    /// Wrap a building block into a `Profiler`.
    pub fn new(cache: C) -> Self {
        Profiler {
            cache,
            stats: SharedPtr::from(Stats::new()),
        }
    }

    /// Get a report summary for the
    /// [`contains()`](trait.BuildingBlock.html#tymethod.contain) method.
    /// Values returned are respectively:
    /// 0. The number of calls to the method.
    /// 1. The total time spent in the method (in nanoseconds).
    pub fn contains_stats(&self) -> (u64, u64) {
        let (count, time, _) = self.stats.as_ref().contains.read();
        (count, time)
    }

    /// Get a report summary for the
    /// [`take()`](trait.BuildingBlock.html#tymethod.take) method.
    /// Values returned are respectively:
    /// 0. The number of calls to the method.
    /// 1. The total time spent in the method (in nanoseconds).
    /// 2. The size difference of the container before and after the method was
    /// called.
    pub fn take_stats(&self) -> (u64, u64, u64) {
        self.stats.as_ref().take.read()
    }

    /// Get a report summary for the
    /// [`pop()`](trait.BuildingBlock.html#tymethod.pop) method.
    /// Values returned are respectively:
    /// 0. The number of calls to the method.
    /// 1. The total time spent in the method (in nanoseconds).
    /// 2. The size difference of the container before and after the method was
    /// called.
    pub fn pop_stats(&self) -> (u64, u64, u64) {
        self.stats.as_ref().pop.read()
    }

    /// Get a report summary for the
    /// [`push()`](trait.BuildingBlock.html#tymethod.push) method.
    /// Values returned are respectively:
    /// 0. The number of calls to the method.
    /// 1. The total time spent in the method (in nanoseconds).
    /// The size difference of the container before and after the method is
    /// called does not make sense for this method because it may result in
    /// evictions and show a small variation of the container size compared to
    /// the size of the elements to insert.
    pub fn push_stats(&self) -> (u64, u64) {
        let (count, time, _) = self.stats.as_ref().push.read();
        (count, time)
    }

    /// Get a report summary for the
    /// [`flush()`](trait.BuildingBlock.html#tymethod.flush) method.
    /// Values returned are respectively:
    /// 0. The number of calls to the method.
    /// 1. The total time spent in the method (in nanoseconds).
    /// 2. The size difference of the container before and after the method was
    /// called.
    pub fn flush_stats(&self) -> (u64, u64, u64) {
        self.stats.as_ref().flush.read()
    }

    /// Get a summary of (0) the number of iterations performed on an
    /// iterator obtained through
    /// [`flush()`](trait.BuildingBlock.html#tymethod.flush) method.
    /// Values returned are respectively:
    /// 0. The number of calls to the method.
    /// 1. The total time spent in the method (in nanoseconds).
    pub fn flush_iter_stats(&self) -> (u64, u64) {
        let (count, time, _) = self.stats.as_ref().flush_iter.read();
        (count, time)
    }

    /// Get a report summary for the
    /// [`get()`](trait.Get.html#tymethod.get) method.
    /// Values returned are respectively:
    /// 0. The number of calls to the method.
    /// 1. The total time spent in the method (in nanoseconds).
    pub fn get_stats(&self) -> (u64, u64) {
        let (count, time, _) = self.stats.as_ref().get.read();
        (count, time)
    }

    /// Get a report summary for the
    /// [`get_mut()`](trait.Get.html#tymethod.get_mut) method.
    /// Values returned are respectively:
    /// 0. The number of calls to the method.
    /// 1. The total time spent in the method (in nanoseconds).
    pub fn get_mut_stats(&self) -> (u64, u64) {
        let (count, time, _) = self.stats.as_ref().get_mut.read();
        (count, time)
    }

    /// Get the total amount of time user key query was matched with a key
    /// in the container when calling
    /// [`contains()`](trait.BuildingBlock.html#tymethod.contains),
    /// [`take()`](trait.BuildingBlock.html#tymethod.take),
    /// [`get()`](trait.Get.html#tymethod.get) or
    /// [`get_mut()`](trait.Get.html#tymethod.get_mut) methods.
    /// The first element is the number of hits, and the second element is the
    /// total time spent in the method on a hit (in nanoseconds),
    pub fn hit_stats(&self) -> (u64, u64) {
        let (count, time, _) = self.stats.as_ref().hit.read();
        (count, time)
    }

    /// Get the total amount of time user key query was not matched with a
    /// key in the container when calling
    /// [`contains()`](trait.BuildingBlock.html#tymethod.contains),
    /// [`take()`](trait.BuildingBlock.html#tymethod.take),
    /// [`get()`](trait.Get.html#tymethod.get) or
    /// [`get_mut()`](trait.Get.html#tymethod.get_mut) methods.
    /// The first element is the number of misses, and the second element
    /// is the total time spent in the method on a miss (in nanoseconds).
    pub fn miss_stats(&self) -> (u64, u64) {
        let (count, time, _) = self.stats.as_ref().miss.read();
        (count, time)
    }

    /// Get the total time spent in methods call so far.
    pub fn time_stats(&self) -> u64 {
        let contains_time = self.stats.as_ref().contains.read().1;
        let take_time = self.stats.as_ref().take.read().1;
        let pop_time = self.stats.as_ref().pop.read().1;
        let push_time = self.stats.as_ref().push.read().1;
        let flush_time = self.stats.as_ref().flush.read().1;
        let flush_iter_time = self.stats.as_ref().flush_iter.read().1;
        let get_time = self.stats.as_ref().get.read().1;
        let get_mut_time = self.stats.as_ref().get_mut.read().1;
        contains_time
            + take_time
            + pop_time
            + push_time
            + flush_time
            + flush_iter_time
            + get_time
            + get_mut_time
    }

    /// Set all stats to 0.
    pub fn reset(&mut self) {
        self.stats.as_mut().reset();
    }
}

impl<'a, K, V, C> From<Profiler<C>> for crate::DynBuildingBlock<'a, K, V>
where
    K: 'a,
    V: 'a,
    C: 'a + crate::BuildingBlock<K, V> + crate::Concurrent,
{
    fn from(profiler: Profiler<C>) -> Self {
        crate::DynBuildingBlock::new(profiler, true)
    }
}
