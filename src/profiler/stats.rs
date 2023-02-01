use std::sync::atomic::{AtomicU64, Ordering};

/// Accumulator of call count and elapsed time.
pub(super) struct MethodStats {
    // The total number of call.
    count: AtomicU64,
    // The total elapsed time for the calls.
    elapsed: AtomicU64,
    // The size moved/accessed by this call.
    size: AtomicU64,
}

impl MethodStats {
    /// Create zeroed MethodStats.
    pub fn new() -> Self {
        MethodStats {
            count: AtomicU64::new(0),
            elapsed: AtomicU64::new(0),
            size: AtomicU64::new(0),
        }
    }

    /// Reinitialize values to 0.
    pub fn reset(&mut self) {
        self.count = AtomicU64::new(0);
        self.elapsed = AtomicU64::new(0);
        self.size = AtomicU64::new(0);
    }

    /// Accumulate count and elapsed time.
    pub fn add(&mut self, count: usize, elapsed: u128, size: usize) {
        self.count.fetch_add(count as u64, Ordering::SeqCst);
        self.elapsed.fetch_add(elapsed as u64, Ordering::SeqCst);
        self.size.fetch_add(size as u64, Ordering::SeqCst);
    }

    /// Get (count, time) stats.
    pub fn read(&self) -> (u64, u64, u64) {
        let count = self.count.load(Ordering::Relaxed);
        let elapsed = self.elapsed.load(Ordering::Relaxed);
        let size = self.size.load(Ordering::Relaxed);
        (count, elapsed, size)
    }
}

/// Accumulator of building block method stats.
pub(super) struct Stats {
    pub contains: MethodStats,
    pub take: MethodStats,
    pub pop: MethodStats,
    pub push: MethodStats,
    pub flush: MethodStats,
    pub flush_iter: MethodStats,
    pub get: MethodStats,
    pub get_mut: MethodStats,
    pub hit: MethodStats,
    pub miss: MethodStats,
}

impl Stats {
    /// Initialize zeroed method stats.
    pub fn new() -> Self {
        Stats {
            contains: MethodStats::new(),
            take: MethodStats::new(),
            pop: MethodStats::new(),
            push: MethodStats::new(),
            flush: MethodStats::new(),
            flush_iter: MethodStats::new(),
            get: MethodStats::new(),
            get_mut: MethodStats::new(),
            hit: MethodStats::new(),
            miss: MethodStats::new(),
        }
    }

    /// Reinitialize values to 0.
    pub fn reset(&mut self) {
        self.contains.reset();
        self.take.reset();
        self.pop.reset();
        self.push.reset();
        self.flush.reset();
        self.flush_iter.reset();
        self.get.reset();
        self.get_mut.reset();
        self.hit.reset();
        self.miss.reset();
    }
}
