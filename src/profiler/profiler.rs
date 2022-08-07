use super::Stats;
use crate::internal::SharedPtr;
#[cfg(feature = "serde")]
use serde::Deserialize;
use std::fs::File;
use std::io::Write;

macro_rules! write_it {
    ($struct:expr, $field:ident, $prefix:ident, $file:ident) => {
        let (n, time) = $struct.$field.read();
        writeln!(
            $file,
            "{}{} {} {}",
            $prefix,
            stringify!($field),
            n,
            time,
        )
        .unwrap();
    };
}

macro_rules! print_it {
    ($struct:expr, $field:ident, $prefix:ident) => {
        let (n, time) = $struct.$field.read();
        println!("{}{} {} {}", $prefix, stringify!($field), n, time);
    };
}

/// Possible ways of printing output stats when a `Profiler` container
/// is dropped.
#[derive(Clone)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize),
    serde(tag = "kind", content = "filename")
)]
pub enum ProfilerOutputKind {
    /// No output is printed.
    None,
    /// Output is printed to stdout.
    Stdout,
    /// Output is printed to a file of the given name.
    File(String),
}

/// [`BuildingBlock`](trait.BuildingBlock.html) wrapper to collect
/// accesses, misses, hits and statistics about methods access time.
///
/// [`Profiler`] is a building block wrapper that inherits the root traits from
/// [this crate](index.html), of the
/// [`BuildingBlock`](trait.BuildingBlock.html) it wraps.
/// The resulting profiled statistics is dumped when the [`Profiler`] is
/// dropped. The destination where to write the dump is specified by a
/// [`ProfilerOutputKind`](utils/profiler/enum.ProfilerOutputKind.html) enum.
/// It can be either `stdout`, a file or nothing.
///
/// When using the [`Profiler`] wrapper, the following events are counted:
/// * The time spent in methods:
/// [`count()`](trait.BuildingBlock.html#tymethod.count),
/// [`contains()`](trait.BuildingBlock.html#tymethod.contains),
/// [`take()`](trait.BuildingBlock.html#tymethod.take),
/// [`pop()`](trait.BuildingBlock.html#tymethod.pop),
/// [`push()`](trait.BuildingBlock.html#tymethod.push),
/// [`flush()`](trait.BuildingBlock.html#tymethod.contains),
/// [`get()`](trait.Get.html#tymethod.get),
/// [`get_mut()`](trait.GetMut.html#tymethod.get_mut).
/// * The time spent iterating on flushed items,
/// * cache hits and misses: when calling
/// [`contains()`](trait.BuildingBlock.html#tymethod.contains),
/// [`take()`](trait.BuildingBlock.html#tymethod.take),
/// [`get()`](trait.Get.html#tymethod.get) or
/// [`get_mut()`](trait.GetMut.html#tymethod.get_mut), if the key to lookup
/// is indeed in the container, the hit count is incremented, else, the miss
/// count is incremented.
///
/// This building block can also be built with a
/// [builder](builder/trait.Build.html#method.profile) pattern or from a
/// [configuration](config/configs/struct.ProfilerConfig.html).
/// Everything is counted in an atomic type so that it is safe to use the
/// [`Concurrent`](trait.Concurrent.html) trait if the wrapped container
/// implements it.
///
/// ## Examples
///
/// ```
/// use byoc::{BuildingBlock, Get};
/// use byoc::{Array, Profiler};
/// use byoc::utils::profiler::ProfilerOutputKind;
///
/// // Build a cache:
/// let c = Array::new(3);
///
/// // Wrap it into a profiler.
/// let mut c = Profiler::new("example", ProfilerOutputKind::None, c);
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
/// unsafe { c.get(&"second"); }
/// assert_eq!(c.hit_stats().0, 2);
/// assert_eq!(c.miss_stats().0, 1);
/// ```
pub struct Profiler<C> {
    pub(super) cache: C,
    pub(super) name: String,
    pub(super) output: ProfilerOutputKind,
    pub(super) stats: SharedPtr<Stats>,
}

impl<C> Drop for Profiler<C> {
    fn drop(&mut self) {
        let mut prefix = self.name.clone();
        prefix.push(' ');

        match &self.output {
            ProfilerOutputKind::None => {}
            ProfilerOutputKind::Stdout => {
                print_it!(self.stats.as_ref(), count, prefix);
                print_it!(self.stats.as_ref(), contains, prefix);
                print_it!(self.stats.as_ref(), take, prefix);
                print_it!(self.stats.as_ref(), pop, prefix);
                print_it!(self.stats.as_ref(), push, prefix);
                print_it!(self.stats.as_ref(), flush, prefix);
                print_it!(self.stats.as_ref(), flush_iter, prefix);
                print_it!(self.stats.as_ref(), get, prefix);
                print_it!(self.stats.as_ref(), get_mut, prefix);
                print_it!(self.stats.as_ref(), hit, prefix);
                print_it!(self.stats.as_ref(), miss, prefix);
            }
            ProfilerOutputKind::File(s) => match File::create(s) {
                Ok(mut f) => {
                    write_it!(self.stats.as_ref(), count, prefix, f);
                    write_it!(self.stats.as_ref(), contains, prefix, f);
                    write_it!(self.stats.as_ref(), take, prefix, f);
                    write_it!(self.stats.as_ref(), pop, prefix, f);
                    write_it!(self.stats.as_ref(), push, prefix, f);
                    write_it!(self.stats.as_ref(), flush, prefix, f);
                    write_it!(self.stats.as_ref(), flush_iter, prefix, f);
                    write_it!(self.stats.as_ref(), get, prefix, f);
                    write_it!(self.stats.as_ref(), get_mut, prefix, f);
                    write_it!(self.stats.as_ref(), hit, prefix, f);
                    write_it!(self.stats.as_ref(), miss, prefix, f);
                }
                Err(e) => {
                    println!(
                        "Failed to open file for writing: {}.\n{:?}",
                        s, e
                    )
                }
            },
        }
    }
}

