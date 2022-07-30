use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Serialize};
use std::cmp::Ord;
use std::hash::Hash;
use toml;

/// Key trait bound for keys of containers built from a configuration.
///
/// This trait bound compounds the keys trait bounds from all containers
/// implementations that can be built from a configuration.
/// This trait is required on keys that are used with a container built from a
/// configuration. The requirement for this trait bound comes from the fact
/// that the minimum requirement for keys to work with a container built from a
/// configuration file cannot be known at compiler time and cannot be adjusted
/// at runtime. Therefore, to ensure in advance that a key will fit with the
/// requirements of a container, the key has to satisfy at least all of the
/// containers keys trait bounds.
pub trait GenericKey:
    Ord + Copy + Hash + Serialize + DeserializeOwned
{
}
impl<T: Ord + Copy + Hash + Serialize + DeserializeOwned> GenericKey
    for T
{
}

/// Value trait bound for values of containers built from a configuration.
///
/// This trait bound compounds the values trait bounds from all containers
/// implementations that can be built from a configuration.
/// This trait is required on values that are used with a container built from a
/// configuration. The requirement for this trait bound comes from the fact
/// that the minimum requirement for values to work with a container built from
/// a configuration file cannot be known at compiler time and cannot be adjusted
/// at runtime. Therefore, to ensure in advance that a value will fit with the
/// requirements of a container, the value has to satisfy at least all of the
/// containers keys trait bounds.
pub trait GenericValue: Ord + Serialize + DeserializeOwned {}
impl<T: Ord + Serialize + DeserializeOwned> GenericValue for T {}

/// Trait used to instantiate a configuration object from a toml configuration
/// and build a [`BuildingBlock'](../trait.BuildingBlock.html) container.
///
/// The resulting configuration object obtained with the
/// [`from_toml()`](trait.BuildingBlockConfig.html#tymethod.from_toml) method
/// can later be used to create a
/// [`BuildingBlock`](../trait.BuildingBlock.html) after checking that the
/// parsed configuration was valid.
///
/// Implementers of this trait will need to manually update the
/// [`BuilderConfig`] implementation to be able to build the trait implementer
/// configuration.
pub trait BuildingBlockConfig {
    /// Method to create this configuration trait from a parsed toml
    /// [`toml::Value`].
    ///
    /// Implementers of this method can expect that input `Value` object will
    /// match a [`toml::value::Table`] and contain an `id` field.
    /// This is enforced by the [`BuilderConfig`] when building a
    /// configuration from a toml string.
    ///
    /// This method returns either Self on success to parse input toml into
    /// a valid container or an Error describing what went wrong.
    fn from_toml(value: toml::Value) -> Result<Self, ConfigError>
    where
        Self: Sized;

    /// Build the corresponding configuration object into a container.
    fn build<'a, K: 'a + GenericKey, V: 'a + GenericValue>(
        self,
    ) -> Box<dyn BuildingBlock<'a, K, V> + 'a>;
}

mod config;
pub use config::BuilderConfig;
use config::GenericConfig;

mod array;
pub use array::ArrayConfig;

mod associative;
pub use associative::AssociativeConfig;

mod batch;
pub use batch::BatchConfig;

mod btree;
pub use btree::BTreeConfig;

#[cfg(feature = "compression")]
mod compression;
#[cfg(feature = "compression")]
pub use compression::CompressorConfig;

mod multilevel;
pub use multilevel::MultilevelConfig;

mod policy;
pub use policy::{PolicyConfig, PolicyKind};

mod profiler;
pub use profiler::ProfilerConfig;

mod sequential;
pub use sequential::SequentialConfig;

#[cfg(feature = "stream")]
mod stream;
#[cfg(feature = "stream")]
pub use stream::StreamConfig;

mod error;
pub use error::ConfigError;
