use rand::distributions::{DistIter, Uniform};
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Binomial, Distribution, Hypergeometric};

#[derive(Clone, Copy, Debug)]
pub struct RandomUniformGenerator {
    low: u64,
    high: u64,
    seed: u64,
}

impl RandomUniformGenerator {
    pub fn new(low: u64, high: u64, seed: u64) -> Self {
        RandomUniformGenerator { low, high, seed }
    }
}

impl IntoIterator for RandomUniformGenerator {
    type IntoIter = DistIter<Uniform<u64>, StdRng, u64>;
    type Item = u64;

    fn into_iter(self) -> Self::IntoIter {
        let rng = StdRng::seed_from_u64(self.seed);
        Uniform::new(self.low, self.high).sample_iter(rng)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RandomBinomialmGenerator {
    n: u64,
    p: f64,
    seed: u64,
}

impl RandomBinomialmGenerator {
    pub fn new(n: u64, p: f64, seed: u64) -> Self {
        if p <= 0.0 || p >= 1.0 {
            panic!(
                "RandomBinomialmGenerator binomial probability must be in ]0, 1["
            );
        }
        RandomBinomialmGenerator { n, p, seed }
    }
}

impl IntoIterator for RandomBinomialmGenerator {
    type IntoIter = DistIter<Binomial, StdRng, u64>;
    type Item = u64;

    fn into_iter(self) -> Self::IntoIter {
        let rng = StdRng::seed_from_u64(self.seed);
        Binomial::new(self.n, self.p).unwrap().sample_iter(rng)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RandomHypergeometricGenerator {
    total_population_size: u64,
    population_with_feature: u64,
    sample_size: u64,
    seed: u64,
}

impl RandomHypergeometricGenerator {
    pub fn new(
        total_population_size: u64,
        population_with_feature: u64,
        sample_size: u64,
        seed: u64,
    ) -> Self {
        if sample_size >= total_population_size {
            panic!("RandomHypergeometricGenerator total population size should be greater than sample size.");
        }
        RandomHypergeometricGenerator {
            total_population_size,
            population_with_feature,
            sample_size,
            seed,
        }
    }
}

impl IntoIterator for RandomHypergeometricGenerator {
    type IntoIter = DistIter<Hypergeometric, StdRng, u64>;
    type Item = u64;

    fn into_iter(self) -> Self::IntoIter {
        let rng = StdRng::seed_from_u64(self.seed);
        Hypergeometric::new(
            self.total_population_size,
            self.population_with_feature,
            self.sample_size,
        )
        .unwrap()
        .sample_iter(rng)
    }
}