impl<C> Profiler<C> {
    /// Wrap a building block into a `Profiler`.
    pub fn new(name: &str, output: ProfilerOutputKind, cache: C) -> Self {
        Profiler {
            cache,
            name: String::from(name),
            output,
            stats: SharedPtr::from(Stats::new()),
        }
    }

    /// Get a summary of (0) the number of
    /// [`count()`](trait.BuildingBlock.html#tymethod.count) method call
    /// and (1) the total time spent in nanoseconds in these calls.
    pub fn count_stats(&self) -> (u64, u64) {
        self.stats.as_ref().count.read()
    }
    /// Get a summary of (0) the number of
    /// [`contains()`](trait.BuildingBlock.html#tymethod.contain) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn contains_stats(&self) -> (u64, u64) {
        self.stats.as_ref().contains.read()
    }
    /// Get a summary of (0) the number of
    /// [`take()`](trait.BuildingBlock.html#tymethod.take) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn take_stats(&self) -> (u64, u64) {
        self.stats.as_ref().take.read()
    }
    /// Get a summary of (0) the number of
    /// [`pop()`](trait.BuildingBlock.html#tymethod.pop) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn pop_stats(&self) -> (u64, u64) {
        self.stats.as_ref().pop.read()
    }
    /// Get a summary of (0) the number of
    /// [`push()`](trait.BuildingBlock.html#tymethod.push) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn push_stats(&self) -> (u64, u64) {
        self.stats.as_ref().push.read()
    }
    /// Get a summary of (0) the number of
    /// [`flush()`](trait.BuildingBlock.html#tymethod.flush) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn flush_stats(&self) -> (u64, u64) {
        self.stats.as_ref().flush.read()
    }
    /// Get a summary of (0) the number of iterations performed on an
    /// iterator obtained through
    /// [`flush()`](trait.BuildingBlock.html#tymethod.flush) method
    /// and (1) the total time spent in nanoseconds on iterations.
    pub fn flush_iter_stats(&self) -> (u64, u64) {
        self.stats.as_ref().flush_iter.read()
    }
    /// Get a summary of (0) the number of
    /// [`get()`](trait.Get.html#tymethod.get) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn get_stats(&self) -> (u64, u64) {
        self.stats.as_ref().get.read()
    }
    /// Get a summary of (0) the number of
    /// [`get_mut()`](trait.Get.html#tymethod.get_mut) method
    /// call and (1) the total time spent in nanoseconds in these calls.
    pub fn get_mut_stats(&self) -> (u64, u64) {
        self.stats.as_ref().get_mut.read()
    }
    /// Get the total amount of time user key query was matched with a key
    /// in the container when calling
    /// [`contains()`](trait.BuildingBlock.html#tymethod.contains),
    /// [`take()`](trait.BuildingBlock.html#tymethod.take),
    /// [`get()`](trait.Get.html#tymethod.get) or
    /// [`get_mut()`](trait.Get.html#tymethod.get_mut) methods.
    /// The first element is the number of hits, and the second element
    /// is the total time spent in the method on a hit in nanoseconds.
    pub fn hit_stats(&self) -> (u64, u64) {
        self.stats.as_ref().hit.read()
    }
    /// Get the total amount of time user key query was not matched with a
    /// key in the container when calling
    /// [`contains()`](trait.BuildingBlock.html#tymethod.contains),
    /// [`take()`](trait.BuildingBlock.html#tymethod.take),
    /// [`get()`](trait.Get.html#tymethod.get) or
    /// [`get_mut()`](trait.Get.html#tymethod.get_mut) methods.
    /// The first element is the number of misses, and the second element
    /// is the total time spent in the method on a miss in nanoseconds.
    pub fn miss_stats(&self) -> (u64, u64) {
        self.stats.as_ref().miss.read()
    }

    /// Get the total time spent in methods call so far.
    pub fn time_stats(&self) -> u64 {
        let count_time = self.stats.as_ref().count.read().1;
        let contains_time = self.stats.as_ref().contains.read().1;
        let take_time = self.stats.as_ref().take.read().1;
        let pop_time = self.stats.as_ref().pop.read().1;
        let push_time = self.stats.as_ref().push.read().1;
        let flush_time = self.stats.as_ref().flush.read().1;
        let flush_iter_time = self.stats.as_ref().flush_iter.read().1;
        let get_time = self.stats.as_ref().get.read().1;
        let get_mut_time = self.stats.as_ref().get_mut.read().1;
        count_time
            + contains_time
            + take_time
            + pop_time
            + push_time
            + flush_time
            + flush_iter_time
            + get_time
            + get_mut_time
    }
}
