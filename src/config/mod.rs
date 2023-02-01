//! Module to instantiate a cache architecture from a configuration file.
//!
//! There are three ways to build a cache architecture:
//! 1. From components constructors,
//! 2. From a [builder pattern](../builder/index.html),
//! 3. From a [`toml`](https://toml.io/en/) configuration.
//!
//! This module provides the tools to implement the 3. method.
//!
//! The [`ConfigBuilder`] structure is the entry point to create a
//! [`BuildingBlock`](../trait.BuildingBlock.html) instance in the form of a
//! [`DynBuildingBlock`](../struct.DynBuildingBlock.html)
//! structure from a configuration string or file.
//! [`ConfigBuilder`] structure documentation provides examples and details on
//! the quirks and the cruxes of configurations containers built from a
//! configuration.
//!
//! The [`configs`](configs/index.html) module provides a collection of
//! containers configurations and their toml format.
//!
//! Because it may be hard to understand or write complex configurations,
//! we implemented a way to generate them from
//! [container builder patterns](../builder/index.html) via the [`IntoConfig`]
//! trait.
//!
//! [`DynBuildingBlock`](../struct.DynBuildingBlock.html) obtained from
//! [`ConfigBuilder`] will only accept
//! keys implementing the [`GenericKey`] trait and values implementing the
//! [`GenericValue`] trait. Since it is not possible to know at compile time
//! the type of container built from a configuration, it is necessary that keys
//! and values types are compatible with all possible containers that can be
//! generated from a configuration.
//!
//! If the [`DynBuildingBlock`](../struct.DynBuildingBlock.html) obtained from
//! a configuration implements
//! the [`Concurrent`](../trait.Concurrent.html) trait, then it should be
//! detected at runtime and the former struct can be turned respectively into a
//! [`DynConcurrent`](../struct.DynConcurrent.html)
//! [`BuildingBlock`](../trait.BuildingBlock.html)

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

/// Trait to create configuration instances from a `&str`, a [`std::fs::File`],
/// or a [`toml::Value`].
///
/// [`from_toml()`](trait.ConfigInstance.html#method.from_toml) is the only
/// method that requires an implementation.
pub trait ConfigInstance: Serialize
where
    Self: Sized,
{
    /// Get the [`ConfigInstance`] implementation id.
    ///
    /// The default behavior is to return `std::any::type_name::<Self>()`.
    fn id() -> &'static str {
        std::any::type_name::<Self>()
    }

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

    /// Convert a configuration object back to a toml string.
    fn to_toml_string(&self) -> String {
        toml::ser::to_string(self).unwrap()
    }

    /// Convert a configuration object back to a toml string.
    fn to_toml_string_pretty(&self) -> String {
        toml::ser::to_string_pretty(self).unwrap()
    }

    /// Method to create this configuration trait from a parsed
    /// `&str`.
    ///
    /// The string is representing a [`toml::Value`] and is parsed as such.
    fn from_string(s: &str) -> Result<Self, ConfigError> {
        match toml::from_str::<toml::Value>(s) {
            Ok(value) => ConfigInstance::from_toml(&value),
            Err(e) => Err(ConfigError::TomlFormatError(e)),
        }
    }

    /// Method to create this configuration trait from a parsed
    /// [`std::fs::File`].
    ///
    /// The file is read into a string representing a [`toml::Value`] and
    /// parsed as such.
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

    fn build<'a, K: 'a + GenericKey, V: 'a + GenericValue>(
        self,
    ) -> crate::DynBuildingBlock<'a, K, V>;

    /// Return whether this configuration represents a
    /// [`BuildingBlock`](../trait.BuildingBlock.html) that implements the
    /// [`Concurrent`](../trait.Concurrent.html) trait.
    fn is_concurrent(&self) -> bool {
        false
    }
}

/// Convert an object into [`ConfigInstance`].
///
/// The goal of this trait is to ease the construction of custom
/// cache containers configuration by translating from easy to build
/// [cache builder](../builder/index.html) to a matching configuration.
///
/// Understanding [configuration syntax](struct.ConfigBuilder.html) for
/// complex containers can be tedious. This trait allows to generate it from
/// an easier to understand container [builder pattern](../builder/index.html).
///
/// This trait is intended to be implemented by
/// [cache builders](../builder/index.html) mainly but could be implemented by
/// other containers as well.
pub trait IntoConfig<C: ConfigInstance> {
    fn as_config(&self) -> C;
}

#[allow(clippy::module_inception)]
mod config_builder;
pub use config_builder::ConfigBuilder;
mod error;
pub use error::ConfigError;
mod generic_config;
pub(crate) use generic_config::GenericConfig;

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
    #[cfg(feature = "socket")]
    pub use crate::socket::config::{
        SocketClientConfig, SocketServerConfig,
    };
    #[cfg(feature = "stream")]
    pub use crate::stream::config::StreamConfig;
}

#[cfg(test)]
pub(crate) mod tests;
