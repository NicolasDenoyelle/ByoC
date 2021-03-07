use crate::lock::RWLock;
use std::marker::Sync;

/// Facility to compute online statistics with O(1) memory footprint.
#[derive(Debug)]
pub struct OnlineStats {
    n: u64,
    m2: f64,
    mean: f64,
    max: f64,
    min: f64,
}

impl OnlineStats {
    /// Initialize a new statistics counter.
    pub fn new() -> Self {
        OnlineStats {
            n: 0,
            m2: 0.0,
            mean: 0.0,
            max: std::f64::MIN,
            min: std::f64::MAX,
        }
    }

    /// Update statistics acounting for a new element.
    /// After update, variance, mean etc... account for all elements,
    /// provided through this method.
    pub fn push(&mut self, x: f64) {
        // Welford's online algorithm
        // https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance
        self.n += 1;
        let delta = x - self.mean;
        self.mean += delta / self.n as f64;
        let delta2 = x - self.mean;
        self.m2 += delta * delta2;

        // Min, Max
        self.min = if x < self.min { x } else { self.min };
        self.max = if x > self.max { x } else { self.max };
    }

    /// Return the maximum of elements pushed with `push` method.
    pub fn max(&self) -> f64 {
        self.max
    }
    /// Return the minimum of elements pushed with `push` method.
    pub fn min(&self) -> f64 {
        self.min
    }
    /// Return the mean of elements pushed with `push` method.
    pub fn mean(&self) -> f64 {
        if self.n == 0 {
            std::f64::NAN
        } else {
            self.mean
        }
    }
    /// Return the variance of elements pushed with `push` method.
    pub fn var(&self) -> f64 {
        if self.n < 2 {
            std::f64::NAN
        } else {
            self.m2 / self.n as f64
        }
    }
}

/// A structure for computing concurrent incremental statistics
/// This the thread safe version of [OnlineStats](struct.OnlineStats.html)
pub struct SyncOnlineStats {
    stats: OnlineStats,
    lock: RWLock,
}

impl SyncOnlineStats {
    /// Initialize a new statistics counter.
    pub fn new() -> Self {
        SyncOnlineStats {
            stats: OnlineStats::new(),
            lock: RWLock::new(),
        }
    }

    /// Update statistics acounting for a new element.
    /// After update, variance, mean etc... account for all elements,
    /// provided through this method.
    pub fn push(&mut self, x: f64) {
        let _ = self.lock.lock_mut_for(()).unwrap();
        self.stats.push(x)
    }

    /// Return the maximum of elements pushed with `push` method.
    pub fn max(&self) -> f64 {
        let _ = self.lock.lock_for(()).unwrap();
        self.stats.max()
    }

    /// Return the minimum of elements pushed with `push` method.
    pub fn min(&self) -> f64 {
        let _ = self.lock.lock_for(()).unwrap();
        self.stats.min()
    }

    /// Return the mean of elements pushed with `push` method.
    pub fn mean(&self) -> f64 {
        let _ = self.lock.lock_for(()).unwrap();
        self.stats.mean()
    }

    /// Return the variance of elements pushed with `push` method.
    pub fn var(&self) -> f64 {
        let _ = self.lock.lock_for(()).unwrap();
        self.stats.var()
    }
}

unsafe impl Send for SyncOnlineStats {}
unsafe impl Sync for SyncOnlineStats {}

#[cfg(test)]
mod tests {
    use super::OnlineStats;

    #[test]
    fn tests() {
        let mut stats = OnlineStats::new();
        for i in 1..6 {
            stats.push(i as f64)
        }
        assert_eq!(stats.max(), 5f64);
        assert_eq!(stats.min(), 1f64);
        assert_eq!(stats.mean(), 3.0);
        assert_eq!(stats.var(), 2.0);
    }
}
