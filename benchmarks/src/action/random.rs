use crate::action::{Action, ActionType, ACTION_TYPES};

use rand::distributions::{weighted::WeightedError, WeightedIndex};
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::Distribution;
use std::collections::BTreeMap;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct RandomActionTypeIterator {
    distribution: WeightedIndex<f32>,
    rng: StdRng,
}

impl RandomActionTypeIterator {
    pub fn new(self, seed: u64) -> Self {
        Self {
            distribution: WeightedIndex::new(
                ACTION_TYPES.iter().map(|_| 1.0f32),
            )
            .unwrap(),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn with_weights(
        mut self,
        weights: BTreeMap<ActionType, f32>,
    ) -> Result<Self, WeightedError> {
        let weights: Vec<(usize, &f32)> = weights
            .iter()
            .map(|(action, weight)| (*action as usize, weight))
            .collect();
        self.distribution
            .update_weights(weights.as_slice())
            .map(|_| self)
    }
}

impl Iterator for RandomActionTypeIterator {
    type Item = ActionType;

    fn next(&mut self) -> Option<Self::Item> {
        Some(ActionType::from(self.distribution.sample(&mut self.rng)))
    }
}

pub struct RandomActionGenerator<
    K,
    V,
    TakeActionGenerator,
    ContainsActionGenerator,
    GetActionGenerator,
    GetMutActionGenerator,
    PopActionGenerator,
    PushActionGenerator,
    TakeMultipleActionGenerator,
> {
    action_generator: RandomActionTypeIterator,
    take_action_generator: TakeActionGenerator,
    contains_action_generator: ContainsActionGenerator,
    get_action_generator: GetActionGenerator,
    get_mut_action_generator: GetMutActionGenerator,
    pop_action_generator: PopActionGenerator,
    push_action_generator: PushActionGenerator,
    take_multiple_action_generator: TakeMultipleActionGenerator,
    unused: PhantomData<(K, V)>,
}

impl<
        K,
        V,
        TakeActionGenerator,
        ContainsActionGenerator,
        GetActionGenerator,
        GetMutActionGenerator,
        PopActionGenerator,
        PushActionGenerator,
        TakeMultipleActionGenerator,
    >
    RandomActionGenerator<
        K,
        V,
        TakeActionGenerator,
        ContainsActionGenerator,
        GetActionGenerator,
        GetMutActionGenerator,
        PopActionGenerator,
        PushActionGenerator,
        TakeMultipleActionGenerator,
    >
{
    pub fn new(
        action_generator: RandomActionTypeIterator,
        take_action_generator: TakeActionGenerator,
        contains_action_generator: ContainsActionGenerator,
        get_action_generator: GetActionGenerator,
        get_mut_action_generator: GetMutActionGenerator,
        pop_action_generator: PopActionGenerator,
        push_action_generator: PushActionGenerator,
        take_multiple_action_generator: TakeMultipleActionGenerator,
    ) -> Self {
        Self {
            action_generator,
            take_action_generator,
            contains_action_generator,
            get_action_generator,
            get_mut_action_generator,
            pop_action_generator,
            push_action_generator,
            take_multiple_action_generator,
            unused: PhantomData,
        }
    }
}

impl<
        K,
        V,
        TakeActionGenerator: Clone,
        ContainsActionGenerator: Clone,
        GetActionGenerator: Clone,
        GetMutActionGenerator: Clone,
        PopActionGenerator: Clone,
        PushActionGenerator: Clone,
        TakeMultipleActionGenerator: Clone,
    > Clone
    for RandomActionGenerator<
        K,
        V,
        TakeActionGenerator,
        ContainsActionGenerator,
        GetActionGenerator,
        GetMutActionGenerator,
        PopActionGenerator,
        PushActionGenerator,
        TakeMultipleActionGenerator,
    >
{
    fn clone(&self) -> Self {
        RandomActionGenerator {
            action_generator: self.action_generator.clone(),
            take_action_generator: self.take_action_generator.clone(),
            contains_action_generator: self
                .contains_action_generator
                .clone(),
            get_action_generator: self.get_action_generator.clone(),
            get_mut_action_generator: self
                .get_mut_action_generator
                .clone(),
            pop_action_generator: self.pop_action_generator.clone(),
            push_action_generator: self.push_action_generator.clone(),
            take_multiple_action_generator: self
                .take_multiple_action_generator
                .clone(),
            unused: PhantomData,
        }
    }
}

impl<
        K,
        V,
        TakeActionGenerator: IntoIterator<Item = Action<K, V>>,
        ContainsActionGenerator: IntoIterator<Item = Action<K, V>>,
        GetActionGenerator: IntoIterator<Item = Action<K, V>>,
        GetMutActionGenerator: IntoIterator<Item = Action<K, V>>,
        PopActionGenerator: IntoIterator<Item = Action<K, V>>,
        PushActionGenerator: IntoIterator<Item = Action<K, V>>,
        TakeMultipleActionGenerator: IntoIterator<Item = Action<K, V>>,
    > IntoIterator
    for RandomActionGenerator<
        K,
        V,
        TakeActionGenerator,
        ContainsActionGenerator,
        GetActionGenerator,
        GetMutActionGenerator,
        PopActionGenerator,
        PushActionGenerator,
        TakeMultipleActionGenerator,
    >
{
    type IntoIter = RandomActionIterator<
        K,
        V,
        TakeActionGenerator::IntoIter,
        ContainsActionGenerator::IntoIter,
        GetActionGenerator::IntoIter,
        GetMutActionGenerator::IntoIter,
        PopActionGenerator::IntoIter,
        PushActionGenerator::IntoIter,
        TakeMultipleActionGenerator::IntoIter,
    >;
    type Item = Action<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        RandomActionIterator {
            action_iterator: self.action_generator.into_iter(),
            take_action_iterator: self.take_action_generator.into_iter(),
            contains_action_iterator: self
                .contains_action_generator
                .into_iter(),
            get_action_iterator: self.get_action_generator.into_iter(),
            get_mut_action_iterator: self
                .get_mut_action_generator
                .into_iter(),
            pop_action_iterator: self.pop_action_generator.into_iter(),
            push_action_iterator: self.push_action_generator.into_iter(),
            take_multiple_action_iterator: self
                .take_multiple_action_generator
                .into_iter(),
            unused: PhantomData,
        }
    }
}

pub struct RandomActionIterator<
    K,
    V,
    TakeActionIterator,
    ContainsActionIterator,
    GetActionIterator,
    GetMutActionIterator,
    PopActionIterator,
    PushActionIterator,
    TakeMultipleActionIterator,
> {
    action_iterator: RandomActionTypeIterator,
    take_action_iterator: TakeActionIterator,
    contains_action_iterator: ContainsActionIterator,
    get_action_iterator: GetActionIterator,
    get_mut_action_iterator: GetMutActionIterator,
    pop_action_iterator: PopActionIterator,
    push_action_iterator: PushActionIterator,
    take_multiple_action_iterator: TakeMultipleActionIterator,
    unused: PhantomData<(K, V)>,
}

impl<
        K,
        V,
        TakeActionIterator: Iterator<Item = Action<K, V>>,
        ContainsActionIterator: Iterator<Item = Action<K, V>>,
        GetActionIterator: Iterator<Item = Action<K, V>>,
        GetMutActionIterator: Iterator<Item = Action<K, V>>,
        PopActionIterator: Iterator<Item = Action<K, V>>,
        PushActionIterator: Iterator<Item = Action<K, V>>,
        TakeMultipleActionIterator: Iterator<Item = Action<K, V>>,
    > Iterator
    for RandomActionIterator<
        K,
        V,
        TakeActionIterator,
        ContainsActionIterator,
        GetActionIterator,
        GetMutActionIterator,
        PopActionIterator,
        PushActionIterator,
        TakeMultipleActionIterator,
    >
{
    type Item = Action<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        let action = match self.action_iterator.next() {
            None => return None,
            Some(action) => action,
        };

        match action {
            ActionType::Size => Some(Action::Size),
            ActionType::Contains => self.contains_action_iterator.next(),
            ActionType::Take => self.take_action_iterator.next(),
            ActionType::TakeMultiple => {
                self.take_multiple_action_iterator.next()
            }
            ActionType::Push => self.push_action_iterator.next(),
            ActionType::Pop => self.pop_action_iterator.next(),
            ActionType::Flush => Some(Action::Flush),
            ActionType::Get => self.get_action_iterator.next(),
            ActionType::GetMut => self.get_mut_action_iterator.next(),
        }
    }
}
