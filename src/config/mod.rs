//! Module to instantiate a cache architecture from a configuration file.
//!
//! Configuration file/strings are a way to instantiate containers.
//! They describe containers using the [`toml`](https://toml.io/en/)
//! format.
//! The [`Builder`] structure is the entry point to create a
//! container
//! instance from a configuration string or file.
//! For instance, the container described in the
//! [`builder`](../builder/index.html) module can be built as follow:
//! ```
//! use byoc::BuildingBlock;
//! use byoc::builder::Build;
//! use byoc::config::{ConfigBuilder, DynBuildingBlock};
//!
//! let config_str = format!("
//! id='SequentialConfig'
//! policy.kind = 'Lru'
//! [container]
//! id='ExclusiveConfig'
//! [container.front]
//! id='ArrayConfig'
//! capacity=10000
//! [container.back]
//! id='ArrayConfig'
//! capacity=1000000
//! ");
//! let mut container: DynBuildingBlock<u64, u64> =
//!            ConfigBuilder::from_string(config_str.as_str()).unwrap().build();
//! container.push(vec![(1,2)]);
//! ```
//!
//! See the [`Builder`] structure for more details on possible configurations.
//! See the [`configs`](configs/index.html) module for the collection of
//! containers configuration format.

use crate::builder::Build;
use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Serialize};
use std::cmp::Ord;
use std::hash::Hash;
use std::io::Read;
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
    Ord + Copy + Hash + Serialize + DeserializeOwned + Clone
{
}
impl<T: Ord + Copy + Hash + Serialize + DeserializeOwned + Clone>
    GenericKey for T
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
pub trait GenericValue:
    Ord + Serialize + DeserializeOwned + Clone
{
}
impl<T: Ord + Serialize + DeserializeOwned + Clone> GenericValue for T {}

pub(crate) trait ConfigInstance
where
    Self: Sized,
{
    /// Method to create this configuration trait from a parsed toml
    /// [`toml::Value`].
    ///
    /// Implementers of this method can expect that input `Value` object will
    /// match a [`toml::value::Table`] and contain an `id` field.
    /// This is enforced by the [`ConfigBuilder`] when building a
    /// configuration from a toml string.
    ///
    /// This method returns either Self on success to parse input toml into
    /// a valid container or an Error describing what went wrong.
    fn from_toml(toml_value: &toml::Value) -> Result<Self, ConfigError>;

    fn from_file<P: AsRef<std::path::Path> + std::fmt::Debug>(
        path: P,
    ) -> Result<Self, ConfigError> {
        let mut file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => return Err(ConfigError::IOError(e)),
        };
        let mut s = String::new();

        if let Err(e) = file.read_to_string(&mut s) {
            return Err(ConfigError::IOError(e));
        }
        ConfigInstance::from_string(s.as_str())
    }

    fn from_string(s: &str) -> Result<Self, ConfigError> {
        match toml::from_str::<toml::Value>(s) {
            Ok(value) => ConfigInstance::from_toml(&value),
            Err(e) => Err(ConfigError::TomlFormatError(e)),
        }
    }
}

pub(crate) trait ConfigWithTraits {
    /// Return whether this configuration represents a
    /// [`BuildingBlock`](../trait.BuildingBlock.html) that implements the
    /// [`Concurrent`](../trait.Concurrent.html) trait.
    fn is_concurrent(&self) -> bool {
        false
    }

    /// Return whether this configuration represents a
    /// [`BuildingBlock`](../trait.BuildingBlock.html) that implements the
    /// [`Ordered`](../policy/trait.Ordered.html) trait.
    fn is_ordered(&self) -> bool {
        false
    }
}

/// Trait used to instantiate a configuration object from a toml configuration
/// and build a `BuildingBlock` container.
///
/// The resulting configuration object obtained with the
/// [`from_toml()`](trait.BuildingBlockConfig.html#tymethod.from_toml) method
/// can later be used to create a
/// [`BuildingBlock`](../trait.BuildingBlock.html) after checking that the
/// parsed configuration was valid.
///
/// Implementers of this trait will need to manually update the
/// [`ConfigBuilder`] implementation to be able to build the trait implementer
/// configuration.
pub(crate) trait BuildingBlockConfig<'a, K, V>:
    ConfigInstance
    + ConfigWithTraits
    + Build<Box<dyn BuildingBlock<'a, K, V> + 'a>>
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
}

impl<'a, K, V, T> BuildingBlockConfig<'a, K, V> for T
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
    T: ConfigInstance
        + ConfigWithTraits
        + Build<Box<dyn BuildingBlock<'a, K, V> + 'a>>,
{
}

#[allow(clippy::module_inception)]
mod builder;
pub use builder::ConfigBuilder;
pub(crate) use builder::GenericConfig;
mod error;
pub use error::ConfigError;
mod dyn_traits;
pub use dyn_traits::{DynBuildingBlock, DynConcurrent, DynOrdered};

/// The collection of available configurations.
pub mod configs {
    pub use crate::array::config::ArrayConfig;
    pub use crate::associative::config::AssociativeConfig;
    pub use crate::batch::config::BatchConfig;
    pub use crate::btree::config::BTreeConfig;
    #[cfg(feature = "compression")]
    pub use crate::compression::config::CompressedConfig;
    pub use crate::exclusive::config::ExclusiveConfig;
    // pub use crate::inclusive::config::InclusiveConfig;
    pub use crate::profiler::config::ProfilerConfig;
    pub use crate::sequential::config::SequentialConfig;
    #[cfg(feature = "stream")]
    pub use crate::stream::config::StreamConfig;
}
