use rand::distributions::{DistIter, Uniform};
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Binomial, Distribution, Hypergeometric};
use std::iter::Iterator;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct SeedCell {
    counter: Arc<AtomicU64>,
    seed: u64,
    adder: u64,
}

impl Default for SeedCell {
    fn default() -> Self {
        SeedCell {
            counter: Arc::new(AtomicU64::new(1u64)),
            seed: 0u64,
            adder: 1u64,
        }
    }
}

impl SeedCell {
    pub fn get_seed(&self) -> [u8; 32] {
        let seed = self.seed;
        let s0: u8 = (seed & 0xff).try_into().unwrap();
        let s1: u8 = ((seed & 0xff00) >> 8).try_into().unwrap();
        let s2: u8 = ((seed & 0xff0000) >> 16).try_into().unwrap();
        let s3: u8 = ((seed & 0xff000000) >> 32).try_into().unwrap();
        [
            s0, s1, s2, s3, s0, s1, s2, s3, s0, s1, s2, s3, s0, s1, s2,
            s3, s0, s1, s2, s3, s0, s1, s2, s3, s0, s1, s2, s3, s0, s1,
            s2, s3,
        ]
    }
}

impl Clone for SeedCell {
    fn clone(&self) -> Self {
        SeedCell {
            counter: Arc::clone(&self.counter),
            seed: self.counter.fetch_add(self.adder, Ordering::Relaxed),
            adder: self.adder,
        }
    }
}

pub struct RandomUniform {
    low: u64,
    high: u64,
    seed: SeedCell,
    it: DistIter<Uniform<u64>, StdRng, u64>,
}

impl RandomUniform {
    pub fn new(low: u64, high: u64) -> Self {
        let seed = SeedCell::default();
        let rng = StdRng::from_seed(seed.get_seed());
        let d = Uniform::new(low, high);
        RandomUniform {
            low,
            high,
            seed,
            it: d.sample_iter(rng),
        }
    }
}

impl Iterator for RandomUniform {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next()
    }
}

impl Clone for RandomUniform {
    fn clone(&self) -> Self {
        let d = Uniform::new(self.low, self.high);
        let seed_cell = self.seed.clone();
        let rng = StdRng::from_seed(seed_cell.get_seed());
        RandomUniform {
            low: self.low,
            high: self.high,
            seed: seed_cell,
            it: d.sample_iter(rng),
        }
    }
}

pub struct RandomBinomial {
    n: u64,
    p: f64,
    seed: SeedCell,
    it: DistIter<Binomial, StdRng, u64>,
}

impl RandomBinomial {
    pub fn new(n: u64, p: f64) -> Self {
        if p <= 0.0 || p >= 1.0 {
            panic!(
                "RandomBinomial binomial probability must be in ]0, 1["
            );
        }
        let d = Binomial::new(n, p).unwrap();
        let seed = SeedCell::default();
        let rng = StdRng::from_seed(seed.get_seed());
        RandomBinomial {
            n,
            p,
            seed,
            it: d.sample_iter(rng),
        }
    }
}

impl Clone for RandomBinomial {
    fn clone(&self) -> Self {
        let d = Binomial::new(self.n, self.p).unwrap();
        let seed_cell = self.seed.clone();
        let rng = StdRng::from_seed(seed_cell.get_seed());

        RandomBinomial {
            n: self.n,
            p: self.p,
            seed: seed_cell,
            it: d.sample_iter(rng),
        }
    }
}

impl Iterator for RandomBinomial {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next()
    }
}

pub struct RandomHypergeometric {
    total_population_size: u64,
    population_with_feature: u64,
    sample_size: u64,
    seed: SeedCell,
    it: DistIter<Hypergeometric, StdRng, u64>,
}

impl RandomHypergeometric {
    pub fn new(
        total_population_size: u64,
        population_with_feature: u64,
        sample_size: u64,
    ) -> Self {
        if sample_size >= total_population_size {
            panic!("RandomHypergeometric total population size should be greater than sample size.");
        }

        let d = Hypergeometric::new(
            total_population_size,
            population_with_feature,
            sample_size,
        )
        .unwrap();
        let seed = SeedCell::default();
        let rng = StdRng::from_seed(seed.get_seed());
        RandomHypergeometric {
            total_population_size,
            population_with_feature,
            sample_size,
            seed,
            it: d.sample_iter(rng),
        }
    }
}

impl Clone for RandomHypergeometric {
    fn clone(&self) -> Self {
        let d = Hypergeometric::new(
            self.total_population_size,
            self.population_with_feature,
            self.sample_size,
        )
        .unwrap();
        let seed_cell = self.seed.clone();
        let rng = StdRng::from_seed(seed_cell.get_seed());

        RandomHypergeometric {
            total_population_size: self.total_population_size,
            population_with_feature: self.population_with_feature,
            sample_size: self.sample_size,
            seed: seed_cell,
            it: d.sample_iter(rng),
        }
    }
}

impl Iterator for RandomHypergeometric {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next()
    }
}

// pub struct Step {
//     start: u64,
//     end: u64,
//     step: u64,
//     pos: u64,
//     counter: Arc<AtomicU64>,
// }

// impl Iterator for Step {
//     type Item = u64;
//     fn next(&mut self) -> Option<Self::Item> {
//         let cur = self.pos;
//         if self.pos + self.step >= self.end {
//             self.start = (self.start + 1) % self.step;
//             self.pos = self.start;
//         } else {
//             self.pos += self.step;
//         }
//         Some(cur)
//     }
// }

// impl Step {
//     pub fn new(num: u64, step: u64) -> Self {
//         if step >= num {
//             panic!("Step iterator step must be less than the total number iterated");
//         }
//         Step {
//             start: 0u64,
//             end: num,
//             step,
//             pos: 0u64,
//             counter: Arc::new(AtomicU64::new(1u64)),
//         }
//     }
// }

// impl Clone for Step {
//     fn clone(&self) -> Self {
//         let start = self.counter.load(Ordering::Relaxed) % self.step;
//         Step {
//             start,
//             end: self.end,
//             step: self.step,
//             pos: start,
//             counter: Arc::clone(&self.counter),
//         }
//     }
// }
