use super::Action;
use crate::generator::{KeyGenerator, KeyValuePairGenerator};
use crate::utils::iter::{VecGenerator, VecIterator};
use std::marker::PhantomData;

macro_rules! declare_noarg_action_generator {
    ($name:ident, $action:expr) => {
        pub struct $name<K, V> {
            unused: PhantomData<(K, V)>,
        }

        impl<K, V> $name<K, V> {
            pub fn new() -> Self {
                Self {
                    unused: PhantomData,
                }
            }
        }

        impl<K, V> Iterator for $name<K, V> {
            type Item = Action<K, V>;
            fn next(&mut self) -> Option<Self::Item> {
                Some($action)
            }
        }
    };
}

declare_noarg_action_generator!(SizeGenerator, Action::Size);
declare_noarg_action_generator!(FlushGenerator, Action::Flush);

macro_rules! declare_arg_action_generator {
    ($name:ident, $action:expr) => {
        pub struct $name<K, V, G> {
            generator: G,
            unused: PhantomData<(K, V)>,
        }

        impl<K, V, G> $name<K, V, G> {
            pub fn new(generator: G) -> Self {
                Self {
                    generator,
                    unused: PhantomData,
                }
            }
        }

        impl<K, V, G> IntoIterator for $name<K, V, G>
        where
            G: IntoIterator<Item = K>,
        {
            type Item = Action<K, V>;
            type IntoIter =
                std::iter::Map<G::IntoIter, fn(G::Item) -> Self::Item>;
            fn into_iter(self) -> Self::IntoIter {
                self.generator.into_iter().map(|k| $action(k))
            }
        }
    };
}

declare_arg_action_generator!(TakeGenerator, Action::Take);
declare_arg_action_generator!(ContainsGenerator, Action::Contains);
declare_arg_action_generator!(GetGenerator, Action::Get);
declare_arg_action_generator!(GetMutGenerator, Action::GetMut);

pub struct PopGenerator<K, V, G> {
    usize_generator: G,
    unused: PhantomData<(K, V)>,
}

impl<K, V, G> PopGenerator<K, V, G> {
    pub fn new(usize_generator: G) -> Self {
        Self {
            usize_generator,
            unused: PhantomData,
        }
    }
}

impl<K, V, G> IntoIterator for PopGenerator<K, V, G>
where
    G: IntoIterator<Item = usize>,
{
    type Item = Action<K, V>;
    type IntoIter = std::iter::Map<G::IntoIter, fn(G::Item) -> Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.usize_generator
            .into_iter()
            .map(|size| Action::Pop(size))
    }
}

pub struct PushGenerator<G, R> {
    vec_generator: VecGenerator<G, R>,
}

impl<G, R> PushGenerator<G, R> {
    pub fn new(
        key_value_pair_generator: G,
        vec_size_generator: R,
    ) -> Self {
        Self {
            vec_generator: VecGenerator::new(
                key_value_pair_generator,
                vec_size_generator,
            ),
        }
    }
}

impl<G, R> IntoIterator for PushGenerator<G, R>
where
    G: KeyValuePairGenerator,
    R: IntoIterator<Item = usize>,
{
    type Item = Action<G::KeyType, G::ValueType>;
    type IntoIter = std::iter::Map<
        VecIterator<G::IntoIter, R::IntoIter>,
        fn(Vec<G::Item>) -> Self::Item,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.vec_generator.into_iter().map(|vec| Action::Push(vec))
    }
}

pub struct TakeMultipleGenerator<G, R, V> {
    vec_generator: VecGenerator<G, R>,
    unused: PhantomData<V>,
}

impl<G, R, V> TakeMultipleGenerator<G, R, V> {
    pub fn new(
        key_value_pair_generator: G,
        vec_size_generator: R,
    ) -> Self {
        Self {
            vec_generator: VecGenerator::new(
                key_value_pair_generator,
                vec_size_generator,
            ),
            unused: PhantomData,
        }
    }
}

impl<G, R, V> IntoIterator for TakeMultipleGenerator<G, R, V>
where
    G: KeyGenerator,
    R: IntoIterator<Item = usize>,
{
    type Item = Action<G::KeyType, V>;
    type IntoIter = std::iter::Map<
        VecIterator<G::IntoIter, R::IntoIter>,
        fn(Vec<G::Item>) -> Self::Item,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.vec_generator
            .into_iter()
            .map(|vec| Action::TakeMultiple(vec))
    }
}
// struct RandomActionGenerator<K, V>
