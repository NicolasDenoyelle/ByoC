//! Definition of the set of actions that can be performed on a container.

use byoc::utils::get::LifeTimeGuard;
use byoc::{BuildingBlock, Get, GetMut};

pub enum Action<K, V> {
    Size,
    Contains(K),
    Take(K),
    TakeMultiple(Vec<K>),
    Push(Vec<(K, V)>),
    Pop(usize),
    Flush,
    Get(K),
    GetMut(K),
}

#[derive(Clone, Copy)]
pub enum ActionType {
    Size = 0,
    Contains = 1,
    Take = 2,
    TakeMultiple = 3,
    Push = 4,
    Pop = 5,
    Flush = 6,
    Get = 7,
    GetMut = 8,
}

static ACTION_TYPES: [ActionType; 9] = [
    ActionType::Size,
    ActionType::Contains,
    ActionType::Take,
    ActionType::TakeMultiple,
    ActionType::Push,
    ActionType::Pop,
    ActionType::Flush,
    ActionType::Get,
    ActionType::GetMut,
];

impl From<usize> for ActionType {
    fn from(n: usize) -> Self {
        match n {
            0 => ActionType::Size,
            1 => ActionType::Contains,
            2 => ActionType::Take,
            3 => ActionType::TakeMultiple,
            4 => ActionType::Push,
            5 => ActionType::Pop,
            6 => ActionType::Flush,
            7 => ActionType::Get,
            8 => ActionType::GetMut,
            n => panic!("Invalid ActionType initializer: {}", n),
        }
    }
}

pub enum ActionResult<'a, K, V, F, S, E> {
    SizeResult(usize),
    ContainsResult(bool),
    TakeResult(Option<(K, V)>),
    TakeMultipleResult(Vec<(K, V)>),
    PushResult(Vec<(K, V)>),
    PopResult(Vec<(K, V)>),
    FlushResult(F),
    GetResult(Option<LifeTimeGuard<'a, S>>),
    GetMutResult(Option<LifeTimeGuard<'a, E>>),
}

impl<K, V> Action<K, V> {
    pub fn run<'a, F, S, E, B>(
        self,
        container: &'a mut B,
    ) -> ActionResult<'a, K, V, F, S, E>
    where
        F: Iterator<Item = (K, V)>,
        B: BuildingBlock<K, V, FlushIterator = F>
            + Get<K, V, Target = S>
            + GetMut<K, V, Target = E>,
    {
        match self {
            Self::Size => ActionResult::SizeResult(container.size()),
            Self::Contains(key) => {
                ActionResult::ContainsResult(container.contains(&key))
            }
            Self::Take(key) => {
                ActionResult::TakeResult(container.take(&key))
            }
            Self::TakeMultiple(mut keys) => {
                ActionResult::TakeMultipleResult(
                    container.take_multiple(&mut keys),
                )
            }
            Self::Push(elements) => {
                ActionResult::PushResult(container.push(elements))
            }
            Self::Pop(size) => {
                ActionResult::PopResult(container.pop(size))
            }
            Self::Flush => ActionResult::FlushResult(container.flush()),
            Self::Get(key) => ActionResult::GetResult(container.get(&key)),
            Self::GetMut(key) => {
                ActionResult::GetMutResult(container.get_mut(&key))
            }
        }
    }
}

pub(crate) mod random;
pub(crate) mod single;
