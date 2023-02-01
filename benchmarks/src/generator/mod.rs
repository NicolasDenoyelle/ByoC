//! Utilities to generate keys, values and actions.

pub trait KeyGenerator: IntoIterator<Item = Self::KeyType> {
    type KeyType;
}
impl<I: IntoIterator> KeyGenerator for I {
    type KeyType = I::Item;
}

pub trait ValueGenerator: IntoIterator<Item = Self::ValueType> {
    type ValueType;
}

impl<I: IntoIterator> ValueGenerator for I {
    type ValueType = I::Item;
}

pub trait KeyValuePairGenerator:
    IntoIterator<Item = (Self::KeyType, Self::ValueType)>
{
    type KeyType;
    type ValueType;

    fn zip<
        K: KeyGenerator<KeyType = Self::KeyType>,
        V: ValueGenerator<ValueType = Self::ValueType>,
    >(
        key_generator: K,
        value_generator: V,
    ) -> zip::ZipGenerator<K, V> {
        zip::ZipGenerator {
            key_generator,
            value_generator,
        }
    }
}

impl<K, V, I: IntoIterator<Item = (K, V)>> KeyValuePairGenerator for I {
    type KeyType = K;
    type ValueType = V;
}

use crate::action::Action;
pub trait ActionGenerator:
    IntoIterator<Item = Action<Self::KeyType, Self::ValueType>>
{
    type KeyType;
    type ValueType;
}

impl<K, V, IntoIter: IntoIterator<Item = Action<K, V>>> ActionGenerator
    for IntoIter
{
    type KeyType = K;
    type ValueType = V;
}

mod default;
pub use default::DefaultValueGenerator;

mod step;
pub use step::StepGenerator;

mod interleave;
pub use interleave::InterleaveGenerator;

mod random;
pub use random::{
    RandomBinomialmGenerator, RandomHypergeometricGenerator,
    RandomUniformGenerator,
};

mod zip;
pub use zip::ZipGenerator;

pub use crate::action::random::RandomActionGenerator;
pub mod action {
    pub use crate::action::single::{
        ContainsGenerator, FlushGenerator, GetGenerator, GetMutGenerator,
        PopGenerator, PushGenerator, SizeGenerator, TakeGenerator,
        TakeMultipleGenerator,
    };
}
