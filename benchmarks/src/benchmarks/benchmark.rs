use crate::benchmarks::pattern::SeedCell;
use crate::benchmarks::{Action, Initializer, Key, Value};
use byoc::{BuildingBlock, Concurrent, GetMut};
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Distribution, WeightedIndex};
use std::iter::Iterator;
use std::marker::PhantomData;
use std::ops::DerefMut;

pub struct Benchmark<'a, P, T, G, U, C>
where
    P: Iterator<Item = Key>,
    T: Iterator<Item = Key>,
    G: Iterator<Item = Key>,
    U: DerefMut<Target = Value>,
    C: BuildingBlock<'a, Key, Value> + GetMut<Key, Value, U>,
{
    container: C,
    seed: SeedCell,
    action_pattern: (WeightedIndex<f32>, StdRng),
    push_pattern: P,
    take_pattern: T,
    get_mut_pattern: G,
    unused: PhantomData<&'a U>,
}

impl<'a, P, T, G, U, C> Benchmark<'a, P, T, G, U, C>
where
    P: Iterator<Item = Key>,
    T: Iterator<Item = Key>,
    G: Iterator<Item = Key>,
    U: DerefMut<Target = Value>,
    C: BuildingBlock<'a, Key, Value> + GetMut<Key, Value, U>,
{
    fn default_weights() -> [f32; 6] {
        [10f32, 10f32, 100f32, 10f32, 10f32, 10f32]
    }

    pub fn new(
        container: C,
        push_pattern: P,
        take_pattern: T,
        get_mut_pattern: G,
    ) -> Self {
        let seed = SeedCell::default();
        let action_rng = StdRng::from_seed(seed.get_seed());
        let action_pattern =
            WeightedIndex::new(Self::default_weights()).unwrap();

        Self {
            container,
            seed,
            action_pattern: (action_pattern, action_rng),
            push_pattern,
            take_pattern,
            get_mut_pattern,
            unused: PhantomData,
        }
    }

    pub fn set_weight(&mut self, action: Action, weight: f32) {
        let pattern = &mut self.action_pattern.0;

        match action {
            Action::Push => pattern.update_weights(&[(0, &weight)]),
            Action::Take => pattern.update_weights(&[(1, &weight)]),
            Action::GetMut => pattern.update_weights(&[(2, &weight)]),
            Action::PushOrGetMut => {
                pattern.update_weights(&[(3, &weight)])
            }
            Action::PushOrTake => pattern.update_weights(&[(4, &weight)]),
            Action::GetMutOrPush => {
                pattern.update_weights(&[(5, &weight)])
            }
        }
        .expect("Make all benchmark weights >= 0 and with a sum > 0.");
    }

    pub fn initialize(&mut self, initializer: Initializer) {
        drop(self.container.flush());
        initializer.initialize(&mut self.container);
    }

    pub fn run(&mut self, num_iterations: usize) {
        for _ in 0..num_iterations {
            self.next();
        }
    }
}

impl<'a, P, T, G, U, C> Clone for Benchmark<'a, P, T, G, U, C>
where
    P: Iterator<Item = Key> + Clone,
    T: Iterator<Item = Key> + Clone,
    G: Iterator<Item = Key> + Clone,
    U: DerefMut<Target = Value>,
    C: BuildingBlock<'a, Key, Value> + GetMut<Key, Value, U> + Concurrent,
{
    fn clone(&self) -> Self {
        let seed = self.seed.clone();
        let action_rng = StdRng::from_seed(seed.get_seed());
        let action_pattern = self.action_pattern.0.clone();

        Self {
            container: Concurrent::clone(&self.container),
            seed,
            action_pattern: (action_pattern, action_rng),
            push_pattern: self.push_pattern.clone(),
            take_pattern: self.take_pattern.clone(),
            get_mut_pattern: self.get_mut_pattern.clone(),
            unused: PhantomData,
        }
    }
}

impl<'a, P, T, G, U, C> Iterator for Benchmark<'a, P, T, G, U, C>
where
    P: Iterator<Item = Key>,
    T: Iterator<Item = Key>,
    G: Iterator<Item = Key>,
    U: DerefMut<Target = Value>,
    C: BuildingBlock<'a, Key, Value> + GetMut<Key, Value, U>,
{
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        let (pattern, rng) = &mut self.action_pattern;
        match pattern.sample(rng) {
            0 => self.push_pattern.next().map(|key| {
                Action::Push.call_once(key, &mut self.container)
            }),
            1 => self.take_pattern.next().map(|key| {
                Action::Take.call_once(key, &mut self.container)
            }),
            2 => self.get_mut_pattern.next().map(|key| {
                Action::GetMut.call_once(key, &mut self.container)
            }),
            3 => self.push_pattern.next().map(|key| {
                Action::PushOrGetMut.call_once(key, &mut self.container)
            }),
            4 => self.push_pattern.next().map(|key| {
                Action::PushOrTake.call_once(key, &mut self.container)
            }),
            5 => self.get_mut_pattern.next().map(|key| {
                Action::GetMutOrPush.call_once(key, &mut self.container)
            }),
            _ => panic!("Out of range weighted index"),
        }
    }
}
