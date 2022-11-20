use crate::BuildingBlock;

use super::{
    ConfigError, ConfigInstance, ConfigWithTraits, GenericKey,
    GenericValue,
};

use crate::array::config::ArrayConfig;
use crate::associative::config::AssociativeConfig;
use crate::batch::config::BatchConfig;
use crate::btree::config::BTreeConfig;
use crate::builder::Build;
#[cfg(feature = "compression")]
use crate::compression::config::CompressedConfig;
use crate::exclusive::config::ExclusiveConfig;
// use crate::inclusive::config::InclusiveConfig;
use crate::profiler::config::ProfilerConfig;
use crate::sequential::config::SequentialConfig;
#[cfg(feature = "socket")]
use crate::socket::config::SocketClientConfig;
#[cfg(feature = "stream")]
use crate::stream::config::StreamConfig;

use serde::Serialize;
use toml;

/// Private entry point to build a container from a generic configuration.
#[derive(Clone, Serialize)]
pub(crate) struct GenericConfig {
    pub has_concurrent_trait: bool,
    pub has_ordered_trait: bool,
    toml_config: toml::Value,
}

impl GenericConfig {
    /// Attempt to build a specific building block config from a toml object.
    fn into_config<C: ConfigInstance>(
        v: &toml::Value,
    ) -> Result<C, ConfigError> {
        C::from_toml(v)
    }

    fn from_config<C: ConfigWithTraits + ConfigInstance>(
        v: toml::Value,
    ) -> Result<GenericConfig, ConfigError> {
        let toml_value = v.clone();
        C::from_toml(&v).map(move |cfg| GenericConfig {
            has_concurrent_trait: cfg.is_concurrent(),
            has_ordered_trait: cfg.is_ordered(),
            toml_config: toml_value,
        })
    }
}

impl ConfigInstance for GenericConfig {
    /// Build a container from a toml value object representing a configuration.
    /// This function checks that:
    /// * The toml configuration is a toml `Table`,
    /// * The toml configuration contains an "id" field
    /// * The value of the "id" field is a supported value.
    /// * The target configuration identified by "id" is valid.
    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        // Check toml value is a table.
        let table = match &value {
            toml::Value::Table(t) => t,
            _ => {
                return Err(ConfigError::ConfigFormatError(String::from(
                    "Building Block configuration must be a toml table.",
                )))
            }
        };

        // Check config contain an 'id' field.
        let id = match table.get("id") {
            None => {
                return Err(ConfigError::ConfigFormatError(String::from(
                    "Configuration must have an 'id' field.",
                )))
            }
            Some(s) => match s.as_str() {
                Some(s) => String::from(s),
                None => {
                    return Err(ConfigError::ConfigFormatError(
                        String::from("Invalid id type, must be a string."),
                    ))
                }
            },
        };

        let value = toml::value::Value::try_from(table).unwrap();

        // Check id field is a valid id and if it is, try to build the
        // associated config.
        match id.as_str() {
            "ArrayConfig" => Self::from_config::<ArrayConfig>(value),
            "AssociativeConfig" => {
                Self::from_config::<AssociativeConfig>(value)
            }
            "BatchConfig" => Self::from_config::<BatchConfig>(value),
            "BTreeConfig" => Self::from_config::<BTreeConfig>(value),
            #[cfg(feature = "compression")]
            "CompressedConfig" => {
                Self::from_config::<CompressedConfig>(value)
            }
            "ExclusiveConfig" => {
                Self::from_config::<ExclusiveConfig>(value)
            }
            // "InclusiveConfig" => {
            //     Self::from_config::<InclusiveConfig>(value)
            // }
            "ProfilerConfig" => Self::from_config::<ProfilerConfig>(value),
            "SequentialConfig" => {
                Self::from_config::<SequentialConfig>(value)
            }
            #[cfg(feature = "socket")]
            "SocketClientConfig" => {
                Self::from_config::<SocketClientConfig>(value)
            }
            #[cfg(feature = "stream")]
            "StreamConfig" => Self::from_config::<StreamConfig>(value),
            unknown => Err(ConfigError::ConfigFormatError(format!(
                "Invalid container configuration type: {}",
                unknown
            ))),
        }
    }
}

impl<'a, K, V> Build<Box<dyn BuildingBlock<'a, K, V> + 'a>>
    for GenericConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    /// Build the generic config object into an actual container.
    /// At this point we can assume that the checks from `from_toml()`
    /// method have passed. So we can build the configuration.
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        let id = self
            .toml_config
            .as_table()
            .unwrap()
            .get("id")
            .unwrap()
            .as_str()
            .unwrap();

        match id {
            "ArrayConfig" => {
                Self::into_config::<ArrayConfig>(&self.toml_config)
                    .unwrap()
                    .build()
            }
            "AssociativeConfig" => {
                Self::into_config::<AssociativeConfig>(&self.toml_config)
                    .unwrap()
                    .build()
            }
            "BatchConfig" => {
                Self::into_config::<BatchConfig>(&self.toml_config)
                    .unwrap()
                    .build()
            }
            "BTreeConfig" => {
                Self::into_config::<BTreeConfig>(&self.toml_config)
                    .unwrap()
                    .build()
            }
            #[cfg(feature = "compression")]
            "CompressedConfig" => {
                Self::into_config::<CompressedConfig>(&self.toml_config)
                    .unwrap()
                    .build()
            }
            "ExclusiveConfig" => {
                Self::into_config::<ExclusiveConfig>(&self.toml_config)
                    .unwrap()
                    .build()
            }
            // "InclusiveConfig" => {
            //     Self::into_config::<InclusiveConfig>(&self.toml_config)
            //         .unwrap()
            //         .build()
            // }
            "ProfilerConfig" => {
                Self::into_config::<ProfilerConfig>(&self.toml_config)
                    .unwrap()
                    .build()
            }
            "SequentialConfig" => {
                Self::into_config::<SequentialConfig>(&self.toml_config)
                    .unwrap()
                    .build()
            }
            #[cfg(feature = "socket")]
            "SocketClientConfig" => {
                Self::into_config::<SocketClientConfig>(&self.toml_config)
                    .unwrap()
                    .build()
                    .unwrap()
            }
            #[cfg(feature = "stream")]
            "StreamConfig" => {
                Self::into_config::<StreamConfig>(&self.toml_config)
                    .unwrap()
                    .build()
            }
            unknown => {
                panic!("Invalid container configuration type: {}", unknown)
            }
        }
    }
}

impl ConfigWithTraits for GenericConfig {}
