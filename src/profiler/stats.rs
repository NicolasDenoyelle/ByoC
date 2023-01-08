use std::sync::atomic::{AtomicU64, Ordering};

/// Accumulator of call count and elapsed time.
pub(super) struct MethodStats {
    count: AtomicU64,
    elapsed: AtomicU64,
}

impl MethodStats {
    /// Create zeroed MethodStats.
    pub fn new() -> Self {
        MethodStats {
            count: AtomicU64::new(0),
            elapsed: AtomicU64::new(0),
        }
    }

    /// Accumulate count and elapsed time.
    pub fn add(&mut self, count: usize, elapsed: u128) {
        self.count.fetch_add(count as u64, Ordering::SeqCst);
        self.elapsed.fetch_add(elapsed as u64, Ordering::SeqCst);
    }

    /// Get (count, time) stats.
    pub fn read(&self) -> (u64, u64) {
        let count = self.count.load(Ordering::Relaxed);
        let elapsed = self.elapsed.load(Ordering::Relaxed);
        (count, elapsed)
    }
}

/// Accumulator of building block method stats.
pub(super) struct Stats {
    pub size: MethodStats,
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
            size: MethodStats::new(),
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
}
