use crate::benchmarks::{Key, Value};
use byoc::BuildingBlock;

use crate::benchmarks::pattern::{
    RandomBinomial, RandomHypergeometric, RandomUniform,
};
use crate::benchmarks::set::Set;

#[derive(Copy, Clone)]
/// Benchmarks container initialization patterns.
pub enum Initializer {
    /// Fill the container with all keys from 0 to container size.
    Fill,
    /// Enumerate and insert keys sequentially from 0 to container size
    /// separated with a step.
    Step(usize),
    /// Pick a set of `container_size` random keys in [0, `container_size`[
    /// with a uniform distribution,
    /// delete duplicates and insert keys in the container.
    RandomUniform,
    /// Pick a set of `container_size` random keys in [0, `container_size`[
    /// with a binomial distribution,
    /// delete duplicates and insert keys in the container.
    /// The distribution probability parameter can be set with a value in
    /// ]0, 1[.
    RandomBinomial(f64),
    /// Pick a set of `container_size` random keys in [0, `container_size`[
    /// with a hypergeometric distribution,
    /// delete duplicates and insert keys in the container.
    /// The distribution parameters: total population size (0) and
    /// number of trials (1) can be set while the number of successful
    /// features are set to the container size. Therefore, values drawn
    /// can only be in between 0 and container size.
    RandomHypergeometric(u64, u64),
}

#[allow(clippy::from_over_into)]
impl Into<String> for Initializer {
    fn into(self) -> String {
        match self {
            Initializer::Fill => String::from(Initializer::FILL_PREFIX),
            Initializer::Step(n) => {
                format!("{}:{}", Initializer::STEP_PREFIX, n)
            }
            Initializer::RandomUniform => {
                String::from(Initializer::RANDOM_UNIFORM_PREFIX)
            }
            Initializer::RandomBinomial(n) => {
                format!("{}:{}", Initializer::RANDOM_BINOMIAL_PREFIX, n)
            }
            Initializer::RandomHypergeometric(n, m) => {
                format!(
                    "{}:{}:{}",
                    Initializer::RANDOM_HYPERGEOMETRIC_PREFIX,
                    n,
                    m
                )
            }
        }
    }
}

impl From<&str> for Initializer {
    fn from(string: &str) -> Initializer {
        let mut it = string.split(':');

        let prefix = match it.next() {
            Some(p) => p,
            None => panic!("Invalid empty initializer"),
        };

        match prefix {
            Initializer::FILL_PREFIX => Initializer::Fill,
            Initializer::STEP_PREFIX => {
                match it.next().map(|s| s.parse()) {
                    Some(Ok(n)) => Initializer::Step(n),
                    _ => panic!(
                        "Invalid initializer syntax. Expected {}:<usize>",
                        Initializer::STEP_PREFIX
                    ),
                }
            }
            Initializer::RANDOM_UNIFORM_PREFIX => {
                Initializer::RandomUniform
            }
            Initializer::RANDOM_BINOMIAL_PREFIX => {
                match it.next().map(|s| s.parse()) {
                    Some(Ok(n)) => Initializer::RandomBinomial(n),
                    _ => panic!(
                        "Invalid initializer syntax. Expected {}:<f64>",
                        Initializer::RANDOM_BINOMIAL_PREFIX
                    ),
                }
            }
            Initializer::RANDOM_HYPERGEOMETRIC_PREFIX => {
                match (it.next(), it.next()) {
                    (Some(n), Some(m)) => {
                        let n = n.parse().unwrap();
                        let m = m.parse().unwrap();
                        Initializer::RandomHypergeometric(n, m)
                    }
                    _ => panic!(
                        "Invalid initializer syntax. Expected {}:<u64>:<u64>",
                        Initializer::RANDOM_HYPERGEOMETRIC_PREFIX
                    ),
                }
            }
            _ => panic!(
                "Invalid initializer format {}. {}",
                string,
                Initializer::HELP_STR
            ),
        }
    }
}

impl Initializer {
    pub const FILL_PREFIX: &'static str = "fill";
    pub const STEP_PREFIX: &'static str = "step";
    pub const RANDOM_UNIFORM_PREFIX: &'static str = "random_uniform";
    pub const RANDOM_BINOMIAL_PREFIX: &'static str = "random_binomial";
    pub const RANDOM_HYPERGEOMETRIC_PREFIX: &'static str =
        "random_hypergeometric";
    pub const HELP_STR: &'static str = "fill\nstep:<usize>\nrandom_uniform\nrandom_binomial:<probability>\nrandom_hypergeometric:<total_population_size>:<sample_size>\n";

    /// Initialize a container with this initializer.
    /// The container is flushed first then the appropriate initializer is
    /// applied.
    pub fn initialize<'a, C: BuildingBlock<'a, Key, Value>>(
        self,
        container: &mut C,
    ) {
        let num_keys = container.capacity() as u64;
        match self {
            Initializer::Fill => {
                container.push(
                    (0u64..num_keys)
                        .map(|k| (k, Value::default()))
                        .collect(),
                );
            }
            Initializer::Step(n) => {
                if n as u64 >= num_keys {
                    panic!(
                        "Step {} must be less than container capacity {}.",
                        n, num_keys
                    );
                }
                container.push(
                    (0u64..num_keys)
                        .step_by(n)
                        .map(|k| (k, Value::default()))
                        .collect(),
                );
            }
            Initializer::RandomUniform => {
                let ru = RandomUniform::new(0u64, num_keys)
                    .set()
                    .take(num_keys as usize);
                container
                    .push(ru.map(|k| (k, Value::default())).collect());
            }
            Initializer::RandomBinomial(p) => {
                let bin = RandomBinomial::new(num_keys, p)
                    .take(num_keys as usize)
                    .set();
                container
                    .push(bin.map(|k| (k, Value::default())).collect());
            }
            Initializer::RandomHypergeometric(t, n) => {
                if t < num_keys {
                    panic!("RandomHypergeometric total population size must be at least container capacity.");
                }
                let hyper = RandomHypergeometric::new(t, num_keys, n)
                    .take(num_keys as usize)
                    .set();
                container
                    .push(hyper.map(|k| (k, Value::default())).collect());
            }
        };
    }
}
